#!/usr/bin/env python3
"""Colonna sonora procedurale: 7 tracce chiptune in assets/musica/*.ogg.

Stessa filosofia di gen_sprites.py e gen_audio.py: la fonte è testuale e
deterministica (seed fissi, niente dipendenze Python), l'unico strumento
esterno è ffmpeg per la conversione finale WAV → OGG (i WAV intermedi
vivono in una cartella temporanea e non toccano il repo).

È una colonna sonora, non sette brani slegati: tre motivi ricorrono
variati — il tema del CANTIERE (sale a gradini, ottimista), il tema
AURORA (scende, malinconico: è la Vecchia), il tema dell'ATTESA (quinte
aperte sospese). La mappa tracce → blocchi di campagna è nel cablaggio
(src/musica.rs); qui i nomi file sono il contratto:

    menu, cantiere, termica, reliquie, officina, vigilia, finale

Tecnica: rendering CIRCOLARE — ogni nota (coda di rilascio compresa) e
l'eco scrivono modulo la lunghezza del loop, quindi il punto di giunzione
non ha click per costruzione. Mix normalizzato a picco 0.5 su tutte le
tracce.
"""

import math
import os
import struct
import subprocess
import sys
import tempfile
import wave as wavemod

SR = 22050
ATTACCO = int(0.004 * SR)

# ---------------------------------------------------------------- scale

SCALE = {
    "maj": [0, 2, 4, 5, 7, 9, 11],
    "min": [0, 2, 3, 5, 7, 8, 10],
    "mixo": [0, 2, 4, 5, 7, 9, 10],
    "lyd": [0, 2, 4, 6, 7, 9, 11],
}


def freq(midi):
    return 440.0 * 2 ** ((midi - 69) / 12)


def deg(root, scala, grado):
    """Grado di scala (anche negativo o oltre l'ottava) → nota MIDI."""
    ottava, idx = divmod(grado, 7)
    return root + 12 * ottava + scala[idx]


# ---------------------------------------------------------------- motivi

# tema del CANTIERE: sale a gradini, ottimista (beat, durata, grado)
TEMA_CANTIERE = [(0, 0.5, 0), (0.5, 0.5, 1), (1, 0.5, 2), (1.5, 0.5, 4),
                 (2, 1.0, 5), (3, 1.0, 4)]
# risposta del cantiere: ridiscende e si posa
RISPOSTA_CANTIERE = [(0, 0.5, 4), (0.5, 0.5, 2), (1, 1.0, 1), (2, 2.0, 0)]
# tema AURORA: discende, malinconico
TEMA_AURORA = [(0, 1.5, 7), (1.5, 0.5, 5), (2, 1.0, 4), (3, 1.0, 0)]
# AURORA rovesciata: la risoluzione (usata in vigilia e finale)
AURORA_RISOLTA = [(0, 1.0, 0), (1, 1.0, 2), (2, 2.0, 4), (4, 4.0, 7)]
# tema dell'ATTESA: quinte aperte, sospese
TEMA_ATTESA = [(0, 2.0, 0), (2, 2.0, 4)]


# ---------------------------------------------------------------- sintesi

def add_nota(buf, bpm, beat, durata, midi, onda="tri", vol=0.15, duty=0.5):
    """Una nota nel buffer circolare: quadra o triangolare, inviluppo ADS
    corto + rilascio, indici modulo len(buf) — il loop resta pulito."""
    n = len(buf)
    spb = 60.0 / bpm * SR
    inizio = int(beat * spb)
    dur = max(1, int(durata * spb))
    rilascio = int(0.04 * SR)
    f = freq(midi)
    fase = 0.0
    for i in range(dur + rilascio):
        if i < ATTACCO:
            env = i / ATTACCO
        elif i < dur:
            env = 1.0 - 0.18 * (i / dur)
        else:
            env = (1.0 - (i - dur) / rilascio) * 0.82
        fase += f / SR
        p = fase % 1.0
        if onda == "sq":
            s = 1.0 if p < duty else -1.0
        else:  # triangolare
            s = 4.0 * abs(p - 0.5) - 1.0
        buf[(inizio + i) % n] += s * env * vol


def _lcg(seed):
    s = (seed & 0x7FFFFFFF) or 1
    while True:
        s = (1103515245 * s + 12345) & 0x7FFFFFFF
        yield s / 0x40000000 - 1.0


