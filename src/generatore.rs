//! Generatore parametrico di livelli (SPEC-CAMPAGNA.md §2.2).
//!
//! Deterministico per costruzione: la campagna usa `seed = SEME ^ n`, quindi
//! il livello 23 è identico per tutti e a ogni avvio; la modalità casuale
//! passa un seed estratto a caso. Il PRNG è uno splitmix64 scritto qui (10
//! righe) invece della crate `rand`: la sequenza non deve mai cambiare con
//! un aggiornamento di dipendenza, o i livelli della campagna cambierebbero
//! sotto i piedi dei giocatori.
//!
//! Garanzia di risolvibilità: budget ≥ fabbisogno minimo (stessa funzione
//! usata dai test) e area libera connessa ≥ budget (flood fill). I pattern
//! di detriti che violano il vincolo vengono riprovati e, in ultimo,
//! ripiegano su pochi detriti sparsi.

use crate::livelli::{LivelloDef, Obiettivo};
use crate::{GRID_H, GRID_W};

/// Seed base della campagna: cambiarlo rigenera TUTTI i livelli 7-50.
const SEME: u64 = 0x5A7E_5747_10E5_0001;

/// Costi per tick dei moduli, copiati da `modules::TABELLA` come interi.
/// Il test `fabbisogno_allineato_alla_tabella` garantisce che non divergano.
const REATTORE_ENERGIA: u32 = 100;
const REATTORE_CALORE: u32 = 40;
const LS_ENERGIA: u32 = 30;
const LS_OSSIGENO: u32 = 50;
const LS_CALORE: u32 = 5;
const DORM_ENERGIA: u32 = 10;
const DORM_CALORE: u32 = 2;
const DORM_POSTI: u32 = 4;
const LAB_ENERGIA: u32 = 40;
const LAB_CALORE: u32 = 25;
const RAD_ENERGIA: u32 = 5;
const RAD_CALORE: u32 = 50;
const CORRIDOIO_ENERGIA: u32 = 1;
const O2_PER_CREW: u32 = 10;

// ---------------- PRNG ----------------

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// splitmix64: minuscolo, non crittografico, sequenza stabile per sempre.
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Intero uniforme in `[lo, hi)`.
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo).max(1) as u64) as i32
    }
}

// ---------------- fabbisogno minimo ----------------

/// Quanti moduli servono, come minimo, per l'obiettivo: la soluzione ovvia
/// (reattori + life support + dormitori + eventuali laboratori +
/// radiatori) PIÙ i corridoi imposti dalla regola dei conduttori (sim.rs,
/// fase 2: le foglie si allacciano solo a reattori e corridoi, non si
/// prolungano a vicenda). Modello prudente e documentato: ogni reattore
/// allaccia 3 foglie (la quarta faccia resta per la dorsale), ogni
/// corridoio in linea ne allaccia 2 nette; `corridoi =
/// ceil((foglie − 3·reattori) / 2)`, e i corridoi consumano energia.
/// Usata dal generatore per il budget E dai test per la garanzia di
/// risolvibilità: una definizione sola, non possono divergere.
pub fn fabbisogno_minimo(ob: &Obiettivo) -> u32 {
    let (equipaggio, laboratori) = match *ob {
        Obiettivo::Equipaggio { minimo } => (minimo, 0),
        Obiettivo::LabConsecutivi { laboratori, .. }
        | Obiettivo::SopravviviConLab { laboratori, .. } => (2 * laboratori, laboratori),
        // per fare punti in fretta serve comunque gente a bordo
        Obiettivo::PuntiSenzaBlackout { .. } => (8, 0),
        Obiettivo::Colonia { equipaggio, .. } => (equipaggio, 0),
    };
    let dormitori = equipaggio.div_ceil(DORM_POSTI).max(1);
    let life_support = (equipaggio * O2_PER_CREW).div_ceil(LS_OSSIGENO).max(1);
    // radiatori, corridoi e reattori si condizionano a vicenda (radiatori
    // e corridoi consumano energia, i reattori scaldano e allacciano
    // foglie): punto fisso monotono, converge in pochi giri
    let mut radiatori = 0u32;
    let mut corridoi = 0u32;
    let reattori = loop {
        let consumo = LS_ENERGIA * life_support
            + DORM_ENERGIA * dormitori
            + LAB_ENERGIA * laboratori
            + RAD_ENERGIA * radiatori
            + CORRIDOIO_ENERGIA * corridoi;
        let reattori = consumo.div_ceil(REATTORE_ENERGIA).max(1);
        let calore = REATTORE_CALORE * reattori
            + LS_CALORE * life_support
            + DORM_CALORE * dormitori
            + LAB_CALORE * laboratori;
        let rad_servono = calore.div_ceil(RAD_CALORE);
        let foglie = life_support + dormitori + laboratori + radiatori.max(rad_servono);
        let corr_servono = foglie.saturating_sub(3 * reattori).div_ceil(2);
        if rad_servono <= radiatori && corr_servono <= corridoi {
            break reattori;
        }
        radiatori = radiatori.max(rad_servono);
        corridoi = corridoi.max(corr_servono);
    };
    reattori + life_support + dormitori + laboratori + radiatori + corridoi
}

