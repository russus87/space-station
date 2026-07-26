//! Build a Space Station — bilancio di risorse con cascata di guasti.
//! La simulazione (`sim.rs`) è il cuore validato dalla PoC e non si tocca:
//! qui ci sono la griglia in world-space, il piazzamento dei moduli e il
//! cablaggio degli stati dell'app. La UI di gioco sta in `ui.rs`, le
//! schermate di menu in `menu.rs`.

mod audio;
mod generatore;
mod impostazioni;
mod livelli;
mod menu;
mod mercato;
mod musica;
mod personaggi;
mod progressi;
mod prologo;
mod modules;
mod sim;
mod ui;

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::{PrimaryWindow, Window, WindowPlugin};
use livelli::{LIVELLI, Modalita};
use menu::{AppState, Pausa};
use modules::{KINDS, ModuleKind};
use sim::{EventLog, Fermo, Module, Sim, TICK_LAVORO_GRU, TICK_MASSIMO};

const WIN_W: f32 = 1160.0;
const WIN_H: f32 = 800.0;
pub(crate) const GRID_W: i32 = 14;
pub(crate) const GRID_H: i32 = 8;
const ART: f32 = 32.0; // dimensione nativa dell'art di una cella

/// Handle degli sprite: caricati una volta, riusati da griglia, palette,
/// pannello ispezione e pagina della guida.
#[derive(Resource)]
pub struct Art {
    pub moduli: [Handle<Image>; 11],
    /// Ritratti dei personaggi per i fumetti dei briefing, nell'ordine di
    /// `personaggi::PERSONAGGI`.
    pub ritratti: [Handle<Image>; 5],
    pub badge_energia: Handle<Image>,
    pub badge_equipaggio: Handle<Image>,
    pub badge_avaria: Handle<Image>,
    pub icone: [Handle<Image>; 4], // energia, ossigeno, calore, equipaggio
    /// Varianti del corridoio per l'autotiling, indicizzate dalle costanti
    /// `CORRIDOIO_*`: orizzontale, verticale, curva, T, croce.
    pub corridoi: [Handle<Image>; 5],
    /// Tile 64×64 ripetibile dello sfondo stellato, disegnata a ×2 rispetto
    /// all'art di cella (32): a cella 64 px va 1:1, e in generale segue lo
    /// stesso fattore intero `s` della griglia (una tile per cella).
    pub sfondo_stelle: Handle<Image>,
    /// Detrito che occupa una cella nei livelli della campagna.
    pub ostacolo: Handle<Image>,
    /// Mirino di piazzamento: cursore alternativo mostrato in partita.
    pub mirino: Handle<Image>,
    /// Icone delle sei facility del Marketplace, nell'ordine di
    /// `mercato::FACILITIES` (card, HUD scorte, overlay scorte).
    pub facilities: [Handle<Image>; 6],
    /// Freccia pixel-art usata come cursore custom della finestra.
    pub cursore: Handle<Image>,
    /// Medaglie della schermata "livello completato": oro, argento, rame.
    pub medaglie: [Handle<Image>; 3],
    /// I 4 frame dello spin della moneta d'oro + la versione spenta.
    pub monete_accese: [Handle<Image>; 4],
    pub moneta_spenta: Handle<Image>,
}

// Indici in `Art::corridoi`. Gli sprite base sono orientati così:
// la curva collega destra e basso, la T collega sinistra, destra e basso.
const CORRIDOIO_H: usize = 0;
const CORRIDOIO_V: usize = 1;
const CORRIDOIO_CURVA: usize = 2;
const CORRIDOIO_T: usize = 3;
const CORRIDOIO_CROCE: usize = 4;

/// Modulo selezionato nella palette (tasti 1..6).
#[derive(Resource)]
pub struct Selected(pub ModuleKind);

/// Modulo sotto il cursore: lo legge il pannello ispezione.
#[derive(Resource, Default)]
pub struct SottoCursore(pub Option<Entity>);

/// Chiesto un azzeramento della partita (Gioca / Ricomincia / Torna al titolo).
#[derive(Resource, Default)]
pub struct RichiestaReset(pub bool);

/// Occupazione della griglia e contatori per battezzare i moduli ("Reattore 2").
#[derive(Resource, Default)]
pub(crate) struct Station {
    pub(crate) celle: HashMap<IVec2, Entity>,
    /// Celle occupate dai detriti del livello: non edificabili, non
    /// rimovibili. Vuoto in modalità Infinita.
    pub(crate) ostacoli: HashSet<IVec2>,
    /// Tetto di moduli costruibili (corridoi inclusi): `Some` solo in
    /// campagna, dal campo `max_moduli` del livello. Rimuovere un modulo
    /// libera il posto: conta quel che c'è, non quel che è stato costruito.
    pub(crate) max_moduli: Option<u32>,
    contatori: [u32; 11],
    seq: u32,
}

impl Station {
    /// C'è ancora spazio nel budget moduli del livello?
    fn sotto_limite(&self) -> bool {
        self.max_moduli
            .is_none_or(|max| (self.celle.len() as u32) < max)
    }
}

/// Scala e posizione della griglia, ricalcolate a ogni resize: la griglia sta
/// nell'area lasciata libera dai pannelli UI, con celle a multipli interi
/// dell'art (32 px) per non sfocare i pixel.
#[derive(Resource)]
struct Griglia {
    cella: f32,
    centro: Vec2,
}

impl Default for Griglia {
    fn default() -> Self {
        Self {
            cella: ART * 2.0,
            centro: Vec2::ZERO,
        }
    }
}

impl Griglia {
    fn da_finestra(w: f32, h: f32) -> Self {
        // stesse proporzioni dei pannelli in ui.rs
        let hud = (0.09 * h).max(52.0);
        let log = (0.18 * h).max(150.0);
        let col = (0.18 * w).max(190.0);
        let disp_w = (w - col).max(1.0);
        let disp_h = (h - hud - log).max(1.0);
        let s = (disp_w / (GRID_W as f32 * ART))
            .min(disp_h / (GRID_H as f32 * ART))
            .floor()
            .max(1.0);
        Self {
            cella: ART * s,
            centro: Vec2::new(col / 2.0, (log - hud) / 2.0),
        }
    }

