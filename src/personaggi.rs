//! I personaggi chiave della campagna e le loro battute a fumetto.
//!
//! Tabelle costanti sul modello di `LIVELLI` e `TABELLA`: è QUI (e solo qui)
//! che si scrivono nomi, ruoli e battute. Le battute agganciano i temi dei
//! blocchi della campagna (SPEC-CAMPAGNA.md §1) senza citare numeri: i
//! livelli dal 7 in poi sono generati e i numeri cambiano col seed.
//! I ritratti stanno in `Art::ritratti`, indicizzati da `Personaggio::ritratto`
//! nello stesso ordine di `PERSONAGGI`.

pub struct Personaggio {
    pub nome: &'static str,
    pub ruolo: &'static str,
    /// Indice del ritratto in `Art::ritratti`.
    pub ritratto: usize,
}

/// Ordine fissato dal caricamento di `Art::ritratti` in `main.rs`:
/// ingegnere, medico, caposquadra, scienziata, comandante.
pub const PERSONAGGI: [Personaggio; 5] = [
    Personaggio {
        nome: "Vera",
        ruolo: "Ingegnera di bordo",
        ritratto: 0,
    },
    Personaggio {
        nome: "Tomas",
        ruolo: "Medico di stazione",
        ritratto: 1,
    },
    Personaggio {
        nome: "Dario",
        ruolo: "Caposquadra",
        ritratto: 2,
    },
    Personaggio {
        nome: "Mira",
        ruolo: "Scienziata capo",
        ritratto: 3,
    },
    Personaggio {
        nome: "Ilse",
        ruolo: "Comandante",
        ritratto: 4,
    },
];

// indici in PERSONAGGI, per leggere le tabelle senza contare
const INGEGNERA: usize = 0;
const MEDICO: usize = 1;
const CAPOSQUADRA: usize = 2;
const SCIENZIATA: usize = 3;
const COMANDANTE: usize = 4;

/// Chi parla nel briefing di quale livello (1-based), e cosa dice.
/// Copertura: primo contatto (1, 2), primo livello generato (7) e ogni
/// multiplo di 5 — i traguardi della campagna.
const BATTUTE: [(usize, usize, &str); 13] = [
    (
        1,
        COMANDANTE,
        "Stazione nuova, equipaggio in arrivo. Tienili vivi: \
         al resto pensiamo dopo.",
    ),
    (
        2,
        INGEGNERA,
        "La corrente non salta il vuoto: o moduli attaccati, o corridoi. \
         Una rete senza reattore è ferraglia fredda.",
    ),
    (
        5,
        COMANDANTE,
        "Ultimo giro di rodaggio. Dopo di questo, il settore è tuo.",
    ),
    (
        7,
        CAPOSQUADRA,
        "Settore nuovo, carte vecchie: i detriti non sono segnati. \
         Costruisci dove c'è spazio, non dove è comodo.",
    ),
    (
        10,
        INGEGNERA,
        "Due reattori scaldano più del doppio di uno, fidati. \
         Radiatori pronti prima di accendere il secondo.",
    ),
    (
        15,
        MEDICO,
        "L'aria non viaggia da sola: la fa il life support, la consuma \
         la gente. Se la rete è lunga, portala fino in fondo.",
    ),
    (
        20,
        MEDICO,
        "Gli incidenti capitano. Quello che conta è chi respira dopo.",
    ),
    (
        25,
        CAPOSQUADRA,
        "Campo pieno di rocce. La squadra borbotta, io misuro: \
         costruiamo intorno, come sempre.",
    ),
    (
        30,
        CAPOSQUADRA,
        "Arrivano più in fretta di quanto costruiamo cuccette. \
         Non farmi scegliere chi dorme in corridoio.",
    ),
    (
        35,
        INGEGNERA,
        "Stazione densa, calore denso. Ogni modulo in più è una stufa: \
         dissipa prima di costruire.",
    ),
    (
        40,
        SCIENZIATA,
        "I laboratori sono il motivo per cui siamo qui. \
         Tienimeli accesi e i punti si contano da soli.",
    ),
    (
        45,
        COMANDANTE,
        "Questa non è più una stazione, è una colonia. \
         Comandala come tale: margini larghi, nervi saldi.",
    ),
    (
        50,
        COMANDANTE,
        "Ultimo settore. Tutto quello che sai, tutto insieme. \
         Portaci a casa.",
    ),
];

