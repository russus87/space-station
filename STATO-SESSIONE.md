# Stato sessione — 26 luglio 2026, notte

## Attività in corso
Attesa esito CI del tag `v0.6.0` (run `30182346344`, **in_progress** al
momento di questo aggiornamento: job `arch`, `linux`, `windows` avviati,
nessuno step segnalato come fallito finora).

## Appena completato
La v0.5.1 è COMPLETA (cursore ridisegnato: freccia canonica nei menu +
mirino in griglia, sistema `cursore_pixel` a due stati). Poi, su richiesta
utente "fai fare un giro a fable 5": doppia revisione con 2 fork
read-only (tecnica + esperienza) → 13 punti prodotti. Su "fai tutto":
wave di implementazione (4 fork paralleli con ownership rigida sui file
+ 1 fork docs + 1 Sonnet per la zine + main loop di coordinamento).
Commit `30e5bc2`, tag `v0.6.0`.

Contenuto v0.6.0:
1. **Bug fix**: input di costruzione bloccati sotto l'overlay scorte
   (run condition `costruzione_permessa` in `main.rs` — prima un click
   su una scorta piazzava anche un modulo dietro); scorte MAI consumate
   a vuoto (`non_applicabile` estesa in `mercato.rs` con 6 casi: coloni
   senza posti, ossigeno pieno, spurgo a zero, riparazione senza avarie,
   stiva senza budget, sonda senza detriti — overlay che si ricostruisce
   quando l'applicabilità cambia); niente click sonoro sulle azioni
   negate (`audio::BottoneMuto` su slot bloccati e card non acquistabili).
2. **Test sul cuore**: regolamento batterie estratto in funzioni pure
   (`bilancio_batterie_rete`, `regola_batteria` in `sim.rs`) + 6 test di
   cui 3 su mini-World Bevy che esegue `sim_tick` vero (helper
   `mondo_con`/`un_tick`; fix `Time::<()>::default()` per ambiguità).
3. **Velocità ×1/×2/×4**: `Sim.velocita`, tasto V
   (`sim::tasto_velocita`, registrato con
   `run_if(costruzione_permessa)`), HUD mostra ×2/×4, `set_duration`
   preserva l'elapsed, reset a 1 a ogni partita.
4. **Commenti in partita** (nuovo `src/commenti.rs` + `personaggi.rs`
   esteso): 17 battute su 7 eventi (PrimoBlackout, PrimaAvaria,
   PrimoArrivo, OssigenoCritico, OroPreso, DetritoRimosso — scatta sia
   per Gru che Sonda —, TettoVicino ≤25%), mini-fumetto 5s in basso a
   destra (z=12), rilevamento via osservazione risorse con `Local`
   (riarmo sul tick che torna indietro), rotazione varianti deterministica.
5. **Prologo**: solo alla prima visita (`Prologo.visti` HashSet,
   `richiedi(chiave)`, `chiave_casuale` per il casuale), tastiera
   Invio/Spazio/Backspace/←.
6. **Altro**: jingle oro (`assets/audio/oro.wav`, `suona_completato`:
   sblocco > oro > vittoria); "Come si gioca" rifatta (griglia 11 moduli
   con soglie, comandi completi); "Nuovo livello casuale" anche da
   FinePartita; briefing con medaglia attuale + tempi da battere per
   oro/argento; saldo crediti nel sottotitolo selezione; clippy pulito
   (fix veri + `#[allow]` motivati).
7. **Docs**: GUIDA/MANUALE/README/SPEC/SPEC-CAMPAGNA allineati (fork
   docs: 11 moduli, V, F12 ovunque, tabella `src/` completa a 15 file,
   SPEC nota "Iterazione 5 implementata", SPEC-CAMPAGNA: 9.3✓ 9.5✓ 9.4
   superata dalle medaglie; restano 9.1 eventi, 9.2 riparazione con
   costo, 9.6 sfida del giorno, tratti passivi, livelli curati fine
   blocco); zine 24 pagine aggiornata (Sonnet) e ripubblicata, versione
   uniformata a v0.6.

## Prossimo passo immediato
Verificare l'esito della CI del tag `v0.6.0` (`gh run view 30182346344`
o `gh run watch 30182346344`); se un job fallisce, sistemare e ritaggare.
Se verde, comunicare che la release è pronta.

## Passi successivi
1. Esito CI v0.6.0 → se verde, release pronta.
2. Playtest utente della wave: velocità (tasto V), commenti in partita,
   prologo una-tantum, scorte mai vuote/bloccate — mai provato da un
   umano end-to-end con queste novità.
3. Da `SPEC-CAMPAGNA.md` §9 restano (non ancora approvate): eventi con
   scelta (la più grossa), riparazione con costo, sfida del giorno,
   tratti passivi dei personaggi, livelli curati di fine blocco.
4. Bilanciamento generale mai playtestato: curva del generatore, costi
   di marketplace, soglie/crediti delle medaglie, numeri dei moduli.

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` compila **a 0
  warning** (nessuna riga in output).
- Verificato io stesso, ora: `cargo test --quiet` — **38/38** test
  passati, coerente col riassunto ricevuto.
- Verificato io stesso, ora: `cargo clippy --quiet` (default, senza
  `--all-targets`) — **0 warning**, coerente col riassunto ("0 clippy").
  Nota/discrepanza minore: `cargo clippy --quiet --all-targets` (che
  include anche i target di test) segnala 2 warning
  `field_reassign_with_default` di scarsa importanza, in
  `src/livelli.rs:609` e `src/sim.rs:718` (assegnazioni di campo su
  `Sim::default()` nei test) — non coperti dal comando di default né
  dalla CI (che non invoca clippy in `.github/workflows/build.yml`).
  Non bloccante, segnalato per completezza.
- Verificato io stesso, ora: `git log` mostra `30e5bc2` in testa su
  `main` (con `89e2ee1` e `069b289` sotto); tag `v0.6.0` presente e
  puntato allo stesso commit; `Cargo.toml` riporta `version = "0.6.0"`.
  `git status` pulito, nulla in sospeso.
- Verificato io stesso, ora, via `gh run view 30182346344`: run per il
  tag `v0.6.0`, trigger push, **in_progress**, job
  `arch`/`linux`/`windows` avviati, nessun fallimento segnalato finora.
- Riferito (non ri-eseguito da me in questa passata): smoke test del
  gioco in esecuzione (bug fix scorte/costruzione, velocità V, commenti
  in partita, prologo una-tantum) — verificato dalla sessione precedente
  ma non ripetuto qui; nessun playtest umano completo finora.

## Decisioni prese in sessione
- Wave di implementazione con 4 fork paralleli a ownership rigida sui
  file (per evitare conflitti di merge tra fork che scrivono in
  parallelo), più 1 fork dedicato solo ai docs e 1 sub-agent Sonnet
  dedicato solo alla zine — separazione per tenere il lavoro grafico/
  editoriale fuori dai fork che toccano codice.
- Regolamento batterie estratto in funzioni pure separate da Bevy
  (`bilancio_batterie_rete`, `regola_batteria`) per renderlo testabile
  senza dover sempre passare da un `World` Bevy completo — pur
  mantenendo comunque 3 test che esercitano `sim_tick` reale su un
  mini-World, per coprire anche l'integrazione col sistema.
