//! Schermate di menu: Titolo, "Come si gioca", overlay di pausa e fine partita.
//! Due pause diverse e volutamente distinte: `Spazio` ferma la simulazione ma
//! lascia costruire (l'HUD mostra l'anteprima del bilancio), `Esc` apre questo
//! menu e congela tutto, timer del tick compreso.

use crate::audio::BottoneMuto;
use crate::generatore;
use crate::impostazioni::{Impostazioni, ciclo};
use crate::livelli::{
    ClassificaInfinita, ClassificaSfida, LIVELLI, LivelloCasuale, LivelloScelto, Modalita,
    Progressione, Record, SfidaDelGiorno, UltimoPiazzamento, giorni_fa, giorno_corrente,
};
use crate::mercato::FACILITIES;
use crate::progressi::Portafoglio;
use rand::RngExt;
use crate::modules::{KINDS, TABELLA};
use crate::personaggi::{PERSONAGGI, annuncio_sblocco};
use crate::sim::{MotivoFine, OSSIGENO_PER_CREW, Sim, TICK_SURRISCALDAMENTO, TICK_SECS};
use crate::ui::{
    BIANCO, CIANO, GIALLO, GRIGIO_MEDIO, GRIGIO_SCAFO, METALLO, NERO, ROSSO, SCAFO_SCURO, VERDE,
};
use crate::{Art, RichiestaReset};
use bevy::app::AppExit;
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Titolo,
    ComeSiGioca,
    /// Campagna: griglia dei 50 livelli con stato completato/disponibile/bloccato.
    SelezioneLivello,
    /// Schermata di storia (diario di bordo) prima dei livelli chiave della
    /// campagna: solo la prima volta che ci si arriva. Da qui (o
    /// direttamente dalla selezione) si entra in partita: nome, obiettivo e
    /// numeri del livello li dà il prologo sopra la griglia (prologo.rs) —
    /// la vecchia schermata di briefing non esiste più.
    Intermezzo,
    /// Top 10 delle partite in modalità infinita.
    SchermataClassifica,
    /// Catalogo delle facilities: si compra coi crediti delle medaglie,
    /// mai con valuta reale. Le scorte comprate si usano in partita
    /// cliccando le loro icone nella colonna sinistra.
    Marketplace,
    InGioco,
    /// Obiettivo del livello raggiunto: punteggio, tick e avanzamento.
    LivelloCompletato,
    /// Stazione persa: schermata sopra la scena di gioco (che resta visibile
    /// sotto, come per l'overlay di pausa), con punteggio e statistiche.
    FinePartita,
}

/// Overlay di pausa: vive dentro InGioco, quindi non è uno stato a sé (la
/// scena di gioco deve restare visibile sotto).
#[derive(Resource, Default)]
pub struct Pausa {
    pub aperta: bool,
}

/// Da dove si è arrivati a "Come si gioca": si torna lì, non sempre al titolo.
#[derive(Resource)]
pub struct Origine {
    pub stato: AppState,
    pub da_pausa: bool,
}

impl Default for Origine {
    fn default() -> Self {
        Self {
            stato: AppState::Titolo,
            da_pausa: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct Selezione {
    pub idx: usize,
    pub n: usize,
    /// Voce in attesa di conferma (azioni che distruggono la stazione).
    pub conferma: Option<usize>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Azione {
    /// Apre la selezione livello della campagna.
    ApriCampagna,
    /// Nuova partita in modalità infinita (sandbox, punteggio in classifica).
    GiocaInfinita,
    /// Come Infinita ma col tetto di tick: classifica separata.
    GiocaSfida,
    /// Genera un livello con seed casuale e lo gioca (fuori progressione).
    GiocaCasuale,
    /// La sfida del giorno: stesso livello per tutti oggi, miglior tempo
    /// personale registrato. Fuori da classifiche e progressione.
    GiocaGiornaliera,
    ApriClassifica,
    ApriMarketplace,
    /// Compra la facility `i` del catalogo, se i crediti bastano.
    CompraFacility(usize),
    /// Dalla selezione livello dritti in partita (via intermezzo se il
    /// livello ne ha uno mai visto), col prologo aperto sopra la griglia.
    ScegliLivello(usize),
    /// Avvia il livello già in `LivelloScelto` (usata dall'intermezzo).
    IniziaLivello,
    /// Da "livello completato" al livello successivo, dritti in partita.
    LivelloSuccessivo,
    /// Apre assets/manuale.html nel browser (la zine: la versione bella).
    ApriManuale,
    /// Apre assets/manuale.pdf col visualizzatore (per chi stampa).
    ApriManualePdf,
    ComeSiGioca,
    Riprendi,
    /// Passo successivo del volume musica (100→75→50→25→0→100).
    CicloMusica,
    /// Passo successivo del volume effetti.
    CicloEffetti,
    Ricomincia,
    TornaAlTitolo,
    /// Torna alla schermata precedente registrata in `Origine` (guida).
    Indietro,
    /// Torna al titolo da una schermata di solo menu: nessuna stazione da
    /// perdere, quindi niente conferma (a differenza di `TornaAlTitolo`).
    IndietroTitolo,
    Esci,
}

impl Azione {
    /// Le azioni che buttano via la stazione chiedono conferma inline.
    fn distruttiva(self) -> bool {
        matches!(self, Azione::Ricomincia | Azione::TornaAlTitolo)
    }
}

#[derive(Component)]
pub struct Voce {
    pub idx: usize,
    pub azione: Azione,
    pub etichetta: String,
}

#[derive(Component)]
pub struct SchermataTitolo;

#[derive(Component)]
pub struct SchermataGuida;

#[derive(Component)]
pub struct SchermataPausa;

#[derive(Component)]
pub struct SchermataFine;

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

/// Colore "di riposo" di una voce che non deve essere il METALLO standard:
/// `evidenzia_voci` lo rispetta quando la voce non è selezionata. Usato
/// per i numeri di livello colorati dalla medaglia.
#[derive(Component)]
pub struct ColoreFisso(pub Color);

/// Variante compatta di `voce` per la griglia dei livelli: una cella
/// quadrata col numero, stessi componenti e stessi sistemi di navigazione.
/// `colore` è il colore della medaglia (None = nessuna medaglia).
fn voce_cella(
    p: &mut ChildSpawnerCommands,
    idx: usize,
    azione: Azione,
    etichetta: String,
    colore: Option<Color>,
) {
    p.spawn((
        Node {
            width: Val::Px(46.0),
            padding: UiRect::axes(Val::Px(0.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        Button,
        Voce {
            idx,
            azione,
            etichetta: etichetta.clone(),
        },
    ))
    .with_children(|c| {
        let base = colore.unwrap_or(METALLO);
        let mut cella = c.spawn(testo(etichetta, 16.0, base));
        if let Some(colore) = colore {
            cella.insert(ColoreFisso(colore));
        }
    });
}

fn voce(p: &mut ChildSpawnerCommands, idx: usize, azione: Azione, etichetta: impl Into<String>) {
    let etichetta = etichetta.into();
    p.spawn((
        Node {
            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            min_width: Val::Px(280.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        Button,
        Voce {
            idx,
            azione,
            etichetta: etichetta.clone(),
        },
    ))
    .with_children(|c| {
        c.spawn(testo(etichetta, 18.0, METALLO));
    });
}

/// Box a fumetto: ritratto del personaggio a sinistra, nome/ruolo e
/// balloon con la battuta a destra. Usato nel briefing e a livello
/// completato (annuncio degli sblocchi).
fn fumetto(p: &mut ChildSpawnerCommands, art: &Art, personaggio: usize, battuta: &str) {
    let chi = &PERSONAGGI[personaggio];
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(10.0),
        margin: UiRect::bottom(Val::Px(14.0)),
        max_width: Val::Px(500.0),
        ..default()
    })
    .with_children(|riga| {
        riga.spawn((
            ImageNode::new(art.ritratti[chi.ritratto].clone()),
            Node {
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                flex_shrink: 0.0,
                ..default()
            },
        ));
        riga.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn(testo(format!("{} — {}", chi.nome, chi.ruolo), 13.0, CIANO));
            col.spawn((
                Node {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
                    max_width: Val::Px(420.0),
                    ..default()
                },
                BorderColor::all(GRIGIO_SCAFO),
                BackgroundColor(SCAFO_SCURO),
            ))
            .with_children(|balloon| {
                balloon.spawn(testo(format!("\u{201C}{battuta}\u{201D}"), 14.0, BIANCO));
            });
        });
    });
}

/// Tick di simulazione in tempo reale "m:ss" (gemello del helper privato
/// di prologo.rs: entrambi piccoli, nessuna casa comune che li meriti).
fn tick_in_tempo_menu(tick: u64) -> String {
    let secondi = (tick as f32 * TICK_SECS) as u64;
    format!("{}:{:02}", secondi / 60, secondi % 60)
}

fn radice_centrata() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(10.0),
        ..default()
    }
}

// ---------------- Titolo ----------------

pub fn entra_titolo(
    mut commands: Commands,
    portafoglio: Res<Portafoglio>,
    mut sel: ResMut<Selezione>,
) {
    *sel = Selezione {
        idx: 0,
        n: 9,
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            // semitrasparente: dietro vive l'attract mode (attract.rs)
            BackgroundColor(NERO.with_alpha(0.82)),
            GlobalZIndex(10),
            SchermataTitolo,
        ))
        .with_children(|r| {
            r.spawn(testo("SPACE STATION", 46.0, BIANCO));
            r.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(testo(
                    "costruisci · avvia · leggi il bilancio · reagisci al guasto",
                    14.0,
                    GRIGIO_MEDIO,
                ));
            });
            voce(r, 0, Azione::ApriCampagna, "Campagna");
            voce(r, 1, Azione::GiocaInfinita, "Infinita");
            voce(r, 2, Azione::GiocaSfida, "Sfida");
            voce(r, 3, Azione::GiocaCasuale, "Livello casuale");
            voce(r, 4, Azione::GiocaGiornaliera, etichetta_giornaliera(&portafoglio));
            voce(r, 5, Azione::ApriMarketplace, "Marketplace");
            voce(r, 6, Azione::ApriClassifica, "Classifica");
            voce(r, 7, Azione::ComeSiGioca, "Come si gioca");
            voce(r, 8, Azione::Esci, "Esci");
        });
}

