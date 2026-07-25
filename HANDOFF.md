# Handoff — Space Station

> Fotografia dello stato del progetto al **25 luglio 2026**. Non aggiornare
> questo documento in place quando la situazione cambia: scrivine uno nuovo
> (o riscrivilo daccapo) alla prossima sessione di consegna, perché tutto
> quello che segue — in particolare "stato di verifica" e "prossimi passi" —
> è vero solo a questa data.

## Cos'è

PoC/iterazione 2 di un gestionale di stazione spaziale: si piazzano moduli
su una griglia, si avvia una simulazione a tick che bilancia quattro
risorse (energia, ossigeno, calore, equipaggio), e uno sbilancio innesca
una cascata di guasti a 4 stadi (blackout → life support giù → ossigeno
giù → equipaggio che muore). Rust + Bevy 0.19.0, edizione 2024, dipendenze:
`bevy` e `rand` (0.10.2). Nessun asset esterno scaricato: gli sprite sono
generati da uno script Python proprietario (vedi sotto).

Il progetto nasce come una delle idee da "cernita" nel portale progetti
dell'utente (id 48, batch "Desktop sim trio"): l'obiettivo è capire se
l'idea regge prima di investire oltre.

### A che punto è

Due fasi completate:

1. **PoC** (`POC.md`): griglia, 6 moduli, 4 risorse, cascata di guasti, log
   a schermo. Giocata dall'utente: la meccanica di bilancio/cascata è
   piaciuta, la UI (quadratini colorati, testo `Text2d`) no — giudicata
   "troppo astrusa da capire". Il log invece ha funzionato ed è stato
   tenuto.
2. **Iterazione 2** (`SPEC.md` + `GUIDA.md`, questa sessione): riscritta la
   UI con Bevy UI ancorata ai bordi finestra, generati 13 sprite pixel art,
   aggiunti i menu (titolo, guida, pausa). La simulazione (`sim.rs`) non è
   stata ridisegnata, solo estesa con output aggiuntivi (vedi "Decisioni").

**La nuova UI non è ancora stata giudicata dall'utente.** È il prossimo
passo, prima di qualunque altro lavoro su questo progetto.

## Come si avvia

```sh
cargo run
```

Prima build: compila Bevy, qualche minuto. Successive: rapide.

