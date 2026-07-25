# SPACE STATION — Manuale di gioco

*Manuale operativo di bordo, edizione v0.3 — da leggere prima del primo
turno in cabina di comando. O durante. Nessuno ti giudica.*

---

Benvenuto a bordo, Direttore.

Space Station è un gioco di equilibri: costruisci una stazione spaziale su
una griglia, moduli attaccati ad altri moduli, e poi premi Spazio e guardi
se regge. L'energia scorre solo tra celle adiacenti, l'aria basta finché
basta, il calore si accumula in silenzio. Quando qualcosa cede — e
qualcosa cede sempre — i guasti arrivano in fila indiana, e il registro
eventi in fondo allo schermo te li racconta riga per riga.

Il ciclo è tutto qui: **costruisci → avvia → leggi il bilancio → reagisci
al guasto**. Il resto di questo manuale spiega i dettagli; la sostanza è
che ogni numero sullo schermo vuole dirti qualcosa, prima che sia tardi.

---

## 1. Le quattro risorse

| Risorsa | Come funziona |
|---|---|
| **Energia** | Non si accumula (salvo Batterie): a ogni tick la produzione dei reattori si distribuisce ai moduli della stessa rete, in ordine di priorità. Se non basta, i moduli meno critici si spengono. |
| **Ossigeno** | Riserva unica di stazione, da 0 a 100. Sale con i life support (e le serre) attivi, scende di 10 per membro dell'equipaggio a ogni tick. Sotto 30 scatta l'allarme. |
| **Calore** | Quasi tutto scalda; radiatori e condotti dissipano. Un bilancio positivo per 6 tick consecutivi manda in avaria un modulo a caso. |
| **Equipaggio** | Con posti letto liberi e aria buona (riserva sopra 50) arriva una persona ogni 4 tick. Con l'ossigeno a zero, ne muore una ogni 3. |

**La regola d'oro dell'energia**: la corrente non salta il vuoto. Ogni
gruppo di moduli ortogonalmente adiacenti è una **rete**, e una rete senza
reattore funzionante è ferraglia fredda. I corridoi esistono per questo:
un filo di corrente da 1 di energia a cella.

### La cascata di guasti

Il gioco non ti punisce con un colpo solo: ti mostra la catena.

1. **Blackout** — la rete consuma più di quanto produce: i moduli si
   spengono in ordine inverso di priorità (prima i laboratori, per ultimo
   il life support).
2. **Life support giù** — l'ossigeno smette di rigenerarsi; la riserva
   scende di 10 a persona, a ogni tick.
3. **Ossigeno a zero** — l'equipaggio comincia a morire, uno ogni 3 tick.
4. **Equipaggio a zero** — la stazione è persa.

Ogni anello della catena si può spezzare, se te ne accorgi in tempo: il
registro eventi e i badge sui moduli (fulmine = energia, omino = manca
equipaggio, triangolo = avaria) sono lì per quello.

**Tetto dei tick**: in Campagna, Sfida e Livello casuale la partita si
chiude comunque a 400 tick ("TEMPO SCADUTO"). Solo l'Infinita non ha
orologio.

---

## 2. I moduli

Undici moduli: sei disponibili da subito, cinque si guadagnano avanzando
nella campagna (e una volta sbloccati valgono in **tutte** le modalità).
I valori sono *per tick*. La **priorità** dice l'ordine di alimentazione:
0 è servito per primo, 4 si spegne per primo quando la corrente manca.

| | Modulo | Tasto | Energia | Ossigeno | Calore | Note | Priorità |
|---|---|---|---|---|---|---|---|
| <img src="docs/img/reattore.png" width="48"> | **Reattore** | 1 | +100 | — | +40 | accende la sua rete | 0 |
| <img src="docs/img/life_support.png" width="48"> | **Life Support** | 2 | −30 | +50 | +5 | l'aria per 5 persone | 0 |
| <img src="docs/img/dormitorio.png" width="48"> | **Dormitorio** | 3 | −10 | — | +2 | 4 posti letto | 3 |
| <img src="docs/img/laboratorio.png" width="48"> | **Laboratorio** | 4 | −40 | — | +25 | impegna 2 di equipaggio; senza, non lavora e non consuma | 4 |
| <img src="docs/img/radiatore.png" width="48"> | **Radiatore** | 5 | −5 | — | −50 | il tuo migliore amico | 1 |
| <img src="docs/img/corridoio.png" width="48"> | **Corridoio** | 6 | −1 | — | — | collega; si orienta da solo | 2 |
| <img src="docs/img/batteria.png" width="48"> | **Batteria** | 7 | 0 | — | +2 | *sblocco: livello 5* | 1 |
| <img src="docs/img/serra.png" width="48"> | **Serra** | 8 | −10 | +20 | +8 | *sblocco: livello 15* | 0 |
| <img src="docs/img/gru.png" width="48"> | **Gru** | 9 | −20 | — | +5 | *sblocco: livello 25* | 4 |
| <img src="docs/img/condotto.png" width="48"> | **Condotto termico** | 0 | −15 | — | −90 | *sblocco: livello 35* | 1 |
| <img src="docs/img/centro_comando.png" width="48"> | **Centro comando** | C | −25 | — | +5 | *sblocco: livello 45* | 3 |

