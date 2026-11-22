//! Cartes génériques (sans classe) des decks Hala, Dorinthea, Ira et Bravo.

use super::{by_color, deck_card, equipment, grant, power, target_obj, COLORS};
use fab_engine::card::{CardDb, CardDef, Color, Delta, Keywords, Subtypes, Supertypes, Types};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, Speed, TriggerDef, Until, Zone,
};

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
    "Blade Beckoner Helm",
    "Bluster Buff",
    "Razor Reflex",
    "Reinforce the Line",
    "Scar for a Scar",
    "Might",
    "Vigor",
    "Quicken",
    "Pummel",
    "Unflinching Foothold",
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
    // Blade Beckoner Helm — Generic Equipment - Head, déf. 1.
    // « This gets +1{d} while defending a weapon attack. Guardwell »
    db.add(blade_beckoner("Blade Beckoner Boots", Subtypes::LEGS));
    db.add(blade_beckoner("Blade Beckoner Helm", Subtypes::HEAD));

    // Bluster Buff (rouge) — Generic Action - Attack, 6, coût 1, déf. 3.
    // « When this attacks, it gets -1{p} unless you pay {r}. »
    let mut d = deck_card(
        "Bluster Buff",
        Color::Red,
        G,
        Types::ACTION,
        Some(1),
        Some(6),
        Some(3),
    );
    d.triggers = vec![minus_one_unless_you_pay()];
    db.add(d);

    // Razor Reflex — Generic Attack Reaction, coût 1, déf. 2.
    // « Choose 1;
    // * Target dagger or sword weapon attack gets +3{p}.
    // * Target attack action card with cost 1 or less gets +3{p} and "When
    //   this hits, it gets **go again**." » (+3 / +2 / +1)
    for c in COLORS {
        let mut d = deck_card(
            "Razor Reflex",
            c,
            G,
            Types::ATTACK_REACTION,
            Some(1),
            None,
            Some(2),
        );
        d.modes = vec![
            "attaque d'arme dague ou épée",
            "carte d'attaque de coût 1 ou moins (+ go again si elle touche)",
        ];
        d.targets = vec![TargetSpec {
            label: "attaque",
            // La cible dépend du mode choisi (CR 1.7.5, 5.1.4).
            filter: |g, c, t| {
                let Target::Object(r) = t else { return false };
                if g.active_attack() != Some(r.id) {
                    return false;
                }
                if c.mode == 0 {
                    g.is_weapon_attack(r.id)
                        && g.def(r.id).subtypes.any(Subtypes::DAGGER | Subtypes::SWORD)
                } else {
                    is_attack_action_card(g, r.id) && g.def(r.id).cost.unwrap_or(0) <= 1
                }
            },
            optional: false,
        }];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                g.modify(r, power(by_color(g, c, [3, 2, 1])), Until::EndOfTurn);
                if c.mode == 1 {
                    g.grant_trigger(r, c.card, 0, Until::EndOfTurn);
                }
            }
        })];
        // La capacité accordée : « When this hits, it gets go again. »
        d.granted = vec![TriggerDef {
            when: this_hits,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source {
                    g.modify(s, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                }
            })],
        }];
        db.add(d);
    }

    // Reinforce the Line — Generic Instant, coût 0.
    // « Target defending attack action card gets +4{d}. » (+4 / +3 / +2)
    for c in COLORS {
        let mut d = deck_card(
            "Reinforce the Line",
            c,
            G,
            Types::INSTANT,
            Some(0),
            None,
            None,
        );
        d.targets = vec![TargetSpec {
            label: "carte d'attaque qui défend",
            // CR 7.0.5a : une carte défend tant qu'elle est sur la chaîne.
            filter: |g, _, t| {
                matches!(t, Target::Object(r) if g.obj(r.id).zone == Zone::CombatChain
                    && g.combat.links.iter().any(|l| l.defenders.contains(&r.id))
                    && is_attack_action_card(g, r.id))
            },
            optional: false,
        }];
        d.resolve = vec![Op::Do(|g, c: &mut Ctx| {
            if let Some(r) = target_obj(c, 0) {
                let d = Delta {
                    defense: by_color(g, c, [4, 3, 2]),
                    ..Default::default()
                };
                // CR 6.2.2a : sans durée écrite, l'effet finit à la fin du tour.
                g.modify(r, d, Until::EndOfTurn);
            }
        })];
        db.add(d);
    }

    // Scar for a Scar — Generic Action - Attack, 4/3/2, coût 0, déf. 2.
    // « When this is played, if you have less {h} than an opposing hero, it
    // gets **go again**. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Scar for a Scar",
            c,
            G,
            Types::ACTION,
            Some(0),
            Some(4 - i as u8),
            Some(2),
        );
        d.triggers = vec![TriggerDef {
            when: |_, me, e| matches!(e, Event::Played { card, .. } if *card == me),
            anywhere: false,
            targets: vec![],
            // CR 6.6.1c : la condition est vérifiée à la résolution.
            script: vec![Op::Do(|g, c| {
                let p = c.controller;
                if g.player(p).life < g.player(p.opponent()).life {
                    if let Some(s) = c.source {
                        g.modify(s, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                    }
                }
            })],
        }];
        db.add(d);
    }

    tokens(db);
    bravo_generics(db);
}

