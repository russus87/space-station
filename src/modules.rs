//! Tabella dei moduli: l'UNICO punto dove si tara produzione/consumo.
//! Tutti i valori sono "per tick". `priorita` è l'ordine di alimentazione
//! elettrica (basso = alimentato per primo): in un blackout i moduli si
//! spengono in ordine inverso, quindi i laboratori cadono per primi e il
//! life support per ultimo, come da spec.

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModuleKind {
    Reattore,
    LifeSupport,
    Dormitorio,
    Laboratorio,
    Radiatore,
    Corridoio,
    // sbloccabili lungo la campagna (campo `sblocco` in tabella)
    Batteria,
    Serra,
    Gru,
    Condotto,
    CentroComando,
}

/// Ordine della palette (tasti 1..6, poi 7 8 9 0 C per gli sbloccabili).
pub const KINDS: [ModuleKind; 11] = [
    ModuleKind::Reattore,
    ModuleKind::LifeSupport,
    ModuleKind::Dormitorio,
    ModuleKind::Laboratorio,
    ModuleKind::Radiatore,
    ModuleKind::Corridoio,
    ModuleKind::Batteria,
    ModuleKind::Serra,
    ModuleKind::Gru,
    ModuleKind::Condotto,
    ModuleKind::CentroComando,
];

pub struct ModuleDef {
    pub nome: &'static str,
    pub sigla: &'static str,   // etichetta corta, ormai usata solo nei testi
    pub sprite: &'static str,  // path dell'art 32x32 sotto assets/
    pub energia: f32,
    pub ossigeno: f32,
    pub calore: f32,
    pub posti_letto: u32,
    pub equipaggio_richiesto: u32,
    pub priorita: u8,
    pub colore: Color,
    /// Livelli di campagna da completare per averlo (0 = disponibile da
    /// subito). Lo sblocco vale in TUTTE le modalità: la campagna è il modo
    /// di ottenerlo, non l'unico posto dove usarlo.
    pub sblocco: usize,
}

pub const TABELLA: [ModuleDef; 11] = [
    ModuleDef {
        nome: "Reattore",
        sigla: "REA",
        sprite: "sprites/moduli/reattore.png",
        energia: 100.0,
        ossigeno: 0.0,
        calore: 40.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 0, // i produttori sono sempre attivi, il valore non conta
        colore: Color::srgb(0.93, 0.60, 0.16),
        sblocco: 0,
    },
    ModuleDef {
        nome: "Life Support",
        sigla: "LIF",
        sprite: "sprites/moduli/life_support.png",
        energia: -30.0,
        ossigeno: 50.0,
        calore: 5.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 0,
        colore: Color::srgb(0.30, 0.75, 0.90),
        sblocco: 0,
    },
    ModuleDef {
        nome: "Dormitorio",
        sigla: "DOR",
        sprite: "sprites/moduli/dormitorio.png",
        energia: -10.0,
        ossigeno: 0.0,
        calore: 2.0,
        posti_letto: 4,
        equipaggio_richiesto: 0,
        priorita: 3,
        colore: Color::srgb(0.62, 0.48, 0.88),
        sblocco: 0,
    },
    ModuleDef {
        nome: "Laboratorio",
        sigla: "LAB",
        sprite: "sprites/moduli/laboratorio.png",
        energia: -40.0,
        ossigeno: 0.0,
        calore: 25.0,
        posti_letto: 0,
        equipaggio_richiesto: 2,
        priorita: 4,
        colore: Color::srgb(0.42, 0.80, 0.45),
        sblocco: 0,
    },
    ModuleDef {
        nome: "Radiatore",
        sigla: "RAD",
        sprite: "sprites/moduli/radiatore.png",
        energia: -5.0,
        ossigeno: 0.0,
        calore: -50.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 1,
        colore: Color::srgb(0.55, 0.65, 0.72),
        sblocco: 0,
    },
    ModuleDef {
        nome: "Corridoio",
        sigla: "COR",
        sprite: "sprites/moduli/corridoio.png",
        energia: -1.0,
        ossigeno: 0.0,
        calore: 0.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 2,
        colore: Color::srgb(0.45, 0.45, 0.50),
        sblocco: 0,
    },
    // ---- sbloccabili (SPEC-CAMPAGNA §3, approvati) ----
    ModuleDef {
        nome: "Batteria",
        sigla: "BAT",
        sprite: "sprites/moduli/batteria.png",
        // l'energia nominale è 0: carica e scarica stanno in sim.rs
        // (CAPIENZA_BATTERIA, RICARICA_BATTERIA), sul residuo della rete
        energia: 0.0,
        ossigeno: 0.0,
        calore: 2.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 1,
        colore: Color::srgb(0.85, 0.78, 0.35),
        sblocco: 5,
    },
    ModuleDef {
        nome: "Serra",
        sigla: "SER",
        sprite: "sprites/moduli/serra.png",
        energia: -10.0,
        ossigeno: 20.0,
        calore: 8.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 0, // fa ossigeno: critica come il life support
        colore: Color::srgb(0.35, 0.70, 0.40),
        sblocco: 15,
    },
    ModuleDef {
        nome: "Gru",
        sigla: "GRU",
        sprite: "sprites/moduli/gru.png",
        // dopo 12 tick attivi consecutivi rimuove un detrito adiacente e
        // si smonta (contatore in sim.rs, effetto spaziale in main.rs)
        energia: -20.0,
        ossigeno: 0.0,
        calore: 5.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 4, // sacrificabile per prima, come il laboratorio
        colore: Color::srgb(0.80, 0.55, 0.30),
        sblocco: 25,
    },
    ModuleDef {
        nome: "Condotto termico",
        sigla: "CON",
        sprite: "sprites/moduli/condotto.png",
        energia: -15.0,
        ossigeno: 0.0,
        calore: -90.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 1,
        colore: Color::srgb(0.40, 0.50, 0.60),
        sblocco: 35,
    },
    ModuleDef {
        nome: "Centro comando",
        sigla: "CDO",
        sprite: "sprites/moduli/centro_comando.png",
        // massimo uno per stazione (vincolo in input_mouse); se attivo gli
        // arrivi passano da 1 ogni 4 tick a 1 ogni 2 (sim.rs)
        energia: -25.0,
        ossigeno: 0.0,
        calore: 5.0,
        posti_letto: 0,
        equipaggio_richiesto: 0,
        priorita: 3,
        colore: Color::srgb(0.75, 0.75, 0.80),
        sblocco: 45,
    },
];

impl ModuleKind {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn def(self) -> &'static ModuleDef {
        &TABELLA[self.index()]
    }
}
