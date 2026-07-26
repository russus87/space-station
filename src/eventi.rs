//! Eventi con scelta: il bivio che trasforma l'osservatore in comandante
//! (SPEC-CAMPAGNA §9.1).
//!
//! Rari per disegno (1-2 a partita): grazia iniziale, cooldown lungo, mai
//! durante un imprevisto (in preavviso, in corso o con la sirena ancora
//! accesa — `Imprevisti::tranquillo`), mai nei livelli tutorial 1-7, e
//! ogni evento al massimo UNA volta per partita. Quando scatta, la
//! simulazione SI CONGELA (`EventoAperto`, che main.rs usa come run
//! condition su `sim_tick`): il bivio si decide con calma, non sotto
//! pressione — la pressione è il mestiere degli imprevisti.
//!
//! Il patto col giocatore è esplicito: ogni opzione dichiara la sua
//! conseguenza PRIMA del click, in piccolo sotto la scelta. Niente
//! sorprese nascoste; le opzioni impossibili (comprare senza punti,
//! scaricare batterie che non hai) sono spente col motivo scritto.
//!
//! La macchina a stati è deterministica e testabile: la casualità entra
//! solo dall'esterno, come in imprevisti.rs.

use crate::Art;
use crate::audio::{self, BottoneMuto, Suoni};
use crate::impostazioni::Impostazioni;
use crate::imprevisti::Imprevisti;
use crate::livelli::Modalita;
use crate::menu::{AppState, Pausa};
use crate::modules::ModuleKind;
use crate::personaggi::PERSONAGGI;
use crate::progressi::Portafoglio;
use crate::sim::{EventLog, Gravita, Module, O2_MAX, Sim};
use crate::ui::{
    BIANCO, CIANO, GIALLO, GRIGIO_MEDIO, GRIGIO_SCAFO, METALLO, NERO, SCAFO_SCURO,
};
use bevy::prelude::*;
use rand::RngExt;

/// Nessun evento prima di questo tick.
pub const TICK_GRAZIA: u64 = 60;
/// Tick minimi tra un evento chiuso e il prossimo.
pub const TICK_COOLDOWN: u64 = 80;
/// Probabilità di aprire un bivio a ogni tick eleggibile: 1 su N.
const UNO_SU: u32 = 90;
/// In campagna gli eventi partono da questo livello (1-based), come gli
/// imprevisti: i primi sette livelli sono scuola guida.
const PRIMO_LIVELLO: usize = 8;

// ---------------- il catalogo ----------------

struct OpzioneDef {
    /// Testo del bottone (senza il numero: lo aggiunge la UI).
    scelta: &'static str,
    /// La conseguenza dichiarata, mostrata sotto la scelta.
    conseguenza: &'static str,
}

struct EventoDef {
    titolo: &'static str,
    /// Indice in `personaggi::PERSONAGGI` di chi presenta il bivio.
    presentatore: usize,
    /// La situazione, in voce, dentro il balloon.
    situazione: &'static str,
    opzioni: [OpzioneDef; 2],
}