    fn cella_in_mondo(&self, c: IVec2) -> Vec2 {
        let o = self.centro
            - Vec2::new(GRID_W as f32 * self.cella, GRID_H as f32 * self.cella) / 2.0;
        o + (c.as_vec2() + Vec2::splat(0.5)) * self.cella
    }

    fn mondo_in_cella(&self, p: Vec2) -> Option<IVec2> {
        let o = self.centro
            - Vec2::new(GRID_W as f32 * self.cella, GRID_H as f32 * self.cella) / 2.0;
        let d = (p - o) / self.cella;
        let c = IVec2::new(d.x.floor() as i32, d.y.floor() as i32);
        ((0..GRID_W).contains(&c.x) && (0..GRID_H).contains(&c.y)).then_some(c)
    }
}

#[derive(Component)]
struct Ghost;

/// Detrito su una cella (solo campagna). La simulazione non lo vede:
/// esiste come sprite e come vincolo di piazzamento in `Station::ostacoli`.
#[derive(Component)]
pub(crate) struct Ostacolo {
    pub(crate) cella: IVec2,
}

#[derive(Component)]
struct Overlay;

#[derive(Component)]
struct Badge;

#[derive(Component)]
struct Numero;

#[derive(Component)]
struct VisualeGriglia;

/// Marca tutto ciò che vive in world-space e va nascosto fuori dalla partita.
#[derive(Component)]
struct Scena;

/// Bevy cerca `assets/` accanto all'eseguibile e ripiega su `CARGO_MANIFEST_DIR`
/// solo sotto `cargo run`: lanciando il binario a mano gli sprite sparivano.
/// Qui si prova prima la cartella accanto all'eseguibile (build distribuita) e
/// poi quella del sorgente (sviluppo).
fn percorso_assets() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("assets")))
        .filter(|p| p.is_dir())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!("{}/assets", env!("CARGO_MANIFEST_DIR")))
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: percorso_assets(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Station".into(),
                        resolution: (WIN_W as u32, WIN_H as u32).into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 960.0,
                            min_height: 640.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                // i pixel dell'art vanno ingranditi a blocchi, non interpolati
                .set(ImagePlugin::default_nearest()),
        )
        .init_state::<AppState>()
        .insert_resource(ClearColor(ui::SCAFO_SCURO))
        .insert_resource(Selected(ModuleKind::Reattore))
        .init_resource::<Station>()
        .init_resource::<Griglia>()
        .init_resource::<SottoCursore>()
        .init_resource::<RichiestaReset>()
        .init_resource::<Pausa>()
        .init_resource::<menu::Origine>()
        .init_resource::<menu::Selezione>()
        .init_resource::<Modalita>()
        .init_resource::<livelli::StatoLivello>()
        .init_resource::<livelli::LivelloScelto>()
        .init_resource::<livelli::LivelloCasuale>()
        .init_resource::<mercato::Mercato>()
        .init_resource::<musica::StatoMusica>()
        .init_resource::<livelli::UltimaMedaglia>()
        .init_resource::<prologo::Prologo>()
        .insert_resource(progressi::carica())
        .insert_resource(impostazioni::carica())
        .init_resource::<livelli::UltimoPiazzamento>()
        // classifiche e progressione si leggono dal disco una volta
        // all'avvio; file assenti o rotti equivalgono a "nessun dato", senza
        // errori. Infinita e Sfida hanno file separati: i punteggi non sono
        // confrontabili fra le due modalità.
        .insert_resource(livelli::carica_classifica_infinita())
        .insert_resource(livelli::carica_classifica_sfida())
        .insert_resource(livelli::carica_progressione())
        .insert_resource(Sim::default())
        .insert_resource(EventLog::default())
        .add_systems(
            Startup,
            (font_principale, carica_art, audio::carica_suoni, (setup, ui::setup_ui)).chain(),
        )
        .add_systems(OnEnter(AppState::Marketplace), menu::entra_marketplace)
        .add_systems(OnExit(AppState::Marketplace), menu::esci_marketplace)
        .add_systems(OnEnter(AppState::LivelloCompletato), audio::suona_completato)
        .add_systems(OnEnter(AppState::FinePartita), audio::suona_sconfitta)
        .add_systems(OnEnter(AppState::Titolo), menu::entra_titolo)
        .add_systems(OnExit(AppState::Titolo), menu::esci_titolo)
        .add_systems(OnEnter(AppState::ComeSiGioca), menu::entra_guida)
        .add_systems(OnExit(AppState::ComeSiGioca), menu::esci_guida)
        .add_systems(OnEnter(AppState::SelezioneLivello), menu::entra_selezione)
        .add_systems(OnExit(AppState::SelezioneLivello), menu::esci_selezione)
        .add_systems(OnEnter(AppState::Briefing), menu::entra_briefing)
        .add_systems(OnExit(AppState::Briefing), menu::esci_briefing)
        .add_systems(OnEnter(AppState::Intermezzo), menu::entra_intermezzo)
        .add_systems(OnExit(AppState::Intermezzo), menu::esci_intermezzo)
        .add_systems(OnEnter(AppState::SchermataClassifica), menu::entra_classifica)
        .add_systems(OnExit(AppState::SchermataClassifica), menu::esci_classifica)
        .add_systems(OnEnter(AppState::LivelloCompletato), menu::entra_completato)
        .add_systems(OnExit(AppState::LivelloCompletato), menu::esci_completato)
        .add_systems(OnEnter(AppState::FinePartita), menu::entra_fine)
        .add_systems(OnExit(AppState::FinePartita), menu::esci_fine)
        .add_systems(
            Update,
            (
                aggiorna_layout,
                menu::naviga,
                menu::click_voci,
                menu::sincronizza_pausa,
                menu::evidenzia_voci,
                menu::colore_sfondo_menu,
                applica_reset,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                cursore_sopra,
                // col prologo aperto la stazione non si tocca: né tasti
                // (Spazio compreso) né click di costruzione né mercato
                input_tastiera.run_if(not(prologo::attivo)),
                input_mouse.run_if(not(prologo::attivo)),
                prologo::click,
                ui::click_palette,
                ui::click_bottone_menu,
                mercato::toggle_tasto.run_if(not(prologo::attivo)),
                mercato::click_bottone.run_if(not(prologo::attivo)),
                mercato::click_scorte,
                sim::sim_tick.run_if(sim_attiva),
                applica_gru,
                // in Infinita/Sfida il codice degli obiettivi non gira proprio
                livelli::controlla_obiettivo.run_if(livelli::obiettivi_attivi),
                controlla_fine,
            )
                .chain()
                .after(applica_reset)
                .run_if(in_state(AppState::InGioco)),
        )
        .add_systems(
            Update,
            (
                // scena e overlay
                (
                    aggiorna_ghost,
                    aggiorna_visuali,
                    orienta_corridoi,
                    mercato::sincronizza,
                    prologo::sincronizza,
                    visibilita_scena,
                ),
                // audio e musica
                (
                    audio::suona_log,
                    audio::suona_arrivi,
                    audio::suona_click,
                    musica::gestisci_musica,
                    musica::applica_volume,
                ),
                // etichette dinamiche, servizi e pannelli UI
                (
                    menu::aggiorna_voci_volume,
                    menu::aggiorna_voci_marketplace,
                    menu::anima_monete,
                    screenshot_tasto,
                    demo_foto,
                    cursore_pixel,
                ),
                (
                    ui::visibilita_gioco,
                    ui::update_hud,
                    ui::update_palette,
                    ui::update_scorte_hud,
                    ui::update_log,
                    ui::update_ispezione,
                ),
            )
                .after(applica_reset),
        )
        .run();
}

