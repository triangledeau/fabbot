//! Cartes génériques (sans classe) des 9 decks.

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
    "Agility",
    "Pummel",
    "Unflinching Foothold",
    "Blade Beckoner Plating",
    "Blade Beckoner Gauntlets",
    "Ravenous Rabble",
    "Wounded Bull",
    "Unmovable",
    "Timesnap Potion",
    "Deep Blue",
    "Seeker's Leggings",
    "Time Skippers",
    "Jack Be Quick",
    "Nimblism",
    "Oasis Respite",
    "Outed",
    "Snatch",
    "Public Bounty",
    "Pursue to the Edge of Oblivion",
    "Pursue to the Pits of Despair",
    "Relentless Pursuit",
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
            in_graveyard: false,
            in_banished: false,
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
            in_graveyard: false,
            in_banished: false,
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
    iyslander_generics(db);
    dromai_generics(db);
    arakni_generics(db);
}

/// Cartes génériques du deck Iyslander.
fn iyslander_generics(db: &mut CardDb) {
    // Blade Beckoner Plating — Generic Equipment - Chest, déf. 1.
    // Blade Beckoner Gauntlets — Generic Equipment - Arms, déf. 1.
    // « This gets +1{d} while defending a weapon attack. Guardwell »
    db.add(blade_beckoner("Blade Beckoner Plating", Subtypes::CHEST));
    db.add(blade_beckoner("Blade Beckoner Gauntlets", Subtypes::ARMS));

    // Ravenous Rabble (rouge) — Generic Action - Attack, 5, coût 0, déf. 2.
    // « When this attacks, reveal the top card of your deck. This gets -X{p},
    //   where X is the pitch value of the card revealed this way.
    //   Go again »
    let mut d = deck_card(
        "Ravenous Rabble",
        Color::Red,
        G,
        Types::ACTION,
        Some(0),
        Some(5),
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            // Deck vide : rien n'est révélé, X = 0 (CR 8.5.17).
            let Some(top) = g.top_of_deck(c.controller) else {
                return;
            };
            g.emit(Event::Revealed {
                player: c.controller,
                card: top,
            });
            let x = g.def(top).pitch_value() as i16;
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                if x > 0 {
                    g.modify(s, power(-x), Until::EndOfTurn);
                }
            }
        })],
    }];
    db.add(d);

    // Wounded Bull (rouge) — Generic Action - Attack, 7, coût 3, déf. 2.
    // « When this is played, if you have less {h} than an opposing hero, it
    //   gets +1{p}. »
    let mut d = deck_card(
        "Wounded Bull",
        Color::Red,
        G,
        Types::ACTION,
        Some(3),
        Some(7),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Played { card, .. } if *card == me),
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            // Condition vérifiée à la résolution du déclenchement.
            let me = g.player(c.controller).life;
            if me < g.player(c.controller.opponent()).life {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.modify(s, power(1), Until::EndOfTurn);
                }
            }
        })],
    }];
    db.add(d);

    // Unmovable (rouge) — Generic Defense Reaction, coût 3, déf. 7.
    // « If this was played from arsenal, it gets +1{d}. »
    let mut d = deck_card(
        "Unmovable",
        Color::Red,
        G,
        Types::DEFENSE_REACTION,
        Some(3),
        None,
        Some(7),
    );
    d.self_mod = Some(|g, me| {
        if !matches!(g.obj(me).zone, Zone::Stack | Zone::CombatChain) {
            return Delta::default();
        }
        // La dernière fois que cette carte a été jouée (ce tour).
        let from = g.turn_events().iter().rev().find_map(|e| match *e {
            Event::Played { card, from, .. } if card == me => Some(from),
            _ => None,
        });
        if from == Some(Zone::Arsenal) {
            Delta {
                defense: 1,
                ..Default::default()
            }
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Timesnap Potion (bleu) — Generic Action - Item, coût 0, pas de défense.
    // « Action -- Destroy this: Gain 2 action points. »
    let mut d = deck_card(
        "Timesnap Potion",
        Color::Blue,
        G,
        Types::ACTION,
        Some(0),
        None,
        None,
    );
    d.subtypes = Subtypes::ITEM;
    d.activated = vec![ActivatedDef {
        label: "Détruire : gagner 2 points d'action",
        speed: Speed::Action,
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.gain_action_points(c.controller, 2))],
        ..Default::default()
    }];
    db.add(d);
}