const EVENTI: [EventoDef; 5] = [
    EventoDef {
        titolo: "Nave in avaria chiede attracco",
        presentatore: 4, // Ilse
        situazione: "Cargo civile, motori spenti, tre a bordo. Chiedono \
                     attracco. Il regolamento dice no; il regolamento non è \
                     mai stato là fuori.",
        opzioni: [
            OpzioneDef {
                scelta: "Autorizza l'attracco",
                conseguenza: "+3 equipaggio subito, anche oltre i posti letto",
            },
            OpzioneDef {
                scelta: "Rifiuta",
                conseguenza: "+20 punti: il regolamento è regolamento",
            },
        ],
    },
    EventoDef {
        titolo: "Mercante di passaggio",
        presentatore: 2, // Dario
        situazione: "Un mercante. Bombole d'ossigeno \u{201C}quasi nuove\u{201D}. \
                     Conosco il tipo: metà fregatura, metà salvezza.",
        opzioni: [
            OpzioneDef {
                scelta: "Compra le bombole",
                conseguenza: "\u{2212}30 punti, riserva d'ossigeno al massimo",
            },
            OpzioneDef {
                scelta: "Lascia perdere",
                conseguenza: "nessun effetto",
            },
        ],
    },
    EventoDef {
        titolo: "Sciame in rotta di collisione",
        presentatore: 0, // Vera
        situazione: "Sciame in rotta. Posso friggere i condensatori e \
                     deviarlo, o incrociamo le dita. Le dita non sono uno \
                     strumento di lavoro.",
        opzioni: [
            OpzioneDef {
                scelta: "Scarica i condensatori",
                conseguenza: "le batterie si svuotano, lo sciame è deviato",
            },
            OpzioneDef {
                scelta: "Rischia",
                conseguenza: "50%: un modulo a caso in avaria",
            },
        ],
    },
    EventoDef {
        titolo: "Richiesta del centro: straordinari",
        presentatore: 3, // Mira
        situazione: "Il centro chiede numeri extra entro stasera. Posso \
                     spremere i sistemi: scaldano, ma i numeri arrivano.",
        opzioni: [
            OpzioneDef {
                scelta: "Accetta gli straordinari",
                conseguenza: "+40 punti, +2 di surriscaldamento subito",
            },
            OpzioneDef {
                scelta: "Rifiuta",
                conseguenza: "nessun effetto: la salute prima",
            },
        ],
    },
    EventoDef {
        titolo: "Clandestino a bordo",
        presentatore: 1, // Tomas
        situazione: "Ho trovato un passeggero non in lista, nascosto nel \
                     vano filtri. Respira. È già più di quanto chieda il \
                     modulo di sbarco.",
        opzioni: [
            OpzioneDef {
                scelta: "Tienilo a bordo",
                conseguenza: "+1 equipaggio",
            },
            OpzioneDef {
                scelta: "Consegnalo alla prossima nave",
                conseguenza: "+1 credito per il Marketplace",
            },
        ],
    },
];

// ---------------- la macchina ----------------

/// C'è un bivio a schermo: main.rs la usa come run condition per congelare
/// `sim_tick` (e per spegnere gli input di costruzione).
#[derive(Resource, Default)]
pub struct EventoAperto(pub bool);

/// Run condition già pronta: la simulazione avanza solo senza bivi aperti.
pub fn nessun_evento(aperto: Res<EventoAperto>) -> bool {
    !aperto.0
}

/// Lo stato della macchina degli eventi. Come `Imprevisti`: vive per tutta
/// la partita e si riarma quando il tick torna indietro.
#[derive(Resource, Default)]
pub struct Eventi {
    corrente: Option<usize>,
    usati: [bool; EVENTI.len()],
    cooldown_fino: u64,
    ultimo_tick: u64,
}

impl Eventi {
    fn riarma(&mut self) {
        *self = Eventi::default();
    }

    /// L'indice dell'evento a schermo, se un bivio è aperto.
    #[allow(dead_code)] // API d'ispezione per test/debug
    pub fn corrente(&self) -> Option<usize> {
        self.corrente
    }

    /// Decide se aprire un bivio al tick corrente. `estrai(n)` è l'unica
    /// fonte di casualità (uniforme in [0, n)). Ritorna l'evento estratto.
    fn pianifica(&mut self, tick: u64, mut estrai: impl FnMut(u32) -> u32) -> Option<usize> {
        let liberi: Vec<usize> = (0..EVENTI.len()).filter(|&i| !self.usati[i]).collect();
        if self.corrente.is_some()
            || liberi.is_empty()
            || tick < TICK_GRAZIA
            || tick < self.cooldown_fino
            || estrai(UNO_SU) != 0
        {
            return None;
        }
        let scelto = liberi[estrai(liberi.len() as u32) as usize];
        self.usati[scelto] = true;
        self.corrente = Some(scelto);
        Some(scelto)
    }

    fn chiudi(&mut self, tick: u64) {
        self.corrente = None;
        self.cooldown_fino = tick + TICK_COOLDOWN;
    }
}

/// Gli eventi sono ammessi in questa modalità/livello? (Come imprevisti.)
fn ammessi(modalita: &Modalita) -> bool {
    match modalita {
        Modalita::Campagna(i) => i + 1 >= PRIMO_LIVELLO,
        _ => true,
    }
}

