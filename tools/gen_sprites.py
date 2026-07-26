#!/usr/bin/env python3
"""Genera tutti gli sprite pixel-art del gioco (19 PNG RGBA) senza dipendenze
esterne: solo zlib e struct della standard library.

Come ritoccare uno sprite a mano:
  1. modifica la sua mappa ASCII qui sotto (una stringa per riga di pixel;
     ogni carattere e' un colore della PALETTE, "." = trasparente);
  2. rilancia  python3 tools/gen_sprites.py  dalla radice del progetto.
Le dimensioni e i caratteri vengono validati con assert: se sbagli una riga
lo script si ferma invece di produrre uno sprite storto.

Palette (SPEC.md sez. 2.2), carattere -> colore:
  .  trasparente          K  #0B0E14 nero spazio    D  #1A1F2B scafo scuro
  g  #2E3644 grigio scafo m  #546170 grigio medio   l  #8C99A8 metallo chiaro
  w  #D8DEE6 quasi bianco O  #F2A33C arancio        r  #B3541E ruggine
  C  #4FC3E8 ciano        c  #1E6E8C ciano scuro    V  #9B7BDB viola
  G  #57C25B verde        e  #2E7D46 verde scuro    Y  #F2D24B giallo
  R  #E84C3D rosso        x  #7A2318 rosso scuro
"""
import os
import struct
import zlib

PALETTE = {
    ".": (0x00, 0x00, 0x00, 0),    # trasparente
    "K": (0x0B, 0x0E, 0x14, 255),  # nero spazio (contorni)
    "D": (0x1A, 0x1F, 0x2B, 255),  # scafo scuro
    "g": (0x2E, 0x36, 0x44, 255),  # grigio scafo
    "m": (0x54, 0x61, 0x70, 255),  # grigio medio
    "l": (0x8C, 0x99, 0xA8, 255),  # metallo chiaro
    "w": (0xD8, 0xDE, 0xE6, 255),  # quasi bianco
    "O": (0xF2, 0xA3, 0x3C, 255),  # arancio
    "r": (0xB3, 0x54, 0x1E, 255),  # ruggine
    "C": (0x4F, 0xC3, 0xE8, 255),  # ciano
    "c": (0x1E, 0x6E, 0x8C, 255),  # ciano scuro
    "V": (0x9B, 0x7B, 0xDB, 255),  # viola
    "G": (0x57, 0xC2, 0x5B, 255),  # verde
    "e": (0x2E, 0x7D, 0x46, 255),  # verde scuro
    "Y": (0xF2, 0xD2, 0x4B, 255),  # giallo
    "R": (0xE8, 0x4C, 0x3D, 255),  # rosso
    "x": (0x7A, 0x23, 0x18, 255),  # rosso scuro
}

# ============================ MODULI 32x32 ============================

REATTORE = [
    "................................",
    "..........KKKKKKKKKKKK..........",
    ".........KOOOOOOOOOOOOK.........",
    "........KOOOOOOOOOOOOOOK........",
    "........KOOOOOOOOOOOOOOK........",
    ".......KOOOOOOOOOOOOOOOOK.......",
    "......KOOOOOOOOOOOOOOOOOOK......",
    "......KOOOOOOOOOOOOOOOOOOK......",
    ".....KOOOOOOOOOOOOOOOOOOOOK.....",
    "....KOOOOOOOOOrrrrOOOOOOOOOK....",
    "....KOOOOOOOrrrrrrrrOOOOOOOK....",
    "...KOOOOOOOrrrYYYYrrrOOOOOOOK...",
    "..KOOOOOOOrrYYYYYYYYrrOOOOOOOK..",
    "KKOOOOOOOOrrYwwYYYYYrrOOOOOOOOKK",
    "KmOOOOOOOrrYYYYYYYYYYrrOOOOOOOmK",
    "KmOOOOOOOrrYYYYYYYYYYrrOOOOOOOmK",
    "KmOOOOOOOrrYYYYYYYYYYrrOOOOOOOmK",
    "KmOOOOOOOrrYYYYYYYYYYrrOOOOOOOmK",
    "KKOOOOOOOOrrYYYYYYYYrrOOOOOOOOKK",
    "..KOOOOOOOrrYYYYYYYYrrOOOOOOOK..",
    "...KOOOOOOOrrrYYYYrrrOOOOOOOK...",
    "....KOOOOOOOrrrrrrrrOOOOOOOK....",
    "....KOOOOOOOOOrrrrOOOOOOOOOK....",
    ".....KrrrrrrrrrrrrrrrrrrrrK.....",
    "......KrrrrrrrrrrrrrrrrrrK......",
    "......KrrrrrrrrrrrrrrrrrrK......",
    ".......KrrrrrrrrrrrrrrrrK.......",
    "........KrrrrrrrrrrrrrrK........",
    "........KrrrrrrrrrrrrrrK........",
    ".........KrrrrrrrrrrrrK.........",
    "..........KKKKKKKKKKKK..........",
    "................................",
]

LIFE_SUPPORT = [
    ".........KKKKKKKKKKKKKK.........",
    ".......KKCCCCCCCCCCCCCCKK.......",
    "......KCCCCCCCCCCCCCCCCCCK......",
    ".....KCCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK..CC.",
    ".....KwcccccccccccccccccccK..CC.",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwcccccccccccccccccccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK..CC.",
    ".....KwcccccccccccccccccccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwcccccccccccccccccccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK...C.",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    ".....KCCllllCCCCCCCCCllllcK.....",
    ".....KlwllllllCCCCClwlllllK.....",
    ".....KllllllllCCCCClllllllK.....",
    ".....KlllllllllCCClllllllllK....",
    ".....KlllllllllKKKlllllllllK....",
    ".....KllllllllK...KllllllllK....",
    ".....KllllllllK...KllllllllK....",
    "......KmmmmmmK.....KmmmmmmK.....",
    "......KKmmmmKK.....KKmmmmKK.....",
    "........KKKK.........KKKK.......",
]

