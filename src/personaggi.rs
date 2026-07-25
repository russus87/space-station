//! I personaggi della campagna, le loro battute a fumetto e la storia.
//!
//! Tabelle costanti sul modello di `LIVELLI` e `TABELLA`: è QUI (e solo qui)
//! che vivono nomi, ruoli, battute, intermezzi e finale. Il *perché* dicono
//! quello che dicono sta in `STORIA.md` (la bibbia narrativa): se un testo
//! nuovo la contraddice, si cambia uno dei due — mai tenerli in disaccordo.
//! Le battute agganciano i temi dei blocchi senza citare numeri: i livelli
//! dal 7 in poi sono generati e i numeri cambiano col seed.
//! I ritratti stanno in `Art::ritratti`, indicizzati da `Personaggio::ritratto`
//! nello stesso ordine di `PERSONAGGI`.

pub struct Personaggio {
    pub nome: &'static str,
    pub ruolo: &'static str,
    /// Indice del ritratto in `Art::ritratti`.
    pub ritratto: usize,
}

/// Ordine fissato dal caricamento di `Art::ritratti` in `main.rs`:
/// ingegnere, medico, caposquadra, scienziata, comandante.
pub const PERSONAGGI: [Personaggio; 5] = [
    Personaggio {
        nome: "Vera",
        ruolo: "Ingegnera di bordo",
        ritratto: 0,
    },
    Personaggio {
        nome: "Tomas",
        ruolo: "Medico di stazione",
        ritratto: 1,
    },
    Personaggio {
        nome: "Dario",
        ruolo: "Caposquadra",
        ritratto: 2,
    },
    Personaggio {
        nome: "Mira",
        ruolo: "Scienziata capo",
        ritratto: 3,
    },
    Personaggio {
        nome: "Ilse",
        ruolo: "Comandante",
        ritratto: 4,
    },
];

// indici in PERSONAGGI, per leggere le tabelle senza contare
const INGEGNERA: usize = 0;
const MEDICO: usize = 1;
const CAPOSQUADRA: usize = 2;
const SCIENZIATA: usize = 3;
const COMANDANTE: usize = 4;

