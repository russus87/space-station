#!/usr/bin/env python3
"""Verifica del test silhouette (SPEC.md sez. 2.1/2.3).

Per ogni coppia dei 6 moduli calcola la maschera di opacita' (pixel con
alpha > 0) e stampa quanto le due sagome si sovrappongono:
  - IoU        = intersezione / unione (metrica principale);
  - inter/min  = intersezione / area della sagoma piu' piccola
                 (100% = una sagoma contenuta nell'altra).
Due maschere quasi identiche (IoU molto alto) = test fallito: correggere
la forma nella mappa ASCII di tools/gen_sprites.py e rigenerare.

Uso: python3 tools/check_sprites.py
"""
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gen_sprites as gs

MODULI = {
    "reattore": gs.REATTORE,
    "life_support": gs.LIFE_SUPPORT,
    "dormitorio": gs.DORMITORIO,
    "laboratorio": gs.LABORATORIO,
    "radiatore": gs.RADIATORE,
    "corridoio": gs.CORRIDOIO,
}

SOGLIA_IOU = 80.0  # sopra questa soglia due sagome sono "quasi identiche"


def mask(ascii_map):
    return [[gs.PALETTE[ch][3] > 0 for ch in row] for row in ascii_map]


def area(m):
    return sum(sum(row) for row in m)


def main():
    masks = {}
    for nome, mappa in MODULI.items():
        gs.validate(mappa, 32, 32, nome)
        masks[nome] = mask(mappa)

    print("riempimento cella 32x32:")
    for nome, m in masks.items():
        a = area(m)
        print(f"  {nome:14s} {a:4d} px  {100.0 * a / 1024:5.1f}%")

    print("\nsovrapposizione silhouette (coppie):")
    nomi = list(masks)
    fallite = []
    for i in range(len(nomi)):
        for j in range(i + 1, len(nomi)):
            A, B = masks[nomi[i]], masks[nomi[j]]
            inter = sum(
                1 for y in range(32) for x in range(32) if A[y][x] and B[y][x]
            )
            aA, aB = area(A), area(B)
            iou = 100.0 * inter / (aA + aB - inter)
            imin = 100.0 * inter / min(aA, aB)
            flag = "  <-- TROPPO SIMILI" if iou > SOGLIA_IOU else ""
            print(f"  {nomi[i]:12s} vs {nomi[j]:12s}"
                  f"  IoU {iou:5.1f}%   inter/min {imin:5.1f}%{flag}")
            if iou > SOGLIA_IOU:
                fallite.append((nomi[i], nomi[j], iou))

    if fallite:
        print(f"\nFALLITO: {len(fallite)} coppie sopra la soglia "
              f"IoU {SOGLIA_IOU}%")
        sys.exit(1)
    print(f"\nOK: nessuna coppia sopra la soglia IoU {SOGLIA_IOU}%")


if __name__ == "__main__":
    main()