DORMITORIO = [
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".KKKKKKKKKKKKKKKKKKKKKKKKKKKKKK.",
    "KllllllllllllllllllllllllllllllK",
    "KmmmmmmmmmmmmmmmmmmmmmmmlllllmmK",
    "KmmmmmmmmmmmmmmmmmmmmmmllcccllmK",
    "KmmmmmmmmmmmmmmmmmmmmmmlcwccclmK",
    "KmgmmmmmmmmmmmmmmmgmmmmlccccclmK",
    "KmgwwwwVVVVVVVVVVVgmmmmlccccclmK",
    "KmgwwwwVVVVVVVVVVVgmmmmllcccllmK",
    "KmgVVVVVVVVVVVVVVVgmmmmmlllllmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgwwwwVVVVVVVVVVVgmmmmmmmmmmmmK",
    "KmgwwwwVVVVVVVVVVVgmmmmmmmmmmmmK",
    "KmgVVVVVVVVVVVVVVVgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KmgmmmmmmmmmmmmmmmgmmmmmmmmmmmmK",
    "KggggggggggggggggggggggggggggggK",
    "KggggggggggggggggggggggggggggggK",
    ".KKKKKKKKKKKKKKKKKKKKKKKKKKKKKK.",
    "................................",
    "................................",
]

LABORATORIO = [
    "................................",
    "................................",
    "................................",
    ".........KKKK...................",
    ".........KCcK...................",
    ".........KwcK...................",
    ".........KCcK..........KKKK.....",
    ".........KCcK..........KccK.....",
    "........KcCccK.........KccK.....",
    "........KcCccK.........KccK.....",
    ".......KccCcccK........KccK.....",
    ".......KccCcccK........KccK.....",
    "......KcccCccccK.......KccK.....",
    "......KcccCccccK.......KccK.....",
    "......KcccccccccK......KccK.....",
    ".....KccccccccccK......KCCK.....",
    ".....KCCCCCCCCCCK......KCCK.....",
    "....KCCCCCCCCCCCCK.....KCCK.....",
    "....KCCCCCCCCCCCCK.....KCCK.....",
    "...KCCCCCCCCCCCCCCK....KCCK.....",
    "...KccccccccccccccK.....KK......",
    ".KKGGGGGGGGGGGGGGGGKKKKKGGKKKKK.",
    ".KGGGGGGGGGGGGGGGGGGGGGGGGGGGGK.",
    ".KGGGGGGGGGGGGGGGGGGGGGGGGGGGGK.",
    ".KeeeeeeeeeeeeeeeeeeeeeeeeeeeeK.",
    ".KeeeeeeeeeeeeeeeeeeeeeeeeeeeeK.",
    ".KeeeeeeeeeeeeeeeeeeeeeeeeeeeeK.",
    ".KeeeeeeeeeeeeeeeeeeeeeeeeeeeeK.",
    ".KeeeeeeeeeeeeeeeeeeeeeeeeeeeeK.",
    ".KeeeeeeeeeeeeeeeeeeeeeeeeeeeeK.",
    ".KKKKKKKKKKKKKKKKKKKKKKKKKKKKKK.",
    "................................",
]

RADIATORE = [
    ".KKKKK...KKKKK....KKKKK...KKKKK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    ".KwlmK...KwlmK....KwlmK...KwlmK.",
    "KmmmmmKKKmmmmmKKKKmmmmmKKKmmmmmK",
    "KllllllllllllllllllllllllllllllK",
    "KmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmK",
    "KmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmK",
    "KggggggggggggggggggggggggggggggK",
    "KggggggggggggggggggggggggggggggK",
    "KmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
]