### Schede speciali

- **Batteria** — immagazzina fino a **150** di energia: si carica dal
  surplus della sua rete (al massimo 15 per tick) e la restituisce quando
  la rete va in deficit, *prima* che qualunque modulo si spenga. Quando
  l'HUD mostra il margine energia in negativo ma niente è in blackout,
  stai andando a batteria: goditi il tempo comprato, ma compra anche un
  reattore.
- **Serra** — fa più ossigeno per unità di energia del life support (2,0
  contro 1,67) ma meno per cella, e scalda parecchio. Ottima per
  allungare l'aria in fondo a una rete lunga.
- **Gru** — piazzala **adiacente a un detrito** e tienila accesa: dopo 12
  tick di lavoro consecutivi rimuove il detrito e si smonta da sola. Due
  celle libere al prezzo di un modulo. Se si spegne a metà, il contatore
  riparte da zero; se non ha detriti vicini, consuma e basta.
- **Condotto termico** — dissipa quasi come due radiatori in una cella
  sola, ma beve il triplo della corrente. Per stazioni dense.
- **Centro comando** — **massimo uno per stazione**: con lui attivo i
  nuovi arrivi passano da uno ogni 4 tick a uno ogni 2. Il comando non si
  divide.

### I detriti

<img src="docs/img/ostacolo.png" width="48">

Rocce alla deriva che occupano celle della griglia: non ci si costruisce,
non si rimuovono (tranne che con la Gru). Non consumano niente e non
fanno niente — ti tolgono solo lo spazio, che a fine campagna è la
risorsa più preziosa di tutte. Il log ti avvisa quando ci sono:
*"Detriti sulla griglia: costruisci intorno"*.

---

## 3. L'equipaggio della stazione

Cinque persone ti accompagnano lungo la campagna. Compaiono nei briefing
con le loro battute, e a ogni traguardo uno di loro ti consegna un modulo
nuovo. Ascoltali: dicono sempre qualcosa sul livello che stai per
giocare.

### <img src="docs/img/ingegnere.png" width="96"><br>Vera — Ingegnera di bordo

La incontri subito, al livello 2, a spiegarti che la corrente non salta
il vuoto. È lei che ti consegna la **Batteria** (livello 5: «mangia il
surplus e lo ridà quando la rete annaspa. Non è un reattore, è tempo
comprato») e il **Condotto termico** (livello 35: «dissipa quanto due
radiatori in una cella sola. Beve corrente, ma il calore non perdona»).
Se parla di calore, prendi appunti.

### <img src="docs/img/medico.png" width="96"><br>Tomas — Medico di stazione

Compare quando l'aria si fa questione seria (livelli 15 e 20). Ti
consegna la **Serra** (livello 15: «chiede meno corrente del life support
e fa un po' d'aria quasi gratis. Ma scalda, occhio»). Il suo mantra:
*«Gli incidenti capitano. Quello che conta è chi respira dopo.»*

### <img src="docs/img/caposquadra.png" width="96"><br>Dario — Caposquadra

La voce del cantiere: primo livello generato (7), campi pieni di rocce
(25), cuccette che non bastano (30). Ti consegna la **Gru** (livello 25:
«mettila accanto a una roccia e lasciala lavorare. Quando ha finito
sparisce, roccia compresa»).

### <img src="docs/img/scienziata.png" width="96"><br>Mira — Scienziata capo

Arriva tardi (livello 40) e va dritta al punto: *«I laboratori sono il
motivo per cui siamo qui. Tienimeli accesi e i punti si contano da
soli.»* Quando c'è lei nel briefing, l'obiettivo passa dai laboratori.

### <img src="docs/img/comandante.png" width="96"><br>Ilse — Comandante

Apre la campagna («Stazione nuova, equipaggio in arrivo. Tienili vivi»),
chiude il rodaggio al livello 5, e torna per il finale (45 e 50). Ti
consegna il **Centro comando** (livello 45: «la gente arriva al doppio
della velocità. Uno solo per stazione — il comando non si divide»).
L'ultima battuta della campagna è sua: *«Tutto quello che sai, tutto
insieme. Portaci a casa.»*

---

## 4. Le modalità di gioco

| Modalità | Obiettivi | Tetto tick | Classifica |
|---|---|---|---|
| **Campagna** | 50 livelli in sequenza | 400 | no (progressione) |
| **Infinita** | nessuno: resisti | nessuno | top 10 Infinita |
| **Sfida** | nessuno: punti entro il tempo | 400 | top 10 Sfida |
| **Livello casuale** | uno generato al momento | 400 | no |