def add_rumore(buf, bpm, beat, durata_s, vol, seed, brillante=False,
               attacco_s=0.001):
    """Colpo di rumore (percussioni, swell). Seed derivato da posizione:
    deterministico qualunque sia l'ordine di rendering."""
    n = len(buf)
    inizio = int(beat * (60.0 / bpm) * SR)
    dur = int(durata_s * SR)
    att = max(1, int(attacco_s * SR))
    gen = _lcg(seed * 7919 + int(beat * 1000))
    prec = 0.0
    for i in range(dur):
        r = next(gen)
        s = (r - prec) if brillante else r
        prec = r
        env = min(1.0, i / att) * (1.0 - i / dur) ** 2
        buf[(inizio + i) % n] += s * env * vol


def add_cassa(buf, bpm, beat, vol=0.11):
    """Cassa: quadra bassa con glissando 70→42 Hz, 90 ms."""
    n = len(buf)
    inizio = int(beat * (60.0 / bpm) * SR)
    dur = int(0.09 * SR)
    fase = 0.0
    for i in range(dur):
        f = 70.0 * (42.0 / 70.0) ** (i / dur)
        fase += f / SR
        s = 1.0 if (fase % 1.0) < 0.5 else -1.0
        env = (1.0 - i / dur) ** 1.5
        buf[(inizio + i) % n] += s * env * vol


def eco(buf, bpm, beat_ritardo, gain):
    """Eco a due riprese in lettura circolare sul segnale asciutto:
    convoluzione circolare, quindi loop-pulita per costruzione."""
    n = len(buf)
    d = int(beat_ritardo * (60.0 / bpm) * SR)
    asciutto = buf[:]
    for i in range(n):
        buf[i] = (asciutto[i]
                  + gain * asciutto[(i - d) % n]
                  + gain * gain * asciutto[(i - 2 * d) % n])


def normalizza(buf, picco=0.5):
    m = max(abs(x) for x in buf) or 1.0
    k = picco / m
    for i in range(len(buf)):
        buf[i] *= k


# ------------------------------------------------------------ strumenti

def pad(buf, bpm, scala, root, grado, beat, durata, vol=0.11):
    """Fondamentale + quinta, lunghe: il tappeto di quasi tutte le tracce."""
    base = deg(root, scala, grado)
    add_nota(buf, bpm, beat, durata, base, "tri", vol)
    add_nota(buf, bpm, beat, durata, base + 7, "tri", vol * 0.8)


def melodia(buf, bpm, scala, root, grado_accordo, beat, motivo,
            vol=0.15, onda="tri", ottava=12, stira=1.0):
    for off, durata, g in motivo:
        add_nota(buf, bpm, beat + off * stira, durata * stira,
                 deg(root, scala, grado_accordo + g) + ottava, onda, vol)


def basso(buf, bpm, scala, root, grado, beat, schema, vol=0.16, duty=0.3):
    """Schema = [(beat_rel, durata, offset_semitoni)] sulla fondamentale."""
    base = deg(root, scala, grado)
    for off, durata, semi in schema:
        add_nota(buf, bpm, beat + off, durata, base + semi, "sq", vol, duty)


def arpeggio(buf, bpm, scala, root, grado, beat, battiti, passo=0.5,
             vol=0.08, ottava=12):
    gradi = [0, 2, 4, 7]
    i = 0
    t = 0.0
    while t < battiti - 1e-9:
        add_nota(buf, bpm, beat + t, passo * 0.9,
                 deg(root, scala, grado + gradi[i % 4]) + ottava, "tri", vol)
        i += 1
        t += passo


def stab(buf, bpm, scala, root, grado, beat, vol=0.07):
    for g in (0, 2, 4):
        add_nota(buf, bpm, beat, 0.4, deg(root, scala, grado + g) + 12,
                 "sq", vol, 0.4)


# ---------------------------------------------------------------- tracce

