# SPEC — Campagna lunga, generatore e sblocchi (iterazione 4)

> **STATO: IN GRAN PARTE IMPLEMENTATA** (approvazione dell'utente arrivata a
> pezzi durante il playtest). Fatto: generatore e campagna a 50 (§2, in
> SPEC.md §13.2), livello casuale (§2.4), i 5 moduli sbloccabili (§3), i
> personaggi come voci narrative a fumetto nei briefing e negli annunci di
> sblocco (parte di §4 — i **tratti passivi** no, restano proposta), più il
> **mercato interno** (non in questo documento: catalogo e regole in
> `MANUALE.md` e `src/mercato.rs` — si paga in punti partita, niente valuta
> reale). Restano proposte: tratti dei personaggi (§4), intermezzi (§6),
> livelli curati di fine blocco, e le estensioni §9 (eventi, riparazione
> con costo, velocità, stelle, audio, sfida del giorno).

Obiettivo dell'iterazione, nelle parole del giocatore: campagna lunga
(~50 livelli) percorsa in modo lineare, sblocchi che danno cose utili ogni
5 livelli, un modo per giocare un livello singolo fuori campagna, e —
forse — un filo di storia. Questo documento trasforma quella lista in un
progetto coerente.

## 1. Il principio: gli sblocchi finanziano i livelli

Con 6 moduli e 4 risorse lo spazio di design copre 10-15 livelli davvero
diversi; oltre, i livelli diventano permutazioni. La campagna da 50 regge
solo se ogni blocco di livelli introduce materia nuova: **ogni 5 livelli
uno sblocco, e i 5 livelli successivi sono costruiti attorno a quello
sblocco**. Livelli e sblocchi non sono due feature: sono la stessa.

```
blocco  livelli   sblocco al termine        tema del blocco successivo
  1      1–5      Batteria (modulo)         gestire i picchi di energia
  2      6–10     Ingegnere (personaggio)   reattori multipli, calore
  3     11–15     Serra (modulo)            ossigeno su reti lunghe
  4     16–20     Medico (personaggio)      sopravvivere agli incidenti
  5     21–25     Gru (modulo)              griglie ostili, detriti
  6     26–30     Caposquadra (personaggio) crescita rapida equipaggio
  7     31–35     Condotto termico (modulo) stazioni dense, calore alto
  8     36–40     Scienziata (personaggio)  laboratori e punteggio
  9     41–45     Centro comando (modulo)   colonie grandi
 10     46–50     Comandante (personaggio)  tutto insieme, finale
```

I 6 livelli attuali restano i livelli 1–6 (curati, già playtestati in
parte); il primo sblocco arriva quindi a fine livello 5, in mezzo al
tutorial — presto abbastanza da far capire subito che la campagna "paga".

## 2. Livelli: ibrido curato + generato

### 2.1 Due sorgenti, una tabella

`LivelloDef` resta l'unità. Due sorgenti:

- **Curati** (a mano, nella tabella `LIVELLI` come oggi): i livelli 1–6
  esistenti e i **fine blocco** (10, 15, 20, 25, 30, 35, 40, 45, 50 — i
  livelli che introducono lo sblocco appena ottenuto vanno disegnati, non
  estratti).
- **Generati**: tutti gli altri, dal generatore parametrico con **seed
  fisso derivato dall'indice** (`seed = SEME_CAMPAGNA ^ n`): il livello 23
  è identico per tutti e a ogni avvio, si può bilanciare guardandolo.

### 2.2 Il generatore

Input: indice `n` (1–50) o seed casuale (modalità random) + curva di
difficoltà. Output: `LivelloDef` completo. Parametri generati:

- **Obiettivo**: estratto dal pool dei tipi esistenti (`Equipaggio`,
  `LabConsecutivi`, `SopravviviConLab`, `PuntiSenzaBlackout`, `Colonia`),
  pesato sul tema del blocco (sez. 1), con i numeri scalati sull'indice
  (es. equipaggio richiesto da 4 a 20, tick da 15 a 120).