/// Chi parla nel briefing di quale livello (1-based), e cosa dice.
/// Copertura completa 1..=50: ogni livello ha la sua voce, a rotazione sul
/// tema del blocco (il "padrone di casa" del blocco parla di più) e
/// sull'arco di `STORIA.md`. La vecchia stazione Aurora — "la Vecchia" —
/// si nomina, la sua fine si spiega una volta sola (intermezzo del 31).
const BATTUTE: [(usize, usize, &str); 50] = [
    // ---- blocco 1 (1-5): le basi. si presentano tutti e cinque ----
    (
        1,
        CAPOSQUADRA,
        "Reattore, aria, letti. In quest'ordine. Il resto è poesia, \
         e la poesia non respira.",
    ),
    (
        2,
        INGEGNERA,
        "La corrente non salta il vuoto: o moduli attaccati, o corridoi. \
         Una rete senza reattore è ferraglia fredda.",
    ),
    (
        3,
        SCIENZIATA,
        "I laboratori vogliono gente, non solo corrente. La scienza \
         si fa con le mani. E le mani respirano.",
    ),
    (
        4,
        MEDICO,
        "Il colpo di calore è una diagnosi noiosa. Preferisco non \
         farla mai: radiatori, e non se ne parla più.",
    ),
    (
        5,
        COMANDANTE,
        "Resta in piedi senza mai restare al buio. Chi tiene il \
         margine tiene tutto: fine del rodaggio.",
    ),
    // ---- blocco 2 (6-10): reattori e calore. casa di Vera ----
    (
        6,
        COMANDANTE,
        "Adesso tienila in piedi davvero. Se sembra un ordine è \
         perché lo è. Ma è anche una preghiera: non dirlo alla squadra.",
    ),
    (
        7,
        CAPOSQUADRA,
        "Settore nuovo, carte vecchie: i detriti non sono segnati. \
         Costruisci dove c'è spazio, non dove è comodo.",
    ),
    (
        8,
        INGEGNERA,
        "Secondo reattore: doppio orgoglio, doppio calore. L'orgoglio \
         non si dissipa. Il calore sì, per fortuna.",
    ),
    (
        9,
        MEDICO,
        "Più corrente, più moduli, più gente. Più gente, più aria. \
         Il mio lavoro è ricordartelo prima, non dopo.",
    ),
    (
        10,
        INGEGNERA,
        "Due reattori scaldano più del doppio di uno, non chiedermi \
         la formula. Radiatori pronti, poi accendi.",
    ),
    // ---- blocco 3 (11-15): ossigeno su reti lunghe. casa di Tomas ----
    (
        11,
        MEDICO,
        "Le reti si allungano e l'aria resta indietro. Un life support \
         in fondo alla fila salva più vite del mio bisturi.",
    ),
    (
        12,
        INGEGNERA,
        "Corridoi: un filo di corrente e tanta pazienza. La pazienza \
         almeno non consuma ossigeno.",
    ),
    (
        13,
        CAPOSQUADRA,
        "La squadra dorme stretta e lavora larga. Se l'aria cala, \
         prima i letti, poi le lamentele.",
    ),
    (
        14,
        MEDICO,
        "'Ossigeno critico' è un termine tecnico. Quello non tecnico \
         non te lo dico: si spaventano tutti.",
    ),
    (
        15,
        MEDICO,
        "L'aria non viaggia da sola: qualcuno la fa, tutti la \
         consumano. Portala fino all'ultimo modulo della fila.",
    ),
    // ---- blocco 4 (16-20): incidenti. ancora Tomas, con Vera e Mira ----
    (
        16,
        MEDICO,
        "Gli incidenti capitano. Quello che conta è chi respira dopo.",
    ),
    (
        17,
        INGEGNERA,
        "Un'avaria non è un tradimento: è una macchina stanca. \
         Riparala, e chiedile scusa da parte mia.",
    ),
    (
        18,
        SCIENZIATA,
        "Ho letto i log della Vecchia. I guasti non arrivano mai \
         soli: arrivano in fila. Spezza la fila presto.",
    ),
    (
        19,
        CAPOSQUADRA,
        "Il turno di notte dice che il radiatore fischia. I radiatori \
         non fischiano: è il margine che se ne sta andando.",
    ),
    (
        20,
        MEDICO,
        "Oggi il registro segna zero incidenti. Facciamo che resti \
         così: è l'unica pagina che amo bianca.",
    ),
    // ---- blocco 5 (21-25): detriti. casa di Dario ----
    (
        21,
        CAPOSQUADRA,
        "Quelle rocce là fuori hanno i bulloni. Le rocce non hanno \
         bulloni. Costruiamo intorno, e non guardate troppo.",
    ),
    (
        22,
        SCIENZIATA,
        "Ogni frammento che scanso è il pezzo di qualcuno che ha \
         sbagliato prima di noi. Io catalogo. Non giudico.",
    ),
    (
        23,
        CAPOSQUADRA,
        "Campo stretto, squadra larga: si serpeggia. Non è elegante, \
         ma l'eleganza non paga l'ossigeno.",
    ),
    (
        24,
        INGEGNERA,
        "Costruire tra i detriti è riparare col motore acceso. \
         Si può fare. Si deve, pure.",
    ),
    (
        25,
        CAPOSQUADRA,
        "La squadra borbotta, io misuro. Le macerie non si spostano \
         coi borbottii. Con altro, forse, presto.",
    ),
    // ---- blocco 6 (26-30): crescita. Dario e i nuovi arrivi ----
    (
        26,
        CAPOSQUADRA,
        "Arrivano a ondate, e ognuno porta fame d'aria e di letto. \
         Costruisci come se ti guardassero: ti guardano.",
    ),
    (
        27,
        MEDICO,
        "Nuovi arrivi: visite, vaccini, e la domanda che non fanno \
         mai ad alta voce: 'è sicuro, qui?'. Rispondi tu.",
    ),
    (
        28,
        COMANDANTE,
        "Una stazione piena è un successo e un rischio con lo stesso \
         nome. Cresci pure. Ma cresci in ordine.",
    ),
    (
        29,
        CAPOSQUADRA,
        "Non farmi scegliere chi dorme in corridoio. L'ho già fatto \
         una volta, su un'altra stazione. Non lo rifaccio.",
    ),
    (
        30,
        CAPOSQUADRA,
        "Cuccette, aria, corrente: le lamentele arrivano in \
         quest'ordine. Falle sparire nello stesso ordine.",
    ),
    // ---- blocco 7 (31-35): denso e caldo. Vera, con l'ombra del referto ----
    (
        31,
        SCIENZIATA,
        "Il referto sulla Vecchia è pronto, e non ti piacerà. \
         Ne parliamo a fine turno. Intanto: margini.",
    ),
    (
        32,
        INGEGNERA,
        "Stazione densa: bella, calda, pericolosa. Come certe idee. \
         Dissipa entrambe per tempo.",
    ),
    (
        33,
        MEDICO,
        "Il caldo uccide piano e senza rumore. Preferisco i problemi \
         maleducati: almeno bussano prima di entrare.",
    ),
    (
        34,
        INGEGNERA,
        "Ogni modulo in più è una stufa con un secondo lavoro. \
         Dissipare non è un lusso: è memoria.",
    ),
    (
        35,
        INGEGNERA,
        "Qui dentro fa caldo come in sala macchine, quella notte. \
         Lo dico da sola, piano: non finirà allo stesso modo.",
    ),
    // ---- blocco 8 (36-40): laboratori e punti. casa di Mira ----
    (
        36,
        SCIENZIATA,
        "I laboratori accesi sono il motivo per cui il centro paga \
         il carburante. Il motivo per cui restiamo è un altro.",
    ),
    (
        37,
        SCIENZIATA,
        "La ricerca non aspetta, ma non respira nemmeno per te. \
         Prima l'aria, poi i dati. Me lo ripeto ogni giorno.",
    ),
    (
        38,
        COMANDANTE,
        "Il centro vuole numeri. Io voglio nomi, tutti interi a fine \
         turno. Dammi entrambi e non litighiamo mai.",
    ),
    (
        39,
        SCIENZIATA,
        "Ogni punto in archivio è un'ora della vita di qualcuno. \
         Spendili come se costassero. Costano.",
    ),
    (
        40,
        SCIENZIATA,
        "Tienimi i laboratori accesi: è lì che la Vecchia smette \
         di essere un relitto e diventa una lezione.",
    ),
    // ---- blocco 9 (41-45): colonia. casa di Ilse ----
    (
        41,
        COMANDANTE,
        "Da oggi vietata la parola 'avamposto'. Si chiama casa. \
         E le case non si evacuano: si difendono.",
    ),
    (
        42,
        MEDICO,
        "Dodici cartelle cliniche. Tredici, se conti la stazione: \
         è il paziente che preferisco, non si lamenta mai.",
    ),
    (
        43,
        CAPOSQUADRA,
        "Una colonia vera: letti veri, turni veri, litigi veri in \
         mensa. Vuol dire che funziona.",
    ),
    (
        44,
        SCIENZIATA,
        "Un giorno i bambini chiederanno cosa c'era qui prima. \
         Sto scrivendo la risposta. Sarà onesta.",
    ),
    (
        45,
        COMANDANTE,
        "Comandala come una colonia: margini larghi, nervi saldi. \
         Il resto l'hai già imparato — l'ho visto.",
    ),
    // ---- blocco 10 (46-50): tutto insieme. il congedo dei cinque ----
    (
        46,
        INGEGNERA,
        "Ultimo blocco. Le macchine lo sentono, ronzano diverse. \
         O forse sono io. Tu controlla i radiatori comunque.",
    ),
    (
        47,
        MEDICO,
        "Ho smesso di contare i respiri: ora conto le risate in \
         mensa. Statistica ufficiale: in crescita.",
    ),
    (
        48,
        CAPOSQUADRA,
        "La squadra non borbotta quasi più. Mi manca, un po'. \
         Chiudiamo bene: così borbotto io, di gioia.",
    ),
    (
        49,
        SCIENZIATA,
        "Il referto finale dirà: 'costruito con margine'. \
         È la frase più romantica che io conosca.",
    ),
    (
        50,
        COMANDANTE,
        "Ultimo settore. Tutto quello che sai, tutto insieme. \
         Portaci a casa.",
    ),
];

