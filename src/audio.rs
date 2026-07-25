//! Effetti sonori: caricamento e riproduzione. I WAV sono generati da
//! `tools/gen_audio.py` (stdlib pura, stessa filosofia degli sprite).
//!
//! Regola d'oro: un suono racconta un EVENTO, mai uno stato — niente loop
//! d'allarme continuo, suona il passaggio di stato e basta. Il log è già la
//! cronaca: qui si sonorizza la sua gravità (Avviso/Allarme), più gli
//! eventi "fisici" (costruzione, arrivi, acquisti) e le transizioni di
//! schermata (vittoria, sblocco, sconfitta).

use crate::livelli::Modalita;
use crate::personaggi::annuncio_sblocco;
use crate::sim::{EventLog, Gravita, Sim};
use bevy::prelude::*;

#[derive(Resource)]
pub struct Suoni {
    pub click: Handle<AudioSource>,
    pub costruzione: Handle<AudioSource>,
    pub rimozione: Handle<AudioSource>,
    pub avviso: Handle<AudioSource>,
    pub allarme: Handle<AudioSource>,
    pub arrivo: Handle<AudioSource>,
    pub acquisto: Handle<AudioSource>,
    pub sblocco: Handle<AudioSource>,
    pub vittoria: Handle<AudioSource>,
    pub sconfitta: Handle<AudioSource>,
}

pub fn carica_suoni(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Suoni {
        click: assets.load("audio/click.wav"),
        costruzione: assets.load("audio/costruzione.wav"),
        rimozione: assets.load("audio/rimozione.wav"),
        avviso: assets.load("audio/avviso.wav"),
        allarme: assets.load("audio/allarme.wav"),
        arrivo: assets.load("audio/arrivo.wav"),
        acquisto: assets.load("audio/acquisto.wav"),
        sblocco: assets.load("audio/sblocco.wav"),
        vittoria: assets.load("audio/vittoria.wav"),
        sconfitta: assets.load("audio/sconfitta.wav"),
    });
}

/// Riproduce un suono una volta (l'entità si smonta da sola a fine clip).
pub fn suona(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(handle.clone()), PlaybackSettings::DESPAWN));
}

/// Sonorizza le righe nuove del log per gravità: al massimo UN suono per
/// frame (il più grave), altrimenti una cascata diventa una sirena.
pub fn suona_log(
    mut commands: Commands,
    log: Res<EventLog>,
    suoni: Res<Suoni>,
    mut viste: Local<usize>,
) {
    let righe = log.ultimi(60);
    let totale = log.totale();
    let nuove = totale.saturating_sub(*viste);
    *viste = totale;
    if nuove == 0 {
        return;
    }
    let peggiore = righe
        .iter()
        .rev()
        .take(nuove)
        .map(|r| r.gravita)
        .max_by_key(|g| match g {
            Gravita::Info => 0,
            Gravita::Avviso => 1,
            Gravita::Allarme => 2,
        });
    match peggiore {
        Some(Gravita::Allarme) => suona(&mut commands, &suoni.allarme),
        Some(Gravita::Avviso) => suona(&mut commands, &suoni.avviso),
        _ => {}
    }
}

/// Trillo gentile quando l'equipaggio cresce (arrivi e trasporti coloni).
pub fn suona_arrivi(
    mut commands: Commands,
    sim: Res<Sim>,
    suoni: Res<Suoni>,
    mut prima: Local<u32>,
) {
    if sim.equipaggio > *prima && sim.tick > 0 {
        suona(&mut commands, &suoni.arrivo);
    }
    *prima = sim.equipaggio;
}

/// Click su qualunque bottone della UI (voci di menu, palette, mercato).
pub fn suona_click(
    mut commands: Commands,
    suoni: Res<Suoni>,
    q: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed) {
        suona(&mut commands, &suoni.click);
    }
}

/// Livello completato: fanfara di sblocco se questo traguardo sblocca un
/// modulo, fanfara normale altrimenti.
pub fn suona_completato(
    mut commands: Commands,
    suoni: Res<Suoni>,
    modalita: Res<Modalita>,
) {
    let sblocco = matches!(*modalita, Modalita::Campagna(i) if annuncio_sblocco(i + 1).is_some());
    if sblocco {
        suona(&mut commands, &suoni.sblocco);
    } else {
        suona(&mut commands, &suoni.vittoria);
    }
}

pub fn suona_sconfitta(mut commands: Commands, suoni: Res<Suoni>) {
    suona(&mut commands, &suoni.sconfitta);
}