/// La simulazione avanza solo in partita e col menu chiuso: `Esc` congela
/// tutto, timer del tick compreso.
fn sim_attiva(pausa: Res<Pausa>) -> bool {
    !pausa.aperta
}

/// Il font di default di Bevy è un subset ASCII di Fira Mono: «·», «—»,
/// «→», «º» e le accentate diventavano quadrati. Qui si sostituisce
/// l'asset al suo stesso id con DejaVu Sans completo (licenza in
/// `assets/fonts/LICENSE-DejaVu.txt`), incorporato nel binario: nessun
/// file font da distribuire e nessun `TextFont` da cambiare, tutto il
/// testo del gioco lo usa automaticamente.
fn font_principale(mut fonts: ResMut<Assets<Font>>) {
    const DATI: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/DejaVuSans.ttf"
    ));
    let _ = fonts.insert(AssetId::default(), Font::from_bytes(DATI.to_vec()));
}

fn carica_art(mut commands: Commands, assets: Res<AssetServer>) {
    let moduli = KINDS.map(|k| assets.load(k.def().sprite));
    commands.insert_resource(Art {
        moduli,
        ritratti: [
            assets.load("sprites/ritratti/ingegnere.png"),
            assets.load("sprites/ritratti/medico.png"),
            assets.load("sprites/ritratti/caposquadra.png"),
            assets.load("sprites/ritratti/scienziata.png"),
            assets.load("sprites/ritratti/comandante.png"),
        ],
        badge_energia: assets.load("sprites/badge/energia.png"),
        badge_equipaggio: assets.load("sprites/badge/equipaggio.png"),
        badge_avaria: assets.load("sprites/badge/avaria.png"),
        icone: [
            assets.load("sprites/icone/energia.png"),
            assets.load("sprites/icone/ossigeno.png"),
            assets.load("sprites/icone/calore.png"),
            assets.load("sprites/icone/equipaggio.png"),
        ],
        corridoi: [
            assets.load("sprites/moduli/corridoio.png"),
            assets.load("sprites/moduli/corridoio_v.png"),
            assets.load("sprites/moduli/corridoio_curva.png"),
            assets.load("sprites/moduli/corridoio_t.png"),
            assets.load("sprites/moduli/corridoio_croce.png"),
        ],
        sfondo_stelle: assets.load("sprites/sfondo/stelle.png"),
        ostacolo: assets.load("sprites/ostacolo.png"),
        facilities: [
            assets.load("sprites/facilities/ossigeno.png"),
            assets.load("sprites/facilities/riparazione.png"),
            assets.load("sprites/facilities/coloni.png"),
            assets.load("sprites/facilities/stiva.png"),
            assets.load("sprites/facilities/spurgo.png"),
            assets.load("sprites/facilities/sonda.png"),
        ],
        cursore: assets.load("sprites/cursore.png"),
        mirino: assets.load("sprites/mirino.png"),
        medaglie: [
            assets.load("sprites/medaglie/oro.png"),
            assets.load("sprites/medaglie/argento.png"),
            assets.load("sprites/medaglie/rame.png"),
        ],
        monete_accese: [
            assets.load("sprites/monete/accesa_1.png"),
            assets.load("sprites/monete/accesa_2.png"),
            assets.load("sprites/monete/accesa_3.png"),
            assets.load("sprites/monete/accesa_4.png"),
        ],
        moneta_spenta: assets.load("sprites/monete/spenta.png"),
    });
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.35), Vec2::splat(ART)),
        Transform::from_xyz(0.0, 0.0, 3.0),
        Visibility::Hidden,
        Ghost,
        Scena,
    ));
}

