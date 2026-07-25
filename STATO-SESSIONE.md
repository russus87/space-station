# Stato sessione — 25 luglio 2026, sera

## Attività in corso
Nessuna: si attende che l'utente legga `SPEC-CAMPAGNA.md` e approvi/modifichi
le sezioni ⚠ (sblocco moduli, personaggi bonus, punti laboratorio).

## Appena completato
- **Playtest utente sul livello 6 "Colonia"**: confermata fattibilità con 18
  moduli (minimo 11, spiegato all'utente); emerso che il laboratorio, fuori
  dai livelli 3-4, non serve a nulla — difetto di design riconosciuto.
  Proposta di correzione (+5 punti/tick per laboratorio attivo) non ancora
  implementata, ora tracciata in `SPEC-CAMPAGNA.md` sez. 5.
- **Tasto "MENU esc" nell'HUD**, sempre visibile in alto a destra, perché
  l'utente non trovava come uscire dalla partita (il menu era raggiungibile
  solo con Esc, non scopribile). Apre l'overlay di pausa già esistente.
  `src/ui.rs`: componente `BottoneMenu` (riga 40) + sistema
  `click_bottone_menu` (riga 608); registrato in `src/main.rs` (riga 263).
  Durante l'edit un inserimento aveva spezzato il `#[derive]` di `CampoHud`
  (struct finita tra derive ed enum); corretto subito — promemoria per stare
  attenti agli edit vicino ad attributi.
- **Sessione di design campagna lunga** con l'utente (~50 livelli lineari,
  sblocchi ogni 5 livelli, eventuale modalità "livello random" e
  intermezzi/storia). Con AskUserQuestion ha scelto: approccio IBRIDO
  (generatore parametrico a seed fisso + livelli curati ai punti chiave),
  sblocchi SIA moduli SIA personaggi alternati, e DESIGN DOC prima del
  codice.
- **`SPEC-CAMPAGNA.md`** scritto (nuovo file, ~180 righe, marcato "PROPOSTA
  in attesa di approvazione"; rimando aggiunto in testa a `SPEC.md`).
  Contenuto: 10 blocchi da 5 livelli con tema e sblocco a fine blocco;
  generatore parametrico (obiettivo/detriti/budget scalati, seed
  deterministico per livello, risolvibilità garantita via fabbisogno_minimo
  condiviso + flood fill); griglia selezione 10×5; livello casuale con 3
  difficoltà; 5 moduli nuovi con numeri esatti (Batteria liv.5, Serra
  liv.15, Gru liv.25, Condotto termico liv.35, Centro comando liv.45); 5
  personaggi, uno schierabile per partita (Ingegnere liv.10, Medico liv.20,
  Caposquadra liv.30, Scienziata liv.40, Comandante liv.50); regola base +5
  punti/tick per laboratorio attivo; intermezzi diario di bordo ogni 10
  livelli; persistenza invariata (sblocchi derivati da progressione.txt);
  piano in 5 fasi con gate di approvazione ⚠; rischi dichiarati (Batteria
  tocca l'allocazione energia = zona delicata, bilanciamento a tavolino, 50
  livelli non è sacro — tagliare a 30 se il ritmo crolla).
- **Aggiunta sez. 9** a `SPEC-CAMPAGNA.md` (i vecchi Rischi diventano sez.
  10), in risposta alla domanda dell'utente "si può migliorare ancora?":
  "Oltre la campagna: profondità, non solo ampiezza", autocritica che il
  piano moltiplica contenuto ma non decisioni durante la partita. Sei
  estensioni con priorità/costo: 9.1 eventi con scelta (⚠, da osservatore a
  comandante), 9.2 riparazione moduli con equipaggio impegnato (⚠), 9.3
  velocità 1×/2×/4×, 9.4 stelle per livello (1-3, `stelle.txt`), 9.5 audio
  generato via script Python, 9.6 sfida del giorno (seed dalla data). Più
  lista esplicita di cosa NON aggiungere (valute, online, altre modalità).
  Collocazione: 9.3/9.4 in fase 2, 9.1/9.2 come fase 6 a sé, 9.5/9.6
  indipendenti. Nessun codice toccato.

## Prossimo passo immediato
Attendere che l'utente legga `SPEC-CAMPAGNA.md` e approvi/modifichi le
sezioni ⚠ 3 (moduli), 4 (personaggi), 5 (punti laboratorio). Se approvato,
la fase 1 del piano (sez. 8) è "punti laboratorio": piccola e immediata.

## Passi successivi
1. Punti laboratorio (+5/tick per lab attivo) — se approvata la sez. 5.
2. Generatore parametrico + campagna 50 livelli + modalità livello casuale.
3. Moduli sbloccabili uno alla volta (Batteria per ultima, tocca l'energia).
4. Personaggi bonus, uno schierabile per partita.
5. Intermezzi diario di bordo ogni 10 livelli.
6. Residui da sessioni precedenti, ancora validi: bilanciamento budget
   `max_moduli` dei livelli attuali (tarati a tavolino, ora confermati
   giocabili da playtest su L6); debolezza equipaggio/posti-letto
   (equipaggio in eccesso dopo blackout dormitori non muore né emigra);
   calore con accumulo invece del contatore binario `calore_netto > 0`.

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` — 0 errori, 0 warning;
  `cargo test --quiet` — 14/14 passati.
- Verificato io stesso, ora, nel codice: `BottoneMenu` in `src/ui.rs` (righe
  40 e 262), `click_bottone_menu` in `src/ui.rs` (riga 608) registrato in
  `src/main.rs` (riga 263); `#[derive(Component, Clone, Copy)]` di
  `CampoHud` correttamente seguito dall'enum (righe 42-43), nessuna struct
  frapposta — la correzione riferita è confermata sul filesystem.
- Verificato io stesso, ora: `SPEC-CAMPAGNA.md` esiste, riga 3 marcata
  "STATO: PROPOSTA, in attesa di approvazione", sezioni 1-10 presenti con
  gli header attesi (righe 15-209: generatore, sblocchi moduli/personaggi,
  punti lab, intermezzi, piano, sez. 9 "Oltre la campagna" con le sei
  estensioni 9.1-9.6, sez. 10 Rischi); `SPEC.md` riga 16 contiene il
  rimando a `SPEC-CAMPAGNA.md`.
- Riferito (non verificato da me in questa sessione): il playtest
  dell'utente sul livello 6 e i suoi commenti sul laboratorio; il contenuto
  esatto delle scelte fatte con AskUserQuestion durante la sessione di
  design.
- Non provato: nessuna implementazione di codice della campagna lunga, del
  generatore, dei nuovi moduli/personaggi o degli intermezzi — esiste solo
  il design doc, non ancora approvato.

## Decisioni prese in sessione
- Approccio ibrido per i livelli: generatore parametrico a seed fisso +
  livelli curati ai punti chiave (non ancora in SPEC.md, solo in
  SPEC-CAMPAGNA.md finché non è approvato).
- Sblocchi sia di moduli sia di personaggi, alternati ogni 5 livelli.
- Design doc (`SPEC-CAMPAGNA.md`) scritto e da approvare prima di scrivere
  codice.
- Regola +5 punti/tick per laboratorio attivo proposta come fix al difetto
  di design emerso dal playtest (non implementata).
