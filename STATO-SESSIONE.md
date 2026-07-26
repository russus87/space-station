# Stato sessione — 26 luglio 2026, sera

## Attività in corso
Nessuna. Sessione conclusa dall'utente ("ci vediamo nei prossimi
giorni"); release v0.8.0 pubblicata e verificata.

## Appena completato
Dalla v0.7.0 (release completa con manuale HTML embeddato in assets,
apribile in-game accanto al PDF): utente ha approvato il piano
miglioramenti scegliendo i punti 1,8,2,3,4,5,9 (l'inglese "col tempo") →
WAVE GAMEPLAY con 5 fork paralleli (R sim, T prologo, E eventi, B
menu/livelli/generatore/progressi/ui, P9 particelle+attract) + contratto
`squadra.rs` scritto dall'agente + 2 agent docs (ripresi due volte dopo
session-limit). Commit `8fb1df2`, tag `v0.8.0`. **CI v0.8.0 completa**
(job `linux`/`windows`/`arch` tutti verdi) e **release pubblicata** con i
3 artefatti: `space-station-0.8.0-1-x86_64.pkg.tar.zst` (35.6 MB),
`space-station-0.8.0-linux-x86_64.tar.gz` (41.1 MB),
`space-station-0.8.0-windows-x86_64.zip` (29.3 MB).

Contenuto v0.8.0:
1. **Riparazione con costo** (`sim.rs`): R avvia cantiere —
   `TICK_RIPARAZIONE=8`, `EQUIPAGGIO_RIPARAZIONE=2`, campo
   `Module.riparazione`, impegno equipaggio prima dei lab, sospensione
   senza braccia (ordine seq), guardia anti-scorta,
   `Sim.equipaggio_impegnato` + `equipaggio_libero()`; ispezione mostra i
   tick mancanti.
2. **Squadra** (nuovo `src/squadra.rs` + selettore nel prologo di fork T):
   tratti Vera/Tomas/Dario/Mira/Ilse (−25% calore reattori / asfissia ×2
   tick / arrivi 3 tick / −25% energia lab / doppio posto), sblocchi
   10/20/30/40/50, Resource `Squadra` azzerata al reset, `sim_tick` la
   legge via metodi con default=oggi. Fila ritratti 48px nel prologo
   (bordo verde=schierato, bloccati con soglia, solo mouse).
3. **Eventi con scelta** (nuovo `src/eventi.rs`): 5 bivi (nave in
   avaria/mercante/sciame in rotta/straordinari/clandestino),
   presentatore col fumetto, conseguenze dichiarate in giallo, opzioni
   impossibili spente, click o tasti 1/2; `EventoAperto` congela
   `sim_tick` (gate in `main` con `and_then`), grazia 60/cooldown
   80/1 su 90/livello 8+/una volta per partita/mai durante imprevisti
   (`imprevisti::tranquillo` nuovo). `costruzione_permessa` e cursore
   includono il gate.
4. **Bonus + sfida del giorno** (fork B): `livelli::Bonus`
   (SenzaDemolire/SenzaScorte/SottoBudget/OssigenoMai50) deterministico
   per livello, `sorveglia_bonus` (demolizione=celle che calano più degli
   ostacoli), +1 credito una tantum (progressi.txt riga `bonus=`), HUD
   col bonus accanto all'obiettivo, esito nella schermata vittoria;
   sfida del giorno: voce titolo idx4 con ✓ dinamica,
   `genera_giornaliera(giorno)` difficoltà 25, riusa `Modalita::Casuale`
   + flag `SfidaDelGiorno`, miglior tempo del giorno persistito
   (`giorno=`/`giorno_tick=`), NUOVO RECORD in schermata vittoria.
5. **Atmosfera** (fork P9): `particelle.rs` (fumo avarie/scintille
   reattori/bollicine LS+serre, cap 120, z 0.8) e `attract.rs` (titolo
   vivo: 10 stelle pulsanti, pianeta in transito ~40s, meteore; sfondo
   titolo ora nero alpha 0.82).
