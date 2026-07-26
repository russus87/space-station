//! Catalogo delle facilities e uso delle SCORTE in partita.
//!
//! Le facilities si comprano nel **Marketplace** del menu principale
//! (`menu.rs`) coi CREDITI delle medaglie (`progressi.rs`) — pochi e
//! sudati, mai valuta reale. Quelle comprate diventano scorte persistenti
//! (`Portafoglio::scorte`); in partita vivono come ICONE nella colonna
//! sinistra (`ui.rs`: fila sopra il pannello ispezione, tooltip al
//! passaggio del mouse): click sull'icona = si consuma la scorta e
//! l'effetto si applica subito. Una scorta che ADESSO non avrebbe effetto
//! — ampliamento senza budget, sonda senza detriti, coloni senza posti
//! liberi, ossigeno a riserva piena, spurgo a stazione fredda, riparazione
//! senza avarie — si mostra smorzata col motivo nel tooltip e non si può
//! sprecare (`motivo_non_applicabile`, ricontrollato anche al click).

use crate::progressi::Portafoglio;
use crate::sim::{EventLog, Module, O2_MAX, Sim};
use crate::{GRID_H, GRID_W, Ostacolo, Station};
use bevy::prelude::*;

pub struct FacilityDef {
    pub nome: &'static str,
    pub descrizione: &'static str,
    /// Costo nel Marketplace, in crediti-medaglia. Volutamente caro
    /// rispetto a quanto rendono le medaglie (3/2/1): questi aiuti
    /// accorciano i livelli, non devono piovere.
    pub costo_crediti: u32,
    effetto: Effetto,
}

#[derive(Clone, Copy)]
enum Effetto {
    Ossigeno,
    Riparazione,
    Coloni,
    Stiva,
    Spurgo,
    Demolizione,
}

/// Il catalogo completo, in ordine di prezzo crescente a parità di tema.
pub const FACILITIES: [FacilityDef; 6] = [
    FacilityDef {
        nome: "Scorta d'ossigeno",
        descrizione: "riserva d'ossigeno subito al massimo",
        costo_crediti: 2,
        effetto: Effetto::Ossigeno,
    },
    FacilityDef {
        nome: "Squadra di riparazione",
        descrizione: "ripara tutte le avarie in un colpo",
        costo_crediti: 3,
        effetto: Effetto::Riparazione,
    },
    FacilityDef {
        nome: "Trasporto coloni",
        descrizione: "+2 equipaggio subito, se ci sono posti",
        costo_crediti: 4,
        effetto: Effetto::Coloni,
    },
    FacilityDef {
        nome: "Ampliamento stiva",
        descrizione: "+2 al budget moduli del livello",
        costo_crediti: 3,
        effetto: Effetto::Stiva,
    },
    FacilityDef {
        nome: "Spurgo termico",
        descrizione: "azzera il surriscaldamento accumulato",
        costo_crediti: 2,
        effetto: Effetto::Spurgo,
    },
    FacilityDef {
        nome: "Sonda demolitrice",
        descrizione: "rimuove il detrito più vicino al centro",
        costo_crediti: 5,
        effetto: Effetto::Demolizione,
    },
];

/// Residuo dell'overlay scorte (rimosso: le scorte sono icone in `ui.rs`).
/// Resta come risorsa sempre-`false` perché main.rs la consulta nelle run
/// condition (`costruzione_permessa`, cursore): si può eliminare insieme a
/// quei riferimenti.
#[derive(Resource, Default)]
pub struct Mercato {
    pub aperto: bool,
}

/// Le scorte raggruppate: (indice di catalogo, quante ne hai), in ordine
/// di catalogo. Indici fuori catalogo (file scritto a mano) si ignorano.
pub fn conteggio_scorte(scorte: &[usize]) -> Vec<(usize, usize)> {
    let mut conta = vec![0usize; FACILITIES.len()];
    for &s in scorte {
        if s < FACILITIES.len() {
            conta[s] += 1;
        }
    }
    conta
        .iter()
        .enumerate()
        .filter(|e| *e.1 > 0)
        .map(|(i, &n)| (i, n))
        .collect()
}

/// Perché la scorta di catalogo `indice` NON è usabile ADESSO; `None` =
/// usabile. Copre sia i vincoli di livello (budget, detriti) sia lo stato
/// corrente della simulazione: una scorta non si spreca mai per un effetto
/// nullo — niente coloni senza posti, niente ossigeno a riserva piena,
/// niente riparazioni senza avarie, niente spurghi a stazione fredda.
pub fn motivo_non_applicabile(
    indice: usize,
    station: &Station,
    sim: &Sim,
    avarie: usize,
) -> Option<&'static str> {
    let effetto = FACILITIES.get(indice)?.effetto;
    match effetto {
        Effetto::Stiva if station.max_moduli.is_none() => Some("qui non c'è un budget moduli"),
        Effetto::Demolizione if station.ostacoli.is_empty() => Some("non ci sono detriti"),
        Effetto::Coloni if sim.posti_letto <= sim.equipaggio => {
            Some("non ci sono posti letto liberi")
        }
        Effetto::Ossigeno if sim.ossigeno >= O2_MAX => Some("riserva già al massimo"),
        Effetto::Spurgo if sim.surriscaldamento == 0 => Some("nessun surriscaldamento"),
        Effetto::Riparazione if avarie == 0 => Some("nessun modulo in avaria"),
        _ => None,
    }
}