CORRIDOIO = [
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "...KKK........KKK........KKK....",
    "...KlK........KlK........KlK....",
    "KKKglgKKKKKKKKglgKKKKKKKKglgKKKK",
    "KllglgllllllllglgllllllllglglllK",
    "KllglgllllllllglgllllllllglglllK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmglgmmmmmmmmglgmmmK",
    "KggglgggggggggglgggggggggglggggK",
    "KggglgggggggggglgggggggggglggggK",
    "KKKglgKKKKKKKKglgKKKKKKKKglgKKKK",
    "...KlK........KlK........KlK....",
    "...KKK........KKK........KKK....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

# ====================== VARIANTI CORRIDOIO 32x32 ======================
# Contratto con il codice (che ruota questi sprite a passi di 90 gradi):
#   corridoio.png       tubo orizzontale (sinistra-destra)
#   corridoio_v.png     tubo verticale (alto-basso)
#   corridoio_curva.png curva che collega destra e basso
#   corridoio_t.png     T che collega sinistra, destra e basso
#   corridoio_croce.png incrocio a quattro vie
# Continuita': ogni bocca aperta occupa la fascia 11..23 (righe per i
# tratti orizzontali, colonne per i verticali) con sezione K/ll/mmmmmmm/
# gg/K, identica a corridoio.png: due celle adiacenti combaciano sempre.

CORRIDOIO_V = [
    "...........KKKKKKKKKKKKK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    ".........KKgggggggggggggKK......",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    ".........KKgggggggggggggKK......",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    ".........KKgggggggggggggKK......",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KKKKKKKKKKKKK........",
]

CORRIDOIO_CURVA = [
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".........................KKK....",
    ".........................KlK....",
    ".............KKKKKKKKKKKKglgKKKK",
    "............KllllllllllllglglllK",
    "...........KlllllllllllllglglllK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmmmmmglgmmmK",
    "...........KllmmmmmmmggggglggggK",
    "...........KllmmmmmmmggggglggggK",
    "...........KllmmmmmmmggKKglgKKKK",
    "...........KllmmmmmmmggK.KlK....",
    ".........KKgggggggggggggKKKK....",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KKKKKKKKKKKKK........",
]

CORRIDOIO_T = [
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "...KKK...................KKK....",
    "...KlK...................KlK....",
    "KKKglgKKKKKKKKKKKKKKKKKKKglgKKKK",
    "KllglglllllllllllllllllllglglllK",
    "KllglglllllllllllllllllllglglllK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KggglggggggggggggggggggggglggggK",
    "KggglggggggggggggggggggggglggggK",
    "KKKglgKKKKKKllmmmmmmmggKKglgKKKK",
    "...KlK.....KllmmmmmmmggK.KlK....",
    "...KKK...KKgggggggggggggKKKK....",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KKKKKKKKKKKKK........",
]

CORRIDOIO_CROCE = [
    "...........KKKKKKKKKKKKK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    ".........KKgggggggggggggKK......",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...KKK.....KllmmmmmmmggK.KKK....",
    "...KlK.....KllmmmmmmmggK.KlK....",
    "KKKglgKKKKKKllmmmmmmmggKKglgKKKK",
    "KllglglllllllllllllllllllglglllK",
    "KllglglllllllllllllllllllglglllK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KmmglgmmmmmmmmmmmmmmmmmmmglgmmmK",
    "KggglggggggggggggggggggggglggggK",
    "KggglggggggggggggggggggggglggggK",
    "KKKglgKKKKKKllmmmmmmmggKKglgKKKK",
    "...KlK.....KllmmmmmmmggK.KlK....",
    "...KKK...KKgggggggggggggKKKK....",
    ".........KlllllllllllllllK......",
    ".........KKgggggggggggggKK......",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KllmmmmmmmggK........",
    "...........KKKKKKKKKKKKK........",
]

# ========================== OSTACOLO 32x32 ============================
# Detrito/asteroide che occupa una cella della griglia: la stazione ci si
# costruisce intorno. Silhouette volutamente irregolare e organica (blob
# roccioso sbozzato, nessun lato dritto completo) per dire "non e' un
# edificio" a colpo d'occhio, in contrasto con i moduli geometrici.
# Solo colori neutri: contorno K, corpo D/g/m, poche luci l (rim light
# alto-sinistra, crateri, glint); MAI colori di stato (Y/R/G/C/O/w).
# Riempimento ~61% della cella.

OSTACOLO = [
    "................................",
    "................................",
    "...........KKKKKKKKKK...........",
    ".........KKlllllllllmK..........",
    ".......KKllmmmlmmmmmgmKK........",
    "......KllmmmmmmmmmmmmmggK.......",
    "......KlmlggmmgmmmgggggmgKK.....",
    ".....KlmgmgggmmggmmmggmggggK....",
    "....KlmmmggDggmmgmmgggggggKK....",
    "...KlmgmmgDDDmmmmggggggggK......",
    "...KlmmmmggDmmmmmgmgggggggK.....",
    "..KlmmmmmmgmmmmmgggggggmgmgKKK..",
    "..KlmmmlmgmmmmmgggggggggggggggK.",
    ".KlmgmmmgmmmmmgggggmggggggggggK.",
    ".KlgmgmgggmmmggggggggDgggggggK..",
    "..KlmmmgDlmmggggggggDDDlggggDgK.",
    "..KlmmmgmlmggmgmmggggDlmmggDDgK.",
    ".KlmmmmmmmgggggggggggllgggDDgK..",
    "..KlmmmmmmmggggggggggggggDDDgK..",
    "..KKKmgmggmggggmgmgggmggDDDgK...",
    ".....KmggggggDmgmggggggDDDDDgK..",
    ".....KgggggmgmlgmgggggDDDDDgK...",
    "....KgmggggggggggggggDDDDDgK....",
    "....KgmgggggggggggggDDDDDgK.....",
    ".....KgggggggggggggDDDDDDggK....",
    ".....KKggggggggggmDDDDDggKK.....",
    ".......KgggmgggggDDDDggKK.......",
    "........KKggggggDDDDgKK.........",
    "..........KKgggDDDggK...........",
    "............KKggggKK............",
    "..............KKKK..............",
    "................................",
]

# ===================== MODULI SBLOCCABILI 32x32 =======================
# I cinque moduli della campagna lunga (SPEC-CAMPAGNA.md sez. 3), sbloccati
# ai livelli 5/15/25/35/45. Stesse regole di stile dei sei base: contorno K,
# colori di stato (Y/R) mai su parti strutturali.

# Cella di accumulo: pila verticale con tacche di carica verdi e terminali.
# Silhouette alta e stretta (come radiatore/corridoio, il riempimento cede
# alla leggibilita' della sagoma).
BATTERIA = [
    "................................",
    "..........KKK......KKK..........",
    "..........KlK......KlK..........",
    "........KKKKKKKKKKKKKKKK........",
    "........KllllllllllllllK........",
    "........KmmmmmmmmmmmmmmK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgGGGGGGGGGGGGgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KgeeeeeeeeeeeegK........",
    "........KgeeeeeeeeeeeegK........",
    "........KgDDDDDDDDDDDDgK........",
    "........KllllllllllllllK........",
    "........KmmmmmmmmmmmmmmK........",
    "......KKKKKKKKKKKKKKKKKKKK......",
    "......KmmmmmmmmmmmmmmmmmmK......",
    "......KKKKKKKKKKKKKKKKKKKK......",
    "................................",
]

# Serra: cupola di vetro ciano con vegetazione dentro, vasca di coltura e
# base tecnica su piedini. Silhouette a campana, unica nel set.
SERRA = [
    "................................",
    "................................",
    "............KKKKKKKK............",
    "..........KwCCCCCCccK...........",
    "........KwCCCCCCCCCCcccK........",
    ".......KwCCCCCCCCCCCCcccK.......",
    "......KwCCCCCCCCCCCCCCcccK......",
    ".....KwCCCCCCCCCCCCCCCCcccK.....",
    "....KwCCCCCCCCCCCCCCCCCCcccK....",
    "....KwCCCCCCCCCCCCCCCCCCcccK....",
    "....KwCCCCGCCCGCCCCGCCCCcccK....",
    "....KwCCGGGCCGGGCCCGGGCCcccK....",
    "....KwCGGGGGCGGGGGCGGGGGCccK....",
    "....KwGGGGGGGGGGGGGGGGGGeeeK....",
    "....KweGGGGGGGGGGGGGGGGeeeeK....",
    "....KrrrrrrrrrrrrrrrrrrrrrrK....",
    "....KmmmmmmmmmmmmmmmmmmmmmmK....",
    "..KKKKKKKKKKKKKKKKKKKKKKKKKKKK..",
    "..KllllllllllllllllllllllllllK..",
    "..KmmDDmmDDmmDDmmDDmmDDmmDDmmK..",
    "..KmmmmmmmmmmmmmmmmmmmmmmmmmmK..",
    "..KggggggggggggggggggggggggggK..",
    "..KKKKKKKKKKKKKKKKKKKKKKKKKKKK..",
    "....KmmK................KmmK....",
    "....KKKK................KKKK....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

# Gru spazzadetriti: torretta cingolata con braccio a bandiera e pinza
# calata dal carrello. Silhouette a L rovesciata, irregolare apposta.
GRU = [
    "................................",
    "................................",
    ".....KKK........................",
    ".....KmllllllllllllllllllllmK...",
    ".....KKKKKKKKKKKKKKKKKKKKKKKK...",
    ".....KmlmK..............KlK.....",
    ".....KmlmK..............KlK.....",
    ".....KmlmK.............KKlKK....",
    ".....KmlmK............KlKKKlK...",
    ".....KmlmK............KlK.KlK...",
    ".....KmlmK............KKK.KKK...",
    ".....KmlmK......................",
    ".....KmlmK......................",
    ".....KmlmK......................",
    ".....KmOmK......................",
    ".....KmlmK......................",
    ".....KmlmK......................",
    ".....KmlmK......................",
    ".....KmlmK......................",
    "...KKKKKKKKKKKKK................",
    "...KmmCCmmmmmmmK................",
    "...KmmmmmmmmmmmK................",
    "...KgggggggggggK................",
    "...KKKKKKKKKKKKK................",
    "..KKKKKKKKKKKKKKKKKK............",
    "..KmlmlmlmlmlmlmlmlK............",
    "..KlmlmlmlmlmlmlmlmK............",
    "..KmlmlmlmlmlmlmlmlK............",
    "..KKKKKKKKKKKKKKKKKK............",
    "................................",
    "................................",
    "................................",
]

# Condotto termico: radiatore pesante, blocco scuro con condotto interno
# chiaro e cinque pettini di alette orizzontali. Il radiatore base ha barre
# verticali sottili: qui il pettine e' orizzontale e massiccio.
CONDOTTO = [
    "................................",
    "................................",
    "..........KKKKKKKKKKKK..........",
    "..........KDDDDllDDDDK..........",
    "..........KDDDDllDDDDK..........",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..KmmmmmmKKDDDDllDDDDKKmmmmmmK..",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..........KDDDDllDDDDK..........",
    "..........KDDDDllDDDDK..........",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..KmmmmmmKKDDDDllDDDDKKmmmmmmK..",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..........KDDDDllDDDDK..........",
    "..........KDDDDllDDDDK..........",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..KmmmmmmKKDDDDllDDDDKKmmmmmmK..",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..........KDDDDllDDDDK..........",
    "..........KDDDDllDDDDK..........",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..KmmmmmmKKDDDDllDDDDKKmmmmmmK..",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..........KDDDDllDDDDK..........",
    "..........KDDDDllDDDDK..........",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..KmmmmmmKKDDDDllDDDDKKmmmmmmK..",
    "..KKKKKKKKKDDDDllDDDDKKKKKKKKK..",
    "..........KDDDDllDDDDK..........",
    "..........KKKKKKKKKKKK..........",
    "................................",
    "................................",
]

# Centro di comando: torre di controllo con vetrata ciano, console
# arancio, antenna con faro e basamento largo su piedini.
CENTRO_COMANDO = [
    "................................",
    "...............KCK..............",
    "...............KlK..............",
    "...............KlK..............",
    "............KKKKlKKKK...........",
    "..........KKKKKKKKKKKK..........",
    "..........KmllllllllmK..........",
    "..........KmllllllllmK..........",
    "..........KmCCCCCCCCmK..........",
    "..........KmCCCCCCCCmK..........",
    "..........KmCCwCCCCCmK..........",
    "..........KmCCCCCCCCmK..........",
    "..........KmccccccccmK..........",
    "..........KmllllllllmK..........",
    "..........KmlOOllOOlmK..........",
    "..........KmllllllllmK..........",
    "..........KmmmmmmmmmmK..........",
    "..........KggggggggggK..........",
    "..........KKKKKKKKKKKK..........",
    "....KKKKKKKKKKKKKKKKKKKKKKKK....",
    "....KllllllllllllllllllllllK....",
    "....KmmDDmmDDmmDDmmDDmmDDmmK....",
    "....KmmmmmmmmmmmmmmmmmmmmmmK....",
    "....KmmmmmmmmmmmmmmmmmmmmmmK....",
    "....KggggggggggggggggggggggK....",
    "....KggggggggggggggggggggggK....",
    "....KKKKKKKKKKKKKKKKKKKKKKKK....",
    "......KmmK............KmmK......",
    "......KKKK............KKKK......",
    "................................",
    "................................",
    "................................",
]

# ========================= RITRATTI 32x32 =============================
# Busti dei personaggi per i fumetti dei briefing. Pelle in l/m/w (la
# palette non ha incarnati: i toni metallici tengono il set coerente),
# un colore dominante diverso per ciascuno, mai Y/R strutturali.

INGEGNERE = [
    "................................",
    "................................",
    "..........KKKKKKKKKKKK..........",
    ".........KOOOOOOOOOOOOK.........",
    "........KOOOOOOOOOOOOOOK........",
    "........KOOOOOOOOOOOOOOK........",
    "........KOOrOOOOOOOOrOOK........",
    "......KKOOOOOOOOOOOOOOOOKK......",
    "......KKKKKKKKKKKKKKKKKKKK......",
    "........KllllllllllllllK........",
    "........KlKKKKKllKKKKKlK........",
    "........KlKCCCKllKCCCKlK........",
    "........KlKKKKKllKKKKKlK........",
    "........KllllllllllllllK........",
    "........KmllllllllllllmK........",
    "........KmllllKKKKllllmK........",
    "........KmmllllllllllmmK........",
    ".........KmmmllllllmmmK.........",
    "............KllllK..............",
    ".....KKKKKKKKKllllKKKKKKKKK.....",
    ".....KggggggOgllllgOggggggK.....",
    ".....KggggggOggggggOggggggK.....",
    ".....KggggggOOOOOOOOggggggK.....",
    ".....KggggggggggggggggggggK.....",
    ".....KggggggggggggggggggggK.....",
    ".....KDDDDDDDDDDDDDDDDDDDDK.....",
    ".....KKKKKKKKKKKKKKKKKKKKKK.....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

MEDICO = [
    "................................",
    "................................",
    "................................",
    "..........KKKKKKKKKKKK..........",
    ".........KmmmmmmmmmmmmK.........",
    "........KmmllllllllllmmK........",
    "........KllllllllllllllK........",
    "........KllllllllllllllK........",
    "........KlllKKllllKKlllK........",
    "........KlllmmllllmmlllK........",
    "........KllllllllllllllK........",
    "........KllllllllllllllK........",
    "........KlllllKKKKlllllK........",
    "........KmllllllllllllmK........",
    ".........KmllllllllllmK.........",
    "............KllllK..............",
    ".....KKKKKKKKKllllKKKKKKKKK.....",
    ".....KwwwwwwwwllllwwwwwwwwK.....",
    ".....KwwwwwwwwwwwwwwwwwwwwK.....",
    ".....KwwwGGwwwwwwwwwwwwwwwK.....",
    ".....KwwGGGGwwwwwwwwwwwwwwK.....",
    ".....KwwwGGwwwwwwwwwwwwwwwK.....",
    ".....KwwwwwwwwwwwwwwwwwwwwK.....",
    ".....KwwwwwwwwmmwwwwwwwwwwK.....",
    ".....KllllllllllllllllllllK.....",
    ".....KKKKKKKKKKKKKKKKKKKKKK.....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

CAPOSQUADRA = [
    "................................",
    "................................",
    "................................",
    ".........KKKKKKKKKKKKKK.........",
    "........KDDDDDDDDDDDDDDK........",
    "........KDDDDDDDDDDDDDDK........",
    "......KKDDDDDDDDDDDDDDDDKK......",
    "......KKKKKKKKKKKKKKKKKKKK......",
    "........KllllllllllllllK........",
    "........KllKKKllllKKKllK........",
    "........KllllllllllllllK........",
    "........KllllllllllllllK........",
    "........KllllKKKKKKllllK........",
    "........KmmllllllllllmmK........",
    "........KmmmmmmmmmmmmmmK........",
    "...........KllllllK.............",
    ".....KKKKKKKKllllllKKKKKKKK.....",
    ".....KgggggggllllllgggggggK.....",
    ".....KggggggggggggggggggggK.....",
    ".....KggKKggggggggggKKggggK.....",
    ".....KggggggggggggggggggggK.....",
    ".....KggggggggggggggggggggK.....",
    ".....KDDDDDDDDDDDDDDDDDDDDK.....",
    ".....KKKKKKKKKKKKKKKKKKKKKK.....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

SCIENZIATA = [
    "................................",
    "................................",
    "..............KKKK..............",
    ".............KrrrrK.............",
    "..........KKKKKKKKKKKK..........",
    ".........KrrrrrrrrrrrrK.........",
    "........KrrrrrrrrrrrrrrK........",
    "........KrrllllllllllrrK........",
    "........KrllllllllllllrK........",
    "........KrKKKKllKKKKllrK........",
    "........KrKCCKllKCCKllrK........",
    "........KrllllllllllllrK........",
    "........KrlllllKKlllllrK........",
    "........KrrllllllllllrrK........",
    ".........KrmllllllllmrK.........",
    "............KllllK..............",
    ".....KKKKKKKKKllllKKKKKKKKK.....",
    ".....KeeeeeeeGllllGeeeeeeeK.....",
    ".....KwwwwwwwwwwwwwwwwwwwwK.....",
    ".....KwwwwwwwwGGwwwwwwwwwwK.....",
    ".....KwwwwwwwwwwwwwwwwwwwwK.....",
    ".....KwwwwwwwmmwwwwwwwwwwwK.....",
    ".....KwwwwwwwwwwwwwwwwwwwwK.....",
    ".....KllllllllllllllllllllK.....",
    ".....KKKKKKKKKKKKKKKKKKKKKK.....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

COMANDANTE = [
    "................................",
    "................................",
    "................................",
    "..........KKKKKKKKKKKK..........",
    ".........KggggggggggggK.........",
    "........KggllllllllllggK........",
    "........KgllllllllllllgK........",
    "........KmllllllllllllmK........",
    "........KlllKKllllKKlllK........",
    "........KllllllllllllllK........",
    "........KllllllllllllllK........",
    "........KllllKKKKKKllllK........",
    "........KmllllllllllllmK........",
    ".........KmmllllllllmmK.........",
    "............KllllK..............",
    ".....KKKKKKKKKllllKKKKKKKKK.....",
    ".....KOOOKDDDDDDDDDDDDKOOOK.....",
    ".....KDDDDDDDDDDDDDDDDDDDDK.....",
    ".....KDDDDDDDDDwwDDDDDDDDDK.....",
    ".....KDDDDDDDDDDDDDDDDDDDDK.....",
    ".....KDDDDDDDDDwwDDDDDDDDDK.....",
    ".....KDDDDDDDDDDDDDDDDDDDDK.....",
    ".....KggggggggggggggggggggK.....",
    ".....KKKKKKKKKKKKKKKKKKKKKK.....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]

# =========================== SFONDO 64x64 =============================
# Tile ripetibile (il bordo destro continua nel sinistro, l'alto nel
# basso) da affiancare sotto la griglia. Solo colori scurissimi per non
# competere con i segnali di stato: base K, polvere D, stelle g e
# pochissimi pixel m come stelle piu' vive. Mai w/Y/R/C/O/G.

STELLE = [
    "KKKKKKKKKKKKKKKKKKgKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKgKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDgDKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKmKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKDKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKKKKDKKK",
    "KKKKKKKKKKgKKKKKKKKKKKKKKKKKKKKKKKKgKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKgKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKmKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKgKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKgKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKK",
    "KKKKKKKKKKKKKKKKDgDKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKmKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKgKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKK",
    "KKKKKKKKKgKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKDKKKKKKKKKKKKKKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKKKKDKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKmKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKgKKKKKKKKKKKKKDKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKD",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKgKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKmKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
]

# ========================= BADGE E ICONE 8x8 ==========================
# Il fulmine e' lo stesso disegno per badge/energia e icone/energia
# (stesso colore #F2D24B in SPEC 2.4 e 2.5). Il busto ha due varianti di
# colore: grigio #8C99A8 per il badge, quasi bianco #D8DEE6 per l'icona.

FULMINE = [
    "...YY...",
    "..YYY...",
    ".YYYY...",
    ".YYYYY..",
    "...YYY..",
    "..YYY...",
    ".YYY....",
    ".YY.....",
]

BUSTO_BADGE = [
    "........",
    "...ll...",
    "..llll..",
    "..llll..",
    "...ll...",
    ".llllll.",
    "llllllll",
    "llllllll",
]

BUSTO_ICONA = [
    "........",
    "...ww...",
    "..wwww..",
    "..wwww..",
    "...ww...",
    ".wwwwww.",
    "wwwwwwww",
    "wwwwwwww",
]

AVARIA = [
    "...RR...",
    "...RR...",
    "..RKKR..",
    "..RKKR..",
    ".RRKKRR.",
    ".RRRRRR.",
    "RRRKKRRR",
    "RRRRRRRR",
]

BOLLA = [
    "..CCCC..",
    ".CwwCCC.",
    "CwwCCCCC",
    "CCCCCCCC",
    "CCCCCCcc",
    ".CCCCcc.",
    "..Cccc..",
    "........",
]

FIAMMA = [
    "....O...",
    "...OO...",
    "...OOO..",
    "..OOOO..",
    ".OOYYOO.",
    ".OYYYYO.",
    ".OOYYOO.",
    "..OOOO..",
]

# =========================== FILE DA GENERARE =========================

# ========================== CURSORE 16x16 =============================
# Freccia pixel-art per il cursore custom della finestra: corpo quasi
# bianco, contorno nero, coda in ciano. La punta e' in (0,0): l'hotspot
# in main.rs deve combaciare.

CURSORE = [
    ".K..............",
    ".KK.............",
    ".KwK............",
    ".KwwK...........",
    ".KwwwK..........",
    ".KwwwwK.........",
    ".KwwwwwK........",
    ".KwwwwwwK.......",
    ".KwwwwwwlK......",
    ".KwwwwwwwlK.....",
    ".KwwwwwKKKKK....",
    ".KwwKwwlK.......",
    ".KwK.KwwlK......",
    ".KK...KwwlK.....",
    ".K.....KwlK.....",
    "........KK......",
]

# ========================== MIRINO 16x16 ==============================
# Cursore di piazzamento in partita: croce sottile con centro libero,
# bracci quasi-bianchi bordati di nero, punte esterne ciano. Hotspot al
# centro (8,8).

MIRINO = [
    "................",
    ".......KK.......",
    ".......KCK......",
    ".......KwK......",
    ".......KwK......",
    ".......KwK......",
    "................",
    ".KKKKKK...KKKKK.",
    "KCwwwwK...KwwwCK",
    ".KKKKKK...KKKKK.",
    "................",
    ".......KwK......",
    ".......KwK......",
    ".......KwK......",
    ".......KCK......",
    ".......KK.......",
]

# ========================= MEDAGLIE 24x24 =============================
# Tondo con nastro rosso scuro; cambia solo il metallo del disco. Le tre
# versioni sono generate dallo stesso template sostituendo i caratteri
# B (corpo), S (ombra), H (riflesso) — cosi' la sagoma resta identica.

_MEDAGLIA_TEMPLATE = [
    "........KxxK..KxxK......",
    "........KxxK..KxxK......",
    ".........KxxK.KxxK......",
    ".........KxxKKxxK.......",
    "..........KxxxxxxK......",
    "..........KxxxxxxK......",
    ".........KKKKKKKK.......",
    ".......KKBBBBBBBBKK.....",
    "......KBBHHBBBBBBBBK....",
    ".....KBBHHBBBBBBBBBSK...",
    ".....KBHHBBBBBBBBBBSK...",
    "....KBBHBBBBBBBBBBSSSK..",
    "....KBBBBBBBBBBBBBSSSK..",
    "....KBBBBBBBBBBBBBSSSK..",
    "....KBBBBBBBBBBBBBSSSK..",
    "....KBBBBBBBBBBBBSSSSK..",
    ".....KBBBBBBBBBBBSSSK...",
    ".....KBBBBBBBBBBBSSSK...",
    "......KBBBBBBBBBSSSK....",
    ".......KKBBBBBBSSKK.....",
    ".........KKKKKKKK.......",
    "........................",
    "........................",
    "........................",
]


def _medaglia(corpo, ombra, riflesso):
    tabella = str.maketrans({"B": corpo, "S": ombra, "H": riflesso})
    return [riga.translate(tabella) for riga in _MEDAGLIA_TEMPLATE]


MEDAGLIA_ORO = _medaglia("Y", "O", "w")
MEDAGLIA_ARGENTO = _medaglia("l", "m", "w")
MEDAGLIA_RAME = _medaglia("r", "x", "O")

# ========================== MONETE 16x16 ==============================
# Moneta d'oro che ruota su se stessa in 4 frame (piena, scorcio, taglio,
# scorcio dall'altro lato) piu' la versione spenta (grigia, statica) per
# le medaglie non d'oro nella schermata "livello completato".

MONETA_1 = [
    "....KKKKKKKK....",
    "..KKYYYYYYYYKK..",
    "..KYwwYYYYYYOK..",
    ".KYwwYYYYYYYOOK.",
    ".KYwYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    ".KYYYYYYYYYYOOK.",
    "..KYYYYYYYYYOK..",
    "..KKYYYYYYYYKK..",
    "....KKKKKKKK....",
    "................",
]

MONETA_2 = [
    "......KKKK......",
    ".....KYYYYK.....",
    "....KYwYYYOK....",
    "....KYwYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    "....KYYYYYOK....",
    ".....KYYYYK.....",
    "......KKKK......",
    "................",
]

MONETA_3 = [
    ".......KK.......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    "......KYOK......",
    ".......KK.......",
    "................",
]

# lo scorcio opposto e' lo specchio del frame 2
MONETA_4 = [riga[::-1] for riga in MONETA_2]

MONETA_SPENTA = [
    riga.translate(str.maketrans({"Y": "g", "O": "D", "w": "m"}))
    for riga in MONETA_1
]


# ======================= ICONE FACILITY 16x16 =========================
# Le sei facility del Marketplace, in card e nell'HUD scorte. Silhouette
# distinte: bombola, valvola che sfiata, chiave inglese, cassa, capsula,
# sonda-trivella. Colori dominanti diversi per riconoscerle a colpo d'occhio.

FAC_OSSIGENO = [
    "......KKKK......",
    "......KmlK......",
    ".....KKllKK.....",
    "....KCCCCCCK....",
    "...KCCwwCCCcK...",
    "...KCwwCCCCcK...",
    "...KCwCCCCCcK...",
    "...KCwCCCCCcK...",
    "...KCCCCCCCcK...",
    "...KCCCCCCccK...",
    "...KCCCCCcccK...",
    "...KCCCCCcccK...",
    "...KcCCCCcccK...",
    "....KccccccK....",
    ".....KKKKKK.....",
    "................",
]

FAC_SPURGO = [
    "....O..w........",
    "...w..O.........",
    "..KKKKKK........",
    "..KllllK........",
    "...KllK.........",
    "..KKmmKK........",
    ".KmmmmmmK.......",
    "KKmllllmKKKKKK..",
    "KmmllllmmmmmmK..",
    "KmmllllmmmmmmK..",
    "KKmllllmKKKKKK..",
    ".KmmmmmmK.......",
    "..KKKKKK........",
    "................",
    "................",
    "................",
]

FAC_RIPARAZIONE = [
    ".........KKKK...",
    "........KlwwlK..",
    ".......KlwKKwK..",
    ".......KlwK.KK..",
    "......KKlwK.....",
    "......KlwlK.....",
    ".....KlwlK......",
    "....KlwlK.......",
    "...KlwlK........",
    "..KlwlK.........",
    ".KlwwlK.........",
    "KKlwwlKK........",
    "KlwKKwwlK.......",
    "KlwK.KwlK.......",
    ".KK...KK........",
    "................",
]

FAC_STIVA = [
    "................",
    "..KKKKKKKKKKKK..",
    ".KrrrrrrrrrrrrK.",
    ".KrOrrrrrrrrOrK.",
    ".KKKKKKKKKKKKKK.",
    ".KrrKrrrrrrKrrK.",
    ".KrrKrrrrrrKrrK.",
    ".KrrKrrKKrrKrrK.",
    ".KrrKrrKKrrKrrK.",
    ".KrrKrrrrrrKrrK.",
    ".KrrKrrrrrrKrrK.",
    ".KKKKKKKKKKKKKK.",
    ".KrrrrrrrrrrrrK.",
    "..KKKKKKKKKKKK..",
    "................",
    "................",
]

FAC_COLONI = [
    ".......KK.......",
    "......KwwK......",
    ".....KwwwwK.....",
    "....KwwwwwwK....",
    "....KwlwwlwK....",
    "...KwwKKKKwwK...",
    "...KwwKCCKwwK...",
    "...KwKCCCCKwK...",
    "...KwKCCcCKwK...",
    "...KwwKCcKwwK...",
    "...KwwKKKKwwK...",
    "...KlwwwwwwlK...",
    "..KKlKlwwlKlKK..",
    "..KOOK.KK.KOOK..",
    "...KK......KK...",
    "................",
]

FAC_SONDA = [
    "......KKK.......",
    ".....KlwlK......",
    ".....KlwlK......",
    "....KllwllK.....",
    "....KllwllK.....",
    "....KmlwlmK.....",
    "....KmlllmK.....",
    "...KKmlllmKK....",
    "...KOmlllmOK....",
    "...KOKlllKOK....",
    "....KKmmmKK.....",
    ".....KmmmK......",
    ".....KKmKK......",
    "......KmK.......",
    ".......K........",
    "................",
]

SPRITES = {
    "sprites/cursore.png": (CURSORE, 16, 16),
    "sprites/mirino.png": (MIRINO, 16, 16),
    "sprites/facilities/ossigeno.png": (FAC_OSSIGENO, 16, 16),
    "sprites/facilities/spurgo.png": (FAC_SPURGO, 16, 16),
    "sprites/facilities/riparazione.png": (FAC_RIPARAZIONE, 16, 16),
    "sprites/facilities/stiva.png": (FAC_STIVA, 16, 16),
    "sprites/facilities/coloni.png": (FAC_COLONI, 16, 16),
    "sprites/facilities/sonda.png": (FAC_SONDA, 16, 16),
    "sprites/medaglie/oro.png": (MEDAGLIA_ORO, 24, 24),
    "sprites/medaglie/argento.png": (MEDAGLIA_ARGENTO, 24, 24),
    "sprites/medaglie/rame.png": (MEDAGLIA_RAME, 24, 24),
    "sprites/monete/accesa_1.png": (MONETA_1, 16, 16),
    "sprites/monete/accesa_2.png": (MONETA_2, 16, 16),
    "sprites/monete/accesa_3.png": (MONETA_3, 16, 16),
    "sprites/monete/accesa_4.png": (MONETA_4, 16, 16),
    "sprites/monete/spenta.png": (MONETA_SPENTA, 16, 16),
    "sprites/moduli/reattore.png":     (REATTORE, 32, 32),
    "sprites/moduli/life_support.png": (LIFE_SUPPORT, 32, 32),
    "sprites/moduli/dormitorio.png":   (DORMITORIO, 32, 32),
    "sprites/moduli/laboratorio.png":  (LABORATORIO, 32, 32),
    "sprites/moduli/radiatore.png":    (RADIATORE, 32, 32),
    "sprites/moduli/corridoio.png":    (CORRIDOIO, 32, 32),
    "sprites/moduli/corridoio_v.png":  (CORRIDOIO_V, 32, 32),
    "sprites/moduli/corridoio_curva.png": (CORRIDOIO_CURVA, 32, 32),
    "sprites/moduli/corridoio_t.png":  (CORRIDOIO_T, 32, 32),
    "sprites/moduli/corridoio_croce.png": (CORRIDOIO_CROCE, 32, 32),
    "sprites/moduli/batteria.png":     (BATTERIA, 32, 32),
    "sprites/moduli/serra.png":        (SERRA, 32, 32),
    "sprites/moduli/gru.png":          (GRU, 32, 32),
    "sprites/moduli/condotto.png":     (CONDOTTO, 32, 32),
    "sprites/moduli/centro_comando.png": (CENTRO_COMANDO, 32, 32),
    "sprites/ritratti/ingegnere.png":  (INGEGNERE, 32, 32),
    "sprites/ritratti/medico.png":     (MEDICO, 32, 32),
    "sprites/ritratti/caposquadra.png": (CAPOSQUADRA, 32, 32),
    "sprites/ritratti/scienziata.png": (SCIENZIATA, 32, 32),
    "sprites/ritratti/comandante.png": (COMANDANTE, 32, 32),
    "sprites/ostacolo.png":             (OSTACOLO, 32, 32),
    "sprites/badge/energia.png":       (FULMINE, 8, 8),
    "sprites/badge/equipaggio.png":    (BUSTO_BADGE, 8, 8),
    "sprites/badge/avaria.png":        (AVARIA, 8, 8),
    "sprites/icone/energia.png":       (FULMINE, 8, 8),
    "sprites/icone/ossigeno.png":      (BOLLA, 8, 8),
    "sprites/icone/calore.png":        (FIAMMA, 8, 8),
    "sprites/icone/equipaggio.png":    (BUSTO_ICONA, 8, 8),
    "sprites/sfondo/stelle.png":       (STELLE, 64, 64),
}


def validate(ascii_map, w, h, name):
    assert len(ascii_map) == h, (
        f"{name}: {len(ascii_map)} righe invece di {h}")
    for i, row in enumerate(ascii_map):
        assert len(row) == w, (
            f"{name} riga {i}: {len(row)} caratteri invece di {w}")
        for ch in row:
            assert ch in PALETTE, (
                f"{name} riga {i}: carattere {ch!r} non in palette")


def png_chunk(tag, data):
    return (struct.pack(">I", len(data)) + tag + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))


def encode_png(ascii_map, w, h):
    """PNG RGBA 8-bit, filtro 0 per ogni riga, un solo IDAT."""
    raw = bytearray()
    for row in ascii_map:
        raw.append(0)  # filtro 0 (None)
        for ch in row:
            raw.extend(PALETTE[ch])
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n"
            + png_chunk(b"IHDR", ihdr)
            + png_chunk(b"IDAT", zlib.compress(bytes(raw), 9))
            + png_chunk(b"IEND", b""))


def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets = os.path.join(root, "assets")
    for rel, (ascii_map, w, h) in SPRITES.items():
        validate(ascii_map, w, h, rel)
        path = os.path.join(assets, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as f:
            f.write(encode_png(ascii_map, w, h))
        print(f"scritto {rel} ({w}x{h})")
    print(f"{len(SPRITES)} sprite generati in {assets}")


if __name__ == "__main__":
    main()
