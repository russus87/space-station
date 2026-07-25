//! UI di gioco: HUD produzione/consumo, palette dei moduli, pannello ispezione
//! e registro eventi. Tutto in Bevy UI (`Node` + percentuali) ancorato ai bordi
//! della finestra: la PoC usava `Text2d` a coordinate mondo tarate su 1160×800
//! e al resize i testi si scollavano dalla scena.
//! Solo la griglia e i moduli restano in world-space (vedi `main.rs`).

use crate::livelli::{LIVELLI, LivelloCasuale, Modalita, Progressione, StatoLivello};
use crate::menu::{AppState, Pausa};
use crate::modules::{KINDS, ModuleKind};
use crate::sim::{
    EventLog, Fermo, Gravita, Module, O2_MAX, SOGLIA_O2_CRITICO, Sim,
    TICK_SURRISCALDAMENTO,
};
use crate::{Art, Selected, SottoCursore};
use bevy::prelude::*;

// --- palette (SPEC.md §2.2): 16 colori, nessun colore fuori da questa lista ---
pub const NERO: Color = Color::srgb_u8(0x0B, 0x0E, 0x14);
pub const SCAFO_SCURO: Color = Color::srgb_u8(0x1A, 0x1F, 0x2B);
pub const GRIGIO_SCAFO: Color = Color::srgb_u8(0x2E, 0x36, 0x44);
pub const GRIGIO_MEDIO: Color = Color::srgb_u8(0x54, 0x61, 0x70);
pub const METALLO: Color = Color::srgb_u8(0x8C, 0x99, 0xA8);
pub const BIANCO: Color = Color::srgb_u8(0xD8, 0xDE, 0xE6);
#[allow(dead_code)] // la palette resta completa: e' il riferimento per gli sprite
pub const ARANCIO: Color = Color::srgb_u8(0xF2, 0xA3, 0x3C);
pub const CIANO: Color = Color::srgb_u8(0x4F, 0xC3, 0xE8);
pub const VERDE: Color = Color::srgb_u8(0x57, 0xC2, 0x5B);
pub const GIALLO: Color = Color::srgb_u8(0xF2, 0xD2, 0x4B);
pub const ROSSO: Color = Color::srgb_u8(0xE8, 0x4C, 0x3D);
pub const ROSSO_SCURO: Color = Color::srgb_u8(0x7A, 0x23, 0x18);

const LOG_RIGHE: usize = 8;

/// Radice della UI di gioco: si spegne (Display::None) fuori da InGioco.
#[derive(Component)]
pub struct RadiceGioco;

/// Il tasto MENU nell'HUD: apre la pausa, come Esc.
#[derive(Component)]
pub struct BottoneMenu;

/// Il tasto MERCATO nell'HUD: apre il mercato, come M (vedi mercato.rs).
#[derive(Component)]
pub struct BottoneMercato;

#[derive(Component, Clone, Copy)]
pub enum CampoHud {
    Stato,
    EnergiaFlusso,
    EnergiaMargine,
    OssigenoFlusso,
    OssigenoRiserva,
    CaloreFlusso,
    CaloreNetto,
    EquipaggioFlusso,
    EquipaggioTotale,
    /// Obiettivo del livello e progresso ("equipaggio 5/8", "12/15 tick"):
    /// in campagna è l'informazione che dice al giocatore cosa sta facendo.
    /// In infinita resta vuoto.
    Obiettivo,
    Punteggio,
}

#[derive(Component)]
pub struct BarraOssigeno;

/// Slot della palette laterale; l'indice segue `KINDS` (vedi `tasto_slot`).
#[derive(Component)]
pub struct SlotPalette(pub usize);

/// La riga dei costi di uno slot: mostra i costi se il modulo è sbloccato,
/// la soglia di sblocco altrimenti (aggiornata da `update_palette`).
#[derive(Component)]
pub struct CostoSlot(pub usize);