Verificato in questa sessione (25/07/2026):
- `cargo build` pulito (rimosso l'incrementale e ricompilato): **0 errori,
  0 warning**.
- Il binario compilato (`target/debug/space-station`) lanciato **da una
  directory diversa dalla radice del progetto** parte, crea la finestra,
  inizializza il renderer Vulkan e non produce errori di asset mancanti in
  4 secondi di esecuzione osservata — verifica diretta del fix descritto
  sotto "Decisioni".

Verifica sprite (facoltativa, richiede solo Python 3 stdlib):

```sh
python3 tools/gen_sprites.py    # rigenera i 13 PNG in assets/sprites/
python3 tools/check_sprites.py  # test di leggibilità silhouette (SPEC §2.1)
```

`check_sprites.py` eseguito in questa sessione: nessuna coppia di moduli
sopra la soglia IoU 80% (la più alta è reattore/life_support al 73,7%,
sotto soglia ma la più vicina — vedi Debolezze).

## Mappa dei file

| File | Ruolo | Si tocca per |
|---|---|---|
| `src/sim.rs` | Cuore della simulazione: tick di bilancio, cascata di guasti, `EventLog`. **Validato dalla PoC, non si ridisegna** (vedi `SPEC.md` riga 3-4). | Cambiare le costanti di ritmo in testa al file (`TICK_SECS`, `OSSIGENO_PER_CREW`, `TICK_SURRISCALDAMENTO`, `TICK_MORTE`, `TICK_ARRIVO`, `SOGLIA_O2_CRITICO`). **Non toccare** l'ordine delle fasi nel tick né la logica di allocazione energia senza motivo forte: c'è un bug già corretto lì (vedi sotto) e la sequenza è delicata. |
| `src/modules.rs` | `TABELLA`: **unico punto** dove si tara produzione/consumo per tick di ogni modulo (energia, ossigeno, calore, posti letto, equipaggio richiesto, priorità di spegnimento). | Bilanciamento: qualunque numero di gioco (costi, produzioni, priorità) si cambia solo qui. |
| `src/main.rs` | Griglia in world-space, piazzamento/rimozione moduli, stati visivi (overlay+badge), resize/scala della griglia, cablaggio dei sistemi Bevy, calcolo del percorso `assets/` a runtime. | Interazione con la griglia (click, ghost, layout). Contiene anche `percorso_assets()`: **non rimuoverla**, è il fix per l'esecuzione fuori da `cargo run` (vedi Decisioni). |
| `src/ui.rs` | HUD (produzione/consumo per le 4 risorse), palette moduli, pannello ispezione, log eventi — tutto in Bevy UI (`Node`/flexbox), ancorato ai bordi finestra. Definisce anche la palette dei 16 colori (`SPEC.md` §2.2) come costanti pubbliche. | Layout e testo della UI. I colori vanno presi da qui (`ui::VERDE`, `ui::ROSSO`, ecc.), mai duplicati. |
| `src/menu.rs` | Le 4 schermate/stati (`AppState`: Titolo, ComeSiGioca, InGioco) e l'overlay di pausa (`Pausa`, separato dallo stato App perché la scena di gioco deve restare visibile sotto). Navigazione da tastiera e mouse, conferme inline per azioni distruttive. | Aggiungere/modificare voci di menu, testi della schermata guida in-game. |
| `tools/gen_sprites.py` | Genera i 13 PNG (6 moduli 32×32, 3 badge 8×8, 4 icone 8×8) da mappe ASCII incorporate nello script, zero dipendenze esterne (solo `zlib`/`struct`). | **Qui e solo qui** si ritoccano gli sprite: si modifica la mappa ASCII del modulo interessato e si rilancia lo script. Non editare i PNG a mano — si perderebbe la fonte. |
| `tools/check_sprites.py` | Calcola riempimento cella e sovrapposizione (IoU) di silhouette fra coppie di moduli; soglia di fallimento all'80% di IoU. | Verifica dopo ogni modifica agli sprite. |
| `assets/sprites/{moduli,badge,icone}/*.png` | Output generato da `gen_sprites.py`. 13 file, ~52 KB totali. | Mai a mano (vedi sopra). |
| `POC.md` | Specifica originale della PoC (fase 1). Storico: descrive uno scope "nessun asset esterno" ormai superato dalla pixel art. | Riferimento storico, non normativo per lo stato attuale. |
| `SPEC.md` | Specifica prescrittiva dell'iterazione 2: art direction, palette 16 colori, layout schermo, HUD, menu, comandi. **Documento di riferimento per "perché la UI è fatta così".** | Consultare prima di ogni modifica a `ui.rs`/`menu.rs`/sprite: la sezione 2 fissa i vincoli pixel art, la 3-9 il layout, la 10 elenca le modifiche a `sim.rs` approvate. |
| `GUIDA.md` | Guida giocatore in linguaggio naturale (obiettivo, comandi, tabella moduli, esempio passo-passo della cascata). Non normativa per l'implementazione ma buon controllo di coerenza con `sim.rs`. | Aggiornare se cambiano numeri o regole visibili al giocatore. |
| `README.md` | **Obsoleto**: descrive ancora lo stato della sola PoC ("nessun asset esterno", nessuna menzione di `menu.rs`/`ui.rs` riscritta o degli sprite). Non riflette l'iterazione 2. | Da riscrivere quando la UI sarà validata dall'utente (non prioritario prima). |

## Decisioni prese e perché

- **Bug corretto in `sim.rs` (fase PoC, riportato qui perché il codice
  attuale lo documenta ancora in un commento a riga 210-212)**:
  l'allocazione energia sommava i produttori man mano che li incontrava
  nell'ordine `(priorità, seq)`. Reattore e Life Support hanno entrambi
  `priorita: 0`: costruire il Life Support prima del reattore lo mandava
  in blackout pur essendoci corrente sufficiente. Corretto sommando
  **tutti** i produttori in un pre-passaggio prima di distribuire
  l'energia (righe 213-217 di `sim.rs`).
- **Le 3 proposte di `SPEC.md` §10 sono state approvate e implementate**,
  nessuna cambia le regole simulate:
  1. `Sim` espone produzione/consumo come campi separati (`energia_prod`,
     `energia_cons`, `o2_prod`, `o2_cons`, `calore_prod`, `calore_diss`,
     ecc.), valorizzati in `sim_tick` dove le somme esistevano già. Prima
     l'HUD avrebbe dovuto ricalcolarle duplicando la definizione di
     "attivo" fuori da `sim.rs`.
  2. `EventLog::push` prende una `Gravita` (`Info | Avviso | Allarme`)
     assegnata alla sorgente dell'evento, non dedotta a valle con pattern
     matching sulle stringhe del messaggio. Usata da `ui::update_log` per
     colorare le righe.
  3. Il Corridoio resta in palette, dichiarato "decorativo, in attesa
     della v2" nella Guida — non è stato rimosso (era l'alternativa
     proposta). Vedi Debolezze: resta un puro costo senza funzione finché
     l'adiacenza non esiste.
