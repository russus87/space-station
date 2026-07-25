# STORIA — La bibbia narrativa di Space Station

> Documento per chi scrive (o riscrive) i testi del gioco: battute dei
> briefing, annunci di sblocco, intermezzi, finale. I testi vivi stanno in
> `src/personaggi.rs`; questo file spiega **perché dicono quello che
> dicono**. Se un testo nuovo contraddice questa bibbia, o si cambia la
> bibbia o si cambia il testo — mai tenere le due cose in disaccordo.

## Il tono

Ironico **e** drammatico, insieme, sempre. La regola pratica: ogni
personaggio ride per non piangere, e il giocatore deve sentire entrambe le
cose nella stessa battuta. Sarcasmo stanco di gente competente che si vuole
bene e non se lo dice. Vietati: la parodia (nessuno è un buffone), il
melodramma (nessuno recita il dolore), il cinismo vero (nessuno ha smesso
di tenerci). Se una battuta è solo divertente, manca metà. Se è solo
triste, manca l'altra metà.

## La premessa

Nel **settore K** orbitano i resti della **Aurora** — una stazione di
ricerca che dieci anni fa si è spenta in una notte sola: blackout, life
support giù, ossigeno giù. La cronaca ufficiale dice "concatenazione di
guasti". L'equipaggio la chiama **la Vecchia**, con l'affetto storto che si
riserva ai morti di famiglia.

I **detriti** su cui il giocatore costruisce non sono rocce qualsiasi:
sono i pezzi della Aurora. Il gioco lo rivela per gradi (vedi gli
intermezzi): all'inizio sono ostacoli, a metà campagna diventano reliquie,
alla fine fondamenta.

Il centro ha mandato **cinque persone** a costruire la stazione nuova
sopra il relitto della vecchia — ognuna, per motivi propri, legata alla
Aurora. Il **giocatore è "il progettista"**: mai nominato, sempre in
seconda persona. I cinque gli parlano nei briefing come si parla al
collega nuovo di cui ci si fida un po' più a ogni turno.

## I cinque, e i loro archi

| Personaggio | Chi è | Ferita | Arco (da → a) |
|---|---|---|---|
| **Vera**, ingegnera di bordo | Crede solo in ciò che si può riparare. Sarcasmo da officina. | Era l'ingegnera di turno la notte della Aurora. Non fu colpa sua; non se l'è mai perdonato lo stesso. | Dal riparare come espiazione → al costruire ridondanza come atto d'amore (la Batteria è "tempo comprato", il Condotto è il suo progetto riabilitativo: "l'ho disegnato pensando a quella notte"). |
| **Tomas**, medico di stazione | Fatalista gentile, tiene "il conto dei respiri". Umorismo da obitorio, mani da pediatra. | Nell'evacuazione della Aurora scelse chi salire per primo sulle scialuppe. | Dal contare i morti → al contare gli arrivi: il suo registro, per la prima volta, chiude "in pareggio con la vita". La Serra è la sua "prescrizione verde". |
| **Dario**, caposquadra | Misura tutto, borbotta sempre, vuole bene alla squadra in modo ruvido e totale. | Sei anni di lavoro sul Ponte 3 della Aurora — che ora è una delle "rocce" là fuori. | Dal costruire **intorno** ai problemi → al rimuoverli (la Gru): smette di girare attorno alle macerie, letteralmente e no. |
| **Mira**, scienziata capo | Precisa, riservata, è lì ufficialmente per i laboratori — in realtà per analizzare i frammenti della Aurora. | Sa (o teme di sapere) perché la Vecchia è caduta, e per metà campagna non lo dice. | Dalla ricerca come indagine privata → alla verità detta ad alta voce (intermezzo "Il referto"): la Aurora non cadde per sfortuna ma perché **i margini furono spesi per fare numeri**. Il suo monito è anche il monito del gioco: i punti non valgono un respiro. |
| **Ilse**, comandante | Voce del comando: asciutta, mai un aggettivo di troppo. | Prima ufficiale della Aurora: ordinò l'evacuazione contro l'ordine del suo comandante. Salvò quaranta persone e "perse una stazione" — è così che la chiamano ancora. | Dal comando come seconda occasione → al comando come casa: "ho perso una stazione, una volta. Questa no." Nel finale consegna la stazione al progettista. |

