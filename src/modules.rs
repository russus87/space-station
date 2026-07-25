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
}

/// Ordine della palette (tasti 1..6).
pub const KINDS: [ModuleKind; 6] = [
    ModuleKind::Reattore,
    ModuleKind::LifeSupport,
    ModuleKind::Dormitorio,
    ModuleKind::Laboratorio,
    ModuleKind::Radiatore,
    ModuleKind::Corridoio,
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
}

pub const TABELLA: [ModuleDef; 6] = [
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