/// Ricalcola la scala della griglia quando la finestra cambia e ridisegna
/// sfondo e linee; i moduli vengono riposizionati sulle nuove coordinate.
fn aggiorna_layout(
    mut commands: Commands,
    finestre: Query<&Window, With<PrimaryWindow>>,
    art: Res<Art>,
    mut griglia: ResMut<Griglia>,
    vecchie: Query<Entity, With<VisualeGriglia>>,
    mut moduli: Query<(&Module, &mut Transform, &mut Sprite), Without<Ostacolo>>,
    mut ostacoli: Query<(&Ostacolo, &mut Transform, &mut Sprite), Without<Module>>,
) {
    let Ok(finestra) = finestre.single() else {
        return;
    };
    let nuova = Griglia::da_finestra(finestra.width(), finestra.height());
    if !vecchie.is_empty() && nuova.cella == griglia.cella && nuova.centro == griglia.centro {
        return;
    }
    *griglia = nuova;

    for e in &vecchie {
        commands.entity(e).despawn();
    }
    let w = GRID_W as f32 * griglia.cella;
    let h = GRID_H as f32 * griglia.cella;
    commands.spawn((
        Sprite::from_color(ui::NERO, Vec2::new(w + 8.0, h + 8.0)),
        Transform::from_xyz(griglia.centro.x, griglia.centro.y, 0.0),
        VisualeGriglia,
        Scena,
    ));
    // Sfondo stellato sopra il pannello nero (che resta come cornice) e sotto
    // le linee: una tile per cella, allineata ai centri cella così il pattern
    // non "salta" al resize. La tile 64 è art a ×2, quindi a `cella` px segue
    // lo stesso fattore intero della griglia (1:1 a cella 64).
    for i in 0..GRID_W {
        for j in 0..GRID_H {
            let p = griglia.cella_in_mondo(IVec2::new(i, j));
            commands.spawn((
                Sprite {
                    image: art.sfondo_stelle.clone(),
                    custom_size: Some(Vec2::splat(griglia.cella)),
                    ..default()
                },
                Transform::from_xyz(p.x, p.y, 0.1),
                VisualeGriglia,
                Scena,
            ));
        }
    }
    for i in 0..=GRID_W {
        let x = griglia.centro.x - w / 2.0 + i as f32 * griglia.cella;
        commands.spawn((
            Sprite::from_color(ui::GRIGIO_SCAFO, Vec2::new(1.0, h)),
            Transform::from_xyz(x, griglia.centro.y, 0.5),
            VisualeGriglia,
            Scena,
        ));
    }
    for j in 0..=GRID_H {
        let y = griglia.centro.y - h / 2.0 + j as f32 * griglia.cella;
        commands.spawn((
            Sprite::from_color(ui::GRIGIO_SCAFO, Vec2::new(w, 1.0)),
            Transform::from_xyz(griglia.centro.x, y, 0.5),
            VisualeGriglia,
            Scena,
        ));
    }

    for (m, mut tf, mut sprite) in &mut moduli {
        let p = griglia.cella_in_mondo(m.cella);
        tf.translation.x = p.x;
        tf.translation.y = p.y;
        sprite.custom_size = Some(Vec2::splat(griglia.cella));
    }
    for (o, mut tf, mut sprite) in &mut ostacoli {
        let p = griglia.cella_in_mondo(o.cella);
        tf.translation.x = p.x;
        tf.translation.y = p.y;
        sprite.custom_size = Some(Vec2::splat(griglia.cella));
    }
}

fn cella_sotto_cursore(
    finestre: &Query<&Window, With<PrimaryWindow>>,
    camere: &Query<(&Camera, &GlobalTransform)>,
    griglia: &Griglia,
) -> Option<IVec2> {
    let finestra = finestre.single().ok()?;
    let cursore = finestra.cursor_position()?;
    let (camera, tf) = camere.single().ok()?;
    let mondo = camera.viewport_to_world_2d(tf, cursore).ok()?;
    griglia.mondo_in_cella(mondo)
}

fn cursore_sopra(
    finestre: Query<&Window, With<PrimaryWindow>>,
    camere: Query<(&Camera, &GlobalTransform)>,
    griglia: Res<Griglia>,
    station: Res<Station>,
    mut sotto: ResMut<SottoCursore>,
) {
    sotto.0 = cella_sotto_cursore(&finestre, &camere, &griglia)
        .and_then(|c| station.celle.get(&c).copied());
}

fn input_tastiera(
    tasti: Res<ButtonInput<KeyCode>>,
    pausa: Res<Pausa>,
    sotto: Res<SottoCursore>,
    progressione: Res<livelli::Progressione>,
    mut sel: ResMut<Selected>,
    mut sim: ResMut<Sim>,
    mut log: ResMut<EventLog>,
    mut moduli: Query<&mut Module>,
) {
    if pausa.aperta {
        return;
    }
    // 1..6 per i moduli base, 7 8 9 0 C per gli sbloccabili
    const TASTI: [KeyCode; 11] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
        KeyCode::KeyC,
    ];
    for (i, tasto) in TASTI.iter().enumerate() {
        if tasti.just_pressed(*tasto) && progressione.completati >= KINDS[i].def().sblocco {
            sel.0 = KINDS[i];
        }
    }
    if tasti.just_pressed(KeyCode::Space) {
        sim.running = !sim.running;
        let msg = if sim.running {
            "Simulazione avviata"
        } else {
            "Simulazione in pausa"
        };
        log.info(sim.tick, msg);
    }
    if tasti.just_pressed(KeyCode::KeyR)
        && let Some(e) = sotto.0
        && let Ok(mut m) = moduli.get_mut(e)
        && m.broken
    {
        m.broken = false;
        log.info(sim.tick, format!("Riparato: {}", m.etichetta));
    }
}