pub fn esci_titolo(mut commands: Commands, q: Query<Entity, With<SchermataTitolo>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Come si gioca ----------------

/// Apre un file di `assets/` con l'applicazione di sistema (browser per
/// l'HTML, visualizzatore per il PDF). Il percorso è risolto come
/// `percorso_assets` in main.rs: accanto all'eseguibile (build
/// distribuita), ripiego sulla radice del sorgente. Fallisce in silenzio:
/// un manuale che non si apre non merita un allarme di stazione.
fn apri_da_assets(nome: &str) {
    let percorso = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("assets").join(nome)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| {
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).join(nome)
        });
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&percorso)
            .spawn();
    } else {
        let _ = std::process::Command::new("xdg-open").arg(&percorso).spawn();
    }
}

pub fn entra_guida(
    mut commands: Commands,
    art: Res<Art>,
    progressione: Res<Progressione>,
    mut sel: ResMut<Selezione>,
) {
    *sel = Selezione {
        idx: 0,
        n: 3,
        conferma: None,
    };
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
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataGuida,
        ))
        .with_children(|r| {
            r.spawn(testo("COME SI GIOCA", 30.0, BIANCO));
            r.spawn(testo(
                "Piazza moduli sulla griglia e tieni in pari quattro risorse.",
                14.0,
                METALLO,
            ));
            r.spawn(testo(
                "Se l'energia non basta i moduli si spengono a partire dai meno critici: \
                 salta il life support, finisce l'ossigeno, l'equipaggio muore.",
                13.0,
                GRIGIO_MEDIO,
            ));

            // risorse
            r.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(18.0),
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                },
            ))
            .with_children(|riga| {
                let voci = [
                    (0, "Energia: la producono i reattori".to_string()),
                    (
                        1,
                        format!("Ossigeno: {:.0} per persona/tick", OSSIGENO_PER_CREW),
                    ),
                    (2, "Calore: lo dissipano i radiatori".to_string()),
                    (3, "Equipaggio: serve ai laboratori".to_string()),
                ];
                for (i, desc) in voci {
                    riga.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.0),
                            ..default()
                        },
                    ))
                    .with_children(|c| {
                        c.spawn((
                            ImageNode::new(art.icone[i].clone()),
                            Node {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                ..default()
                            },
                        ));
                        c.spawn(testo(desc, 12.0, GRIGIO_MEDIO));
                    });
                }
            });

            // gli 11 moduli, a griglia (6+5): gli sbloccabili non ancora
            // conquistati mostrano la soglia invece dei costi
            r.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    max_width: Val::Px(640.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(6.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ))
            .with_children(|griglia| {
                for (i, kind) in KINDS.iter().enumerate() {
                    let def = &TABELLA[kind.index()];
                    let bloccato = progressione.completati < def.sblocco;
                    griglia
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(2.0),
                                width: Val::Px(96.0),
                                ..default()
                            },
                        ))
                        .with_children(|c| {
                            c.spawn((
                                ImageNode {
                                    image: art.moduli[i].clone(),
                                    color: if bloccato { GRIGIO_MEDIO } else { Color::WHITE },
                                    ..default()
                                },
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                            ));
                            const TASTI_GUIDA: [&str; 11] =
                                ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "C"];
                            c.spawn(testo(
                                format!("{}  {}", TASTI_GUIDA[i], def.nome),
                                11.0,
                                if bloccato { GRIGIO_MEDIO } else { BIANCO },
                            ));
                            c.spawn(testo(
                                if bloccato {
                                    format!("liv. {}", def.sblocco)
                                } else {
                                    format!("En {:+.0} · Cal {:+.0}", def.energia, def.calore)
                                },
                                10.0,
                                GRIGIO_SCAFO,
                            ));
                        });
                }
            });

            for riga in [
                "1-6 e 7 8 9 0 C  scegli il modulo          click sinistro  piazza",
                "click destro  rimuove          R  ripara (2 di equipaggio per 8 tick)",
                "Spazio  avvia/ferma          V  velocita'          F12  screenshot",
                "Esc  apre il menu (volumi compresi) e congela tutto",
                "Finisci i livelli in fretta: le medaglie fruttano crediti per il Marketplace",
            ] {
                r.spawn(testo(riga, 13.0, CIANO));
            }
            r.spawn(testo(
                format!(
                    "Un tick dura {:.1}s. Con calore netto positivo per {} tick di fila un modulo va in avaria.",
                    TICK_SECS, TICK_SURRISCALDAMENTO
                ),
                12.0,
                GRIGIO_MEDIO,
            ));

            r.spawn(Node {
                height: Val::Px(16.0),
                ..default()
            });
            voce(r, 0, Azione::ApriManuale, "Manuale illustrato");
            voce(r, 1, Azione::ApriManualePdf, "Manuale in PDF (per la stampa)");
            voce(r, 2, Azione::Indietro, "Indietro");
        });
}

