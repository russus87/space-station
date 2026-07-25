//! Prologo del livello: l'overlay a fumetto grande che offusca la griglia
//! all'avvio di una partita di campagna o casuale.
//!
//! Due pagine: la prima è il personaggio in grande con la sua battuta in
//! una nuvola di fumetto vera (carta bianca, coda a pallini); la seconda è
//! l'obiettivo del livello con "Indietro" e "Gioca!". Il briefing testuale
//! pre-partita resta dov'è: questo è il sipario, non il libretto.
//!
//! Lo accende `applica_reset` (`prologo.pagina = Some(0)`) solo per
//! Campagna e Casuale; finché è acceso, `attivo` fa da run condition per
//! bloccare gli input di costruzione in `main.rs`. Il menu di pausa (Esc)
//! passa sopra: l'overlay si nasconde e ricompare alla chiusura.

use crate::Art;
use crate::livelli::{LIVELLI, LivelloCasuale, LivelloDef, Modalita};
use crate::menu::{AppState, Pausa};
use crate::personaggi::{PERSONAGGI, battuta_briefing};
use crate::ui::{BIANCO, CIANO, GIALLO, GRIGIO_MEDIO, GRIGIO_SCAFO, METALLO, NERO, VERDE};
use bevy::prelude::*;

/// La battuta di servizio quando il livello non ne ha una propria (livello
/// casuale; in campagna la copertura è totale). Parla Vera: è lei che apre
/// la campagna, tanto vale che apra anche l'ignoto.
const FRASE_DI_SERVIZIO: (usize, &str) =
    (0, "Settore sconosciuto. Regole solite: aria, corrente, margine.");

/// Pagina corrente del prologo: `None` = spento, `Some(0)` = personaggio,
/// `Some(1)` = obiettivo. Attivato da `applica_reset`.
#[derive(Resource, Default)]
pub struct Prologo {
    pub pagina: Option<u8>,
}

/// Run condition: finché il prologo è a schermo, niente input di gioco.
pub fn attivo(p: Res<Prologo>) -> bool {
    p.pagina.is_some()
}

#[derive(Component)]
pub struct SchermataPrologo;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Azione {
    Avanti,
    Indietro,
    Gioca,
}

/// Un pulsante del prologo: bottoni propri, non le `Voce` del menu (qui
/// non c'è `Selezione` da tastiera, si clicca).
#[derive(Component)]
pub struct BottonePrologo {
    azione: Azione,
    /// Colore del bordo quando il cursore ci passa sopra.
    evidenzia: Color,
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

fn bottone(
    p: &mut ChildSpawnerCommands,
    azione: Azione,
    etichetta: &str,
    evidenzia: Color,
    px: f32,
) {
    p.spawn((
        Node {
            padding: UiRect::axes(Val::Px(22.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor::all(GRIGIO_SCAFO),
        Button,
        BottonePrologo { azione, evidenzia },
    ))
    .with_children(|c| {
        c.spawn(testo(etichetta, px, evidenzia));
    });
}

/// Il livello in corso e la sua battuta, secondo la modalità.
fn livello_corrente<'a>(
    modalita: &Modalita,
    casuale: &'a LivelloCasuale,
) -> Option<(&'a LivelloDef, Option<usize>, (usize, &'a str))> {
    match *modalita {
        Modalita::Campagna(i) => {
            let battuta = battuta_briefing(i + 1).unwrap_or(FRASE_DI_SERVIZIO);
            Some((&LIVELLI[i], Some(i), battuta))
        }
        Modalita::Casuale => casuale.0.as_ref().map(|l| (l, None, FRASE_DI_SERVIZIO)),
        Modalita::Infinita | Modalita::Sfida => None,
    }
}

/// L'overlay esiste solo a prologo acceso, in partita e senza pausa sopra;
/// si ricostruisce a ogni cambio pagina.
#[allow(clippy::too_many_arguments)]
pub fn sincronizza(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    pausa: Res<Pausa>,
    prologo: Res<Prologo>,
    modalita: Res<Modalita>,
    casuale: Res<LivelloCasuale>,
    art: Res<Art>,
    q: Query<Entity, With<SchermataPrologo>>,
) {
    let dati = livello_corrente(&modalita, &casuale);
    let deve_esserci = prologo.pagina.is_some()
        && *stato.get() == AppState::InGioco
        && !pausa.aperta
        && dati.is_some();
    let c_e = !q.is_empty();
    if deve_esserci == c_e && !(deve_esserci && prologo.is_changed()) {
        return;
    }
    for e in &q {
        commands.entity(e).despawn();
    }
    if !deve_esserci {
        return;
    }
    let (livello, indice, (chi, battuta)) = dati.unwrap();
    let pagina = prologo.pagina.unwrap_or(0);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },
            // l'offuscamento: la griglia si intravede appena sotto
            BackgroundColor(NERO.with_alpha(0.88)),
            GlobalZIndex(18),
            SchermataPrologo,
        ))
        .with_children(|r| {
            if pagina == 0 {
                pagina_personaggio(r, &art, chi, battuta);
            } else {
                pagina_obiettivo(r, livello, indice);
            }
        });
}