/// Piazza fisicamente un modulo sulla cella: sprite, figli (overlay, badge,
/// numero), registrazione in `Station`. Nessuna validazione: i chiamanti
/// (click del giocatore, servizio fotografico) controllano prima.
fn costruisci_modulo(
    commands: &mut Commands,
    station: &mut Station,
    art: &Art,
    griglia: &Griglia,
    kind: ModuleKind,
    cella: IVec2,
) -> String {
    let def = kind.def();
    station.contatori[kind.index()] += 1;
    station.seq += 1;
    let numero = station.contatori[kind.index()];
    let etichetta = format!("{} {}", def.nome, numero);
    let pos = griglia.cella_in_mondo(cella);
    let lato = griglia.cella;
    let e = commands
        .spawn((
            Sprite {
                image: art.moduli[kind.index()].clone(),
                custom_size: Some(Vec2::splat(lato)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 1.0),
            Module {
                kind,
                etichetta: etichetta.clone(),
                cella,
                seq: station.seq,
                powered: true,
                broken: false,
                staffed: true,
                collegato: true, // il primo tick (anche in anteprima) lo ricalcola
                carica: 0.0,
                lavoro: 0,
            },
            Scena,
        ))
        .with_children(|p| {
            // velo scuro sopra lo sprite quando il modulo è fermo
            p.spawn((
                Sprite::from_color(Color::NONE, Vec2::splat(ART)),
                Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec3::splat(1.0)),
                Visibility::Hidden,
                Overlay,
            ));
            // badge del motivo: fulmine / omino / triangolo
            p.spawn((
                Sprite {
                    image: Handle::default(),
                    custom_size: Some(Vec2::splat(ART * 0.3)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.2),
                Visibility::Hidden,
                Badge,
            ));
            p.spawn((
                Text2d::new(numero.to_string()),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(ui::BIANCO),
                Anchor::BOTTOM_RIGHT,
                Transform::from_xyz(0.0, 0.0, 0.3),
                Numero,
            ));
        })
        .id();
    station.celle.insert(cella, e);
    etichetta
}

fn input_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    finestre: Query<&Window, With<PrimaryWindow>>,
    camere: Query<(&Camera, &GlobalTransform)>,
    griglia: Res<Griglia>,
    art: Res<Art>,
    suoni: Res<audio::Suoni>,
    imp: Res<impostazioni::Impostazioni>,
    pausa: Res<Pausa>,
    mut station: ResMut<Station>,
    sel: Res<Selected>,
    sim: Res<Sim>,
    mut log: ResMut<EventLog>,
    moduli: Query<&Module>,
    mut commands: Commands,
) {
    if pausa.aperta {
        return;
    }
    let sinistro = mouse.just_pressed(MouseButton::Left);
    let destro = mouse.just_pressed(MouseButton::Right);
    if !sinistro && !destro {
        return;
    }
    let Some(cella) = cella_sotto_cursore(&finestre, &camere, &griglia) else {
        return;
    };

    if sinistro
        && !station.celle.contains_key(&cella)
        && !station.ostacoli.contains(&cella)
        && !station.sotto_limite()
    {
        // il limite è del livello, non della cella: avvisare qui, dove il
        // giocatore sta provando a costruire, non seppellirlo nel briefing
        log.push(
            sim.tick,
            sim::Gravita::Avviso,
            format!(
                "Limite moduli raggiunto ({}): rimuovi qualcosa col tasto destro",
                station.max_moduli.unwrap_or(0)
            ),
        );
        return;
    }

    if sinistro && !station.celle.contains_key(&cella) && !station.ostacoli.contains(&cella) {
        let kind = sel.0;
        // il Centro comando coordina gli arrivi di tutta la stazione:
        // il secondo non avrebbe niente da coordinare
        if kind == ModuleKind::CentroComando
            && moduli.iter().any(|m| m.kind == ModuleKind::CentroComando)
        {
            log.push(
                sim.tick,
                sim::Gravita::Avviso,
                "Centro comando: massimo uno per stazione",
            );
            return;
        }
        let etichetta = costruisci_modulo(&mut commands, &mut station, &art, &griglia, kind, cella);
        log.info(sim.tick, format!("Costruito: {}", etichetta));
        audio::suona(&mut commands, &suoni.costruzione, imp.effetti_lineare());
    }

    if destro && let Some(e) = station.celle.remove(&cella) {
        if let Ok(m) = moduli.get(e) {
            log.info(sim.tick, format!("Rimosso: {}", m.etichetta));
        }
        commands.entity(e).despawn();
        audio::suona(&mut commands, &suoni.rimozione, imp.effetti_lineare());
    }
}

/// L'anteprima usa lo sprite del modulo selezionato, snappata alla cella.
#[allow(clippy::too_many_arguments)]
fn aggiorna_ghost(
    sel: Res<Selected>,
    art: Res<Art>,
    griglia: Res<Griglia>,
    station: Res<Station>,
    pausa: Res<Pausa>,
    prologo_res: Res<prologo::Prologo>,
    scorte: Res<mercato::Mercato>,
    stato: Res<State<AppState>>,
    finestre: Query<&Window, With<PrimaryWindow>>,
    camere: Query<(&Camera, &GlobalTransform)>,
    mut q: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<Ghost>>,
) {
    let Ok((mut sprite, mut tf, mut vis)) = q.single_mut() else {
        return;
    };
    // niente anteprima sotto gli overlay: con prologo o scorte aperti il
    // giocatore non sta piazzando (e il click è comunque bloccato)
    if pausa.aperta
        || prologo_res.pagina.is_some()
        || scorte.aperto
        || *stato.get() != AppState::InGioco
    {
        *vis = Visibility::Hidden;
        return;
    }
    match cella_sotto_cursore(&finestre, &camere, &griglia) {
        Some(cella)
            if !station.celle.contains_key(&cella)
                && !station.ostacoli.contains(&cella)
                && station.sotto_limite() =>
        {
            sprite.image = art.moduli[sel.0.index()].clone();
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.4);
            sprite.custom_size = Some(Vec2::splat(griglia.cella));
            let p = griglia.cella_in_mondo(cella);
            tf.translation.x = p.x;
            tf.translation.y = p.y;
            *vis = Visibility::Visible;
        }
        _ => *vis = Visibility::Hidden,
    }
}

