//! Tick di bilancio e cascata di guasti: il cuore della PoC.
//! Ogni tick: assegnazione equipaggio ai laboratori → reti elettriche per
//! adiacenza e allocazione energia per priorità dentro ciascuna rete (in
//! deficit i moduli si spengono a partire dai meno critici) → bilancio
//! ossigeno → morti/arrivi → bilancio calore → avarie casuali → punteggio e
//! controllo di fine partita.
//! In pausa si ricalcola solo l'allocazione (anteprima del bilancio nell'HUD),
//! senza consumare ossigeno né rompere nulla: si costruisce senza conseguenze.

use crate::modules::ModuleKind;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rand::RngExt;

pub const TICK_SECS: f32 = 0.7; // durata di un tick di simulazione
pub const O2_MAX: f32 = 100.0;
pub const OSSIGENO_PER_CREW: f32 = 10.0; // consumo per membro per tick
pub const SOGLIA_O2_CRITICO: f32 = 30.0;
pub const TICK_SURRISCALDAMENTO: u32 = 6; // tick consecutivi di calore in eccesso prima di un'avaria
pub const TICK_MORTE: u32 = 3; // con ossigeno a zero, un morto ogni N tick
pub const TICK_ARRIVO: u32 = 4; // con posti liberi e aria buona, un arrivo ogni N tick
pub const CAPIENZA_BATTERIA: f32 = 150.0; // energia immagazzinabile per batteria
pub const RICARICA_BATTERIA: f32 = 15.0; // carica massima per tick, dal surplus di rete
pub const TICK_LAVORO_GRU: u32 = 12; // tick attivi consecutivi per rimuovere un detrito
/// Tetto di sicurezza: oltre questo tick la partita finisce comunque, vinta o
/// no. Senza, una stazione bloccata in una spirale di asfissia lenta (morti
/// a un membro ogni `TICK_MORTE` tick) può trascinarsi per centinaia di tick
/// prima di risolversi da sola.
pub const TICK_MASSIMO: u64 = 400;

#[derive(Component)]
pub struct Module {
    pub kind: ModuleKind,
    pub etichetta: String, // es. "Laboratorio 2", usata dal log
    pub cella: IVec2,      // serve a riposizionare i moduli quando la griglia cambia scala
    pub seq: u32,          // ordine di costruzione, per un esito deterministico
    pub powered: bool,
    pub broken: bool,
    pub staffed: bool,   // significativo solo per i laboratori
    pub collegato: bool, // la sua rete contiene un reattore non in avaria
    /// Energia immagazzinata (solo Batteria), 0..=CAPIENZA_BATTERIA.
    pub carica: f32,
    /// Tick attivi consecutivi (solo Gru): a TICK_LAVORO_GRU scatta la
    /// rimozione del detrito adiacente (effetto spaziale in main.rs).
    pub lavoro: u32,
}

impl Module {
    /// La definizione di "attivo" della simulazione, in un solo posto.
    pub fn attivo(&self) -> bool {
        self.powered && !self.broken
    }

    /// Perché il modulo è fermo; `None` se sta lavorando.
    /// Priorità: avaria > scollegato > equipaggio > energia — "scollegato"
    /// (nessun reattore nella rete) è un problema più a monte del blackout
    /// e di qualunque questione di equipaggio.
    pub fn motivo_fermo(&self) -> Option<Fermo> {
        if self.broken {
            Some(Fermo::Avaria)
        } else if !self.collegato {
            Some(Fermo::Scollegato)
        } else if self.kind == ModuleKind::Laboratorio && !self.staffed {
            Some(Fermo::Equipaggio)
        } else if !self.powered {
            Some(Fermo::Energia)
        } else {
            None
        }
    }
}

/// Le quattro risposte diverse alla domanda "perché è fermo?".
/// `Scollegato` (la rete non ha reattori) non è `Energia` (la rete ha un
/// reattore ma la corrente non basta): due problemi, due rimedi.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fermo {
    Energia,
    Scollegato,
    Equipaggio,
    Avaria,
}