/// Tokens génériques Might, Vigor et Quicken.
fn tokens(db: &mut CardDb) {
    let token = |name: &str, trigger: TriggerDef| CardDef {
        name: name.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        triggers: vec![trigger],
        ..Default::default()
    };
    let start_of_your_turn = |g: &GameState, me: ObjectId, e: &Event| matches!(e, Event::StartOfTurn(p) if *p == g.obj(me).controller);

    // Might — Generic Token - Aura.
    // « At the start of your turn, destroy this, then your next attack this
    // turn gets +1{p}. »
    db.add(token(
        "Might",
        TriggerDef {
            when: start_of_your_turn,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                destroy_source(g, c);
                g.next_attack(c.controller, None, power(1), false);
            })],
        },
    ));

    // Vigor — Generic Token - Aura.
    // « At the start of your turn, destroy this, then gain {r}. »
    db.add(token(
        "Vigor",
        TriggerDef {
            when: start_of_your_turn,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                destroy_source(g, c);
                g.gain_resources(c.controller, 1);
            })],
        },
    ));

    // Quicken — Generic Token - Aura.
    // « When you play an attack action card or activate a weapon attack,
    // destroy this and the attack gets go again. »
    db.add(token(
        "Quicken",
        TriggerDef {
            when: |g, me, e| {
                let me = g.obj(me).controller;
                match *e {
                    Event::Played { player, card, .. } => {
                        player == me && is_attack_action_card(g, card)
                    }
                    Event::Activated {
                        player,
                        source,
                        ability,
                    } => {
                        let def = g.def(source);
                        player == me
                            && def.types.has(Types::WEAPON)
                            && def.activated[ability as usize].attack
                    }
                    _ => false,
                }
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                destroy_source(g, c);
                let attack = match c.event {
                    Some(Event::Played { card, .. }) => Some(card),
                    // L'attaque-proxy de l'arme, encore sur la pile.
                    Some(Event::Activated { source, .. }) => g
                        .stack
                        .iter()
                        .rev()
                        .map(|l| l.object.id)
                        .find(|&o| g.obj(o).proxy_of == Some(source)),
                    _ => None,
                };
                if let Some(a) = attack {
                    // Seulement si l'attaque est encore là (sur la pile ou la chaîne).
                    if matches!(g.obj(a).zone, Zone::Stack | Zone::CombatChain) {
                        g.modify(g.obj_ref(a), grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                    }
                }
            })],
        },
    ));
}

/// « Destroy this » : détruit la source de la capacité si elle est encore là.
pub(crate) fn destroy_source(g: &mut GameState, c: &Ctx) {
    if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
        g.destroy(s.id);
    }
}