#[derive(Component)]
pub struct RigaLog(pub usize);

#[derive(Component)]
pub enum CampoIspezione {
    Nome,
    Stato,
    Valori,
}

#[derive(Component)]
pub struct MiniaturaIspezione;

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

/// Un pannello dell'HUD: icona + nome, riga produzione/consumo, riga principale.
fn pannello_risorsa(
    p: &mut ChildSpawnerCommands,
    art: &Art,
    icona: usize,
    nome: &str,
    campo_flusso: CampoHud,
    campo_principale: CampoHud,
    barra: bool,
) {
    p.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
            min_width: Val::Px(180.0),
            row_gap: Val::Px(2.0),
            ..default()
        },
    ))
    .with_children(|c| {
        c.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|r| {
            r.spawn((
                ImageNode::new(art.icone[icona].clone()),
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
            ));
            r.spawn(testo(nome, 13.0, METALLO));
        });
        c.spawn((testo("", 12.0, GRIGIO_MEDIO), campo_flusso));
        c.spawn((testo("", 17.0, BIANCO), campo_principale));
        if barra {
            c.spawn((
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(GRIGIO_SCAFO),
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(VERDE),
                    BarraOssigeno,
                ));
            });
        }
    });
}

pub fn setup_ui(mut commands: Commands, art: Res<Art>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RadiceGioco,
        ))
        .with_children(|root| {
            // ---------- HUD (barra superiore) ----------
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(9.0),
                    min_height: Val::Px(52.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(SCAFO_SCURO),
            ))
            .with_children(|hud| {
                hud.spawn((
                    Node {
                        min_width: Val::Px(110.0),
                        padding: UiRect::horizontal(Val::Px(14.0)),
                        ..default()
                    },
                ))
                .with_children(|c| {
                    c.spawn((testo("PAUSA", 16.0, METALLO), CampoHud::Stato));
                });
                pannello_risorsa(
                    hud,
                    &art,
                    0,
                    "ENERGIA",
                    CampoHud::EnergiaFlusso,
                    CampoHud::EnergiaMargine,
                    false,
                );
                pannello_risorsa(
                    hud,
                    &art,
                    1,
                    "OSSIGENO",
                    CampoHud::OssigenoFlusso,
                    CampoHud::OssigenoRiserva,
                    true,
                );
                pannello_risorsa(
                    hud,
                    &art,
                    2,
                    "CALORE",
                    CampoHud::CaloreFlusso,
                    CampoHud::CaloreNetto,
                    false,
                );
                pannello_risorsa(
                    hud,
                    &art,
                    3,
                    "EQUIPAGGIO",
                    CampoHud::EquipaggioFlusso,
                    CampoHud::EquipaggioTotale,
                    false,
                );
                // obiettivo di livello (solo campagna): in alto e leggibile,
                // è ciò che dice al giocatore cosa sta facendo
                hud.spawn((
                    Node {
                        margin: UiRect::left(Val::Auto),
                        padding: UiRect::horizontal(Val::Px(14.0)),
                        ..default()
                    },
                ))
                .with_children(|c| {
                    c.spawn((testo("", 15.0, GIALLO), CampoHud::Obiettivo));
                });
                // punteggio: discreto, in fondo alla barra — è un numero che
                // si guarda a fine partita, non il protagonista dello schermo
                hud.spawn((
                    Node {
                        padding: UiRect::horizontal(Val::Px(14.0)),
                        ..default()
                    },
                ))
                .with_children(|c| {
                    c.spawn((testo("", 13.0, GRIGIO_MEDIO), CampoHud::Punteggio));
                });
                hud.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                        margin: UiRect::right(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(GRIGIO_SCAFO),
                    Button,
                    BottoneMercato,
                ))
                .with_children(|c| {
                    c.spawn(testo("MERCATO m", 13.0, METALLO));
                });
                // uscita sempre visibile: il menu esisteva solo dietro Esc e
                // al playtest nessuno l'ha trovato — un tasto a schermo è
                // l'unica affordance che non richiede di conoscere il comando
                hud.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                        margin: UiRect::right(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(GRIGIO_SCAFO),
                    Button,
                    BottoneMenu,
                ))
                .with_children(|c| {
                    c.spawn(testo("MENU esc", 13.0, METALLO));
                });
            });

            // ---------- corpo: colonna sinistra + area griglia ----------
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
            ))
            .with_children(|corpo| {
                corpo
                    .spawn((
                        Node {
                            width: Val::Percent(18.0),
                            min_width: Val::Px(190.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(8.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(SCAFO_SCURO),
                    ))
                    .with_children(|col| {
                        for (i, kind) in KINDS.iter().enumerate() {
                            slot_palette(col, &art, i, *kind);
                        }
                        // il pannello ispezione occupa il fondo della colonna
                        col.spawn((
                            Node {
                                margin: UiRect::top(Val::Auto),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(8.0)),
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                            BackgroundColor(NERO),
                        ))
                        .with_children(|isp| {
                            isp.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                            ))
                            .with_children(|r| {
                                r.spawn((
                                    ImageNode::new(Handle::default()),
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                    MiniaturaIspezione,
                                ));
                                r.spawn((testo("", 13.0, BIANCO), CampoIspezione::Nome));
                            });
                            isp.spawn((testo("", 12.0, METALLO), CampoIspezione::Stato));
                            isp.spawn((testo("", 12.0, GRIGIO_MEDIO), CampoIspezione::Valori));
                        });
                    });
                // area griglia: vuota, serve solo a riservare lo spazio
                corpo.spawn((Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    ..default()
                },));
            });

            // ---------- log eventi (barra inferiore) ----------
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(18.0),
                    min_height: Val::Px(150.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(SCAFO_SCURO),
            ))
            .with_children(|log| {
                for i in 0..LOG_RIGHE {
                    log.spawn((testo("", 13.0, METALLO), RigaLog(i)));
                }
            });
        });
}