#[derive(Resource)]
pub struct Sim {
    pub running: bool,
    pub tick: u64,
    pub timer: Timer,
    pub ossigeno: f32,
    pub equipaggio: u32,
    pub surriscaldamento: u32,
    pub cd_morte: u32,
    pub cd_arrivo: u32,
    pub o2_avvisato: bool,
    // valori calcolati a ogni tick, letti dall'HUD.
    // Produzione e consumo sono tenuti separati apposta: il netto da solo non
    // dice se sei al limite o se hai margine, ed è metà del problema di
    // leggibilità che questa iterazione risolve.
    pub energia_prod: f32,
    pub energia_cons: f32,
    pub energia_margine: f32,
    /// Somma delle cariche delle batterie non rotte (0 se non ce ne sono).
    pub batterie_carica: f32,
    pub batterie_capienza: f32,
    pub in_blackout: bool,
    pub o2_prod: f32,
    pub o2_cons: f32,
    pub o2_delta: f32,
    pub calore_prod: f32,
    pub calore_diss: f32,
    pub calore_netto: f32,
    pub posti_letto: u32,
    pub lab_fabbisogno: u32, // equipaggio richiesto dai laboratori non rotti
    /// Avarie da surriscaldamento dall'inizio della partita: contatore di
    /// eventi, solo output. Lo legge l'obiettivo del livello "Termica"
    /// (`livelli.rs`) per accorgersi di una nuova avaria senza tenere stato
    /// di livello dentro `Sim`.
    pub avarie_surriscaldamento: u32,
    // obiettivo e fine partita
    pub punteggio: u64,      // persone·tick: +equipaggio a ogni tick effettivo
    pub equipaggio_max: u32, // massimo storico, mostrato a fine partita
    pub partita_finita: bool, // equipaggio a 0 dopo essere stato sopra 0, o tetto tick raggiunto
    /// Perché `partita_finita` è vero; `None` finché la partita continua.
    /// Letta solo dalla schermata di fine partita per scegliere il testo.
    pub motivo_fine: Option<MotivoFine>,
    /// Tetto di tick di questa partita, se ne ha uno: `Some(TICK_MASSIMO)` in
    /// Campagna e Sfida, `None` in Infinita (nessun limite, come da spirito
    /// "resisti quanto vuoi" della modalità). Impostato da `applica_reset` in
    /// `main.rs` secondo la modalità scelta, non qui: `Sim` non sa cos'è una
    /// `Modalita`.
    pub tetto_tick: Option<u64>,
}

/// Le due cause di fine partita: equipaggio azzerato o tetto di tick
/// raggiunto (vedi `TICK_MASSIMO`). Due schermate diverse, stesso flusso.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MotivoFine {
    EquipaggioMorto,
    TempoScaduto,
}

impl Default for Sim {
    fn default() -> Self {
        Self {
            running: false,
            tick: 0,
            timer: Timer::from_seconds(TICK_SECS, TimerMode::Repeating),
            ossigeno: O2_MAX,
            equipaggio: 0,
            surriscaldamento: 0,
            cd_morte: 0,
            cd_arrivo: 0,
            o2_avvisato: false,
            energia_prod: 0.0,
            energia_cons: 0.0,
            energia_margine: 0.0,
            batterie_carica: 0.0,
            batterie_capienza: 0.0,
            in_blackout: false,
            o2_prod: 0.0,
            o2_cons: 0.0,
            o2_delta: 0.0,
            calore_prod: 0.0,
            calore_diss: 0.0,
            calore_netto: 0.0,
            posti_letto: 0,
            lab_fabbisogno: 0,
            avarie_surriscaldamento: 0,
            punteggio: 0,
            equipaggio_max: 0,
            partita_finita: false,
            motivo_fine: None,
            tetto_tick: None,
        }
    }
}