/// Tokens génériques Might, Vigor, Agility et Quicken.
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

    // Agility — Generic Token - Aura (deck Levia).
    // « At the start of your turn, destroy this, then your next attack this
    // turn gets go again. »
    db.add(token(
        "Agility",
        TriggerDef {
            when: start_of_your_turn,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                destroy_source(g, c);
                g.next_attack(c.controller, None, grant(Keywords::GO_AGAIN), false);
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
        in_graveyard: false,
        in_banished: false,
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

/// Opt 1 (CR 8.5.22) : le joueur regarde la carte du dessus de son deck, puis
/// la laisse dessus ou la met dessous (CR 8.5.22b : rien si le deck est vide).
pub(crate) fn opt1() -> Vec<Op> {
    vec![
        Op::Ask(|g, c| {
            g.top_of_deck(c.controller).map(|top| Ask {
                player: c.controller,
                label: "Opt 1 : garder cette carte sur le deck ? (Passer : la mettre dessous)",
                options: vec![Choice::Object(top), Choice::Pass],
            })
        }),
        Op::Do(|g, c| {
            if c.answer.take() == Some(Choice::Pass) {
                if let Some(top) = g.top_of_deck(c.controller) {
                    g.put_on_bottom(top);
                }
            }
        }),
    ]
}

/// Cartes génériques du deck Dromai.
fn dromai_generics(db: &mut CardDb) {
    // Deep Blue — Generic Equipment - Chest, déf. 0.
    // « Action -- Put a card from your hand on the bottom of your deck,
    //   destroy this: Gain {r}{r}{r}. Go again »
    let mut d = equipment("Deep Blue", G, Subtypes::CHEST, 0);
    d.activated = vec![ActivatedDef {
        label: "Carte de la main sous le deck, détruire : gagner {r}{r}{r}",
        speed: Speed::Action,
        go_again: true,
        condition: Some(|g, p, _| !g.player(p).hand.is_empty()),
        effect_cost: vec![
            Op::Ask(|g, c| {
                let hand = &g.player(c.controller).hand;
                (hand.len() > 1).then(|| Ask {
                    player: c.controller,
                    label: "Carte de la main à mettre sous le deck",
                    options: hand.iter().map(|&x| Choice::Object(x)).collect(),
                })
            }),
            Op::Do(|g, c| {
                let card = match c.answer.take() {
                    Some(Choice::Object(x)) if g.obj(x).zone == Zone::Hand => Some(x),
                    _ => g.player(c.controller).hand.first().copied(),
                };
                if let Some(x) = card {
                    g.put_on_bottom(x);
                }
                destroy_source(g, c);
            }),
        ],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 3))],
        ..Default::default()
    }];
    db.add(d);

    // Seeker's Leggings — Generic Equipment - Legs (pas de défense).
    // « Instant -- {r}, destroy this: Prevent the next 1 damage that would be
    //   dealt to you this turn. Opt 1 »
    let mut d = equipment("Seeker's Leggings", G, Subtypes::LEGS, 0);
    d.defense = None;
    let mut script = vec![Op::Do(|g, c| g.prevent_next(c.controller, 1, false, None))];
    script.extend(opt1());
    d.activated = vec![ActivatedDef {
        label: "{r}, détruire : prévenir 1 dégât, opt 1",
        speed: Speed::Instant,
        cost: 1,
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script,
        ..Default::default()
    }];
    db.add(d);

    // Time Skippers — Generic Equipment - Legs, déf. 0.
    // « Action -- {r}{r}{r}, destroy this: Gain 2 action points. »
    let mut d = equipment("Time Skippers", G, Subtypes::LEGS, 0);
    d.activated = vec![ActivatedDef {
        label: "{r}{r}{r}, détruire : gagner 2 points d'action",
        speed: Speed::Action,
        cost: 3,
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.gain_action_points(c.controller, 2))],
        ..Default::default()
    }];
    db.add(d);

    // Jack Be Quick (rouge) — Generic Action - Attack, coût 0, 3{p} 3{d}.
    // « When this attacks, you may banish a Nimblism from your graveyard. If
    //   you do, this gets +1{p} and go again.
    //   When this hits a hero, {u} an ally they control, then steal it until
    //   the end of this action phase. »
    let mut d = deck_card(
        "Jack Be Quick",
        Color::Red,
        G,
        Types::ACTION,
        Some(0),
        Some(3),
        Some(3),
    );
    d.triggers = vec![
        TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Ask(|g, c| {
                    let n: Vec<ObjectId> = nimblisms_in_graveyard(g, c.controller);
                    (!n.is_empty()).then(|| Ask {
                        player: c.controller,
                        label: "Bannir un Nimblism de votre cimetière (+1{p} et go again) ?",
                        options: n
                            .into_iter()
                            .map(Choice::Object)
                            .chain(std::iter::once(Choice::Pass))
                            .collect(),
                    })
                }),
                Op::Do(|g, c| {
                    if let Some(Choice::Object(x)) = c.answer.take() {
                        if g.obj(x).zone == Zone::Graveyard {
                            g.banish(c.controller, x, false);
                            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                                let delta = Delta {
                                    power: 1,
                                    grants: Keywords::GO_AGAIN,
                                    ..Default::default()
                                };
                                g.modify(s, delta, Until::EndOfTurn);
                            }
                        }
                    }
                }),
            ],
        },
        TriggerDef {
            when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Ask(|g, c| {
                    let allies = allies_of(g, hit_target(c)?);
                    (allies.len() > 1).then(|| Ask {
                        player: c.controller,
                        label: "Allié à dégager puis voler",
                        options: allies.into_iter().map(Choice::Object).collect(),
                    })
                }),
                Op::Do(|g, c| {
                    let Some(them) = hit_target(c) else { return };
                    let allies = allies_of(g, them);
                    let pick = match c.answer.take() {
                        Some(Choice::Object(x)) if allies.contains(&x) => Some(x),
                        _ => allies.first().copied(),
                    };
                    if let Some(x) = pick {
                        // {u} : dégager (CR 8.5.56), puis steal (CR 8.5.54).
                        g.obj_mut(x).tapped = false;
                        g.steal_until_end_of_action_phase(c.controller, x);
                    }
                }),
            ],
        },
    ];
    db.add(d);

    // Nimblism (rouge) — Generic Action, coût 0, déf. 2.
    // « The next attack action card with cost 1 or less you play this turn
    //   gets +3{p}.
    //   Go again »
    let mut d = deck_card(
        "Nimblism",
        Color::Red,
        G,
        Types::ACTION,
        Some(0),
        None,
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![Op::Do(|g, c| {
        g.next_attack(
            c.controller,
            Some(|g, a| is_attack_action_card(g, a) && g.def(a).cost.unwrap_or(0) <= 1),
            power(3),
            true,
        );
    })];
    db.add(d);

    // Oasis Respite (rouge) — Generic Instant, coût 1.
    // « Prevent the next 4 damage that would be dealt to target hero this
    //   turn by a source of your choice. If they have less {h} than each
    //   other hero, they may gain 1{h}. »
    // La source est choisie à la résolution, parmi les objets publics de
    // l'arène et de la pile (CR 9.1.2c : l'effet suit une carte retournable).
    let mut d = deck_card(
        "Oasis Respite",
        Color::Red,
        G,
        Types::INSTANT,
        Some(1),
        None,
        None,
    );
    d.targets = vec![super::wizard::TARGET_HERO];
    d.resolve = vec![
        Op::Ask(|g, c| {
            hero_target(c)?;
            Some(Ask {
                player: c.controller,
                label: "Source dont les dégâts sont prévenus",
                options: possible_sources(g)
                    .into_iter()
                    .map(Choice::Object)
                    .collect(),
            })
        }),
        Op::Do(|g, c| {
            let Some(p) = hero_target(c) else { return };
            let src = match c.answer.take() {
                Some(Choice::Object(x)) => x,
                _ => return,
            };
            let source = Some(g.obj_ref(src));
            g.add_prevention(fab_engine::Prevention {
                id: 0,
                player: p,
                remaining: Some(4),
                arcane_only: false,
                source,
                source_filter: None,
                fixed: false,
                on_prevent: None,
                until: Until::EndOfTurn,
            });
        }),
        Op::Ask(|g, c| {
            let p = hero_target(c)?;
            (g.player(p).life < g.player(p.opponent()).life).then(|| Ask {
                player: p,
                label: "Gagner 1 PV ?",
                options: vec![Choice::Yes, Choice::No],
            })
        }),
        Op::Do(|g, c| {
            if let (Some(Choice::Yes), Some(p)) = (c.answer.take(), hero_target(c)) {
                g.gain_life(p, 1);
            }
        }),
    ];
    db.add(d);

    // Outed (rouge) — Generic Action - Attack, coût 0, 3{p} (pas de défense).
    // « If you are marked, you can't play this.
    //   If the defending hero is marked, this gets +1{p}.
    //   Go again »
    let mut d = deck_card(
        "Outed",
        Color::Red,
        G,
        Types::ACTION,
        Some(0),
        Some(3),
        None,
    );
    d.keywords = Keywords::GO_AGAIN;
    d.play_condition = Some(|g, p, _| !g.player(p).marked);
    d.self_mod = Some(|g, me| {
        let defending_marked = g
            .combat
            .links
            .iter()
            .any(|l| l.attack.id == me && g.player(l.target).marked);
        if defending_marked {
            power(1)
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Snatch (rouge) — Generic Action - Attack, coût 0, 4{p} 2{d}.
    // « When this hits, draw a card. »
    let mut d = deck_card(
        "Snatch",
        Color::Red,
        G,
        Types::ACTION,
        Some(0),
        Some(4),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: this_hits,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            g.draw(c.controller);
        })],
    }];
    db.add(d);
}