/// Perché un'opzione NON è selezionabile adesso; `None` = disponibile.
fn non_disponibile(
    evento: usize,
    opzione: usize,
    sim: &Sim,
    carica_batterie: f32,
) -> Option<&'static str> {
    match (evento, opzione) {
        (1, 0) if sim.punteggio < 30 => Some("non hai 30 punti"),
        (2, 0) if carica_batterie <= 0.0 => Some("non hai batterie cariche"),
        _ => None,
    }
}

// ---------------- pianificazione ----------------

/// Apre i bivi: una volta per tick effettivo, solo in acque tranquille.
#[allow(clippy::too_many_arguments)]
pub fn pianifica(
    mut commands: Commands,
    mut stato: ResMut<Eventi>,
    mut aperto: ResMut<EventoAperto>,
    sim: Res<Sim>,
    imprevisti: Res<Imprevisti>,
    modalita: Res<Modalita>,
    suoni: Option<Res<Suoni>>,
    imp: Res<Impostazioni>,
) {
    // riarmo su reset: il tick torna indietro a partita nuova
    if sim.tick < stato.ultimo_tick {
        stato.riarma();
        aperto.0 = false;
    }
    let tick_nuovo = sim.tick != stato.ultimo_tick;
    stato.ultimo_tick = sim.tick;
    if !tick_nuovo
        || !sim.running
        || sim.partita_finita
        || !ammessi(&modalita)
        || !imprevisti.tranquillo(sim.tick)
    {
        return;
    }
    let mut rng = rand::rng();
    if stato
        .pianifica(sim.tick, |n| rng.random_range(0..n))
        .is_some()
    {
        aperto.0 = true;
        if let Some(suoni) = suoni {
            audio::suona(&mut commands, &suoni.avviso, imp.effetti_lineare());
        }
    }
}

// ---------------- l'overlay ----------------

#[derive(Component)]
pub struct SchermataEvento;

/// Un bottone-opzione del bivio (0 o 1).
#[derive(Component)]
pub struct OpzioneEvento(pub usize);

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

/// Costruisce/distrugge l'overlay del bivio. Vive sotto la pausa (z 17 <
/// 20): Esc resta padrone dello schermo.
#[allow(clippy::too_many_arguments)]
pub fn sincronizza(
    mut commands: Commands,
    stato_app: Res<State<AppState>>,
    aperto: Res<EventoAperto>,
    stato: Res<Eventi>,
    sim: Res<Sim>,
    art: Res<Art>,
    moduli: Query<&Module>,
    q: Query<Entity, With<SchermataEvento>>,
) {
    let deve_esserci =
        aperto.0 && stato.corrente.is_some() && *stato_app.get() == AppState::InGioco;
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
    let evento = stato.corrente.unwrap();
    let def = &EVENTI[evento];
    let chi = &PERSONAGGI[def.presentatore];
    let carica: f32 = moduli
        .iter()
        .filter(|m| m.kind == ModuleKind::Batteria && !m.broken)
        .map(|m| m.carica)
        .sum();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(NERO.with_alpha(0.85)),
            GlobalZIndex(17),
            SchermataEvento,
        ))
        .with_children(|r| {
            r.spawn(testo(def.titolo, 26.0, BIANCO));
            // il fumetto compatto: chi parla, e la situazione
            r.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: Val::Px(10.0),
                max_width: Val::Px(540.0),
                margin: UiRect::bottom(Val::Px(8.0)),
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
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
                            max_width: Val::Px(450.0),
                            ..default()
                        },
                        BorderColor::all(GRIGIO_SCAFO),
                        BackgroundColor(BIANCO),
                    ))
                    .with_children(|balloon| {
                        balloon.spawn(testo(
                            format!("\u{201C}{}\u{201D}", def.situazione),
                            14.0,
                            NERO,
                        ));
                    });
                });
            });
            // le due strade, col patto scritto sotto
            r.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(18.0),
                ..default()
            })
            .with_children(|riga| {
                for (i, opzione) in def.opzioni.iter().enumerate() {
                    let motivo = non_disponibile(evento, i, &sim, carica);
                    let spenta = motivo.is_some();
                    let mut bottone = riga.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(6.0),
                            width: Val::Px(280.0),
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(if spenta { NERO } else { SCAFO_SCURO }),
                        BorderColor::all(if spenta { GRIGIO_SCAFO } else { METALLO }),
                        Button,
                        OpzioneEvento(i),
                    ));
                    if spenta {
                        bottone.insert(BottoneMuto);
                    }
                    bottone.with_children(|c| {
                        c.spawn(testo(
                            format!("{}  \u{00B7}  {}", i + 1, opzione.scelta),
                            17.0,
                            if spenta { GRIGIO_SCAFO } else { BIANCO },
                        ));
                        let riga_sotto = match motivo {
                            Some(motivo) => format!("({motivo})"),
                            None => opzione.conseguenza.to_string(),
                        };
                        c.spawn(testo(
                            riga_sotto,
                            12.0,
                            if spenta { GRIGIO_SCAFO } else { GIALLO },
                        ));
                    });
                }
            });
            r.spawn(testo("1 / 2 oppure click", 11.0, GRIGIO_MEDIO));
        });
}

