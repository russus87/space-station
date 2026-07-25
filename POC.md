# Build a Space Station — PoC

Obiettivo della PoC: verificare che il **cuore del gioco** (bilancio di risorse + cascata di
guasti) sia divertente **prima** di investire in grafica, salvataggi e contenuti.
Se guardando i numeri diventare rossi non provi niente, il progetto si archivia.

## Cosa NON è
Non è una simulazione fisica, non ci sono agenti liberi, non c'è pathfinding.
È un **grafo di flusso contabile a stato stazionario**: a ogni tick si sommano
produzione e consumo per risorsa e si decide verde/rosso.

## Le 4 risorse
| Risorsa   | Semantica |
|-----------|-----------|
| Energia   | prodotta dai reattori, consumata da quasi tutto. Deficit = blackout. |
| Ossigeno  | prodotto dal life support, consumato dall'equipaggio. Deficit = l'equipaggio muore. |
| Calore    | prodotto da reattori/laboratori, dissipato dai radiatori. Eccesso = surriscaldamento. |
| Equipaggio| posti letto = capienza; i laboratori richiedono equipaggio per lavorare. |

## I 6 moduli (v1)
| Modulo       | Energia | Ossigeno | Calore | Equipaggio |
|--------------|---------|----------|--------|------------|
| Reattore     | +100    | 0        | +40    | 0          |
| Life Support | -30     | +50      | +5     | 0          |
| Dormitori    | -10     | 0        | +2     | +4 (posti) |
| Laboratorio  | -40     | 0        | +25    | richiede 2 |
| Radiatore    | -5      | 0        | -50    | 0          |
| Corridoio    | -1      | 0        | 0      | 0          |

I numeri sono un punto di partenza da tarare giocando: vanno tenuti in **un unico posto**
nel codice (tabella di costanti), perché il bilanciamento è il lavoro vero.

## La cascata di guasti (il divertimento)
La cascata è l'unica ragione per cui questa PoC esiste. Deve essere visibile e a stadi:
1. **Energia in deficit** → i moduli si spengono in ordine di priorità (prima laboratori,
   poi dormitori, per ultimo il life support) finché il bilancio non torna in pari.
2. **Life support spento** → l'ossigeno inizia a scendere.
3. **Ossigeno a zero** → l'equipaggio inizia a morire (uno ogni tot tick).
4. **Calore in eccesso** → dopo N tick di surriscaldamento un modulo casuale va in avaria
   (non produce più nulla finché non lo si ripara/rimuove), il che può innescare 1.

Ogni evento della cascata scrive una riga in un **log a schermo** ("Blackout: Laboratorio 2
spento", "Ossigeno critico", "Equipaggio: 3 → 2"). Il log è ciò che rende leggibile la
catena causale.

## Interazione minima
- Griglia 2D, click per piazzare il modulo selezionato, click destro per rimuoverlo.
- Tasti `1..6` per scegliere il modulo dalla palette.
- `Spazio` avvia/ferma la simulazione (in pausa si costruisce senza conseguenze).
- HUD sempre visibile con le 4 risorse: valore, delta per tick, colore verde/giallo/rosso.
- I moduli spenti o in avaria si vedono a colpo d'occhio (grigio / bordo rosso).

## Regole di scope
- Nessun asset esterno: forme geometriche e testo, colori per distinguere i moduli.
- Nessun salvataggio, nessun menu, nessun audio, nessuna generazione procedurale.
- Un solo binario, dipendenze: `bevy` (+ `rand` se serve per l'avaria casuale).
- L'adiacenza tra moduli **non** conta in v1 (niente grafo topologico): la stazione è un
  unico bilancio globale. Se la PoC convince, l'adiacenza è il primo passo della v2.

## Criterio di riuscita
Si costruisce una stazione, si preme Spazio, e togliendo un reattore si vede la catena
blackout → life support giù → ossigeno giù → morti, raccontata dal log, in meno di un minuto.