pub fn esci_guida(mut commands: Commands, q: Query<Entity, With<SchermataGuida>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Selezione livello ----------------

#[derive(Component)]
pub struct SchermataSelezione;

/// I 50 livelli in griglia (10 per riga). Solo i livelli sbloccati (completati o il primo
/// disponibile) sono voci navigabili; i bloccati sono testo spento, non
/// selezionabile né cliccabile.
pub fn entra_selezione(
    mut commands: Commands,
    progressione: Res<Progressione>,
    portafoglio: Res<Portafoglio>,
    mut sel: ResMut<Selezione>,
) {
    let sbloccati = (progressione.completati + 1).min(LIVELLI.len());
    *sel = Selezione {
        // si parte dal primo livello non completato, non dall'1
        idx: progressione.completati.min(sbloccati - 1),
        n: sbloccati + 1, // livelli sbloccati + Indietro
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataSelezione,
        ))
        .with_children(|r| {
            r.spawn(testo("CAMPAGNA", 34.0, BIANCO));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(18.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(
                    format!(
                        "50 livelli — completati {} · il colore è la medaglia (oro, argento, rame) · hai {} crediti · frecce e Invio",
                        progressione.completati, portafoglio.crediti
                    ),
                    13.0,
                    GRIGIO_MEDIO,
                ));
            });
            // griglia 10 per riga: i primi 6 sono i livelli curati, dal 7 in
            // poi generati (nome e obiettivo si vedono nel briefing)
            r.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                max_width: Val::Px(10.0 * 50.0),
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(4.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|griglia| {
                for i in 0..LIVELLI.len() {
                    if i < sbloccati {
                        let etichetta = if i < progressione.completati {
                            format!("{}·", i + 1) // il punto marca il completato
                        } else {
                            format!("{}", i + 1)
                        };
                        let colore = match portafoglio.medaglia(i) {
                            crate::progressi::ORO => Some(GIALLO),
                            crate::progressi::ARGENTO => Some(BIANCO),
                            crate::progressi::RAME => Some(crate::ui::RUGGINE),
                            _ => None,
                        };
                        voce_cella(griglia, i, Azione::ScegliLivello(i), etichetta, colore);
                    } else {
                        griglia
                            .spawn((Node {
                                width: Val::Px(46.0),
                                padding: UiRect::axes(Val::Px(0.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },))
                            .with_children(|c| {
                                c.spawn(testo(format!("{}", i + 1), 16.0, GRIGIO_SCAFO));
                            });
                    }
                }
            });
            r.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            voce(r, sbloccati, Azione::IndietroTitolo, "Indietro");
        });
}

pub fn esci_selezione(mut commands: Commands, q: Query<Entity, With<SchermataSelezione>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Intermezzo (storia) ----------------

#[derive(Component)]
pub struct SchermataIntermezzo;

/// Diario di bordo dei livelli chiave (1, 11, 21, 31, 41): il personaggio
/// del blocco racconta la svolta. Si vede solo la prima volta che si
/// raggiunge il livello; "Continua" entra in partita (col prologo sopra).
pub fn entra_intermezzo(
    mut commands: Commands,
    scelto: Res<LivelloScelto>,
    art: Res<Art>,
    mut sel: ResMut<Selezione>,
) {
    let Some(intermezzo) = crate::personaggi::intermezzo_per(scelto.0 + 1) else {
        // non dovrebbe succedere: lo stato si apre solo se l'intermezzo c'è
        return;
    };
    *sel = Selezione {
        idx: 0,
        n: 1,
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataIntermezzo,
        ))
        .with_children(|r| {
            r.spawn(testo(
                format!("DIARIO DI BORDO — LIVELLO {}", scelto.0 + 1),
                15.0,
                GRIGIO_MEDIO,
            ));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(intermezzo.titolo, 30.0, BIANCO));
            });
            fumetto(r, &art, intermezzo.personaggio, intermezzo.testo);
            r.spawn(Node {
                height: Val::Px(8.0),
                ..default()
            });
            voce(r, 0, Azione::IniziaLivello, "Continua");
        });
}

pub fn esci_intermezzo(mut commands: Commands, q: Query<Entity, With<SchermataIntermezzo>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Classifica ----------------

#[derive(Component)]
pub struct SchermataClassificaUi;

/// Una colonna della classifica: intestazione + righe, o l'avviso di
/// tabella vuota. I punteggi di Infinita e Sfida non sono confrontabili
/// (l'una non ha tetto di tick, l'altra sì), quindi due colonne separate
/// invece di un'unica tabella mescolata.
fn colonna_classifica(p: &mut ChildSpawnerCommands, titolo: &str, righe: &[Record]) {
    p.spawn(Node {
        flex_direction: FlexDirection::Column,
        min_width: Val::Px(360.0),
        row_gap: Val::Px(4.0),
        ..default()
    })
    .with_children(|c| {
        c.spawn(Node {
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|h| {
            h.spawn(testo(titolo, 16.0, CIANO));
        });
        if righe.is_empty() {
            c.spawn(testo("nessuna partita registrata", 14.0, GRIGIO_MEDIO));
        } else {
            for (i, record) in righe.iter().enumerate() {
                c.spawn(testo(
                    format!(
                        "{:>2}.  {:>6} punti   {:>5} tick   equipaggio {:>2}   {}",
                        i + 1,
                        record.punteggio,
                        record.tick,
                        record.equipaggio_max,
                        giorni_fa(record.epoch)
                    ),
                    14.0,
                    if i == 0 { BIANCO } else { METALLO },
                ));
            }
        }
    });
}

/// Le 10 migliori partite di Infinita e di Sfida, in due colonne: i
/// punteggi delle due modalità non sono confrontabili fra loro.
pub fn entra_classifica(
    mut commands: Commands,
    infinita: Res<ClassificaInfinita>,
    sfida: Res<ClassificaSfida>,
    mut sel: ResMut<Selezione>,
) {
    *sel = Selezione {
        idx: 0,
        n: 1,
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataClassificaUi,
        ))
        .with_children(|r| {
            r.spawn(testo("CLASSIFICA", 34.0, BIANCO));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(18.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(
                    "top 10 di Infinita e Sfida, separate: non sono punteggi confrontabili",
                    13.0,
                    GRIGIO_MEDIO,
                ));
            });
            r.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(48.0),
                ..default()
            })
            .with_children(|riga| {
                colonna_classifica(riga, "INFINITA — nessun tetto di tick", &infinita.0.righe);
                colonna_classifica(riga, "SFIDA — tetto 400 tick", &sfida.0.righe);
            });
            r.spawn(Node {
                height: Val::Px(16.0),
                ..default()
            });
            voce(r, 0, Azione::IndietroTitolo, "Indietro");
        });
}

