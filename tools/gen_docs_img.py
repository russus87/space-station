#!/usr/bin/env python3
"""Genera le immagini del manuale (MANUALE.md) in docs/img/.

Sul modello di gen_icon.py: riusa mappe ASCII, palette ed encoder PNG di
gen_sprites.py e scala nearest-neighbour per duplicazione di pixel, zero
dipendenze fuori dalla stdlib. Da rilanciare quando cambiano gli sprite:

    python3 tools/gen_docs_img.py
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_sprites import SPRITES, encode_png, validate

# (chiave in SPRITES, dimensione di uscita)
IMMAGINI = [
    # gli 11 moduli della palette, 96×96 (32 × 3)
    ("sprites/moduli/reattore.png", 96),
    ("sprites/moduli/life_support.png", 96),
    ("sprites/moduli/dormitorio.png", 96),
    ("sprites/moduli/laboratorio.png", 96),
    ("sprites/moduli/radiatore.png", 96),
    ("sprites/moduli/corridoio.png", 96),
    ("sprites/moduli/batteria.png", 96),
    ("sprites/moduli/serra.png", 96),
    ("sprites/moduli/gru.png", 96),
    ("sprites/moduli/condotto.png", 96),
    ("sprites/moduli/centro_comando.png", 96),
    # i 5 ritratti dei personaggi, 160×160 (32 × 5)
    ("sprites/ritratti/ingegnere.png", 160),
    ("sprites/ritratti/medico.png", 160),
    ("sprites/ritratti/caposquadra.png", 160),
    ("sprites/ritratti/scienziata.png", 160),
    ("sprites/ritratti/comandante.png", 160),
    # il detrito
    ("sprites/ostacolo.png", 96),
]


def scala(ascii_map, fattore):
    """Ingrandisce la mappa ASCII duplicando ogni carattere e ogni riga."""
    return [
        "".join(ch * fattore for ch in row)
        for row in ascii_map
        for _ in range(fattore)
    ]


def main():
    radice = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    out = os.path.join(radice, "docs", "img")
    os.makedirs(out, exist_ok=True)
    for chiave, taglia in IMMAGINI:
        ascii_map, w, h = SPRITES[chiave]
        validate(ascii_map, w, h, chiave)
        fattore = taglia // w
        grande = scala(ascii_map, fattore)
        nome = os.path.basename(chiave)
        path = os.path.join(out, nome)
        with open(path, "wb") as f:
            f.write(encode_png(grande, w * fattore, h * fattore))
        print(f"scritto docs/img/{nome} ({taglia}x{taglia})")
    print(f"{len(IMMAGINI)} immagini in {out}")


if __name__ == "__main__":
    main()
