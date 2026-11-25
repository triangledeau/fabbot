//! fab-ui — jouer contre le bot.
//!
//! - étape 2 : interface texte dans le terminal ([`jeu`], [`vue`]) ;
//! - étape 5 : page web locale ([`serveur`], [`web`] ; plateau, boutons de
//!   choix, « Annuler », journal), lancée par `cargo run --release -p fab-ui --bin fab-web`.

pub mod jeu;
pub mod serveur;
pub mod textes;
pub mod vue;
pub mod web;

use fab_cards::decklist::Decklist;
use fab_cards::real;
use fab_engine::{CardDb, GameSetup, GameState, Loadout, PlayerId};
use std::sync::Arc;

/// Les decks jouables dans l'interface : (nom court, fichier de `data/decks/`).
pub const DECKS: &[(&str, &str)] = &[
    ("ira", "01_ira.txt"),
    ("hala", "02_hala.txt"),
    ("dorinthea", "03_dorinthea.txt"),
    ("bravo", "04_bravo.txt"),
    ("iyslander", "05_iyslander.txt"),
    ("florian", "06_florian.txt"),
    ("levia", "07_levia.txt"),
    ("dromai", "08_dromai.txt"),
    ("arakni", "09_arakni.txt"),
];

/// Charge un deck jouable par son nom court, avec le choix provisoire
/// d'équipement et de 40 cartes (`real::default_loadout`).
pub fn charger_deck(db: &CardDb, nom: &str) -> Result<Loadout, String> {
    let fichier = DECKS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(nom))
        .map(|(_, f)| *f)
        .ok_or_else(|| {
            let noms: Vec<&str> = DECKS.iter().map(|(n, _)| *n).collect();
            format!(
                "deck inconnu : {nom} (decks jouables : {})",
                noms.join(", ")
            )
        })?;
    let chemin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/decks")
        .join(fichier);
    let texte = std::fs::read_to_string(&chemin)
        .map_err(|e| format!("lecture de {}: {e}", chemin.display()))?;
    let liste = Decklist::parse(&texte).map_err(|e| format!("{fichier} : {e:?}"))?;
    real::default_loadout(db, &liste)
}

/// Prépare une partie : l'humain est le joueur 0, le bot le joueur 1.
/// Le joueur qui choisit qui commence (CR 4.1.3) est tiré au sort.
pub fn nouvelle_partie(
    deck_humain: &str,
    deck_bot: &str,
    graine: u64,
) -> Result<GameState, String> {
    let db = Arc::new(real::build());
    let lo = [
        charger_deck(&db, deck_humain)?,
        charger_deck(&db, deck_bot)?,
    ];
    let setup = GameSetup {
        players: lo,
        chooser: PlayerId((graine % 2) as u8),
        seed: graine,
        shuffle: true,
        max_turns: 200,
    };
    let mut g = GameState::new(db, setup).map_err(|e| format!("mise en place : {e:?}"))?;
    g.record = Some(Vec::new());
    Ok(g)
}
