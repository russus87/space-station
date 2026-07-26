//! Effetti sonori: caricamento e riproduzione. I WAV sono generati da
//! `tools/gen_audio.py` (stdlib pura, stessa filosofia degli sprite).
//!
//! Regola d'oro: un suono racconta un EVENTO, mai uno stato — niente loop
//! d'allarme continuo, suona il passaggio di stato e basta. Il log è già la
//! cronaca: qui si sonorizza la sua gravità (Avviso/Allarme), più gli
//! eventi "fisici" (costruzione, arrivi, acquisti) e le transizioni di
//! schermata (vittoria, sblocco, sconfitta).

use crate::impostazioni::Impostazioni;
use crate::livelli::Modalita;
use crate::personaggi::annuncio_sblocco;
use crate::sim::{EventLog, Gravita, Sim};
use bevy::audio::Volume;
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
    /// Fanfara dedicata alla medaglia d'oro appena conquistata.
    pub oro: Handle<AudioSource>,
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
        oro: assets.load("audio/oro.wav"),
        sconfitta: assets.load("audio/sconfitta.wav"),
    });
}

/// Riproduce un suono una volta al volume dato (0..=1); l'entità si
/// smonta da sola a fine clip. A volume zero non spawna proprio.
pub fn suona(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
    if volume <= 0.0 {
        return;
    }
    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings {
            volume: Volume::Linear(volume),
            ..PlaybackSettings::DESPAWN
        },
    ));
}

/// Sonorizza le righe nuove del log per gravità: al massimo UN suono per
/// frame (il più grave), altrimenti una cascata diventa una sirena.
pub fn suona_log(
    mut commands: Commands,
    log: Res<EventLog>,
    suoni: Res<Suoni>,
    imp: Res<Impostazioni>,
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
        Some(Gravita::Allarme) => suona(&mut commands, &suoni.allarme, imp.effetti_lineare()),
        Some(Gravita::Avviso) => suona(&mut commands, &suoni.avviso, imp.effetti_lineare()),
        _ => {}
    }
}

/// Trillo gentile quando l'equipaggio cresce (arrivi e trasporti coloni).
pub fn suona_arrivi(
    mut commands: Commands,
    sim: Res<Sim>,
    suoni: Res<Suoni>,
    imp: Res<Impostazioni>,
    mut prima: Local<u32>,
) {
    if sim.equipaggio > *prima && sim.tick > 0 {
        suona(&mut commands, &suoni.arrivo, imp.effetti_lineare());
    }
    *prima = sim.equipaggio;
}

/// Marca un bottone che NON deve suonare al click: azioni negate (slot
/// palette bloccati, card non acquistabili, scorte non usabili). Chi
/// gestisce lo stato del bottone inserisce/rimuove questo componente.
#[derive(Component)]
pub struct BottoneMuto;

/// Click su qualunque bottone della UI (voci di menu, palette, mercato),
/// tranne i bottoni marcati muti: un'azione negata non dà feedback
/// positivo.
#[allow(clippy::type_complexity)] // filtro di query Bevy, non un tipo da nominare
pub fn suona_click(
    mut commands: Commands,
    suoni: Res<Suoni>,
    imp: Res<Impostazioni>,
    q: Query<&Interaction, (Changed<Interaction>, With<Button>, Without<BottoneMuto>)>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed) {
        suona(&mut commands, &suoni.click, imp.effetti_lineare());
    }
}

/// Livello completato, in ordine di rarità dell'evento: la fanfara di
/// sblocco se il traguardo consegna un modulo, quella dell'oro se la run
/// ha preso la medaglia d'oro, la vittoria normale altrimenti.
pub fn suona_completato(
    mut commands: Commands,
    suoni: Res<Suoni>,
    modalita: Res<Modalita>,
    medaglia: Res<crate::livelli::UltimaMedaglia>,
    imp: Res<Impostazioni>,
) {
    let sblocco = matches!(*modalita, Modalita::Campagna(i) if annuncio_sblocco(i + 1).is_some());
    let oro = matches!(medaglia.0, Some((presa, _)) if presa == crate::progressi::ORO);
    let clip = if sblocco {
        &suoni.sblocco
    } else if oro {
        &suoni.oro
    } else {
        &suoni.vittoria
    };
    suona(&mut commands, clip, imp.effetti_lineare());
}

pub fn suona_sconfitta(mut commands: Commands, suoni: Res<Suoni>, imp: Res<Impostazioni>) {
    suona(&mut commands, &suoni.sconfitta, imp.effetti_lineare());
}
