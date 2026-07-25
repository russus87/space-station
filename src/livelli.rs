//! Gamification: modalità di gioco, livelli della campagna, classifica locale
//! e progressione persistente.
//!
//! La tabella `LIVELLI` è l'UNICO punto dove si tarano nomi, briefing e numeri
//! degli obiettivi, sul modello di `TABELLA` in `modules.rs`. I testi degli
//! obiettivi sono generati dai numeri (`Obiettivo::descrizione`), così non
//! esistono due copie da tenere allineate.
//!
//! Persistenza: due file di testo semplice in
//! `$XDG_DATA_HOME/space-station/` (ripiego `~/.local/share/space-station/`),
//! scritti SOLO a fine partita o a completamento livello, mai a ogni tick.
//! Formato volutamente a prova di mano umana: una riga malformata si salta,
//! un file assente o illeggibile equivale a "nessun dato", senza errori.

use crate::generatore;
use crate::menu::AppState;
use crate::modules::ModuleKind;
use crate::sim::{EventLog, Module, Sim};
use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Quante partite tiene la classifica.
pub const TOP_N: usize = 10;

// ---------------- modalità ----------------

/// In che modalità si sta giocando. In `Infinita` il codice degli obiettivi
/// non gira proprio (run condition `campagna_attiva`) e il punteggio entra in
/// classifica; in `Campagna` vale l'obiettivo del livello e la classifica
/// non si tocca.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modalita {
    #[default]
    Infinita,
    /// Come `Infinita` ma con il tetto di tick di `sim::TICK_MASSIMO`: partite
    /// più brevi e comparabili fra loro. Classifica separata apposta: un
    /// punteggio di `Infinita` non è confrontabile con uno di `Sfida`, la
    /// prima può sempre vincere restando in piedi più a lungo.
    Sfida,
    /// Indice 0-based nel vettore `LIVELLI`.
    Campagna(usize),
    /// Un livello generato con seed casuale (`LivelloCasuale`): obiettivo
    /// attivo come in campagna, ma fuori da progressione e classifiche.
    Casuale,
}

/// Il livello della modalità casuale in corso. `None` fuori da `Casuale`;
/// resta valorizzato durante la partita così "Riprova" rigioca lo stesso.
#[derive(Resource, Default)]
pub struct LivelloCasuale(pub Option<LivelloDef>);

/// Run condition: gli obiettivi esistono in campagna e nel livello casuale.
pub fn obiettivi_attivi(modalita: Res<Modalita>) -> bool {
    matches!(*modalita, Modalita::Campagna(_) | Modalita::Casuale)
}

/// Livello selezionato nelle schermate di campagna (selezione → briefing).
#[derive(Resource, Default)]
pub struct LivelloScelto(pub usize);

/// Stato di avanzamento del livello in corso. Vive QUI, non in `Sim`:
/// la simulazione non sa niente degli obiettivi. Si azzera a ogni reset.
#[derive(Resource, Default)]
pub struct StatoLivello {
    /// Ultimo tick già valutato: l'obiettivo avanza una volta per tick.
    pub ultimo_tick: u64,
    /// Contatore di progresso: tick consecutivi o punti, secondo l'obiettivo.
    /// Si azzera quando la condizione decade (lab spenti, blackout, avaria).
    pub progresso: u64,
    /// Avarie da surriscaldamento già viste (per accorgersi delle nuove).
    pub avarie_viste: u32,
    /// Laboratori attivi adesso, per l'HUD (aggiornato anche in anteprima).
    pub lab_attivi: u32,
    pub completato: bool,
}

/// Ultimo livello completato (1-based: 0 = nessuno). Persistito.
#[derive(Resource, Default)]
pub struct Progressione {
    pub completati: usize,
}

/// Posizione in classifica dell'ultima partita infinita (None = fuori top 10
/// o partita di campagna). La legge la schermata di fine partita.
#[derive(Resource, Default)]
pub struct UltimoPiazzamento(pub Option<usize>);

// ---------------- livelli ----------------