def comp_menu(buf, bpm):
    """Il titolo: la stazione vista da fuori. Quinte sospese, un frammento
    di Aurora a metà, tanta eco."""
    sc, root = SCALE["min"], 45  # La minore, registro basso
    prog = [0, 5, 2, 6]
    for bar in range(16):
        pad(buf, bpm, sc, root, prog[bar // 4], bar * 4, 4, 0.12)
    for bar in (0, 4, 8, 12):
        melodia(buf, bpm, sc, root, prog[bar // 4], bar * 4, TEMA_ATTESA,
                0.14, "tri", 24)
    # il frammento della Vecchia, mezza velocità, quasi un ricordo
    melodia(buf, bpm, sc, root, 0, 40, TEMA_AURORA, 0.12, "tri", 24, stira=2.0)
    eco(buf, bpm, 0.75, 0.35)


def comp_cantiere(buf, bpm):
    """Blocchi 1-2: si costruisce. Maggiore, passo regolare, tema del
    cantiere in primo piano con la sua risposta."""
    sc, root = SCALE["maj"], 48  # Do maggiore
    prog_a, prog_b = [0, 5, 3, 4], [3, 4, 2, 5]
    schema_basso = [(0, 0.75, 0), (1, 0.75, 0), (2, 0.75, 7), (3, 0.75, 12)]
    for bar in range(32):
        sezione_b = 16 <= bar < 24
        grado = (prog_b if sezione_b else prog_a)[bar % 4]
        beat = bar * 4
        basso(buf, bpm, sc, root, grado, beat, schema_basso)
        if sezione_b and bar < 20:
            melodia(buf, bpm, sc, root, grado, beat, TEMA_AURORA, 0.15, "sq", 12)
        elif bar % 2 == 0:
            melodia(buf, bpm, sc, root, grado, beat, TEMA_CANTIERE, 0.16, "sq", 12)
        else:
            melodia(buf, bpm, sc, root, grado, beat, RISPOSTA_CANTIERE, 0.15, "sq", 12)
        if bar >= 4:
            arpeggio(buf, bpm, sc, root, grado, beat, 4, 0.5, 0.07)
        add_cassa(buf, bpm, beat, 0.10)
        add_cassa(buf, bpm, beat + 2, 0.09)
        for b in range(4):
            add_rumore(buf, bpm, beat + b + 0.5, 0.03, 0.05, 11, True)
    eco(buf, bpm, 0.5, 0.22)


def comp_termica(buf, bpm):
    """Blocchi 3-4: il calore sale. Minore, ostinato di basso, cantiere
    che si fa corto e teso."""
    sc, root = SCALE["min"], 52  # Mi minore
    prog = [0, 0, 5, 6]
    cantiere_teso = [(0, 0.5, 0), (0.5, 0.5, 1), (1, 0.5, 2), (1.5, 0.5, 4),
                     (2, 0.5, 2), (2.5, 1.5, 1)]
    for bar in range(32):
        grado = prog[bar % 4] if not 16 <= bar < 24 else [5, 6, 0, 0][bar % 4]
        beat = bar * 4
        for b in range(4):  # ostinato: ottave martellate
            basso(buf, bpm, sc, root, grado, beat + b,
                  [(0, 0.4, 0), (0.5, 0.4, 12)], 0.17, 0.25)
        if bar % 2 == 0:
            melodia(buf, bpm, sc, root, grado, beat, cantiere_teso, 0.15, "sq", 12)
        for b in range(4):
            add_cassa(buf, bpm, beat + b, 0.10)
            add_rumore(buf, bpm, beat + b + 0.5, 0.025, 0.04, 23, True)
        if bar >= 8:
            add_rumore(buf, bpm, beat + 2, 0.07, 0.06, 37)
    eco(buf, bpm, 0.375, 0.22)


def comp_reliquie(buf, bpm):
    """Blocchi 5-6: i detriti hanno un nome. Lenta, rada, il tema Aurora
    per esteso, silenzi che pesano."""
    sc, root = SCALE["min"], 57  # La minore, registro medio
    prog = [0, 5, 2, 6]
    for bar in range(20):
        grado = prog[(bar // 2) % 4] if bar < 16 else 0
        pad(buf, bpm, sc, root, grado, bar * 4, 4, 0.10)
    for bar in (1, 5, 9, 13):
        grado = prog[(bar // 2) % 4]
        melodia(buf, bpm, sc, root, grado, bar * 4, TEMA_AURORA,
                0.15, "tri", 24, stira=1.5)
    # respiro di rumore, come aria che sfiata da uno scafo vecchio
    add_rumore(buf, bpm, 64, 3.0, 0.03, 71, False, attacco_s=1.5)
    # l'ultima nota: la fondamentale, sola
    add_nota(buf, bpm, 76, 3.5, deg(root, sc, 0) + 12, "tri", 0.13)
    eco(buf, bpm, 1.0, 0.4)


def comp_officina(buf, bpm):
    """Blocchi 7-8: la verità è detta, si lavora. Mixolidio, groove
    sincopato, cantiere compresso che risponde colpo su colpo."""
    sc, root = SCALE["mixo"], 55  # Sol misolidio
    prog = [0, 6, 3, 0]
    schema_basso = [(0, 0.5, 0), (1, 0.5, 7), (1.5, 0.5, 0), (2.5, 0.5, 7),
                    (3, 0.5, 12)]
    for bar in range(32):
        grado = prog[bar % 4]
        beat = bar * 4
        basso(buf, bpm, sc, root, grado, beat, schema_basso, 0.17)
        if bar % 2 == 0:
            melodia(buf, bpm, sc, root, grado, beat, TEMA_CANTIERE,
                    0.16, "sq", 12, stira=0.5)
            melodia(buf, bpm, sc, root, grado, beat + 2, RISPOSTA_CANTIERE,
                    0.14, "sq", 12, stira=0.5)
        if bar % 4 >= 2:
            stab(buf, bpm, sc, root, grado, beat + 1.5)
            stab(buf, bpm, sc, root, grado, beat + 3.5)
        add_cassa(buf, bpm, beat, 0.11)
        add_cassa(buf, bpm, beat + 2, 0.10)
        add_rumore(buf, bpm, beat + 1, 0.08, 0.07, 41)
        add_rumore(buf, bpm, beat + 3, 0.08, 0.07, 43)
        for b in range(4):
            add_rumore(buf, bpm, beat + b + 0.5, 0.025, 0.04, 47, True)
    eco(buf, bpm, 0.5, 0.18)


def comp_vigilia(buf, bpm):
    """Blocco 9: la scelta di restare. Lidio caldo, il materiale del menu
    che si scioglie, l'Aurora rovesciata che finalmente sale."""
    sc, root = SCALE["lyd"], 60  # Do lidio
    prog = [0, 1, 4, 0]
    attesa_calda = [(0, 2.0, 0), (2, 1.5, 4), (3.5, 0.5, 5)]
    for bar in range(16):
        grado = prog[bar // 4]
        beat = bar * 4
        pad(buf, bpm, sc, root, grado, beat, 4, 0.11)
        arpeggio(buf, bpm, sc, root, grado, beat, 4, 1.0, 0.06)
        if bar % 4 == 1:
            melodia(buf, bpm, sc, root, grado, beat, attesa_calda, 0.13, "tri", 12)
    # la risoluzione: il tema della Vecchia, rovesciato, che sale
    melodia(buf, bpm, sc, root, 0, 48, AURORA_RISOLTA, 0.15, "tri", 12)
    eco(buf, bpm, 0.75, 0.35)


def comp_finale(buf, bpm):
    """Blocco 10: i due temi si incontrano. La traccia più ricca: cantiere
    e Aurora in maggiore, insieme, sopra tutto il resto."""
    sc, root = SCALE["maj"], 48  # Do maggiore
    prog_a, prog_b = [0, 5, 3, 4], [3, 4, 0, 5]
    schema_basso = [(0, 0.75, 0), (1, 0.75, 0), (2, 0.75, 7), (3, 0.75, 12)]
    for bar in range(28):
        beat = bar * 4
        if bar < 8:
            grado = prog_a[bar % 4]
        elif bar < 16:
            grado = prog_b[bar % 4]
        elif bar < 24:
            grado = prog_a[bar % 4]
        else:
            grado = [3, 4, 0, 0][bar % 4]
        basso(buf, bpm, sc, root, grado, beat, schema_basso)
        if bar < 8:
            if bar % 2 == 0:
                melodia(buf, bpm, sc, root, grado, beat, TEMA_CANTIERE, 0.16, "sq", 12)
        elif bar < 16:
            melodia(buf, bpm, sc, root, grado, beat, TEMA_AURORA, 0.15, "sq", 12)
        elif bar < 24:
            # i due temi insieme: cantiere sopra, Aurora sotto a mezza voce
            if bar % 2 == 0:
                melodia(buf, bpm, sc, root, grado, beat, TEMA_CANTIERE, 0.16, "sq", 24)
            melodia(buf, bpm, sc, root, grado, beat, TEMA_AURORA,
                    0.11, "tri", 0, stira=1.0)
        else:
            arpeggio(buf, bpm, sc, root, grado, beat, 4, 0.25, 0.08, 24)
        if bar >= 4:
            arpeggio(buf, bpm, sc, root, grado, beat, 4, 0.5, 0.06)
        if bar % 4 >= 2 and bar < 24:
            stab(buf, bpm, sc, root, grado, beat + 1.5)
        add_cassa(buf, bpm, beat, 0.10)
        add_cassa(buf, bpm, beat + 2, 0.09)
        add_rumore(buf, bpm, beat + 1, 0.08, 0.06, 53)
        add_rumore(buf, bpm, beat + 3, 0.08, 0.06, 59)
    # l'ultimo accordo, tenuto: la stazione che resta accesa
    for g in (0, 2, 4):
        add_nota(buf, bpm, 108, 4.0, deg(root, sc, g) + 12, "tri", 0.10)
    eco(buf, bpm, 0.5, 0.25)


# (nome, bpm, battiti totali, compositore) — durata = battiti·60/bpm
TRACCE = [
    ("menu", 60, 64, comp_menu),          # 64.0 s
    ("cantiere", 112, 128, comp_cantiere),  # 68.6 s
    ("termica", 126, 128, comp_termica),  # 61.0 s
    ("reliquie", 72, 80, comp_reliquie),  # 66.7 s
    ("officina", 118, 128, comp_officina),  # 65.1 s
    ("vigilia", 66, 64, comp_vigilia),    # 58.2 s
    ("finale", 100, 112, comp_finale),    # 67.2 s
]


def _tabella_crc_ogg():
    """CRC-32 di OGG: polinomio 0x04C11DB7, non riflesso, init e xorout 0."""
    tab = []
    for i in range(256):
        r = i << 24
        for _ in range(8):
            r = ((r << 1) ^ 0x04C11DB7 if r & 0x80000000 else r << 1) & 0xFFFFFFFF
        tab.append(r)
    return tab


_CRC_OGG = _tabella_crc_ogg()


def _crc_ogg(dati):
    r = 0
    for b in dati:
        r = ((r << 8) & 0xFFFFFFFF) ^ _CRC_OGG[((r >> 24) & 0xFF) ^ b]
    return r


def fissa_serial(percorso, serial=0x5AB05747):
    """ffmpeg estrae a caso il numero di serie del flusso OGG: due run
    identici darebbero byte diversi. Qui si riscrive il serial con un
    valore fisso e si ricalcolano i CRC di pagina: output riproducibile
    al byte, come tutto il resto della pipeline."""
    dati = bytearray(open(percorso, "rb").read())
    pos = 0
    while pos < len(dati):
        assert dati[pos:pos + 4] == b"OggS", "pagina OGG non allineata"
        nseg = dati[pos + 26]
        corpo = sum(dati[pos + 27:pos + 27 + nseg])
        fine = pos + 27 + nseg + corpo
        dati[pos + 14:pos + 18] = struct.pack("<I", serial)
        dati[pos + 22:pos + 26] = b"\0\0\0\0"
        dati[pos + 22:pos + 26] = struct.pack("<I", _crc_ogg(dati[pos:fine]))
        pos = fine
    open(percorso, "wb").write(bytes(dati))


def scrivi_wav(percorso, buf):
    with wavemod.open(percorso, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        dati = bytearray()
        for x in buf:
            v = max(-1.0, min(1.0, x))
            dati += struct.pack("<h", int(v * 32767))
        w.writeframes(bytes(dati))


def main():
    radice = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    cartella = os.path.join(radice, "assets", "musica")
    os.makedirs(cartella, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        for nome, bpm, battiti, compositore in TRACCE:
            n = int(round(battiti * 60.0 / bpm * SR))
            buf = [0.0] * n
            compositore(buf, bpm)
            normalizza(buf, 0.5)
            wav = os.path.join(tmp, f"{nome}.wav")
            ogg = os.path.join(cartella, f"{nome}.ogg")
            scrivi_wav(wav, buf)
            subprocess.run(
                ["ffmpeg", "-y", "-loglevel", "error", "-i", wav,
                 "-c:a", "libvorbis", "-q:a", "4", "-f", "ogg", ogg],
                check=True,
            )
            fissa_serial(ogg)
            durata = n / SR
            dim = os.path.getsize(ogg)
            print(f"{nome:9} {durata:5.1f}s  {bpm:3} bpm  {dim // 1024:4} KB")
    print(f"{len(TRACCE)} tracce in {cartella}")


if __name__ == "__main__":
    sys.exit(main())
