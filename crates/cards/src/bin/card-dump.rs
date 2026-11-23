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
            (Types::DEMI_HERO, "Demi-Hero"),
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
            (Subtypes::HAMMER, "Hammer"),
            (Subtypes::ALLY, "Ally"),
            (Subtypes::DEMON, "Demon"),
            (Subtypes::DRAGON, "Dragon"),
            (Subtypes::INVOCATION, "Invocation"),
            (Subtypes::ASH, "Ash"),
            (Subtypes::ITEM, "Item"),
            (Subtypes::SCEPTER, "Scepter"),
            (Subtypes::TRAP, "Trap"),
            (Subtypes::DISEASE, "Disease"),
        ] {
            if d.subtypes.has(t) {
                words.push(w);
            }
        }
        for (t, w) in [
            (Supertypes::WARRIOR, "Warrior"),
            (Supertypes::NINJA, "Ninja"),
            (Supertypes::BRUTE, "Brute"),
            (Supertypes::SHADOW, "Shadow"),
            (Supertypes::DRACONIC, "Draconic"),
            (Supertypes::ILLUSIONIST, "Illusionist"),
            (Supertypes::ASSASSIN, "Assassin"),
            (Supertypes::CHAOS, "Chaos"),
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
            (Keywords::BLOOD_DEBT, "Blood Debt"),
            (Keywords::PHANTASM, "Phantasm"),
            (Keywords::SPECTRA, "Spectra"),
            (Keywords::STEALTH, "Stealth"),
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
        if d.ward > 0 {
            kw.push("Ward");
        }
        // Face arrière d'une carte retournable (CR 9.1.3) : comparée à la
        // face 2 de la carte (nommée par sa face avant) dans le fichier officiel.
        let back_of = d
            .flip
            .filter(|f| f.back.0 as usize == i)
            .map(|f| db.get(f.front).name.clone());
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
            "back_of": back_of,
        });
        println!("{line}");
    }
}
