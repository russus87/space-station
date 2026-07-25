//! Impostazioni del giocatore: volume di musica ed effetti, a passi del
//! 25% (100 → 75 → 50 → 25 → 0 → 100), regolabili dal menu di pausa.
//! Persistite in `impostazioni.txt` nella cartella dati (stesso stile dei
//! file classifica: testo semplice, una riga malformata si ignora, file
//! assente = valori pieni).

use crate::livelli::cartella_dati;
use bevy::prelude::*;

#[derive(Resource)]
pub struct Impostazioni {
    /// Volume musica in percento (0..=100).
    pub musica: u32,
    /// Volume effetti in percento (0..=100).
    pub effetti: u32,
}

impl Default for Impostazioni {
    fn default() -> Self {
        Self {
            musica: 100,
            effetti: 100,
        }
    }
}

impl Impostazioni {
    pub fn musica_lineare(&self) -> f32 {
        self.musica as f32 / 100.0
    }

    pub fn effetti_lineare(&self) -> f32 {
        self.effetti as f32 / 100.0
    }

    /// Scrive il file. Come per le classifiche, un errore di I/O non ferma
    /// il gioco: al prossimo avvio si riparte dai default.
    pub fn salva(&self) {
        let Some(dir) = cartella_dati() else {
            return;
        };
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            dir.join("impostazioni.txt"),
            format!("musica={}\neffetti={}\n", self.musica, self.effetti),
        );
    }
}

/// Il passo successivo del ciclo di volume (100→75→50→25→0→100).
pub fn ciclo(volume: u32) -> u32 {
    match volume {
        100 => 75,
        75 => 50,
        50 => 25,
        25 => 0,
        _ => 100,
    }
}

/// Carica all'avvio. Righe `chiave=valore`; sconosciute o rotte si saltano.
pub fn carica() -> Impostazioni {
    let mut imp = Impostazioni::default();
    let Some(testo) = cartella_dati()
        .map(|d| d.join("impostazioni.txt"))
        .and_then(|p| std::fs::read_to_string(p).ok())
    else {
        return imp;
    };
    for riga in testo.lines() {
        let Some((chiave, valore)) = riga.split_once('=') else {
            continue;
        };
        let Ok(valore) = valore.trim().parse::<u32>() else {
            continue;
        };
        match chiave.trim() {
            "musica" => imp.musica = valore.min(100),
            "effetti" => imp.effetti = valore.min(100),
            _ => {}
        }
    }
    imp
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn il_ciclo_copre_tutti_i_passi_e_torna_a_cento() {
        assert_eq!(ciclo(100), 75);
        assert_eq!(ciclo(75), 50);
        assert_eq!(ciclo(50), 25);
        assert_eq!(ciclo(25), 0);
        assert_eq!(ciclo(0), 100);
        // un valore fuori scala (file scritto a mano) rientra nel ciclo
        assert_eq!(ciclo(63), 100);
    }
}