/// Stato del modulo: velo + badge che dice *perché* è fermo. Lo sprite base
/// non cambia mai, così l'identità del modulo resta leggibile.
/// Badge e numero compensano la rotazione del genitore (i corridoi ruotano
/// con l'autotiling): rotazione locale inversa e offset contro-ruotato, così
/// simboli e cifre restano dritti e nell'angolo giusto dello schermo.
fn aggiorna_visuali(
    tempo: Res<Time>,
    art: Res<Art>,
    griglia: Res<Griglia>,
    moduli: Query<(&Module, &Transform), (Without<Badge>, Without<Numero>)>,
    mut overlay: Query<(&ChildOf, &mut Sprite, &mut Visibility), (With<Overlay>, Without<Badge>)>,
    mut badge: Query<
        (&ChildOf, &mut Sprite, &mut Visibility, &mut Transform),
        (With<Badge>, Without<Overlay>),
    >,
    mut numeri: Query<(&ChildOf, &mut Transform), (With<Numero>, Without<Badge>)>,
) {
    let lampeggia = (tempo.elapsed_secs() * 2.0) as i32 % 2 == 0;
    let lato = griglia.cella;

    for (padre, mut sprite, mut vis) in &mut overlay {
        let Ok((m, _)) = moduli.get(padre.parent()) else {
            continue;
        };
        sprite.custom_size = Some(Vec2::splat(lato));
        match m.motivo_fermo() {
            None => *vis = Visibility::Hidden,
            Some(Fermo::Avaria) => {
                sprite.color = ui::ROSSO_SCURO.with_alpha(0.45);
                *vis = Visibility::Visible;
            }
            Some(_) => {
                sprite.color = ui::NERO.with_alpha(0.55);
                *vis = Visibility::Visible;
            }
        }
    }

    for (padre, mut sprite, mut vis, mut tf) in &mut badge {
        let Ok((m, tf_padre)) = moduli.get(padre.parent()) else {
            continue;
        };
        let lato_badge = lato * 0.3;
        sprite.custom_size = Some(Vec2::splat(lato_badge));
        let inv = tf_padre.rotation.inverse();
        tf.rotation = inv;
        let d = lato / 2.0 - lato_badge / 2.0 - 2.0;
        tf.translation = inv * Vec3::new(d, d, 0.2);
        match m.motivo_fermo() {
            None => *vis = Visibility::Hidden,
            Some(Fermo::Energia) => {
                sprite.image = art.badge_energia.clone();
                sprite.color = Color::WHITE;
                *vis = Visibility::Visible;
            }
            Some(Fermo::Scollegato) => {
                // stesso fulmine del blackout ma tinto di grigio: "qui la
                // corrente non arriva proprio", non "la corrente non basta"
                sprite.image = art.badge_energia.clone();
                sprite.color = ui::METALLO;
                *vis = Visibility::Visible;
            }
            Some(Fermo::Equipaggio) => {
                sprite.image = art.badge_equipaggio.clone();
                sprite.color = Color::WHITE;
                *vis = Visibility::Visible;
            }
            Some(Fermo::Avaria) => {
                sprite.image = art.badge_avaria.clone();
                sprite.color = Color::WHITE;
                // il lampeggio distingue l'avaria dagli altri stati anche
                // senza guardare il disegno del badge
                *vis = if lampeggia {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    for (padre, mut tf) in &mut numeri {
        let Ok((_, tf_padre)) = moduli.get(padre.parent()) else {
            continue;
        };
        let inv = tf_padre.rotation.inverse();
        tf.rotation = inv;
        tf.translation = inv * Vec3::new(lato / 2.0 - 2.0, -lato / 2.0 + 2.0, 0.3);
    }
}

/// Sceglie sprite e rotazione di un corridoio dalla maschera dei 4 vicini
/// ortogonali (qualsiasi modulo, non solo corridoi: la grafica comunica
/// esattamente la regola dell'adiacenza elettrica). Ritorna l'indice in
/// `Art::corridoi` e la rotazione in radianti (multipli di 90°, l'art è a
/// pixel quadrati e non si sfoca). Sprite base: curva = destra+basso,
/// T = sinistra+destra+basso.
fn sprite_corridoio(su: bool, giu: bool, sin: bool, des: bool) -> (usize, f32) {
    use std::f32::consts::{FRAC_PI_2, PI};
    // rotazioni CCW: +90° manda destra→su, su→sinistra, sinistra→giù, giù→destra
    let vicini = u8::from(su) + u8::from(giu) + u8::from(sin) + u8::from(des);
    match vicini {
        4 => (CORRIDOIO_CROCE, 0.0),
        3 => {
            if !su {
                (CORRIDOIO_T, 0.0)
            } else if !sin {
                (CORRIDOIO_T, FRAC_PI_2)
            } else if !giu {
                (CORRIDOIO_T, PI)
            } else {
                (CORRIDOIO_T, PI + FRAC_PI_2)
            }
        }
        2 => {
            if su && giu {
                (CORRIDOIO_V, 0.0)
            } else if sin && des {
                (CORRIDOIO_H, 0.0)
            } else if des && giu {
                (CORRIDOIO_CURVA, 0.0)
            } else if su && des {
                (CORRIDOIO_CURVA, FRAC_PI_2)
            } else if sin && su {
                (CORRIDOIO_CURVA, PI)
            } else {
                (CORRIDOIO_CURVA, PI + FRAC_PI_2) // giù e sinistra
            }
        }
        // 0 o 1 vicino: orizzontale, a meno che l'unico vicino sia verticale
        _ => {
            if su || giu {
                (CORRIDOIO_V, 0.0)
            } else {
                (CORRIDOIO_H, 0.0)
            }
        }
    }
}

/// Riorienta i corridoi in base ai moduli adiacenti. Gira a ogni frame su
/// `Station::celle` (aggiornata in modo sincrono da piazzamenti e rimozioni),
/// quindi copre anche i vicini di un modulo qualsiasi appena piazzato o
/// rimosso; scrive sprite e rotazione solo quando cambiano.
fn orienta_corridoi(
    art: Res<Art>,
    station: Res<Station>,
    mut q: Query<(&Module, &mut Sprite, &mut Transform)>,
) {
    for (m, mut sprite, mut tf) in &mut q {
        if m.kind != ModuleKind::Corridoio {
            continue;
        }
        let occupata = |d: IVec2| station.celle.contains_key(&(m.cella + d));
        let (idx, rot) = sprite_corridoio(
            occupata(IVec2::Y),
            occupata(IVec2::NEG_Y),
            occupata(IVec2::NEG_X),
            occupata(IVec2::X),
        );
        if sprite.image != art.corridoi[idx] {
            sprite.image = art.corridoi[idx].clone();
        }
        let rotazione = Quat::from_rotation_z(rot);
        if tf.rotation != rotazione {
            tf.rotation = rotazione;
        }
    }
}

fn visibilita_scena(
    stato: Res<State<AppState>>,
    mut q: Query<&mut Visibility, (With<Scena>, Without<Ghost>)>,
) {
    if !stato.is_changed() {
        return;
    }
    // A fine partita la stazione resta visibile sotto l'overlay, come in pausa.
    let v = if matches!(*stato.get(), AppState::InGioco | AppState::FinePartita) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut q {
        *vis = v;
    }
}

/// Sostituisce il cursore di sistema con la freccia pixel-art appena
/// l'immagine è caricata (una volta sola): l'hotspot (0,0) è la punta,
/// come disegnata in `gen_sprites.py`.
/// Cursore pixel-art a due stati: freccia nei menu e negli overlay, mirino
/// (hotspot al centro) quando si sta davvero piazzando sulla griglia.
#[allow(clippy::too_many_arguments)]
fn cursore_pixel(
    mut attuale: Local<Option<bool>>,
    immagini: Res<Assets<Image>>,
    art: Res<Art>,
    stato: Res<State<AppState>>,
    pausa: Res<Pausa>,
    prologo_res: Res<prologo::Prologo>,
    scorte: Res<mercato::Mercato>,
    finestre: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};
    if immagini.get(&art.cursore).is_none() || immagini.get(&art.mirino).is_none() {
        return;
    }
    let mirino = *stato.get() == AppState::InGioco
        && !pausa.aperta
        && prologo_res.pagina.is_none()
        && !scorte.aperto;
    if *attuale == Some(mirino) {
        return;
    }
    let Ok(finestra) = finestre.single() else {
        return;
    };
    let (handle, hotspot) = if mirino {
        (art.mirino.clone(), (8, 8))
    } else {
        (art.cursore.clone(), (1, 0))
    };
    commands
        .entity(finestra)
        .insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle,
            hotspot,
            ..default()
        })));
    *attuale = Some(mirino);
}