/// Etichetta del tasto dello slot `i`: 1..6 per i base, poi 7 8 9 0 C.
fn tasto_slot(i: usize) -> &'static str {
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "C"][i]
}

fn slot_palette(col: &mut ChildSpawnerCommands, art: &Art, i: usize, kind: ModuleKind) {
    let def = kind.def();
    col.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(SCAFO_SCURO),
        BorderColor::all(Color::NONE),
        Button,
        SlotPalette(i),
    ))
    .with_children(|s| {
        s.spawn((
            ImageNode::new(art.moduli[i].clone()),
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..default()
            },
        ));
        s.spawn((Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },))
            .with_children(|t| {
                t.spawn(testo(
                    format!("{}  {}", tasto_slot(i), def.nome),
                    12.0,
                    BIANCO,
                ));
                // il contenuto vero lo scrive `update_palette` a ogni frame:
                // costi se sbloccato, soglia di sblocco se ancora no
                t.spawn((testo("", 11.0, GRIGIO_MEDIO), CostoSlot(i)));
            });
    });
}

/// Riga di costi/produzioni del modulo, solo i valori non nulli: è ciò che
/// permette di sapere quanto costa un modulo *prima* di piazzarlo.
fn costi_compatti(kind: ModuleKind) -> String {
    let d = kind.def();
    let mut parti = Vec::new();
    if d.energia != 0.0 {
        parti.push(format!("En {:+.0}", d.energia));
    }
    if d.ossigeno != 0.0 {
        parti.push(format!("O2 {:+.0}", d.ossigeno));
    }
    if d.calore != 0.0 {
        parti.push(format!("Cal {:+.0}", d.calore));
    }
    if d.posti_letto > 0 {
        parti.push(format!("+{} posti", d.posti_letto));
    }
    if d.equipaggio_richiesto > 0 {
        parti.push(format!("serve {}", d.equipaggio_richiesto));
    }
    parti.join(" · ")
}