pub fn esci_classifica(mut commands: Commands, q: Query<Entity, With<SchermataClassificaUi>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Marketplace ----------------

#[derive(Component)]
pub struct SchermataMarketplace;

/// Il testo del saldo crediti nella schermata Marketplace: aggiornato da
/// `aggiorna_voci_marketplace` dopo ogni acquisto.
#[derive(Component)]
pub struct SaldoCrediti;

/// Una voce-card del Marketplace: `evidenzia_voci` NON tocca né i testi né
/// lo stile delle card (hanno figli propri: icona, monete); lo stato
/// dorata/scura lo tiene aggiornato `aggiorna_voci_marketplace`.
#[derive(Component)]
pub struct StileCard {
    pub dorata: bool,
}

/// L'icona della facility dentro una card: si smorza quando i crediti non
/// bastano.
#[derive(Component)]
pub struct IconaCard(pub usize);

/// Il contatore "ne hai N" dentro la card della facility `i`.
#[derive(Component)]
pub struct PossedutePer(pub usize);

/// Una card del catalogo: quadrata, icona pixel-art, nome, possedute e
/// prezzo in monete. Dorata quando i crediti bastano, scura quando no.
/// Resta una `Voce` (navigazione con frecce e Invio, click, `esegui`).
fn card_facility(
    p: &mut ChildSpawnerCommands,
    idx: usize,
    i: usize,
    art: &Art,
    dorata: bool,
    possedute: usize,
) {
    let f = &FACILITIES[i];
    let mut card = p.spawn((
        Node {
            width: Val::Px(140.0),
            height: Val::Px(150.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(5.0),
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(SCAFO_SCURO),
        BorderColor::all(if dorata { GIALLO } else { GRIGIO_SCAFO }),
        Button,
        Voce {
            idx,
            azione: Azione::CompraFacility(i),
            etichetta: f.nome.into(),
        },
        StileCard { dorata },
    ));
    // una card che non puoi permetterti non deve nemmeno suonare al click
    if !dorata {
        card.insert(BottoneMuto);
    }
    card.with_children(|c| {
        c.spawn((
            ImageNode {
                image: art.facilities[i].clone(),
                color: if dorata { Color::WHITE } else { GRIGIO_MEDIO },
                ..default()
            },
            Node {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                ..default()
            },
            IconaCard(i),
        ));
        c.spawn(testo(f.nome, 12.0, BIANCO));
        let iniziale = if possedute > 0 {
            format!("ne hai {possedute}")
        } else {
            String::new()
        };
        c.spawn((testo(iniziale, 11.0, CIANO), PossedutePer(i)));
        // il prezzo: una moneta per credito, ferma (l'animazione è un
        // premio della schermata medaglia, qui è un cartellino)
        c.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(3.0),
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        })
        .with_children(|monete| {
            for _ in 0..f.costo_crediti {
                monete.spawn((
                    ImageNode::new(art.monete_accese[0].clone()),
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                ));
            }
        });
    });
}

/// Catalogo delle facilities: si compra coi crediti delle medaglie
/// (oro 3, argento 2, rame 1 — e solo la differenza quando si migliora),
/// mai con soldi veri. Le scorte comprate si usano in partita cliccando
/// le loro icone nella colonna sinistra (tooltip col motivo se inutili).
pub fn entra_marketplace(
    mut commands: Commands,
    portafoglio: Res<Portafoglio>,
    art: Res<Art>,
    mut sel: ResMut<Selezione>,
) {
    *sel = Selezione {
        idx: 0,
        n: FACILITIES.len() + 1, // catalogo + Indietro
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataMarketplace,
        ))
        .with_children(|r| {
            r.spawn(testo("MARKETPLACE", 34.0, BIANCO));
            r.spawn(testo(
                "si compra coi crediti delle medaglie — mai con soldi veri",
                13.0,
                GRIGIO_MEDIO,
            ));
            // saldo: moneta + numero, ben visibile
            r.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                margin: UiRect::bottom(Val::Px(14.0)),
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    ImageNode::new(art.monete_accese[0].clone()),
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                ));
                c.spawn((
                    testo(format!("hai {} crediti", portafoglio.crediti), 16.0, GIALLO),
                    SaldoCrediti,
                ));
            });
            // il catalogo: 6 card in griglia, 3 per riga
            r.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                max_width: Val::Px(3.0 * 152.0),
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|griglia| {
                for (i, f) in FACILITIES.iter().enumerate() {
                    let dorata = portafoglio.crediti >= f.costo_crediti;
                    let possedute =
                        portafoglio.scorte.iter().filter(|&&s| s == i).count();
                    card_facility(griglia, i, i, &art, dorata, possedute);
                }
            });
            r.spawn(Node {
                height: Val::Px(14.0),
                ..default()
            });
            voce(r, FACILITIES.len(), Azione::IndietroTitolo, "Indietro");
        });
}

