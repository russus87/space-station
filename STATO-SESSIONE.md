# Stato sessione — 26 luglio 2026, notte

## Attività in corso
Attesa esito CI del tag `v0.5.0` (run `30180279242`, avviato alle
23:56:10 UTC, ancora **in_progress** al momento di questo aggiornamento:
job `arch`, `linux`, `windows` tutti avviati, nessuno step segnalato come
fallito finora).

## Appena completato
La v0.4.0 è COMPLETA in release (3 artefatti). Poi, con 2 fork Fable + 1
agente Sonnet + main loop, sono state implementate insieme tre richieste
utente, commit `643586b`, tag `v0.5.0`:

1. **Prologo a fumetto** (fork P, nuovo `src/prologo.rs`): all'avvio di
   ogni livello campagna/casuale la griglia si oscura (nero 0.88, z=18):
   pagina 0 = ritratto 160px + battuta del livello in nuvola di fumetto
   bianca (border_radius, coda a 3 pallini) + "Avanti →"; pagina 1 =
   titolo/obiettivo/budget/detriti + "← Indietro" e "Gioca!". Risorsa
   `Prologo{pagina:Option<u8>}`, attivata da `applica_reset`; input di
   costruzione/tastiera/mercato bloccati con
   `run_if(not(prologo::attivo))`; il briefing classico resta.
2. **Marketplace** (fork M + nuovo `src/progressi.rs`): `AppState::Marketplace`
   dal titolo (voce idx 4, n=8): catalogo FACILITIES a CREDITI (Scorta O2
   2, Spurgo 2, Riparazione 3, Stiva 3, Coloni 4, Sonda 5), acquisto →
   scorte persistenti; in partita M/bottone "SCORTE m" apre l'inventario
   (`mercato.rs` ridisegnato: `click_scorte` consuma via
   `Portafoglio::usa` e applica l'effetto; scorte non applicabili mostrate
   spente). Niente più offerte casuali né costi in punti partita.
3. **Timer e medaglie**: HUD `TEMPO m:ss` countdown (giallo ≤25%, rosso
   ≤10%) al posto di `TICK n/tetto`; al completamento livello campagna
   medaglia oro/argento/rame (soglie 40%/70%/100% del tetto,
   `progressi::medaglia_per_tempo`), colorata nella griglia di selezione
   (GIALLO/BIANCO/RUGGINE via componente `ColoreFisso` rispettato da
   `evidenzia_voci`), mostrata nella schermata completato (risorsa
   `livelli::UltimaMedaglia`), crediti una tantum solo al miglioramento
   (1/2/3, `Portafoglio::registra_livello`), persistenza in
   `progressi.txt` (cartella dati, formato `chiave=valore`:
   crediti/medaglie/scorte).
4. **Docs**: `MANUALE.md` (§4 medaglie+prologo, §5 "Il Marketplace e le
   scorte"), `GUIDA.md` (timer/medaglie/M), zine `docs/manuale.html`+pdf
   aggiornata da agente Sonnet (22 pagine, verificata sul codice) e
   ripubblicata sullo stesso artifact:
   https://claude.ai/code/artifact/ff20f694-c945-4f72-9724-032f501f2345 .
   `HANDOFF.md` locale aggiornato.
5. **Nota tecnica**: nuovo colore `pub RUGGINE` in `ui.rs`; fix
   pattern-matching edition 2024 in `mercato.rs` (`conteggio_scorte`).

## Prossimo passo immediato
Verificare l'esito della CI del tag `v0.5.0` (run `30180279242`,
`gh run view 30180279242` o `gh run watch 30180279242`); se un job
fallisce, sistemare e ritaggare. Se verde, comunicare che la release è
pronta sui 3 artefatti.

## Passi successivi
1. Esito CI v0.5.0 → se verde, release pronta.
2. Playtest completo dell'utente: prologo, marketplace col giro
   medaglie→crediti→scorte, timer — tutto tarato a tavolino (soglie
   40/70, crediti 1/2/3, costi 2-5), mai giocato da un umano.
3. Nota aperta: i vecchi salvataggi hanno livelli completati senza
   medaglia registrata (colore assente in griglia finché non li
   rigiocano) — comportamento atteso, non un bug, ma da tenere a mente
   nel playtest.
4. Da `SPEC-CAMPAGNA.md` §9 restano (non ancora approvate): eventi con
   scelta, riparazione con costo, velocità 1×/2×/4×, stelle per livello
   (da valutare se superate dalle medaglie), sfida del giorno.
5. Tratti passivi dei personaggi (proposta non implementata).
6. Bilanciamento generale mai playtestato: curva del generatore, costi
   di marketplace, soglie/crediti delle medaglie, numeri dei moduli.

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` compila **a 0
  warning** (nessuna riga in output).
- Verificato io stesso, ora: `cargo test --quiet` — **31/31** test
  passati (coerente col riassunto ricevuto).
- Verificato io stesso, ora: `git log` mostra `643586b` in testa su
  `main`; tag `v0.5.0` presente e puntato allo stesso commit; `Cargo.toml`
  riporta `version = "0.5.0"`. `git status`: pulito tranne `Cargo.lock`
  modificato ma non in staging (probabile aggiornamento locale non
  ancora committato, non blocca nulla).
- Verificato io stesso, ora, sul filesystem: `src/prologo.rs` e
  `src/progressi.rs` esistono. `progressi.txt` **non** trovato nella
  radice del repo — cercando nel codice, il file vive nella "cartella
  dati" restituita da `cartella_dati()` (stessa dove sta
  `impostazioni.txt`), non nella root: nessuna discrepanza, solo percorso
  diverso da dove l'ho cercato prima.
- Verificato io stesso, ora, via `gh run list` e `gh run view
  30180279242`: run `30180279242` per il tag `v0.5.0`, trigger push,
  **in_progress**, job `arch`/`linux`/`windows` avviati da poco (~1-2
  minuti), nessun fallimento segnalato. Coerente col riassunto ricevuto
  ("CI in corso").
- Non riprovato da me in questa passata: smoke test del gioco in
  esecuzione (prologo, marketplace, timer/medaglie) — riferito come
  verificato dalla sessione precedente ma non ri-eseguito qui; nessun
  playtest umano completo finora.

## Decisioni prese in sessione
- Marketplace: pagamento SOLO coi crediti guadagnati dalle medaglie,
  nessuna valuta reale — scelta esplicita dell'utente, in continuità con
  la decisione precedente sul mercato interno (ora sostituito da questo
  sistema).
- Timer e medaglie: soglie (40%/70%/100%) e crediti (1/2/3) tarati a
  tavolino dall'utente/sessione, non ancora validati da playtest.
