//! Progressi persistenti del giocatore oltre la progressione di campagna:
//! medaglie per livello, crediti e scorte del Marketplace.
//!
//! Le medaglie premiano la VELOCITÀ: completare un livello entro il 40%
//! del tempo limite vale l'oro, entro il 70% l'argento, entro il limite il
//! rame. I crediti sono volutamente pochi (3/2/1 per medaglia, e solo la
//! DIFFERENZA quando si migliora una medaglia già presa): gli aiuti del
//! Marketplace accorciano i livelli, non devono essere gratis.
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

/// La medaglia per un livello completato in `tick` su un tetto di `tetto`.
/// Chi completa ha SEMPRE almeno il rame: il tetto stesso è la soglia.
pub fn medaglia_per_tempo(tick: u64, tetto: u64) -> Medaglia {
    if tick * 10 <= tetto * 4 {
        ORO
    } else if tick * 10 <= tetto * 7 {
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
        let _ = std::fs::write(
            dir.join("progressi.txt"),
            format!(
                "crediti={}\nmedaglie={}\nscorte={}\n",
                self.crediti,
                medaglie.join(","),
                scorte.join(",")
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
            _ => {}
        }
    }
    p
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn le_soglie_delle_medaglie_sono_40_e_70_percento() {
        assert_eq!(medaglia_per_tempo(160, 400), ORO);
        assert_eq!(medaglia_per_tempo(161, 400), ARGENTO);
        assert_eq!(medaglia_per_tempo(280, 400), ARGENTO);
        assert_eq!(medaglia_per_tempo(281, 400), RAME);
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