/// Pagina 0: la nuvola di fumetto sopra, la coda a pallini, il ritratto
/// grande sotto — il classico balloon dei fumetti, in carta bianca.
fn pagina_personaggio(r: &mut ChildSpawnerCommands, art: &Art, chi: usize, battuta: &str) {
    let personaggio = &PERSONAGGI[chi];
    // nuvola
    r.spawn((
        Node {
            max_width: Val::Px(560.0),
            padding: UiRect::axes(Val::Px(26.0), Val::Px(18.0)),
            border_radius: BorderRadius::all(Val::Px(28.0)),
            ..default()
        },
        BackgroundColor(BIANCO),
    ))
    .with_children(|c| {
        c.spawn(testo(battuta, 18.0, NERO));
    });
    // coda a pallini, sfalsata verso il ritratto
    for (lato, scarto) in [(14.0, 30.0), (9.0, 12.0), (5.0, 0.0)] {
        r.spawn((
            Node {
                width: Val::Px(lato),
                height: Val::Px(lato),
                margin: UiRect::left(Val::Px(scarto)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(BIANCO),
        ));
    }
    // ritratto grande (il nearest-neighbor del plugin lo tiene pixelato)
    r.spawn((
        ImageNode::new(art.ritratti[personaggio.ritratto].clone()),
        Node {
            width: Val::Px(160.0),
            height: Val::Px(160.0),
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        },
    ));
    r.spawn(testo(
        format!("{} — {}", personaggio.nome, personaggio.ruolo),
        15.0,
        CIANO,
    ));
    r.spawn(Node {
        height: Val::Px(18.0),
        ..default()
    });
    bottone(r, Azione::Avanti, "Avanti →", BIANCO, 17.0);
}

/// Pagina 1: l'obiettivo nero su bianco (anzi: giallo su nero) e la scelta
/// fra ripensarci e giocare.
fn pagina_obiettivo(r: &mut ChildSpawnerCommands, livello: &LivelloDef, indice: Option<usize>) {
    let eyebrow = match indice {
        Some(i) => format!("LIVELLO {}", i + 1),
        None => "LIVELLO CASUALE".to_string(),
    };
    r.spawn(testo(eyebrow, 14.0, GRIGIO_MEDIO));
    r.spawn((Node {
        margin: UiRect::bottom(Val::Px(14.0)),
        ..default()
    },))
    .with_children(|c| {
        c.spawn(testo(livello.nome.clone(), 30.0, BIANCO));
    });
    r.spawn((Node {
        max_width: Val::Px(560.0),
        ..default()
    },))
    .with_children(|c| {
        c.spawn(testo(
            format!("Obiettivo: {}", livello.obiettivo.descrizione()),
            18.0,
            GIALLO,
        ));
    });
    r.spawn(testo(
        format!("Moduli disponibili: {} (corridoi inclusi)", livello.max_moduli),
        15.0,
        CIANO,
    ));
    if !livello.ostacoli.is_empty() {
        r.spawn(testo(
            "Detriti sulla griglia: costruisci intorno",
            14.0,
            METALLO,
        ));
    }
    r.spawn(Node {
        height: Val::Px(18.0),
        ..default()
    });
    r.spawn(Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(16.0),
        ..default()
    })
    .with_children(|riga| {
        bottone(riga, Azione::Indietro, "← Indietro", METALLO, 16.0);
        bottone(riga, Azione::Gioca, "Gioca!", VERDE, 20.0);
    });
}

/// Click e hover dei tre pulsanti. Il suono del click lo dà già il sistema
/// globale (`audio::suona_click` ascolta ogni `Button`).
pub fn click(
    mut prologo: ResMut<Prologo>,
    mut q: Query<(&Interaction, &BottonePrologo, &mut BorderColor), Changed<Interaction>>,
) {
    for (interazione, b, mut bordo) in &mut q {
        match interazione {
            Interaction::Pressed => {
                prologo.pagina = match b.azione {
                    Azione::Avanti => Some(1),
                    Azione::Indietro => Some(0),
                    Azione::Gioca => None,
                };
            }
            Interaction::Hovered => *bordo = BorderColor::all(b.evidenzia),
            Interaction::None => *bordo = BorderColor::all(GRIGIO_SCAFO),
        }
    }
}
