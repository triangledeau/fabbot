//! Cartes **fictives** pour tester le moteur de règles.
//!
//! Elles n'existent pas dans le vrai jeu : leurs noms sont en français et
//! préfixés « Essai ». Elles couvrent les mécanismes de l'étape 1 : attaques,
//! go again, dominate, réactions d'attaque et de défense, instants, blocs,
//! armes, équipements (blade break, battleworn, temper), déclenchements.

use fab_engine::card::{CardDb, CardDef, Color, Keywords, Op, Subtypes, Types, WeaponAttack};
use fab_engine::{CardId, Loadout};

fn pitch_of(c: Color) -> u8 {
    match c {
        Color::Red => 1,
        Color::Yellow => 2,
        Color::Blue => 3,
    }
}

fn deck_card(
    name: &str,
    color: Color,
    types: Types,
    cost: u8,
    power: Option<u8>,
    defense: Option<u8>,
) -> CardDef {
    CardDef {
        name: name.into(),
        color: Some(color),
        pitch: Some(pitch_of(color)),
        cost: Some(cost),
        power,
        defense,
        types,
        subtypes: if power.is_some() {
            Subtypes::ATTACK
        } else {
            Subtypes::NONE
        },
        ..Default::default()
    }
}

const COLORS: [Color; 3] = [Color::Red, Color::Yellow, Color::Blue];

/// Construit la base des cartes d'essai.
pub fn build() -> CardDb {
    let mut db = CardDb::new();
    // Héros
    db.add(CardDef {
        name: "Essai Héros".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        ..Default::default()
    });
    for (i, &c) in COLORS.iter().enumerate() {
        let i = i as u8;
        // Attaque simple : coût 1, puissance 4/3/2, défense 3.
        db.add(deck_card(
            "Essai Frappe",
            c,
            Types::ACTION,
            1,
            Some(4 - i),
            Some(3),
        ));
        // Attaque avec go again : coût 0, puissance 3/2/1.
        let mut d = deck_card("Essai Assaut", c, Types::ACTION, 0, Some(3 - i), Some(2));
        d.keywords = Keywords::GO_AGAIN;
        db.add(d);
        // Attaque avec dominate : coût 2, puissance 6/5/4.
        let mut d = deck_card(
            "Essai Écrasement",
            c,
            Types::ACTION,
            2,
            Some(6 - i),
            Some(3),
        );
        d.keywords = Keywords::DOMINATE;
        db.add(d);
        // Attaque qui fait gagner 1 PV quand elle touche.
        let mut d = deck_card("Essai Morsure", c, Types::ACTION, 1, Some(3 - i), Some(2));
        d.on_hit = vec![Op::GainLife(1)];
        db.add(d);
        // Action non-attaque : go again, prochaine attaque +2/+1/+1... ici +3-i.
        let mut d = deck_card("Essai Élan", c, Types::ACTION, 0, None, Some(2));
        d.keywords = Keywords::GO_AGAIN;
        d.on_resolve = vec![Op::NextAttackGetsPower(3 - i as i8)];
        db.add(d);
        // Réaction d'attaque : l'attaque gagne +3/+2/+1.
        let mut d = deck_card("Essai Riposte", c, Types::ATTACK_REACTION, 0, None, Some(2));
        d.on_resolve = vec![Op::ActiveAttackGetsPower(3 - i as i8)];
        db.add(d);
        // Réaction de défense : défense 4/3/2.
        db.add(deck_card(
            "Essai Parade",
            c,
            Types::DEFENSE_REACTION,
            0,
            None,
            Some(4 - i),
        ));
        // Instant : 1 dégât au héros adverse.
        let mut d = deck_card("Essai Étincelle", c, Types::INSTANT, 1, None, None);
        d.on_resolve = vec![Op::DamageOpposingHero(1)];
        db.add(d);
    }
    // Bloc : ne peut pas être joué, défense 4 (CR 8.1.12).
    db.add(CardDef {
        name: "Essai Rempart".into(),
        color: Some(Color::Blue),
        pitch: Some(3),
        defense: Some(4),
        types: Types::BLOCK,
        ..Default::default()
    });
    // Ressource : ne peut pas être jouée (CR 8.1.7).
    db.add(CardDef {
        name: "Essai Joyau".into(),
        pitch: Some(3),
        types: Types::RESOURCE,
        ..Default::default()
    });
    // Arme à deux mains : « Once per Turn Action — {1} : Attack », puissance 3.
    db.add(CardDef {
        name: "Essai Hache".into(),
        power: Some(3),
        types: Types::WEAPON,
        subtypes: Subtypes::TWO_HAND,
        weapon_attack: Some(WeaponAttack {
            cost: 1,
            once_per_turn: true,
            go_again: false,
        }),
        ..Default::default()
    });
    // Arme à une main avec go again : « Once per Turn Action — {1} : Attack. Go again. »
    db.add(CardDef {
        name: "Essai Dague".into(),
        power: Some(1),
        types: Types::WEAPON,
        subtypes: Subtypes::ONE_HAND,
        weapon_attack: Some(WeaponAttack {
            cost: 1,
            once_per_turn: true,
            go_again: true,
        }),
        ..Default::default()
    });
    let equip = |name: &str, sub: Subtypes, def: u8, kw: Keywords| CardDef {
        name: name.into(),
        defense: Some(def),
        types: Types::EQUIPMENT,
        subtypes: sub,
        keywords: kw,
        ..Default::default()
    };
    db.add(equip(
        "Essai Casque",
        Subtypes::HEAD,
        1,
        Keywords::BLADE_BREAK,
    ));
    db.add(equip(
        "Essai Plastron",
        Subtypes::CHEST,
        2,
        Keywords::BATTLEWORN,
    ));
    db.add(equip(
        "Essai Gantelets",
        Subtypes::ARMS,
        1,
        Keywords::TEMPER,
    ));
    db.add(equip(
        "Essai Bottes",
        Subtypes::LEGS,
        1,
        Keywords::BLADE_BREAK,
    ));
    db
}

