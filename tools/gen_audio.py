#!/usr/bin/env python3
"""Genera gli effetti sonori del gioco (assets/audio/*.wav).

Stessa filosofia di gen_sprites.py: la fonte è parametrica e testuale (la
tabella SUONI qui sotto), zero dipendenze fuori dalla stdlib (wave, math,
struct), output deterministico — il rumore viene da un LFSR con seed fisso,
non da random. Stile chiptune coerente con la pixel art: onde quadre e
triangolari con inviluppo, niente campionamenti.

Formato: WAV mono 16 bit 22050 Hz. Ogni suono è normalizzato allo stesso
picco (~0.5) così nessun effetto spara più forte degli altri, e ogni
segmento ha attacco e rilascio propri: mai click di troncamento.

Uso:
    python3 tools/gen_audio.py    # riscrive i WAV in assets/audio/
"""

import math
import os
import struct
import wave

RATE = 22050
PICCO = 0.5  # picco di normalizzazione comune a tutti i suoni


# ---------------- sintesi ----------------

def rumore_lfsr(n, seed=0xACE1):
    """Rumore pseudo-casuale deterministico (LFSR 16 bit, seed fisso)."""
    reg = seed
    out = []
    for _ in range(n):
        bit = ((reg >> 0) ^ (reg >> 2) ^ (reg >> 3) ^ (reg >> 5)) & 1
        reg = ((reg >> 1) | (bit << 15)) & 0xFFFF
        out.append((reg & 0xFF) / 127.5 - 1.0)
    return out


def seg(dur, f0, f1=None, forma="quadra", vol=1.0, rumore=0.0):
    """Un segmento di nota: durata in secondi, glissando f0→f1 (Hz),
    forma d'onda, volume relativo e quota di rumore (0..1)."""
    n = int(dur * RATE)
    f1 = f0 if f1 is None else f1
    grezzo = rumore_lfsr(n) if rumore > 0 else None
    fase = 0.0
    campioni = []
    for i in range(n):
        t = i / max(n - 1, 1)
        f = f0 + (f1 - f0) * t
        fase += f / RATE
        x = fase % 1.0
        if forma == "quadra":
            s = 1.0 if x < 0.5 else -1.0
        elif forma == "triangolo":
            s = 4.0 * abs(x - 0.5) - 1.0
        else:  # "seno"
            s = math.sin(2.0 * math.pi * x)
        if grezzo is not None:
            s = s * (1.0 - rumore) + grezzo[i] * rumore
        campioni.append(s * vol)
    return inviluppo(campioni)


def sil(dur):
    """Silenzio, per staccare le note."""
    return [0.0] * int(dur * RATE)


def inviluppo(campioni, attacco=0.004, rilascio=0.025):
    """Attacco e rilascio lineari su ogni segmento: niente click."""
    na = min(int(attacco * RATE), len(campioni))
    nr = min(int(rilascio * RATE), len(campioni))
    for i in range(na):
        campioni[i] *= i / max(na, 1)
    for i in range(nr):
        campioni[-1 - i] *= i / max(nr, 1)
    return campioni


# ---------------- il repertorio ----------------
# Ogni suono è una lista di segmenti/silenzi, concatenati nell'ordine.

