# Space Station — Specifica di design (iterazione post-PoC)

Documento prescrittivo per chi implementa. La PoC ha validato il cuore del gioco
(bilancio + cascata di guasti in `src/sim.rs`): **quel codice non si ridisegna**.
Questa iterazione attacca un solo problema: la leggibilità dell'informazione.
Ordine di priorità degli interventi:

1. **Identità del modulo** — guardando una cella si deve capire cosa fa e quanto costa, senza legenda.
2. **Stato esplicito** — un modulo fermo deve dire *perché* è fermo (energia / equipaggio / avaria).
3. **HUD produzione contro consumo** — i due termini a confronto, non solo il netto.
4. Solo dopo: sprite più belli.

Il log eventi funziona e resta, concettualmente invariato.

> **Iterazione 4 (implementata)**: campagna a 50 livelli col generatore
> (§13.2), moduli sbloccabili, personaggi a fumetto con storia completa
> (intermezzi inclusi) e colonna sonora.
>
> **Iterazione 5 (implementata)**: il tetto tick è un **timer** nell'HUD
> (`TEMPO m:ss`, giallo/rosso; velocità di gioco ×1/×2/×4 col tasto `V`,
> `sim.rs`); completare un livello vale una **medaglia** (oro ≤40% del
> tetto, argento ≤70%, rame entro il limite — `progressi.rs`) che colora
> la griglia di selezione e frutta crediti una tantum (1/2/3); col saldo
> si compra nel **Marketplace** del titolo (card a icone, `menu.rs`) e le
> scorte si usano in partita col tasto `M` (`mercato.rs`, mai a vuoto);
> ogni livello apre con un **prologo a fumetto** (solo alla prima visita,
> `prologo.rs`); i personaggi **commentano gli eventi** in partita
> (`commenti.rs`); cursore pixel-art con mirino in griglia. Il quadro
> giocatore completo è in `MANUALE.md`; ciò che resta proposto è
> nell'header di `SPEC-CAMPAGNA.md`.
>
> **Iterazione 6 (implementata)**: **regola dei conduttori** (§12.1:
> l'energia viaggia solo lungo reattori e corridoi, gli altri moduli sono
> foglie — fabbisogni, budget e livelli 7-50 rigenerati di conseguenza);
> **imprevisti casuali** (`imprevisti.rs`: meteorite, tempesta EM, sciame
> sui corridoi, passaggio del pianeta; preavviso di 4 tick con sirena
> animata e musica sospesa, grazia al tick 40, cooldown 50, campagna dal
> livello 8); medaglie ritarate (oro ≤35%, argento ≤60%); Batteria
> (capienza 250, ricarica 25, calore +1) e Serra (−8/+25/+6) ribilanciate
> con obiettivi pesati per blocco; Registro collassabile, scorte a icone
> con tooltip (via il tasto M), briefing sostituito dal prologo.
>
> **Iterazione 7 (implementata)**: **riparazione con costo** (`sim.rs`:
> il tasto R apre un cantiere da 2 di equipaggio × 8 tick, sospeso senza
> braccia; la scorta "Squadra di riparazione" resta l'unica via
> istantanea); **squadra schierabile** (`squadra.rs` + selettore nel
> prologo: tratti passivi di Vera/Tomas/Dario/Mira/Ilse, sblocchi ai
> traguardi 10/20/30/40/50, comando doppio con Ilse); **eventi con
> scelta** (`eventi.rs`: 5 bivi rari che congelano la sim, conseguenze
> dichiarate, mai durante gli imprevisti, grazia 60/cooldown 80, campagna
> dal livello 8); **obiettivi bonus** di campagna (`livelli.rs`:
> senza-demolire / senza-scorte / sotto-budget / ossigeno-mai-50, +1
> credito una tantum, sorveglianza senza falsi positivi per Gru e Sonda);
> **Sfida del giorno** (`generatore::genera_giornaliera`: livello dalla
> data, miglior tempo per giorno in `progressi.txt`); **particelle**
> (`particelle.rs`: fumo/scintille/bollicine, cap 120) e **titolo vivo**
> (`attract.rs`: pianeta, meteore e stelle dietro il menu
> semitrasparente).

---

## 1. Visione

Sei il progettista di una stazione spaziale. Piazzi moduli su una griglia,
avvii la simulazione e guardi il bilancio di quattro risorse (energia,
ossigeno, calore, equipaggio) tick dopo tick. Quando qualcosa si rompe — o
quando l'hai progettata male — parte una cascata di guasti a stadi:
blackout → life support giù → ossigeno giù → equipaggio che muore.
Il ciclo di gioco è: **costruisci → avvia → leggi il bilancio → reagisci al guasto**.

---

## 2. Art direction: pixel art

### 2.1 Vincoli tecnici

| Vincolo | Valore |
|---|---|
| Dimensione sprite modulo | **32×32 px** (art nativa), un file per modulo |
| Dimensione badge di stato | 8×8 px |
| Dimensione icone risorsa | 8×8 px |
| Scaling | Solo fattori interi (×1, ×2, ×3…), campionamento nearest-neighbor (`ImagePlugin::default_nearest()`) |
| Dettaglio minimo | Niente dettagli sotto il pixel: niente antialiasing, niente alpha parziale dentro lo sprite (alpha solo 0 o 255) |
| Palette | Fissa, 16 colori (sezione 2.2). Ogni pixel di ogni sprite usa uno di questi 16 colori |
| Test di leggibilità | Ogni sprite, convertito in scala di grigi (luminanza), deve restare identificabile tra i 6 per sola silhouette. Il test si fa desaturando lo sheet: se due moduli si confondono, la silhouette va corretta, non il colore |

### 2.2 Palette (16 colori, esadecimali)