/// L'applicabilità di tutto il catalogo: la fila di icone in `ui.rs` si
/// ricostruisce quando questo vettore cambia (la sim gira e una scorta può
/// diventare inutile — o utile — da un tick all'altro).
pub fn applicabilita(station: &Station, sim: &Sim, avarie: usize) -> Vec<bool> {
    (0..FACILITIES.len())
        .map(|i| motivo_non_applicabile(i, station, sim, avarie).is_none())
        .collect()
}

// ---------------- uso delle scorte ----------------

/// Click su un'icona scorta (`ui::IconaScorta`): consuma e applica.
#[allow(clippy::too_many_arguments)]
pub fn click_scorte(
    q: Query<(&Interaction, &crate::ui::IconaScorta), Changed<Interaction>>,
    mut commands: Commands,
    mut portafoglio: ResMut<Portafoglio>,
    mut sim: ResMut<Sim>,
    mut station: ResMut<Station>,
    mut log: ResMut<EventLog>,
    suoni: Res<crate::audio::Suoni>,
    imp: Res<crate::impostazioni::Impostazioni>,
    mut moduli: Query<&mut Module>,
    ostacoli_q: Query<(Entity, &Ostacolo)>,
) {
    for (interazione, icona) in &q {
        if *interazione != Interaction::Pressed {
            continue;
        }
        let Some(f) = FACILITIES.get(icona.0) else {
            continue;
        };
        // doppio controllo: l'icona smorzata è già muta e inerte, ma tra
        // frame e click lo stato può essere cambiato — e una scorta non si
        // consuma MAI per un effetto nullo
        let avarie = moduli.iter().filter(|m| m.broken).count();
        if motivo_non_applicabile(icona.0, &station, &sim, avarie).is_some() {
            continue;
        }
        if !portafoglio.usa(icona.0) {
            continue;
        }
        match f.effetto {
            Effetto::Ossigeno => {
                sim.ossigeno = O2_MAX;
            }
            Effetto::Riparazione => {
                for mut m in &mut moduli {
                    m.broken = false;
                }
            }
            Effetto::Coloni => {
                let posti_liberi = sim.posti_letto.saturating_sub(sim.equipaggio);
                sim.equipaggio += posti_liberi.min(2);
            }
            Effetto::Stiva => {
                if let Some(max) = &mut station.max_moduli {
                    *max += 2;
                }
            }
            Effetto::Spurgo => {
                sim.surriscaldamento = 0;
            }
            Effetto::Demolizione => {
                // il detrito più vicino al centro della griglia: quello che
                // con più probabilità sta strozzando la stazione
                let centro = Vec2::new(GRID_W as f32 / 2.0 - 0.5, GRID_H as f32 / 2.0 - 0.5);
                if let Some(cella) = station
                    .ostacoli
                    .iter()
                    .copied()
                    .min_by_key(|c| (Vec2::new(c.x as f32, c.y as f32).distance_squared(centro)
                        * 100.0) as i64)
                {
                    station.ostacoli.remove(&cella);
                    if let Some((e, _)) = ostacoli_q.iter().find(|(_, o)| o.cella == cella) {
                        commands.entity(e).despawn();
                    }
                }
            }
        }
        log.info(sim.tick, format!("Scorte: {} usata", f.nome));
        crate::audio::suona(&mut commands, &suoni.acquisto, imp.effetti_lineare());
    }
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn il_conteggio_raggruppa_e_ordina_per_catalogo() {
        let scorte = vec![4, 0, 4, 5];
        assert_eq!(conteggio_scorte(&scorte), vec![(0, 1), (4, 2), (5, 1)]);
        assert!(conteggio_scorte(&[]).is_empty());
    }

    #[test]
    fn gli_indici_fuori_catalogo_non_rompono_il_conteggio() {
        // un progressi.txt scritto a mano non deve far crashare la fila
        assert_eq!(conteggio_scorte(&[99, 2]), vec![(2, 1)]);
    }

    #[test]
    fn un_indice_fuori_catalogo_non_panica() {
        let station = Station::default();
        let sim = Sim::default();
        // il caso non si presenta (le icone nascono dal conteggio filtrato)
        // ma un progressi.txt a mano non deve far crollare nulla
        motivo_non_applicabile(99, &station, &sim, 0);
        // il vettore di applicabilità copre esattamente il catalogo
        assert_eq!(applicabilita(&station, &sim, 0).len(), FACILITIES.len());
    }
}