6. Fix integrazione fatti dal coordinatore: `applica_reset` oltre 16
   parametri → tupla `accessori` di 4 `ResMut`; ordine argomenti
   `esegui`; `Time::<()>`; registrazioni; guida in-game R col costo;
   allow clippy mirati.
7. **Docs**: 5 `.md` allineati (GUIDA tabella L2 corretta 12→13, MANUALE
   §7 bivi + frontespizio v0.8, README struttura completa, SPEC
   Iterazione 7, SPEC-CAMPAGNA: resta proposto SOLO livelli curati di
   fine blocco) + zine 29 pagine sincronizzata nelle 4 copie + artifact
   ripubblicato.
8. Creato in questa sessione l'agente `aggiorna-contesto` (definizione in
   `.claude/agents/aggiorna-contesto.md`); non ancora registrato col suo
   nome nella lista agenti disponibile a questa sessione (finora invocato
   in fallback general-purpose) — dovrebbe risultare disponibile col nome
   proprio dalla prossima riapertura.

## Prossimo passo immediato
Alla riapertura, raccogliere il playtest dell'utente sulla v0.8.0
(riparazione con costo, squadra, bivi, bonus, sfida del giorno, oltre a
conduttori e imprevisti già in v0.7.0 — tutto tarato finora solo a
tavolino) e bilanciare da lì.

## Passi successivi
1. Playtest utente completo della v0.8.0 → bilanciamento conseguente.
2. In coda per scelta dell'utente (nessun ordine deciso):
   - traduzione inglese "col tempo" (nessuna scadenza, priorità bassa);
   - salvataggio a metà partita, "dopo i test";
   - livelli curati di fine blocco (unico punto ancora proposto in
     `SPEC-CAMPAGNA.md`).
3. Fattibilità Android valutata: **fattibile**, ~2 wave (prima UX touch,
   poi packaging). Percorso consigliato: touch → WASM → APK. Non
   ancora avviata, in attesa di priorità dall'utente.

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` compila **a 0
  warning** (nessuna riga in output).
- Verificato io stesso, ora: `cargo clippy --quiet` **0 warning**
  (nessuna riga in output).
- Verificato io stesso, ora: `cargo test --quiet` — **64/64** test
  passati, coerente col riassunto ricevuto.
- Verificato io stesso, ora: `git log` mostra `8fb1df2` in testa su
  `main` (con `093dfab` e `7008909` sotto); tag `v0.8.0` presente;
  `Cargo.toml` riporta `version = "0.8.0"`. `git status` pulito, nulla
  in sospeso.
- Verificato io stesso, ora, via `gh run view 30214518033`: run per il
  tag `v0.8.0` **completato**, tutti i job verdi (`linux` 12m40s,
  `windows` 23m50s, `arch` 16m21s), artefatti prodotti
  (`windows-zip`, `arch-package`, `linux-tarball`); nessun fallimento.
- Verificato io stesso, ora, via `gh release view v0.8.0`: release
  pubblicata (non draft, non prerelease) il 2026-07-26T18:33:26Z, con i
  3 asset attesi (`.pkg.tar.zst`, `.tar.gz`, `.zip`).
- Verificato io stesso, ora, sul filesystem: `src/squadra.rs` (4267
  byte), `src/eventi.rs` (22157 byte), `src/particelle.rs` (6131 byte),
  `src/attract.rs` (6334 byte) presenti, tutti modificati oggi
  pomeriggio.
- Riferito (non ri-verificato da me in questa passata): smoke test del
  gioco in esecuzione — riferito dal riassunto ricevuto come "ok", non
  ripetuto in questa verifica; nessun playtest umano completo finora.

## Decisioni prese in sessione
- Fattibilità Android confermata dal coordinatore: percorso consigliato
  touch → WASM → APK, ~2 wave (UX touch, poi packaging). Non ancora
  riflessa in `SPEC.md`/`SPEC-CAMPAGNA.md`.