/// Chi presenta il modulo appena sbloccato, a livello completato.
/// I livelli sono quelli dei traguardi: 5=Batteria, 15=Serra, 25=Gru,
/// 35=Condotto termico, 45=Centro comando. Le voci seguono gli archi di
/// `STORIA.md`: ogni sblocco è un pezzo della storia di chi lo consegna.
const SBLOCCHI: [(usize, usize, &str); 5] = [
    (
        5,
        INGEGNERA,
        "Ti ho montato una Batteria: mangia il surplus e lo ridà \
         quando la rete annaspa. Sulla Vecchia ne avessimo avuta una.",
    ),
    (
        15,
        MEDICO,
        "La Serra è pronta. La chiamo 'prescrizione verde': poca \
         corrente, un filo d'aria, e la mensa smette di sapere di lattina.",
    ),
    (
        25,
        CAPOSQUADRA,
        "La Gru è operativa: mettila accanto a una roccia e lasciala \
         lavorare. Quando finisce si smonta da sola. Senza borbottare.",
    ),
    (
        35,
        INGEGNERA,
        "Il Condotto termico: dissipa come due radiatori in una cella. \
         L'ho disegnato pensando a quella notte. Funziona.",
    ),
    (
        45,
        COMANDANTE,
        "Il Centro comando è tuo: la gente arriverà al doppio della \
         velocità. Uno per stazione. Il comando non si divide: lo so bene.",
    ),
];