pub fn esci_marketplace(mut commands: Commands, q: Query<Entity, With<SchermataMarketplace>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Livello completato ----------------

#[derive(Component)]
pub struct SchermataCompletato;

/// Obiettivo raggiunto: punteggio e tick della partita, e la strada avanti.
/// All'ultimo livello di campagna la voce "Livello successivo" non esiste;
/// nel livello casuale al suo posto c'è "Nuovo livello casuale".
#[allow(clippy::too_many_arguments)]
pub fn entra_completato(
    mut commands: Commands,
    sim: Res<Sim>,
    modalita: Res<Modalita>,
    casuale: Res<LivelloCasuale>,
    art: Res<Art>,
    medaglia: Res<crate::livelli::UltimaMedaglia>,
    bonus: Res<crate::livelli::UltimoBonus>,
    giornaliera: Res<crate::livelli::UltimaGiornaliera>,
    mut sel: ResMut<Selezione>,
) {
    let in_casuale = matches!(*modalita, Modalita::Casuale);
    let (intestazione, obiettivo) = match *modalita {
        Modalita::Campagna(i) => (
            format!("{}. {}", i + 1, LIVELLI[i].nome),
            LIVELLI[i].obiettivo,
        ),
        Modalita::Casuale => match &casuale.0 {
            Some(l) => (format!("Livello casuale — {}", l.nome), l.obiettivo),
            None => return, // non succede: lo stato arriva solo con un livello
        },
        // non succede: lo stato si raggiunge solo con un obiettivo attivo
        Modalita::Infinita | Modalita::Sfida => return,
    };
    let ultimo = matches!(*modalita, Modalita::Campagna(i) if i + 1 >= LIVELLI.len());
    *sel = Selezione {
        idx: 0,
        n: if ultimo { 1 } else { 2 },
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataCompletato,
        ))
        .with_children(|r| {
            r.spawn(testo("LIVELLO COMPLETATO", 34.0, VERDE));
            r.spawn(testo(intestazione, 15.0, METALLO));
            r.spawn((Node {
                margin: UiRect::top(Val::Px(12.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(
                    format!("Obiettivo raggiunto: {}", obiettivo.descrizione()),
                    14.0,
                    GIALLO,
                ));
            });
            r.spawn(testo(format!("Punteggio: {}", sim.punteggio), 22.0, BIANCO));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(format!("Tick: {}", sim.tick), 14.0, METALLO));
            });
            // la medaglia disegnata, e sotto le tre monete: accese quante ne
            // vale la medaglia (oro 3, argento 2, rame 1), le altre spente.
            // Le accese ruotano (anima_monete).
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
            },))
            .with_children(|c| {
                if let Some((presa, crediti)) = medaglia.0 {
                    let indice = match presa {
                        crate::progressi::ORO => 0,
                        crate::progressi::ARGENTO => 1,
                        _ => 2,
                    };
                    c.spawn((
                        ImageNode::new(art.medaglie[indice].clone()),
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                    ));
                    c.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|monete| {
                        let accese = presa as usize;
                        for i in 0..3 {
                            if i < accese {
                                monete.spawn((
                                    ImageNode::new(art.monete_accese[0].clone()),
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                    MonetaAnimata,
                                ));
                            } else {
                                monete.spawn((
                                    ImageNode::new(art.moneta_spenta.clone()),
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    });
                    if crediti > 0 {
                        c.spawn(testo(
                            format!("+{crediti} crediti per il Marketplace"),
                            13.0,
                            GIALLO,
                        ));
                    }
                }
            });
            // l'esito del bonus (solo campagna): il credito extra si vede
            // qui, dove si tirano le somme
            if let Some((quale, rispettato, nuovo)) = bonus.0 {
                let (riga, colore) = if nuovo {
                    (
                        format!("BONUS \u{2713} {} — +1 credito", quale.descrizione()),
                        VERDE,
                    )
                } else if rispettato {
                    (
                        format!("Bonus rispettato ({}) — già incassato", quale.descrizione()),
                        METALLO,
                    )
                } else {
                    (
                        format!("Bonus mancato: {}", quale.descrizione()),
                        GRIGIO_MEDIO,
                    )
                };
                r.spawn((Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|c| {
                    c.spawn(testo(riga, 14.0, colore));
                });
            }
            // l'esito della sfida del giorno: il record è la sua classifica
            if let Some((tick_run, migliore, record)) = giornaliera.0 {
                r.spawn((Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    ..default()
                },))
                .with_children(|c| {
                    if record {
                        c.spawn(testo(
                            format!(
                                "NUOVO RECORD DI OGGI: {}",
                                tick_in_tempo_menu(tick_run)
                            ),
                            15.0,
                            GIALLO,
                        ));
                    } else {
                        c.spawn(testo(
                            format!(
                                "Tempo: {} — il tuo migliore di oggi resta {}",
                                tick_in_tempo_menu(tick_run),
                                tick_in_tempo_menu(migliore)
                            ),
                            14.0,
                            METALLO,
                        ));
                    }
                });
            }
            // ai traguardi della campagna il personaggio di turno presenta
            // il modulo appena sbloccato (comparirà nella palette)
            if let Modalita::Campagna(i) = *modalita
                && let Some((personaggio, battuta)) = annuncio_sblocco(i + 1)
            {
                fumetto(r, &art, personaggio, battuta);
            }
            if ultimo {
                r.spawn((Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },))
                .with_children(|c| {
                    c.spawn(testo(
                        "Hai completato la campagna: la stazione è tua.",
                        15.0,
                        BIANCO,
                    ));
                });
                // il finale della storia chiude tutti gli archi
                let (personaggio, testo_finale) = crate::personaggi::FINALE;
                fumetto(r, &art, personaggio, testo_finale);
                voce(r, 0, Azione::IndietroTitolo, "Torna al titolo");
            } else if in_casuale {
                voce(r, 0, Azione::GiocaCasuale, "Nuovo livello casuale");
                voce(r, 1, Azione::IndietroTitolo, "Torna al titolo");
            } else {
                voce(r, 0, Azione::LivelloSuccessivo, "Livello successivo");
                voce(r, 1, Azione::IndietroTitolo, "Torna al titolo");
            }
        });
}

pub fn esci_completato(mut commands: Commands, q: Query<Entity, With<SchermataCompletato>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Pausa ----------------

/// L'overlay compare/sparisce seguendo `Pausa::aperta`, ma solo dentro la
/// partita: uscendo verso "Come si gioca" va nascosto e poi ricostruito.
pub fn sincronizza_pausa(
    mut commands: Commands,
    stato: Res<State<AppState>>,
    pausa: Res<Pausa>,
    imp: Res<Impostazioni>,
    mut sel: ResMut<Selezione>,
    q: Query<Entity, With<SchermataPausa>>,
) {
    let deve_esserci = pausa.aperta && *stato.get() == AppState::InGioco;
    let c_e = !q.is_empty();
    if deve_esserci == c_e {
        return;
    }
    if !deve_esserci {
        for e in &q {
            commands.entity(e).despawn();
        }
        return;
    }
    *sel = Selezione {
        idx: 0,
        n: 7,
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO.with_alpha(0.7)),
            GlobalZIndex(20),
            SchermataPausa,
        ))
        .with_children(|r| {
            r.spawn(testo("PAUSA", 34.0, BIANCO));
            r.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(18.0)),
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(testo("la simulazione è congelata", 13.0, GRIGIO_MEDIO));
            });
            voce(r, 0, Azione::Riprendi, "Riprendi");
            voce(r, 1, Azione::ComeSiGioca, "Come si gioca");
            voce(r, 2, Azione::CicloMusica, format!("Musica: {}%", imp.musica));
            voce(r, 3, Azione::CicloEffetti, format!("Effetti: {}%", imp.effetti));
            voce(r, 4, Azione::Ricomincia, "Ricomincia");
            voce(r, 5, Azione::TornaAlTitolo, "Torna al titolo");
            voce(r, 6, Azione::Esci, "Esci");
        });
}

