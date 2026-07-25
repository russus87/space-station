# Stato sessione — 25 luglio 2026, notte

## Attività in corso
Attesa esito CI del tag `v0.3.0` (run `30171337920`, ancora in corso al
momento di questo aggiornamento — nessun job fallito finora, tutti e tre
sono allo step "Build binary"); il gioco gira già in locale per l'utente
con tutte le novità di questa sessione.

## Appena completato
Sessione molto intensa, lavoro in parallelo (3 fork del modello + main
loop). Tutte richieste esplicite dell'utente:

1. **Font**: DejaVu Sans incorporato nel binario, sostituito all'asset di
   default in `main.rs` (`font_principale`) — spariti i quadrati (·, —, →,
   º, accentate). Commit `69ae6be`.
2. **Moduli sbloccabili (5 nuovi, `ModuleKind` a 11 varianti)**:
   - Batteria (liv. 5): accumula fino a 150, ricarica max 15/tick dal
     surplus, copre i deficit di rete prima degli spegnimenti —
     regolamento in `sim.rs` dopo l'allocazione, campo `Module.carica`.
   - Serra (liv. 15): -10 energia, +20 O2, +8 calore.
   - Gru (liv. 25): 12 tick attivi consecutivi → rimuove detrito
     adiacente e si smonta; contatore `Module.lavoro` in `sim.rs`, effetto
     `applica_gru` in `main.rs`.
   - Condotto termico (liv. 35): -90 calore.
   - Centro comando (liv. 45): max 1, arrivi ogni 2 tick invece di 4.
   - Campo `sblocco` in `ModuleDef`, palette a 11 slot con soglie visibili
     ("si sblocca al livello N"), tasti `7 8 9 0 C`, HUD energia mostra
     "batt carica/capienza" quando esistono batterie.
3. **Personaggi a fumetto**: nuovo `src/personaggi.rs` (Vera ingegnera,
   Tomas medico, Dario caposquadra, Mira scienziata, Ilse comandante) con
   battute nei briefing dei livelli 1, 2, 7 e multipli di 5, e annunci di
   sblocco nella schermata livello completato (5/15/25/35/45); ritratti
   pixel art 32×32 in `assets/sprites/ritratti/`; box fumetto in
   `menu.rs` (ritratto+nome+balloon). Solo narrativa: i tratti passivi dei
   personaggi restano proposta non implementata.
4. **Mercato interno**: nuovo `src/mercato.rs` — tasto `M` o bottone HUD
   "MERCATO m", 3 offerte casuali per partita da un catalogo di 6
   facilities una tantum (Scorta d'ossigeno 80, Squadra di riparazione
   120, Trasporto coloni 150, Ampliamento stiva 100 solo con budget,
   Spurgo termico 60, Sonda demolitrice 200 solo con detriti), pagate coi
   PUNTI partita (il punteggio scende: tradeoff con la classifica).
   Nessuna valuta reale, per scelta esplicita dell'utente. Offerte
   rinnovate in `applica_reset`.
5. **MANUALE.md**: manuale illustrato completo (fork dedicato) con
   immagini generate in `docs/img/` da `tools/gen_docs_img.py` (11 moduli
   96px, 5 ritratti 160px, ostacolo) — schede personaggi con ritratti come
   chiesto dall'utente.
6. **Sprite**: 10 mappe ASCII nuove in `gen_sprites.py` (5 moduli + 5
   ritratti), `check_sprites` esteso a 11 moduli, tutte le coppie sotto
   IoU 80%.
7. **Docs allineate**: `SPEC.md` (§13.1 Quattro modalità + riga Casuale,
   nota iterazione 4 implementata), `SPEC-CAMPAGNA.md` (header
   aggiornato: implementato quasi tutto, restano tratti
   personaggi/intermezzi/§9), `README.md` (comandi `1..0+C`, `M`, conteggio
   sprite, link MANUALE), versione `0.3.0`.
8. Commit `7439828` "Moduli sbloccabili, personaggi a fumetto, mercato
   interno e manuale" pushato; tag `v0.3.0` creato e pushato.

La release v0.2.0 (50 livelli + font DejaVu embedded al posto del subset
Bevy che mostrava quadrati) resta COMPLETA sui 3 artefatti.

## Prossimo passo immediato
Verificare l'esito della CI del tag `v0.3.0` (run `30171337920`,
`gh run view 30171337920` o `gh run watch 30171337920`); se un job
fallisce, sistemare e ritaggare.

## Passi successivi
1. Esito CI v0.3.0 → se verde, comunicare che la release è pronta sui 3
   artefatti.
2. Playtest utente: moduli sbloccabili (nota: la sua progressione ha già
   5 livelli completati quindi la Batteria è già visibile), mercato
   interno, fumetti dei personaggi.
3. Da `SPEC-CAMPAGNA.md` restano (sezioni ⚠, non ancora approvate):
   tratti passivi dei personaggi, intermezzi diario di bordo, estensioni
   §9 (eventi, riparazione con costo, velocità 1×/2×/4×, stelle per
   livello, audio, sfida del giorno).
4. Bilanciamento generale mai playtestato: curva del generatore, costi
   del mercato, numeri dei moduli nuovi (Batteria/Serra/Gru/Condotto/
   Centro comando).

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` — nessun output, 0
  errori/0 warning.
- Verificato io stesso, ora: `cargo test --quiet` — 24/24 passati.
- Verificato io stesso, ora: `git log` mostra `7439828` in testa su
  `main` (preceduto da `69ae6be` font e `9ad8a5c` 50 livelli); tag
  `v0.1.0`/`v0.2.0`/`v0.3.0` presenti; `git status --short` pulito
  (nessuna modifica non committata).
- Verificato io stesso, ora, sul filesystem: `src/personaggi.rs` (6905
  byte), `src/mercato.rs` (11964 byte), `MANUALE.md` (12906 byte),
  `tools/gen_docs_img.py` esistono; `assets/sprites/ritratti/` contiene 5
  PNG; `docs/img/` contiene 17 PNG (11 moduli + 5 ritratti + ostacolo);
  `Cargo.toml` riporta `version = "0.3.0"`.
- Verificato io stesso, ora, via `gh run view 30171337920 --json
  status,conclusion,jobs`: i tre job (`linux`, `windows`, `arch`) sono
  **in_progress**, tutti fermi allo step "Build binary" — nessuno step
  precedente fallito, ma build non ancora completata né confermata verde.
- Non provato: smoke test locale della build di QUESTA sessione (font,
  moduli sbloccabili, mercato, fumetti) è riferito dall'utente come
  funzionante ma non ri-eseguito da questo aggiornamento; il pacchetto
  Arch aggiornato non è stato reinstallato dagli artefatti CI; la curva
  di bilanciamento dei moduli nuovi e dei costi di mercato non è mai
  stata giocata da un umano.

## Decisioni prese in sessione
- Mercato interno: pagamento SOLO in punti partita, nessuna valuta reale
  — scelta esplicita dell'utente, non nella proposta originale di
  SPEC-CAMPAGNA.md.
- Personaggi: implementata solo la parte narrativa (battute + sblocco);
  i tratti passivi restano deliberatamente non implementati, in attesa di
  ulteriore richiesta esplicita.
- Font: sostituzione diretta dell'asset di default con DejaVu Sans
  incorporato nel binario, invece di un subset custom Bevy (causa dei
  quadrati nella v0.2.0).
