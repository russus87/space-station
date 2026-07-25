//! Mercato interno: facilities una tantum comprate coi PUNTI della partita.
//!
//! Nessuna valuta reale e nessuna transazione: si paga col punteggio, che
//! scende — comprare aiuta adesso ma costa in classifica, ed è tutta lì la
//! decisione. Ogni partita pesca 3 offerte dal catalogo `FACILITIES`
//! (filtrate su ciò che ha senso: l'ampliamento stiva solo dove c'è un
//! budget moduli, la sonda solo se ci sono detriti); ognuna si compra una
//! volta sola. Tasto `M` o bottone MERCATO nell'HUD.

use crate::menu::{AppState, Pausa};
use crate::sim::{EventLog, Gravita, Module, O2_MAX, Sim};
use crate::ui::{BIANCO, CIANO, GIALLO, GRIGIO_MEDIO, GRIGIO_SCAFO, METALLO, NERO, SCAFO_SCURO};
use crate::{GRID_H, GRID_W, Ostacolo, Station};
use bevy::prelude::*;
use rand::RngExt;

pub struct FacilityDef {
    pub nome: &'static str,
    pub descrizione: &'static str,
    pub costo: u64,
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

/// Il catalogo completo. I costi sono in punti partita (persone·tick):
/// tarati perché comprare presto sia quasi impossibile e comprare tardi
/// sia una rinuncia visibile in classifica.
pub const FACILITIES: [FacilityDef; 6] = [
    FacilityDef {
        nome: "Scorta d'ossigeno",
        descrizione: "riserva d'ossigeno subito al massimo",
        costo: 80,
        effetto: Effetto::Ossigeno,
    },
    FacilityDef {
        nome: "Squadra di riparazione",
        descrizione: "ripara tutte le avarie in un colpo",
        costo: 120,
        effetto: Effetto::Riparazione,
    },
    FacilityDef {
        nome: "Trasporto coloni",
        descrizione: "+2 equipaggio subito, se ci sono posti",
        costo: 150,
        effetto: Effetto::Coloni,
    },
    FacilityDef {
        nome: "Ampliamento stiva",
        descrizione: "+2 al budget moduli del livello",
        costo: 100,
        effetto: Effetto::Stiva,
    },
    FacilityDef {
        nome: "Spurgo termico",
        descrizione: "azzera il surriscaldamento accumulato",
        costo: 60,
        effetto: Effetto::Spurgo,
    },
    FacilityDef {
        nome: "Sonda demolitrice",
        descrizione: "rimuove il detrito più vicino al centro",
        costo: 200,
        effetto: Effetto::Demolizione,
    },
];

/// Le offerte della partita in corso. Si rinnova a ogni reset.
#[derive(Resource, Default)]
pub struct Mercato {
    pub aperto: bool,
    /// (indice in FACILITIES, già comprata)
    pub offerte: Vec<(usize, bool)>,
}

impl Mercato {
    /// Pesca 3 offerte valide per la nuova partita. Il seed è il rand di
    /// sistema: le offerte sono la parte "random" voluta, non riproducibile.
    pub fn rinnova(&mut self, con_budget: bool, con_detriti: bool) {
        self.aperto = false;
        let mut valide: Vec<usize> = (0..FACILITIES.len())
            .filter(|&i| match FACILITIES[i].effetto {
                Effetto::Stiva => con_budget,
                Effetto::Demolizione => con_detriti,
                _ => true,
            })
            .collect();
        let mut rng = rand::rng();
        self.offerte.clear();
        for _ in 0..3.min(valide.len()) {
            let scelta = rng.random_range(0..valide.len());
            self.offerte.push((valide.swap_remove(scelta), false));
        }
        // ordine di catalogo, non di pesca: la lista a schermo è stabile
        self.offerte.sort_by_key(|&(i, _)| i);
    }
}

// ---------------- input ----------------

/// `M` apre e chiude il mercato (solo in partita, mai sotto il menu Esc).
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

/// Il bottone MERCATO nell'HUD (marker in ui.rs) fa lo stesso di `M`.
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

/// Una riga d'offerta comprabile; l'indice è la posizione in `offerte`.
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

/// L'overlay esiste solo quando il mercato è aperto, in partita e senza il
/// menu di pausa sopra; si ricostruisce quando cambia qualcosa (acquisti).
pub fn sincronizza(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    pausa: Res<Pausa>,
    mercato: Res<Mercato>,
    sim: Res<Sim>,
    q: Query<Entity, With<SchermataMercato>>,
) {
    let deve_esserci = mercato.aperto && *stato.get() == AppState::InGioco && !pausa.aperta;
    let c_e = !q.is_empty();
    if deve_esserci == c_e && !(deve_esserci && (mercato.is_changed() || sim.is_changed())) {
        return;
    }
    for e in &q {
        commands.entity(e).despawn();
    }
    if !deve_esserci {
        return;
    }
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
            r.spawn(testo("MERCATO", 34.0, BIANCO));
            r.spawn(testo(
                "si paga in punti partita — niente soldi veri, mai",
                13.0,
                GRIGIO_MEDIO,
            ));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(
                    format!("hai {} punti", sim.punteggio),
                    15.0,
                    GIALLO,
                ));
            });
            for (pos, &(idx, comprata)) in mercato.offerte.iter().enumerate() {
                let f = &FACILITIES[idx];
                let riga = format!(
                    "{}  —  {}  —  {} punti",
                    f.nome, f.descrizione, f.costo
                );
                if comprata {
                    r.spawn(testo(format!("{riga}  (comprata)"), 15.0, GRIGIO_SCAFO));
                } else {
                    r.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(SCAFO_SCURO),
                        BorderColor::all(if sim.punteggio >= f.costo {
                            CIANO
                        } else {
                            GRIGIO_SCAFO
                        }),
                        Button,
                        VoceMercato(pos),
                    ))
                    .with_children(|c| {
                        c.spawn(testo(
                            riga,
                            15.0,
                            if sim.punteggio >= f.costo {
                                METALLO
                            } else {
                                GRIGIO_SCAFO
                            },
                        ));
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

// ---------------- acquisti ----------------

#[allow(clippy::too_many_arguments)]
pub fn click_offerte(
    q: Query<(&Interaction, &VoceMercato), Changed<Interaction>>,
    mut commands: Commands,
    mut mercato: ResMut<Mercato>,
    mut sim: ResMut<Sim>,
    mut station: ResMut<Station>,
    mut log: ResMut<EventLog>,
    suoni: Res<crate::audio::Suoni>,
    mut moduli: Query<&mut Module>,
    ostacoli_q: Query<(Entity, &Ostacolo)>,
) {
    for (interazione, voce) in &q {
        if *interazione != Interaction::Pressed {
            continue;
        }
        let Some(&(idx, comprata)) = mercato.offerte.get(voce.0) else {
            continue;
        };
        let f = &FACILITIES[idx];
        if comprata {
            continue;
        }
        if sim.punteggio < f.costo {
            log.push(
                sim.tick,
                Gravita::Avviso,
                format!("{}: servono {} punti", f.nome, f.costo),
            );
            continue;
        }
        sim.punteggio -= f.costo;
        mercato.offerte[voce.0].1 = true;
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
        log.info(sim.tick, format!("Mercato: {} ({} punti)", f.nome, f.costo));
        crate::audio::suona(&mut commands, &suoni.acquisto);
    }
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn le_offerte_rispettano_i_requisiti_del_livello() {
        // senza budget e senza detriti, stiva e demolizione non compaiono mai
        for _ in 0..50 {
            let mut m = Mercato::default();
            m.rinnova(false, false);
            assert!(m.offerte.len() <= 3);
            for &(i, comprata) in &m.offerte {
                assert!(!comprata);
                assert!(!matches!(
                    FACILITIES[i].effetto,
                    Effetto::Stiva | Effetto::Demolizione
                ));
            }
        }
    }

    #[test]
    fn con_tutto_disponibile_si_pescano_sempre_tre_offerte_distinte() {
        for _ in 0..50 {
            let mut m = Mercato::default();
            m.rinnova(true, true);
            assert_eq!(m.offerte.len(), 3);
            let mut idx: Vec<usize> = m.offerte.iter().map(|&(i, _)| i).collect();
            idx.dedup();
            assert_eq!(idx.len(), 3, "offerte duplicate");
        }
    }
}