/// Les Nimblism du cimetière de `p`.
fn nimblisms_in_graveyard(g: &GameState, p: fab_engine::PlayerId) -> Vec<ObjectId> {
    g.player(p)
        .graveyard
        .iter()
        .copied()
        .filter(|&x| g.def(x).name == "Nimblism")
        .collect()
}

/// Les alliés que `p` contrôle.
fn allies_of(g: &GameState, p: fab_engine::PlayerId) -> Vec<ObjectId> {
    g.permanents
        .iter()
        .copied()
        .filter(|&o| g.obj(o).controller == p && g.def(o).subtypes.has(Subtypes::ALLY))
        .collect()
}

/// Le héros touché (« they ») d'une capacité « when this hits a hero ».
fn hit_target(c: &Ctx) -> Option<fab_engine::PlayerId> {
    match c.event {
        Some(Event::Hit { target, .. }) => Some(target),
        _ => None,
    }
}

/// Le héros ciblé (cible n° 0), s'il l'est encore.
fn hero_target(c: &Ctx) -> Option<fab_engine::PlayerId> {
    match c.target(0) {
        Some(Target::Player(p)) => Some(p),
        _ => None,
    }
}

/// « A source of your choice » : les objets publics de l'arène (héros,
/// armes, équipements, permanents, chaîne de combat) et de la pile.
fn possible_sources(g: &GameState) -> Vec<ObjectId> {
    (0..g.objects.len() as u32)
        .map(ObjectId)
        .filter(|&o| {
            let x = g.obj(o);
            (x.zone.in_arena() || x.zone == Zone::Stack) && !x.face_down && x.proxy_of.is_none()
        })
        .collect()
}