// ---------------- Fine partita ----------------

/// Schermata "STAZIONE PERSA": overlay sopra la scena di gioco, come la
/// pausa. Ricomincia e Torna al titolo qui NON chiedono conferma: la
/// stazione è già persa, non c'è niente da proteggere.
/// In campagna la prima voce diventa "Riprova il livello"; in infinita, se
/// la partita è entrata in top 10, la schermata lo dice esplicitamente.
pub fn entra_fine(
    mut commands: Commands,
    sim: Res<Sim>,
    modalita: Res<Modalita>,
    casuale: Res<LivelloCasuale>,
    piazzamento: Res<UltimoPiazzamento>,
    mut sel: ResMut<Selezione>,
) {
    // "Riprova il livello" vale anche per il casuale: il livello resta in
    // `LivelloCasuale`, quindi il reset lo rigioca identico. Nel casuale
    // c'è anche la terza via: un livello nuovo di zecca.
    let con_livello = matches!(*modalita, Modalita::Campagna(_) | Modalita::Casuale);
    let in_casuale = matches!(*modalita, Modalita::Casuale);
    *sel = Selezione {
        idx: 0,
        n: if in_casuale { 3 } else { 2 },
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO.with_alpha(0.7)),
            GlobalZIndex(20),
            SchermataFine,
        ))
        .with_children(|r| {
            let (titolo, sottotitolo) = match sim.motivo_fine {
                Some(MotivoFine::TempoScaduto) => (
                    "TEMPO SCADUTO",
                    "tetto di tick raggiunto senza riuscire".to_string(),
                ),
                _ => ("STAZIONE PERSA", "tutto l'equipaggio è morto".to_string()),
            };
            r.spawn(testo(titolo, 34.0, ROSSO));
            r.spawn(testo(sottotitolo, 13.0, GRIGIO_MEDIO));
            match *modalita {
                Modalita::Campagna(i) => {
                    r.spawn(testo(
                        format!("livello {}. {} non superato", i + 1, LIVELLI[i].nome),
                        13.0,
                        GRIGIO_MEDIO,
                    ));
                }
                Modalita::Casuale => {
                    if let Some(l) = &casuale.0 {
                        r.spawn(testo(
                            format!("livello casuale — {} non superato", l.nome),
                            13.0,
                            GRIGIO_MEDIO,
                        ));
                    }
                }
                Modalita::Infinita | Modalita::Sfida => {}
            }
            r.spawn((Node {
                margin: UiRect::top(Val::Px(14.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(format!("Punteggio: {}", sim.punteggio), 22.0, BIANCO));
            });
            r.spawn(testo(
                format!("Tick sopravvissuti: {}", sim.tick),
                14.0,
                METALLO,
            ));
            r.spawn(testo(
                format!("Equipaggio massimo: {}", sim.equipaggio_max),
                14.0,
                METALLO,
            ));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(18.0)),
                ..default()
            },))
            .with_children(|c| {
                // solo in infinita/sfida: il punteggio è entrato in classifica
                if let Some(p) = piazzamento.0 {
                    let nome = if matches!(*modalita, Modalita::Sfida) {
                        "Sfida"
                    } else {
                        "Infinita"
                    };
                    c.spawn(testo(
                        format!("Nuovo record: {}º posto in classifica {nome}", p + 1),
                        15.0,
                        GIALLO,
                    ));
                }
            });
            if con_livello {
                voce(r, 0, Azione::Ricomincia, "Riprova il livello");
            } else {
                voce(r, 0, Azione::Ricomincia, "Ricomincia");
            }
            if in_casuale {
                voce(r, 1, Azione::GiocaCasuale, "Nuovo livello casuale");
                voce(r, 2, Azione::TornaAlTitolo, "Torna al titolo");
            } else {
                voce(r, 1, Azione::TornaAlTitolo, "Torna al titolo");
            }
        });
}