/// Un obiettivo misurabile. I numeri stanno nella tabella `LIVELLI`.
#[derive(Clone, Copy)]
pub enum Obiettivo {
    /// Avere almeno N di equipaggio a bordo.
    Equipaggio { minimo: u32 },
    /// Tenere N laboratori attivi per M tick consecutivi.
    LabConsecutivi { laboratori: u32, tick: u32 },
    /// Sopravvivere M tick con N laboratori attivi e zero avarie da
    /// surriscaldamento: lab spenti o avaria azzerano il progresso.
    SopravviviConLab { laboratori: u32, tick: u32 },
    /// Accumulare N punti senza mai andare in blackout: il blackout azzera
    /// il progresso (non fa perdere la partita).
    PuntiSenzaBlackout { punti: u64 },
    /// Avere N di equipaggio e sopravvivere M tick.
    Colonia { equipaggio: u32, tick: u64 },
}

#[derive(Clone)]
pub struct LivelloDef {
    pub nome: String,
    /// Una riga, mostrata prima di iniziare il livello.
    pub briefing: String,
    pub obiettivo: Obiettivo,
    /// Celle occupate da detriti: non ci si costruisce, non si rimuovono.
    /// Coordinate 0-based della griglia 14×8, origine in basso a sinistra.
    /// La simulazione non li vede: sono solo un vincolo di piazzamento.
    pub ostacoli: Vec<(i32, i32)>,
    /// Quanti moduli si possono costruire in questo livello (i corridoi
    /// contano). Tarato sull'obiettivo: fabbisogno minimo della soluzione
    /// più ovvia + margine per corridoi ed errori, così il livello chiede
    /// efficienza senza diventare un incastro unico obbligato. Solo
    /// campagna e casuale: Infinita e Sfida restano senza limite.
    pub max_moduli: u32,
}

/// I 50 livelli della campagna: i primi 6 curati a mano (ordine didattico,
/// ognuno insegna un meccanismo; i detriti entrano dal livello 2 e i
/// livelli d'esordio di un meccanismo nuovo restano a griglia libera), dal
/// 7 al 50 generati da `generatore::genera_campagna` con seed fisso per
/// indice: il livello 23 è identico per tutti e a ogni avvio.
pub static LIVELLI: LazyLock<Vec<LivelloDef>> = LazyLock::new(|| {
    let mut livelli = livelli_curati();
    for n in 7..=50 {
        livelli.push(generatore::genera_campagna(n));
    }
    livelli
});

fn livelli_curati() -> Vec<LivelloDef> {
    vec![
        LivelloDef {
            nome: "Primo respiro".into(),
            briefing: "Un reattore, un life support, un dormitorio: attaccati tra loro.".into(),
            obiettivo: Obiettivo::Equipaggio { minimo: 4 },
            ostacoli: vec![],
            // fabbisogno minimo 4 (reattore, life support, dormitorio, radiatore)
            max_moduli: 6,
        },
        LivelloDef {
            nome: "La rete".into(),
            briefing: "Un solo reattore non basta più: allunga con i corridoi o costruisci una seconda rete.".into(),
            obiettivo: Obiettivo::Equipaggio { minimo: 8 },
            // muro verticale al centro, aggirabile in alto e in basso
            ostacoli: vec![(6, 2), (6, 3), (6, 4), (6, 5)],
            // fabbisogno minimo 7, più i corridoi per aggirare il muro
            max_moduli: 12,
        },
        LivelloDef {
            nome: "Sala macchine".into(),
            briefing: "I laboratori non vogliono corrente: vogliono gente.".into(),
            obiettivo: Obiettivo::LabConsecutivi {
                laboratori: 2,
                tick: 15,
            },
            ostacoli: vec![],
            // fabbisogno minimo 9 (2 reattori, life support, dormitorio, 2 lab, 3 radiatori)
            max_moduli: 13,
        },
        LivelloDef {
            nome: "Termica".into(),
            briefing: "Tutto quello che lavora scalda.".into(),
            obiettivo: Obiettivo::SopravviviConLab {
                laboratori: 2,
                tick: 60,
            },
            // detriti sparsi: spezzano i blocchi compatti, i radiatori vanno incastrati
            ostacoli: vec![(2, 5), (4, 1), (7, 4), (10, 6), (11, 2)],
            // come Sala macchine, con un radiatore di margine per le avarie
            max_moduli: 14,
        },
        LivelloDef {
            nome: "Autonomia".into(),
            briefing: "Il margine serve a questo: a non restare mai al buio.".into(),
            obiettivo: Obiettivo::PuntiSenzaBlackout { punti: 400 },
            ostacoli: vec![],
            // stazione libera, ma il margine energetico va costruito con poco
            max_moduli: 15,
        },
        LivelloDef {
            nome: "Colonia".into(),
            briefing: "Adesso tienila in piedi davvero.".into(),
            obiettivo: Obiettivo::Colonia {
                equipaggio: 12,
                tick: 100,
            },
            // fascia diagonale: la stazione grande va fatta serpeggiare
            ostacoli: vec![(4, 6), (5, 5), (6, 4), (7, 3), (8, 2), (9, 1)],
            // fabbisogno minimo 11, più i corridoi per serpeggiare tra i detriti
            max_moduli: 18,
        },
    ]
}