/// Gravità di una riga di log: assegnata alla sorgente, non dedotta a valle
/// facendo pattern matching sulle stringhe.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Gravita {
    Info,
    Avviso,
    Allarme,
}

pub struct Riga {
    pub tick: u64,
    pub gravita: Gravita,
    pub testo: String,
}

#[derive(Resource, Default)]
pub struct EventLog {
    righe: Vec<Riga>,
}

impl EventLog {
    pub fn push(&mut self, tick: u64, gravita: Gravita, testo: impl Into<String>) {
        self.righe.push(Riga {
            tick,
            gravita,
            testo: testo.into(),
        });
        if self.righe.len() > 60 {
            self.righe.drain(..self.righe.len() - 60);
        }
    }
    pub fn info(&mut self, tick: u64, testo: impl Into<String>) {
        self.push(tick, Gravita::Info, testo);
    }
    pub fn ultimi(&self, n: usize) -> &[Riga] {
        let start = self.righe.len().saturating_sub(n);
        &self.righe[start..]
    }
    pub fn svuota(&mut self) {
        self.righe.clear();
    }
}

pub fn sim_tick(
    time: Res<Time>,
    mut sim: ResMut<Sim>,
    mut log: ResMut<EventLog>,
    mut q: Query<&mut Module>,
) {
    // A stazione persa la simulazione si ferma: la schermata di fine partita
    // fotografa lo stato, non deve continuare a evolvere sotto.
    if sim.partita_finita {
        return;
    }
    sim.timer.tick(time.delta());
    if !sim.timer.just_finished() {
        return;
    }
    // In pausa: solo anteprima (niente log, niente stato che evolve).
    let vivo = sim.running;
    if vivo {
        sim.tick += 1;
    }
    let tick = sim.tick;

    let mut mods: Vec<Mut<Module>> = q.iter_mut().collect();
    mods.sort_by_key(|m| (m.kind.def().priorita, m.seq));

    // --- 1) equipaggio nei laboratori (ordine di costruzione) ---
    let mut disponibili = sim.equipaggio;
    for m in mods.iter_mut() {
        if m.kind != ModuleKind::Laboratorio || m.broken {
            continue;
        }
        let req = m.kind.def().equipaggio_richiesto;
        let ok = disponibili >= req;
        if ok {
            disponibili -= req;
        }
        if vivo && ok != m.staffed {
            if ok {
                log.info(tick, format!("{} operativo: equipaggio assegnato", m.etichetta));
            } else {
                log.push(
                    tick,
                    Gravita::Avviso,
                    format!("{} fermo: equipaggio insufficiente", m.etichetta),
                );
            }
        }
        m.staffed = ok;
    }
    sim.lab_fabbisogno = mods
        .iter()
        .filter(|m| m.kind == ModuleKind::Laboratorio && !m.broken)
        .map(|m| m.kind.def().equipaggio_richiesto)
        .sum();

    // --- 2) reti elettriche e allocazione per priorità (cascata stadio 1) ---
    // L'energia si propaga solo tra moduli su celle ortogonalmente adiacenti
    // (4 vicini): ogni componente connessa è una rete isolata, alimentata solo
    // se contiene almeno un reattore non in avaria. I moduli in avaria
    // conducono comunque corrente — sono ancora tubi, non buchi — ma un
    // reattore rotto non produce e non "accende" la sua rete.
    // Ossigeno, calore ed equipaggio restano a livello di stazione.
    let per_cella: HashMap<IVec2, usize> =
        mods.iter().enumerate().map(|(i, m)| (m.cella, i)).collect();
    let mut comp = vec![usize::MAX; mods.len()];
    let mut n_comp = 0;
    for inizio in 0..mods.len() {
        if comp[inizio] != usize::MAX {
            continue;
        }
        comp[inizio] = n_comp;
        let mut coda = vec![inizio];
        while let Some(i) = coda.pop() {
            let cella = mods[i].cella;
            for delta in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                if let Some(&j) = per_cella.get(&(cella + delta))
                    && comp[j] == usize::MAX
                {
                    comp[j] = n_comp;
                    coda.push(j);
                }
            }
        }
        n_comp += 1;
    }
    // In ogni rete i produttori vanno sommati TUTTI prima di distribuire: se
    // li si contasse man mano nell'ordine di priorità, un modulo costruito
    // prima del reattore andrebbe in blackout pur essendoci corrente
    // (bug corretto nella PoC: non tornare indietro).
    let mut pool = vec![0.0f32; n_comp];
    let mut con_reattore = vec![false; n_comp];
    for (i, m) in mods.iter().enumerate() {
        if !m.broken && m.kind.def().energia > 0.0 {
            pool[comp[i]] += m.kind.def().energia;
            con_reattore[comp[i]] = true;
        }
    }
    // Le batterie mettono la carica accumulata a disposizione PRIMA della
    // distribuzione: in deficit la rete attinge da loro invece di spegnere
    // moduli. Quanto hanno ceduto davvero si regola dopo il giro; una rete
    // senza reattore non le attiva (la carica resta lì, inerte).
    let mut accumulo = vec![0.0f32; n_comp];
    for (i, m) in mods.iter().enumerate() {
        if !m.broken && m.kind == ModuleKind::Batteria && con_reattore[comp[i]] {
            accumulo[comp[i]] += m.carica;
        }
    }
    for c in 0..n_comp {
        pool[c] += accumulo[c];
    }
    let mut blackout = false;
    for (i, m) in mods.iter_mut().enumerate() {
        let def = m.kind.def();
        let collegato = con_reattore[comp[i]];
        let era_collegato = m.collegato;
        // Scollegato e blackout sono due problemi diversi e il log li separa.
        // Per i moduli in avaria tace: lì il problema dichiarato è l'avaria.
        if vivo && !m.broken && collegato != era_collegato {
            if collegato {
                log.info(tick, format!("{} collegato alla rete", m.etichetta));
            } else {
                log.push(
                    tick,
                    Gravita::Avviso,
                    format!("{} non è collegato a un reattore", m.etichetta),
                );
            }
        }
        m.collegato = collegato;
        if m.broken {
            m.powered = false;
            continue;
        }
        if def.energia > 0.0 {
            m.powered = true;
            continue;
        }
        // un laboratorio senza equipaggio non chiede energia
        if m.kind == ModuleKind::Laboratorio && !m.staffed {
            m.powered = false;
            continue;
        }
        // rete senza reattore: il modulo è scollegato, non in blackout
        if !collegato {
            m.powered = false;
            continue;
        }
        // la batteria non consuma dal pool: è il pool (carica/scarica
        // regolate dopo il giro), quindi collegata = sempre operativa
        if m.kind == ModuleKind::Batteria {
            m.powered = true;
            continue;
        }
        let rete = &mut pool[comp[i]];
        if *rete + def.energia >= 0.0 {
            *rete += def.energia;
            // il ricollegamento è già loggato sopra: niente doppia riga
            if vivo && !m.powered && era_collegato {
                log.info(tick, format!("{} riavviato", m.etichetta));
            }
            m.powered = true;
        } else {
            if vivo && m.powered {
                log.push(
                    tick,
                    Gravita::Allarme,
                    format!("Blackout: {} spento", m.etichetta),
                );
            }
            m.powered = false;
            blackout = true;
        }
    }
    sim.energia_prod = mods
        .iter()
        .filter(|m| !m.broken && m.kind.def().energia > 0.0)
        .map(|m| m.kind.def().energia)
        .sum();
    sim.energia_cons = mods
        .iter()
        .filter(|m| m.powered && !m.broken && m.kind.def().energia < 0.0)
        .map(|m| -m.kind.def().energia)
        .sum();
    // Regolamento batterie, per rete: quanto è stato attinto (accumulo −
    // residuo, se positivo) si scala dalle cariche in ordine di costruzione;
    // il surplus vero dei generatori le ricarica (max RICARICA_BATTERIA a
    // testa per tick). In anteprima (pausa) le cariche non si toccano.
    if vivo {
        let mut da_scalare: Vec<f32> = (0..n_comp)
            .map(|c| (accumulo[c] - pool[c]).max(0.0))
            .collect();
        let mut surplus: Vec<f32> = (0..n_comp)
            .map(|c| (pool[c] - accumulo[c]).max(0.0))
            .collect();
        for (i, m) in mods.iter_mut().enumerate() {
            if m.broken || m.kind != ModuleKind::Batteria || !con_reattore[comp[i]] {
                continue;
            }
            let c = comp[i];
            let prelievo = da_scalare[c].min(m.carica);
            m.carica -= prelievo;
            da_scalare[c] -= prelievo;
            let ricarica = surplus[c].min(RICARICA_BATTERIA).min(CAPIENZA_BATTERIA - m.carica);
            m.carica += ricarica;
            surplus[c] -= ricarica;
        }
    }
    // margine = surplus reale dei generatori (negativo quando le reti
    // stanno andando a batteria: il numero racconta esattamente questo)
    sim.energia_margine = (0..n_comp).map(|c| pool[c] - accumulo[c]).sum();
    sim.batterie_carica = mods
        .iter()
        .filter(|m| !m.broken && m.kind == ModuleKind::Batteria)
        .map(|m| m.carica)
        .sum();
    sim.batterie_capienza = mods
        .iter()
        .filter(|m| !m.broken && m.kind == ModuleKind::Batteria)
        .count() as f32
        * CAPIENZA_BATTERIA;
    sim.in_blackout = blackout;

    // La Gru lavora solo da attiva: il contatore avanza coi tick effettivi
    // e riparte da zero se si spegne; a TICK_LAVORO_GRU scatta l'effetto
    // (rimozione del detrito adiacente, in main.rs: qui non c'è la griglia).
    if vivo {
        for m in mods.iter_mut() {
            if m.kind == ModuleKind::Gru {
                if m.attivo() {
                    m.lavoro += 1;
                } else {
                    m.lavoro = 0;
                }
            }
        }
    }

    // --- 3) bilancio ossigeno (cascata stadio 2) ---
    let attivo = |m: &Module| m.attivo();
    sim.o2_prod = mods.iter().filter(|m| attivo(m)).map(|m| m.kind.def().ossigeno).sum();
    sim.o2_cons = sim.equipaggio as f32 * OSSIGENO_PER_CREW;
    sim.o2_delta = sim.o2_prod - sim.o2_cons;
    sim.posti_letto = mods
        .iter()
        .filter(|m| attivo(m))
        .map(|m| m.kind.def().posti_letto)
        .sum();
    sim.calore_prod = mods
        .iter()
        .filter(|m| attivo(m) && m.kind.def().calore > 0.0)
        .map(|m| m.kind.def().calore)
        .sum();
    sim.calore_diss = mods
        .iter()
        .filter(|m| attivo(m) && m.kind.def().calore < 0.0)
        .map(|m| -m.kind.def().calore)
        .sum();
    sim.calore_netto = sim.calore_prod - sim.calore_diss;

    if !vivo {
        return;
    }

    let o2_prima = sim.ossigeno;
    sim.ossigeno = (sim.ossigeno + sim.o2_delta).clamp(0.0, O2_MAX);
    if sim.ossigeno <= 0.0 && o2_prima > 0.0 {
        log.push(tick, Gravita::Allarme, "Ossigeno esaurito");
    } else if sim.ossigeno <= SOGLIA_O2_CRITICO && !sim.o2_avvisato && sim.equipaggio > 0 {
        log.push(
            tick,
            Gravita::Avviso,
            format!("Ossigeno critico ({:.0})", sim.ossigeno),
        );
        sim.o2_avvisato = true;
    }
    if sim.ossigeno > SOGLIA_O2_CRITICO + 10.0 {
        sim.o2_avvisato = false;
    }

    // --- 4) equipaggio: asfissia (cascata stadio 3) e nuovi arrivi ---
    if sim.ossigeno <= 0.0 && sim.equipaggio > 0 {
        sim.cd_morte += 1;
        if sim.cd_morte >= TICK_MORTE {
            sim.cd_morte = 0;
            let prima = sim.equipaggio;
            sim.equipaggio -= 1;
            log.push(
                tick,
                Gravita::Allarme,
                format!("Equipaggio: {} → {} (asfissia)", prima, sim.equipaggio),
            );
        }
    } else {
        sim.cd_morte = 0;
    }
    // il Centro comando attivo dimezza l'attesa dei nuovi arrivi
    let cadenza_arrivo = if mods
        .iter()
        .any(|m| m.kind == ModuleKind::CentroComando && m.attivo())
    {
        TICK_ARRIVO / 2
    } else {
        TICK_ARRIVO
    };
    if sim.equipaggio < sim.posti_letto && sim.ossigeno > 50.0 {
        sim.cd_arrivo += 1;
        if sim.cd_arrivo >= cadenza_arrivo {
            sim.cd_arrivo = 0;
            let prima = sim.equipaggio;
            sim.equipaggio += 1;
            log.info(tick, format!("Equipaggio: {} → {} (nuovo arrivo)", prima, sim.equipaggio));
        }
    } else {
        sim.cd_arrivo = 0;
    }

    // --- 5) calore in eccesso → avaria casuale (cascata stadio 4) ---
    if sim.calore_netto > 0.0 {
        sim.surriscaldamento += 1;
        if sim.surriscaldamento == 1 {
            log.push(
                tick,
                Gravita::Avviso,
                format!("Surriscaldamento: +{:.0} calore/tick", sim.calore_netto),
            );
        }
        if sim.surriscaldamento >= TICK_SURRISCALDAMENTO {
            sim.surriscaldamento = 0;
            let vittime: Vec<usize> = mods
                .iter()
                .enumerate()
                .filter(|(_, m)| !m.broken)
                .map(|(i, _)| i)
                .collect();
            if let Some(&i) = vittime.get(rand::rng().random_range(0..vittime.len().max(1))) {
                let m = &mut mods[i];
                m.broken = true;
                m.powered = false;
                sim.avarie_surriscaldamento += 1;
                log.push(
                    tick,
                    Gravita::Allarme,
                    format!("Avaria da surriscaldamento: {} fuori uso", m.etichetta),
                );
            }
        }
    } else {
        sim.surriscaldamento = 0;
    }

    // --- 6) punteggio e fine partita ---
    // Il punteggio è persone·tick: cresce tanto più in fretta quanto più
    // equipaggio riesci a tenere in vita. La partita finisce quando
    // l'equipaggio torna a zero dopo essere stato almeno una volta sopra.
    sim.punteggio += sim.equipaggio as u64;
    sim.equipaggio_max = sim.equipaggio_max.max(sim.equipaggio);
    if sim.equipaggio == 0 && sim.equipaggio_max > 0 {
        sim.partita_finita = true;
        sim.motivo_fine = Some(MotivoFine::EquipaggioMorto);
        log.push(
            tick,
            Gravita::Allarme,
            "Stazione persa: tutto l'equipaggio è morto",
        );
    } else if let Some(tetto) = sim.tetto_tick
        && sim.tick >= tetto
    {
        sim.partita_finita = true;
        sim.motivo_fine = Some(MotivoFine::TempoScaduto);
        log.push(
            tick,
            Gravita::Allarme,
            format!("Tempo scaduto: {tetto} tick raggiunti"),
        );
    }
}
