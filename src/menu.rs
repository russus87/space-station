//! Schermate di menu: Titolo, "Come si gioca", overlay di pausa e fine partita.
//! Due pause diverse e volutamente distinte: `Spazio` ferma la simulazione ma
//! lascia costruire (l'HUD mostra l'anteprima del bilancio), `Esc` apre questo
//! menu e congela tutto, timer del tick compreso.

use crate::generatore;
use crate::livelli::{
    ClassificaInfinita, ClassificaSfida, LIVELLI, LivelloCasuale, LivelloScelto, Modalita,
    Progressione, Record, UltimoPiazzamento, giorni_fa,
};
use rand::RngExt;
use crate::modules::{KINDS, TABELLA};
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
    /// Nome, briefing e obiettivo del livello scelto, prima di cominciare.
    Briefing,
    /// Top 10 delle partite in modalità infinita.
    SchermataClassifica,
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
    ApriClassifica,
    /// Dalla selezione livello al briefing del livello i (0-based).
    ScegliLivello(usize),
    /// Dal briefing alla partita, in modalità campagna.
    IniziaLivello,
    /// Da "livello completato" al briefing del livello successivo.
    LivelloSuccessivo,
    ComeSiGioca,
    Riprendi,
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

/// Variante compatta di `voce` per la griglia dei livelli: una cella
/// quadrata col numero, stessi componenti e stessi sistemi di navigazione.
fn voce_cella(p: &mut ChildSpawnerCommands, idx: usize, azione: Azione, etichetta: String) {
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
        c.spawn(testo(etichetta, 16.0, METALLO));
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

pub fn entra_titolo(mut commands: Commands, mut sel: ResMut<Selezione>) {
    *sel = Selezione {
        idx: 0,
        n: 7,
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
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
            voce(r, 4, Azione::ApriClassifica, "Classifica");
            voce(r, 5, Azione::ComeSiGioca, "Come si gioca");
            voce(r, 6, Azione::Esci, "Esci");
        });
}