Il **punteggio** è persone·tick: a ogni tick guadagni tanti punti quante
persone respirano a bordo. Infinita e Sfida hanno classifiche separate
apposta: senza tetto di tick, l'Infinita vincerebbe sempre solo durando.

Tutto si salva in file di testo semplici in
`~/.local/share/space-station/` (o `$XDG_DATA_HOME/space-station/`):
`classifica.txt`, `classifica_sfida.txt`, `progressione.txt`. Cancellarli
azzera solo quello che contengono, niente di peggio.

### La campagna in breve

Cinquanta livelli: i primi 6 disegnati a mano (uno per meccanismo), dal 7
in poi generati con seed fisso — il livello 23 è il livello 23 per tutti.
Ogni livello ha un obiettivo, spesso dei detriti, e un **budget di
moduli** (corridoi inclusi) mostrato nel briefing e nell'HUD: a budget
esaurito si rimuove col tasto destro e si riorganizza. Ogni dieci livelli,
ai traguardi 5, 15, 25, 35 e 45, si sblocca un modulo nuovo — ed è lì che
i livelli cominciano a pretenderlo. Nella griglia di selezione le frecce
su/giù saltano di riga, sinistra/destra di casella.

---

## 5. Il mercato interno

Premi **M** (o il bottone MERCATO nell'HUD) durante la partita: si apre
il mercato interno della stazione. Ogni partita offre **3 facilities**
pescate a caso dal catalogo, ognuna acquistabile **una sola volta**,
pagando in **punti** della partita: il punteggio scende di quanto spendi.
Comprare aiuta adesso e costa in classifica — è il patto.

*Nessuna valuta reale, nessuna transazione: tutto succede dentro il
gioco, tutto è gratis nel mondo vero.*

| Facility | Costo (punti) | Effetto |
|---|---|---|
| **Scorta d'ossigeno** | 80 | riserva d'ossigeno subito al massimo |
| **Spurgo termico** | 60 | azzera il surriscaldamento accumulato |
| **Ampliamento stiva** | 100 | +2 al budget moduli *(offerta solo nei livelli col budget)* |
| **Squadra di riparazione** | 120 | ripara tutte le avarie in un colpo |
| **Trasporto coloni** | 150 | +2 equipaggio subito, se ci sono posti letto |
| **Sonda demolitrice** | 200 | rimuove il detrito più vicino al centro *(offerta solo se ci sono detriti)* |

---

## 6. Comandi

| Input | Effetto |
|---|---|
| `1`–`6` | seleziona un modulo base |
| `7` `8` `9` `0` `C` | seleziona uno sbloccabile (Batteria, Serra, Gru, Condotto, Centro comando) |
| click sulla palette | come i tasti |
| click sinistro | costruisce sulla cella |
| click destro | rimuove il modulo sulla cella |
| `R` | ripara il modulo in avaria sotto il cursore |
| `Spazio` | avvia/ferma la simulazione (da fermi si costruisce senza conseguenze, l'HUD mostra l'anteprima) |
| `M` | apre il mercato interno |
| `Esc` | menu di pausa (congela anche il tempo) — anche col bottone MENU nell'HUD |
| frecce + `Invio` | navigano i menu |
| frecce nella griglia livelli | su/giù cambia riga, sinistra/destra cella |

---

## 7. Consigli di sopravvivenza

1. **Un radiatore prima del secondo reattore.** Due reattori sono +80 di
   calore: il surriscaldamento perdona 6 tick, poi rompe qualcosa a caso
   — magari proprio un radiatore.
2. **Conta l'aria: 10 a testa.** Un life support regge 5 persone, non
   una di più. Se i posti letto superano l'aria disponibile, stai
   costruendo la coda per l'obitorio.
3. **Il corridoio è il modulo migliore del gioco.** 1 di energia per
   portare la corrente ovunque. Quando il budget stringe, sostituire
   moduli con corridoi ben piazzati è quasi sempre la risposta.
4. **I laboratori si spengono da soli, e va bene così.** In blackout
   cadono per primi (priorità 4): è il gioco che protegge il life
   support. Non ricostruirli: ricollegali.
5. **La Batteria non produce niente — e vince partite.** Un picco di
   consumo (il laboratorio che si riattiva, l'avaria del reattore
   gemello) si assorbe con 150 di carica. Guarda `batt` nell'HUD.
6. **Riparare è gratis, accorgersene no.** Il tasto `R` sistema
   un'avaria all'istante: il costo vero è il tempo in cui il modulo è
   rimasto fermo senza che te ne accorgessi. Leggi il registro eventi.

---

*Le immagini di questo manuale sono generate da `tools/gen_docs_img.py`
a partire dalle stesse mappe ASCII degli sprite di gioco
(`tools/gen_sprites.py`). Se i numeri del bilanciamento cambiano
(`src/modules.rs`, `src/sim.rs`), il manuale va aggiornato a mano —
questo è il prezzo della carta stampata.*
