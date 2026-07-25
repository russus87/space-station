# Space Station

Gioco di bilancio di risorse con cascata di guasti: si costruisce una stazione
su una griglia, si avvia la simulazione e si reagisce ai guasti a catena.
L'energia si propaga solo tra moduli adiacenti (ogni gruppo connesso è una
rete, e serve un reattore per rete); l'obiettivo è far crescere l'equipaggio
e tenerlo vivo: il punteggio è persone·tick e la partita finisce quando
l'equipaggio torna a zero. Rust + Bevy 0.19, pixel art generata da script
(`tools/gen_sprites.py`).

Quattro modalità: la **Campagna** (50 livelli in sequenza — 6 curati che
insegnano i meccanismi, 44 generati con seed fisso a difficoltà crescente —
con obiettivo misurabile nell'HUD, detriti da aggirare e un budget di
moduli per livello; completarne uno sblocca il successivo), l'**Infinita**
(sandbox senza obiettivi né limiti di tempo), la **Sfida** (come
l'Infinita ma con un tetto di 400 tick: partite brevi e confrontabili) e
il **Livello casuale** (generato al momento, fuori da progressione e
classifiche). Infinita e Sfida hanno ciascuna la propria **classifica**
locale top 10. Classifiche e
progressione della campagna sono file di testo semplice in
`$XDG_DATA_HOME/space-station/` (ripiego `~/.local/share/space-station/`):
una riga malformata si ignora, un file assente vale "nessun dato". I
corridoi si orientano da soli (dritto, curva, T, incrocio) in base ai
moduli adiacenti.

Documenti: `MANUALE.md` (manuale di gioco illustrato), `SPEC.md` (specifica di design), `GUIDA.md` (guida rapida),
`POC.md` (la spec della PoC originale, storica), `STORIA.md` (bibbia narrativa).

## Download

Dalla [pagina Releases](https://github.com/russus87/space-station/releases):

- **Arch Linux**: `space-station-<ver>-1-x86_64.pkg.tar.zst` →
  `sudo pacman -U <file>`
- **Linux (generico)**: `space-station-<ver>-linux-x86_64.tar.gz` →
  estrai e lancia `./space-station`
- **Windows**: `space-station-<ver>-windows-x86_64.zip` →
  estrai e lancia `space-station.exe`

Gli artefatti sono prodotti dal workflow `build` (tag `v*` o lancio
manuale).

## Avvio da sorgente

```sh
cargo run
```

La prima build compila Bevy: qualche minuto. Le successive sono rapide.
Il binario funziona anche lanciato a mano (`./target/debug/space-station`):
il percorso di `assets/` è risolto a runtime.

## Comandi

| Input | Effetto |
|---|---|
| `1..6`, `7 8 9 0 C` / click sulla palette | seleziona il modulo (i 6 base più i 5 sbloccabili lungo la campagna) |
| click sinistro | piazza il modulo selezionato sulla cella |
| click destro | rimuove il modulo sulla cella |
| `R` | ripara il modulo in avaria sotto il cursore |
| `Spazio` | avvia/ferma la simulazione (da fermo si costruisce senza conseguenze, l'HUD mostra l'anteprima del bilancio) |
| `Esc` | apre il menu di pausa e **congela il tick** (è un'altra cosa rispetto a `Spazio`) |
| `M` | apre il mercato interno (facilities una tantum pagate coi punti partita) |
| frecce + `Invio` | navigano i menu (nella griglia livelli: su/giù riga, sinistra/destra cella) |

Un modulo fermo dice sempre *perché*: velo scuro con fulmine giallo = la sua
rete non ha energia a sufficienza, fulmine grigio = scollegato (la sua rete
non ha un reattore funzionante: servono moduli adiacenti che lo colleghino,
i corridoi costano poco), omino = manca equipaggio, velo rosso con triangolo
lampeggiante = avaria. Il pannello in basso a sinistra descrive per esteso il
modulo sotto il cursore. Il registro eventi in fondo racconta la catena
causale (ultime 8 righe, timestamp in tick, colorate per gravità).
Quando l'equipaggio muore del tutto compare la schermata "STAZIONE PERSA"
con punteggio, tick sopravvissuti ed equipaggio massimo; in campagna
propone di riprovare il livello, in infinita dice se il punteggio è
entrato in top 10.

## Dove si tara il bilanciamento

- **`src/modules.rs` → `TABELLA`**: produzione/consumo per tick di ogni modulo
  (energia, ossigeno, calore, posti letto, equipaggio richiesto) più il path
  del suo sprite. È l'unico punto da toccare per cambiare i numeri.
- **`src/livelli.rs` → `livelli_curati()`**: nomi, briefing e numeri dei 6
  livelli curati della campagna. I 44 generati si tarano in
  **`src/generatore.rs`** (curva di difficoltà, quota detriti, margine sul
  budget). I testi mostrati sono generati dai numeri: si tara qui e basta.
- **`src/sim.rs` → costanti in testa**, le quattro che contano:
  - `OSSIGENO_PER_CREW` (10): consumo d'aria per membro per tick — decide
    quanti equipaggi regge un Life Support.
  - `TICK_SURRISCALDAMENTO` (6): tick consecutivi di calore in eccesso prima
    di un'avaria casuale — decide quanto perdona il calore.
  - `TICK_MORTE` (3): con ossigeno a zero, un morto ogni N tick.
  - `TICK_SECS` (0.7): durata del tick — il ritmo di tutto il gioco.

## Dove si ritoccano gli sprite

Le mappe ASCII degli sprite (11 moduli e un detrito 32×32, 5 ritratti, badge, icone e sfondo) stanno in
`tools/gen_sprites.py`, una stringa per riga di pixel con la legenda
carattere → colore nel docstring. Si modifica la mappa e si rigenera:

```sh
python3 tools/gen_sprites.py    # riscrive i PNG in assets/
python3 tools/check_sprites.py  # verifica che le silhouette restino distinguibili
```

Nessuna dipendenza esterna: l'encoder PNG usa solo `zlib` e `struct`.
La palette dei 16 colori è definita due volte per forza di cose — nello script
e in `src/ui.rs` — e va tenuta allineata a `SPEC.md` §2.2.

## Struttura

| File | Ruolo |
|---|---|
| `src/sim.rs` | tick di bilancio, reti elettriche per adiacenza, cascata di guasti, punteggio e fine partita. **È il cuore validato: non si ridisegna.** |
| `src/modules.rs` | la tabella dei moduli |
| `src/livelli.rs` | modalità, i 6 livelli curati + i 44 generati (50 totali), obiettivi, classifiche e progressione persistenti |
| `src/main.rs` | griglia, piazzamento, autotiling dei corridoi, scala responsive, stati dell'app |
| `src/ui.rs` | HUD (risorse + obiettivo di livello + punteggio), palette laterale, pannello ispezione, log |
| `src/menu.rs` | titolo, campagna (selezione livello, briefing, livello completato), classifica, "come si gioca", pausa, fine partita |