pub fn esci_fine(mut commands: Commands, q: Query<Entity, With<SchermataFine>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- navigazione comune ----------------

#[allow(clippy::too_many_arguments)]
pub fn naviga(
    tasti: Res<ButtonInput<KeyCode>>,
    mut sel: ResMut<Selezione>,
    mut pausa: ResMut<Pausa>,
    mut origine: ResMut<Origine>,
    mut reset: ResMut<RichiestaReset>,
    mut modalita: ResMut<Modalita>,
    mut scelto: ResMut<LivelloScelto>,
    mut casuale: ResMut<LivelloCasuale>,
    progressione: Res<Progressione>,
    mut sfida: ResMut<SfidaDelGiorno>,
    mut imp: ResMut<Impostazioni>,
    mut portafoglio: ResMut<Portafoglio>,
    stato: Res<State<AppState>>,
    mut prossimo: ResMut<NextState<AppState>>,
    mut esci: MessageWriter<AppExit>,
    voci: Query<&Voce>,
) {
    let attuale = *stato.get();
    let menu_attivo = attuale != AppState::InGioco || pausa.aperta;

    if tasti.just_pressed(KeyCode::Escape) {
        match attuale {
            AppState::InGioco => {
                if pausa.aperta {
                    if sel.conferma.is_some() {
                        sel.conferma = None; // Esc annulla prima la conferma
                    } else {
                        pausa.aperta = false;
                    }
                } else {
                    pausa.aperta = true;
                }
            }
            AppState::ComeSiGioca => {
                prossimo.set(origine.stato);
                pausa.aperta = origine.da_pausa;
            }
            AppState::SelezioneLivello
            | AppState::SchermataClassifica
            | AppState::Marketplace => {
                prossimo.set(AppState::Titolo);
            }
            AppState::Intermezzo => {
                prossimo.set(AppState::SelezioneLivello);
            }
            // A stazione persa (o a livello finito) Esc non ha scorciatoie:
            // si sceglie una voce.
            AppState::Titolo | AppState::FinePartita | AppState::LivelloCompletato => {}
        }
        return;
    }
    if !menu_attivo || sel.n == 0 {
        return;
    }

    // nella griglia dei livelli su/giù saltano di riga (10 celle),
    // sinistra/destra di una; negli altri menu su/giù scorrono le voci
    let salto = if attuale == AppState::SelezioneLivello {
        10
    } else {
        1
    };
    if tasti.just_pressed(KeyCode::ArrowUp) {
        sel.idx = (sel.idx + sel.n - salto.min(sel.n)) % sel.n;
        sel.conferma = None;
    }
    if tasti.just_pressed(KeyCode::ArrowDown) {
        sel.idx = (sel.idx + salto) % sel.n;
        sel.conferma = None;
    }
    if tasti.just_pressed(KeyCode::ArrowLeft) {
        sel.idx = (sel.idx + sel.n - 1) % sel.n;
        sel.conferma = None;
    }
    if tasti.just_pressed(KeyCode::ArrowRight) {
        sel.idx = (sel.idx + 1) % sel.n;
        sel.conferma = None;
    }
    if tasti.just_pressed(KeyCode::Enter) || tasti.just_pressed(KeyCode::NumpadEnter) {
        let idx = sel.idx;
        if let Some(v) = voci.iter().find(|v| v.idx == idx) {
            esegui(
                v.azione,
                idx,
                &mut sel,
                &mut pausa,
                &mut origine,
                &mut reset,
                &mut modalita,
                &mut scelto,
                &mut casuale,
                &mut sfida,
                &progressione,
                &mut imp,
                &mut portafoglio,
                attuale,
                &mut prossimo,
                &mut esci,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn click_voci(
    q: Query<(&Interaction, &Voce), Changed<Interaction>>,
    mut sel: ResMut<Selezione>,
    mut pausa: ResMut<Pausa>,
    mut origine: ResMut<Origine>,
    mut reset: ResMut<RichiestaReset>,
    mut modalita: ResMut<Modalita>,
    mut scelto: ResMut<LivelloScelto>,
    mut casuale: ResMut<LivelloCasuale>,
    progressione: Res<Progressione>,
    mut sfida: ResMut<SfidaDelGiorno>,
    mut imp: ResMut<Impostazioni>,
    mut portafoglio: ResMut<Portafoglio>,
    stato: Res<State<AppState>>,
    mut prossimo: ResMut<NextState<AppState>>,
    mut esci: MessageWriter<AppExit>,
) {
    for (interazione, v) in &q {
        match interazione {
            Interaction::Hovered => {
                if sel.idx != v.idx {
                    sel.idx = v.idx;
                    sel.conferma = None;
                }
            }
            Interaction::Pressed => {
                sel.idx = v.idx;
                let attuale = *stato.get();
                esegui(
                    v.azione,
                    v.idx,
                    &mut sel,
                    &mut pausa,
                    &mut origine,
                    &mut reset,
                    &mut modalita,
                    &mut scelto,
                    &mut casuale,
                    &mut sfida,
                    &progressione,
                    &mut imp,
                    &mut portafoglio,
                    attuale,
                    &mut prossimo,
                    &mut esci,
                );
            }
            Interaction::None => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn esegui(
    azione: Azione,
    idx: usize,
    sel: &mut Selezione,
    pausa: &mut Pausa,
    origine: &mut Origine,
    reset: &mut RichiestaReset,
    modalita: &mut Modalita,
    scelto: &mut LivelloScelto,
    casuale: &mut LivelloCasuale,
    sfida: &mut SfidaDelGiorno,
    progressione: &Progressione,
    imp: &mut Impostazioni,
    portafoglio: &mut Portafoglio,
    attuale: AppState,
    prossimo: &mut NextState<AppState>,
    esci: &mut MessageWriter<AppExit>,
) {
    // Prima attivazione di un'azione distruttiva: chiede conferma sulla voce.
    // A stazione persa non c'è niente da distruggere: nessuna conferma.
    if azione.distruttiva() && attuale != AppState::FinePartita && sel.conferma != Some(idx) {
        sel.conferma = Some(idx);
        return;
    }
    sel.conferma = None;
    match azione {
        Azione::ApriCampagna => prossimo.set(AppState::SelezioneLivello),
        Azione::GiocaInfinita => {
            *modalita = Modalita::Infinita;
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::GiocaSfida => {
            *modalita = Modalita::Sfida;
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::GiocaCasuale => {
            // il seed viene dal rand di sistema: qui la non-riproducibilità
            // è il punto; la campagna invece usa seed fissi (generatore.rs)
            casuale.0 = Some(generatore::genera_casuale(rand::rng().random::<u64>()));
            sfida.attiva = false;
            *modalita = Modalita::Casuale;
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::GiocaGiornaliera => {
            // stesso seed per tutti nel giorno: il confronto è col mondo
            // (e con se stessi: si rigioca per il miglior tempo)
            let giorno = giorno_corrente();
            casuale.0 = Some(generatore::genera_giornaliera(giorno));
            *sfida = SfidaDelGiorno {
                attiva: true,
                giorno,
            };
            *modalita = Modalita::Casuale;
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::ApriClassifica => prossimo.set(AppState::SchermataClassifica),
        Azione::ApriMarketplace => prossimo.set(AppState::Marketplace),
        Azione::CompraFacility(i) => {
            // se i crediti non bastano `compra` rifiuta e non succede nulla:
            // l'etichetta lo dice già ("crediti insufficienti")
            portafoglio.compra(i, FACILITIES[i].costo_crediti);
        }
        Azione::ScegliLivello(i) => {
            scelto.0 = i;
            // la storia si mostra solo la prima volta che si arriva al
            // livello; altrimenti dritti in partita: obiettivo e numeri li
            // dà il prologo sopra la griglia
            if progressione.completati == i && crate::personaggi::intermezzo_per(i + 1).is_some()
            {
                prossimo.set(AppState::Intermezzo);
            } else {
                *modalita = Modalita::Campagna(i);
                reset.0 = true;
                pausa.aperta = false;
                prossimo.set(AppState::InGioco);
            }
        }
        Azione::IniziaLivello => {
            *modalita = Modalita::Campagna(scelto.0);
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::LivelloSuccessivo => {
            // il livello corrente sta nella modalità: si passa al successivo,
            // con la stessa deviazione-intermezzo della selezione (i diari
            // di bordo vivono proprio sui livelli raggiunti così)
            if let Modalita::Campagna(i) = *modalita {
                scelto.0 = (i + 1).min(LIVELLI.len() - 1);
            }
            let i = scelto.0;
            if progressione.completati == i && crate::personaggi::intermezzo_per(i + 1).is_some()
            {
                prossimo.set(AppState::Intermezzo);
            } else {
                *modalita = Modalita::Campagna(i);
                reset.0 = true;
                pausa.aperta = false;
                prossimo.set(AppState::InGioco);
            }
        }
        Azione::ApriManuale => apri_da_assets("manuale.html"),
        Azione::ApriManualePdf => apri_da_assets("manuale.pdf"),
        Azione::ComeSiGioca => {
            origine.stato = if attuale == AppState::InGioco {
                AppState::InGioco
            } else {
                AppState::Titolo
            };
            origine.da_pausa = pausa.aperta;
            prossimo.set(AppState::ComeSiGioca);
        }
        Azione::Indietro => {
            prossimo.set(origine.stato);
            pausa.aperta = origine.da_pausa;
        }
        Azione::IndietroTitolo => prossimo.set(AppState::Titolo),
        Azione::Riprendi => pausa.aperta = false,
        Azione::CicloMusica => {
            imp.musica = ciclo(imp.musica);
            imp.salva();
        }
        Azione::CicloEffetti => {
            imp.effetti = ciclo(imp.effetti);
            imp.salva();
        }
        Azione::Ricomincia => {
            reset.0 = true;
            pausa.aperta = false;
            // dal menu di pausa si è già in gioco; da fine partita si rientra
            if attuale == AppState::FinePartita {
                prossimo.set(AppState::InGioco);
            }
        }
        Azione::TornaAlTitolo => {
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::Titolo);
        }
        Azione::Esci => {
            esci.write(AppExit::Success);
        }
    }
}

/// Evidenzia la voce selezionata e mostra la richiesta di conferma.
/// Moneta accesa nella schermata "livello completato": ruota ciclando i
/// 4 frame dello spin, tutte in sincrono (~8 fps).
#[derive(Component)]
pub struct MonetaAnimata;

pub fn anima_monete(
    tempo: Res<Time>,
    art: Res<Art>,
    mut monete: Query<&mut ImageNode, With<MonetaAnimata>>,
) {
    let frame = (tempo.elapsed_secs() * 8.0) as usize % art.monete_accese.len();
    for mut img in &mut monete {
        let nuovo = &art.monete_accese[frame];
        if img.image != *nuovo {
            img.image = nuovo.clone();
        }
    }
}

/// Tiene aggiornate le etichette dei volumi nel menu di pausa: si scrive
/// `Voce.etichetta` (non il testo) perché `evidenzia_voci` la ricopia a
/// ogni frame.
/// Etichetta della voce "Sfida del giorno": la spunta dice che il record
/// di oggi è già in cascina (si può sempre rigiocare per migliorarlo).
fn etichetta_giornaliera(portafoglio: &Portafoglio) -> String {
    if portafoglio.giornaliera_fatta(giorno_corrente()) {
        "Sfida del giorno ✓".into()
    } else {
        "Sfida del giorno".into()
    }
}

/// Tiene aggiornata la spunta della sfida del giorno quando il portafoglio
/// cambia (completamento della sfida mentre il titolo è ancora vivo).
pub fn aggiorna_voce_giornaliera(
    portafoglio: Res<Portafoglio>,
    mut voci: Query<&mut Voce>,
) {
    if !portafoglio.is_changed() {
        return;
    }
    for mut v in &mut voci {
        if v.azione == Azione::GiocaGiornaliera {
            v.etichetta = etichetta_giornaliera(&portafoglio);
        }
    }
}

pub fn aggiorna_voci_volume(imp: Res<Impostazioni>, mut voci: Query<&mut Voce>) {
    if !imp.is_changed() {
        return;
    }
    for mut v in &mut voci {
        match v.azione {
            Azione::CicloMusica => v.etichetta = format!("Musica: {}%", imp.musica),
            Azione::CicloEffetti => v.etichetta = format!("Effetti: {}%", imp.effetti),
            _ => {}
        }
    }
}

/// Dopo un acquisto nel Marketplace aggiorna saldo ed etichette del
/// catalogo: stessa tecnica di `aggiorna_voci_volume` (si scrive
/// `Voce.etichetta`, che `evidenzia_voci` ricopia sul testo a ogni frame);
/// il saldo non è una voce e si scrive direttamente.
pub fn aggiorna_voci_marketplace(
    mut commands: Commands,
    portafoglio: Res<Portafoglio>,
    mut card: Query<(Entity, &Voce, &mut StileCard, &mut BorderColor)>,
    mut icone: Query<(&IconaCard, &mut ImageNode)>,
    mut possedute: Query<(&PossedutePer, &mut Text), Without<SaldoCrediti>>,
    mut saldi: Query<&mut Text, With<SaldoCrediti>>,
) {
    if !portafoglio.is_changed() {
        return;
    }
    for (e, v, mut stile, mut bordo) in &mut card {
        if let Azione::CompraFacility(i) = v.azione {
            stile.dorata = portafoglio.crediti >= FACILITIES[i].costo_crediti;
            *bordo = BorderColor::all(if stile.dorata { GIALLO } else { GRIGIO_SCAFO });
            if stile.dorata {
                commands.entity(e).remove::<BottoneMuto>();
            } else {
                commands.entity(e).insert(BottoneMuto);
            }
        }
    }
    for (icona, mut img) in &mut icone {
        let dorata = portafoglio.crediti >= FACILITIES[icona.0].costo_crediti;
        img.color = if dorata { Color::WHITE } else { GRIGIO_MEDIO };
    }
    for (per, mut t) in &mut possedute {
        let n = portafoglio.scorte.iter().filter(|&&s| s == per.0).count();
        t.0 = if n > 0 {
            format!("ne hai {n}")
        } else {
            String::new()
        };
    }
    for mut t in &mut saldi {
        t.0 = format!("hai {} crediti", portafoglio.crediti);
    }
}

pub fn evidenzia_voci(
    sel: Res<Selezione>,
    mut voci: Query<(
        &Voce,
        &Children,
        &mut BackgroundColor,
        &mut BorderColor,
        Option<&StileCard>,
    )>,
    mut testi: Query<(&mut Text, &mut TextColor, Option<&ColoreFisso>)>,
) {
    for (v, figli, mut bg, mut bordo, card) in &mut voci {
        let scelta = v.idx == sel.idx;
        let in_conferma = sel.conferma == Some(v.idx);
        // le card del Marketplace hanno stile e figli propri (icona, monete):
        // qui si segnala solo la selezione col bordo, il resto non si tocca
        if let Some(card) = card {
            *bordo = BorderColor::all(if scelta {
                BIANCO
            } else if card.dorata {
                GIALLO
            } else {
                GRIGIO_SCAFO
            });
            continue;
        }
        bg.0 = if scelta { GRIGIO_SCAFO } else { Color::NONE };
        *bordo = BorderColor::all(if in_conferma {
            ROSSO
        } else if scelta {
            BIANCO
        } else {
            Color::NONE
        });
        for figlio in figli.iter() {
            if let Ok((mut t, mut c, fisso)) = testi.get_mut(figlio) {
                if in_conferma {
                    t.0 = "Sicuro? La stazione andrà persa — Invio conferma, Esc annulla".into();
                    c.0 = ROSSO;
                } else {
                    t.0 = v.etichetta.clone();
                    // le medaglie tengono il loro colore anche a riposo
                    c.0 = if scelta {
                        BIANCO
                    } else {
                        fisso.map(|f| f.0).unwrap_or(METALLO)
                    };
                }
            }
        }
    }
}

/// Sfondo delle schermate a tutto campo (evita che si veda la scena 2D dietro
/// il titolo, che resta viva). A fine partita la scena resta visibile sotto
/// l'overlay, quindi lo sfondo è quello di gioco.
pub fn colore_sfondo_menu(stato: Res<State<AppState>>, mut clear: ResMut<ClearColor>) {
    if stato.is_changed() {
        clear.0 = if matches!(*stato.get(), AppState::InGioco | AppState::FinePartita) {
            NERO
        } else {
            SCAFO_SCURO
        };
    }
}
