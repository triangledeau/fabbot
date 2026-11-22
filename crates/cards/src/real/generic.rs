//! Cartes génériques (sans classe) des decks Hala et Dorinthea.

use super::{by_color, deck_card, equipment, power, COLORS};
use fab_engine::card::{CardDb, CardDef, Color, Delta, Keywords, Subtypes, Supertypes, Types};
use fab_engine::script::{Ask, Op};
use fab_engine::{Choice, Event, GameState, ObjectId, TriggerDef};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Arcane Polarity",
    "Brutal Assault",
    "Raging Onslaught",
    "Chest Puff",
    "Look Tuff",
    "Fyendal's Fighting Spirit",
    "Humble",
    "Sirens of Safe Harbor",
    "Lay Low",
    "On the Horizon",
    "Sift",
    "Nullrune Hood",
    "Nullrune Robe",
    "Nullrune Gloves",
    "Nullrune Boots",
    "Blade Beckoner Boots",
];

const G: Supertypes = Supertypes::NONE;

/// « When this attacks ».
fn when_this_attacks(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Attack { attack, .. } if *attack == me)
}

/// « When this defends ».
fn when_this_defends(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Defend { card, .. } if *card == me)
}

/// Chest Puff, Look Tuff : « When this attacks, it gets -1{p} unless you pay {r}. »
fn minus_one_unless_you_pay() -> TriggerDef {
    TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![],
        script: vec![
            // Le joueur ne choisit que s'il peut payer.
            Op::Ask(|g, c| {
                (g.available_resources(c.controller, None) >= 1).then(|| Ask {
                    player: c.controller,
                    label: "Payer {r} ? Sinon, l'attaque a -1{p}",
                    options: vec![Choice::Yes, Choice::No],
                })
            }),
            Op::Do(|g, c| {
                if c.answer == Some(Choice::Yes) {
                    g.pay(c.controller, 1);
                } else if let Some(s) = c.source {
                    g.modify(s, power(-1), fab_engine::Until::EndOfTurn);
                }
            }),
        ],
    }
}