- **Detriti**: quantità 0→12 crescente, disposti secondo pattern (sparsi /
  muro / diagonale / croce / doppio muro), pattern pescato dal seed.
- **Budget moduli**: `fabbisogno_minimo(obiettivo) × margine(n)`, margine
  che scende linearmente da ×1,6 (livello 7) a ×1,15 (livello 50). La
  funzione `fabbisogno_minimo` è la stessa usata nei test: unica
  definizione, generatore e verifica non possono divergere.
- **Nome e briefing**: da liste per tema ("Settore K-{n}", frasi brevi per
  blocco), niente testo generato a caso.

**Garanzia di risolvibilità** (test automatico su tutti i 50 seed della
campagna, a ogni build): budget ≥ fabbisogno minimo; detriti ≤ 12; esiste
un'area libera **connessa** ≥ budget (flood fill sulla griglia) così la
stazione ci sta fisicamente; gli obiettivi con laboratori hanno budget per
i laboratori.

### 2.3 Selezione livello e navigazione

50 voci non stanno nella lista attuale. La schermata Campagna diventa:

- **"Continua — Livello N"** in evidenza (il primo non completato).
- Sotto, una **griglia 10×5 di numeri**: completati pieni, il prossimo
  evidenziato, i bloccati spenti (stessa logica di adesso, layout a celle).
- La progressione resta lineare: si sblocca solo il successivo
  (`progressione.txt` invariato, vale 0–50).

### 2.4 Livello casuale (fuori campagna)

Nuova voce del titolo: **"Livello casuale"**. Stesso generatore, seed
casuale, difficoltà a scelta su tre livelli (Facile / Media / Difficile =
curva del livello 10 / 25 / 40). Usa solo gli sblocchi già ottenuti in
campagna (così il random non regala contenuti). Fuori classifica; a fine
partita "Rigioca (nuovo seed)" e "Torna al titolo".

## 3. ⚠ Sblocco moduli (5 nuovi)

Ogni modulo nuovo è una riga in più in `TABELLA` + eventuali regole nel
tick. Numeri di primo tentativo, da tarare giocando. Un modulo non
sbloccato non compare nella palette (né in Infinita/Sfida: gli sblocchi
valgono ovunque, la campagna è il modo di ottenerli).

| Modulo | Energia | O2 | Calore | Regola speciale | Sblocco |
|---|---|---|---|---|---|
| **Batteria** | 0 | 0 | +2 | Immagazzina fino a 150: si carica col surplus della sua rete (max 15/tick), e quando la rete va in deficit restituisce quanto ha **prima** che i moduli si spengano. | liv. 5 |
| **Serra** | −10 | +20 | +8 | Nessuna. Ossigeno più efficiente per energia del Life Support (2,0 vs 1,67 O2/energia) ma meno per cella e più caldo: conviene dove l'energia è scarsa. | liv. 15 |
| **Gru** | −20 | 0 | +5 | Dopo 12 tick attivi consecutivi **rimuove un detrito ortogonalmente adiacente** e si smonta: spariscono entrambi, due celle libere. Se non ha detriti adiacenti è solo un costo. | liv. 25 |
| **Condotto termico** | −15 | 0 | **−90** | Nessuna. Radiatore pesante: metà celle a parità di dissipazione, ma il triplo dell'energia. | liv. 35 |
| **Centro comando** | −25 | 0 | +5 | **Max 1 per stazione.** Se attivo, gli arrivi passano da 1 ogni 4 tick a 1 ogni 2. | liv. 45 |

Nota d'implementazione: la Batteria è l'unica che tocca l'allocazione
energia in `sim.rs` — la zona col bug storico documentato (somma dei
produttori in pre-passaggio). Va progettata come passo separato dopo
l'allocazione esistente (carica/scarica sul residuo di rete), senza
riordinare le fasi. La Gru richiede stato per modulo (contatore tick).

## 4. ⚠ Personaggi bonus (5)