/// F12: screenshot della finestra nella cartella corrente, in ogni schermata.
fn screenshot_tasto(mut commands: Commands, tasti: Res<ButtonInput<KeyCode>>) {
    use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
    if tasti.just_pressed(KeyCode::F12) {
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("space-station-{epoch}.png")));
    }
}

/// Servizio fotografico: con `DEMO_FOTO=<cartella>` il gioco si guida da
/// solo — fotografa il titolo, apre una partita Infinita, costruisce una
/// stazione d'esempio, avvia la simulazione, scatta due volte ed esce.
/// Serve al manuale (docs/img) e a qualunque materiale illustrativo: foto
/// riproducibili senza mani umane. I numeri sono frame a ~60 fps.
#[allow(clippy::too_many_arguments)]
fn demo_foto(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut modalita: ResMut<Modalita>,
    mut reset: ResMut<RichiestaReset>,
    mut prossimo: ResMut<NextState<AppState>>,
    mut station: ResMut<Station>,
    mut sim: ResMut<Sim>,
    art: Res<Art>,
    griglia: Res<Griglia>,
    mut esci: MessageWriter<AppExit>,
) {
    use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
    let Ok(cartella) = std::env::var("DEMO_FOTO") else {
        return;
    };
    *frame += 1;
    let scatta = |commands: &mut Commands, nome: &str| {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("{cartella}/{nome}")));
    };
    match *frame {
        20 => scatta(&mut commands, "titolo.png"),
        40 => {
            *modalita = Modalita::Infinita;
            reset.0 = true;
            prossimo.set(AppState::InGioco);
        }
        80 => {
            // stazione d'esempio in equilibrio: un reattore regge tutto
            let piano = [
                (ModuleKind::Reattore, 5, 4),
                (ModuleKind::LifeSupport, 6, 4),
                (ModuleKind::Dormitorio, 7, 4),
                (ModuleKind::Dormitorio, 8, 4),
                (ModuleKind::Radiatore, 5, 3),
                (ModuleKind::Corridoio, 6, 3),
                (ModuleKind::Laboratorio, 7, 3),
            ];
            for (kind, x, y) in piano {
                costruisci_modulo(
                    &mut commands,
                    &mut station,
                    &art,
                    &griglia,
                    kind,
                    IVec2::new(x, y),
                );
            }
            sim.running = true;
        }
        110 => scatta(&mut commands, "costruzione.png"),
        560 => scatta(&mut commands, "partita.png"),
        600 => {
            esci.write(AppExit::Success);
        }
        _ => {}
    }
}

/// A lavoro compiuto (TICK_LAVORO_GRU tick attivi consecutivi, contati in
/// sim.rs) la Gru rimuove UN detrito ortogonalmente adiacente e si smonta:
/// due celle libere al prezzo di un modulo. Senza detriti adiacenti resta
/// lì a consumare: piazzarla bene è parte del gioco.
fn applica_gru(
    mut commands: Commands,
    mut station: ResMut<Station>,
    sim: Res<Sim>,
    mut log: ResMut<EventLog>,
    moduli: Query<(Entity, &Module)>,
    ostacoli_q: Query<(Entity, &Ostacolo)>,
) {
    for (e, m) in &moduli {
        if m.kind != ModuleKind::Gru || m.lavoro < TICK_LAVORO_GRU {
            continue;
        }
        let Some(cella_detrito) = [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y]
            .into_iter()
            .map(|d| m.cella + d)
            .find(|c| station.ostacoli.contains(c))
        else {
            continue;
        };
        station.ostacoli.remove(&cella_detrito);
        if let Some((e_detrito, _)) = ostacoli_q.iter().find(|(_, o)| o.cella == cella_detrito) {
            commands.entity(e_detrito).despawn();
        }
        station.celle.remove(&m.cella);
        commands.entity(e).despawn();
        log.info(
            sim.tick,
            format!("{}: detrito rimosso, la gru si smonta", m.etichetta),
        );
    }
}

/// Quando la simulazione alza il flag di fine partita, l'app passa alla
/// schermata "STAZIONE PERSA". Solo in modalità Infinita la partita entra
/// in classifica (e il file si scrive qui, una volta sola, mai a ogni tick).
fn controlla_fine(
    sim: Res<Sim>,
    stato: Res<State<AppState>>,
    modalita: Res<Modalita>,
    mut classifica_infinita: ResMut<livelli::ClassificaInfinita>,
    mut classifica_sfida: ResMut<livelli::ClassificaSfida>,
    mut piazzamento: ResMut<livelli::UltimoPiazzamento>,
    mut prossimo: ResMut<NextState<AppState>>,
) {
    if sim.partita_finita && *stato.get() == AppState::InGioco {
        piazzamento.0 = None;
        match *modalita {
            Modalita::Infinita => {
                piazzamento.0 = classifica_infinita.registra(&sim);
                classifica_infinita.salva();
            }
            Modalita::Sfida => {
                piazzamento.0 = classifica_sfida.registra(&sim);
                classifica_sfida.salva();
            }
            Modalita::Campagna(_) | Modalita::Casuale => {}
        }
        prossimo.set(AppState::FinePartita);
    }
}

