//! Particelle ambientali della griglia: fumo dai moduli in avaria,
//! scintille dai reattori, bollicine da life support e serre. Solo
//! atmosfera: nessun effetto di gioco, cap globale severo, tutto sparisce
//! fuori dalla partita.
//!
//! Le particelle sono quadratini `Sprite::from_color` in world-space a
//! z = 0.8 (sopra lo sfondo stellato, sotto i moduli a z = 1): niente
//! sprite nuovi, niente asset. Il moto è deterministico dal tempo
//! (integrazione + deriva sinusoidale con fase per-particella); il caso
//! decide solo nascita e fase.

use crate::menu::AppState;
use crate::modules::ModuleKind;
use crate::sim::Module;
use crate::ui::{ARANCIO, BIANCO, CIANO, GIALLO, GRIGIO_MEDIO, GRIGIO_SCAFO};
use bevy::prelude::*;
use rand::RngExt;

/// Oltre questo numero l'emissione salta il giro: il fumo di dieci moduli
/// in avaria non deve diventare una nevicata.
const CAP: usize = 120;

/// Cadenze di emissione, in secondi.
const OGNI_FUMO: f32 = 0.5;
const OGNI_SCINTILLA: f32 = 1.5;
const OGNI_BOLLA: f32 = 2.5;

#[derive(Component)]
pub struct Particella {
    velocita: Vec2,
    vita: f32,
    vita_max: f32,
    /// Ampiezza della deriva laterale (px/s); 0 = moto rettilineo.
    oscillazione: f32,
    fase: f32,
    colore: Color,
}

/// Accumulatori dei tre emettitori.
#[derive(Default)]
pub struct Orologi {
    fumo: f32,
    scintille: f32,
    bolle: f32,
}

fn spawna(
    commands: &mut Commands,
    origine: Vec3,
    lato: f32,
    colore: Color,
    velocita: Vec2,
    vita: f32,
    oscillazione: f32,
) {
    let fase = rand::rng().random_range(0.0..std::f32::consts::TAU);
    commands.spawn((
        Sprite::from_color(colore, Vec2::splat(lato)),
        Transform::from_translation(origine.with_z(0.8)),
        Particella {
            velocita,
            vita,
            vita_max: vita,
            oscillazione,
            fase,
            colore,
        },
    ));
}

/// Emette dalle sorgenti attive, alle rispettive cadenze, fino al cap.
pub fn emetti(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    tempo: Res<Time>,
    mut orologi: Local<Orologi>,
    moduli: Query<(&Module, &Transform)>,
    esistenti: Query<(), With<Particella>>,
) {
    if *stato.get() != AppState::InGioco {
        return;
    }
    let dt = tempo.delta_secs();
    orologi.fumo += dt;
    orologi.scintille += dt;
    orologi.bolle += dt;

    let mut posti = CAP.saturating_sub(esistenti.iter().count());
    let mut rng = rand::rng();

    if orologi.fumo >= OGNI_FUMO {
        orologi.fumo = 0.0;
        for (m, tf) in &moduli {
            if !m.broken || posti == 0 {
                continue;
            }
            for _ in 0..rng.random_range(1..=2usize) {
                if posti == 0 {
                    break;
                }
                let colore = if rng.random_range(0..2) == 0 {
                    GRIGIO_MEDIO
                } else {
                    GRIGIO_SCAFO
                };
                let scarto = Vec3::new(rng.random_range(-8.0..8.0), 6.0, 0.0);
                spawna(
                    &mut commands,
                    tf.translation + scarto,
                    rng.random_range(3.0..5.0),
                    colore,
                    Vec2::new(0.0, rng.random_range(14.0..24.0)),
                    rng.random_range(1.6..2.4),
                    rng.random_range(6.0..14.0),
                );
                posti -= 1;
            }
        }
    }

    if orologi.scintille >= OGNI_SCINTILLA {
        orologi.scintille = 0.0;
        for (m, tf) in &moduli {
            if m.kind != ModuleKind::Reattore || !m.attivo() || posti == 0 {
                continue;
            }
            let colore = if rng.random_range(0..2) == 0 { GIALLO } else { ARANCIO };
            spawna(
                &mut commands,
                tf.translation + Vec3::new(rng.random_range(-10.0..10.0), 4.0, 0.0),
                3.0,
                colore,
                // saltella corto e ricade: la gravità la mette `muovi`
                Vec2::new(rng.random_range(-22.0..22.0), rng.random_range(28.0..44.0)),
                0.6,
                0.0,
            );
            posti -= 1;
        }
    }

    if orologi.bolle >= OGNI_BOLLA {
        orologi.bolle = 0.0;
        for (m, tf) in &moduli {
            let respira =
                matches!(m.kind, ModuleKind::LifeSupport | ModuleKind::Serra) && m.attivo();
            // rade: non tutte le sorgenti a ogni giro
            if !respira || posti == 0 || rng.random_range(0..3) != 0 {
                continue;
            }
            let colore = if rng.random_range(0..3) == 0 { BIANCO } else { CIANO };
            spawna(
                &mut commands,
                tf.translation + Vec3::new(rng.random_range(-10.0..10.0), 8.0, 0.0),
                3.0,
                colore,
                Vec2::new(0.0, rng.random_range(10.0..16.0)),
                rng.random_range(1.8..2.6),
                rng.random_range(3.0..7.0),
            );
            posti -= 1;
        }
    }
}

/// Integra, dissolve e ripulisce; le scintille (oscillazione 0) sentono la
/// gravità. Fuori dalla partita smonta tutto.
pub fn muovi(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    tempo: Res<Time>,
    mut particelle: Query<(Entity, &mut Particella, &mut Transform, &mut Sprite)>,
) {
    let in_gioco = *stato.get() == AppState::InGioco;
    let dt = tempo.delta_secs();
    let t = tempo.elapsed_secs();
    for (e, mut p, mut tf, mut sprite) in &mut particelle {
        if !in_gioco {
            commands.entity(e).despawn();
            continue;
        }
        p.vita -= dt;
        if p.vita <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        if p.oscillazione == 0.0 {
            // scintilla: parabola corta
            p.velocita.y -= 160.0 * dt;
        }
        tf.translation.x += (p.velocita.x + (t * 3.0 + p.fase).cos() * p.oscillazione) * dt;
        tf.translation.y += p.velocita.y * dt;
        sprite.color = p.colore.with_alpha(p.vita / p.vita_max);
    }
}