pub fn esci_titolo(mut commands: Commands, q: Query<Entity, With<SchermataTitolo>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------- Come si gioca ----------------

pub fn entra_guida(mut commands: Commands, art: Res<Art>, mut sel: ResMut<Selezione>) {
    *sel = Selezione {
        idx: 0,
        n: 1,
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

            // i sei moduli
            r.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(14.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ))
            .with_children(|riga| {
                for (i, kind) in KINDS.iter().enumerate() {
                    let def = &TABELLA[kind.index()];
                    riga.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(3.0),
                            width: Val::Px(120.0),
                            ..default()
                        },
                    ))
                    .with_children(|c| {
                        c.spawn((
                            ImageNode::new(art.moduli[i].clone()),
                            Node {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                ..default()
                            },
                        ));
                        c.spawn(testo(def.nome, 12.0, BIANCO));
                        c.spawn(testo(
                            format!("En {:+.0} · Cal {:+.0}", def.energia, def.calore),
                            11.0,
                            GRIGIO_MEDIO,
                        ));
                    });
                }
            });

            for riga in [
                "1-6  scegli il modulo          click sinistro  piazza",
                "click destro  rimuove          R  ripara un modulo in avaria",
                "Spazio  avvia/ferma la simulazione (da fermo costruisci senza conseguenze)",
                "Esc  apre il menu e congela tutto",
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
            voce(r, 0, Azione::Indietro, "Indietro");
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
                        "50 livelli in ordine — completati {} · frecce per muoverti, Invio per il briefing",
                        progressione.completati
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
                        voce_cella(griglia, i, Azione::ScegliLivello(i), etichetta);
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

// ---------------- Briefing ----------------

#[derive(Component)]
pub struct SchermataBriefing;

/// Nome, briefing e obiettivo per esteso, prima di cominciare il livello.
pub fn entra_briefing(
    mut commands: Commands,
    scelto: Res<LivelloScelto>,
    mut sel: ResMut<Selezione>,
) {
    let l = &LIVELLI[scelto.0];
    *sel = Selezione {
        idx: 0,
        n: 2,
        conferma: None,
    };
    commands
        .spawn((
            radice_centrata(),
            BackgroundColor(NERO),
            GlobalZIndex(10),
            SchermataBriefing,
        ))
        .with_children(|r| {
            r.spawn(testo(format!("LIVELLO {}", scelto.0 + 1), 15.0, GRIGIO_MEDIO));
            r.spawn(testo(l.nome.clone(), 34.0, BIANCO));
            r.spawn((Node {
                margin: UiRect::top(Val::Px(12.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(l.briefing.clone(), 15.0, METALLO));
            });
            r.spawn(testo(
                format!("Obiettivo: {}", l.obiettivo.descrizione()),
                15.0,
                GIALLO,
            ));
            r.spawn((Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(
                    format!("Moduli disponibili: {} (corridoi inclusi)", l.max_moduli),
                    14.0,
                    CIANO,
                ));
            });
            voce(r, 0, Azione::IniziaLivello, "Inizia");
            voce(r, 1, Azione::ApriCampagna, "Indietro");
        });
}

pub fn esci_briefing(mut commands: Commands, q: Query<Entity, With<SchermataBriefing>>) {
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

// ---------------- Livello completato ----------------

#[derive(Component)]
pub struct SchermataCompletato;

/// Obiettivo raggiunto: punteggio e tick della partita, e la strada avanti.
/// All'ultimo livello di campagna la voce "Livello successivo" non esiste;
/// nel livello casuale al suo posto c'è "Nuovo livello casuale".
pub fn entra_completato(
    mut commands: Commands,
    sim: Res<Sim>,
    modalita: Res<Modalita>,
    casuale: Res<LivelloCasuale>,
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
                margin: UiRect::bottom(Val::Px(18.0)),
                ..default()
            },))
            .with_children(|c| {
                c.spawn(testo(format!("Tick: {}", sim.tick), 14.0, METALLO));
            });
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
        n: 5,
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
            voce(r, 2, Azione::Ricomincia, "Ricomincia");
            voce(r, 3, Azione::TornaAlTitolo, "Torna al titolo");
            voce(r, 4, Azione::Esci, "Esci");
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
    *sel = Selezione {
        idx: 0,
        n: 2,
        conferma: None,
    };
    // "Riprova il livello" vale anche per il casuale: il livello resta in
    // `LivelloCasuale`, quindi il reset lo rigioca identico
    let con_livello = matches!(*modalita, Modalita::Campagna(_) | Modalita::Casuale);
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
            voce(r, 1, Azione::TornaAlTitolo, "Torna al titolo");
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
            AppState::SelezioneLivello | AppState::SchermataClassifica => {
                prossimo.set(AppState::Titolo);
            }
            AppState::Briefing => prossimo.set(AppState::SelezioneLivello),
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
            *modalita = Modalita::Casuale;
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::ApriClassifica => prossimo.set(AppState::SchermataClassifica),
        Azione::ScegliLivello(i) => {
            scelto.0 = i;
            prossimo.set(AppState::Briefing);
        }
        Azione::IniziaLivello => {
            *modalita = Modalita::Campagna(scelto.0);
            reset.0 = true;
            pausa.aperta = false;
            prossimo.set(AppState::InGioco);
        }
        Azione::LivelloSuccessivo => {
            // il livello corrente sta nella modalità: si passa al successivo
            if let Modalita::Campagna(i) = *modalita {
                scelto.0 = (i + 1).min(LIVELLI.len() - 1);
            }
            prossimo.set(AppState::Briefing);
        }
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
pub fn evidenzia_voci(
    sel: Res<Selezione>,
    mut voci: Query<(&Voce, &Children, &mut BackgroundColor, &mut BorderColor)>,
    mut testi: Query<(&mut Text, &mut TextColor)>,
) {
    for (v, figli, mut bg, mut bordo) in &mut voci {
        let scelta = v.idx == sel.idx;
        let in_conferma = sel.conferma == Some(v.idx);
        bg.0 = if scelta { GRIGIO_SCAFO } else { Color::NONE };
        *bordo = BorderColor::all(if in_conferma {
            ROSSO
        } else if scelta {
            BIANCO
        } else {
            Color::NONE
        });
        for figlio in figli.iter() {
            if let Ok((mut t, mut c)) = testi.get_mut(figlio) {
                if in_conferma {
                    t.0 = "Sicuro? La stazione andrà persa — Invio conferma, Esc annulla".into();
                    c.0 = ROSSO;
                } else {
                    t.0 = v.etichetta.clone();
                    c.0 = if scelta { BIANCO } else { METALLO };
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
