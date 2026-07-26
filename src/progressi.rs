//! Progressi persistenti del giocatore oltre la progressione di campagna:
//! medaglie per livello, crediti e scorte del Marketplace.
//!
//! Le medaglie premiano la VELOCITÀ: completare un livello entro il 35%
//! del tempo limite vale l'oro, entro il 60% l'argento, entro il limite il
//! rame. (Erano 40/70: il playtest faceva oro troppo spesso. Non scendere
//! sotto il 35: l'obiettivo intrinsecamente più lungo dei 50 livelli vale
//! ~130 tick e la soglia oro su 400 è 140 — l'oro deve restare possibile
//! OVUNQUE, e un test in generatore.rs lo garantisce.) I crediti sono
//! volutamente pochi (3/2/1 per medaglia, e solo la DIFFERENZA quando si
//! migliora una medaglia già presa): gli aiuti del Marketplace accorciano
//! i livelli, non devono essere gratis.
//!
//! Persistenza in `progressi.txt` (cartella dati): righe `chiave=valore`,
//! stesso patto dei file classifica — una riga rotta si salta, file
//! assente = si parte da zero.

use crate::livelli::{LIVELLI, cartella_dati};
use bevy::prelude::*;

/// Nessuna medaglia = 0; rame 1, argento 2, oro 3.
pub type Medaglia = u8;

pub const ORO: Medaglia = 3;
pub const ARGENTO: Medaglia = 2;
pub const RAME: Medaglia = 1;

/// Crediti guadagnati da una medaglia (indice = medaglia).
const CREDITI_PER_MEDAGLIA: [u32; 4] = [0, 1, 2, 3];

/// Soglie delle medaglie in percento del tetto: UNICA definizione, usata
/// anche dalla UI per mostrare i tempi da battere.
pub const SOGLIA_ORO_PERCENTO: u64 = 35;
pub const SOGLIA_ARGENTO_PERCENTO: u64 = 60;

/// La medaglia per un livello completato in `tick` su un tetto di `tetto`.
/// Chi completa ha SEMPRE almeno il rame: il tetto stesso è la soglia.
pub fn medaglia_per_tempo(tick: u64, tetto: u64) -> Medaglia {
    if tick * 100 <= tetto * SOGLIA_ORO_PERCENTO {
        ORO
    } else if tick * 100 <= tetto * SOGLIA_ARGENTO_PERCENTO {
        ARGENTO
    } else {
        RAME
    }
}

/// Portafoglio del giocatore: crediti spendibili, scorte comprate (indici
/// nel catalogo `mercato::FACILITIES`, ripetibili) e medaglia migliore per
/// ciascun livello di campagna.
#[derive(Resource, Default)]
pub struct Portafoglio {
    pub crediti: u32,
    pub scorte: Vec<usize>,
    pub medaglie: Vec<Medaglia>,
    /// Bonus di livello già incassati (0/1 per livello): +1 credito una
    /// tantum, rigiocare non frutta due volte.
    pub bonus: Vec<u8>,
    /// Sfida del giorno: il giorno (giorni dall'epoch) e il miglior tempo
    /// in tick di quel giorno. Un giorno nuovo azzera il record.
    pub giorno: u64,
    pub giorno_tick: u64,
}

impl Portafoglio {
    /// Registra l'esito di un livello (0-based) completato in `tick` col
    /// tetto `tetto`: aggiorna la medaglia se migliore e accredita la
    /// DIFFERENZA di crediti. Ritorna (medaglia di questa run, crediti
    /// guadagnati adesso). Salva su disco se qualcosa è cambiato.
    pub fn registra_livello(&mut self, livello: usize, tick: u64, tetto: u64) -> (Medaglia, u32) {
        if self.medaglie.len() < LIVELLI.len() {
            self.medaglie.resize(LIVELLI.len(), 0);
        }
        let medaglia = medaglia_per_tempo(tick, tetto);
        let precedente = self.medaglie[livello];
        let guadagno = if medaglia > precedente {
            self.medaglie[livello] = medaglia;
            let delta = CREDITI_PER_MEDAGLIA[medaglia as usize]
                - CREDITI_PER_MEDAGLIA[precedente as usize];
            self.crediti += delta;
            self.salva();
            delta
        } else {
            0
        };
        (medaglia, guadagno)
    }

    pub fn medaglia(&self, livello: usize) -> Medaglia {
        self.medaglie.get(livello).copied().unwrap_or(0)
    }

    /// Incassa il bonus del livello se non era già stato incassato:
    /// +1 credito e `true` solo la prima volta.
    pub fn registra_bonus(&mut self, livello: usize) -> bool {
        if self.bonus.len() < LIVELLI.len() {
            self.bonus.resize(LIVELLI.len(), 0);
        }
        if self.bonus[livello] != 0 {
            return false;
        }
        self.bonus[livello] = 1;
        self.crediti += 1;
        self.salva();
        true
    }

    /// Registra il tempo della sfida del giorno; ritorna (nuovo record,
    /// miglior tempo di oggi). Un giorno diverso riparte da zero.
    pub fn registra_giornaliera(&mut self, giorno: u64, tick: u64) -> (bool, u64) {
        let record = self.giorno != giorno || tick < self.giorno_tick;
        if record {
            self.giorno = giorno;
            self.giorno_tick = tick;
            self.salva();
        }
        (record, self.giorno_tick)
    }

    /// La sfida di questo giorno è già stata completata almeno una volta?
    pub fn giornaliera_fatta(&self, giorno: u64) -> bool {
        self.giorno == giorno && self.giorno_tick > 0
    }