**Intreccio**: tutti e cinque sono reduci della Aurora o suoi orfani.
Nessuno lo dichiara al livello 1; emerge a schegge nelle battute e nei
cinque intermezzi. La rivelazione strutturale (il *perché* della caduta) è
di Mira al livello 31; la rivelazione emotiva (le macerie hanno un nome) è
di Dario al 21; la decisione (questa è casa) è di Ilse al 41.

## L'arco in dieci blocchi

Ogni blocco di 5 livelli ha un tema meccanico (SPEC-CAMPAGNA §1) e un
gradino narrativo. Gli **intermezzi** cadono ai livelli 1, 11, 21, 31, 41
(l'inizio dei blocchi dispari); il **finale** al completamento del 50.

| Blocchi | Livelli | Tema meccanico | Gradino narrativo |
|---|---|---|---|
| 1–2 | 1–10 | basi; reattori e calore | *Arrivo.* Il settore K è un cimitero con una bella vista. Presentazione dei cinque; la Aurora è nominata ma non spiegata. **Intermezzo 1, Ilse: "Ferraglia e stelle".** |
| 3–4 | 11–20 | ossigeno su reti lunghe; incidenti | *Responsabilità.* Arrivano coloni veri che si fidano senza averci visto. Tomas al centro. **Intermezzo 11, Tomas: "Il conto dei respiri".** |
| 5–6 | 21–30 | detriti; crescita rapida | *Riconoscimento.* Una "roccia" ha una targa: AURORA, PONTE 3. Dario al centro; le macerie diventano reliquie. **Intermezzo 21, Dario: "Le macerie hanno un nome".** |
| 7–8 | 31–40 | stazioni dense e calde; laboratori e punti | *Verità.* Mira consegna il referto: la Vecchia non è caduta, è stata **spesa**. Il blocco dei laboratori ha sotto questa ombra: fare punti senza rifare l'errore. **Intermezzo 31, Mira: "Il referto".** |
| 9–10 | 41–50 | colonia grande; tutto insieme | *Casa.* Ilse rifiuta il rientro: non un avamposto, una casa. Il finale chiude tutti gli archi sopra il relitto diventato fondamenta. **Intermezzo 41, Ilse: "Non un avamposto". Finale, Ilse.** |

## Il finale

La colonia sta dove stava il relitto. Nessuna apoteosi: un diario di bordo
che tira le somme, un arco chiuso per ciascuno (le macchine che "ronzano
contente" per Vera, il registro "in pareggio con la vita" per Tomas, il
borbottio "di gioia" per Dario, il referto "costruito con margine" per
Mira), e Ilse che consegna la stazione al progettista restando comunque a
bordo: *"qualcuno dovrà pur spegnere l'ultima luce. Non oggi."*

## Regole per i testi (vincolanti)

- **Battute di briefing**: max ~2 righe da ~60 caratteri. Mai numeri di
  obiettivi (i livelli 7+ sono generati). Il personaggio giusto per il
  tema del blocco; la rotazione può portare chiunque, ma il "padrone di
  casa" del blocco parla di più.
- **Intermezzi**: 4–6 righe da ~60 caratteri, voce in prima persona,
  formato "diario/registro/rapporto/nota" (ogni personaggio ha il suo).
- **La Aurora si nomina, la tragedia non si spiega mai due volte**: il
  referto di Mira (31) è l'unica esposizione completa.
- I cinque **non si insultano mai tra loro** e non fanno battute sui morti
  della Aurora: l'ironia è sul presente (macchine, turni, mensa,
  burocrazia del "centro"), il dramma è sul passato e sulla posta in gioco.
- Il giocatore è "tu"/"progettista": mai un nome, mai un genere.