| # | Hex | Nome | Uso principale |
|---|---|---|---|
| 1 | `#0B0E14` | Nero spazio | Sfondo finestra, ombre dure, contorni sprite |
| 2 | `#1A1F2B` | Scafo scuro | Sfondo pannelli UI, ombre sprite |
| 3 | `#2E3644` | Grigio scafo | Linee griglia, corpi metallici in ombra |
| 4 | `#546170` | Grigio medio | Corpi metallici, corridoio |
| 5 | `#8C99A8` | Metallo chiaro | Luci sui metalli, radiatore, badge equipaggio |
| 6 | `#D8DEE6` | Quasi bianco | Testo principale, highlight, icona equipaggio |
| 7 | `#F2A33C` | Arancio | Reattore, icona calore |
| 8 | `#B3541E` | Ruggine | Ombra del reattore, metallo caldo |
| 9 | `#4FC3E8` | Ciano | Life support, icona ossigeno |
| 10 | `#1E6E8C` | Ciano scuro | Ombra life support, vetri |
| 11 | `#9B7BDB` | Viola | Dormitorio |
| 12 | `#57C25B` | Verde | Laboratorio, stato OK nell'HUD |
| 13 | `#2E7D46` | Verde scuro | Ombra laboratorio |
| 14 | `#F2D24B` | Giallo | Stato warning nell'HUD, badge "senza energia", nucleo del reattore |
| 15 | `#E84C3D` | Rosso | Stato critico nell'HUD, badge avaria, bordo avaria |
| 16 | `#7A2318` | Rosso scuro | Overlay avaria, ombre d'allarme |

Regole d'uso: i colori 12/14/15 (verde/giallo/rosso) sono il linguaggio di
stato dell'HUD e dei badge; non usarli come colore dominante di uno sprite
che non c'entra con quello stato (eccezioni ammesse e già elencate: nucleo
del reattore in giallo, corpo del laboratorio in verde — sono campiture
interne allo sprite, non segnali di stato a schermo intero).

### 2.3 Silhouette dei 6 moduli (32×32)

Ogni sprite ha contorno esterno in `#0B0E14` (1 px) e riempie almeno il 70%
della cella; la forma è ciò che identifica il modulo, il colore aiuta soltanto.

| Modulo | Silhouette (vincolante) |
|---|---|
| Reattore | Esagono pieno quasi a tutta cella con nucleo circolare luminoso al centro (`#F2D24B` su corpo `#F2A33C`/`#B3541E`) e due condotti corti che sporgono a sinistra e a destra. |
| Life Support | Serbatoio cilindrico verticale arrotondato con griglia di ventilazione a lamelle orizzontali nella metà alta e due bombole tonde alla base; 2–3 bollicine (`#4FC3E8`) che salgono su un lato. |
| Dormitorio | Blocco rettangolare orizzontale con letto a castello di profilo (due cuccette sovrapposte, `#9B7BDB`) e un oblò circolare in alto a destra. |
| Laboratorio | Banco basso con sopra una beuta triangolare grande e una provetta: la sagoma triangolare del vetro è il tratto distintivo. Corpo `#57C25B`/`#2E7D46`, vetro `#1E6E8C`. |
| Radiatore | Quattro alette verticali sottili e parallele su un collettore orizzontale alla base (sagoma "a pettine", `#8C99A8`/`#546170`). È l'unico modulo fatto di lamelle. |
| Corridoio | Tubo orizzontale a tutta larghezza, basso e piatto, con tre centine (anelli) verticali (`#546170`/`#2E3644`). La silhouette più semplice delle sei. |

Verifica in scala di grigi: esagono ≠ cilindro ≠ blocco a cuccette ≠ beuta
triangolare ≠ pettine ≠ tubo piatto. Se il test fallisce si esagera la
silhouette, non si ritocca la palette.

### 2.4 Stati del modulo: overlay, mai sprite alternativi

Lo sprite base è **uno solo** per modulo. Gli stati si comunicano con
overlay ed effetti sopra lo sprite, così l'identità resta sempre leggibile.

| Stato | Condizione (dai campi di `Module`) | Resa visiva |
|---|---|---|
| Attivo | `powered && !broken` (per il laboratorio anche `staffed`) | Sprite base, nessun overlay |
| Non alimentato | `!powered && !broken` (e, se laboratorio, `staffed == true`) | Overlay `#0B0E14` al 55% di alpha su tutta la cella + badge **fulmine** `#F2D24B` 8×8 nell'angolo alto-destro |
| Scollegato | `!collegato && !broken` (la sua rete non ha reattori funzionanti, sez. 12.1) | Stesso overlay scuro al 55% + lo **stesso sprite del fulmine tinto grigio** `#8C99A8`: "qui la corrente non arriva proprio", non "la corrente non basta" |
| Fermo per equipaggio | Laboratorio con `!staffed && !broken` | Stesso overlay scuro al 55% + badge **busto/omino** `#8C99A8` 8×8 nell'angolo alto-destro |
| In avaria | `broken` | Overlay `#7A2318` al 45% + bordo di 2 px (in scala art) `#E84C3D` sul perimetro cella + badge **triangolo con punto esclamativo** `#E84C3D` 8×8, lampeggiante (visibility toggle ogni 0,5 s) |

Regole:
- Un solo badge per cella; se più condizioni coesistono la priorità è avaria > scollegato > equipaggio > energia.
- L'overlay è un secondo sprite/quad sopra quello base (z superiore), non una modifica del colore dello sprite.
- I badge sono gli stessi 3 disegni riusati ovunque (celle, pannello ispezione, eventualmente log).

### 2.5 Icone risorsa (8×8, riusate ovunque)

Quattro icone pixel condivise da HUD, palette, pannello ispezione e badge:

| Risorsa | Icona | Colore |
|---|---|---|
| Energia | Fulmine | `#F2D24B` |
| Ossigeno | Bolla (cerchio con riflesso) | `#4FC3E8` |
| Calore | Fiamma | `#F2A33C` |
| Equipaggio | Busto (testa+spalle) | `#D8DEE6` |

