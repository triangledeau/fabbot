//! Cartes **fictives** pour tester le moteur de règles.
//!
//! Elles n'existent pas dans le vrai jeu : leurs noms sont en français et
//! préfixés « Essai ». Chacune reproduit un mécanisme rencontré dans les
//! vraies cartes des 9 decks (indiqué en commentaire), pour tester le moteur
//! avant de coder les vraies cartes.

use fab_engine::card::{
    AdditionalCost, CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, Types,
};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, CardId, Choice, Event, GameState, Loadout, ObjectId, PlayerId, TriggerDef, Until,
    Zone,
};

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

fn delta_power(n: i16) -> Delta {
    Delta {
        power: n,
        ..Default::default()
    }
}

/// Le héros adverse du contrôleur.
fn opposing_hero(_: &GameState, c: &Ctx, t: Target) -> bool {
    t == Target::Player(c.controller.opponent())
}

/// N'importe quel héros.
fn any_hero(_: &GameState, _: &Ctx, t: Target) -> bool {
    matches!(t, Target::Player(_))
}

/// L'attaque du maillon actif.
fn active_attack(g: &GameState, _: &Ctx, t: Target) -> bool {
    matches!(t, Target::Object(r) if g.active_attack() == Some(r.id))
}

/// Déclencheur : « when this hits ».
fn when_this_hits(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Hit { attack, .. } if *attack == me)
}

/// Déclencheur : « when this defends ».
fn when_this_defends(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Defend { card, .. } if *card == me)
}

/// Déclencheur : « at the beginning of your action phase ».
fn at_your_action_phase(g: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::BeginActionPhase(p) if *p == g.obj(me).controller)
}