// ---------------- la scelta ----------------

/// Applica la strada scelta (click o tasti 1/2), chiude il bivio e
/// riavvia il mondo.
#[allow(clippy::too_many_arguments)]
pub fn scegli(
    mut commands: Commands,
    interazioni: Query<(&Interaction, &OpzioneEvento, Option<&BottoneMuto>), Changed<Interaction>>,
    tasti: Res<ButtonInput<KeyCode>>,
    pausa: Res<Pausa>,
    mut aperto: ResMut<EventoAperto>,
    mut stato: ResMut<Eventi>,
    mut sim: ResMut<Sim>,
    mut portafoglio: ResMut<Portafoglio>,
    mut log: ResMut<EventLog>,
    mut moduli: Query<&mut Module>,
    schermate: Query<Entity, With<SchermataEvento>>,
) {
    if !aperto.0 || pausa.aperta {
        return;
    }
    let Some(evento) = stato.corrente else {
        return;
    };
    let mut scelta = None;
    for (interazione, opzione, muto) in &interazioni {
        if *interazione == Interaction::Pressed && muto.is_none() {
            scelta = Some(opzione.0);
        }
    }
    if tasti.just_pressed(KeyCode::Digit1) {
        scelta = Some(0);
    }
    if tasti.just_pressed(KeyCode::Digit2) {
        scelta = Some(1);
    }
    let Some(scelta) = scelta else {
        return;
    };
    // la tastiera non aggira le opzioni spente
    let carica: f32 = moduli
        .iter()
        .filter(|m| m.kind == ModuleKind::Batteria && !m.broken)
        .map(|m| m.carica)
        .sum();
    if non_disponibile(evento, scelta, &sim, carica).is_some() {
        return;
    }

    let tick = sim.tick;
    match (evento, scelta) {
        (0, 0) => {
            sim.equipaggio += 3;
            log.info(
                tick,
                "Attracco autorizzato: +3 a bordo (i posti letto sono un problema di domani)",
            );
        }
        (0, 1) => {
            sim.punteggio += 20;
            log.info(tick, "Attracco rifiutato: +20 punti, il regolamento ringrazia");
        }
        (1, 0) => {
            sim.punteggio = sim.punteggio.saturating_sub(30);
            sim.ossigeno = O2_MAX;
            log.info(tick, "Bombole comprate: riserva d'ossigeno al massimo (\u{2212}30 punti)");
        }
        (1, 1) => log.info(tick, "Il mercante se ne va, salutando un po' troppo a lungo"),
        (2, 0) => {
            for mut m in &mut moduli {
                if m.kind == ModuleKind::Batteria {
                    m.carica = 0.0;
                }
            }
            log.info(tick, "Condensatori scaricati: lo sciame è deviato");
        }
        (2, 1) => {
            let mut rng = rand::rng();
            if rng.random_range(0..2) == 0 {
                let mut vivi: Vec<_> = moduli.iter_mut().filter(|m| !m.broken).collect();
                if vivi.is_empty() {
                    log.info(tick, "Lo sciame sfiora una stazione vuota");
                } else {
                    let i = rng.random_range(0..vivi.len());
                    let m = &mut vivi[i];
                    m.broken = true;
                    m.powered = false;
                    log.push(
                        tick,
                        Gravita::Allarme,
                        format!("Lo sciame colpisce: {} fuori uso", m.etichetta),
                    );
                }
            } else {
                log.info(tick, "Lo sciame sfiora la stazione: nessun danno");
            }
        }
        (3, 0) => {
            sim.punteggio += 40;
            sim.surriscaldamento += 2;
            log.info(tick, "Straordinari accettati: +40 punti, i sistemi scottano");
        }
        (3, 1) => log.info(tick, "Straordinari rifiutati: la salute prima dei numeri"),
        (4, 0) => {
            sim.equipaggio += 1;
            log.info(tick, "Il clandestino resta: +1 a bordo, il registro si arrangia");
        }
        (4, 1) => {
            portafoglio.crediti += 1;
            portafoglio.salva();
            log.info(tick, "Clandestino consegnato: +1 credito per il Marketplace");
        }
        _ => {}
    }

    stato.chiudi(tick);
    aperto.0 = false;
    for e in &schermate {
        commands.entity(e).despawn();
    }
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    /// Prima estrazione: il dado 1-su-N (0 = successo); seconda: quale
    /// evento tra i liberi.
    fn sempre(colpo: u32, quale: u32) -> impl FnMut(u32) -> u32 {
        let mut chiamata = 0;
        move |_| {
            chiamata += 1;
            if chiamata == 1 { colpo } else { quale }
        }
    }

    #[test]
    fn niente_bivi_prima_della_grazia_o_in_cooldown() {
        let mut e = Eventi::default();
        assert_eq!(e.pianifica(TICK_GRAZIA - 1, sempre(0, 0)), None);
        assert!(e.pianifica(TICK_GRAZIA, sempre(0, 0)).is_some());
        e.chiudi(TICK_GRAZIA + 3);
        let dopo = TICK_GRAZIA + 3;
        assert_eq!(e.pianifica(dopo + TICK_COOLDOWN - 1, sempre(0, 0)), None);
        assert!(e.pianifica(dopo + TICK_COOLDOWN, sempre(0, 0)).is_some());
    }

    #[test]
    fn il_dado_deve_dire_si() {
        let mut e = Eventi::default();
        assert_eq!(e.pianifica(TICK_GRAZIA, sempre(5, 0)), None);
        assert!(e.corrente.is_none());
    }

    #[test]
    fn ogni_evento_esce_al_massimo_una_volta() {
        let mut e = Eventi::default();
        let mut usciti = Vec::new();
        let mut tick = TICK_GRAZIA;
        for _ in 0..EVENTI.len() {
            let ev = e.pianifica(tick, sempre(0, 0)).unwrap();
            assert!(!usciti.contains(&ev), "evento ripetuto: {ev}");
            usciti.push(ev);
            e.chiudi(tick);
            tick += TICK_COOLDOWN;
        }
        // catalogo esaurito: mai più bivi in questa partita
        assert_eq!(e.pianifica(tick + 1000, sempre(0, 0)), None);
    }

    #[test]
    fn mentre_un_bivio_e_aperto_non_se_ne_apre_un_altro() {
        let mut e = Eventi::default();
        assert!(e.pianifica(TICK_GRAZIA, sempre(0, 0)).is_some());
        assert_eq!(e.pianifica(TICK_GRAZIA + 1, sempre(0, 0)), None);
    }

    #[test]
    fn le_opzioni_impossibili_sono_dichiarate() {
        let mut sim = Sim::default();
        sim.punteggio = 10;
        assert!(non_disponibile(1, 0, &sim, 0.0).is_some()); // compra senza punti
        sim.punteggio = 30;
        assert!(non_disponibile(1, 0, &sim, 0.0).is_none());
        assert!(non_disponibile(2, 0, &sim, 0.0).is_some()); // scarica senza batterie
        assert!(non_disponibile(2, 0, &sim, 5.0).is_none());
        assert!(non_disponibile(0, 0, &sim, 0.0).is_none()); // le altre sempre libere
    }

    #[test]
    fn in_campagna_partono_dal_livello_otto() {
        assert!(!ammessi(&Modalita::Campagna(6)));
        assert!(ammessi(&Modalita::Campagna(7)));
        assert!(ammessi(&Modalita::Infinita));
    }
}