### 2.6 Etichetta numerica di istanza

Nell'angolo **basso-destro** della cella: il numero di istanza (lo stesso che
compare nel log: "Laboratorio 2" → la cella del laboratorio mostra "2").
Colore `#D8DEE6` con ombra 1 px `#0B0E14`. Le sigle a tre lettere (REA, LAB…)
spariscono dalle celle: l'identità la dà lo sprite, il numero serve solo a
correlare cella e riga di log.

### 2.7 Ghost di piazzamento

L'anteprima sotto il cursore usa lo sprite del modulo selezionato al 40% di
alpha (tinta bianca), snappata alla cella; nascosta se fuori griglia o su
cella occupata (comportamento attuale, da conservare).

---

## 3. Layout dello schermo

### 3.1 Problema da correggere

Oggi HUD, palette e log sono `Text2d` a coordinate mondo fisse tarate su
1160×800 (`ui.rs`): ridimensionando la finestra i testi si scollano dalla
scena. **Tutta la UI testuale va rifatta con Bevy UI (`Node`, flexbox,
`Val::Percent`) ancorata ai bordi della finestra.** Solo la griglia e i
moduli restano in world-space.

### 3.2 Zone (percentuali della finestra)

```
+--------------------------------------------------------------+
|  HUD (100% larghezza, ~9% altezza, min 52 px)                |
+------------+-------------------------------------------------+
|  Palette   |                                                 |
|  + Pannello|            Griglia 14×8                         |
|  ispezione |            (centrata nell'area residua)         |
|  (~18% w)  |                                                 |
+------------+-------------------------------------------------+
|  Log eventi (100% larghezza, ~18% altezza, min 8 righe)      |
+--------------------------------------------------------------+
```

| Zona | Posizione | Dimensione |
|---|---|---|
| HUD | Barra superiore, tutta la larghezza | Altezza 9% (min 52 px) |
| Palette + ispezione | Colonna sinistra, tra HUD e log | Larghezza 18% (min 190 px) |
| Griglia | Area residua, griglia centrata dentro | Il resto (~82% × ~73%) |
| Log | Barra inferiore, tutta la larghezza | Altezza 18% (min 8 righe di testo) |

### 3.3 Scala della griglia

La griglia resta 14×8 celle logiche. L'art nativa della cella è 32×32.
Fattore di resa: **il più grande intero `s` tale che `14·32·s` e `8·32·s`
stanno nell'area griglia** (minimo `s = 1`). La griglia è centrata
nell'area; al resize si ricalcola `s`. Alla risoluzione di default
1160×800 risulta `s = 2` (celle a 64 px, come la PoC).

Finestra: default 1160×800, dimensione minima 960×640.

---

## 4. HUD: produzione contro consumo

Principio: **mai solo il netto**. Ogni risorsa mostra i due termini a
confronto e il margine che ne deriva. Quattro pannelli affiancati più un
indicatore di stato simulazione.

### 4.1 Indicatore di stato simulazione

A sinistra nell'HUD: `TICK 123` (bianco) quando gira, `PAUSA` (grigio
`#8C99A8`) da fermo. In pausa l'HUD continua a mostrare **l'anteprima** del
bilancio (il ricalcolo in pausa esiste già in `sim_tick` e va conservato):
è la funzione che permette di progettare senza conseguenze.

### 4.2 I quattro pannelli

Ogni pannello: icona risorsa + nome, riga produzione/consumo, riga margine
(grande, colorata secondo soglia).

| Pannello | Riga prod/cons | Riga principale | Note |
|---|---|---|---|
| Energia | `PROD 200 · CONS 85` | Margine `+115/t` | PROD = somma energia dei produttori attivi non rotti; CONS = somma consumi dei moduli alimentati. Il margine è `sim.energia_margine`. In blackout aggiungere la dicitura `BLACKOUT` in rosso |
| Ossigeno | `PROD 50 · CONS 40` | Riserva `72/100` con mini-barra orizzontale + delta `(+10/t)` | CONS = `equipaggio × 10` (`OSSIGENO_PER_CREW`). La barra usa il colore di stato |
| Calore | `PROD 72 · DISS 100` | Netto `−28/t` | Se surriscaldamento in corso: countdown `avaria tra N tick` (già calcolabile: `TICK_SURRISCALDAMENTO − sim.surriscaldamento`) |
| Equipaggio | `A BORDO 4 · POSTI 8` | `4/8` + `lab richiedono 4` | Il fabbisogno è `2 ×` numero laboratori non rotti |

### 4.3 Soglie di colore (invariate rispetto alla PoC, qui rese esplicite)

| Risorsa | Verde `#57C25B` | Giallo `#F2D24B` | Rosso `#E84C3D` |
|---|---|---|---|
| Energia | margine ≥ 20 e nessun blackout | 0 ≤ margine < 20, nessun blackout | blackout (≥1 modulo non alimentato) |
| Ossigeno | riserva > 30 e delta ≥ 0 | delta < 0 (riserva > 30) | riserva ≤ 30 |
| Calore | netto ≤ 0 | netto > 0, surriscaldamento < 3 tick | surriscaldamento ≥ 3 tick |
| Equipaggio | a bordo = posti letto | a bordo < posti letto | asfissia in corso (O2 = 0 e a bordo > 0) |

### 4.4 Da dove arrivano PROD e CONS

`sim.rs` espone oggi solo i margini. I termini separati si calcolano in un
sistema UI **di sola lettura** che itera `Query<&Module>` e somma positivi e
negativi, replicando la definizione di "attivo" della simulazione
(`powered && !broken`; laboratorio anche `staffed`). Le costanti necessarie
(`OSSIGENO_PER_CREW`, `TICK_SURRISCALDAMENTO`, `SOGLIA_O2_CRITICO`) sono già
`pub` in `sim.rs`. Nessuna modifica alla simulazione è richiesta.
(Alternativa più pulita ma che tocca il file `sim.rs`: vedi sezione 10.)

