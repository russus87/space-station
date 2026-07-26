# Space Station — Guida al gioco

## Obiettivo

Costruisci una stazione spaziale piazzando moduli su una griglia e **fai
crescere l'equipaggio tenendolo vivo**: l'energia deve arrivare a tutti i
moduli attraverso la rete, l'ossigeno deve bastare per tutte le persone a
bordo, il calore va smaltito.

Il **punteggio** cresce a ogni tick di tanti punti quante sono le persone
vive a bordo: più equipaggio riesci a mantenere, e più a lungo, più punti
fai. Lo vedi in alto a destra nell'HUD.

La partita **finisce quando l'equipaggio torna a zero** dopo che almeno una
persona era salita a bordo: la stazione è persa, e la schermata finale ti
mostra punteggio, tick sopravvissuti ed equipaggio massimo raggiunto. Ogni
guasto innesca una catena di conseguenze, e il registro eventi in basso te
la racconta riga per riga.

In Campagna, Sfida e Livello casuale c'è un **tempo limite**, mostrato
come timer nell'HUD (`TEMPO 4:40`, ingiallisce e poi arrossisce): allo
scadere la partita finisce con "TEMPO SCADUTO" invece di "STAZIONE
PERSA". In Infinita non c'è: lì l'idea è proprio vedere fin dove riesci
ad arrivare, senza un orologio sopra.

In campagna il tempo fa anche le **medaglie**: finisci il livello entro
il 35% del limite ed è **oro** (su un limite di 4:40 vuol dire entro
1:38), entro il 60% **argento** (entro 2:48), entro il limite **rame**. La medaglia colora il numero del livello nella griglia di
selezione e frutta **crediti** (1/2/3, solo quando migliori) da spendere
nel **Marketplace** del titolo per comprare scorte.

## Le quattro modalità

Dal titolo scegli come giocare:

- **Campagna** — **50 livelli** in sequenza, ognuno con un obiettivo
  scritto in alto nell'HUD insieme al progresso ("equipaggio 5/8", "12/15
  tick"). I primi 6 insegnano i meccanismi uno alla volta; dal 7 in poi i
  livelli crescono di difficoltà (più detriti, budget più stretti,
  obiettivi più lunghi) e sono uguali per tutti. Il livello si vince
  raggiungendo l'obiettivo; si perde perdendo la stazione o a tetto di
  tick raggiunto, e in entrambi i casi puoi riprovare subito. Completare
  un livello **sblocca il successivo**, e il gioco se lo ricorda tra una
  sessione e l'altra. Il punteggio si vede anche qui, ma non va in
  classifica. Nella griglia di selezione le frecce su/giù saltano di riga,
  sinistra/destra di casella.
- **Infinita** — la sandbox senza obiettivi e senza tetto di tick:
  costruisci, resisti quanto riesci, fai punti. Va nella classifica
  **Infinita**.
- **Sfida** — la stessa sandbox, stesso punteggio, ma con il tetto di 400
  tick della Campagna: sai fin da subito che la partita si chiude entro un
  tempo fisso. Va in una classifica **separata**, la Sfida: i punteggi
  delle due sandbox non si mescolano, perché uno ha un limite di tempo e
  l'altro no.
- **Livello casuale** — un livello mai visto, generato al momento:
  obiettivo, detriti e budget nella fascia media della campagna. Fuori da
  progressione e classifiche; se lo perdi puoi riprovare **lo stesso**
  livello, se lo vinci te ne genera un altro.

### I primi sei livelli della campagna

Ogni livello del blocco iniziale insegna un meccanismo, nell'ordine in cui
il gioco te li fa incontrare (dal 7 al 50 si mette tutto insieme, con
difficoltà crescente):

| # | Nome | Obiettivo | Moduli |
|---|---|---|---|
| 1 | Primo respiro | avere 4 di equipaggio a bordo | 6 |
| 2 | La rete | avere 8 di equipaggio a bordo | 12 |
| 3 | Sala macchine | tenere 2 laboratori attivi per 15 tick consecutivi | 13 |
| 4 | Termica | sopravvivere 60 tick con almeno 2 laboratori attivi e zero avarie da surriscaldamento | 14 |
| 5 | Autonomia | raggiungere 400 punti senza mai andare in blackout | 15 |
| 6 | Colonia | avere 12 di equipaggio e sopravvivere 100 tick | 18 |