impl Obiettivo {
    /// L'obiettivo scritto per esteso (briefing, schermate).
    pub fn descrizione(&self) -> String {
        match *self {
            Obiettivo::Equipaggio { minimo } => {
                format!("avere {minimo} di equipaggio a bordo")
            }
            Obiettivo::LabConsecutivi { laboratori, tick } => {
                format!("tenere {laboratori} laboratori attivi per {tick} tick consecutivi")
            }
            Obiettivo::SopravviviConLab { laboratori, tick } => format!(
                "sopravvivere {tick} tick con almeno {laboratori} laboratori attivi \
                 e zero avarie da surriscaldamento"
            ),
            Obiettivo::PuntiSenzaBlackout { punti } => {
                format!("raggiungere {punti} punti senza mai andare in blackout")
            }
            Obiettivo::Colonia { equipaggio, tick } => {
                format!("avere {equipaggio} di equipaggio e sopravvivere {tick} tick")
            }
        }
    }

    /// Il progresso corrente in forma compatta, per l'HUD.
    pub fn progresso(&self, sim: &Sim, s: &StatoLivello) -> String {
        match *self {
            Obiettivo::Equipaggio { minimo } => {
                format!("OBIETTIVO  equipaggio {}/{}", sim.equipaggio, minimo)
            }
            Obiettivo::LabConsecutivi { laboratori, tick } => format!(
                "OBIETTIVO  lab attivi {}/{} · {}/{} tick",
                s.lab_attivi, laboratori, s.progresso, tick
            ),
            Obiettivo::SopravviviConLab { laboratori, tick } => format!(
                "OBIETTIVO  lab attivi {}/{} · {}/{} tick senza avarie",
                s.lab_attivi, laboratori, s.progresso, tick
            ),
            Obiettivo::PuntiSenzaBlackout { punti } => format!(
                "OBIETTIVO  {}/{} punti senza blackout",
                s.progresso, punti
            ),
            Obiettivo::Colonia { equipaggio, tick } => format!(
                "OBIETTIVO  equipaggio {}/{} · tick {}/{}",
                sim.equipaggio,
                equipaggio,
                sim.tick.min(tick),
                tick
            ),
        }
    }