/// Raccourci : identifiant d'une carte d'essai (panique si elle n'existe pas).
pub fn id(db: &CardDb, name: &str, color: Option<Color>) -> CardId {
    db.find(name, color)
        .unwrap_or_else(|| panic!("carte d'essai inconnue : {name} {color:?}"))
}

/// Deck d'essai « agressif » : 40 cartes, Essai Hache + 4 équipements.
pub fn aggro_loadout(db: &CardDb) -> Loadout {
    let mut deck = Vec::new();
    for c in COLORS {
        for (name, n) in [
            ("Essai Frappe", 3),
            ("Essai Assaut", 3),
            ("Essai Écrasement", 2),
            ("Essai Morsure", 2),
            ("Essai Élan", 1),
            ("Essai Riposte", 1),
            ("Essai Étincelle", 1),
        ] {
            for _ in 0..n {
                deck.push(id(db, name, Some(c)));
            }
        }
    }
    deck.truncate(40);
    while deck.len() < 40 {
        deck.push(id(db, "Essai Frappe", Some(Color::Blue)));
    }
    Loadout {
        hero: id(db, "Essai Héros", None),
        arena: vec![
            id(db, "Essai Hache", None),
            id(db, "Essai Casque", None),
            id(db, "Essai Plastron", None),
            id(db, "Essai Gantelets", None),
            id(db, "Essai Bottes", None),
        ],
        deck,
    }
}

/// Deck d'essai « défensif » : réactions de défense, blocs, Essai Dague.
pub fn defensive_loadout(db: &CardDb) -> Loadout {
    let mut deck = Vec::new();
    for c in COLORS {
        for (name, n) in [
            ("Essai Frappe", 3),
            ("Essai Assaut", 2),
            ("Essai Morsure", 2),
            ("Essai Parade", 3),
            ("Essai Élan", 1),
            ("Essai Étincelle", 1),
        ] {
            for _ in 0..n {
                deck.push(id(db, name, Some(c)));
            }
        }
    }
    for _ in 0..2 {
        deck.push(id(db, "Essai Rempart", Some(Color::Blue)));
    }
    deck.push(id(db, "Essai Joyau", None));
    deck.truncate(40);
    while deck.len() < 40 {
        deck.push(id(db, "Essai Parade", Some(Color::Blue)));
    }
    Loadout {
        hero: id(db, "Essai Héros", None),
        arena: vec![
            id(db, "Essai Dague", None),
            id(db, "Essai Casque", None),
            id(db, "Essai Plastron", None),
            id(db, "Essai Gantelets", None),
            id(db, "Essai Bottes", None),
        ],
        deck,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_decks_d_essai_font_40_cartes() {
        let db = build();
        assert_eq!(aggro_loadout(&db).deck.len(), 40);
        assert_eq!(defensive_loadout(&db).deck.len(), 40);
    }
}