Ogni livello ha un **limite di moduli** (colonna "Moduli", corridoi
inclusi): lo vedi nel prologo del livello e nell'HUD accanto all'obiettivo
("moduli 7/12"). A limite raggiunto non puoi più costruire — ma rimuovere
un modulo col tasto destro libera il posto, quindi puoi sempre
riorganizzare. Il limite è tarato sull'obiettivo: c'è margine per i
corridoi e per qualche ripensamento, non per costruire a caso.

Dal livello 2 in poi alcuni livelli hanno **detriti** sulla griglia:
rocce su cui non puoi costruire e che non puoi rimuovere. Non consumano
niente e non fanno niente — ti tolgono solo spazio, e ti costringono a
far serpeggiare la stazione con i corridoi. Il log te lo dice quando ci
sono ("Detriti sulla griglia: costruisci intorno").

Due cose da sapere sugli obiettivi "a tempo": i contatori di tick
consecutivi **ripartono da zero** se la condizione decade (un laboratorio si
spegne, arriva un'avaria, va via la corrente); e nei livelli 4 e 5
l'incidente (avaria da surriscaldamento, blackout) azzera solo il progresso
— la partita continua, persa è solo se muore tutto l'equipaggio.

### La classifica

Due top 10 affiancate, una per Infinita e una per Sfida, ciascuna con
punteggio, tick, equipaggio massimo e quanto tempo fa: si apre dal titolo,
e a fine partita la schermata ti dice subito se sei entrato in classifica
(specificando in quale delle due). Sono salvate in due normali file di
testo in `~/.local/share/space-station/` (o
`$XDG_DATA_HOME/space-station/` se la variabile è impostata):
`classifica.txt` per l'Infinita, `classifica_sfida.txt` per la Sfida —
insieme a `progressione.txt` che ricorda a che punto della campagna sei.
Cancellare uno di questi file azzera solo quella classifica, niente di
peggio.

## Lo schermo

- **In alto**: l'HUD con le quattro risorse. Per ciascuna vedi produzione e
  consumo a confronto, e il margine colorato: verde va bene, giallo è un
  avvertimento, rosso è un'emergenza. In fondo alla barra, il punteggio.
- **A sinistra**: la palette degli undici moduli (tasti 1–6 e 7 8 9 0 C;
  gli sbloccabili mostrano "si sblocca al livello N" finché non li
  conquisti) con i loro costi; sotto, le **icone delle scorte** comprate
  nel Marketplace (passaci sopra per il tooltip, cliccale per usarle) e il
  pannello ispezione: punta un modulo col mouse e leggi cosa fa.
- **Al centro**: la griglia 14×8 dove costruisci.
- **In basso**: il **Registro** eventi, chiuso di default — clicca
  l'icona "Registro" per aprirlo. Ogni cosa che succede — un blackout,
  un'avaria, un morto — scrive una riga lì. È lo strumento principale per
  capire le catene di causa ed effetto.

Ogni modulo sulla griglia è riconoscibile dalla forma; il numero
nell'angolo lo collega alle righe del log ("Laboratorio 2" = il laboratorio
con il 2). Un modulo fermo mostra un simbolo che dice il perché:

| Simbolo sul modulo | Significato | Rimedio |
|---|---|---|
| Fulmine giallo | Non alimentato: l'energia della sua rete non basta | Aggiungi reattori o spegni consumi |
| Fulmine grigio | Scollegato: la sua rete non ha reattori | Collegalo al reattore con altri moduli (i corridoi costano poco) |
| Omino grigio | Fermo: manca equipaggio | Costruisci dormitori e aspetta gli arrivi |
| Triangolo rosso lampeggiante | In avaria | Puntalo e premi `R` |

## Comandi