    /// Avanza il progresso di UN tick di simulazione; `true` se l'obiettivo
    /// è raggiunto. Le condizioni accessorie (lab spenti, blackout, avaria)
    /// azzerano `progresso`, non fanno perdere: perdere si perde solo con la
    /// stazione (logica già in `sim.rs`).
    pub fn avanza(&self, s: &mut StatoLivello, sim: &Sim) -> bool {
        match *self {
            Obiettivo::Equipaggio { minimo } => sim.equipaggio >= minimo,
            Obiettivo::LabConsecutivi { laboratori, tick } => {
                if s.lab_attivi >= laboratori {
                    s.progresso += 1;
                } else {
                    s.progresso = 0;
                }
                s.progresso >= u64::from(tick)
            }
            Obiettivo::SopravviviConLab { laboratori, tick } => {
                if sim.avarie_surriscaldamento > s.avarie_viste {
                    // nuova avaria da surriscaldamento: si riparte da zero
                    s.avarie_viste = sim.avarie_surriscaldamento;
                    s.progresso = 0;
                } else if s.lab_attivi >= laboratori {
                    s.progresso += 1;
                } else {
                    s.progresso = 0;
                }
                s.progresso >= u64::from(tick)
            }
            Obiettivo::PuntiSenzaBlackout { punti } => {
                if sim.in_blackout {
                    s.progresso = 0;
                } else {
                    s.progresso += u64::from(sim.equipaggio);
                }
                s.progresso >= punti
            }
            Obiettivo::Colonia { equipaggio, tick } => {
                sim.equipaggio >= equipaggio && sim.tick >= tick
            }
        }
    }
}

/// Valuta l'obiettivo del livello in corso, una volta per tick effettivo.
/// Gira solo con un obiettivo attivo (`obiettivi_attivi`) e in `InGioco`.
pub fn controlla_obiettivo(
    modalita: Res<Modalita>,
    casuale: Res<LivelloCasuale>,
    sim: Res<Sim>,
    mut stato: ResMut<StatoLivello>,
    mut progressione: ResMut<Progressione>,
    moduli: Query<&Module>,
    mut log: ResMut<EventLog>,
    mut prossimo: ResMut<NextState<AppState>>,
) {
    let obiettivo = match *modalita {
        Modalita::Campagna(i) => LIVELLI[i].obiettivo,
        Modalita::Casuale => match &casuale.0 {
            Some(l) => l.obiettivo,
            None => return,
        },
        Modalita::Infinita | Modalita::Sfida => return,
    };
    // conteggio vivo per l'HUD, anche in pausa-costruzione (anteprima)
    stato.lab_attivi = moduli
        .iter()
        .filter(|m| m.kind == ModuleKind::Laboratorio && m.attivo() && m.staffed)
        .count() as u32;
    // l'obiettivo avanza solo su tick nuovi, e la fine partita vince sempre
    if stato.completato || sim.partita_finita || sim.tick == stato.ultimo_tick {
        return;
    }
    stato.ultimo_tick = sim.tick;
    if obiettivo.avanza(&mut stato, &sim) {
        stato.completato = true;
        log.info(
            sim.tick,
            format!("Obiettivo raggiunto: {}", obiettivo.descrizione()),
        );
        // la progressione avanza solo in campagna: il casuale è fuori gara
        if let Modalita::Campagna(i) = *modalita
            && i + 1 > progressione.completati
        {
            progressione.completati = i + 1;
            salva_progressione(progressione.completati);
        }
        prossimo.set(AppState::LivelloCompletato);
    }
}

// ---------------- classifica ----------------

/// Un record della classifica (solo modalità infinita).
#[derive(Clone, PartialEq, Eq)]
pub struct Record {
    pub punteggio: u64,
    pub tick: u64,
    pub equipaggio_max: u32,
    /// Secondi dall'epoch Unix: si mostra come "N giorni fa" (`giorni_fa`),
    /// così non serve nessuna crate di date.
    pub epoch: u64,
}

/// Classifica locale, top 10, ordinata per punteggio decrescente.
#[derive(Default, Clone)]
pub struct Classifica {
    pub righe: Vec<Record>,
}

impl Classifica {
    /// Inserisce la partita appena finita; ritorna la posizione (0-based)
    /// se entra in top 10. Non scrive su disco: chiamare `salva` dopo.
    pub fn registra(&mut self, sim: &Sim) -> Option<usize> {
        let nuovo = Record {
            punteggio: sim.punteggio,
            tick: sim.tick,
            equipaggio_max: sim.equipaggio_max,
            epoch: epoch_adesso(),
        };
        self.righe.push(nuovo.clone());
        // sort stabile + push in coda: a parità di punteggio vince il record
        // più vecchio, il nuovo scivola sotto
        self.righe.sort_by_key(|r| std::cmp::Reverse(r.punteggio));
        self.righe.truncate(TOP_N);
        self.righe.iter().position(|r| *r == nuovo)
    }

