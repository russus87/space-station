//! Commenti in partita: i personaggi chiosano gli eventi chiave con un
//! mini-fumetto temporaneo in basso a destra, sopra il registro eventi.
//!
//! Il rilevamento osserva `Sim` (e le risorse esistenti) con stato locale:
//! niente hook dentro `sim.rs`, che resta intoccato. Gli eventi "prima
//! volta per partita" si riarmano quando il tick torna indietro (reset).
//! Un commento vive ~5 secondi e viene sostituito da quello successivo;
//! le varianti ruotano con un contatore, non a caso, così due partite di
//! fila non ripetono la stessa voce.

use crate::Art;
use crate::livelli::UltimaMedaglia;
use crate::menu::AppState;
use crate::personaggi::{EventoCommento, PERSONAGGI, commento};
use crate::progressi::ORO;
use crate::sim::{SOGLIA_O2_CRITICO, Sim};
use crate::ui::{BIANCO, CIANO, GRIGIO_SCAFO, SCAFO_SCURO};
use bevy::prelude::*;

/// Quanto resta a schermo un commento, in secondi.
const DURATA: f32 = 5.0;

/// Il mini-fumetto a schermo (al più uno alla volta).
#[derive(Component)]
pub struct Commento {
    timer: Timer,
}

/// Stato del rilevatore: cosa è già scattato in QUESTA partita.
#[derive(Default)]
pub struct Visti {
    ultimo_tick: u64,
    equipaggio_prima: u32,
    ostacoli_prima: usize,
    blackout: bool,
    avaria: bool,
    arrivo: bool,
    ossigeno: bool,
    tetto: bool,
    /// Contatore globale di rotazione delle varianti.
    variante: usize,
}

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

/// Osserva la partita e fa parlare i personaggi. Registrare in Update
/// dopo `applica_reset` (gruppo servizi/visuali).
#[allow(clippy::too_many_arguments)]
pub fn rileva_eventi(
    mut commands: Commands,
    mut visti: Local<Visti>,
    sim: Res<Sim>,
    medaglia: Res<UltimaMedaglia>,
    station: Res<crate::Station>,
    stato: Res<State<AppState>>,
    art: Res<Art>,
    in_corso: Query<Entity, With<Commento>>,
) {
    // reset partita: il tick è tornato indietro → si riarma tutto
    if sim.tick < visti.ultimo_tick {
        let variante = visti.variante;
        *visti = Visti {
            equipaggio_prima: sim.equipaggio,
            ostacoli_prima: station.ostacoli.len(),
            variante,
            ..Visti::default()
        };
    }
    visti.ultimo_tick = sim.tick;

    let mut evento = None;

    if *stato.get() == AppState::InGioco && !sim.partita_finita {
        if sim.in_blackout && !visti.blackout && sim.tick > 0 {
            visti.blackout = true;
            evento = Some(EventoCommento::PrimoBlackout);
        } else if sim.avarie_surriscaldamento > 0 && !visti.avaria {
            visti.avaria = true;
            evento = Some(EventoCommento::PrimaAvaria);
        } else if sim.equipaggio > visti.equipaggio_prima && !visti.arrivo && sim.tick > 0 {
            visti.arrivo = true;
            evento = Some(EventoCommento::PrimoArrivo);
        } else if sim.ossigeno <= SOGLIA_O2_CRITICO
            && !visti.ossigeno
            && sim.equipaggio > 0
            && sim.tick > 0
        {
            visti.ossigeno = true;
            evento = Some(EventoCommento::OssigenoCritico);
        } else if station.ostacoli.len() < visti.ostacoli_prima {
            // gru o sonda: un detrito in meno (le battute non nominano
            // l'attrezzo apposta)
            evento = Some(EventoCommento::DetritoRimosso);
        } else if let Some(tetto) = sim.tetto_tick
            && !visti.tetto
            && sim.tick > 0
            && (tetto.saturating_sub(sim.tick)) * 4 <= tetto
        {
            visti.tetto = true;
            evento = Some(EventoCommento::TettoVicino);
        }
    }
    visti.equipaggio_prima = sim.equipaggio;
    visti.ostacoli_prima = station.ostacoli.len();

    // l'oro si commenta anche se nel frattempo si è già sulla schermata
    // di vittoria: è lì che il giocatore lo sta guardando
    if medaglia.is_changed()
        && !medaglia.is_added()
        && matches!(medaglia.0, Some((m, _)) if m == ORO)
    {
        evento = Some(EventoCommento::OroPreso);
    }

    let Some(evento) = evento else {
        return;
    };
    let (personaggio, battuta) = commento(evento, visti.variante);
    visti.variante += 1;

    for e in &in_corso {
        commands.entity(e).despawn();
    }
    spawn_commento(&mut commands, &art, personaggio, battuta);
}

/// Spegne i commenti scaduti, e tutti quanti fuori dalle schermate in cui
/// hanno senso (partita e vittoria).
pub fn scadenza(
    mut commands: Commands,
    tempo: Res<Time>,
    stato: Res<State<AppState>>,
    mut q: Query<(Entity, &mut Commento)>,
) {
    let ammesso = matches!(
        *stato.get(),
        AppState::InGioco | AppState::LivelloCompletato
    );
    for (e, mut c) in &mut q {
        c.timer.tick(tempo.delta());
        if c.timer.is_finished() || !ammesso {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_commento(commands: &mut Commands, art: &Art, personaggio: usize, battuta: &str) {
    let chi = &PERSONAGGI[personaggio];
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                // sopra il registro eventi (che occupa il 18% in basso)
                bottom: Val::Percent(20.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                column_gap: Val::Px(8.0),
                max_width: Val::Px(340.0),
                ..default()
            },
            GlobalZIndex(12),
            Commento {
                timer: Timer::from_seconds(DURATA, TimerMode::Once),
            },
        ))
        .with_children(|riga| {
            riga.spawn((
                ImageNode::new(art.ritratti[chi.ritratto].clone()),
                Node {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            riga.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn(testo(chi.nome, 11.0, CIANO));
                col.spawn((
                    Node {
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                        max_width: Val::Px(280.0),
                        ..default()
                    },
                    BorderColor::all(GRIGIO_SCAFO),
                    BackgroundColor(SCAFO_SCURO),
                ))
                .with_children(|balloon| {
                    balloon.spawn(testo(format!("\u{201C}{battuta}\u{201D}"), 12.0, BIANCO));
                });
            });
        });
}
