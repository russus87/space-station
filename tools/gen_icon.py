#!/usr/bin/env python3
"""Genera le icone dell'app (128/256/512) dallo sprite del reattore.

Riusa mappa ASCII, palette ed encoder PNG di gen_sprites.py: l'icona resta
pixel art coerente col gioco, scalata a blocchi interi (nearest neighbour),
zero dipendenze fuori dalla stdlib. Uso:

    python3 tools/gen_icon.py [cartella_output]     # default: packaging/
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_sprites import SPRITES, encode_png, validate

TAGLIE = (128, 256, 512)
SORGENTE = "sprites/moduli/reattore.png"


def scala(ascii_map, fattore):
    """Ingrandisce la mappa ASCII duplicando ogni carattere e ogni riga."""
    return [
        "".join(ch * fattore for ch in row)
        for row in ascii_map
        for _ in range(fattore)
    ]


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "packaging")
    os.makedirs(out, exist_ok=True)
    ascii_map, w, h = SPRITES[SORGENTE]
    validate(ascii_map, w, h, SORGENTE)
    for taglia in TAGLIE:
        fattore = taglia // w
        grande = scala(ascii_map, fattore)
        path = os.path.join(out, f"icon-{taglia}.png")
        with open(path, "wb") as f:
            f.write(encode_png(grande, w * fattore, h * fattore))
        print(f"scritto {path} ({taglia}x{taglia})")


if __name__ == "__main__":
    main()