/// Cartes génériques du deck Bravo.
fn bravo_generics(db: &mut CardDb) {
    // Pummel (rouge) — Generic Attack Reaction, coût 2, déf. 2.
    // « Choose 1;
    // * Target club or hammer weapon attack gets +4{p}.
    // * Target attack action card with cost 2 or more gets +4{p} and "When
    //   this hits a hero, they discard a card." »
    // Seule la version rouge est utilisée par nos decks.
    let mut d = deck_card(
        "Pummel",
        Color::Red,
        G,
        Types::ATTACK_REACTION,
        Some(2),
        None,
        Some(2),
    );
    d.modes = vec![
        "attaque d'arme massue ou marteau",
        "carte d'attaque de coût 2 ou plus (+ défausse si elle touche un héros)",
    ];
    d.targets = vec![TargetSpec {
        label: "attaque",
        // La cible dépend du mode choisi (CR 1.7.5, 5.1.4).
        filter: |g, c, t| {
            let Target::Object(r) = t else { return false };
            if g.active_attack() != Some(r.id) {
                return false;
            }
            if c.mode == 0 {
                g.is_weapon_attack(r.id)
                    && g.def(r.id).subtypes.any(Subtypes::CLUB | Subtypes::HAMMER)
            } else {
                is_attack_action_card(g, r.id) && g.def(r.id).cost.unwrap_or(0) >= 2
            }
        },
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(r) = target_obj(c, 0) {
            g.modify(r, power(4), Until::EndOfTurn);
            if c.mode == 1 {
                g.grant_trigger(r, c.card, 0, Until::EndOfTurn);
            }
        }
    })];
    // La capacité accordée : « When this hits a hero, they discard a card. »
    // Le joueur touché choisit la carte (CR 8.5.5).
    d.granted = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                let Some(Event::Hit { target, .. }) = c.event else {
                    return None;
                };
                let hand = &g.player(target).hand;
                (hand.len() > 1).then(|| Ask {
                    player: target,
                    label: "Carte à défausser",
                    options: hand.iter().map(|&x| Choice::Object(x)).collect(),
                })
            }),
            Op::Do(|g, c| {
                let Some(Event::Hit { target, .. }) = c.event else {
                    return;
                };
                let card = match c.answer {
                    Some(Choice::Object(x)) if g.obj(x).zone == Zone::Hand => Some(x),
                    _ => g.player(target).hand.first().copied(),
                };
                if let Some(x) = card {
                    g.discard(x);
                }
            }),
        ],
    }];
    db.add(d);

    // Unflinching Foothold — Generic Equipment - Legs, déf. 1.
    // « Instant -- Destroy this: Target attack loses and can't gain dominate.
    // Battleworn »
    let mut d = equipment("Unflinching Foothold", G, Subtypes::LEGS, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.activated = vec![ActivatedDef {
        label: "Détruire : l'attaque perd dominate",
        speed: Speed::Instant,
        targets: vec![super::TARGET_ATTACK],
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let d = Delta {
                    loses: Keywords::DOMINATE,
                    ..Default::default()
                };
                // Pas de durée écrite : tant que l'attaque existe.
                g.modify(r, d, Until::EndOfTurn);
            }
        })],
        ..Default::default()
    }];
    db.add(d);
}

/// « This gets +1{d} while defending a weapon attack. Guardwell »
fn blade_beckoner(name: &str, slot: Subtypes) -> CardDef {
    let mut d: CardDef = equipment(name, G, slot, 1);
    d.keywords = Keywords::GUARDWELL;
    d.self_mod = Some(|g, me| {
        let defending_weapon_attack = g.obj(me).zone == Zone::CombatChain
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
    d
}

/// Une carte d'action d'attaque (pas une attaque-proxy d'arme).
pub(crate) fn is_attack_action_card(g: &GameState, id: ObjectId) -> bool {
    let d = g.def(id);
    g.obj(id).proxy_of.is_none() && d.is_action() && d.is_attack_card()
}

/// « When this hits » pour une attaque `me` (carte ou attaque-proxy : pour une
/// proxy, l'événement « hit » désigne l'arme).
pub(crate) fn this_hits(g: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Hit { attack, .. }
        if *attack == me || g.obj(me).proxy_of == Some(*attack))
}
