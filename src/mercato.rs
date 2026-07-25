//! Catalogo delle facilities e SCORTE in partita.
//!
//! Le facilities si comprano nel **Marketplace** del menu principale
//! (`menu.rs`) coi CREDITI delle medaglie (`progressi.rs`) — pochi e
//! sudati, mai valuta reale. Quelle comprate diventano scorte persistenti
//! (`Portafoglio::scorte`); in partita il tasto `M` (o il bottone
//! nell'HUD) apre questo overlay, che le elenca: click su una scorta = la
//! si consuma e l'effetto si applica subito. Una scorta che qui non
//! avrebbe effetto (ampliamento senza budget moduli, sonda senza detriti)
//! si mostra spenta col motivo e non si può sprecare.

use crate::menu::{AppState, Pausa};
use crate::progressi::Portafoglio;
use crate::sim::{EventLog, Module, O2_MAX, Sim};
use crate::ui::{BIANCO, CIANO, GIALLO, GRIGIO_MEDIO, GRIGIO_SCAFO, METALLO, NERO, SCAFO_SCURO};
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

/// Stato dell'overlay scorte: solo aperto/chiuso. L'inventario vero vive
/// in `Portafoglio` (persistente), non qui.
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

/// Perché una scorta NON è usabile in questa partita; `None` = usabile.
fn non_applicabile(effetto: Effetto, station: &Station) -> Option<&'static str> {
    match effetto {
        Effetto::Stiva if station.max_moduli.is_none() => Some("qui non c'è un budget moduli"),
        Effetto::Demolizione if station.ostacoli.is_empty() => Some("non ci sono detriti"),
        _ => None,
    }
}

// ---------------- input ----------------

/// `M` apre e chiude le scorte (solo in partita, mai sotto il menu Esc).
pub fn toggle_tasto(
    tasti: Res<ButtonInput<KeyCode>>,
    pausa: Res<Pausa>,
    mut mercato: ResMut<Mercato>,
) {
    if pausa.aperta {
        return;
    }
    if tasti.just_pressed(KeyCode::KeyM) {
        mercato.aperto = !mercato.aperto;
    }
}

/// Il bottone nell'HUD (marker in ui.rs) fa lo stesso di `M`.
pub fn click_bottone(
    q: Query<&Interaction, (Changed<Interaction>, With<crate::ui::BottoneMercato>)>,
    pausa: Res<Pausa>,
    mut mercato: ResMut<Mercato>,
) {
    for interazione in &q {
        if *interazione == Interaction::Pressed && !pausa.aperta {
            mercato.aperto = !mercato.aperto;
        }
    }
}

// ---------------- overlay ----------------

#[derive(Component)]
pub struct SchermataMercato;

/// Una riga di scorta usabile; il valore è l'indice nel catalogo.
#[derive(Component)]
pub struct VoceMercato(pub usize);

fn testo(t: impl Into<String>, px: f32, colore: Color) -> impl Bundle {
    (
        Text::new(t),
        TextFont {
            font_size: FontSize::Px(px),
            ..default()
        },
        TextColor(colore),
    )
}

/// L'overlay esiste solo quando le scorte sono aperte, in partita e senza
/// il menu di pausa sopra; si ricostruisce quando l'inventario cambia.
pub fn sincronizza(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    pausa: Res<Pausa>,
    mercato: Res<Mercato>,
    portafoglio: Res<Portafoglio>,
    station: Res<Station>,
    q: Query<Entity, With<SchermataMercato>>,
) {
    let deve_esserci = mercato.aperto && *stato.get() == AppState::InGioco && !pausa.aperta;
    let c_e = !q.is_empty();
    if deve_esserci == c_e && !(deve_esserci && portafoglio.is_changed()) {
        return;
    }
    for e in &q {
        commands.entity(e).despawn();
    }
    if !deve_esserci {
        return;
    }
    let scorte = conteggio_scorte(&portafoglio.scorte);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(NERO.with_alpha(0.7)),
            GlobalZIndex(15),
            SchermataMercato,
        ))
        .with_children(|r| {
            r.spawn(testo("SCORTE", 34.0, BIANCO));
            r.spawn(testo(
                "si comprano nel Marketplace, dal titolo — le medaglie fruttano crediti",
                13.0,
                GRIGIO_MEDIO,
            ));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(
                    format!("hai {} crediti", portafoglio.crediti),
                    15.0,
                    GIALLO,
                ));
            });
            if scorte.is_empty() {
                r.spawn(testo(
                    "Nessuna scorta: compra nel Marketplace dal titolo",
                    15.0,
                    GRIGIO_MEDIO,
                ));
            }
            for (idx, quante) in scorte {
                let f = &FACILITIES[idx];
                let riga = if quante > 1 {
                    format!("{} ×{}  —  {}", f.nome, quante, f.descrizione)
                } else {
                    format!("{}  —  {}", f.nome, f.descrizione)
                };
                if let Some(motivo) = non_applicabile(f.effetto, &station) {
                    r.spawn(testo(
                        format!("{riga}  (qui non serve: {motivo})"),
                        15.0,
                        GRIGIO_SCAFO,
                    ));
                } else {
                    r.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(SCAFO_SCURO),
                        BorderColor::all(CIANO),
                        Button,
                        VoceMercato(idx),
                    ))
                    .with_children(|c| {
                        c.spawn(testo(riga, 15.0, METALLO));
                    });
                }
            }
            r.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });
            r.spawn(testo("M per chiudere", 12.0, GRIGIO_MEDIO));
        });
}

// ---------------- uso delle scorte ----------------

#[allow(clippy::too_many_arguments)]
pub fn click_scorte(
    q: Query<(&Interaction, &VoceMercato), Changed<Interaction>>,
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
    for (interazione, voce) in &q {
        if *interazione != Interaction::Pressed {
            continue;
        }
        let Some(f) = FACILITIES.get(voce.0) else {
            continue;
        };
        // doppio controllo: l'overlay non mostra bottoni per le scorte non
        // applicabili, ma tra spawn e click la griglia può essere cambiata
        if non_applicabile(f.effetto, &station).is_some() {
            continue;
        }
        if !portafoglio.usa(voce.0) {
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
        // un progressi.txt scritto a mano non deve far crashare l'overlay
        assert_eq!(conteggio_scorte(&[99, 2]), vec![(2, 1)]);
    }
}