    /// Compra una facility (indice di catalogo) se i crediti bastano.
    pub fn compra(&mut self, indice: usize, costo: u32) -> bool {
        if self.crediti < costo {
            return false;
        }
        self.crediti -= costo;
        self.scorte.push(indice);
        self.scorte.sort_unstable();
        self.salva();
        true
    }

    /// Consuma una scorta posseduta; `true` se c'era.
    pub fn usa(&mut self, indice: usize) -> bool {
        if let Some(pos) = self.scorte.iter().position(|&s| s == indice) {
            self.scorte.remove(pos);
            self.salva();
            true
        } else {
            false
        }
    }

    pub fn salva(&self) {
        let Some(dir) = cartella_dati() else {
            return;
        };
        let _ = std::fs::create_dir_all(&dir);
        let medaglie: Vec<String> = self.medaglie.iter().map(|m| m.to_string()).collect();
        let scorte: Vec<String> = self.scorte.iter().map(|s| s.to_string()).collect();
        let bonus: Vec<String> = self.bonus.iter().map(|b| b.to_string()).collect();
        let _ = std::fs::write(
            dir.join("progressi.txt"),
            format!(
                "crediti={}\nmedaglie={}\nscorte={}\nbonus={}\ngiorno={}\ngiorno_tick={}\n",
                self.crediti,
                medaglie.join(","),
                scorte.join(","),
                bonus.join(","),
                self.giorno,
                self.giorno_tick
            ),
        );
    }
}

pub fn carica() -> Portafoglio {
    let mut p = Portafoglio::default();
    let Some(testo) = cartella_dati()
        .map(|d| d.join("progressi.txt"))
        .and_then(|f| std::fs::read_to_string(f).ok())
    else {
        return p;
    };
    for riga in testo.lines() {
        let Some((chiave, valore)) = riga.split_once('=') else {
            continue;
        };
        match chiave.trim() {
            "crediti" => p.crediti = valore.trim().parse().unwrap_or(0),
            "medaglie" => {
                p.medaglie = valore
                    .split(',')
                    .filter_map(|v| v.trim().parse::<Medaglia>().ok())
                    .map(|m| m.min(ORO))
                    .collect();
            }
            "scorte" => {
                p.scorte = valore
                    .split(',')
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
            }
            "bonus" => {
                p.bonus = valore
                    .split(',')
                    .filter_map(|v| v.trim().parse::<u8>().ok())
                    .map(|b| b.min(1))
                    .collect();
            }
            "giorno" => p.giorno = valore.trim().parse().unwrap_or(0),
            "giorno_tick" => p.giorno_tick = valore.trim().parse().unwrap_or(0),
            _ => {}
        }
    }
    p
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn le_soglie_delle_medaglie_sono_35_e_60_percento() {
        assert_eq!(medaglia_per_tempo(140, 400), ORO);
        assert_eq!(medaglia_per_tempo(141, 400), ARGENTO);
        assert_eq!(medaglia_per_tempo(240, 400), ARGENTO);
        assert_eq!(medaglia_per_tempo(241, 400), RAME);
        assert_eq!(medaglia_per_tempo(400, 400), RAME);
    }

    #[test]
    fn migliorare_una_medaglia_accredita_solo_la_differenza() {
        let mut p = Portafoglio::default();
        let (m, g) = p.registra_livello(0, 400, 400);
        assert_eq!((m, g), (RAME, 1));
        // stessa medaglia: nessun credito, niente farming
        let (_, g) = p.registra_livello(0, 300, 400);
        assert_eq!(g, 0);
        // salto rame → oro: la differenza, non il pieno
        let (m, g) = p.registra_livello(0, 100, 400);
        assert_eq!((m, g), (ORO, 2));
        assert_eq!(p.crediti, 3);
    }

    #[test]
    fn il_bonus_di_un_livello_si_incassa_una_volta_sola() {
        let mut p = Portafoglio::default();
        assert!(p.registra_bonus(3));
        assert_eq!(p.crediti, 1);
        assert!(!p.registra_bonus(3)); // niente farming
        assert_eq!(p.crediti, 1);
        assert!(p.registra_bonus(4));
        assert_eq!(p.crediti, 2);
    }

    #[test]
    fn la_giornaliera_tiene_il_miglior_tempo_e_riparte_col_giorno_nuovo() {
        let mut p = Portafoglio::default();
        let (record, best) = p.registra_giornaliera(100, 200);
        assert!(record);
        assert_eq!(best, 200);
        assert!(p.giornaliera_fatta(100));
        // tempo peggiore: nessun record, resta il migliore
        let (record, best) = p.registra_giornaliera(100, 250);
        assert!(!record);
        assert_eq!(best, 200);
        // tempo migliore: record
        let (record, best) = p.registra_giornaliera(100, 150);
        assert!(record);
        assert_eq!(best, 150);
        // giorno nuovo: si riparte
        assert!(!p.giornaliera_fatta(101));
        let (record, best) = p.registra_giornaliera(101, 300);
        assert!(record);
        assert_eq!(best, 300);
    }

    #[test]
    fn comprare_e_usare_scorte_muove_i_crediti_e_l_inventario() {
        let mut p = Portafoglio {
            crediti: 5,
            ..Default::default()
        };
        assert!(p.compra(2, 3));
        assert_eq!(p.crediti, 2);
        assert!(!p.compra(2, 3)); // non bastano più
        assert!(p.usa(2));
        assert!(!p.usa(2)); // esaurita
    }
}