- **Fix del percorso assets a runtime** (`main.rs`, `percorso_assets()`):
  Bevy cerca `assets/` accanto all'eseguibile e ripiega su
  `CARGO_MANIFEST_DIR` solo quando lanciato via `cargo run`. Lanciando il
  binario compilato direttamente, tutti gli sprite risultavano mancanti.
  Ora si prova prima la cartella accanto all'eseguibile (build
  distribuita), poi quella del sorgente (sviluppo). Verificato in questa
  sessione (vedi "Come si avvia").
- **Art direction = pixel art generata via script**, non pack CC0 né
  sprite disegnati a mano: decisione dell'utente. Il vantaggio dichiarato
  è che la fonte (mappe ASCII in `gen_sprites.py`) è testuale, versionabile
  e ritoccabile senza editor grafico.
- **Sequenza di lavoro scelta**: specifiche (`SPEC.md`/`GUIDA.md`) prima di
  tutto, poi sprite e menu in parallelo, handoff per ultimo — a valle di
  codice e asset, non a monte.

## Stato di verifica onesto

Provato in questa sessione:
- Build pulita: 0 errori, 0 warning.
- Avvio del binario compilato da directory diversa dalla radice: nessun
  crash, nessun errore di asset in 4s di esecuzione osservata.
- `tools/check_sprites.py`: nessuna coppia di sprite sopra la soglia di
  ambiguità (IoU 80%).

Non provato, e da fare come primo passo:
- **Nessuna sessione di gioco con la nuova UI.** Non si sa se l'HUD
  produzione/consumo, il pannello ispezione, i badge di stato e i menu
  siano effettivamente più leggibili di quanto lo fosse la PoC per un
  giocatore reale: è un'ipotesi di design, non un fatto verificato.
- Nessun test automatico esiste nel progetto (nessun `#[test]`, nessuna
  cartella `tests/`): tutta la verifica finora è stata build + lettura
  codice + ispezione visiva mancante.
- Il bilanciamento numerico (tabella moduli, soglie di colore, tempi della
  cascata) non è mai stato tarato giocando per davvero con la nuova UI: i
  valori sono ancora quelli di primo tentativo della PoC.

## Debolezze note

- **Equipaggio e posti letto**: se i dormitori si spengono (es. blackout),
  l'equipaggio in eccesso rispetto ai posti letto rimanenti non muore e
  non se ne va — semplicemente blocca i nuovi arrivi (`sim.rs`, condizione
  `sim.equipaggio < sim.posti_letto`). È il punto in cui il modello
  tradisce lo spirito "la cascata ha conseguenze reali": perdere i
  dormitori dovrebbe fare più male di così.