Tratti passivi. Se ne **schiera uno solo per partita**, scelto nel
briefing (campagna) o all'avvio (Infinita/Sfida/random); il Comandante
alza il limite a due. Nessuna grafica nuova obbligatoria: nome + riga di
descrizione nella UI di scelta (ritratti pixel art rimandabili).

| Personaggio | Tratto | Tocca | Sblocco |
|---|---|---|---|
| **Ingegnere** | Il calore dei reattori scende del 25% (40→30/tick) | tabella valori al volo | liv. 10 |
| **Medico** | Asfissia dimezzata: un morto ogni 6 tick invece di 3 | `TICK_MORTE` | liv. 20 |
| **Caposquadra** | Arrivi più rapidi: 1 ogni 3 tick invece di 4 | `TICK_ARRIVO` | liv. 30 |
| **Scienziata** | I punti dei laboratori raddoppiano (vedi sez. 5) | punteggio | liv. 40 |
| **Comandante** | Puoi schierare **due** personaggi insieme | meta-regola | liv. 50 |

Implementazione: una risorsa `Squadra` con i modificatori attivi; il tick
legge i valori modificati da lì invece che dalle costanti, con default =
valori attuali. Nessun personaggio schierato = gioco identico a oggi.

## 5. ⚠ Regola base: i laboratori valgono punti

Prerequisito della Scienziata e risposta al difetto emerso al playtest
("il laboratorio a cosa serve?"): **ogni laboratorio attivo produce +5
punti/tick**, in tutte le modalità, sbloccato fin da subito. Il lab
diventa la scelta rischio/ricompensa del gioco a punteggio: più punti, ma
−40 energia, +25 calore e 2 membri di equipaggio impegnati.

Nota di compatibilità: cambia la scala dei punteggi → le classifiche
esistenti restano confrontabili solo per partite senza laboratori. Le
classifiche **non si azzerano**: la scala nuova vale da qui in avanti (le
partite vecchie in top 10 verranno superate naturalmente).

## 6. Intermezzi (storia leggera)

