//! Liste les vraies cartes codées avec leurs valeurs imprimées, une par ligne
//! (format JSON), pour `tools/check_coded_cards.py` qui les compare au
//! fichier officiel.
//!
//! Usage : cargo run -q -p fab-cards --bin card-dump

use fab_cards::real;
use fab_engine::card::{Color, Keywords, Subtypes, Supertypes, Types};
use serde_json::json;

fn main() {
    let db = real::build();
    for i in 0..db.len() {
        let d = db.get(fab_engine::CardId(i as u16));
        // Carte double : seule la carte entière est dans le fichier officiel
        // (ses côtés et sa version fusionnée sont des définitions internes).
        if d.split.is_some_and(|s| s.whole.0 as usize != i) {
            continue;
        }
        let color = d.color.map(|c| match c {
            Color::Red => "Red",
            Color::Yellow => "Yellow",
            Color::Blue => "Blue",
        });
        let mut words: Vec<&str> = Vec::new();
        for (t, w) in [
            (Types::ACTION, "Action"),
            (Types::ATTACK_REACTION, "Attack Reaction"),
            (Types::DEFENSE_REACTION, "Defense Reaction"),
            (Types::INSTANT, "Instant"),
            (Types::EQUIPMENT, "Equipment"),
            (Types::WEAPON, "Weapon"),
            (Types::HERO, "Hero"),
            (Types::TOKEN, "Token"),
            (Types::BLOCK, "Block"),
        ] {
            if d.types.has(t) {
                words.push(w);
            }
        }
        for (t, w) in [
            (Subtypes::ATTACK, "Attack"),
            (Subtypes::TWO_HAND, "(2H)"),
            (Subtypes::ONE_HAND, "(1H)"),
            (Subtypes::HEAD, "Head"),
            (Subtypes::CHEST, "Chest"),
            (Subtypes::ARMS, "Arms"),
            (Subtypes::LEGS, "Legs"),
            (Subtypes::AURA, "Aura"),
            (Subtypes::YOUNG, "Young"),
            (Subtypes::SWORD, "Sword"),
            (Subtypes::AXE, "Axe"),
            (Subtypes::DAGGER, "Dagger"),
        ] {
            if d.subtypes.has(t) {
                words.push(w);
            }
        }
        for (t, w) in [
            (Supertypes::WARRIOR, "Warrior"),
            (Supertypes::NINJA, "Ninja"),
        ] {
            if d.supertypes.has(t) {
                words.push(w);
            }
        }
        let mut kw: Vec<&str> = Vec::new();
        for (k, w) in [
            (Keywords::GO_AGAIN, "Go again"),
            (Keywords::DOMINATE, "Dominate"),
            (Keywords::BATTLEWORN, "Battleworn"),
            (Keywords::TEMPER, "Temper"),
            (Keywords::GUARDWELL, "Guardwell"),
            (Keywords::BLADE_BREAK, "Blade Break"),
            (Keywords::AMBUSH, "Ambush"),
            (Keywords::EPHEMERAL, "Ephemeral"),
            (Keywords::SUSPENSE, "Suspense"),
            (Keywords::CRUSH, "Crush"),
        ] {
            if d.keywords.has(k) {
                kw.push(w);
            }
        }
        // « Go again » d'une capacité activée (ex. Hala).
        if d.activated.iter().any(|a| a.go_again) {
            kw.push("Go again");
        }
        if d.arcane_barrier > 0 {
            kw.push("Arcane Barrier");
        }
        let line = json!({
            "name": d.name,
            "color": color,
            "pitch": d.pitch,
            "cost": d.cost,
            "power": d.power,
            "defense": d.defense,
            "life": d.life,
            "intellect": d.intellect,
            "arcane_barrier": d.arcane_barrier,
            "typebox": words,
            "keywords": kw,
        });
        println!("{line}");
    }
}