- **Surriscaldamento binario**: `sim.surriscaldamento` conta i tick
  consecutivi con `calore_netto > 0`, senza accumulo termico — un netto di
  +1 e uno di +60 pesano allo stesso modo (6 tick e via, sempre). Primo
  candidato al cambiamento se il feel del calore non convince in gioco.
- **Sprite**: verificato con `check_sprites.py` in questa sessione — il
  dormitorio riempie il 74,6% della cella (il vincolo del 70% di `SPEC.md`
  §2.3 è rispettato) ma resta il più debole *visivamente*: può leggersi
  come un pannello con due barre viola piuttosto che come un letto a
  castello. Laboratorio (50,9%) e corridoio (44,1%) **non rispettano** il
  vincolo di riempimento del 70% richiesto da `SPEC.md` §2.3, perché quel
  vincolo confligge con le silhouette prescritte nella stessa sezione
  ("banco basso", "tubo piatto" sono per natura sottili): a parità di
  conflitto ha prevalso la silhouette sul riempimento. La coppia di moduli
  con la maggiore sovrapposizione di sagoma è reattore/life_support, IoU
  73,7% — sotto la soglia di fallimento (80%) ma la più vicina, quindi la
  prima da controllare se in gioco si confondono.
- **Il Corridoio non ha funzione di gioco**: finché l'adiacenza fra moduli
  non esiste (rimandata alla v2, `SPEC.md` §11), è puro costo (−1
  energia/tick) senza contropartita. Il giocatore non ha motivo di
  costruirlo.
- **Bilanciamento mai tarato giocando davvero** con la UI nuova: i valori
  della tabella moduli (`modules.rs`) e le soglie di colore (`ui.rs`,
  `SPEC.md` §4.3) sono un primo tentativo, ereditato dalla PoC.
- **`README.md` è obsoleto**: descrive solo lo stato della PoC fase 1.

## Prossimi passi

In attesa di un giudizio dell'utente (non si può decidere senza):

1. **Far giocare la nuova UI all'utente e raccogliere il verdetto** — è il
   blocco che ha aperto questa iterazione (la vecchia UI "astrusa") e
   nessun lavoro ulteriore ha senso prima di sapere se il problema è
   risolto. In particolare va verificato se le quattro priorità di
   leggibilità elencate in `SPEC.md` (identità del modulo, stato esplicito
   col motivo, HUD prod/cons, sprite) funzionano nella pratica.
2. Se il verdetto è positivo: decidere se e come tarare il bilanciamento
   (tabella moduli, soglie, tempi) sulla base del gioco reale, non più a
   tavolino.
3. Se il verdetto individua problemi specifici sugli sprite: usare
   `tools/gen_sprites.py` + `tools/check_sprites.py` per iterare (in
   particolare dormitorio, e la coppia reattore/life_support se si
   confondono a schermo).

Già decisi, eseguibili senza aspettare (ma probabilmente da fare *dopo*
il punto 1, per non lavorare alla cieca):

4. Decidere una correzione per la debolezza equipaggio/posti-letto (es.
   far morire o far emigrare l'equipaggio in eccesso quando i posti letto
   calano) — è un cambio di regola simulata, quindi richiede la stessa
   approvazione esplicita che `SPEC.md` §10 ha chiesto per le altre
   modifiche a `sim.rs`.
5. Valutare un modello di calore con accumulo invece del contatore binario
   di tick consecutivi.
6. Aggiornare `README.md` per riflettere lo stato reale del progetto
   (menu, UI Bevy, pixel art) — rimandabile, non blocca nulla.
7. Prima v2 (fuori scope per ora, solo se il progetto continua oltre
   questa iterazione): adiacenza fra moduli, che darebbe finalmente una
   funzione al Corridoio.