    /// Scrive il file (TAB-separated) nella cartella dati. Gli errori di I/O
    /// non fermano il gioco: la classifica è un di più, non un requisito per
    /// giocare.
    fn salva(&self, nome_file: &str) {
        let Some(dir) = cartella_dati() else {
            return;
        };
        let _ = std::fs::create_dir_all(&dir);
        let testo: String = self
            .righe
            .iter()
            .map(|r| format!("{}\t{}\t{}\t{}\n", r.punteggio, r.tick, r.equipaggio_max, r.epoch))
            .collect();
        let _ = std::fs::write(dir.join(nome_file), testo);
    }
}

fn carica_classifica_da(nome_file: &str) -> Classifica {
    let testo = cartella_dati()
        .map(|d| d.join(nome_file))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    Classifica {
        righe: parse_classifica(&testo),
    }
}

/// Classifica della modalità Infinita (nessun tetto di tick): file
/// `classifica.txt`, invariato rispetto a prima che esistesse Sfida.
#[derive(Resource, Default)]
pub struct ClassificaInfinita(pub Classifica);

impl ClassificaInfinita {
    pub fn registra(&mut self, sim: &Sim) -> Option<usize> {
        self.0.registra(sim)
    }
    pub fn salva(&self) {
        self.0.salva("classifica.txt");
    }
}

/// Classifica della modalità Sfida (tetto di tick): file separato, i
/// punteggi non sono comparabili con quelli dell'Infinita.
#[derive(Resource, Default)]
pub struct ClassificaSfida(pub Classifica);

impl ClassificaSfida {
    pub fn registra(&mut self, sim: &Sim) -> Option<usize> {
        self.0.registra(sim)
    }
    pub fn salva(&self) {
        self.0.salva("classifica_sfida.txt");
    }
}

/// Parsing del file classifica: quattro campi separati da TAB per riga.
/// Una riga malformata (scritta a mano, troncata, vuota) si salta e basta:
/// il file dell'utente non deve mai rompere l'avvio.
pub fn parse_classifica(testo: &str) -> Vec<Record> {
    let mut righe: Vec<Record> = testo
        .lines()
        .filter_map(|riga| {
            let mut campi = riga.split('\t');
            let punteggio = campi.next()?.trim().parse().ok()?;
            let tick = campi.next()?.trim().parse().ok()?;
            let equipaggio_max = campi.next()?.trim().parse().ok()?;
            let epoch = campi.next()?.trim().parse().ok()?;
            Some(Record {
                punteggio,
                tick,
                equipaggio_max,
                epoch,
            })
        })
        .collect();
    righe.sort_by_key(|r| std::cmp::Reverse(r.punteggio));
    righe.truncate(TOP_N);
    righe
}

/// Carica le due classifiche all'avvio. File assente o illeggibile = vuota.
pub fn carica_classifica_infinita() -> ClassificaInfinita {
    ClassificaInfinita(carica_classifica_da("classifica.txt"))
}

pub fn carica_classifica_sfida() -> ClassificaSfida {
    ClassificaSfida(carica_classifica_da("classifica_sfida.txt"))
}

// ---------------- progressione ----------------

/// Carica la progressione all'avvio. File assente o illeggibile = si
/// riparte dal livello 1, senza errori a schermo.
pub fn carica_progressione() -> Progressione {
    let completati = percorso_progressione()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| t.trim().parse::<usize>().ok())
        .unwrap_or(0)
        .min(LIVELLI.len());
    Progressione { completati }
}

pub fn salva_progressione(completati: usize) {
    let Some(percorso) = percorso_progressione() else {
        return;
    };
    if let Some(dir) = percorso.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(percorso, format!("{completati}\n"));
}

// ---------------- percorsi e tempo ----------------

/// `$XDG_DATA_HOME/space-station`, ripiego `$HOME/.local/share/space-station`.
pub(crate) fn cartella_dati() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
        })?;
    Some(base.join("space-station"))
}

fn percorso_progressione() -> Option<PathBuf> {
    cartella_dati().map(|d| d.join("progressione.txt"))
}