/// La UI di gioco esiste sempre ma è visibile solo in partita.
pub fn visibilita_gioco(
    stato: Res<State<AppState>>,
    mut q: Query<&mut Node, With<RadiceGioco>>,
) {
    if !stato.is_changed() {
        return;
    }
    // A fine partita la UI resta visibile sotto l'overlay, come in pausa.
    for mut n in &mut q {
        n.display = if matches!(*stato.get(), AppState::InGioco | AppState::FinePartita) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_hud(
    tempo: Res<Time>,
    sim: Res<Sim>,
    pausa: Res<Pausa>,
    modalita: Res<Modalita>,
    casuale: Res<LivelloCasuale>,
    stato_livello: Res<StatoLivello>,
    moduli: Query<(), With<Module>>,
    mut q: Query<(&CampoHud, &mut Text, &mut TextColor)>,
    mut barra: Query<(&mut Node, &mut BackgroundColor), With<BarraOssigeno>>,
) {
    // stesso ritmo del badge avaria in `aggiorna_visuali`
    let lampeggia = (tempo.elapsed_secs() * 2.0) as i32 % 2 == 0;
    let col_energia = if sim.in_blackout {
        ROSSO
    } else if sim.energia_margine < 20.0 {
        GIALLO
    } else {
        VERDE
    };
    let col_o2 = if sim.ossigeno <= SOGLIA_O2_CRITICO {
        ROSSO
    } else if sim.o2_delta < 0.0 {
        GIALLO
    } else {
        VERDE
    };
    let col_calore = if sim.surriscaldamento >= TICK_SURRISCALDAMENTO / 2 {
        ROSSO
    } else if sim.calore_netto > 0.0 {
        GIALLO
    } else {
        VERDE
    };
    let col_crew = if sim.ossigeno <= 0.0 && sim.equipaggio > 0 {
        ROSSO
    } else if sim.equipaggio < sim.posti_letto {
        GIALLO
    } else {
        VERDE
    };

    for (campo, mut t, mut c) in &mut q {
        match campo {
            CampoHud::Stato => {
                if pausa.aperta {
                    t.0 = "MENU".into();
                    c.0 = METALLO;
                } else if sim.running {
                    t.0 = match sim.tetto_tick {
                        Some(tetto) => format!("TICK {}/{tetto}", sim.tick),
                        None => format!("TICK {}", sim.tick),
                    };
                    c.0 = BIANCO;
                } else if sim.tick == 0 && !moduli.is_empty() {
                    // playtest: a stazione costruita, "PAUSA" non spiegava il
                    // passo successivo. Prima del primo avvio il campo diventa
                    // un invito esplicito, lampeggiante come il badge avaria.
                    t.0 = "PREMI SPAZIO PER AVVIARE".into();
                    c.0 = if lampeggia { GIALLO } else { GIALLO.with_alpha(0.0) };
                } else {
                    t.0 = "PAUSA".into();
                    c.0 = METALLO;
                }
            }
            CampoHud::EnergiaFlusso => {
                t.0 = if sim.batterie_capienza > 0.0 {
                    format!(
                        "prod {:.0} · cons {:.0} · batt {:.0}/{:.0}",
                        sim.energia_prod, sim.energia_cons, sim.batterie_carica, sim.batterie_capienza
                    )
                } else {
                    format!("prod {:.0} · cons {:.0}", sim.energia_prod, sim.energia_cons)
                };
                c.0 = GRIGIO_MEDIO;
            }
            CampoHud::EnergiaMargine => {
                // In blackout il margine resta positivo (è il pool avanzato,
                // insufficiente per il modulo successivo): si mostrano
                // entrambi, senza "aggiustare" il numero.
                t.0 = if sim.in_blackout {
                    format!("BLACKOUT  {:+.0}/t", sim.energia_margine)
                } else {
                    format!("{:+.0}/t", sim.energia_margine)
                };
                c.0 = col_energia;
            }
            CampoHud::OssigenoFlusso => {
                t.0 = format!("prod {:.0} · cons {:.0}", sim.o2_prod, sim.o2_cons);
                c.0 = GRIGIO_MEDIO;
            }
            CampoHud::OssigenoRiserva => {
                t.0 = format!("{:.0}/{:.0}  ({:+.0}/t)", sim.ossigeno, O2_MAX, sim.o2_delta);
                c.0 = col_o2;
            }
            CampoHud::CaloreFlusso => {
                t.0 = format!("prod {:.0} · diss {:.0}", sim.calore_prod, sim.calore_diss);
                c.0 = GRIGIO_MEDIO;
            }
            CampoHud::CaloreNetto => {
                t.0 = if sim.surriscaldamento > 0 {
                    format!(
                        "{:+.0}/t  avaria tra {}",
                        sim.calore_netto,
                        TICK_SURRISCALDAMENTO - sim.surriscaldamento
                    )
                } else {
                    format!("{:+.0}/t", sim.calore_netto)
                };
                c.0 = col_calore;
            }
            CampoHud::EquipaggioFlusso => {
                t.0 = format!(
                    "a bordo {} · posti {}",
                    sim.equipaggio, sim.posti_letto
                );
                c.0 = GRIGIO_MEDIO;
            }
            CampoHud::EquipaggioTotale => {
                t.0 = if sim.lab_fabbisogno > 0 {
                    format!(
                        "{}/{}  lab servono {}",
                        sim.equipaggio, sim.posti_letto, sim.lab_fabbisogno
                    )
                } else {
                    format!("{}/{}", sim.equipaggio, sim.posti_letto)
                };
                c.0 = col_crew;
            }
            CampoHud::Obiettivo => {
                let livello = match *modalita {
                    Modalita::Campagna(i) => Some(&LIVELLI[i]),
                    Modalita::Casuale => casuale.0.as_ref(),
                    Modalita::Infinita | Modalita::Sfida => None,
                };
                match livello {
                    Some(l) => {
                        t.0 = format!(
                            "{}   moduli {}/{}",
                            l.obiettivo.progresso(&sim, &stato_livello),
                            moduli.iter().count(),
                            l.max_moduli
                        );
                        c.0 = if stato_livello.completato { VERDE } else { GIALLO };
                    }
                    None => t.0.clear(),
                }
            }
            CampoHud::Punteggio => {
                t.0 = format!("PUNTI {}", sim.punteggio);
                c.0 = GRIGIO_MEDIO;
            }
        }
    }

    for (mut n, mut bg) in &mut barra {
        n.width = Val::Percent(sim.ossigeno / O2_MAX * 100.0);
        bg.0 = col_o2;
    }
}

pub fn update_palette(
    sel: Res<Selected>,
    progressione: Res<Progressione>,
    mut slot_q: Query<(&SlotPalette, &mut BackgroundColor, &mut BorderColor)>,
    mut costi_q: Query<(&CostoSlot, &mut Text, &mut TextColor)>,
) {
    for (slot, mut bg, mut bordo) in &mut slot_q {
        let bloccato = progressione.completati < KINDS[slot.0].def().sblocco;
        let scelto = !bloccato && KINDS[slot.0] == sel.0;
        bg.0 = if scelto {
            GRIGIO_SCAFO
        } else if bloccato {
            NERO
        } else {
            SCAFO_SCURO
        };
        *bordo = BorderColor::all(if scelto { BIANCO } else { Color::NONE });
    }
    for (costo, mut t, mut c) in &mut costi_q {
        let def = KINDS[costo.0].def();
        if progressione.completati < def.sblocco {
            t.0 = format!("si sblocca al livello {}", def.sblocco);
            c.0 = GRIGIO_SCAFO;
        } else {
            t.0 = costi_compatti(KINDS[costo.0]);
            c.0 = GRIGIO_MEDIO;
        }
    }
}

/// Click su uno slot = stessa cosa del tasto: gli slot bloccati non
/// rispondono.
pub fn click_palette(
    q: Query<(&Interaction, &SlotPalette), Changed<Interaction>>,
    progressione: Res<Progressione>,
    mut sel: ResMut<Selected>,
) {
    for (interazione, slot) in &q {
        if *interazione == Interaction::Pressed
            && progressione.completati >= KINDS[slot.0].def().sblocco
        {
            sel.0 = KINDS[slot.0];
        }
    }
}

/// Il tasto MENU dell'HUD apre l'overlay di pausa, identico a Esc.
pub fn click_bottone_menu(
    q: Query<&Interaction, (Changed<Interaction>, With<BottoneMenu>)>,
    mut pausa: ResMut<Pausa>,
) {
    for interazione in &q {
        if *interazione == Interaction::Pressed {
            pausa.aperta = true;
        }
    }
}

pub fn update_log(
    log: Res<EventLog>,
    mut q: Query<(&RigaLog, &mut Text, &mut TextColor)>,
) {
    if !log.is_changed() {
        return;
    }
    let righe = log.ultimi(LOG_RIGHE);
    for (idx, mut t, mut c) in &mut q {
        match righe.get(idx.0) {
            Some(r) => {
                t.0 = format!("T{:>4}  {}", r.tick, r.testo);
                c.0 = match r.gravita {
                    Gravita::Allarme => ROSSO,
                    Gravita::Avviso => GIALLO,
                    Gravita::Info => METALLO,
                };
            }
            None => t.0.clear(),
        }
    }
}

/// Pannello ispezione: risponde a "cos'è questo e perché è fermo" per il
/// modulo sotto il cursore.
pub fn update_ispezione(
    sotto: Res<SottoCursore>,
    art: Res<Art>,
    moduli: Query<&Module>,
    mut testi: Query<(&CampoIspezione, &mut Text, &mut TextColor)>,
    mut mini: Query<(&mut ImageNode, &mut Node), With<MiniaturaIspezione>>,
) {
    let modulo = sotto.0.and_then(|e| moduli.get(e).ok());
    for (campo, mut t, mut c) in &mut testi {
        match (campo, modulo) {
            (CampoIspezione::Nome, Some(m)) => {
                t.0 = m.etichetta.clone();
                c.0 = BIANCO;
            }
            (CampoIspezione::Stato, Some(m)) => {
                let (s, col) = match m.motivo_fermo() {
                    None => ("ATTIVO".to_string(), VERDE),
                    Some(Fermo::Energia) => ("SENZA ENERGIA".to_string(), GIALLO),
                    Some(Fermo::Scollegato) => (
                        "SCOLLEGATO — nessun reattore collegato".to_string(),
                        GIALLO,
                    ),
                    Some(Fermo::Equipaggio) => (
                        format!(
                            "SENZA EQUIPAGGIO (servono {})",
                            m.kind.def().equipaggio_richiesto
                        ),
                        GIALLO,
                    ),
                    Some(Fermo::Avaria) => {
                        ("IN AVARIA — premi R per riparare".to_string(), ROSSO)
                    }
                };
                t.0 = s;
                c.0 = col;
            }
            (CampoIspezione::Valori, Some(m)) => {
                t.0 = costi_compatti(m.kind);
                c.0 = GRIGIO_MEDIO;
            }
            (CampoIspezione::Nome, None) => {
                t.0 = "—".into();
                c.0 = GRIGIO_MEDIO;
            }
            (_, None) => t.0.clear(),
        }
    }
    for (mut img, mut n) in &mut mini {
        match modulo {
            Some(m) => {
                img.image = art.moduli[m.kind.index()].clone();
                n.width = Val::Px(24.0);
            }
            None => {
                img.image = Handle::default();
                n.width = Val::Px(0.0);
            }
        }
    }
}