/// La battuta del briefing del livello (1-based), se il livello ne ha una:
/// `(indice in PERSONAGGI, testo)`.
pub fn battuta_briefing(livello: usize) -> Option<(usize, &'static str)> {
    BATTUTE
        .iter()
        .find(|(l, _, _)| *l == livello)
        .map(|&(_, p, t)| (p, t))
}

/// L'annuncio del modulo sbloccato completando `livello_completato`
/// (1-based), se quel livello è un traguardo di sblocco.
pub fn annuncio_sblocco(livello_completato: usize) -> Option<(usize, &'static str)> {
    SBLOCCHI
        .iter()
        .find(|(l, _, _)| *l == livello_completato)
        .map(|&(_, p, t)| (p, t))
}

// ---------------- intermezzi ----------------

/// Una pagina di storia prima del briefing del suo livello: la svolta del
/// blocco che comincia, in prima persona, nel formato del personaggio
/// (diario, registro, rapporto, nota). Vedi `STORIA.md`.
pub struct Intermezzo {
    pub livello: usize,
    pub titolo: &'static str,
    pub personaggio: usize,
    pub testo: &'static str,
}

/// Cinque svolte: arrivo, responsabilità, riconoscimento, verità, casa.
pub const INTERMEZZI: [Intermezzo; 5] = [
    Intermezzo {
        livello: 1,
        titolo: "Ferraglia e stelle",
        personaggio: COMANDANTE,
        testo: "Diario del comandante, giorno uno.\n\
                Il settore K è un cimitero con una bella vista: là fuori\n\
                gira ancora quel che resta della vecchia Aurora.\n\
                Il centro ci manda a costruire sopra i suoi pezzi.\n\
                Cinque persone, una lista di errori da non rifare.\n\
                Cominciamo dal respiro.",
    },
    Intermezzo {
        livello: 11,
        titolo: "Il conto dei respiri",
        personaggio: MEDICO,
        testo: "Registro medico, settimana quattro.\n\
                Arrivano trasporti pieni di gente che si fida di noi\n\
                senza averci mai visto in faccia. Incoscienti. Benvenuti.\n\
                Ogni nome nuovo è aria che oggi non abbiamo ancora.\n\
                Io tengo il conto dei respiri: fa' che il conto torni.\n\
                È tutto quello che chiedo. Quasi.",
    },
    Intermezzo {
        livello: 21,
        titolo: "Le macerie hanno un nome",
        personaggio: CAPOSQUADRA,
        testo: "Rapporto squadra, turno lungo.\n\
                Oggi una delle 'rocce' aveva una targa: AURORA, PONTE 3.\n\
                Ho lavorato sei anni su quel ponte.\n\
                La squadra ha fatto silenzio, poi ha ripreso a battere.\n\
                È il nostro modo di pregare, credo.\n\
                Costruiamo intorno. Ma adesso so intorno a cosa.",
    },
    Intermezzo {
        livello: 31,
        titolo: "Il referto",
        personaggio: SCIENZIATA,
        testo: "Nota di ricerca, riservata. Da oggi non più.\n\
                Ho finito l'analisi dei frammenti: nessun sabotaggio,\n\
                nessuna sfortuna. Solo margini rosicchiati per fare\n\
                numeri, un turno dopo l'altro, finché è bastato un guasto.\n\
                La Vecchia non è caduta: è stata spesa.\n\
                Noi no. Firmato: Mira.",
    },
    Intermezzo {
        livello: 41,
        titolo: "Non un avamposto",
        personaggio: COMANDANTE,
        testo: "Diario del comandante, giorno che ho smesso di contare.\n\
                Il centro chiede quando rientriamo. Ho risposto oggi:\n\
                non rientriamo. Qui la gente pianta serre, litiga in\n\
                mensa, dà i nomi ai corridoi.\n\
                Ho perso una stazione, una volta. Questa no.\n\
                Questa è casa, e le case si difendono.",
    },
];

