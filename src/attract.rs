//! Il titolo vivo (attract mode, versione ambientale — scelta deliberata:
//! nessuna simulazione che gira, solo atmosfera): dietro la UI del titolo
//! passano lentamente il pianeta con l'anello, qualche meteora e un cielo
//! di stelle che pulsano. Tutto in world-space, montato entrando in
//! `Titolo` e smontato uscendone.
//!
//! Perché si veda, lo sfondo della schermata del titolo deve essere
//! leggermente trasparente (vedi nota in fondo al file): la UI sta sempre
//! sopra il mondo 2D, qualunque z.

use crate::menu::AppState;
use crate::ui::{BIANCO, METALLO};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::RngExt;

/// Sprite riusati dagli imprevisti: qui viaggiano per bellezza.
#[derive(Resource)]
pub struct ArteAttract {
    pianeta: Handle<Image>,
    meteora: Handle<Image>,
}

pub fn carica_arte(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(ArteAttract {
        pianeta: assets.load("sprites/imprevisti/pianeta.png"),
        meteora: assets.load("sprites/imprevisti/meteora.png"),
    });
}

#[derive(Component)]
pub struct ElementoAttract;

/// Marker del solo pianeta: uno in volo alla volta.
#[derive(Component)]
pub struct Pianeta;

/// Corpo in transito rettilineo; despawn quando esce di ~margine.
#[derive(Component)]
pub struct Transito {
    velocita: Vec2,
}

/// Stella che respira d'alpha con la sua fase.
#[derive(Component)]
pub struct Stella {
    fase: f32,
    colore: Color,
}

/// Conti alla rovescia per le prossime nascite (pianeta, meteora).
#[derive(Default)]
pub struct Attese {
    pianeta: f32,
    meteora: f32,
    cielo_pronto: bool,
}

/// Un unico sistema governa tutto: monta il cielo entrando nel titolo,
/// muove pianeta/meteore/stelle finché ci si resta, smonta uscendo.
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // sistema Bevy: query e risorse, non un tipo da nominare
pub fn anima(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    tempo: Res<Time>,
    arte: Option<Res<ArteAttract>>,
    finestre: Query<&Window, With<PrimaryWindow>>,
    mut attese: Local<Attese>,
    mut elementi: Query<
        (
            Entity,
            &mut Transform,
            Option<&Transito>,
            Option<&Stella>,
            &mut Sprite,
        ),
        With<ElementoAttract>,
    >,
    pianeti: Query<(), (With<ElementoAttract>, With<Pianeta>)>,
    meteore: Query<(), (With<Transito>, Without<Pianeta>)>,
) {
    let al_titolo = *stato.get() == AppState::Titolo;
    if !al_titolo {
        if attese.cielo_pronto {
            for (e, ..) in &elementi {
                commands.entity(e).despawn();
            }
            attese.cielo_pronto = false;
        }
        return;
    }
    let Some(arte) = arte else { return };
    let Ok(finestra) = finestre.single() else {
        return;
    };
    let (w, h) = (finestra.width(), finestra.height());
    let mut rng = rand::rng();

    // primo ingresso: le stelle, subito; pianeta e meteore con attesa breve
    if !attese.cielo_pronto {
        attese.cielo_pronto = true;
        attese.pianeta = rng.random_range(2.0..6.0);
        attese.meteora = rng.random_range(3.0..9.0);
        for _ in 0..10 {
            let colore = if rng.random_range(0..3) == 0 { BIANCO } else { METALLO };
            commands.spawn((
                Sprite::from_color(colore, Vec2::splat(rng.random_range(2.0..3.5))),
                Transform::from_xyz(
                    rng.random_range(-w / 2.0..w / 2.0),
                    rng.random_range(-h / 2.0..h / 2.0),
                    0.2,
                ),
                ElementoAttract,
                Stella {
                    fase: rng.random_range(0.0..std::f32::consts::TAU),
                    colore,
                },
            ));
        }
    }

    let dt = tempo.delta_secs();
    let t = tempo.elapsed_secs();

    // nascite: un pianeta alla volta, meteore fino a 3 in volo
    attese.pianeta -= dt;
    attese.meteora -= dt;
    if attese.pianeta <= 0.0 && pianeti.is_empty() {
        attese.pianeta = rng.random_range(60.0..120.0);
        let da_sinistra = rng.random_range(0..2) == 0;
        let y = rng.random_range(-h * 0.35..h * 0.35);
        let vx = w / 40.0 * if da_sinistra { 1.0 } else { -1.0 };
        commands.spawn((
            Sprite {
                image: arte.pianeta.clone(),
                custom_size: Some(Vec2::splat(120.0)),
                ..default()
            },
            Transform::from_xyz(if da_sinistra { -w / 2.0 - 80.0 } else { w / 2.0 + 80.0 }, y, 0.3),
            ElementoAttract,
            Pianeta,
            Transito {
                velocita: Vec2::new(vx, 0.0),
            },
        ));
    }
    if attese.meteora <= 0.0 && meteore.iter().count() < 3 {
        attese.meteora = rng.random_range(8.0..20.0);
        let x = rng.random_range(-w / 2.0..w / 2.0);
        commands.spawn((
            Sprite {
                image: arte.meteora.clone(),
                custom_size: Some(Vec2::splat(28.0)),
                ..default()
            },
            Transform::from_xyz(x, h / 2.0 + 40.0, 0.4),
            ElementoAttract,
            Transito {
                velocita: Vec2::new(rng.random_range(-160.0..160.0), -rng.random_range(300.0..450.0)),
            },
        ));
    }

    // moto e respiro
    let margine = 140.0;
    for (e, mut tf, transito, stella, mut sprite) in &mut elementi {
        if let Some(tr) = transito {
            tf.translation.x += tr.velocita.x * dt;
            tf.translation.y += tr.velocita.y * dt;
            if tf.translation.x.abs() > w / 2.0 + margine
                || tf.translation.y.abs() > h / 2.0 + margine
            {
                commands.entity(e).despawn();
            }
        }
        if let Some(s) = stella {
            let respiro = 0.30 + 0.55 * (0.5 + 0.5 * (t * 0.8 + s.fase).sin());
            sprite.color = s.colore.with_alpha(respiro);
        }
    }
}

// NOTA per il cablaggio (main.rs, a carico del parent):
// - Startup: `attract::carica_arte`
// - Update (sempre attivo): `attract::anima`
// e in menu.rs, `entra_titolo`, lo sfondo deve lasciar trasparire il cielo:
//     BackgroundColor(NERO.with_alpha(0.82)),
// al posto di `BackgroundColor(NERO),` sulla radice della schermata titolo.