Una schermata di **diario di bordo** ogni 10 livelli (dopo il 10, 20, 30,
40 e prima del 50): riusa la struttura della schermata briefing, 3-4
righe di testo fisso per dare tono e progressione ("Il settore è più
denso di detriti di quanto dicessero le carte…"). Costo minimo, si scrive
in una tabella `INTERMEZZI`. **Niente** cutscene, personaggi parlanti o
ramificazioni: se la campagna lunga funzionerà, si valuterà dopo.

## 7. Persistenza e compatibilità

- `progressione.txt`: invariato (0–50). **Gli sblocchi si derivano dal
  numero**: niente file nuovo, niente migrazione, un save vecchio con "4"
  è semplicemente a metà del blocco 1.
- Classifiche: invariate (sez. 5 per la nota sulla scala punti).
- I 6 livelli attuali restano identici: nessuna rottura per chi ha già
  giocato.

## 8. Piano di implementazione (ogni fase giocabile e approvabile)

1. **Punti laboratorio** (sez. 5) — piccola, sblocca il senso del lab
   subito. ⚠
2. **Generatore + campagna a 50 + griglia selezione + livello casuale**
   (sez. 2) — nessuna regola simulata nuova, solo contenuto e UI. I
   livelli 7-50 escono generati con gli sblocchi ancora spenti (la
   campagna è giocabile per intero anche prima delle fasi 3-4).
3. **Moduli sbloccabili** (sez. 3), in quest'ordine: Serra e Condotto
   (solo tabella), poi Centro comando (regola arrivi), poi Gru (stato per
   modulo), per ultima la Batteria (tocca l'allocazione energia). ⚠ una
   per una.
4. **Personaggi** (sez. 4) + UI di schieramento. ⚠
5. **Intermezzi** (sez. 6) + eventuale ritocco curva difficoltà dopo i
   primi playtest della campagna lunga.

## 9. Oltre la campagna: profondità, non solo ampiezza

Autocritica del piano sez. 1–8: moltiplica il *contenuto* (livelli,
sblocchi) ma non le *decisioni durante la partita* — dopo la costruzione
il giocatore guarda la simulazione e può solo costruire/demolire. Le
estensioni qui sotto attaccano quel limite, in ordine di valore:

| # | Estensione | Cosa aggiunge | Costo | ⚠ |
|---|---|---|---|---|
| 9.1 | **Eventi con scelta** | Ogni 20–40 tick (frequenza dal seed) un bivio a tempo con 2 opzioni e trade-off secco: "Nave in avaria: +3 equipaggio ora, −30 ossigeno" / "Sciame di meteoriti: sacrifica un modulo a scelta o rischia un'avaria casuale". Pausa automatica all'evento; tabella `EVENTI` sul modello di `LIVELLI`. Il giocatore passa da osservatore a comandante. | medio | ⚠ |
| 9.2 | **Riparazione con costo** | Oggi la riparazione esiste ed è gratuita e istantanea (tasto `R` sul modulo in avaria): l'avaria è solo un fastidio. Proposta: riparare impegna 2 di equipaggio per 10 tick (intanto non lavorano nei lab). L'equipaggio diventa una risorsa da allocare e l'avaria una decisione. Sinergia coi personaggi (Ingegnere: riparazioni più rapide?). | medio | ⚠ |
| 9.3 | **Velocità 1×/2×/4×** | Tasti +/- o click sull'HUD: scala `TICK_SECS` a parità di regole. Qualità di vita per i livelli lunghi (100 tick a 0,7 s/tick = oltre un minuto di attesa a stazione stabile). | piccolo | — |
| 9.4 | **Stelle per livello** | 1–3 stelle: completato / sotto budget (≤ max−2 moduli) / senza morti. Mostrate nella griglia di selezione; motivo per rigiocare. Persistenza: `stelle.txt` (50 numeri 0–3). | piccolo | — |
| 9.5 | **Audio generato** | Il gioco è muto. WAV generati da script Python (stessa filosofia degli sprite: fonte testuale, zero asset esterni): allarme, click di costruzione, avaria, ronzio ambiente. | medio | — |
| 9.6 | **Sfida del giorno** | Seed derivato dalla data (AAAAMMGG): livello uguale per tutti, una classifica locale dedicata ("oggi"). Quasi gratis una volta fatto il generatore. | piccolo | — |

Collocazione nel piano (sez. 8): 9.3 e 9.4 entrano in fase 2 (sono
contenuto/UI); 9.1 e 9.2 sono una **fase 6** a sé, dopo che la campagna
regge; 9.5 e 9.6 quando si vuole (indipendenti da tutto).

**Cosa NON aggiungere**, per non diluire: valute o meta-progressione oltre
agli sblocchi già previsti; multiplayer/classifiche online; modalità
ulteriori oltre le quattro (Campagna, Infinita, Sfida, Casuale + eventuale
giornaliera). Prima si approfondisce, poi — semmai — si allarga ancora.

## 10. Rischi dichiarati

- **Bilanciamento a tavolino**: tutti i numeri di questo documento sono
  primi tentativi; la curva del generatore va tarata giocando (il budget
  ×1,15 al livello 50 potrebbe essere impossibile o banale).
- **La Batteria tocca la zona più delicata di `sim.rs`**: da fare per
  ultima, con test dedicati sull'allocazione.
- **Il Comandante** (due personaggi) moltiplica le combinazioni da
  testare: 10 coppie. Se in playtest emergono coppie rotte (Medico +
  Caposquadra rende quasi immortali?), si vieta la coppia, non si
  ribilancia tutto.
- **50 livelli sono tanti da percorrere anche da vincitore**: se al
  playtest il ritmo crolla a metà campagna, meglio tagliare a 30 con gli
  stessi 10 sblocchi (uno ogni 3) che allungare il brodo. Il numero 50
  non è sacro.