/// Construit la base des cartes d'essai.
pub fn build() -> CardDb {
    let mut db = CardDb::new();
    let hero = |name: &str| CardDef {
        name: name.into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        ..Default::default()
    };
    db.add(hero("Essai Héros"));
    // Comme Ira : « Your second attack each turn gets +1{p} ».
    let mut ninja = hero("Essai Ninja");
    ninja.supertypes = Supertypes::NINJA;
    ninja.aura = Some(|g, src, target| {
        let me = g.obj(src).controller;
        let is_second = g.obj(target).zone == Zone::CombatChain
            && g.active_attack() == Some(target)
            && g.obj(target).controller == me
            && g.attacks_this_turn(me) == 2;
        if is_second {
            delta_power(1)
        } else {
            Delta::default()
        }
    });
    db.add(ninja);

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
        // « When this hits, gain 1{h} ».
        let mut d = deck_card("Essai Morsure", c, Types::ACTION, 1, Some(3 - i), Some(2));
        d.triggers = vec![TriggerDef {
            when: when_this_hits,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
        }];
        db.add(d);
        // Action non-attaque : go again, prochaine attaque +3/+2/+1.
        let mut d = deck_card("Essai Élan", c, Types::ACTION, 0, None, Some(2));
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(match i {
            0 => |g: &mut GameState, c: &mut Ctx| {
                g.next_attack(c.controller, None, delta_power(3), false)
            },
            1 => |g: &mut GameState, c: &mut Ctx| {
                g.next_attack(c.controller, None, delta_power(2), false)
            },
            _ => |g: &mut GameState, c: &mut Ctx| {
                g.next_attack(c.controller, None, delta_power(1), false)
            },
        })];
        db.add(d);
        // Réaction d'attaque : « Target attack gets +3/+2/+1{p} ».
        let mut d = deck_card("Essai Riposte", c, Types::ATTACK_REACTION, 0, None, Some(2));
        d.targets = vec![TargetSpec {
            label: "attaque",
            filter: active_attack,
            optional: false,
        }];
        d.resolve = vec![Op::Do(match i {
            0 => |g: &mut GameState, c: &mut Ctx| buff_target(g, c, 3),
            1 => |g: &mut GameState, c: &mut Ctx| buff_target(g, c, 2),
            _ => |g: &mut GameState, c: &mut Ctx| buff_target(g, c, 1),
        })];
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
        // Instant : 1 dégât au héros adverse ciblé.
        let mut d = deck_card("Essai Étincelle", c, Types::INSTANT, 1, None, None);
        d.targets = vec![TargetSpec {
            label: "héros adverse",
            filter: opposing_hero,
            optional: false,
        }];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(Target::Player(p)) = c.target(0) {
                g.deal_damage(c.source_id(), c.controller, p, 1, false);
            }
        })];
        db.add(d);
    }

    // Comme Hurl / Staunch Response : coût additionnel optionnel.
    let mut d = deck_card("Essai Rage", Color::Red, Types::ACTION, 0, Some(3), Some(2));
    d.additional_cost = Some(AdditionalCost {
        resources: 1,
        optional: true,
    });
    // « If the additional cost is paid, this gets +2{p} » : le paiement est
    // noté par un compteur sur la carte (elle garde son identité sur la pile
    // et la chaîne de combat).
    d.self_mod = Some(|g, me| {
        if g.obj(me).counters == 1 {
            delta_power(2)
        } else {
            Delta::default()
        }
    });
    d.effect_cost = vec![Op::Do(|g, c| {
        if c.optional_paid {
            let me = c.source_id().unwrap();
            g.obj_mut(me).counters = 1;
        }
    })];
    db.add(d);

    // Comme Razor Reflex / Pummel : « Choose 1 ; target attack gets +2{p}, or go again ».
    let mut d = deck_card(
        "Essai Dilemme",
        Color::Red,
        Types::ATTACK_REACTION,
        0,
        None,
        Some(2),
    );
    d.modes = vec!["+2 puissance", "go again"];
    d.targets = vec![TargetSpec {
        label: "attaque",
        filter: active_attack,
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Object(r)) = c.target(0) {
            let delta = if c.mode == 0 {
                delta_power(2)
            } else {
                Delta {
                    grants: Keywords::GO_AGAIN,
                    ..Default::default()
                }
            };
            g.modify(r, delta, Until::EndOfTurn);
        }
    })];
    db.add(d);

    // Comme Wrecker Romp : « As an additional cost to play this, discard a random card ».
    let mut d = deck_card(
        "Essai Sacrifice",
        Color::Red,
        Types::ACTION,
        1,
        Some(7),
        Some(3),
    );
    d.play_condition = Some(|g, p, me| g.player(p).hand.iter().any(|&c| c != me));
    d.effect_cost = vec![Op::Do(|g, c| {
        if let Some(x) = g.discard_random(c.controller) {
            c.picked.push(x);
        }
    })];
    db.add(d);

    // Comme Sirens of Safe Harbor : « When this is put into your graveyard from anywhere, gain 1{h} ».
    let mut d = deck_card(
        "Essai Sirène",
        Color::Blue,
        Types::ACTION,
        2,
        Some(4),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: |g, me, e| matches!(e, Event::ToGraveyard { card, owner, .. } if *card == me && *owner == g.obj(me).owner),
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
    }];
    db.add(d);

    // Comme On the Horizon / Test of Vigor : bloc avec « When this defends, … ».
    let mut d = CardDef {
        name: "Essai Vigie".into(),
        color: Some(Color::Blue),
        pitch: Some(3),
        defense: Some(3),
        types: Types::BLOCK,
        ..Default::default()
    };
    d.triggers = vec![TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_life(c.controller, 2))],
    }];
    db.add(d);

    // Comme Sift : « Put up to 2 cards from your hand on the bottom of your deck,
    // then draw that many cards. Go again ».
    let mut d = deck_card("Essai Tri", Color::Blue, Types::ACTION, 0, None, Some(3));
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![
        Op::Ask(|g, c| {
            if c.picked.len() >= 2 || g.player(c.controller).hand.is_empty() {
                return None;
            }
            let mut options: Vec<Choice> = g
                .player(c.controller)
                .hand
                .iter()
                .map(|&x| Choice::Object(x))
                .collect();
            options.push(Choice::Pass);
            Some(Ask {
                player: c.controller,
                options,
            })
        }),
        Op::Do(|g, c| {
            if let Some(Choice::Object(x)) = c.answer.take() {
                g.put_on_bottom(x);
                c.picked.push(x);
                c.flag = true; // on continue la boucle
            } else {
                c.flag = false;
            }
        }),
        Op::Back(|_, c| c.flag, 2),
        Op::Do(|g, c| {
            for _ in 0..c.picked.len() {
                g.draw(c.controller);
            }
        }),
    ];
    db.add(d);

    // Comme Toe the Line : « The next time you would be dealt damage this turn, prevent 2 of that damage ».
    let mut d = deck_card("Essai Garde", Color::Yellow, Types::INSTANT, 0, None, None);
    d.resolve = vec![Op::Do(|g, c| g.prevent_next(c.controller, 2, false, None))];
    db.add(d);

    // Comme Voltic Bolt (« Deal 5 arcane damage to target hero ») en plus petit.
    let mut d = deck_card("Essai Éclair", Color::Red, Types::ACTION, 1, None, Some(2));
    d.targets = vec![TargetSpec {
        label: "héros",
        filter: any_hero,
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Player(p)) = c.target(0) {
            g.deal_damage(c.source_id(), c.controller, p, 3, true);
        }
    })];
    db.add(d);

    // Comme Arcane Polarity : « Gain 1{h}. If you've been dealt arcane damage this turn, instead gain 4{h} ».
    let mut d = deck_card("Essai Polarité", Color::Red, Types::INSTANT, 0, None, None);
    d.resolve = vec![Op::Do(|g, c| {
        let n = if g.arcane_damage_taken_this_turn(c.controller) > 0 {
            4
        } else {
            1
        };
        g.gain_life(c.controller, n);
    })];
    db.add(d);

    // Comme Timesnap Potion : objet, « Action — Destroy this: Gain 2 action points ».
    let mut d = deck_card("Essai Potion", Color::Blue, Types::ACTION, 0, None, None);
    d.subtypes = Subtypes::ITEM;
    d.activated = vec![ActivatedDef {
        label: "Détruire : 2 points d'action",
        speed: Speed::Action,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| g.gain_action_points(c.controller, 2))],
        ..Default::default()
    }];
    db.add(d);

    // Comme Harvest Season : « Go again. At the beginning of your action phase,
    // destroy this, then gain 3{h} ».
    let mut d = deck_card("Essai Aura", Color::Yellow, Types::ACTION, 1, None, Some(2));
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: at_your_action_phase,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                g.destroy(s.id);
            }
            g.gain_life(c.controller, 3);
        })],
    }];
    db.add(d);
    // Une deuxième aura différente, pour tester l'ordre des déclenchements.
    let mut d = deck_card(
        "Essai Présage",
        Color::Yellow,
        Types::ACTION,
        0,
        None,
        Some(2),
    );
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: at_your_action_phase,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                g.destroy(s.id);
            }
            g.draw(c.controller);
        })],
    }];
    db.add(d);

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
    let weapon = |name: &str, power: u8, sub: Subtypes, cost: u8, go_again: bool| CardDef {
        name: name.into(),
        power: Some(power),
        types: Types::WEAPON,
        subtypes: sub,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost,
            once_per_turn: true,
            attack: true,
            go_again,
            ..Default::default()
        }],
        ..Default::default()
    };
    // « Once per Turn Action — {r} : Attack », 2H, puissance 3.
    db.add(weapon(
        "Essai Hache",
        3,
        Subtypes::TWO_HAND | Subtypes::AXE,
        1,
        false,
    ));
    // « Once per Turn Action — {r} : Attack. Go again », 1H, puissance 1.
    db.add(weapon(
        "Essai Dague",
        1,
        Subtypes::ONE_HAND | Subtypes::DAGGER,
        1,
        true,
    ));
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
    // Comme Blossom of Spring / Spellfire Cloak : « Instant — Destroy this: Gain {r} ».
    let mut b = equip("Essai Bouclier", Subtypes::CHEST, 1, Keywords::NONE);
    b.activated = vec![ActivatedDef {
        label: "Détruire : gagner 1 ressource",
        speed: Speed::Instant,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(b);
    db
}

fn buff_target(g: &mut GameState, c: &mut Ctx, n: i16) {
    if let Some(Target::Object(r)) = c.target(0) {
        g.modify(r, delta_power(n), Until::EndOfTurn);
    }
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
    deck.truncate(34);
    for (name, c) in [
        ("Essai Rage", Color::Red),
        ("Essai Dilemme", Color::Red),
        ("Essai Sacrifice", Color::Red),
        ("Essai Potion", Color::Blue),
        ("Essai Aura", Color::Yellow),
        ("Essai Présage", Color::Yellow),
    ] {
        deck.push(id(db, name, Some(c)));
    }
    Loadout {
        hero: id(db, "Essai Ninja", None),
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
            ("Essai Frappe", 2),
            ("Essai Assaut", 2),
            ("Essai Morsure", 2),
            ("Essai Parade", 3),
            ("Essai Élan", 1),
        ] {
            for _ in 0..n {
                deck.push(id(db, name, Some(c)));
            }
        }
    }
    for (name, c, n) in [
        ("Essai Rempart", Some(Color::Blue), 1),
        ("Essai Joyau", None, 1),
        ("Essai Sirène", Some(Color::Blue), 2),
        ("Essai Vigie", Some(Color::Blue), 2),
        ("Essai Tri", Some(Color::Blue), 1),
        ("Essai Garde", Some(Color::Yellow), 1),
        ("Essai Éclair", Some(Color::Red), 1),
        ("Essai Polarité", Some(Color::Red), 1),
    ] {
        for _ in 0..n {
            deck.push(id(db, name, c));
        }
    }
    assert_eq!(deck.len(), 40);
    Loadout {
        hero: id(db, "Essai Héros", None),
        arena: vec![
            id(db, "Essai Dague", None),
            id(db, "Essai Casque", None),
            id(db, "Essai Bouclier", None),
            id(db, "Essai Gantelets", None),
            id(db, "Essai Bottes", None),
        ],
        deck,
    }
}

/// Tous les joueurs du jeu d'essai (pour les tests).
pub const PLAYERS: [PlayerId; 2] = PlayerId::BOTH;

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
