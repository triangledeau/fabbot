//! Vérifie une liste de deck (nouvelle ou existante) et dit ce qui manque
//! pour qu'elle devienne un deck du projet (procédure : `docs/nouveau-deck.md`).
//!
//! Usage :
//!     cargo run -p fab-cards --bin verifier-deck -- data/decks/en_cours/10_kayo.txt
//!     cargo run -p fab-cards --bin verifier-deck -- ira        # un deck du projet
//!     cargo run -p fab-cards --bin verifier-deck               # tous les decks du projet
//!
//! Pour chaque liste : le format Silver Age (55 cartes, 40 au moins dans le
//! deck, copies, cartes bannies, héros au banc), les cartes **pas encore
//! codées** dans `crates/cards/src/real/`, et les cartes absentes du
//! tableau `docs/cards/STATUS.md`. Code de sortie 1 s'il reste un problème.

use fab_cards::decklist::{Color, Decklist};
use fab_cards::{decks, real};
use fab_engine::{CardDb, FormatConfig};
use std::path::PathBuf;

fn couleur(c: Color) -> fab_engine::Color {
    match c {
        Color::Red => fab_engine::Color::Red,
        Color::Yellow => fab_engine::Color::Yellow,
        Color::Blue => fab_engine::Color::Blue,
    }
}

fn nom_couleur(c: Color) -> &'static str {
    match c {
        Color::Red => "red",
        Color::Yellow => "yellow",
        Color::Blue => "blue",
    }
}

/// Vérifie une liste ; renvoie le nombre de problèmes.
fn verifier(db: &CardDb, format: &FormatConfig, status: &str, chemin: &PathBuf) -> usize {
    println!("== {}", chemin.display());
    let texte = match std::fs::read_to_string(chemin) {
        Ok(t) => t,
        Err(e) => {
            println!("   ✗ lecture impossible : {e}");
            return 1;
        }
    };
    let liste = match Decklist::parse(&texte) {
        Ok(l) => l,
        Err(e) => {
            println!("   ✗ liste illisible : {e}");
            return 1;
        }
    };
    println!(
        "   héros : {} ({}), {} cartes d'arène, {} cartes de deck",
        liste.hero,
        liste.class,
        liste.arena_count(),
        liste.deck_count()
    );
    let mut problemes = 0;
    let fichier = chemin.file_name().unwrap_or_default().to_string_lossy();
    if decks::nom_court(&fichier).is_none() {
        println!("   ✗ nom de fichier : « NN_nom.txt » attendu (ex. 10_kayo.txt)");
        problemes += 1;
    }
    for p in liste.check(format) {
        println!("   ✗ format Silver Age : {p}");
        problemes += 1;
    }
    if liste.arena_count() + liste.deck_count() != format.card_pool_max as u32 {
        println!(
            "   ⚠ card-pool de {} cartes (les decks du projet en ont {})",
            liste.arena_count() + liste.deck_count(),
            format.card_pool_max
        );
    }

    // Cartes codées ?
    let mut a_coder: Vec<String> = Vec::new();
    let mut deja = 0;
    let mut voir = |nom: &str, c: Option<Color>| {
        if db.find(nom, c.map(couleur)).is_some() {
            deja += 1;
        } else {
            let n = match c {
                Some(c) => format!("{nom} ({})", nom_couleur(c)),
                None => nom.to_string(),
            };
            if !a_coder.contains(&n) {
                a_coder.push(n);
            }
        }
    };
    voir(&liste.hero, None);
    for e in &liste.arena {
        voir(&e.name, None);
    }
    for e in &liste.deck {
        voir(&e.name, Some(e.color));
    }
    if a_coder.is_empty() {
        println!("   ✓ toutes les cartes sont codées");
    } else {
        println!(
            "   ✗ {} carte(s) à coder ({deja} lignes de la liste déjà codées, partagées avec d'autres decks) :",
            a_coder.len()
        );
        for n in &a_coder {
            println!("       - {n}");
        }
        problemes += a_coder.len();
    }

    // Lignes de STATUS.md (« | Nom | couleur | … »).
    let present = |nom: &str, coul: &str| {
        status.lines().any(|l| {
            let cols: Vec<&str> = l.split('|').map(|c| c.trim().trim_matches('*')).collect();
            cols.len() > 2 && cols[1] == nom && cols[2] == coul
        })
    };
    let mut absentes: Vec<String> = Vec::new();
    if !present(&liste.hero, "héros") {
        absentes.push(format!("{} (héros)", liste.hero));
    }
    for e in &liste.arena {
        if !present(&e.name, "arène") {
            absentes.push(format!("{} (arène)", e.name));
        }
    }
    for e in &liste.deck {
        let c = nom_couleur(e.color);
        if !present(&e.name, c) {
            absentes.push(format!("{} ({c})", e.name));
        }
    }
    absentes.dedup();
    if absentes.is_empty() {
        println!("   ✓ toutes les cartes ont leur ligne dans docs/cards/STATUS.md");
    } else {
        println!(
            "   ✗ {} ligne(s) absente(s) de docs/cards/STATUS.md :",
            absentes.len()
        );
        for n in &absentes {
            println!("       - {n}");
        }
        problemes += absentes.len();
    }
    if problemes == 0 {
        let dans_le_projet = chemin
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .zip(decks::dossier().canonicalize().ok())
            .is_some_and(|(a, b)| a == b);
        if dans_le_projet {
            println!("   ✓ deck complet");
        } else {
            println!(
                "   ✓ deck complet : il peut aller dans data/decks/ (étape 4 de docs/nouveau-deck.md)"
            );
        }
    }
    problemes
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let db = real::build();
    let format = FormatConfig::silver_age();
    let status = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/cards/STATUS.md"),
    )
    .unwrap_or_default();
    let chemins: Vec<PathBuf> = if args.is_empty() {
        decks::tous().into_iter().map(|d| d.fichier).collect()
    } else {
        args.iter()
            .map(|a| {
                let p = PathBuf::from(a);
                if p.exists() {
                    p
                } else {
                    match decks::trouver(a) {
                        Ok(d) => d.fichier,
                        Err(_) => p,
                    }
                }
            })
            .collect()
    };
    let mut total = 0;
    for c in &chemins {
        total += verifier(&db, &format, &status, c);
    }
    println!();
    if total == 0 {
        println!("Aucun problème.");
    } else {
        println!("{total} problème(s) à régler.");
        std::process::exit(1);
    }
}