// ---------------- generazione ----------------

/// Livello `n` (7..=50) della campagna: seed fisso derivato dall'indice.
pub fn genera_campagna(n: usize) -> LivelloDef {
    genera(n, Rng::new(SEME ^ n as u64), false)
}

/// Livello per la modalità casuale: difficoltà pescata dal seed nella fascia
/// centrale della curva (10..=40), fuori progressione e classifica.
pub fn genera_casuale(seed: u64) -> LivelloDef {
    let mut rng = Rng::new(seed);
    let difficolta = rng.range(10, 41) as usize;
    genera(difficolta, rng, true)
}

/// La sfida del giorno: stesso livello per TUTTI nello stesso giorno
/// (giorni interi dall'epoch Unix), difficoltà fissa di metà campagna.
/// Il seed mescola il giorno con una costante dedicata: cambiarla cambia
/// tutte le sfide future, mai quelle già giocate (il record è per giorno).
pub fn genera_giornaliera(giorno: u64) -> LivelloDef {
    const SEME_GIORNO: u64 = 0x5F1D_A0DE_0DE1_D1A0;
    let mut livello = genera(
        25,
        Rng::new(SEME_GIORNO ^ giorno.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
        true,
    );
    livello.nome = format!("Sfida del giorno — {}", livello.nome);
    livello
}

fn genera(n: usize, mut rng: Rng, casuale: bool) -> LivelloDef {
    let n32 = n as u32;
    // Obiettivi PESATI per blocco (SPEC-CAMPAGNA §1: i livelli dopo uno
    // sblocco valorizzano il modulo appena consegnato). Pesi sull'ordine
    // [Equipaggio, LabConsecutivi, SopravviviConLab, Punti, Colonia]:
    //   7–15  Batteria appena presa → Punti-senza-blackout pesa doppio
    //         (la batteria è l'assicurazione contro l'azzeramento);
    //  16–25  Serra → equipaggi grandi (Equipaggio e Colonia rendono
    //         l'ossigeno-per-watt il collo di bottiglia);
    //  26–35  Gru → pesi piatti ma quota detriti +2 (sotto);
    //  36–50  Centro comando → Colonia più che può.
    let pesi: [u32; 5] = match n {
        0..=15 => [2, 2, 2, 4, 2],
        16..=25 => [4, 2, 2, 2, 3],
        26..=35 => [2, 2, 2, 2, 2],
        _ => [2, 2, 2, 2, 5],
    };
    let totale: i32 = pesi.iter().sum::<u32>() as i32;
    let mut estratto = rng.range(0, totale) as u32;
    let mut tipo = 4;
    for (i, p) in pesi.iter().enumerate() {
        if estratto < *p {
            tipo = i;
            break;
        }
        estratto -= p;
    }
    let obiettivo = match tipo {
        0 => Obiettivo::Equipaggio {
            minimo: (6 + n32 / 4).min(20),
        },
        1 => Obiettivo::LabConsecutivi {
            laboratori: (2 + n32 / 18).min(4),
            tick: (12 + n32).min(90),
        },
        2 => Obiettivo::SopravviviConLab {
            laboratori: (2 + n32 / 22).min(4),
            tick: (25 + n32).min(110),
        },
        3 => Obiettivo::PuntiSenzaBlackout {
            punti: u64::from(150 + 18 * n32),
        },
        _ => Obiettivo::Colonia {
            equipaggio: (5 + n32 / 4).min(18),
            tick: u64::from((30 + 2 * n32).min(200)),
        },
    };

    // margine sul fabbisogno: ×1,6 al livello 7 → ×1,15 al livello 50
    let minimo = fabbisogno_minimo(&obiettivo);
    let margine = 1.6 - (n.clamp(7, 50) - 7) as f32 * (0.45 / 43.0);
    let max_moduli = ((minimo as f32 * margine).ceil() as u32).max(minimo + 2);

    // detriti: quota crescente, pattern dal seed, con garanzia di area;
    // nel blocco della Gru (26-35) due detriti in più: sono il suo mestiere
    let quota_base = (((n - 6) * 12) / 44).clamp(2, 12);
    let quota = if (26..=35).contains(&n) {
        (quota_base + 2).min(12)
    } else {
        quota_base
    };
    let mut ostacoli = Vec::new();
    for tentativo in 0..20 {
        let candidati = if tentativo < 19 {
            pattern(&mut rng, quota)
        } else {
            // ripiego certo: pochi sparsi non tolgono mai l'area necessaria
            pattern_sparsi(&mut rng, 2)
        };
        if area_libera_connessa(&candidati) >= max_moduli {
            ostacoli = candidati;
            break;
        }
    }

    let (nome, briefing) = battesimo(&mut rng, n, &obiettivo, casuale);
    LivelloDef {
        nome,
        briefing,
        obiettivo,
        ostacoli,
        max_moduli,
    }
}

fn battesimo(rng: &mut Rng, n: usize, ob: &Obiettivo, casuale: bool) -> (String, String) {
    const LUOGHI: [&str; 6] = [
        "Avamposto", "Settore", "Anello", "Piattaforma", "Nodo", "Cintura",
    ];
    let luogo = LUOGHI[rng.range(0, LUOGHI.len() as i32) as usize];
    let nome = if casuale {
        format!("{} X-{:02}", luogo, rng.range(10, 100))
    } else {
        format!("{} K-{:02}", luogo, n)
    };
    let briefing = match ob {
        Obiettivo::Equipaggio { .. } => "Porta gente a bordo e tienila viva.",
        Obiettivo::LabConsecutivi { .. } => "La ricerca non aspetta: laboratori accesi, sempre.",
        Obiettivo::SopravviviConLab { .. } => "Lavora sotto pressione senza fondere niente.",
        Obiettivo::PuntiSenzaBlackout { .. } => "Margine, sempre margine: mai al buio.",
        Obiettivo::Colonia { .. } => "Una colonia vera, che duri.",
    };
    (nome, briefing.to_string())
}

// ---------------- pattern di detriti ----------------

fn pattern(rng: &mut Rng, quota: usize) -> Vec<(i32, i32)> {
    match rng.range(0, 5) {
        0 => pattern_sparsi(rng, quota),
        1 => pattern_muro_verticale(rng, quota),
        2 => pattern_muro_orizzontale(rng, quota),
        3 => pattern_diagonale(rng, quota),
        _ => pattern_croce(rng, quota),
    }
}

fn pattern_sparsi(rng: &mut Rng, quota: usize) -> Vec<(i32, i32)> {
    let mut celle = Vec::new();
    while celle.len() < quota {
        let c = (rng.range(0, GRID_W), rng.range(0, GRID_H));
        if !celle.contains(&c) {
            celle.push(c);
        }
    }
    celle
}

fn pattern_muro_verticale(rng: &mut Rng, quota: usize) -> Vec<(i32, i32)> {
    let lunghezza = quota.min(6);
    let x = rng.range(3, GRID_W - 3);
    let y0 = rng.range(0, GRID_H - lunghezza as i32 + 1);
    let mut celle: Vec<_> = (0..lunghezza as i32).map(|i| (x, y0 + i)).collect();
    completa_sparsi(rng, &mut celle, quota);
    celle
}

fn pattern_muro_orizzontale(rng: &mut Rng, quota: usize) -> Vec<(i32, i32)> {
    let lunghezza = quota.min(8);
    let y = rng.range(2, GRID_H - 2);
    let x0 = rng.range(1, GRID_W - lunghezza as i32);
    let mut celle: Vec<_> = (0..lunghezza as i32).map(|i| (x0 + i, y)).collect();
    completa_sparsi(rng, &mut celle, quota);
    celle
}

fn pattern_diagonale(rng: &mut Rng, quota: usize) -> Vec<(i32, i32)> {
    let lunghezza = quota.min(6) as i32;
    let x0 = rng.range(2, GRID_W - lunghezza - 1);
    let y0 = rng.range(lunghezza - 1, GRID_H);
    let mut celle: Vec<_> = (0..lunghezza).map(|i| (x0 + i, y0 - i)).collect();
    completa_sparsi(rng, &mut celle, quota);
    celle
}

fn pattern_croce(rng: &mut Rng, quota: usize) -> Vec<(i32, i32)> {
    let cx = rng.range(3, GRID_W - 3);
    let cy = rng.range(2, GRID_H - 2);
    let mut celle = vec![(cx, cy)];
    for passo in 1..=2 {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if celle.len() >= quota {
                break;
            }
            let c = (cx + dx * passo, cy + dy * passo);
            if dentro(c) && !celle.contains(&c) {
                celle.push(c);
            }
        }
    }
    completa_sparsi(rng, &mut celle, quota);
    celle
}