---

## 5. La cella: identità e stato

Riepilogo di come una cella risponde alle due domande del giocatore.

**"Cosa fa questo modulo?"** — La silhouette pixel art (sez. 2.3) è
l'identità primaria; il colore è rinforzo secondario; il numero di istanza
(sez. 2.6) collega la cella alle righe del log. Nessuna sigla, nessuna
legenda necessaria.

**"Perché è fermo?"** — Overlay + badge (sez. 2.4): fulmine giallo = manca
energia; fulmine grigio = scollegato dalla rete (sez. 12.1); omino grigio =
manca equipaggio; triangolo rosso lampeggiante + bordo rosso = avaria (si
ripara con R). Quattro risposte diverse a quattro problemi diversi: il
grigio indistinto della PoC sparisce.

**Dettaglio a richiesta: il pannello ispezione.** Nella parte bassa della
colonna sinistra, sempre visibile, mostra il modulo sotto il cursore
(vuoto se il cursore non è su un modulo):

- Nome e numero ("Laboratorio 2") + miniatura sprite.
- Stato in chiaro, con il motivo: `ATTIVO` / `SENZA ENERGIA` / `SCOLLEGATO — nessun reattore collegato` / `SENZA EQUIPAGGIO (servono 2)` / `IN AVARIA — premi R per riparare`.
- I valori per tick del modulo con le icone risorsa (es. fulmine `−40`, fiamma `+25`, busto `richiede 2`).

---

## 6. Palette dei moduli (colonna sinistra)

Sei slot verticali, uno per modulo, nell'ordine dei tasti 1–6
(`KINDS` in `modules.rs`). Ogni slot contiene:

- Il tasto (`1`…`6`) e la miniatura dello sprite.
- Il nome del modulo.
- Una riga compatta di costi/produzioni con le icone risorsa, solo i valori
  non nulli: es. Laboratorio → fulmine `−40` · fiamma `+25` · busto `2`;
  Dormitorio → fulmine `−10` · busto `+4 posti`.

Lo slot selezionato ha bordo `#D8DEE6` e sfondo `#2E3644`; gli altri sfondo
`#1A1F2B`. Click sinistro su uno slot = selezione (equivalente al tasto).
I valori si leggono dalla `TABELLA` di `modules.rs`, mai duplicati a mano.

Così il costo di un modulo si vede **prima** di piazzarlo: è metà della
risposta al punto "identità" del problema.

---

## 7. Log eventi (barra inferiore)

Concettualmente invariato: stessa sorgente (`EventLog`), stesso formato
`T{tick}  {messaggio}`, 8 righe visibili, 60 in memoria, scorrimento
automatico sull'ultima. Cambiano solo:

- Posizione: barra inferiore a tutta larghezza (sez. 3.2), Bevy UI.
- Colore per gravità: righe di allarme (`Blackout…`, `Avaria…`, `Ossigeno
  esaurito`, `…asfissia`) in `#E84C3D`; avvisi (`Ossigeno critico`,
  `Surriscaldamento…`, `…fermo: equipaggio insufficiente`) in `#F2D24B`;
  tutte le altre in `#8C99A8`. La classificazione va fatta assegnando una
  gravità al momento del `push` (vedi sez. 10, punto 2), non con pattern
  matching sulle stringhe.

---

## 8. Menu e schermate

