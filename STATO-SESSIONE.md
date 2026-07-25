# Stato sessione — 25 luglio 2026, sera

## Attività in corso
Attesa esito CI del tag `v0.2.0` (run in corso al momento di questo
aggiornamento); il gioco gira già in locale per l'utente con la build
appena compilata.

## Appena completato
- **Richiesta che ha sbloccato la FASE 2**: l'utente ha installato il
  pacchetto v0.1.0 e ha chiesto dove fossero i 50 livelli (erano solo
  proposta in `SPEC-CAMPAGNA.md`) — preso come via libera per la parte del
  piano SENZA cambi alle regole della simulazione.
- **Nuovo `src/generatore.rs`** (376 righe): PRNG splitmix64 interno (non
  la crate `rand`, per avere una sequenza stabile per sempre);
  `fabbisogno_minimo(obiettivo)` condivisa tra generatore e test; pattern
  di detriti (sparsi / muro V / muro H / diagonale / croce) con retry e
  ripiego; `area_libera_connessa` (flood fill); `genera_campagna(n)` con
  seed fisso `SEME^n` e `genera_casuale(seed)` (difficoltà 10-40). Curva:
  margine budget da ×1,6 a ×1,15, quota detriti da 2 a 12, obiettivi
  scalati sull'indice.
- **`src/livelli.rs`** (623 righe): `LivelloDef` ora owned (String/Vec,
  Clone); `LIVELLI` è `LazyLock<Vec<LivelloDef>>` = 6 curati (invariati,
  in `livelli_curati()`) + 44 generati; nuova `Modalita::Casuale` +
  risorsa `LivelloCasuale`; `campagna_attiva` rinominata
  `obiettivi_attivi` (copre anche Casuale); `controlla_obiettivo` gestisce
  Casuale (la progressione avanza solo in campagna).
- **`src/main.rs`**: GRID_W/H resi `pub(crate)`; `mod generatore`;
  `applica_reset` unificato per campagna/casuale (detriti+budget+log da
  un'unica via); `controlla_fine`: Casuale è fuori classifica; tetto tick
  applicato anche in Casuale.
- **`src/menu.rs`**: voce titolo "Livello casuale" (menu a 7 voci);
  `Azione::GiocaCasuale` (seed da rand di sistema; da "livello completato"
  fa da "Nuovo livello casuale"); selezione livello a griglia 10×5 (celle
  numerate compatte via `voce_cella`, completati col punto, bloccati
  spenti, selezione iniziale sul primo non completato); navigazione: in
  selezione su/giù = ±10, sinistra/destra = ±1; negli altri menu anche
  sinistra/destra funzionano; schermate completato/fine adattate al
  casuale ("Riprova il livello" rigioca lo stesso seed).
- **`src/ui.rs`**: HUD obiettivo unificato campagna/casuale.
- **Docs**: SPEC.md §13.2 riscritta (50 livelli, generatore, casuale) e
  §13.3 (griglia, titolo a 7 voci); GUIDA.md "Le quattro modalità";
  README.md aggiornato.
- **Versione 0.2.0** in Cargo.toml.
- **Commit `9ad8a5c`** "Campagna a 50 livelli, generatore parametrico e
  modalità Livello casuale" pushato; tag `v0.2.0` creato e pushato.

## Prossimo passo immediato
Verificare l'esito della CI del tag `v0.2.0` (run `30170174834`,
`gh run view 30170174834` o `gh run watch 30170174834`); se un job fallisce,
sistemare e ritaggare. Poi l'utente aggiorna il pacchetto Arch e playtesta
la campagna lunga.

## Passi successivi
1. Esito CI v0.2.0 → se verde, comunicare che la release è pronta.
2. Playtest utente sulla campagna a 50 livelli e sul livello casuale:
   la curva di difficoltà dei livelli generati è a tavolino, mai giocata —
   raccogliere feedback prima di considerarla definitiva.
3. Solo dopo approvazione esplicita dell'utente (resta in
   `SPEC-CAMPAGNA.md`, sezioni ⚠, non ancora implementato):
   - punti laboratorio;
   - sblocchi moduli/personaggi;
   - intermezzi diario di bordo;
   - velocità 1×/2×/4×;
   - stelle per livello;
   - livelli curati di fine blocco (per ora 7-50 sono tutti generati; ha
     senso curarli quando arriveranno gli sblocchi).

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` — 0 errori, 0 warning.
- Verificato io stesso, ora: `cargo test --quiet` — 18/18 passati (nuovi
  rispetto allo snapshot precedente: determinismo generazione, 50 livelli
  risolvibili, 200 seed casuali risolvibili, costanti allineate a
  TABELLA).
- Verificato io stesso, ora: `git log` mostra il commit `9ad8a5c` in testa
  su `main`; tag `v0.1.0` e `v0.2.0` presenti; `git status --short` pulito
  (nessuna modifica non committata).
- Verificato io stesso, ora, sul filesystem: `src/generatore.rs` (376
  righe) e `src/livelli.rs` (623 righe) esistono con i contenuti attesi;
  grep su `src/menu.rs` conferma `GiocaCasuale`, `voce_cella`, "Livello
  casuale", "Nuovo livello casuale".
- Verificato io stesso, ora, via `gh run list`/`gh run view 30170174834`:
  la CI del tag `v0.2.0` (job `arch`, `linux`, `windows`) risultava
  **ancora in corso** (in_progress) al momento di questo aggiornamento —
  non ancora confermata verde per i tre artefatti.
- Non provato: il pacchetto Arch aggiornato non è stato reinstallato né
  ri-testato da questa sessione dopo la CI (l'utente gioca con la build
  compilata in locale, non con l'artefatto di release); la curva di
  difficoltà dei 50 livelli generati non è mai stata giocata da un umano.

## Decisioni prese in sessione
- Nessuna nuova decisione di design: l'implementazione di questa sessione
  copre solo la parte del piano "senza cambi di regole della simulazione"
  (generatore + 50 livelli + modalità casuale), già prevista in
  `SPEC-CAMPAGNA.md`. Le sezioni ⚠ (punti laboratorio, sblocchi,
  intermezzi, velocità, stelle) restano in attesa di approvazione
  esplicita e non sono state toccate.