/// L'intermezzo che apre `livello`, se c'è.
pub fn intermezzo_per(livello: usize) -> Option<&'static Intermezzo> {
    INTERMEZZI.iter().find(|i| i.livello == livello)
}

/// La chiusura della campagna, mostrata completando il livello 50:
/// `(indice in PERSONAGGI, testo)`. Chiude tutti gli archi di `STORIA.md`.
pub const FINALE: (usize, &'static str) = (
    COMANDANTE,
    "Diario del comandante, ultimo settore. Fine.\n\
     Ce l'abbiamo fatta. Lo scrivo piano, che non si rompa.\n\
     Vera dice che le macchine ronzano contente: le credo.\n\
     Tomas ha chiuso il registro in pareggio con la vita.\n\
     Dario ha smesso di borbottare. No: borbotta di gioia.\n\
     Mira ha firmato il referto nuovo: 'costruito con margine'.\n\
     Dove c'era la Vecchia adesso c'è luce ai finestrini.\n\
     Non abbiamo dimenticato niente: ci abbiamo costruito sopra.\n\
     La stazione è tua, progettista. Io resto lo stesso.\n\
     Qualcuno dovrà pur spegnere l'ultima luce. Non oggi.",
);

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indici_personaggio_validi_in_tutte_le_tabelle() {
        for &(_, p, _) in BATTUTE.iter().chain(&SBLOCCHI) {
            assert!(p < PERSONAGGI.len());
        }
        for i in &INTERMEZZI {
            assert!(i.personaggio < PERSONAGGI.len());
        }
        assert!(FINALE.0 < PERSONAGGI.len());
        assert!(!FINALE.1.is_empty());
        for (i, p) in PERSONAGGI.iter().enumerate() {
            assert_eq!(p.ritratto, i, "{}: ritratto fuori ordine", p.nome);
        }
    }

    #[test]
    fn livelli_delle_battute_dentro_la_campagna() {
        for &(l, _, _) in BATTUTE.iter().chain(&SBLOCCHI) {
            assert!((1..=50).contains(&l), "livello {l} fuori range");
        }
    }

    #[test]
    fn copertura_briefing_su_primo_contatto_e_traguardi() {
        // dopo la bibbia narrativa, la copertura è totale: un briefing
        // muto sarebbe un buco di storia
        for livello in 1..=50 {
            assert!(
                battuta_briefing(livello).is_some(),
                "livello {livello} senza battuta"
            );
        }
        assert!(battuta_briefing(0).is_none());
        assert!(battuta_briefing(51).is_none());
        // nessun livello doppio in tabella: vincerebbe sempre il primo
        for livello in 1..=50 {
            assert_eq!(
                BATTUTE.iter().filter(|(l, _, _)| *l == livello).count(),
                1,
                "livello {livello} duplicato in BATTUTE"
            );
        }
    }

    #[test]
    fn ogni_traguardo_di_sblocco_ha_il_suo_annuncio() {
        for livello in [5, 15, 25, 35, 45] {
            assert!(
                annuncio_sblocco(livello).is_some(),
                "sblocco del livello {livello} senza annuncio"
            );
        }
        assert!(annuncio_sblocco(10).is_none());
        assert!(annuncio_sblocco(50).is_none());
    }

    #[test]
    fn intermezzi_ai_livelli_di_svolta_e_solo_li() {
        let attesi = [1, 11, 21, 31, 41];
        assert_eq!(INTERMEZZI.len(), attesi.len());
        for (intermezzo, atteso) in INTERMEZZI.iter().zip(attesi) {
            assert_eq!(intermezzo.livello, atteso);
            assert!(!intermezzo.titolo.is_empty());
            assert!(!intermezzo.testo.is_empty());
            assert!(intermezzo_per(atteso).is_some());
        }
        for livello in [2, 5, 10, 20, 30, 40, 50] {
            assert!(intermezzo_per(livello).is_none());
        }
    }

    #[test]
    fn ogni_personaggio_parla_almeno_cinque_volte() {
        // la storia è corale: nessuno dei cinque può sparire
        for (i, p) in PERSONAGGI.iter().enumerate() {
            let battute = BATTUTE.iter().filter(|&&(_, chi, _)| chi == i).count();
            assert!(battute >= 5, "{} parla solo {battute} volte", p.nome);
        }
    }
}