Scope onesto: PoC che diventa gioco. Niente opzioni video, niente salvataggi,
niente account, niente audio. Cinque schermate, gestite come stati dell'app
(in Bevy: `States`, `AppState { Titolo, ComeSiGioca, InGioco, FinePartita }`
+ flag per l'overlay di pausa).

### 8.1 Schermata Titolo

- Titolo del gioco ("Space Station") in grande, pixel font o font default.
- Tre voci: **Gioca** (nuova partita), **Come si gioca**, **Esci**.
- Navigazione: frecce su/giù + Invio, oppure click. Voce evidenziata con
  bordo `#D8DEE6`.
- Sfondo: nero spazio `#0B0E14`, opzionalmente qualche pixel-stella statica.

### 8.2 Come si gioca

Una singola pagina statica, testo + le 4 icone risorsa:

- Obiettivo in due righe.
- Tabella comandi (identica alla sez. 9).
- Le 4 risorse in una riga ciascuna.
- Le 6 miniature dei moduli con nome e valori principali.
- `Esc` o voce "Indietro" per tornare **alla schermata da cui si è arrivati**
  (Titolo o menu di pausa).

### 8.3 In gioco

La schermata descritta nelle sezioni 3–7. All'ingresso da "Gioca":
griglia vuota, `Sim::default()` (quindi in pausa: si costruisce senza
conseguenze), log vuoto, selezione = Reattore.

### 8.4 Menu di pausa (overlay)

Aperto con `Esc` durante il gioco. Overlay semitrasparente `#0B0E14` al 70%
sopra la scena di gioco (che resta visibile sotto).

- **Mentre l'overlay è aperto la simulazione è congelata**, indipendentemente
  da `sim.running`: il timer del tick non deve avanzare. Alla chiusura la
  sim riprende nello stato (`running` o pausa-costruzione) in cui era.
  `Esc` (pausa menu) e `Spazio` (pausa simulazione) sono due cose diverse e
  la Guida lo spiega.
- Voci: **Riprendi** (anche `Esc`), **Come si gioca**, **Ricomincia**,
  **Torna al titolo**, **Esci**.
- **Ricomincia** e **Torna al titolo** distruggono la stazione: richiedono
  una conferma inline sulla voce stessa (la voce diventa "Sicuro? La
  stazione andrà persa — Invio conferma, Esc annulla"). Niente finestre di
  dialogo separate.
- Ricomincia = stesso reset dell'ingresso da "Gioca" (8.3).

### 8.5 Fine partita

Quando la simulazione alza `partita_finita` (sez. 12.2) l'app passa allo
stato `FinePartita`: un overlay `#0B0E14` al 70% sopra la scena di gioco
(che resta visibile sotto, come per la pausa) con:

- Titolo in `#E84C3D`: "STAZIONE PERSA" (equipaggio azzerato) o "TEMPO
  SCADUTO" (tetto `TICK_MASSIMO` raggiunto, sez. 12.2), secondo
  `Sim.motivo_fine`.
- Il punteggio, i tick sopravvissuti, l'equipaggio massimo raggiunto.
- Due voci: **Ricomincia** e **Torna al titolo**. Qui **non** chiedono
  conferma: la stazione è già persa, chiederlo sarebbe assurdo. Stessi
  componenti (`Voce`/`Azione`) e stessi sistemi di navigazione degli altri
  menu, non duplicati.
- `Esc` non fa niente: si sceglie una voce.

### 8.6 Transizioni

```
Titolo ── Gioca ──────────────► In gioco
Titolo ── Come si gioca ──────► Come si gioca ── Esc ► Titolo
In gioco ── Esc ──────────────► Pausa (overlay)
In gioco ── equipaggio a 0 ───► Fine partita (overlay-stato)
Pausa ── Riprendi / Esc ──────► In gioco (sim riprende com'era)
Pausa ── Come si gioca ───────► Come si gioca ── Esc ► Pausa
Pausa ── Ricomincia (conferma)► In gioco (reset)
Pausa ── Torna al titolo (conferma) ► Titolo
Fine partita ── Ricomincia ───► In gioco (reset, senza conferma)
Fine partita ── Torna al titolo ► Titolo (senza conferma)
Titolo/Pausa ── Esci ─────────► chiusura applicazione
```

---

## 9. Comandi completi

| Input | Contesto | Azione |
|---|---|---|
| `1`–`6` | In gioco | Seleziona il modulo nella palette (Reattore, Life Support, Dormitorio, Laboratorio, Radiatore, Corridoio) |
| Click sinistro (griglia) | In gioco | Piazza il modulo selezionato nella cella libera sotto il cursore |
| Click sinistro (palette) | In gioco | Seleziona lo slot cliccato |
| Click destro | In gioco | Rimuove il modulo nella cella sotto il cursore |
| `R` | In gioco | Ripara il modulo in avaria sotto il cursore |
| `Spazio` | In gioco | Avvia/ferma la simulazione (in pausa si costruisce senza conseguenze; l'HUD mostra l'anteprima del bilancio) |
| Hover mouse | In gioco | Ghost di piazzamento sulla cella libera; pannello ispezione sul modulo puntato |
| `Esc` | In gioco | Apre il menu di pausa (congela tutto) |
| `Esc` | Menu/pagine | Chiude/torna indietro |
| Frecce su/giù + `Invio` | Menu | Naviga e conferma le voci |
| Click sinistro | Menu | Attiva la voce cliccata |

---

## 10. Proposte che toccano la simulazione (da approvare)

Nessuna regola di gioco viene cambiata da questa spec. Le seguenti sono
modifiche al **file** `sim.rs` che non alterano il comportamento simulato;
vanno approvate prima di implementarle, altrimenti si usano i ripieghi
indicati.

1. **Esporre produzione e consumo separati in `Sim`** (es. `energia_prod`,
   `energia_cons`, `o2_prod`, `calore_prod`, `calore_diss`), valorizzati in
   `sim_tick` dove le somme esistono già. Solo output aggiuntivo, zero
   cambi di regole. Ripiego senza toccare `sim.rs`: il sistema UI ricalcola
   le somme da `Query<&Module>` (sez. 4.4), accettando la duplicazione
   della definizione di "attivo".
2. **Aggiungere una gravità a `EventLog::push`** (`Info | Avviso | Allarme`)
   per colorare il log (sez. 7). `EventLog` vive in `sim.rs` ma è
   infrastruttura di presentazione. Ripiego: log monocromo come oggi.
3. **(Superata dalla sez. 12)** Il Corridoio era un puro costo (−1 energia)
   senza funzione finché l'adiacenza non contava; questa spec lo aveva
   mantenuto in palette dichiarandolo "decorativo, in attesa della v2".
   **Da quando le reti elettriche esistono (sez. 12.1) non è più vero**: il
   Corridoio è il connettore economico della rete. I suoi numeri non sono
   cambiati.

---

## 11. Fuori scope per questa iterazione

- Audio e musica.
- Salvataggi (anche singoli) e persistenza di qualunque tipo.
- Opzioni/impostazioni (video, rebind tasti, lingua).
- Animazioni degli sprite (frame multipli, particelle): gli unici effetti dinamici ammessi sono il lampeggio del badge avaria e il ghost.
- Obiettivi/missioni, tutorial interattivo.
- Generazione procedurale, eventi casuali oltre all'avaria da surriscaldamento esistente.
- Gamepad e touch.
- Qualsiasi modifica alle regole di `sim.rs` non elencata e approvata in sez. 10 o descritta in sez. 12.

L'adiacenza tra moduli e game over/punteggio erano in questa lista quando la
spec è nata; il playtest ("come porto energia? come faccio punti? quando
finisce?") li ha promossi a requisiti e sono stati implementati come
descritto nella sez. 12. Stessa sorte per obiettivi/missioni e per la
persistenza (limitata a classifica e progressione): promossi nella terza
iterazione e descritti nella sez. 13.

---

## 12. Reti elettriche e fine partita (implementate)

Estensione nata dal primo playtest della nuova UI: la giocatrice ha
costruito una fila di corridoi dal reattore convinta di stendere un cavo
(il gioco comunicava una topologia che non esisteva) e ha chiesto come si
fanno punti e quando finisce la partita. Queste regole — a differenza della
sez. 10 — **cambiano la simulazione**, e sono lo stato implementato in
`sim.rs`.

### 12.1 Reti elettriche (adiacenza)

- **Regola dei conduttori** (iterazione 6): l'energia si propaga solo
  attraverso i CONDUTTORI — **Reattore e Corridoio**. Gli altri moduli
  sono FOGLIE: appartengono a una rete se ortogonalmente adiacenti (4
  vicini, niente diagonali) ad almeno un suo conduttore, ma **non
  prolungano** la corrente: una fila di dormitori non è un cavo. La rete è
  la componente connessa dei conduttori (BFS seminata dai conduttori) più
  le foglie annesse; una foglia che tocca due reti finisce nella prima in
  ordine deterministico `(priorita, seq)`; le foglie senza conduttori sono
  isole scollegate.
- Una rete alimenta i suoi moduli **solo se contiene almeno un reattore non
  in avaria**. L'energia non passa da una rete all'altra.
- I conduttori in avaria **conducono comunque** corrente: sono ancora tubi,
  non buchi. Un reattore in avaria conduce ma non produce e non "accende"
  la sua rete.
- Dentro ogni rete l'allocazione è quella di sempre: prima si sommano
  **tutti** i produttori della rete, poi si distribuisce ai consumatori in
  ordine `(priorita, seq)`. Il pre-passaggio dei produttori è la correzione
  di un bug reale della PoC e non si torna indietro.
- Un modulo in una rete senza reattore funzionante non è in blackout: è
  **scollegato** (`Fermo::Scollegato`, priorità sopra `Energia` e sotto
  `Avaria`). Due problemi, due rimedi: scollegato si risolve costruendo un
  collegamento, il blackout aggiungendo produzione.
- Resa visiva: badge fulmine **tinto grigio** `#8C99A8` per scollegato (lo
  sprite è lo stesso del blackout, cambia solo la tinta), fulmine giallo
  pieno per il blackout. Pannello ispezione: `SCOLLEGATO — nessun reattore
  collegato` in giallo.
- Log: `Avviso` quando un modulo passa a scollegato, `Info` quando torna
  collegato. In pausa (anteprima) non si scrive nulla, come per il resto
  del log. I moduli in avaria non loggano il cambio di collegamento: il
  loro problema dichiarato è l'avaria.
- **Ossigeno, calore ed equipaggio restano a livello di stazione**: la
  stazione è una sola, l'adiacenza governa solo l'energia.
- Il Corridoio (−1 energia) non è più opzionale: è metà della regola dei
  conduttori — la dorsale di ogni stazione. I suoi numeri non sono
  cambiati; il suo peso strategico sì (e `fabbisogno_minimo` nel
  generatore ora conta anche i corridoi obbligati).

### 12.2 Punteggio e fine partita

- `Sim` tiene `punteggio: u64` e `equipaggio_max: u32`. A ogni tick
  effettivo (non in anteprima) `punteggio += equipaggio`: il punteggio è
  **persone·tick** e cresce tanto più in fretta quanto più equipaggio resta
  in vita. `equipaggio_max` è il massimo storico.
- **Fine partita**, due cause possibili, entrambe alzano `partita_finita`
  (il tick non gira più a flag alzato) e sono distinte in `Sim.motivo_fine`:
  1. L'equipaggio arriva a 0 dopo essere stato almeno una volta sopra 0
     (`equipaggio_max > 0`): `MotivoFine::EquipaggioMorto`, schermata
     "STAZIONE PERSA".
  2. Il tick raggiunge il tetto `Sim.tetto_tick` (quando presente):
     `MotivoFine::TempoScaduto`, schermata "TEMPO SCADUTO". Tetto di
     sicurezza aggiunto dopo un playtest in cui una stazione bloccata in
     una spirale di asfissia lenta (un morto ogni `TICK_MORTE` tick, senza
     altre conseguenze) si è trascinata per centinaia di tick oltre
     l'obiettivo del livello prima di risolversi da sola: senza tetto, una
     partita persa in sostanza può restare aperta per un tempo
     imprevedibile.
  In entrambi i casi il log scrive una riga di `Allarme`.
- **`Sim.tetto_tick: Option<u64>`**: `None` in Infinita (nessun limite,
  come da spirito "resisti quanto vuoi" della modalità), `Some(400)`
  (`sim::TICK_MASSIMO`) in Sfida e in Campagna — 400 è ampiamente sopra il
  requisito più lungo dichiarato dai livelli campagna (100 tick per
  "Colonia"), quindi non stringe le partite già vincibili. Impostato da
  `applica_reset` in `main.rs` secondo `Modalita`, non dentro `sim.rs`: la
  simulazione non sa cos'è una modalità, riceve solo il numero (o niente).
- L'HUD mostra il tetto come **timer** `TEMPO m:ss` (countdown in tempo
  reale: bianco, giallo sotto il 25% residuo, rosso sotto il 10%; col
  suffisso ` ×2`/` ×4` quando la velocità di gioco non è ×1), altrimenti
  `TICK n`; e `PUNTI n` in fondo alla barra superiore, discreto: è un
  numero che si guarda a fine partita, non il protagonista dello schermo.
- La schermata di fine partita è descritta nella sez. 8.5; il titolo e il
  sottotitolo cambiano secondo `motivo_fine` (vedi `menu::entra_fine`).

---

## 13. Gamification: modalità, livelli, classifica (implementata)

Terza iterazione: il gioco validato dalla sez. 12 riceve una struttura di
obiettivi. Il codice sta in `src/livelli.rs`; la simulazione non cambia
regole (unica aggiunta a `Sim`: il contatore di solo output
`avarie_surriscaldamento`, incrementato dove l'avaria già avveniva).

### 13.1 Cinque modalità

| Modalità | Cos'è | Tetto di tick | Punteggio |
|---|---|---|---|
| **Campagna** | Sei livelli in sequenza, ognuno con un obiettivo misurabile. Si vince il livello raggiungendo l'obiettivo; si perde con la stazione o a tetto di tick raggiunto. Completare un livello sblocca il successivo. | Sì (400) | Mostrato nell'HUD ma **non** entra in classifica |
| **Infinita** | Sandbox senza obiettivi, con la fine partita esistente: si resiste quanto si riesce. | No | Sorgente della classifica **Infinita** |
| **Sfida** | Come Infinita — nessun obiettivo, si punta al punteggio — ma con lo stesso tetto della Campagna: partite più brevi, esito garantito entro un tempo fisso. | Sì (400) | Sorgente della classifica **Sfida** |
| **Casuale** | Un livello generato al momento (obiettivo attivo come in campagna), fuori da progressione e classifiche. | Sì (400) | Solo a schermo |
| **Sfida del giorno** | Il livello del giorno, uguale per tutti (seed dalla data via `genera_giornaliera`): fuori classifiche, conta il miglior tempo personale del giorno (`progressi.txt`). | Sì (400) | Miglior tempo locale |

Aggiunta dopo un playtest in cui una partita Infinita persa in sostanza
(spirale di asfissia lenta) si è trascinata per centinaia di tick senza
concludersi: chi preferisce sapere che la partita finisce comunque entro
un tempo fisso ha Sfida; chi vuole vedere fin dove riesce a spingersi senza
limiti resta su Infinita. **Le due classifiche restano separate** (sez.
13.4): un punteggio Infinita non è confrontabile con uno Sfida, il primo
può sempre vincere solo restando in piedi più a lungo.

La risorsa `Modalita` (`Infinita` / `Sfida` / `Campagna(indice)` / `Casuale`) dice in
che modalità si sta giocando; il sistema degli obiettivi gira **solo** in
Campagna (run condition `campagna_attiva`) — Sfida non ha obiettivi, come
Infinita.

### 13.2 I livelli: 6 curati + 44 generati (50 totali)

`LIVELLI` in `src/livelli.rs` è ora un `LazyLock<Vec<LivelloDef>>`: i primi
6 livelli sono curati a mano (tabella qui sotto, invariata), dal 7 al 50
arrivano da `src/generatore.rs` con **seed fisso derivato dall'indice** —
il livello 23 è identico per tutti e a ogni avvio, quindi si può
bilanciare guardandolo. Il generatore scala obiettivo, detriti (quota
0→12, pattern: sparsi/muro/diagonale/croce) e budget moduli (fabbisogno
minimo × margine 1,6→1,15) sull'indice; il PRNG è uno splitmix64 interno,
non la crate `rand`, così la sequenza non cambia mai con un aggiornamento
di dipendenza. **Garanzia di risolvibilità testata a ogni build**: budget
≥ `fabbisogno_minimo(obiettivo)` (stessa funzione usata dal generatore) e
area libera ortogonalmente connessa ≥ budget (flood fill), su tutti i 50
livelli più 200 seed casuali.

C'è anche una modalità **Livello casuale** (voce del titolo): stesso
generatore, seed dal rand di sistema, difficoltà pescata nella fascia
10–40 della curva. Obiettivo attivo come in campagna ma fuori da
progressione e classifiche; a livello completato la schermata propone
"Nuovo livello casuale", a stazione persa "Riprova il livello" rigioca lo
stesso seed (il livello resta nella risorsa `LivelloCasuale`).

I sei livelli curati — tabella di costanti sul modello di `TABELLA` in
`modules.rs`, è lì che si tarano nomi, briefing e numeri. I testi degli
obiettivi sono generati dai numeri (`Obiettivo::descrizione`).

| # | Nome | Briefing | Obiettivo | Moduli max |
|---|---|---|---|---|
| 1 | Primo respiro | Un reattore, un life support, un dormitorio: attaccati tra loro. | avere 4 di equipaggio a bordo | 6 |
| 2 | La rete | Un solo reattore non basta più: allunga con i corridoi o costruisci una seconda rete. | avere 8 di equipaggio a bordo | 12 |
| 3 | Sala macchine | I laboratori non vogliono corrente: vogliono gente. | tenere 2 laboratori attivi per 15 tick consecutivi | 13 |
| 4 | Termica | Tutto quello che lavora scalda. | sopravvivere 60 tick con almeno 2 laboratori attivi e zero avarie da surriscaldamento | 14 |
| 5 | Autonomia | Il margine serve a questo: a non restare mai al buio. | raggiungere 400 punti senza mai andare in blackout | 15 |
| 6 | Colonia | Adesso tienila in piedi davvero. | avere 12 di equipaggio e sopravvivere 100 tick | 18 |

Semantica:

- "Laboratorio attivo" = la definizione di `Module::attivo()` più `staffed`.
- Gli obiettivi "per N tick consecutivi" usano un contatore nello **stato del
  livello** (`StatoLivello`, non in `Sim`) che si azzera quando la condizione
  decade.
- Nei livelli 4 e 5 la condizione accessoria (avaria da surriscaldamento /
  blackout) **azzera il progresso, non fa perdere la partita**: perdere si
  perde solo con la stazione.
- L'obiettivo avanza una volta per tick effettivo (mai in anteprima) e la
  fine partita vince sempre sul completamento.
- **Limite moduli** (`max_moduli` in `LivelloDef`): ogni livello ha un
  budget di moduli costruibili, corridoi inclusi, tarato sull'obiettivo —
  fabbisogno minimo della soluzione ovvia più margine per corridoi ed
  errori (i minimi sono nei commenti della tabella `LIVELLI`). A budget
  esaurito il click di costruzione non fa nulla e il log lo dice
  ("Limite moduli raggiunto (N): rimuovi qualcosa col tasto destro"); il
  ghost sparisce. Rimuovere un modulo libera il posto: conta ciò che è
  sulla griglia, non ciò che è stato costruito in totale. Il budget è
  dichiarato nel briefing e nel log di inizio livello. Solo campagna:
  Infinita e Sfida restano senza limite.
- HUD: in campagna la barra superiore mostra obiettivo, progresso e budget
  moduli (es. `OBIETTIVO equipaggio 5/8   moduli 7/12`).

### 13.3 Schermate

Stessi componenti (`Voce`/`Azione`) e stessi sistemi di navigazione dei menu
esistenti, non duplicati.

- **Titolo**: Campagna, Infinita, Sfida, Livello casuale, Classifica,
  Come si gioca, Esci.
- **Selezione livello**: griglia 10×5 di celle numerate (completato =
  numero col punto, disponibile = navigabile, bloccato = testo spento non
  selezionabile); la selezione parte dal primo livello non completato.
  Frecce su/giù saltano di riga (±10), sinistra/destra di cella; Invio
  apre il briefing. Esc/Indietro torna al titolo.
- **Briefing**: nome, briefing e obiettivo per esteso + "Inizia".
- **Classifica**: due colonne affiancate, Infinita e Sfida, ciascuna con la
  propria top 10 (posizione, punteggio, tick, equipaggio massimo, quanto
  tempo fa); se una è vuota lo dice solo lei ("nessuna partita
  registrata"), l'altra resta indipendente.
- **Livello completato**: obiettivo raggiunto, punteggio e tick; voci
  "Livello successivo" (assente all'ultimo livello) e "Torna al titolo".
- **Fine partita**: titolo "STAZIONE PERSA" o "TEMPO SCADUTO" secondo la
  causa (sez. 12.2, `motivo_fine`). In campagna le voci diventano "Riprova
  il livello" e "Torna al titolo"; in Infinita/Sfida restano com'erano e,
  se il punteggio entra in top 10, la schermata lo dice, specificando la
  classifica ("Nuovo record: N° posto in classifica Sfida").

### 13.4 Persistenza

Tre file di testo semplice (niente serde, niente dipendenze nuove) in
`$XDG_DATA_HOME/space-station/`, con ripiego
`$HOME/.local/share/space-station/`; le cartelle si creano se mancano.
Scrittura **solo** a fine partita / completamento livello, mai a ogni tick.

- `classifica.txt` (Infinita) e `classifica_sfida.txt` (Sfida): stesso
  formato, un record per riga, campi separati da TAB:
  `punteggio  tick  equipaggio_max  epoch_secs`. La data è l'epoch Unix in
  secondi, mostrata come "oggi / ieri / N giorni fa" (nessuna crate di
  date). Parsing a mano con `split('\t')`: **una riga malformata si salta**
  senza far crashare il gioco; file assente o illeggibile = classifica
  vuota. Top 10, ordinata per punteggio decrescente. Due risorse Bevy
  distinte (`ClassificaInfinita`, `ClassificaSfida`), entrambe newtype
  attorno alla stessa struct `Classifica` — file diverso, stesso codice di
  lettura/scrittura/ordinamento.
- `progressione.txt`: il numero (1-based) dell'ultimo livello completato.
  File assente o illeggibile = si riparte dal livello 1, senza errori a
  schermo.

### 13.5 Corridoio orientato automaticamente

Il corridoio sceglie sprite e rotazione dalla maschera dei 4 vicini
ortogonali, e il calcolo guarda **qualsiasi modulo adiacente** (non solo
altri corridoi): la grafica comunica esattamente la regola dell'adiacenza
elettrica (sez. 12.1). Sprite (in `assets/sprites/moduli/`): `corridoio.png`
(orizzontale), `corridoio_v.png` (verticale), `corridoio_curva.png` (curva
che collega destra e basso), `corridoio_t.png` (T che collega sinistra,
destra e basso), `corridoio_croce.png` (incrocio). Rotazioni con
`Transform::rotation` a passi di 90° (l'art è a pixel quadrati, non si
sfoca):

| Vicini | Sprite |
|---|---|
| 0 o 1, nessuno verticale | orizzontale |
| solo verticali | verticale |
| 2 ad angolo | curva ruotata |
| 3 | T ruotata |
| 4 | croce |

Il ricalcolo copre anche i corridoi **vicini** quando si piazza o si rimuove
un modulo qualsiasi (il sistema rilegge l'occupazione della griglia a ogni
frame e scrive solo i cambiamenti).

### 13.6 Detriti (ostacoli)

Celle occupate da un detrito su cui non si può costruire e che non si può
rimuovere: la stazione ci si costruisce intorno. Regole:

- Esistono **solo in campagna**: il layout è il campo `ostacoli` di
  `LivelloDef` (coordinate cella 0-based, origine in basso a sinistra),
  quindi si tara in `LIVELLI` come tutto il resto. La modalità Infinita
  resta a griglia libera.
- La **simulazione non li vede**: nessuna modifica a `sim.rs`, sono un puro
  vincolo di piazzamento (`Station::ostacoli`) più uno sprite. Bloccano
  click e ghost esattamente come una cella già occupata; non compaiono nel
  pannello ispezione.
- Distribuzione didattica: entrano al livello 2 (dove nasce il problema di
  portare energia lontano) e crescono col numero di livello; i livelli che
  introducono un meccanismo nuovo (1, 3, 5) restano senza, per non sommare
  due difficoltà. Layout attuali: livello 2 muro verticale centrale
  aggirabile, livello 4 cinque detriti sparsi, livello 6 fascia diagonale.
- All'avvio di un livello con detriti il log lo annuncia ("Detriti sulla
  griglia: costruisci intorno").
- Sprite: `assets/sprites/ostacolo.png` (mappa `OSTACOLO` in
  `tools/gen_sprites.py`): blob roccioso irregolare, solo colori neutri,
  nessun lato dritto completo — deve dire "non è un edificio" a colpo
  d'occhio, in contrasto con i moduli geometrici.
