//! Statistiques sur les listes de `data/decks/` : cartes uniques, recouvrement.
//!
//! Usage :
//!     cargo run -p fab-cards --bin deck-stats             # résumé
//!     cargo run -p fab-cards --bin deck-stats -- --status # tableau Markdown pour docs/cards/STATUS.md

use fab_cards::decklist::Decklist;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn main() {
    let status_mode = std::env::args().any(|a| a == "--status");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("dossier data/decks introuvable")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    let decks: Vec<Decklist> = files
        .iter()
        .map(|p| Decklist::parse(&fs::read_to_string(p).unwrap()).unwrap())
        .collect();

    // Clé = (nom, couleur) pour les cartes de deck ; (nom, "") pour l'arène.
    let mut deck_cards: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut arena_cards: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for d in &decks {
        for e in &d.deck {
            deck_cards
                .entry((e.name.clone(), e.color.to_string()))
                .or_default()
                .insert(d.hero.clone());
        }
        for e in &d.arena {
            arena_cards
                .entry(e.name.clone())
                .or_default()
                .insert(d.hero.clone());
        }
    }
    let names: BTreeSet<&String> = deck_cards.keys().map(|(n, _)| n).collect();

    if status_mode {
        println!("| Carte | Couleur | Utilisée par | Statut |");
        println!("|---|---|---|---|");
        for d in &decks {
            println!("| **{}** | héros | {} | NotCoded |", d.hero, d.hero);
        }
        for (name, heroes) in &arena_cards {
            let h: Vec<_> = heroes.iter().map(|s| s.as_str()).collect();
            println!("| {name} | arène | {} | NotCoded |", h.join(", "));
        }
        for ((name, color), heroes) in &deck_cards {
            let h: Vec<_> = heroes.iter().map(|s| s.as_str()).collect();
            println!("| {name} | {color} | {} | NotCoded |", h.join(", "));
        }
        return;
    }

    println!("Decks : {}", decks.len());
    for d in &decks {
        println!(
            "  {:<28} {:<22} arène {:>2}  deck {:>2}",
            d.hero,
            d.class,
            d.arena_count(),
            d.deck_count()
        );
    }
    println!();
    println!("Héros différents             : {}", decks.len());
    println!("Cartes d'arène différentes   : {}", arena_cards.len());
    println!("Cartes de deck (nom+couleur) : {}", deck_cards.len());
    println!("Noms de cartes de deck       : {}", names.len());
    println!(
        "Total à coder (héros + arène + noms de deck) : {}",
        decks.len() + arena_cards.len() + names.len()
    );
    println!();
    println!("Cartes partagées par plusieurs decks :");
    let mut shared: Vec<(usize, String)> = arena_cards
        .iter()
        .map(|(n, h)| (h.len(), format!("{n} [arène]")))
        .chain(
            deck_cards
                .iter()
                .map(|((n, c), h)| (h.len(), format!("{n} ({c})"))),
        )
        .filter(|(k, _)| *k > 1)
        .collect();
    shared.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (k, n) in shared {
        println!("  {k} decks : {n}");
    }
}
