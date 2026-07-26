//! La squadra schierata: tratti passivi dei cinque personaggi
//! (SPEC-CAMPAGNA §4, approvata). Si sceglie CHI portare nel prologo del
//! livello (o si parte senza nessuno); lo sblocco arriva coi traguardi di
//! campagna 10/20/30/40/50, alternati agli sblocchi dei moduli (5/15/...).
//!
//! I tratti NON aggiungono regole: piegano i numeri esistenti, e la
//! simulazione li legge da qui con i default = valori attuali quando non
//! c'è nessuno a bordo plancia. Ilse è speciale: schierarla apre un
//! secondo posto in squadra.

use crate::sim::{TICK_ARRIVO, TICK_MORTE};
use bevy::prelude::*;

/// Indici = personaggi::PERSONAGGI (Vera, Tomas, Dario, Mira, Ilse).
pub const TRATTI: [&str; 5] = [
    "Il calore dei reattori scende del 25%",
    "Asfissia dimezzata: un morto ogni 6 tick invece di 3",
    "Arrivi più rapidi: uno ogni 3 tick invece di 4",
    "I laboratori consumano il 25% di energia in meno",
    "Comando doppio: puoi schierare due persone",
];

/// Livelli di campagna da completare per sbloccare il personaggio i-esimo.
pub const SBLOCCHI: [usize; 5] = [10, 20, 30, 40, 50];

pub fn sbloccato(personaggio: usize, completati: usize) -> bool {
    SBLOCCHI
        .get(personaggio)
        .is_some_and(|&soglia| completati >= soglia)
}

/// Chi è a bordo plancia in questa partita. Non persiste: si sceglie nel
/// prologo (campagna/casuale) e si azzera a ogni reset.
#[derive(Resource, Default)]
pub struct Squadra {
    pub schierati: Vec<usize>,
}

impl Squadra {
        pub fn contiene(&self, personaggio: usize) -> bool {
        self.schierati.contains(&personaggio)
    }

    /// Posti disponibili: 1, o 2 se Ilse (4) è schierata.
        pub fn posti(&self) -> usize {
        if self.contiene(4) { 2 } else { 1 }
    }

    /// Schiera/rimuove un personaggio rispettando i posti. Togliere Ilse
    /// quando i posti scendono a 1 sbarca anche l'eventuale secondo.
        pub fn alterna(&mut self, personaggio: usize) {
        if let Some(pos) = self.schierati.iter().position(|&p| p == personaggio) {
            self.schierati.remove(pos);
            while self.schierati.len() > self.posti() {
                self.schierati.pop();
            }
        } else if self.schierati.len() < self.posti() {
            self.schierati.push(personaggio);
        }
    }

    // ---- i numeri piegati (default = gioco di oggi) ----

    /// Moltiplicatore del calore prodotto dai reattori (Vera).
        pub fn calore_reattore(&self) -> f32 {
        if self.contiene(0) { 0.75 } else { 1.0 }
    }

    /// Tick tra due morti per asfissia (Tomas).
        pub fn tick_morte(&self) -> u32 {
        if self.contiene(1) { TICK_MORTE * 2 } else { TICK_MORTE }
    }

    /// Tick tra due arrivi (Dario); il Centro comando dimezza DOPO.
        pub fn tick_arrivo(&self) -> u32 {
        if self.contiene(2) {
            TICK_ARRIVO - 1
        } else {
            TICK_ARRIVO
        }
    }

    /// Moltiplicatore del consumo energia dei laboratori (Mira).
        pub fn energia_lab(&self) -> f32 {
        if self.contiene(3) { 0.75 } else { 1.0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ilse_apre_il_secondo_posto_e_lo_richiude() {
        let mut s = Squadra::default();
        s.alterna(0);
        s.alterna(1); // niente: un posto solo
        assert_eq!(s.schierati, vec![0]);
        s.alterna(0); // sbarca Vera
        s.alterna(4); // Ilse: due posti
        s.alterna(2);
        assert_eq!(s.schierati, vec![4, 2]);
        s.alterna(4); // via Ilse: resta un posto solo, Dario lo tiene
        assert_eq!(s.schierati, vec![2]);
        s.alterna(2);
        assert!(s.schierati.is_empty());
    }

    #[test]
    fn i_default_senza_squadra_sono_il_gioco_di_oggi() {
        let s = Squadra::default();
        assert_eq!(s.calore_reattore(), 1.0);
        assert_eq!(s.tick_morte(), TICK_MORTE);
        assert_eq!(s.tick_arrivo(), TICK_ARRIVO);
        assert_eq!(s.energia_lab(), 1.0);
    }

    #[test]
    fn gli_sblocchi_seguono_i_traguardi_di_campagna() {
        assert!(!sbloccato(0, 9));
        assert!(sbloccato(0, 10));
        assert!(sbloccato(4, 50));
        assert!(!sbloccato(4, 49));
        assert!(!sbloccato(9, 100)); // indice fuori tabella
    }
}
