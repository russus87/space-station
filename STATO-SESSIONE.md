# Stato sessione — 26 luglio 2026, notte fonda

## Attività in corso
Attesa esito CI del tag `v0.5.1` (run `30181078049`, ancora
**in_progress** al momento di questo aggiornamento: job `arch`, `linux`,
`windows` tutti avviati, nessuno step segnalato come fallito finora).

## Appena completato
La v0.5.0 è COMPLETA in release (3 artefatti). Poi, con due fork Fable in
sequenza (stessi file, quindi serializzati) su richieste dell'utente da
playtest, commit `069b289`, tag `v0.5.1`:

1. **Fix bug da screenshot utente**: il ghost di piazzamento (anteprima
   modulo) seguiva il mouse anche sotto il prologo — ora `aggiorna_ghost`
   lo nasconde con prologo aperto o overlay scorte aperto.
2. **Cursore custom pixel-art 16×16** (`assets/sprites/cursore.png`,
   hotspot in punta): `CursorIcon::Custom(CustomCursor::Image(...))`
   sull'entità `PrimaryWindow`, sistema `cursore_pixel` in `main.rs`.
3. **Schermata livello completato**: al posto del testo "MEDAGLIA D'ORO ·
   +3 crediti", medaglia in pixel art 24×24 mostrata a 48px
   (`assets/sprites/medaglie/{oro,argento,rame}.png`) e sotto 3 monete
   16×16: accese 3/2/1 secondo la medaglia (le altre spente/grigie), le
   accese ruotano con spin a 4 frame ~8fps
   (`assets/sprites/monete/accesa_1..4` + `spenta`, sistema
   `menu::anima_monete`). Nota tecnica: il gruppo visuali di
   `add_systems` superava i 20 sistemi → riorganizzato in sotto-tuple
   annidate.
4. **Marketplace ridisegnato a CARD** (richiesta con screenshot): 6
   icone facility 16×16 nuove
   (`assets/sprites/facilities/{ossigeno,spurgo,riparazione,stiva,coloni,sonda}.png`),
   griglia 3×2 di card 140×150 con icona 48px, nome, "ne hai N", prezzo
   come fila di monete (una per credito, frame statico); cornice DORATA
   (GIALLO 2px) se acquistabile, spenta (GRIGIO_SCAFO) se no; saldo con
   icona moneta. Design: le card restano `Voce` ma con marker
   `StileCard` — `evidenzia_voci` per le card tocca SOLO il bordo (non
   riscrive più i testi figli, che avrebbe distrutto). Aggiornamento live
   post-acquisto via `aggiorna_voci_marketplace` (marker `IconaCard`,
   `PossedutePer`).
5. **Icone scorte in partita**: fila `ScorteHud` nell'HUD accanto al
   bottone "SCORTE m" (icone 20px per tipo, ×N per doppie, nascoste a
   inventario vuoto, sistema `ui::update_scorte_hud`); icona 32px anche
   nelle righe dell'overlay SCORTE.
6. Sprite totali ora **44** (verificato: contati sul filesystem in
   `assets/sprites/`). `HANDOFF.md` locale aggiornato.

NON fatto (nota aperta): `MANUALE.md`/`GUIDA.md`/zine non mostrano ancora
card, icone HUD, cursore e monete — da allineare a un prossimo giro docs.

## Prossimo passo immediato
Verificare l'esito della CI del tag `v0.5.1` (run `30181078049`,
`gh run view 30181078049` o `gh run watch 30181078049`); se un job
fallisce, sistemare e ritaggare. Se verde, comunicare che la release è
pronta sui 3 artefatti.

## Passi successivi
1. Esito CI v0.5.1 → se verde, release pronta.
2. Playtest completo dell'utente del giro intero: medaglie → crediti →
   card dorate → scorte in HUD (mai giocato da un umano end-to-end con
   questa UI).
3. Allineare docs (MANUALE/GUIDA/zine) alle novità grafiche v0.5.1
   (card, icone HUD, cursore, monete) — rimandato a un giro dedicato.
4. Bilanciamento generale mai playtestato: curva del generatore, costi
   di marketplace, soglie/crediti delle medaglie, numeri dei moduli.
5. Da `SPEC-CAMPAGNA.md` §9 restano (non ancora approvate): eventi con
   scelta, riparazione con costo, velocità 1×/2×/4×, sfida del giorno.
6. Tratti passivi dei personaggi (proposta non implementata).

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` compila **a 0
  warning** (nessuna riga in output).
- Verificato io stesso, ora: `cargo test --quiet` — **31/31** test
  passati.
- Verificato io stesso, ora: `git log` mostra `069b289` in testa su
  `main`; tag `v0.5.1` presente e puntato allo stesso commit; `Cargo.toml`
  riporta `version = "0.5.1"`. `git status` pulito, nulla in sospeso.
- Verificato io stesso, ora, sul filesystem: tutti i file citati esistono
  — `assets/sprites/cursore.png`, `assets/sprites/medaglie/{oro,argento,rame}.png`,
  `assets/sprites/monete/{accesa_1..4,spenta}.png`,
  `assets/sprites/facilities/{ossigeno,spurgo,riparazione,stiva,coloni,sonda}.png`;
  conteggio totale sprite in `assets/sprites/`: **44**, coerente col
  riassunto ricevuto.
- Verificato io stesso, ora, via `gh run view 30181078049`: run per il
  tag `v0.5.1`, trigger push, **in_progress**, job
  `arch`/`linux`/`windows` avviati, nessun fallimento segnalato. Coerente
  col riassunto ricevuto ("CI in corso").
- Riferito (non ri-eseguito da me in questa passata): smoke test del
  gioco in esecuzione (ghost nascosto sotto prologo, cursore custom,
  medaglie/monete animate, card marketplace, icone scorte HUD) —
  verificato dalla sessione precedente ma non ripetuto qui; nessun
  playtest umano completo finora.

## Decisioni prese in sessione
- Marketplace a card: le card restano entità `Voce` (per riusare
  navigazione/selezione esistente) ma con marker `StileCard` dedicato,
  per evitare che l'evidenziazione standard riscriva/distrugga i testi
  figli delle card — scelta tecnica per non duplicare la logica di
  navigazione del menu.
- Prezzo delle card mostrato come fila di monete statiche (una per
  credito) invece che come numero, per coerenza visiva col nuovo sistema
  di monete animate della schermata completato.
