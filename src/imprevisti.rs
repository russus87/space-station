//! Imprevisti casuali: il mondo che si mette contro la stazione.
//!
//! Quattro eventi — meteorite, tempesta elettromagnetica, sciame di
//! micrometeoriti, passaggio ravvicinato di un pianeta (l'unico benigno) —
//! con le regole d'ingaggio concordate: mai in anteprima, mai prima del
//! tick di grazia, mai due di fila dello stesso tipo, cooldown tra uno e
//! l'altro, e PREAVVISO di qualche tick per i negativi: la sirena parte
//! prima dell'impatto, così il danno si può ancora limitare. In campagna
//! compaiono solo dal livello 8: i primi sette sono scuola guida.
//!
//! Durante la sirena la musica va in pausa (`musica::MusicaSospesa`) e in
//! alto lampeggia il faro d'emergenza; le animazioni (meteore in volo,
//! tinta della tempesta, pianeta sullo sfondo) sono puro teatro: gli
//! effetti meccanici passano tutti dalle risorse vere (`Sim`, `Module`).
//!
//! La macchina a stati è deterministica e testabile: l'unica casualità
//! entra dall'esterno (`rand` di sistema nei sistemi Bevy, chiusure finte
//! nei test).

use crate::impostazioni::Impostazioni;
use crate::livelli::Modalita;
use crate::menu::{AppState, Pausa};
use crate::modules::ModuleKind;
use crate::musica::MusicaSospesa;
use crate::sim::{EventLog, Gravita, Module, Sim};
use crate::ui::CIANO;

/// Viola della palette (SPEC §2.2, #9B7BDB). Vive qui solo perché ui.rs
/// non lo esporta ancora ed è fuori dalla mia proprietà: da promuovere in
/// ui.rs alla prima occasione.
const VIOLA: Color = Color::srgb_u8(0x9B, 0x7B, 0xDB);
use crate::{GRID_H, GRID_W, Station};
use bevy::prelude::*;
use rand::RngExt;

/// Nessun imprevisto prima di questo tick: si lascia costruire in pace.
pub const TICK_GRAZIA: u64 = 40;
/// Tick minimi tra la fine di un imprevisto e la pianificazione del prossimo.
pub const TICK_COOLDOWN: u64 = 50;
/// Tick di preavviso (sirena) prima dell'impatto dei negativi.
pub const TICK_PREAVVISO: u64 = 4;
/// Probabilità di pianificare a ogni tick eleggibile: 1 su N.
const UNO_SU: u32 = 70;
/// La sirena resta accesa qualche tick oltre l'impatto.
const CODA_SIRENA: u64 = 2;
/// In campagna gli imprevisti partono da questo livello (1-based).
const PRIMO_LIVELLO: usize = 8;

const DURATA_TEMPESTA: u64 = 10;
const DURATA_PIANETA: u64 = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Meteorite,
    Tempesta,
    Sciame,
    Pianeta,
}