/// Chi presenta il modulo appena sbloccato, a livello completato.
/// I livelli sono quelli dei traguardi: 5=Batteria, 15=Serra, 25=Gru,
/// 35=Condotto termico, 45=Centro comando.
const SBLOCCHI: [(usize, usize, &str); 5] = [
    (
        5,
        INGEGNERA,
        "Ti ho montato una Batteria: mangia il surplus e lo ridà quando \
         la rete annaspa. Non è un reattore, è tempo comprato.",
    ),
    (
        15,
        MEDICO,
        "La Serra è pronta: chiede meno corrente del life support e fa \
         un po' d'aria quasi gratis. Ma scalda, occhio.",
    ),
    (
        25,
        CAPOSQUADRA,
        "La Gru è operativa: mettila accanto a una roccia e lasciala \
         lavorare. Quando ha finito sparisce, roccia compresa.",
    ),
    (
        35,
        INGEGNERA,
        "Condotto termico: dissipa quanto due radiatori in una cella \
         sola. Beve corrente, ma il calore non perdona.",
    ),
    (
        45,
        COMANDANTE,
        "Il Centro comando è tuo: la gente arriva al doppio della \
         velocità. Uno solo per stazione — il comando non si divide.",
    ),
];

/// La battuta del briefing del livello (1-based), se il livello ne ha una:
/// `(indice in PERSONAGGI, testo)`.
pub fn battuta_briefing(livello: usize) -> Option<(usize, &'static str)> {
    BATTUTE
        .iter()
        .find(|(l, _, _)| *l == livello)
        .map(|&(_, p, t)| (p, t))
}

/// L'annuncio del modulo sbloccato completando `livello_completato`
/// (1-based), se quel livello è un traguardo di sblocco.
pub fn annuncio_sblocco(livello_completato: usize) -> Option<(usize, &'static str)> {
    SBLOCCHI
        .iter()
        .find(|(l, _, _)| *l == livello_completato)
        .map(|&(_, p, t)| (p, t))
}

// ---------------- test ----------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indici_personaggio_validi_in_tutte_le_tabelle() {
        for &(_, p, _) in BATTUTE.iter().chain(&SBLOCCHI) {
            assert!(p < PERSONAGGI.len());
        }
        for (i, p) in PERSONAGGI.iter().enumerate() {
            assert_eq!(p.ritratto, i, "{}: ritratto fuori ordine", p.nome);
        }
    }

    #[test]
    fn livelli_delle_battute_dentro_la_campagna() {
        for &(l, _, _) in BATTUTE.iter().chain(&SBLOCCHI) {
            assert!((1..=50).contains(&l), "livello {l} fuori range");
        }
    }

    #[test]
    fn copertura_briefing_su_primo_contatto_e_traguardi() {
        for livello in [1, 2, 7, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50] {
            assert!(
                battuta_briefing(livello).is_some(),
                "livello {livello} senza battuta"
            );
        }
        assert!(battuta_briefing(3).is_none());
    }

    #[test]
    fn ogni_traguardo_di_sblocco_ha_il_suo_annuncio() {
        for livello in [5, 15, 25, 35, 45] {
            assert!(
                annuncio_sblocco(livello).is_some(),
                "sblocco del livello {livello} senza annuncio"
            );
        }
        assert!(annuncio_sblocco(10).is_none());
        assert!(annuncio_sblocco(50).is_none());
    }
}