fn epoch_adesso() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// "oggi" / "ieri" / "N giorni fa": basta l'epoch, niente crate di date.
pub fn giorni_fa(epoch: u64) -> String {
    match epoch_adesso().saturating_sub(epoch) / 86_400 {
        0 => "oggi".into(),
        1 => "ieri".into(),
        n => format!("{n} giorni fa"),
    }
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn riga_malformata_si_salta_le_altre_si_leggono() {
        let testo = "100\t50\t4\t1700000000\n\
                     riga scritta a mano senza tab\n\
                     200\t80\t6\t1700000001\n\
                     300\tnon-un-numero\t2\t3\n\
                     \n\
                     50\t10\t2\t1700000002\textra-campo-ignorato\n";
        let righe = parse_classifica(testo);
        // restano la prima, la terza e l'ultima (i campi extra si ignorano)
        assert_eq!(righe.len(), 3);
        assert_eq!(righe[0].punteggio, 200); // ordinata per punteggio
        assert_eq!(righe[1].punteggio, 100);
        assert_eq!(righe[2].punteggio, 50);
    }

    #[test]
    fn file_vuoto_o_assente_da_classifica_vuota() {
        assert!(parse_classifica("").is_empty());
    }

    #[test]
    fn classifica_tiene_solo_dieci_record() {
        let testo: String = (0..15).map(|i| format!("{i}\t1\t1\t0\n")).collect();
        let righe = parse_classifica(&testo);
        assert_eq!(righe.len(), TOP_N);
        assert_eq!(righe[0].punteggio, 14);
    }

    #[test]
    fn lab_consecutivi_riparte_da_zero_se_un_lab_si_spegne() {
        // livello 3: 2 laboratori attivi per 15 tick consecutivi
        let ob = LIVELLI[2].obiettivo;
        let mut s = StatoLivello::default();
        let sim = Sim::default();
        s.lab_attivi = 2;
        for _ in 0..10 {
            assert!(!ob.avanza(&mut s, &sim));
        }
        assert_eq!(s.progresso, 10);
        // al tick 10 un laboratorio si spegne: il contatore riparte da zero
        s.lab_attivi = 1;
        assert!(!ob.avanza(&mut s, &sim));
        assert_eq!(s.progresso, 0);
        // e servono di nuovo 15 tick pieni
        s.lab_attivi = 2;
        for t in 1..=15 {
            assert_eq!(ob.avanza(&mut s, &sim), t == 15);
        }
    }

    #[test]
    fn blackout_azzera_i_punti_ma_la_partita_continua() {
        // livello 5: 400 punti senza mai andare in blackout
        let ob = LIVELLI[4].obiettivo;
        let mut s = StatoLivello::default();
        let mut sim = Sim::default();
        sim.equipaggio = 4;
        for _ in 0..50 {
            assert!(!ob.avanza(&mut s, &sim));
        }
        assert_eq!(s.progresso, 200);
        // blackout al tick 50: il progresso si azzera, nessuna fine partita
        sim.in_blackout = true;
        assert!(!ob.avanza(&mut s, &sim));
        assert_eq!(s.progresso, 0);
        // tornata la corrente, 4 punti/tick: al centesimo tick fa 400
        sim.in_blackout = false;
        for t in 1..=100 {
            assert_eq!(ob.avanza(&mut s, &sim), t == 100);
        }
    }

    #[test]
    fn avaria_da_surriscaldamento_azzera_il_progresso_del_livello_4() {
        let ob = LIVELLI[3].obiettivo;
        let mut s = StatoLivello::default();
        let mut sim = Sim::default();
        s.lab_attivi = 2;
        for _ in 0..30 {
            assert!(!ob.avanza(&mut s, &sim));
        }
        assert_eq!(s.progresso, 30);
        sim.avarie_surriscaldamento = 1;
        assert!(!ob.avanza(&mut s, &sim));
        assert_eq!(s.progresso, 0);
        // la stessa avaria non azzera due volte
        assert!(!ob.avanza(&mut s, &sim));
        assert_eq!(s.progresso, 1);
    }
}