/// Azzeramento della partita: la stazione sparisce e la simulazione riparte
/// da capo, in pausa-costruzione. Si azzera anche lo stato del livello; in
/// campagna il log apre con l'obiettivo, così è chiaro cosa si sta facendo.
#[allow(clippy::too_many_arguments)]
fn applica_reset(
    mut richiesta: ResMut<RichiestaReset>,
    mut commands: Commands,
    mut station: ResMut<Station>,
    mut sim: ResMut<Sim>,
    mut log: ResMut<EventLog>,
    mut sel: ResMut<Selected>,
    mut stato_livello: ResMut<livelli::StatoLivello>,
    mut offerte: ResMut<mercato::Mercato>,
    mut stato_musica: ResMut<musica::StatoMusica>,
    mut prologo_res: ResMut<prologo::Prologo>,
    modalita: Res<Modalita>,
    casuale: Res<livelli::LivelloCasuale>,
    griglia: Res<Griglia>,
    art: Res<Art>,
    moduli: Query<Entity, With<Module>>,
    ostacoli_vecchi: Query<Entity, With<Ostacolo>>,
) {
    if !richiesta.0 {
        return;
    }
    richiesta.0 = false;
    for e in moduli.iter().chain(&ostacoli_vecchi) {
        commands.entity(e).despawn();
    }
    *station = Station::default();
    *sim = Sim::default();
    sim.tetto_tick = match *modalita {
        Modalita::Infinita => None,
        Modalita::Sfida | Modalita::Campagna(_) | Modalita::Casuale => Some(TICK_MASSIMO),
    };
    *stato_livello = livelli::StatoLivello::default();
    sel.0 = ModuleKind::Reattore;
    stato_musica.pesca_casuale();
    log.svuota();
    // campagna e casuale condividono tutto: livello con obiettivo, detriti
    // e budget; cambia solo da dove arriva la definizione
    let livello = match *modalita {
        Modalita::Campagna(i) => Some(&LIVELLI[i]),
        Modalita::Casuale => casuale.0.as_ref(),
        Modalita::Infinita | Modalita::Sfida => None,
    };
    if let Some(livello) = livello {
        station.max_moduli = Some(livello.max_moduli);
        let intestazione = match *modalita {
            Modalita::Campagna(i) => format!("Livello {} — {}", i + 1, livello.nome),
            _ => format!("Livello casuale — {}", livello.nome),
        };
        log.info(
            0,
            format!("{}: {}", intestazione, livello.obiettivo.descrizione()),
        );
        for &(x, y) in &livello.ostacoli {
            let cella = IVec2::new(x, y);
            station.ostacoli.insert(cella);
            let p = griglia.cella_in_mondo(cella);
            commands.spawn((
                Sprite {
                    image: art.ostacolo.clone(),
                    custom_size: Some(Vec2::splat(griglia.cella)),
                    ..default()
                },
                Transform::from_xyz(p.x, p.y, 1.0),
                Ostacolo { cella },
                Scena,
            ));
        }
        if !livello.ostacoli.is_empty() {
            log.info(0, "Detriti sulla griglia: costruisci intorno");
        }
        log.info(0, format!("Moduli disponibili: {}", livello.max_moduli));
        log.info(0, "Costruisci e premi Spazio");
        // il prologo a fumetto copre la griglia finché non si preme Gioca!
        prologo_res.pagina = Some(0);
    } else {
        match *modalita {
            Modalita::Sfida => log.info(
                0,
                format!("Nuova stazione (Sfida, {TICK_MASSIMO} tick): costruisci e premi Spazio"),
            ),
            _ => log.info(0, "Nuova stazione: costruisci e premi Spazio"),
        }
        prologo_res.pagina = None;
    }
    offerte.aperto = false;
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn budget_moduli_dei_livelli_sta_nella_griglia_libera() {
        for (n, livello) in LIVELLI.iter().enumerate() {
            let libere = (GRID_W * GRID_H) as u32 - livello.ostacoli.len() as u32;
            assert!(
                livello.max_moduli <= libere,
                "livello {}: budget {} > {} celle libere",
                n + 1,
                livello.max_moduli,
                libere
            );
            // sotto i 4 moduli (reattore, life support, dormitorio,
            // radiatore) nessun obiettivo è raggiungibile
            assert!(
                livello.max_moduli >= 4,
                "livello {}: budget {} sotto il minimo vitale",
                n + 1,
                livello.max_moduli
            );
        }
    }

    #[test]
    fn ostacoli_dei_livelli_dentro_la_griglia_e_senza_doppioni() {
        for (n, livello) in LIVELLI.iter().enumerate() {
            let mut viste = HashSet::new();
            for &(x, y) in &livello.ostacoli {
                assert!(
                    (0..GRID_W).contains(&x) && (0..GRID_H).contains(&y),
                    "livello {}: ostacolo fuori griglia ({x},{y})",
                    n + 1
                );
                assert!(
                    viste.insert((x, y)),
                    "livello {}: ostacolo duplicato ({x},{y})",
                    n + 1
                );
            }
        }
    }

    // argomenti di sprite_corridoio: (su, giu, sin, des)

    #[test]
    fn corridoio_isolato_o_in_fila_orizzontale() {
        assert_eq!(sprite_corridoio(false, false, false, false).0, CORRIDOIO_H);
        assert_eq!(sprite_corridoio(false, false, true, false).0, CORRIDOIO_H);
        assert_eq!(sprite_corridoio(false, false, true, true).0, CORRIDOIO_H);
    }

    #[test]
    fn corridoio_con_soli_vicini_verticali() {
        assert_eq!(sprite_corridoio(true, false, false, false).0, CORRIDOIO_V);
        assert_eq!(sprite_corridoio(true, true, false, false).0, CORRIDOIO_V);
    }

    #[test]
    fn vicini_sopra_e_a_destra_curva_ruotata_di_90() {
        // la curva base collega destra e basso; +90° CCW la porta su su+destra
        let (idx, rot) = sprite_corridoio(true, false, false, true);
        assert_eq!(idx, CORRIDOIO_CURVA);
        assert!((rot - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn curva_base_destra_basso_senza_rotazione() {
        let (idx, rot) = sprite_corridoio(false, true, false, true);
        assert_eq!(idx, CORRIDOIO_CURVA);
        assert_eq!(rot, 0.0);
    }

    #[test]
    fn tre_vicini_t_ruotata() {
        // la T base collega sinistra+destra+basso: senza "su" resta a 0°
        let (idx, rot) = sprite_corridoio(false, true, true, true);
        assert_eq!(idx, CORRIDOIO_T);
        assert_eq!(rot, 0.0);
        // manca "giù": T capovolta (180°)
        let (idx, rot) = sprite_corridoio(true, false, true, true);
        assert_eq!(idx, CORRIDOIO_T);
        assert!((rot - PI).abs() < 1e-6);
    }

    #[test]
    fn quattro_vicini_croce() {
        assert_eq!(sprite_corridoio(true, true, true, true).0, CORRIDOIO_CROCE);
    }
}