fn completa_sparsi(rng: &mut Rng, celle: &mut Vec<(i32, i32)>, quota: usize) {
    while celle.len() < quota {
        let c = (rng.range(0, GRID_W), rng.range(0, GRID_H));
        if !celle.contains(&c) {
            celle.push(c);
        }
    }
    celle.retain(|&c| dentro(c));
    celle.dedup();
}

fn dentro((x, y): (i32, i32)) -> bool {
    (0..GRID_W).contains(&x) && (0..GRID_H).contains(&y)
}

/// La più grande area libera ortogonalmente connessa, in celle: la stazione
/// deve poterci stare tutta (i moduli funzionano solo in rete adiacente).
pub fn area_libera_connessa(ostacoli: &[(i32, i32)]) -> u32 {
    let occupata = |c: (i32, i32)| ostacoli.contains(&c);
    let mut vista = vec![false; (GRID_W * GRID_H) as usize];
    let indice = |x: i32, y: i32| (y * GRID_W + x) as usize;
    let mut migliore = 0u32;
    for sx in 0..GRID_W {
        for sy in 0..GRID_H {
            if vista[indice(sx, sy)] || occupata((sx, sy)) {
                continue;
            }
            let mut coda = vec![(sx, sy)];
            vista[indice(sx, sy)] = true;
            let mut area = 0u32;
            while let Some((x, y)) = coda.pop() {
                area += 1;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    if dentro((nx, ny)) && !vista[indice(nx, ny)] && !occupata((nx, ny)) {
                        vista[indice(nx, ny)] = true;
                        coda.push((nx, ny));
                    }
                }
            }
            migliore = migliore.max(area);
        }
    }
    migliore
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::livelli::LIVELLI;
    use crate::modules::ModuleKind;

    #[test]
    fn fabbisogno_allineato_alla_tabella() {
        // le costanti intere di questo modulo devono rispecchiare TABELLA:
        // se un valore di bilanciamento cambia là, questo test lo ricorda
        let d = ModuleKind::Reattore.def();
        assert_eq!(d.energia as u32, REATTORE_ENERGIA);
        assert_eq!(d.calore as u32, REATTORE_CALORE);
        let d = ModuleKind::LifeSupport.def();
        assert_eq!(-d.energia as u32, LS_ENERGIA);
        assert_eq!(d.ossigeno as u32, LS_OSSIGENO);
        assert_eq!(d.calore as u32, LS_CALORE);
        let d = ModuleKind::Dormitorio.def();
        assert_eq!(-d.energia as u32, DORM_ENERGIA);
        assert_eq!(d.calore as u32, DORM_CALORE);
        assert_eq!(d.posti_letto, DORM_POSTI);
        let d = ModuleKind::Laboratorio.def();
        assert_eq!(-d.energia as u32, LAB_ENERGIA);
        assert_eq!(d.calore as u32, LAB_CALORE);
        let d = ModuleKind::Radiatore.def();
        assert_eq!(-d.energia as u32, RAD_ENERGIA);
        assert_eq!(-d.calore as u32, RAD_CALORE);
        let d = ModuleKind::Corridoio.def();
        assert_eq!(-d.energia as u32, CORRIDOIO_ENERGIA);
        assert_eq!(crate::sim::OSSIGENO_PER_CREW as u32, O2_PER_CREW);
    }

    #[test]
    fn generazione_deterministica() {
        for n in [7, 23, 50] {
            let a = genera_campagna(n);
            let b = genera_campagna(n);
            assert_eq!(a.nome, b.nome);
            assert_eq!(a.max_moduli, b.max_moduli);
            assert_eq!(a.ostacoli, b.ostacoli);
        }
    }

    #[test]
    fn tutti_i_50_livelli_sono_risolvibili() {
        assert_eq!(LIVELLI.len(), 50);
        for (i, l) in LIVELLI.iter().enumerate() {
            let minimo = fabbisogno_minimo(&l.obiettivo);
            assert!(
                l.max_moduli >= minimo,
                "livello {}: budget {} < fabbisogno {}",
                i + 1,
                l.max_moduli,
                minimo
            );
            assert!(
                area_libera_connessa(&l.ostacoli) >= l.max_moduli,
                "livello {}: la stazione non ci sta tra i detriti",
                i + 1
            );
        }
    }

    #[test]
    fn la_giornaliera_e_deterministica_nel_giorno_e_cambia_col_giorno() {
        let a = genera_giornaliera(20_000);
        let b = genera_giornaliera(20_000);
        assert_eq!(a.nome, b.nome);
        assert_eq!(a.max_moduli, b.max_moduli);
        assert_eq!(a.ostacoli, b.ostacoli);
        assert!(a.nome.starts_with("Sfida del giorno — "));
        // giorni diversi: almeno un tratto del livello cambia
        let c = genera_giornaliera(20_001);
        assert!(a.nome != c.nome || a.ostacoli != c.ostacoli || a.max_moduli != c.max_moduli);
        // e resta risolvibile come ogni livello generato
        assert!(a.max_moduli >= fabbisogno_minimo(&a.obiettivo));
        assert!(area_libera_connessa(&a.ostacoli) >= a.max_moduli);
    }

    #[test]
    fn i_livelli_casuali_sono_risolvibili() {
        // 200 seed arbitrari ma fissi: il vincolo vale per qualunque seed
        for seed in 0..200u64 {
            let l = genera_casuale(seed.wrapping_mul(0x9E37_79B9));
            assert!(l.max_moduli >= fabbisogno_minimo(&l.obiettivo));
            assert!(area_libera_connessa(&l.ostacoli) >= l.max_moduli);
        }
    }

    /// Stima PRUDENTE dei tick minimi per completare un obiettivo, usata
    /// solo dal test dell'oro. Assunzioni documentate:
    /// - gli arrivi vanno a 1 ogni `TICK_ARRIVO` (4) tick, da zero, senza
    ///   Centro comando;
    /// - i laboratori impegnano 2 persone l'uno, che devono prima arrivare;
    /// - PuntiSenzaBlackout: equipaggio plausibile a regime 12 (tre
    ///   dormitori stanno in ogni budget); la rampa (equipaggio che sale
    ///   di 1 ogni 4 tick fino a 12) dura 48 tick e vale 4·(1+…+12) = 312
    ///   punti, il resto arriva a 12 punti/tick.
    fn tempo_intrinseco(ob: &Obiettivo) -> u64 {
        let cadenza = crate::sim::TICK_ARRIVO as u64;
        match *ob {
            Obiettivo::Equipaggio { minimo } => minimo as u64 * cadenza,
            Obiettivo::LabConsecutivi { laboratori, tick } => {
                2 * laboratori as u64 * cadenza + tick as u64
            }
            Obiettivo::SopravviviConLab { laboratori, tick } => {
                2 * laboratori as u64 * cadenza + tick as u64
            }
            Obiettivo::PuntiSenzaBlackout { punti } => {
                const RAMPA_TICK: u64 = 48;
                const RAMPA_PUNTI: u64 = 312;
                if punti <= RAMPA_PUNTI {
                    RAMPA_TICK
                } else {
                    RAMPA_TICK + (punti - RAMPA_PUNTI).div_ceil(12)
                }
            }
            Obiettivo::Colonia { equipaggio, tick } => (equipaggio as u64 * cadenza).max(tick),
        }
    }

    #[test]
    fn l_oro_e_matematicamente_possibile_su_ogni_livello() {
        use crate::progressi::{ORO, medaglia_per_tempo};
        use crate::sim::TICK_MASSIMO;
        for (i, l) in LIVELLI.iter().enumerate() {
            let t = tempo_intrinseco(&l.obiettivo);
            assert_eq!(
                medaglia_per_tempo(t, TICK_MASSIMO),
                ORO,
                "livello {}: tempo intrinseco {} sopra la soglia oro",
                i + 1,
                t
            );
        }
        for seed in 0..200u64 {
            let l = genera_casuale(seed.wrapping_mul(0x9E37_79B9));
            let t = tempo_intrinseco(&l.obiettivo);
            assert_eq!(medaglia_per_tempo(t, TICK_MASSIMO), ORO);
        }
    }
}
