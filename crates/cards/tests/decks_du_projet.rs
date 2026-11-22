//! Vérifie que les 9 listes de `data/decks/` se lisent et respectent le format Silver Age.

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
fn les_9_decks_sont_presents() {
    assert_eq!(listes().len(), 9);
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
