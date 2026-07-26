//! Colonna sonora: una traccia per il menu e sei tracce di gioco che
//! seguono la storia (tools/gen_musica.py per la composizione).
//!
//! In campagna la traccia è quella del blocco narrativo del livello; in
//! Infinita/Sfida/Casuale se ne pesca una a caso a inizio partita. Il
//! cambio avviene solo quando la traccia desiderata è diversa da quella
//! in corso: navigare tra i menu non riavvia la musica del titolo.

use crate::impostazioni::Impostazioni;
use crate::livelli::Modalita;
use crate::menu::AppState;
use bevy::audio::Volume;
use bevy::prelude::*;
use rand::RngExt;

/// Indice 0 = menu; 1..=6 = tracce di gioco in ordine di storia.
pub const TRACCE: [&str; 7] = [
    "musica/menu.ogg",
    "musica/cantiere.ogg",
    "musica/termica.ogg",
    "musica/reliquie.ogg",
    "musica/officina.ogg",
    "musica/vigilia.ogg",
    "musica/finale.ogg",
];

#[derive(Component)]
pub struct TagMusica;

/// La musica è momentaneamente sospesa (sirena degli imprevisti): la
/// traccia resta in pausa, non riparte da capo. La alza e la abbassa
/// `imprevisti.rs`.
#[derive(Resource, Default)]
pub struct MusicaSospesa(pub bool);

#[derive(Resource, Default)]
pub struct StatoMusica {
    /// Indice in `TRACCE` della traccia in riproduzione.
    corrente: Option<usize>,
    /// Traccia pescata per la partita sandbox in corso (1..=6).
    pub casuale: usize,
}

impl StatoMusica {
    /// Da chiamare a ogni nuova partita sandbox: pesca la traccia di gioco.
    pub fn pesca_casuale(&mut self) {
        self.casuale = rand::rng().random_range(1..TRACCE.len());
    }
}

/// La traccia del blocco narrativo del livello di campagna (0-based).
fn traccia_campagna(livello: usize) -> usize {
    match livello {
        0..=9 => 1,   // cantiere: si costruisce con speranza
        10..=19 => 2, // termica: la pressione sale
        20..=29 => 3, // reliquie: i detriti hanno un nome
        30..=39 => 4, // officina: la verità è detta, si lavora
        40..=44 => 5, // vigilia: la scelta di restare
        _ => 6,       // finale
    }
}

/// Decide e avvia la traccia giusta per lo stato corrente dell'app.
pub fn gestisci_musica(
    mut commands: Commands,
    assets: Res<AssetServer>,
    stato_app: Res<State<AppState>>,
    modalita: Res<Modalita>,
    imp: Res<Impostazioni>,
    mut stato: ResMut<StatoMusica>,
    in_corso: Query<Entity, With<TagMusica>>,
) {
    let desiderata = match *stato_app.get() {
        // la musica di gioco resta anche sotto pausa e schermate di esito:
        // la stazione è ancora lì sotto
        AppState::InGioco | AppState::FinePartita | AppState::LivelloCompletato => {
            match *modalita {
                Modalita::Campagna(i) => traccia_campagna(i),
                _ => stato.casuale.clamp(1, TRACCE.len() - 1),
            }
        }
        _ => 0,
    };
    if stato.corrente == Some(desiderata) {
        return;
    }
    for e in &in_corso {
        commands.entity(e).despawn();
    }
    commands.spawn((
        AudioPlayer::new(assets.load(TRACCE[desiderata])),
        PlaybackSettings {
            volume: Volume::Linear(imp.musica_lineare()),
            ..PlaybackSettings::LOOP
        },
        TagMusica,
    ));
    stato.corrente = Some(desiderata);
}

/// Applica in diretta i cambi di volume dal menu di pausa.
pub fn applica_volume(
    imp: Res<Impostazioni>,
    mut sink: Query<&mut AudioSink, With<TagMusica>>,
) {
    if !imp.is_changed() {
        return;
    }
    for mut s in &mut sink {
        s.set_volume(Volume::Linear(imp.musica_lineare()));
    }
}

/// Mette in pausa/riprende la traccia quando la sirena degli imprevisti
/// alza o abbassa `MusicaSospesa`; i sink nati mentre la sospensione è
/// attiva (cambio traccia sotto sirena) nascono già in pausa.
pub fn applica_sospensione(
    sospesa: Res<MusicaSospesa>,
    sink: Query<&AudioSink, With<TagMusica>>,
    nuovi: Query<&AudioSink, (With<TagMusica>, Added<AudioSink>)>,
) {
    if sospesa.is_changed() {
        for s in &sink {
            if sospesa.0 {
                s.pause();
            } else {
                s.play();
            }
        }
    } else if sospesa.0 {
        for s in &nuovi {
            s.pause();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ogni_livello_di_campagna_ha_una_traccia_di_gioco() {
        for livello in 0..50 {
            let t = traccia_campagna(livello);
            assert!((1..TRACCE.len()).contains(&t), "livello {livello}: traccia {t}");
        }
        // i blocchi estremi suonano le tracce estreme
        assert_eq!(traccia_campagna(0), 1);
        assert_eq!(traccia_campagna(49), 6);
    }
}