/// « If you've attacked [ce héros] this turn » : `p` a attaqué le héros de `q`.
fn attacked_hero_this_turn(
    g: &GameState,
    p: fab_engine::PlayerId,
    q: fab_engine::PlayerId,
) -> bool {
    g.turn_events().iter().any(|e| {
        matches!(*e, Event::Attack { attacker, target, target_obj: None, .. } if attacker == p && target == q)
    })
}

/// Cartes génériques du deck Arakni.
fn arakni_generics(db: &mut CardDb) {
    use super::assassin::{this_hits_a_hero, TARGET_OPPOSING_HERO};
    let hit_mark = || TriggerDef {
        when: this_hits_a_hero,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(Event::Hit { target, .. }) = c.event {
                g.mark(target);
            }
        })],
    };

    // Public Bounty — Generic Action, coût 1, déf. 2.
    // « **Mark** target opposing hero.
    // The next time you attack a **marked** hero this turn, the attack gets
    // +3{p} / +2{p} / +1{p}.
    // **Go again** »
    for c in COLORS {
        let mut d = deck_card("Public Bounty", c, G, Types::ACTION, Some(1), None, Some(2));
        d.keywords = Keywords::GO_AGAIN;
        d.targets = vec![TARGET_OPPOSING_HERO];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(Target::Player(p)) = c.target(0) {
                g.mark(p);
            }
            let n = by_color(g, c, [3, 2, 1]);
            // L'attaque qui devient l'attaque active vise un héros marqué.
            g.next_time_you_attack(
                c.controller,
                |g, a| {
                    g.active_link().is_some_and(|l| {
                        l.attack.id == a && l.target_obj.is_none() && g.player(l.target).marked
                    })
                },
                power(n),
            );
        })];
        db.add(d);
    }

    // Pursue to the Edge of Oblivion (rouge) — Generic Action - Attack, 4{p},
    // coût 0, déf. 3. « When this hits a hero, **mark** them. »
    let mut d = deck_card(
        "Pursue to the Edge of Oblivion",
        Color::Red,
        G,
        Types::ACTION,
        Some(0),
        Some(4),
        Some(3),
    );
    d.triggers = vec![hit_mark()];
    db.add(d);

    // Pursue to the Pits of Despair (rouge) — Generic Action - Attack, 5{p},
    // coût 1, déf. 3. « When this hits a hero, **mark** them. »
    let mut d = deck_card(
        "Pursue to the Pits of Despair",
        Color::Red,
        G,
        Types::ACTION,
        Some(1),
        Some(5),
        Some(3),
    );
    d.triggers = vec![hit_mark()];
    db.add(d);

    // Relentless Pursuit (bleu) — Generic Action, coût 0, déf. 3.
    // « **Mark** target opposing hero.
    // If you've attacked them this turn, put this on the bottom of its
    // owner's deck.
    // **Go again** »
    // La carte quitte la pile pendant sa résolution : son go again est lu
    // sur la couche (CR 5.3.5a).
    let mut d = deck_card(
        "Relentless Pursuit",
        Color::Blue,
        G,
        Types::ACTION,
        Some(0),
        None,
        Some(3),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.targets = vec![TARGET_OPPOSING_HERO];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Player(p)) = c.target(0) {
            g.mark(p);
            if attacked_hero_this_turn(g, c.controller, p) {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.put_on_bottom(s.id);
                }
            }
        }
    })];
    db.add(d);
}