pub fn add(db: &mut CardDb) {
    // Arcane Polarity — Generic Instant
    // « Gain 1{h}. If you've been dealt arcane damage this turn, instead gain 4{h} » (3, 2).
    for c in COLORS {
        let mut d = deck_card("Arcane Polarity", c, G, Types::INSTANT, Some(0), None, None);
        d.resolve = vec![Op::Do(|g, c| {
            // CR 6.4.7 : remplacement interne à la capacité.
            let n = if g.arcane_damage_taken_this_turn(c.controller) > 0 {
                by_color(g, c, [4, 3, 2])
            } else {
                1
            };
            g.gain_life(c.controller, n as u16);
        })];
        db.add(d);
    }

    // Brutal Assault — Generic Action - Attack, sans texte. 6/5/4, coût 2, déf. 3.
    // Raging Onslaught — Generic Action - Attack, sans texte. 7/6/5, coût 3, déf. 3.
    for (i, c) in COLORS.into_iter().enumerate() {
        let i = i as u8;
        db.add(deck_card(
            "Brutal Assault",
            c,
            G,
            Types::ACTION,
            Some(2),
            Some(6 - i),
            Some(3),
        ));
        db.add(deck_card(
            "Raging Onslaught",
            c,
            G,
            Types::ACTION,
            Some(3),
            Some(7 - i),
            Some(3),
        ));
    }

    // Chest Puff (rouge) — Generic Action - Attack, 7, coût 2, déf. 3.
    // « When this attacks, it gets -1{p} unless you pay {r}. »
    let mut d = deck_card(
        "Chest Puff",
        Color::Red,
        G,
        Types::ACTION,
        Some(2),
        Some(7),
        Some(3),
    );
    d.triggers = vec![minus_one_unless_you_pay()];
    db.add(d);
    // Look Tuff (rouge) — Generic Action - Attack, 8, coût 3, déf. 3. Même texte.
    let mut d = deck_card(
        "Look Tuff",
        Color::Red,
        G,
        Types::ACTION,
        Some(3),
        Some(8),
        Some(3),
    );
    d.triggers = vec![minus_one_unless_you_pay()];
    db.add(d);

    // Fyendal's Fighting Spirit — Generic Action - Attack, 7/6/5, coût 3, déf. 2.
    // « When this attacks or defends, if you have less {h} than an opposing
    // hero, gain 1{h}. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Fyendal's Fighting Spirit",
            c,
            G,
            Types::ACTION,
            Some(3),
            Some(7 - i as u8),
            Some(2),
        );
        d.triggers = vec![TriggerDef {
            when: |g, me, e| when_this_attacks(g, me, e) || when_this_defends(g, me, e),
            anywhere: false,
            targets: vec![],
            // CR 6.6.1c : la condition « if » fait partie de l'effet, elle est
            // vérifiée à la résolution.
            script: vec![Op::Do(|g, c| {
                let p = c.controller;
                if g.player(p).life < g.player(p.opponent()).life {
                    g.gain_life(p, 1);
                }
            })],
        }];
        db.add(d);
    }

    // Humble — Generic Action - Attack, 6/5/4, coût 2, déf. 2.
    // « When this hits a hero, that hero loses all abilities until the end of
    // their next turn. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Humble",
            c,
            G,
            Types::ACTION,
            Some(2),
            Some(6 - i as u8),
            Some(2),
        );
        d.triggers = vec![TriggerDef {
            when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(Event::Hit { target, .. }) = c.event {
                    let hero = g.player(target).hero;
                    let lose = Delta {
                        lose_abilities: true,
                        ..Default::default()
                    };
                    let until = g.end_of_next_turn(target);
                    g.modify(g.obj_ref(hero), lose, until);
                }
            })],
        }];
        db.add(d);
    }

    // Sirens of Safe Harbor — Generic Action - Attack, 6/5/4, coût 2, déf. 2.
    // « When this is put into your graveyard from anywhere, gain 1{h}. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Sirens of Safe Harbor",
            c,
            G,
            Types::ACTION,
            Some(2),
            Some(6 - i as u8),
            Some(2),
        );
        d.triggers = vec![TriggerDef {
            when: |g, me, e| {
                matches!(e, Event::ToGraveyard { card, owner, .. }
                    if *card == me && *owner == g.obj(me).owner)
            },
            anywhere: true,
            targets: vec![],
            script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
        }];
        db.add(d);
    }

    // Lay Low (jaune) — Generic Defense Reaction, coût 0, déf. 3.
    // « If you are marked, you can't play this. If the attacking hero is
    // marked, their next attack this turn gets -1{p}. »
    let mut d = deck_card(
        "Lay Low",
        Color::Yellow,
        G,
        Types::DEFENSE_REACTION,
        Some(0),
        None,
        Some(3),
    );
    d.play_condition = Some(|g, p, _| !g.player(p).marked);
    d.resolve = vec![Op::Do(|g, _| {
        if let Some(attacker) = g.active_link().map(|l| l.attacker) {
            if g.player(attacker).marked {
                g.next_attack(attacker, None, power(-1), false);
            }
        }
    })];
    db.add(d);

    // On the Horizon — Generic Block, déf. 4/3/2, sans coût.
    // « When this defends, look at the top card of your deck. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "On the Horizon",
            c,
            G,
            Types::BLOCK,
            None,
            None,
            Some(4 - i as u8),
        );
        d.triggers = vec![TriggerDef {
            when: when_this_defends,
            anywhere: false,
            targets: vec![],
            // TODO(info) : regarder une carte ne change pas l'état de la
            // partie ; ce sera une information donnée au joueur quand le
            // moteur gérera ce que chaque joueur sait (bots B2+, interface).
            script: vec![Op::Do(|_, _| {})],
        }];
        db.add(d);
    }

    // Sift — Generic Action, coût 0, déf. 3.
    // « Put up to 4 (3, 2) cards from your hand on the bottom of your deck,
    // then draw that many cards. Go again »
    for c in COLORS {
        let mut d = deck_card("Sift", c, G, Types::ACTION, Some(0), None, Some(3));
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![
            Op::Ask(|g, c| {
                let max = by_color(g, c, [4, 3, 2]) as usize;
                if c.picked.len() >= max || g.player(c.controller).hand.is_empty() {
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
                    label: "Carte de la main à mettre sous le deck (Pass : arrêter)",
                    options,
                })
            }),
            Op::Do(|g, c| {
                if let Some(Choice::Object(x)) = c.answer.take() {
                    g.put_on_bottom(x);
                    c.picked.push(x);
                    c.flag = true;
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
    }

    // Nullrune Hood / Robe / Gloves / Boots — Generic Equipment, déf. 0.
    // « Arcane Barrier 1 »
    for (name, slot) in [
        ("Nullrune Hood", Subtypes::HEAD),
        ("Nullrune Robe", Subtypes::CHEST),
        ("Nullrune Gloves", Subtypes::ARMS),
        ("Nullrune Boots", Subtypes::LEGS),
    ] {
        let mut d = equipment(name, G, slot, 0);
        d.arcane_barrier = 1;
        db.add(d);
    }

    // Blade Beckoner Boots — Generic Equipment - Legs, déf. 1.
    // « This gets +1{d} while defending a weapon attack. Guardwell »
    let mut d: CardDef = equipment("Blade Beckoner Boots", G, Subtypes::LEGS, 1);
    d.keywords = Keywords::GUARDWELL;
    d.self_mod = Some(|g, me| {
        let defending_weapon_attack = g.obj(me).zone == fab_engine::Zone::CombatChain
            && g.combat.links.iter().any(|l| {
                l.defenders.contains(&me)
                    && l.source.is_some_and(|w| g.def(w).types.has(Types::WEAPON))
            });
        if defending_weapon_attack {
            Delta {
                defense: 1,
                ..Default::default()
            }
        } else {
            Delta::default()
        }
    });
    db.add(d);
}
