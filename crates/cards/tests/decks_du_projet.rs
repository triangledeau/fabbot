//! Vérifie que les listes de `data/decks/` se lisent et respectent le format Silver Age.

use fab_cards::decklist::Decklist;
use fab_engine::FormatConfig;
use std::fs;
use std::path::PathBuf;

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn listes() -> Vec<(String, Decklist)> {
    let mut files: Vec<_> = fs::read_dir(racine().join("data/decks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let text = fs::read_to_string(&p).unwrap();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            let d = Decklist::parse(&text).unwrap_or_else(|e| panic!("{name} : {e}"));
            (name, d)
        })
        .collect()
}

#[test]
fn les_9_decks_du_depart_sont_presents() {
    let noms = fab_cards::decks::noms();
    for d in [
        "ira",
        "hala",
        "dorinthea",
        "bravo",
        "iyslander",
        "florian",
        "levia",
        "dromai",
        "arakni",
    ] {
        assert!(noms.iter().any(|n| n == d), "{d} absent de {noms:?}");
    }
    assert_eq!(noms.len(), listes().len());
}

/// Un fichier `.txt` mal nommé serait ignoré sans bruit par
/// `fab_cards::decks` : on refuse donc tout autre nom que `NN_nom.txt`.
#[test]
fn chaque_liste_de_deck_est_nommee_numero_tiret_bas_nom() {
    for (fichier, _) in listes() {
        assert!(
            fab_cards::decks::nom_court(&fichier).is_some(),
            "{fichier} : nom attendu « NN_nom.txt » (ex. 10_kayo.txt)"
        );
    }
}

/// Tout le card-pool de chaque deck est codé (un deck en cours de codage
/// attend dans `data/decks/en_cours/`).
#[test]
fn toutes_les_cartes_de_chaque_deck_sont_codees() {
    let db = fab_cards::real::build();
    for d in fab_cards::decks::noms() {
        let pool = fab_cards::decks::charger_pool(&db, &d).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(pool.arena.len() + pool.deck.len(), 55, "{d}");
    }
}

#[test]
fn chaque_deck_respecte_le_format_silver_age() {
    let text = fs::read_to_string(racine().join("config/silver_age.toml")).unwrap();
    let format = FormatConfig::from_toml(&text).unwrap();
    for (name, d) in listes() {
        let problems = d.check(&format);
        assert!(problems.is_empty(), "{name} : {problems:?}");
        assert_eq!(d.arena_count() + d.deck_count(), 55, "{name}");
    }
}

/// Les decks Hala et Dorinthea sont entièrement codés : un choix d'équipement
/// et de 40 cartes se construit, et 300 parties entre bots aléatoires se
/// jouent sans incohérence (vérification après chaque décision).
#[test]
fn hala_et_dorinthea_jouent_avec_les_vraies_cartes() {
    use fab_ai::{play_out, RandomAgent};
    use fab_engine::{GameSetup, GameState, PlayerId};
    use std::sync::Arc;
    let db = Arc::new(fab_cards::real::build());
    let lire = |f: &str| {
        let text = fs::read_to_string(racine().join("data/decks").join(f)).unwrap();
        let list = Decklist::parse(&text).unwrap();
        // Tout le card-pool de 55 doit être codé.
        for e in &list.deck {
            let c = match e.color {
                fab_cards::decklist::Color::Red => fab_engine::Color::Red,
                fab_cards::decklist::Color::Yellow => fab_engine::Color::Yellow,
                fab_cards::decklist::Color::Blue => fab_engine::Color::Blue,
            };
            fab_cards::real::find(&db, &e.name, Some(c)).unwrap();
        }
        for e in &list.arena {
            fab_cards::real::find(&db, &e.name, None).unwrap();
        }
        fab_cards::real::default_loadout(&db, &list).unwrap()
    };
    let lo = [lire("02_hala.txt"), lire("03_dorinthea.txt")];
    assert_eq!(lo[0].deck.len(), 40);
    assert_eq!(lo[1].deck.len(), 40);
    for seed in 0..300u64 {
        let setup = GameSetup {
            players: lo.clone(),
            chooser: PlayerId((seed % 2) as u8),
            seed,
            shuffle: true,
            max_turns: 200,
        };
        let mut g = GameState::new(db.clone(), setup).unwrap();
        let mut a = RandomAgent::new(seed ^ 0xA5A5);
        let mut b = RandomAgent::new(seed ^ 0x5A5A);
        play_out(&mut g, [&mut a, &mut b], true).unwrap_or_else(|e| panic!("graine {seed} : {e}"));
    }
}
