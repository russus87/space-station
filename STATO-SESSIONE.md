# Stato sessione — 26 luglio 2026, pomeriggio

## Attività in corso
Attesa esito CI del tag `v0.7.0` (run `30203903327`, **in_progress** al
momento di questo aggiornamento: job `arch`, `linux`, `windows` avviati da
~2 minuti, nessuno step segnalato come fallito finora).

## Appena completato
Dalla v0.6.0: WAVE COSMESI+BILANCIAMENTO (9 richieste utente con
screenshot, 3 fork C1/C2/C3) + IMPREVISTI (proposta utente approvata, 1
fork) + giro docs (1 fork .md + 1 Sonnet per la zine, entrambi ripresi
dopo un session-limit e completati). Commit `7008909`, tag `v0.7.0`.

Contenuto v0.7.0:
1. **Regola dei conduttori** (`sim.rs`): l'energia si propaga solo
   attraverso Reattori e Corridoi (BFS dai conduttori); gli altri moduli
   sono foglie che si allacciano a un conduttore adiacente senza
   prolungare. `fabbisogno_minimo` include i corridoi (formula
   `ceil((foglie−3·reattori)/2)`), budget curati ritoccati (L2 13, L3
   10+, L6 13+margine), test aggiornati + 2 nuovi. Stazione demo di
   DEMO_FOTO rifatta con dorsale di corridoi.
2. **Imprevisti** (nuovo `src/imprevisti.rs`, macchina a stati testata
   con casualità iniettata): Meteorite (avaria sul colpito, meteora in
   volo + flash), Tempesta EM (10 tick surriscaldamento extra, tinta
   pulsante), Sciame (rompe 1-2 corridoi), Passaggio del pianeta
   (positivo: arrivi raddoppiati 15 tick, pianeta 48px con anello sullo
   sfondo). Preavviso 4 tick coi negativi: sirena pixel animata in alto
   (2 frame) + musica in pausa (`musica.rs`: `MusicaSospesa` +
   `applica_sospensione` con `AudioSink` pause/play). Grazia tick 40,
   cooldown 50, mai due uguali, 1/70 per tick, campagna dal livello 8.
   Sprite `assets/sprites/imprevisti/*` (sirena_1/2, meteora, pianeta) e
   suoni `assets/audio/{sirena,impatto,pianeta}.wav` nuovi.
3. **Medaglie**: oro ≤35% (1:38), argento ≤60% (2:48), costanti
   `SOGLIA_*_PERCENTO`; test che l'oro resti possibile su tutti i 50
   livelli+200 seed (tempo intrinseco peggiore 130 ≤ 140). Buff:
   Batteria capienza 250/ricarica 25/calore +1; Serra −8/+25/+6 (3,1
   O2/watt). Obiettivi pesati per blocco (dopo ogni sblocco, i livelli
   valorizzano il modulo nuovo); LIVELLI 7-50 RIGENERATI (progressione/
   medaglie utente conservate, contenuti diversi).
4. **UI/flusso**: HUD senza "lab servono N"; Registro chiuso di default
   (icona `sprites/registro.png` + nota "tail -f /var/log/stazione.log —
   il grep è compreso nel prezzo", risorsa `RegistroAperto`); scorte
   come icone 24px con tooltip nella colonna sinistra sopra l'ispezione
   (click=usa, motivo se non applicabile; VIA il tasto M, l'overlay e il
   bottone HUD — `Mercato.aperto` resta come residuo sempre-false);
   `AppState::Briefing` ELIMINATO (selezione→partita col prologo: pagina
   obiettivo sempre con medaglia+tempi da battere via bisezione su
   `medaglia_per_tempo`, vignetta solo prima visita, ridisegnata: cornice
   da tavola, ritratto 144px, balloon con coda a gradini); "Manuale
   illustrato (PDF)" in Come si gioca apre `assets/manuale.pdf`
   (xdg-open/start).
5. **Docs**: tutti e 5 i `.md` (GUIDA/MANUALE/README/SPEC/
   SPEC-CAMPAGNA) + zine (`docs/manuale.html`/`.pdf`, 10 sezioni,
   capitolo "Gli imprevisti") allineati; `docs/manuale.pdf` e
   `assets/manuale.pdf` byte-identici (md5 `d8e67a24...` confermato);
   artifact ripubblicato.

## Prossimo passo immediato
Verificare l'esito della CI del tag `v0.7.0` (`gh run view 30203903327`
o `gh run watch 30203903327`); se un job fallisce, sistemare e ritaggare.
Se verde, comunicare che la release è pronta.

## Passi successivi
1. Esito CI v0.7.0 → se verde, release pronta.
2. Playtest utente: i conduttori e gli imprevisti cambiano parecchio il
   feel (regola energia più severa, eventi random con preavviso/sirena)
   — da verificare che la sfida non frustri; mai provato end-to-end da
   un umano con queste novità.
3. Da `SPEC-CAMPAGNA.md` §9 restano proposte non ancora approvate:
   eventi con scelta (9.1, la più grossa), riparazione con costo, sfida
   del giorno, tratti passivi dei personaggi.

## Stato di verifica
- Verificato io stesso, ora: `cargo build --quiet` compila **a 0
  warning** (nessuna riga in output).
- Verificato io stesso, ora: `cargo test --quiet` — **47/47** test
  passati, coerente col riassunto ricevuto.
- Verificato io stesso, ora: `git log` mostra `7008909` in testa su
  `main` (con `46581ce` e `30e5bc2` sotto, commit delle 15:22 di oggi);
  tag `v0.7.0` presente; `Cargo.toml` riporta `version = "0.7.0"`.
  `git status` pulito, nulla in sospeso.
- Verificato io stesso, ora, via `gh run list`/`gh run view
  30203903327`: run per il tag `v0.7.0`, trigger push, **in_progress**
  da ~2 minuti, job `arch`/`linux`/`windows` avviati, nessun fallimento
  segnalato finora.
- Verificato io stesso, ora, sul filesystem: sprite e suoni degli
  imprevisti presenti (`assets/sprites/imprevisti/{sirena_1,sirena_2,
  meteora,pianeta}.png`, `assets/audio/{sirena,impatto,pianeta}.wav`);
  `assets/manuale.pdf` e `docs/manuale.pdf` byte-identici (stesso md5);
  `docs/manuale.html` contiene la sezione "Gli imprevisti".
- Discrepanza minore rispetto al riassunto ricevuto: la "zine" non vive
  in file `.md` separati ma è `docs/manuale.html` (+ `docs/manuale.pdf`
  generato, copiato in `assets/manuale.pdf`); non esistono file col nome
  letterale "zine" sul filesystem, né una cartella `docs/` con .md — i 5
  `.md` di progetto (GUIDA/MANUALE/README/SPEC/SPEC-CAMPAGNA) sono in
  radice, non in `docs/`. Contenuto e allineamento confermati comunque.
- Riferito (non ri-eseguito da me in questa passata): smoke test del
  gioco in esecuzione — verificato dalla sessione precedente ma non
  ripetuto qui; nessun playtest umano completo finora.

## Decisioni prese in sessione
Nessuna nuova decisione da registrare in questo aggiornamento (lavoro di
verifica e allineamento del solo `STATO-SESSIONE.md`).