SUONI = {
    # sirena degli imprevisti: bitonale che sale e scende, ~2.5 s
    "sirena.wav": [
        seg(0.40, 660, forma="quadra", vol=0.85),
        seg(0.40, 880, forma="quadra", vol=0.85),
        seg(0.40, 660, forma="quadra", vol=0.85),
        seg(0.40, 880, forma="quadra", vol=0.85),
        seg(0.40, 660, forma="quadra", vol=0.85),
        seg(0.45, 880, 640, "quadra", vol=0.8),
    ],
    # impatto del meteorite: botto sordo, ~0.4 s
    "impatto.wav": [
        seg(0.08, 220, 70, "quadra", rumore=0.6),
        seg(0.32, 90, 42, "seno", rumore=0.25),
    ],
    # passaggio del pianeta: whoosh grave gentile, ~1.5 s
    "pianeta.wav": [
        seg(0.70, 52, 110, "seno", rumore=0.40, vol=0.9),
        seg(0.80, 110, 48, "seno", rumore=0.40, vol=0.9),
    ],
    # click UI secco, ~60 ms
    "click.wav": [seg(0.06, 1800, 900, "quadra")],
    # piazzamento modulo: clunk meccanico ascendente, ~150 ms
    "costruzione.wav": [seg(0.15, 90, 230, "quadra", rumore=0.10)],
    # rimozione: discendente, ~120 ms
    "rimozione.wav": [seg(0.12, 420, 130, "quadra", rumore=0.10)],
    # warning morbido a due note, ~250 ms
    "avviso.wav": [
        seg(0.10, 620, forma="triangolo"),
        sil(0.03),
        seg(0.12, 495, forma="triangolo"),
    ],
    # allarme serio: due impulsi bassi con rumore, ~400 ms
    "allarme.wav": [
        seg(0.15, 112, 100, "quadra", rumore=0.35),
        sil(0.08),
        seg(0.17, 112, 92, "quadra", rumore=0.35),
    ],
    # nuovo membro d'equipaggio: trillo gentile ascendente, ~300 ms
    "arrivo.wav": [
        seg(0.09, 523, forma="triangolo"),
        sil(0.015),
        seg(0.09, 659, forma="triangolo"),
        sil(0.015),
        seg(0.09, 784, forma="triangolo"),
    ],
    # compera al mercato: registratore di cassa chiptune, ~250 ms
    "acquisto.wav": [
        seg(0.05, 988, forma="quadra"),
        sil(0.02),
        seg(0.18, 1319, forma="triangolo"),
    ],
    # modulo sbloccato: fanfara mini (3 note), ~500 ms
    "sblocco.wav": [
        seg(0.12, 523, forma="quadra"),
        seg(0.12, 659, forma="quadra"),
        seg(0.26, 784, forma="quadra"),
    ],
    # livello completato: fanfara ascendente (5 note), ~900 ms
    "vittoria.wav": [
        seg(0.13, 262, forma="quadra"),
        sil(0.015),
        seg(0.13, 392, forma="quadra"),
        sil(0.015),
        seg(0.13, 523, forma="quadra"),
        sil(0.015),
        seg(0.13, 659, forma="quadra"),
        sil(0.015),
        seg(0.32, 784, forma="quadra"),
    ],
    # medaglia d'oro: la fanfara di vittoria con l'ottava sopra e uno
    # scintillio finale, ~1.2 s
    "oro.wav": [
        seg(0.11, 262, forma="quadra"),
        sil(0.012),
        seg(0.11, 392, forma="quadra"),
        sil(0.012),
        seg(0.11, 523, forma="quadra"),
        sil(0.012),
        seg(0.11, 659, forma="quadra"),
        sil(0.012),
        seg(0.20, 784, forma="quadra"),
        sil(0.02),
        seg(0.28, 1047, forma="quadra"),
        seg(0.07, 1568, forma="triangolo"),
        sil(0.02),
        seg(0.11, 2093, forma="triangolo"),
    ],
    # stazione persa: discesa cupa, ~1 s
    "sconfitta.wav": [
        seg(0.55, 220, 120, "quadra", rumore=0.15),
        seg(0.45, 120, 55, "quadra", rumore=0.25),
    ],
}


# ---------------- scrittura ----------------

def normalizza(campioni, picco=PICCO):
    massimo = max((abs(s) for s in campioni), default=1.0) or 1.0
    k = picco / massimo
    return [s * k for s in campioni]


def scrivi_wav(path, campioni):
    campioni = normalizza(campioni)
    dati = b"".join(
        struct.pack("<h", int(max(-1.0, min(1.0, s)) * 32767)) for s in campioni
    )
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(dati)


def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    cartella = os.path.join(root, "assets", "audio")
    os.makedirs(cartella, exist_ok=True)
    for nome, parti in SUONI.items():
        campioni = [s for parte in parti for s in parte]
        path = os.path.join(cartella, nome)
        scrivi_wav(path, campioni)
        # verifica numerica: picco entro i limiti e coda in dissolvenza
        picco = max(abs(s) for s in normalizza(campioni))
        coda = max((abs(s) for s in normalizza(campioni)[-44:]), default=0.0)
        assert picco <= PICCO + 1e-6, f"{nome}: picco {picco}"
        assert coda < 0.05, f"{nome}: coda non in dissolvenza ({coda})"
        print(f"scritto {nome}  {len(campioni) / RATE:.2f}s  picco {picco:.2f}")
    print(f"{len(SUONI)} suoni generati in {cartella}")


if __name__ == "__main__":
    main()