impl Tipo {
    fn negativo(self) -> bool {
        self != Tipo::Pianeta
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum Fase {
    #[default]
    Quiete,
    /// La sirena suona: l'impatto arriva al tick indicato.
    Preavviso { tipo: Tipo, impatto: u64 },
    /// Effetto a durata in corso (tempesta o pianeta), fino al tick indicato.
    Attivo { tipo: Tipo, fine: u64 },
}

/// Lo stato della macchina degli imprevisti. Vive per tutta la partita e
/// si riarma da solo quando il tick torna indietro (reset).
#[derive(Resource, Default)]
pub struct Imprevisti {
    fase: Fase,
    /// Non si pianifica prima di questo tick (grazia + cooldown).
    cooldown_fino: u64,
    ultimo_tipo: Option<Tipo>,
    ultimo_tick: u64,
    /// Cella bersaglio del meteorite in arrivo.
    bersaglio: Option<IVec2>,
    /// La sirena resta visibile fino a questo tick.
    sirena_fino: Option<u64>,
}

impl Imprevisti {
    fn riarma(&mut self) {
        *self = Imprevisti::default();
    }

    /// Decide se e cosa pianificare al tick corrente. `estrai(n)` è
    /// l'unica fonte di casualità: un intero uniforme in [0, n).
    /// Ritorna il tipo pianificato, se il dado ha detto sì.
    fn pianifica(
        &mut self,
        tick: u64,
        mut estrai: impl FnMut(u32) -> u32,
    ) -> Option<Tipo> {
        if self.fase != Fase::Quiete
            || tick < TICK_GRAZIA
            || tick < self.cooldown_fino
            || estrai(UNO_SU) != 0
        {
            return None;
        }
        let tipo = scegli_tipo(estrai(4), self.ultimo_tipo);
        self.ultimo_tipo = Some(tipo);
        if tipo.negativo() {
            let impatto = tick + TICK_PREAVVISO;
            self.fase = Fase::Preavviso { tipo, impatto };
            self.sirena_fino = Some(impatto + CODA_SIRENA);
        } else {
            self.fase = Fase::Attivo {
                tipo,
                fine: tick + DURATA_PIANETA,
            };
        }
        Some(tipo)
    }

    fn chiudi(&mut self, tick: u64) {
        self.fase = Fase::Quiete;
        self.cooldown_fino = tick + TICK_COOLDOWN;
        self.bersaglio = None;
    }

    /// La sirena è accesa a questo tick?
    pub fn sirena_accesa(&self, tick: u64) -> bool {
        self.sirena_fino.is_some_and(|fino| tick <= fino)
    }

    /// Nessun imprevisto in preavviso o in corso, e sirena spenta: il
    /// momento buono per qualunque altra interruzione (eventi.rs non apre
    /// bivi mentre piovono meteore).
    pub fn tranquillo(&self, tick: u64) -> bool {
        self.fase == Fase::Quiete && !self.sirena_accesa(tick)
    }

    /// La tempesta elettromagnetica è in corso?
    pub fn tempesta_in_corso(&self) -> bool {
        matches!(
            self.fase,
            Fase::Attivo {
                tipo: Tipo::Tempesta,
                ..
            }
        )
    }
}

/// Il tipo estratto (0..4), scartando l'ultimo già uscito: mai due di
/// fila uguali — se il dado ripete, si scala al successivo.
fn scegli_tipo(estratto: u32, ultimo: Option<Tipo>) -> Tipo {
    const TIPI: [Tipo; 4] = [Tipo::Meteorite, Tipo::Tempesta, Tipo::Sciame, Tipo::Pianeta];
    let mut i = (estratto as usize) % TIPI.len();
    if Some(TIPI[i]) == ultimo {
        i = (i + 1) % TIPI.len();
    }
    TIPI[i]
}

/// Gli imprevisti sono ammessi in questa modalità/livello?
fn ammessi(modalita: &Modalita) -> bool {
    match modalita {
        Modalita::Campagna(i) => i + 1 >= PRIMO_LIVELLO,
        _ => true,
    }
}

// ---------------- suoni dedicati ----------------

/// Handle dei suoni degli imprevisti, caricati a parte (audio::Suoni non
/// si tocca: è territorio di audio.rs).
#[derive(Resource)]
pub struct SuoniImprevisti {
    sirena: Handle<AudioSource>,
    impatto: Handle<AudioSource>,
    pianeta: Handle<AudioSource>,
}

pub fn carica_suoni(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SuoniImprevisti {
        sirena: assets.load("audio/sirena.wav"),
        impatto: assets.load("audio/impatto.wav"),
        pianeta: assets.load("audio/pianeta.wav"),
    });
}

// ---------------- sprite delle animazioni ----------------

#[derive(Resource)]
pub struct ArteImprevisti {
    sirena: [Handle<Image>; 2],
    meteora: Handle<Image>,
    pianeta: Handle<Image>,
}

pub fn carica_arte(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(ArteImprevisti {
        sirena: [
            assets.load("sprites/imprevisti/sirena_1.png"),
            assets.load("sprites/imprevisti/sirena_2.png"),
        ],
        meteora: assets.load("sprites/imprevisti/meteora.png"),
        pianeta: assets.load("sprites/imprevisti/pianeta.png"),
    });
}

// ---------------- marker delle entità sceniche ----------------

/// Meteora in volo (world-space): direzione e velocità in px/s.
#[derive(Component)]
pub struct Meteora {
    verso: Vec2,
    velocita: f32,
    /// Autodistruzione comunque dopo questo tempo (secondi residui).
    vita: f32,
}

/// Lampo bianco sulla cella colpita, si spegne da solo.
#[derive(Component)]
pub struct Flash(Timer);

/// Il pianeta che attraversa lo sfondo, lentissimo.
#[derive(Component)]
pub struct Pianeta;

/// Il faro d'emergenza in alto (UI): marker del nodo radice.
#[derive(Component)]
pub struct Sirena;

/// L'immagine animata dentro il faro (i due frame si alternano qui).
#[derive(Component)]
pub struct SirenaFrame;

/// La tinta pulsante della tempesta (UI a tutto schermo).
#[derive(Component)]
pub struct TintaTempesta;

// ---------------- il cervello ----------------

/// Avanza la macchina a stati una volta per tick effettivo: pianifica,
/// fa scattare gli impatti, spegne gli effetti a durata.
#[allow(clippy::too_many_arguments)]
pub fn pianifica_e_applica(
    mut commands: Commands,
    mut stato: ResMut<Imprevisti>,
    mut sim: ResMut<Sim>,
    mut log: ResMut<EventLog>,
    mut sospesa: ResMut<MusicaSospesa>,
    modalita: Res<Modalita>,
    station: Res<Station>,
    suoni: Option<Res<SuoniImprevisti>>,
    arte: Option<Res<ArteImprevisti>>,
    imp: Res<Impostazioni>,
    mut moduli: Query<(&mut Module, &Transform)>,
) {
    // riarmo su reset: il tick torna indietro quando parte una partita nuova
    if sim.tick < stato.ultimo_tick {
        stato.riarma();
        if sospesa.0 {
            sospesa.0 = false;
        }
    }
    let tick_nuovo = sim.tick != stato.ultimo_tick;
    stato.ultimo_tick = sim.tick;
    if !tick_nuovo || !sim.running || sim.partita_finita || !ammessi(&modalita) {
        // niente tick nuovo (o partita ferma): si aggiorna solo la sirena
        if !stato.sirena_accesa(sim.tick) && sospesa.0 {
            sospesa.0 = false;
        }
        return;
    }
    let (Some(suoni), Some(arte)) = (suoni, arte) else {
        return;
    };
    let tick = sim.tick;

    // 1) pianificazione
    let mut rng = rand::rng();
    if let Some(tipo) = stato.pianifica(tick, |n| rng.random_range(0..n)) {
        match tipo {
            Tipo::Meteorite => {
                let cella = IVec2::new(
                    rng.random_range(0..GRID_W),
                    rng.random_range(0..GRID_H),
                );
                stato.bersaglio = Some(cella);
                log.push(tick, Gravita::Allarme, "Allarme: meteorite in arrivo");
                // il bersaglio esatto lo conosce solo la plancia: la meteora
                // punta la posizione del modulo se c'è, il centro griglia se no
                let dove = station
                    .celle
                    .get(&cella)
                    .and_then(|e| moduli.get(*e).ok())
                    .map(|(_, tf)| tf.translation.truncate())
                    .unwrap_or(Vec2::new(
                        rng.random_range(-150.0..150.0),
                        rng.random_range(-100.0..100.0),
                    ));
                spawna_meteora(&mut commands, &arte, dove, 340.0, 3.6);
            }
            Tipo::Tempesta => {
                log.push(
                    tick,
                    Gravita::Allarme,
                    "Allarme: tempesta elettromagnetica in arrivo",
                );
            }
            Tipo::Sciame => {
                log.push(
                    tick,
                    Gravita::Allarme,
                    "Allarme: sciame di micrometeoriti in arrivo",
                );
                for _ in 0..6 {
                    let dove = Vec2::new(
                        rng.random_range(-350.0..350.0),
                        rng.random_range(-220.0..220.0),
                    );
                    spawna_meteora(
                        &mut commands,
                        &arte,
                        dove,
                        rng.random_range(420.0..640.0),
                        2.6,
                    );
                }
            }
            Tipo::Pianeta => {
                log.info(
                    tick,
                    "Passaggio ravvicinato: tutti vogliono vedere il panorama (arrivi raddoppiati)",
                );
                crate::audio::suona(&mut commands, &suoni.pianeta, imp.effetti_lineare());
                commands.spawn((
                    Sprite {
                        image: arte.pianeta.clone(),
                        custom_size: Some(Vec2::splat(150.0)),
                        ..default()
                    },
                    // sopra le stelle (0.1), sotto le linee della griglia
                    // (0.5): un corpo celeste dietro il cantiere
                    Transform::from_xyz(-620.0, 120.0, 0.3),
                    Pianeta,
                ));
            }
        }
        if tipo.negativo() {
            sospesa.0 = true;
            crate::audio::suona(&mut commands, &suoni.sirena, imp.effetti_lineare());
        }
    }

    // 2) impatti a fine preavviso
    if let Fase::Preavviso { tipo, impatto } = stato.fase
        && tick >= impatto
    {
        match tipo {
            Tipo::Meteorite => {
                let colpito = stato
                    .bersaglio
                    .and_then(|cella| station.celle.get(&cella).copied());
                match colpito.and_then(|e| moduli.get_mut(e).ok()) {
                    Some((mut modulo, tf)) if !modulo.broken => {
                        modulo.broken = true;
                        modulo.powered = false;
                        log.push(
                            tick,
                            Gravita::Allarme,
                            format!("Meteorite: {} fuori uso", modulo.etichetta),
                        );
                        commands.spawn((
                            Sprite::from_color(
                                Color::WHITE.with_alpha(0.9),
                                Vec2::splat(64.0),
                            ),
                            Transform::from_translation(tf.translation + Vec3::Z),
                            Flash(Timer::from_seconds(0.18, TimerMode::Once)),
                        ));
                    }
                    _ => log.info(tick, "Meteorite: caduto nel vuoto"),
                }
                crate::audio::suona(&mut commands, &suoni.impatto, imp.effetti_lineare());
                stato.chiudi(tick);
            }
            Tipo::Sciame => {
                let mut corridoi: Vec<_> = moduli
                    .iter_mut()
                    .filter(|(m, _)| m.kind == ModuleKind::Corridoio && !m.broken)
                    .collect();
                if corridoi.is_empty() {
                    log.info(tick, "Lo sciame passa senza danni");
                } else {
                    let quanti = 2.min(corridoi.len());
                    for _ in 0..quanti {
                        let i = rng.random_range(0..corridoi.len());
                        let (m, _) = &mut corridoi[i];
                        m.broken = true;
                        m.powered = false;
                        log.push(
                            tick,
                            Gravita::Allarme,
                            format!("Sciame: {} fuori uso", m.etichetta),
                        );
                        corridoi.swap_remove(i);
                    }
                }
                crate::audio::suona(&mut commands, &suoni.impatto, imp.effetti_lineare());
                stato.chiudi(tick);
            }
            Tipo::Tempesta => {
                log.push(
                    tick,
                    Gravita::Allarme,
                    "Tempesta elettromagnetica: i sistemi termici impazziscono",
                );
                stato.fase = Fase::Attivo {
                    tipo: Tipo::Tempesta,
                    fine: tick + DURATA_TEMPESTA,
                };
            }
            Tipo::Pianeta => unreachable!("il pianeta non ha preavviso"),
        }
    }

    // 3) effetti a durata
    if let Fase::Attivo { tipo, fine } = stato.fase {
        match tipo {
            Tipo::Tempesta => {
                // pressione termica extra: il contatore di surriscaldamento
                // avanza anche a bilancio in ordine
                sim.surriscaldamento += 1;
            }
            Tipo::Pianeta
                // arrivi raddoppiati: un ospite in più ogni 4 tick, con le
                // stesse regole vere (posti liberi e aria buona)
                if tick.is_multiple_of(4) && sim.equipaggio < sim.posti_letto && sim.ossigeno > 50.0 => {
                    sim.equipaggio += 1;
                    let ora = sim.equipaggio;
                    log.info(
                        tick,
                        format!("Equipaggio: {} → {} (venuti per il panorama)", ora - 1, ora),
                    );
                }
            _ => {}
        }
        if tick >= fine {
            match tipo {
                Tipo::Tempesta => log.info(tick, "La tempesta è passata"),
                Tipo::Pianeta => log.info(tick, "Il pianeta si allontana: si torna al lavoro"),
                _ => {}
            }
            stato.chiudi(tick);
        }
    }

    // 4) sirena spenta = musica che riprende
    if !stato.sirena_accesa(tick) && sospesa.0 {
        sospesa.0 = false;
    }
}

fn spawna_meteora(
    commands: &mut Commands,
    arte: &ArteImprevisti,
    bersaglio: Vec2,
    velocita: f32,
    vita: f32,
) {
    let partenza = bersaglio + Vec2::new(-820.0, 560.0);
    let verso = (bersaglio - partenza).normalize_or_zero();
    commands.spawn((
        Sprite {
            image: arte.meteora.clone(),
            custom_size: Some(Vec2::splat(40.0)),
            ..default()
        },
        Transform::from_translation(partenza.extend(2.5)),
        Meteora {
            verso,
            velocita,
            vita,
        },
    ));
}

// ---------------- il teatro ----------------

/// Muove le meteore, spegne i flash, fa passare il pianeta, tiene accesa
/// la sirena e la tinta della tempesta; e fa pulizia quando la scena non
/// c'è più (menu, fine partita).
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn anima(
    mut commands: Commands,
    tempo: Res<Time>,
    stato: Res<Imprevisti>,
    sim: Res<Sim>,
    stato_app: Res<State<AppState>>,
    pausa: Res<Pausa>,
    arte: Option<Res<ArteImprevisti>>,
    mut meteore: Query<(Entity, &mut Transform, &mut Meteora)>,
    mut flash: Query<(Entity, &mut Flash, &mut Sprite), Without<Meteora>>,
    mut pianeti: Query<
        (Entity, &mut Transform),
        (With<Pianeta>, Without<Meteora>),
    >,
    sirene: Query<Entity, With<Sirena>>,
    mut frame_sirena: Query<&mut ImageNode, With<SirenaFrame>>,
    tinte: Query<Entity, With<TintaTempesta>>,
    mut alpha_tinte: Query<&mut BackgroundColor, With<TintaTempesta>>,
) {
    let in_gioco = *stato_app.get() == AppState::InGioco;
    let dt = tempo.delta_secs();

    // fuori dalla partita il teatro chiude
    if !in_gioco {
        for (e, ..) in &meteore {
            commands.entity(e).despawn();
        }
        for (e, ..) in &flash {
            commands.entity(e).despawn();
        }
        for (e, _) in &pianeti {
            commands.entity(e).despawn();
        }
        for e in &sirene {
            commands.entity(e).despawn();
        }
        for e in &tinte {
            commands.entity(e).despawn();
        }
        return;
    }

    // meteore in volo (si muovono anche in pausa: sono scenografia)
    for (e, mut tf, mut m) in &mut meteore {
        tf.translation += (m.verso * m.velocita * dt).extend(0.0);
        m.vita -= dt;
        if m.vita <= 0.0 {
            commands.entity(e).despawn();
        }
    }

    // lampi d'impatto
    for (e, mut f, mut sprite) in &mut flash {
        f.0.tick(tempo.delta());
        let resto = f.0.fraction_remaining();
        sprite.color = Color::WHITE.with_alpha(0.9 * resto);
        if f.0.is_finished() {
            commands.entity(e).despawn();
        }
    }

    // il pianeta scivola via lentamente
    for (e, mut tf) in &mut pianeti {
        tf.translation.x += 22.0 * dt;
        if tf.translation.x > 700.0 {
            commands.entity(e).despawn();
        }
    }

    let Some(arte) = arte else {
        return;
    };

    // sirena: esiste solo quando la macchina la vuole accesa
    let sirena_su = stato.sirena_accesa(sim.tick) && !pausa.aperta;
    if sirena_su && sirene.is_empty() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(64.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-28.0)),
                    width: Val::Px(56.0),
                    height: Val::Px(56.0),
                    ..default()
                },
                GlobalZIndex(16),
                Sirena,
            ))
            .with_children(|p| {
                p.spawn((
                    ImageNode::new(arte.sirena[0].clone()),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    SirenaFrame,
                ));
            });
    } else if !sirena_su {
        for e in &sirene {
            commands.entity(e).despawn();
        }
    }
    // i due frame si alternano a ~4 fps
    let frame = (tempo.elapsed_secs() * 4.0) as usize % 2;
    for mut img in &mut frame_sirena {
        img.image = arte.sirena[frame].clone();
    }

    // tinta della tempesta: pulsa piano finché dura
    let tempesta = stato.tempesta_in_corso();
    if tempesta && tinte.is_empty() {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(VIOLA.with_alpha(0.06)),
            GlobalZIndex(5),
            TintaTempesta,
        ));
    } else if !tempesta {
        for e in &tinte {
            commands.entity(e).despawn();
        }
    }
    let pulsazione = 0.05 + 0.045 * (tempo.elapsed_secs() * 5.0).sin();
    for mut bg in &mut alpha_tinte {
        let miscela = if (tempo.elapsed_secs() * 2.5) as i32 % 2 == 0 {
            VIOLA
        } else {
            CIANO
        };
        bg.0 = miscela.with_alpha(pulsazione.max(0.02));
    }
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    /// Estrazioni finte: la prima chiamata è il dado 1-su-N (0 = successo),
    /// la seconda il tipo.
    fn sempre(colpo: u32, tipo: u32) -> impl FnMut(u32) -> u32 {
        let mut chiamata = 0;
        move |_| {
            chiamata += 1;
            if chiamata == 1 { colpo } else { tipo }
        }
    }

    #[test]
    fn niente_imprevisti_prima_della_grazia_o_in_cooldown() {
        let mut s = Imprevisti::default();
        assert_eq!(s.pianifica(TICK_GRAZIA - 1, sempre(0, 0)), None);
        assert!(s.pianifica(TICK_GRAZIA, sempre(0, 0)).is_some());
        s.chiudi(TICK_GRAZIA + TICK_PREAVVISO);
        let dopo = TICK_GRAZIA + TICK_PREAVVISO;
        assert_eq!(s.pianifica(dopo + TICK_COOLDOWN - 1, sempre(0, 1)), None);
        assert!(s.pianifica(dopo + TICK_COOLDOWN, sempre(0, 1)).is_some());
    }

    #[test]
    fn il_dado_deve_dire_si() {
        let mut s = Imprevisti::default();
        assert_eq!(s.pianifica(TICK_GRAZIA, sempre(3, 0)), None);
        assert_eq!(s.fase, Fase::Quiete);
    }

    #[test]
    fn mai_due_di_fila_dello_stesso_tipo() {
        assert_eq!(scegli_tipo(0, None), Tipo::Meteorite);
        assert_eq!(scegli_tipo(0, Some(Tipo::Meteorite)), Tipo::Tempesta);
        assert_eq!(scegli_tipo(3, Some(Tipo::Pianeta)), Tipo::Meteorite);
        assert_eq!(scegli_tipo(2, Some(Tipo::Sciame)), Tipo::Pianeta);
    }

    #[test]
    fn i_negativi_hanno_preavviso_e_sirena_il_pianeta_no() {
        let mut s = Imprevisti::default();
        assert_eq!(s.pianifica(50, sempre(0, 0)), Some(Tipo::Meteorite));
        assert!(matches!(s.fase, Fase::Preavviso { impatto, .. } if impatto == 50 + TICK_PREAVVISO));
        assert!(s.sirena_accesa(50 + TICK_PREAVVISO + CODA_SIRENA));
        assert!(!s.sirena_accesa(50 + TICK_PREAVVISO + CODA_SIRENA + 1));

        let mut s = Imprevisti::default();
        assert_eq!(s.pianifica(50, sempre(0, 3)), Some(Tipo::Pianeta));
        assert!(matches!(s.fase, Fase::Attivo { tipo: Tipo::Pianeta, fine } if fine == 50 + DURATA_PIANETA));
        assert!(!s.sirena_accesa(50));
    }

    #[test]
    fn tranquillo_solo_in_quiete_e_a_sirena_spenta() {
        let mut s = Imprevisti::default();
        assert!(s.tranquillo(10));
        assert_eq!(s.pianifica(50, sempre(0, 0)), Some(Tipo::Meteorite));
        assert!(!s.tranquillo(50)); // preavviso in corso
        s.chiudi(50 + TICK_PREAVVISO);
        // fase quieta ma la sirena ha ancora la coda accesa
        assert!(!s.tranquillo(50 + TICK_PREAVVISO + CODA_SIRENA));
        assert!(s.tranquillo(50 + TICK_PREAVVISO + CODA_SIRENA + 1));
    }

    #[test]
    fn in_campagna_partono_dal_livello_otto() {
        assert!(!ammessi(&Modalita::Campagna(6))); // livello 7
        assert!(ammessi(&Modalita::Campagna(7))); // livello 8
        assert!(ammessi(&Modalita::Infinita));
        assert!(ammessi(&Modalita::Casuale));
    }
}