| Input | Azione |
|---|---|
| `1`–`6` | Seleziona un modulo base dalla palette |
| `7` `8` `9` `0` `C` | Seleziona un modulo sbloccabile (Batteria, Serra, Gru, Condotto, Centro comando — si conquistano completando i livelli 5/15/25/35/45 della campagna) |
| Click sinistro | Piazza il modulo selezionato nella cella libera |
| Click destro | Rimuove il modulo sotto il cursore (perso, non recuperabile) |
| `R` | Ripara il modulo in avaria sotto il cursore (gratis) |
| `Spazio` | Avvia/ferma la simulazione |
| `V` | Cambia la velocità di gioco: ×1 → ×2 → ×4 (stesse regole, tick più rapidi; l'HUD mostra ×2/×4) |
| `Esc` | Apre il menu di pausa (riprendi, volumi di musica ed effetti, ricomincia, torna al titolo) |
| `F12` | Salva uno screenshot nella cartella corrente |

All'ingresso in un livello (campagna o casuale) c'è un **prologo a
fumetto**: compare solo la **prima volta** che incontri quel livello e si
naviga anche con `Invio` (avanti / Gioca!) e `Backspace` (indietro).
Durante la partita i personaggi **commentano gli eventi chiave** (primo
blackout, prima avaria, ossigeno critico…) con un mini-fumetto sopra il
log: sparisce da solo dopo qualche secondo.

Per moduli sbloccabili, personaggi, storia e mercato nel dettaglio c'è il
**MANUALE.md** — questa guida copre le basi.

Attenzione alla differenza: `Spazio` ferma il **tempo della stazione** — la
modalità in cui progetti con calma, mentre l'HUD ti mostra comunque
l'anteprima del bilancio. `Esc` apre il **menu** e congela tutto finché non
lo chiudi.

## La rete elettrica

L'energia **non si teletrasporta**, e — regola importante — **viaggia
solo lungo i conduttori: reattori e corridoi**. Tutti gli altri moduli
sono "foglie": si allacciano a un conduttore su una cella adiacente
(sopra/sotto/sinistra/destra, niente diagonali) ma **non prolungano la
corrente**. Una fila di dormitori NON è un cavo: il primo che tocca il
reattore funziona, il secondo è scollegato. Per allontanarti dal reattore
serve una **dorsale di corridoi**, con le foglie appese ai lati.

- Ogni reattore ha 4 facce: al massimo 4 foglie appese direttamente (una
  di solito la spendi per far partire la dorsale).
- Un modulo che non tocca nessun conduttore alimentato è **scollegato**
  (fulmine grigio): non consuma e non fa niente. È un problema diverso dal
  blackout (fulmine giallo), in cui la rete un reattore ce l'ha ma la
  corrente non basta per tutti.
- **I conduttori conducono anche in avaria** (un corridoio rotto resta un
  tubo), ma un reattore in avaria non produce.
- Il Corridoio costa solo 1 di energia e sulla griglia si orienta **da
  solo** — dritto, curva, T o incrocio — così vedi a colpo d'occhio dove
  passa la corrente.

Solo l'energia ragiona per reti: ossigeno, calore ed equipaggio sono in
comune su tutta la stazione.

## Gli imprevisti

Dal **livello 8** della campagna (e sempre nelle sandbox) lo spazio
comincia a dire la sua: 2-4 volte a partita può capitare un imprevisto.
Quelli cattivi arrivano con un **preavviso di 4 tick**: la musica si
ferma, una sirena lampeggia in alto — hai quel tempo per prepararti.

| Imprevisto | Effetto |
|---|---|
| **Meteorite** | Colpisce una cella: se c'è un modulo, avaria. Se cade nel vuoto, tiri il fiato |
| **Tempesta elettromagnetica** | Per 10 tick il surriscaldamento accelera anche a bilancio termico sano |
| **Sciame di micrometeoriti** | Spezza 1-2 corridoi: con la regola dei conduttori, mira alla dorsale |
| **Passaggio del pianeta** | L'unico bello: per 15 tick gli arrivi raddoppiano. Niente sirena, solo panorama |

C'è un periodo di grazia a inizio partita e un intervallo minimo tra un
imprevisto e l'altro: il gioco ti mette alla prova, non ti tende agguati.

## Le quattro risorse

| Risorsa | Cosa succede se va male |
|---|---|
| Energia | Viaggia solo lungo reattori e corridoi; gli altri moduli si allacciano a un conduttore adiacente. Se in una rete il consumo supera la produzione, i moduli si spengono da soli, i meno vitali per primi (blackout) |
| Ossigeno | Riserva da 0 a 100, comune a tutta la stazione. Se scende a zero, l'equipaggio muore uno alla volta |
| Calore | Se ne produci più di quanto ne dissipi, dopo 6 tick un modulo a caso va in avaria |
| Equipaggio | Arriva da solo se ci sono posti letto e aria buona; serve ai laboratori per lavorare. **Se torna a zero la partita è persa** |

## Gli undici moduli

Valori per tick di simulazione. I primi sei sono disponibili da subito;
gli altri cinque si **sbloccano completando i livelli** di campagna
indicati (e una volta conquistati valgono in tutte le modalità).

| Tasto | Modulo | Energia | Ossigeno | Calore | Note | Sblocco |
|---|---|---|---|---|---|---|
| 1 | Reattore | +100 | 0 | +40 | accende la sua rete | — |
| 2 | Life Support | −30 | +50 | +5 | si spegne per ultimo | — |
| 3 | Dormitorio | −10 | 0 | +2 | +4 posti letto | — |
| 4 | Laboratorio | −40 | 0 | +25 | richiede 2 di equipaggio; si spegne per primo | — |
| 5 | Radiatore | −5 | 0 | −50 | — | — |
| 6 | Corridoio | −1 | 0 | 0 | collega, si orienta da solo | — |
| 7 | Batteria | 0 | 0 | +1 | accumula 250 (ricarica 25/tick), copre i deficit di rete | livello 5 |
| 8 | Serra | −8 | +25 | +6 | l'ossigeno più efficiente per energia (3,1 per watt) | livello 15 |
| 9 | Gru | −20 | 0 | +5 | 12 tick e rimuove un detrito adiacente, poi si smonta | livello 25 |
| 0 | Condotto termico | −15 | 0 | −90 | radiatore pesante | livello 35 |
| C | Centro comando | −25 | 0 | +5 | max 1: arrivi ogni 2 tick invece di 4 | livello 45 |

Note:
- Un laboratorio **senza i suoi 2 membri di equipaggio non lavora e non
  consuma energia**. Occhio al rovescio della medaglia: quando l'equipaggio
  arriva, il consumo di 40 scatta di colpo.
- Ogni membro dell'equipaggio consuma **10 ossigeno per tick**: un life
  support (+50) mantiene al massimo 5 persone.
- Il corridoio è l'**unico modo di portare la corrente lontano** dal
  reattore (regola dei conduttori): la dorsale di corridoi è lo scheletro
  di ogni stazione, le foglie ci si appendono ai lati.

## Il bilancio a ogni tick

Un tick dura **0,7 secondi**. A ogni tick, in quest'ordine:

1. **Equipaggio ai laboratori**: i membri disponibili vengono assegnati ai
   laboratori in ordine di costruzione, 2 ciascuno. Chi resta senza è fermo.
2. **Energia**: si individuano le reti (i gruppi di moduli adiacenti) e in
   ciascuna si somma tutta la produzione dei suoi reattori, poi si
   alimentano i moduli della rete in ordine di importanza (life support per
   primo, poi radiatori, corridoi, dormitori, laboratori). Chi non trova
   energia resta spento: è il blackout. Chi sta in una rete senza reattori
   è scollegato, e non conta come blackout.
3. **Ossigeno**: la riserva sale della produzione dei life support attivi e
   scende di 10 per membro dell'equipaggio. Sotto 30 scatta l'allarme.
4. **Equipaggio**: con ossigeno a zero, un morto ogni 3 tick. Con posti
   letto liberi e riserva sopra 50, un nuovo arrivo ogni 4 tick.
5. **Calore**: se il netto è positivo parte il conto alla rovescia; dopo
   **6 tick consecutivi** di surriscaldamento un modulo a caso va in avaria
   (contatore visibile nell'HUD). Un modulo in avaria non fa più nulla
   finché non premi `R` sopra di lui.
6. **Punteggio**: +1 punto per ogni persona viva a bordo. Se l'equipaggio è
   tornato a zero, la partita finisce qui.

Con la simulazione ferma (Spazio) i punti 1 e 2 vengono comunque ricalcolati
come **anteprima**: l'HUD ti dice se il progetto regge prima di avviarlo.

## La cascata di guasti, raccontata

I guasti non sono mai un evento solo: sono una catena a quattro stadi.

1. **Deficit di energia** → i moduli si spengono in ordine, i laboratori per
   primi, il life support per ultimo.
2. **Life support spento** → l'ossigeno smette di rigenerarsi e la riserva
   scende.
3. **Ossigeno a zero** → l'equipaggio muore, uno ogni 3 tick.
4. **Calore in eccesso** (per esempio perché un radiatore si è spento o
   rotto) → dopo 6 tick un'avaria casuale, che può colpire un reattore o un
   life support e riportarti allo stadio 1.

### Esempio passo passo: "tolgo il reattore"

Stazione funzionante, tutta collegata: 1 reattore, 1 life support,
1 dormitorio, 1 laboratorio, 2 radiatori. Equipaggio a bordo: 4. Bilancio:
energia 100 prodotti contro 90 consumati (margine +10), ossigeno +50 contro
−40 (riserva piena a 100), calore 72 prodotti contro 100 dissipati.

Con la simulazione in corsa, rimuovi il reattore col tasto destro:

| Tick | Cosa succede | Cosa dice il log |
|---|---|---|
| 0 | La rete è rimasta senza reattore: tutti i moduli sono scollegati (fulmine grigio) | `Life Support 1 non è collegato a un reattore`, poi radiatori, dormitorio, laboratorio |
| +1 | L'ossigeno non si rigenera più: 4 persone consumano 40 a tick. Riserva 100 → 60 | — |
| +2 | Riserva 60 → 20, sotto la soglia d'allarme | `Ossigeno critico (20)` |
| +3 | Riserva 20 → 0 | `Ossigeno esaurito` |
| +6 | Primo morto per asfissia | `Equipaggio: 4 → 3 (asfissia)` |
| +9, +12 | Un morto ogni 3 tick | `… 3 → 2`, `… 2 → 1` |
| +15 | L'ultimo membro muore: la stazione è persa | `… 1 → 0 (asfissia)`, `Stazione persa: tutto l'equipaggio è morto` |

Dallo strappo alla schermata "STAZIONE PERSA": una quindicina di tick,
circa dieci secondi. Per salvarli bastava ripiazzare un reattore **attaccato
alla stazione** prima del tick +3: al tick successivo il log avrebbe scritto
`Life Support 1 collegato alla rete` e la riserva sarebbe risalita. Un
reattore piazzato in un angolo isolato, invece, non avrebbe salvato nessuno:
avrebbe fatto rete a sé.

## Errori tipici del principiante

- **Moduli piazzati lontano dal reattore.** L'energia viaggia solo tra
  moduli adiacenti: un life support piazzato in un angolo per conto suo è
  **scollegato** (fulmine grigio, `non è collegato a un reattore` nel log)
  e non produce niente. Collegalo con una fila di corridoi, o costruiscilo
  attaccato al resto.
- **Due stazioni in una.** Se dividi i moduli in due gruppi staccati, ogni
  gruppo è una rete a sé e serve un reattore per ciascuno: la corrente del
  primo non aiuta il secondo.
- **Laboratori senza dormitori.** Il laboratorio vuole 2 persone; senza
  posti letto non arriva nessuno e resta fermo per sempre
  (`fermo: equipaggio insufficiente` nel log, omino grigio sulla cella).
- **Dimenticare i radiatori.** Il solo reattore produce +40 di calore a
  tick. Senza radiatori, dopo 6 tick arriva un'avaria **casuale**: se becca
  il life support o il reattore, la cascata parte da sola. Servono radiatori
  a sufficienza da tenere il calore netto a zero o sotto.
- **Margine di energia risicato.** Se produzione e consumo sono quasi pari,
  basta un'avaria — o un laboratorio che si attiva perché è appena arrivato
  l'equipaggio (+40 di colpo) — per finire in blackout. Tieni margine
  (l'HUD passa al giallo sotto +20).
- **Più bocche che aria.** Ogni persona consuma 10 di ossigeno a tick, un
  life support ne produce 50: oltre 5 persone per life support la riserva
  scende. Se costruisci molti dormitori, costruisci anche i life support.
- **Rimuovere invece di riparare.** Il tasto destro **demolisce** il modulo,
  anche uno sano. Un modulo in avaria si ripara gratis con `R`: demolirlo è
  quasi sempre uno spreco.
- **Avviare senza guardare l'anteprima.** Da fermo, l'HUD mostra già il
  bilancio del tuo progetto. Se c'è del giallo o del rosso prima di premere
  Spazio, sistemalo prima: in pausa gli errori non costano niente.
