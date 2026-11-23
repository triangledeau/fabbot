//! Cartes Warrior des decks Hala et Dorinthea.

use super::{
    by_color, card_named, deck_card, equipment, grant, is_sword, is_sword_attack, power, reprise,
    target_obj, COLORS, TARGET_ATTACK, TARGET_SWORD_YOU_CONTROL, TARGET_WEAPON_ATTACK,
};
use fab_engine::card::{
    CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, Types,
};
use fab_engine::replace::{ReplaceKind, Replacement};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, PlayerId, Prevention, TriggerDef, Until,
    WeaponEffectKind, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Hala",
    "Dorinthea",
    "Zenith Blade",
    "Decimator Great Axe",
    "Helm of Unity",
    "Plating of Unity",
    "Gauntlets of Unity",
    "Pillar of Unity",
    "Edge Laden Plate",
    "Refraction Bolters",
    "Flurry",
    "Fatal Engagement",
    "Felling Swing",
    "Hit and Run",
    "Ironsong Response",
    "Overpower",
    "Sharpen Steel",
    "Steelblade Shunt",
    "Display of Craftsmanship",
    "Cut n' Carve",
    "Edict of Steel",
    "Sharp Incline",
    "Swordmaster's Path",
    "Toe the Line",
    "Shove Off",
];

const W: Supertypes = Supertypes::WARRIOR;

/// Une attaque de ce joueur avec une arme a-t-elle eu lieu ce tour ?
fn attacked_with_weapon_this_turn(g: &GameState, p: PlayerId) -> bool {
    // L'événement « attack » d'une attaque-proxy désigne l'arme.
    g.turn_events().iter().any(|e| {
        matches!(e, Event::Attack { attack, attacker, .. }
            if *attacker == p && g.def(*attack).types.has(Types::WEAPON))
    })
}

/// « Sharpen target sword you control » (CR 8.5.58).
fn sharpen_target(g: &mut GameState, c: &mut Ctx) {
    if let Some(r) = target_obj(c, 0) {
        g.sharpen(c.controller, r.id);
    }
}

/// L'épée ciblée a-t-elle au moins `[rouge, jaune, bleu]` compteurs +1{p} ?
fn target_has_counters(g: &GameState, c: &Ctx) -> bool {
    let n = by_color(g, c, [1, 2, 3]);
    target_obj(c, 0)
        .filter(|&r| g.is_current(r))
        .is_some_and(|r| g.obj(r.id).power_counters as i16 >= n)
}

/// Les cartes Sharpen : action Warrior, coût 0, déf. 3, go again.
fn sharpen_card(name: &str, c: Color, then: fn(&mut GameState, &mut Ctx)) -> CardDef {
    let mut d = deck_card(name, c, W, Types::ACTION, Some(0), None, Some(3));
    d.keywords = Keywords::GO_AGAIN;
    d.targets = vec![TARGET_SWORD_YOU_CONTROL];
    d.resolve = vec![Op::Do(sharpen_target), Op::Do(then)];
    d
}

/// « Unity - When this defends together with a card from hand, this gets
/// +1{d} until end of turn. Temper » (CR 8.4.10, 8.3.10).
fn unity(name: &str, slot: Subtypes) -> CardDef {
    let mut d = equipment(name, W, slot, 1);
    d.keywords = Keywords::TEMPER;
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Defend { card, with_hand_card: true, .. } if *card == me),
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(s) = c.source {
                let d = Delta {
                    defense: 1,
                    ..Default::default()
                };
                g.modify(s, d, Until::EndOfTurn);
            }
        })],
    }];
    d
}

/// Le token Flurry, créé par Edict of Steel et Toe the Line.
fn flurry(g: &mut GameState, p: PlayerId) {
    let f = card_named(g, "Flurry");
    g.create_tokens(p, f, 1);
}

pub fn add(db: &mut CardDb) {
    heroes_and_arena(db);
    deck_cards(db);
}

fn heroes_and_arena(db: &mut CardDb) {
    let hero = |name: &str| CardDef {
        name: name.into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: W,
        ..Default::default()
    };

    // Hala — Warrior Hero - Young, 20 PV, intellect 4.
    // « Action -- {r}{r}{r}, {t}: Sharpen target sword you control. Go again »
    let mut d = hero("Hala");
    d.activated = vec![ActivatedDef {
        label: "Aiguiser une épée",
        speed: Speed::Action,
        cost: 3,
        tap: true,
        go_again: true,
        targets: vec![TARGET_SWORD_YOU_CONTROL],
        script: vec![Op::Do(sharpen_target)],
        ..Default::default()
    }];
    db.add(d);

    // Dorinthea — Warrior Hero - Young, 20 PV, intellect 4.
    // « The first time your weapon attack hits each turn, you may attack an
    // additional time with that weapon this turn. »
    let mut d = hero("Dorinthea");
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            let p = g.obj(me).controller;
            let weapon_hit = |e: &Event| {
                matches!(e, Event::Hit { attack, attacker, .. }
                    if *attacker == p && g.def(*attack).types.has(Types::WEAPON))
            };
            // CR 6.6.5d : seulement la première fois du tour (l'événement
            // courant est déjà dans le journal).
            weapon_hit(e) && g.turn_events().iter().filter(|x| weapon_hit(x)).count() == 1
        },
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            // « you may » : une permission, pas de question (CR 5.2.3c).
            if let Some(Event::Hit { attack, .. }) = c.event {
                let w = g.obj_ref(attack);
                g.add_weapon_effect(w, WeaponEffectKind::AdditionalAttack);
            }
        })],
    }];
    db.add(d);

    let weapon = |name: &str, power: u8, sub: Subtypes, cost: u8| CardDef {
        name: name.into(),
        power: Some(power),
        types: Types::WEAPON,
        subtypes: sub,
        supertypes: W,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        ..Default::default()
    };

    // Zenith Blade — Warrior Weapon - Sword (2H), 3{p}.
    // « Hala Specialization. Once per Turn Action -- {r}: Attack. If this has
    // been sharpened this turn, its first attack this turn gets go again. »
    // (La spécialisation ne concerne que la construction du deck.)
    let mut d = weapon("Zenith Blade", 3, Subtypes::SWORD | Subtypes::TWO_HAND, 1);
    d.aura = Some(|g, src, target| {
        let o = g.obj(target);
        // `counters` d'une proxy : numéro de l'attaque de l'arme ce tour.
        if o.proxy_of == Some(src) && o.counters == 1 && g.sharpened_this_turn(src) {
            grant(Keywords::GO_AGAIN)
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Decimator Great Axe — Warrior Weapon - Axe (2H), 4{p}.
    // « Once per Turn Action -- {r}{r}{r}: Attack. The first time this is
    // defended by a non-equipment card each turn, halve the base {d} of target
    // defending card, rounded up, until end of turn. »
    let mut d = weapon(
        "Decimator Great Axe",
        4,
        Subtypes::AXE | Subtypes::TWO_HAND,
        3,
    );
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            let defended_by_card = |e: &Event| {
                matches!(e, Event::Defend { card, attack, .. }
                    if g.obj(*attack).proxy_of == Some(me)
                        && !g.def(*card).types.has(Types::EQUIPMENT))
            };
            defended_by_card(e)
                && g.turn_events()
                    .iter()
                    .filter(|x| defended_by_card(x))
                    .count()
                    == 1
        },
        anywhere: false,
        targets: vec![TargetSpec {
            label: "carte qui défend",
            // CR 7.0.5a : une carte défend tant qu'elle est sur la chaîne.
            // Notes de Dusk till Dawn : n'importe quelle carte qui défend sur
            // la chaîne, pas seulement sur ce maillon, équipements compris ;
            // jusqu'à la fin du tour (Errata Bulletin 7).
            filter: |g, _, t| {
                matches!(t, Target::Object(r) if g.obj(r.id).zone == Zone::CombatChain
                    && g.combat.links.iter().any(|l| l.defenders.contains(&r.id)))
            },
            optional: false,
            in_graveyard: false,
        }],
        script: vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                // Défense de base divisée par 2, arrondie au supérieur :
                // on retire la moitié arrondie à l'inférieur.
                // TODO(règles) : équivalent tant qu'aucun effet ne change la
                // défense de base (aucun dans les 9 decks).
                let base = g.def(r.id).defense.unwrap_or(0) as i16;
                let d = Delta {
                    defense: -(base / 2),
                    ..Default::default()
                };
                g.modify(r, d, Until::EndOfTurn);
            }
        })],
    }];
    db.add(d);

    // Helm / Plating / Gauntlets / Pillar of Unity — Warrior Equipment, déf. 1.
    db.add(unity("Helm of Unity", Subtypes::HEAD));
    db.add(unity("Plating of Unity", Subtypes::CHEST));
    db.add(unity("Gauntlets of Unity", Subtypes::ARMS));
    db.add(unity("Pillar of Unity", Subtypes::LEGS));

    // Edge Laden Plate — Warrior Equipment - Chest, déf. 1.
    // « Instant -- Destroy this: Gain {r}. Activate this only if you've
    // sharpened a sword this turn. Battleworn »
    let mut d = equipment("Edge Laden Plate", W, Subtypes::CHEST, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.activated = vec![ActivatedDef {
        label: "Détruire : gagner 1 ressource",
        speed: Speed::Instant,
        condition: Some(|g, p, _| g.player_sharpened_this_turn(p, is_sword)),
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(d);

    // Refraction Bolters — Warrior Equipment - Legs, déf. 1.
    // « When a weapon attack you control hits, you may destroy this. If you
    // do, the attack gets go again. Battleworn »
    let mut d = equipment("Refraction Bolters", W, Subtypes::LEGS, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            matches!(e, Event::Hit { attack, attacker, .. }
                if *attacker == g.obj(me).controller && g.def(*attack).types.has(Types::WEAPON))
        },
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                c.source.filter(|&s| g.is_current(s)).map(|_| Ask {
                    player: c.controller,
                    label: "Détruire cette carte pour donner go again à l'attaque ?",
                    options: vec![Choice::Yes, Choice::No],
                })
            }),
            Op::If(|_, c| c.answer == Some(Choice::Yes), 1),
            Op::Do(|g, c| {
                g.destroy(c.source_id().unwrap());
                // « the attack » : l'attaque (proxy) de l'arme qui a touché.
                if let Some(Event::Hit { attack, .. }) = c.event {
                    let link = g
                        .combat
                        .links
                        .iter()
                        .rev()
                        .find(|l| l.source == Some(attack));
                    if let Some(a) = link.map(|l| l.attack) {
                        g.modify(a, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                    }
                }
            }),
        ],
    }];
    db.add(d);

    // Flurry — Warrior Token - Aura.
    // « When you activate a weapon attack, destroy this and you may attack
    // with the weapon twice this turn. »
    db.add(CardDef {
        name: "Flurry".into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: W,
        triggers: vec![TriggerDef {
            when: |g, me, e| match *e {
                Event::Activated {
                    player,
                    source,
                    ability,
                } => {
                    let def = g.def(source);
                    player == g.obj(me).controller
                        && def.types.has(Types::WEAPON)
                        && def.activated[ability as usize].attack
                }
                _ => false,
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.destroy(s.id);
                }
                if let Some(Event::Activated { source, .. }) = c.event {
                    // CR 5.2.3e : fixe la limite à 2.
                    let w = g.obj_ref(source);
                    g.add_weapon_effect(w, WeaponEffectKind::AttackTwice);
                }
            })],
        }],
        ..Default::default()
    });
}

fn deck_cards(db: &mut CardDb) {
    for (i, c) in COLORS.into_iter().enumerate() {
        let i = i as u8;

        // Fatal Engagement — Warrior Attack Reaction, coût 2, déf. 3.
        // « Play this only if an attack action card is defending this chain
        // link. Target attack gets +5{p} » (4, 3).
        let mut d = deck_card(
            "Fatal Engagement",
            c,
            W,
            Types::ATTACK_REACTION,
            Some(2),
            None,
            Some(3),
        );
        d.play_condition = Some(|g, _, _| {
            g.active_link().is_some_and(|l| {
                l.defenders.iter().any(|&x| {
                    let def = g.def(x);
                    g.obj(x).zone == Zone::CombatChain
                        && def.types.has(Types::ACTION)
                        && def.subtypes.has(Subtypes::ATTACK)
                })
            })
        });
        d.targets = vec![TARGET_ATTACK];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let n = by_color(g, c, [5, 4, 3]);
                g.modify(r, power(n), Until::EndOfTurn);
            }
        })];
        db.add(d);

        // Felling Swing — Warrior Action, coût 3, déf. 3.
        // « Your next axe attack this turn gets +6{p} (5, 4). Go again »
        let mut d = deck_card("Felling Swing", c, W, Types::ACTION, Some(3), None, Some(3));
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [6, 5, 4]);
            let axe = |g: &GameState, a: ObjectId| g.def(a).subtypes.has(Subtypes::AXE);
            g.next_attack(c.controller, Some(axe), power(n), false);
        })];
        db.add(d);

        // Hit and Run — Warrior Action, coût 0, déf. 3.
        // « Your next weapon attack this turn gets go again. If you've
        // attacked with a weapon this turn, your next attack this turn gets
        // +3{p} (2, 1). Go again »
        let mut d = deck_card("Hit and Run", c, W, Types::ACTION, Some(0), None, Some(3));
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            let p = c.controller;
            let weapon = |g: &GameState, a: ObjectId| g.is_weapon_attack(a);
            g.next_attack(p, Some(weapon), grant(Keywords::GO_AGAIN), false);
            if attacked_with_weapon_this_turn(g, p) {
                let n = by_color(g, c, [3, 2, 1]);
                g.next_attack(p, None, power(n), false);
            }
        })];
        db.add(d);

        // Ironsong Response — Warrior Attack Reaction, coût 0, déf. 3.
        // « Reprise - If the defending hero has defended with a card from
        // their hand this chain link, target weapon attack gets +3{p} » (2, 1).
        let mut d = deck_card(
            "Ironsong Response",
            c,
            W,
            Types::ATTACK_REACTION,
            Some(0),
            None,
            Some(3),
        );
        d.targets = vec![TARGET_WEAPON_ATTACK];
        d.resolve = vec![Op::Do(|g, c| {
            if let (Some(r), true) = (target_obj(c, 0), reprise(g)) {
                let n = by_color(g, c, [3, 2, 1]);
                g.modify(r, power(n), Until::EndOfTurn);
            }
        })];
        db.add(d);

        // Overpower — Warrior Attack Reaction, coût 3, déf. 3.
        // « Target weapon attack gets +4{p} (3, 2). Reprise - If the defending
        // hero has defended with a card from their hand this chain link,
        // instead it gets +6{p} (5, 4). »
        let mut d = deck_card(
            "Overpower",
            c,
            W,
            Types::ATTACK_REACTION,
            Some(3),
            None,
            Some(3),
        );
        d.targets = vec![TARGET_WEAPON_ATTACK];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let n = if reprise(g) {
                    by_color(g, c, [6, 5, 4])
                } else {
                    by_color(g, c, [4, 3, 2])
                };
                g.modify(r, power(n), Until::EndOfTurn);
            }
        })];
        db.add(d);

        // Sharpen Steel — Warrior Action, coût 0, déf. 3.
        // « Your next weapon attack this turn gets +3{p} (2, 1). Go again »
        let mut d = deck_card("Sharpen Steel", c, W, Types::ACTION, Some(0), None, Some(3));
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 1]);
            let weapon = |g: &GameState, a: ObjectId| g.is_weapon_attack(a);
            g.next_attack(c.controller, Some(weapon), power(n), false);
        })];
        db.add(d);

        // Steelblade Shunt — Warrior Defense Reaction, coût 1, déf. 6/5/4.
        // « If this defends a weapon attack, deal 1 damage to the attacking hero. »
        let mut d = deck_card(
            "Steelblade Shunt",
            c,
            W,
            Types::DEFENSE_REACTION,
            Some(1),
            None,
            Some(6 - i),
        );
        d.resolve = vec![Op::Do(|g, c| {
            // La carte va défendre l'attaque du maillon actif (CR 5.3.2b,
            // 7.4.2d) : la condition « if this defends » est vérifiée à la
            // résolution (CR 5.3.4), juste avant que la carte devienne une
            // carte qui défend (CR 5.3.6b), contre l'attaque du maillon actif
            // (confirmé par le propriétaire).
            let Some(link) = g.active_link() else {
                return;
            };
            let weapon = link
                .source
                .is_some_and(|w| g.def(w).types.has(Types::WEAPON));
            let attacker = link.attacker;
            if weapon {
                g.deal_damage(c.source_id(), c.controller, attacker, 1, false);
            }
        })];
        db.add(d);

        // Display of Craftsmanship — Warrior Attack Reaction, coût 2, déf. 3.
        // « Target weapon attack gets +4{p} (3, 2). If the weapon has been
        // sharpened this turn, put a +1{p} counter on it. »
        let mut d = deck_card(
            "Display of Craftsmanship",
            c,
            W,
            Types::ATTACK_REACTION,
            Some(2),
            None,
            Some(3),
        );
        d.targets = vec![TARGET_WEAPON_ATTACK];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let n = by_color(g, c, [4, 3, 2]);
                g.modify(r, power(n), Until::EndOfTurn);
                if let Some(w) = g.obj(r.id).proxy_of {
                    if g.sharpened_this_turn(w) && g.obj(w).zone.in_arena() {
                        // Ce compteur est retiré au début de la phase de fin
                        // par le sharpen de l'arme (« remove all +1{p} counters »).
                        g.obj_mut(w).power_counters += 1;
                    }
                }
            }
        })];
        db.add(d);

        // Cut n' Carve — Warrior Action, coût 0, déf. 3.
        // « Sharpen target sword you control. If it has 1 (2, 3) or more
        // +1{p} counters, its next attack this turn gets dominate. Go again »
        db.add(sharpen_card("Cut n' Carve", c, |g, c| {
            if target_has_counters(g, c) {
                let r = target_obj(c, 0).unwrap();
                g.next_attack_with(c.controller, r, grant(Keywords::DOMINATE));
            }
        }));

        // Edict of Steel — Warrior Action, coût 0, déf. 3.
        // « Sharpen target sword you control. If it has 1 (2, 3) or more
        // +1{p} counters, create a Flurry token. Go again »
        db.add(sharpen_card("Edict of Steel", c, |g, c| {
            if target_has_counters(g, c) {
                flurry(g, c.controller);
            }
        }));

        // Sharp Incline — Warrior Action, coût 0, déf. 3.
        // « Sharpen target sword you control. If it has 1 (2, 3) or more
        // +1{p} counters, your next attack with it this turn costs {r} less
        // to activate. Go again »
        db.add(sharpen_card("Sharp Incline", c, |g, c| {
            if target_has_counters(g, c) {
                let r = target_obj(c, 0).unwrap();
                g.add_weapon_effect(r, WeaponEffectKind::CostLess(1));
            }
        }));

        // Swordmaster's Path — Warrior Action, coût 1, déf. 3.
        // « Your next sword attack this turn gets +3{p} (2, 1). The next time
        // you would sharpen a sword this turn, instead sharpen it an
        // additional time. Go again »
        let mut d = deck_card(
            "Swordmaster's Path",
            c,
            W,
            Types::ACTION,
            Some(1),
            None,
            Some(3),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 1]);
            g.next_attack(c.controller, Some(is_sword_attack), power(n), false);
            g.add_replacement(Replacement {
                id: 0,
                kind: ReplaceKind::Sharpen,
                controller: c.controller,
                data: None,
                bind: None,
                value: 0,
                optional: false,
                applies: |g, w, p| {
                    p.player == w.controller && p.object.is_some_and(|o| is_sword(g, o))
                },
                apply: |_, _, p| p.amount += 1,
                until: Until::EndOfTurn,
                uses: Some(1),
            });
        })];
        db.add(d);
    }

    // Toe the Line (rouge) — Warrior Instant, coût 0.
    // « The next time you would be dealt damage this turn, prevent 2 of that
    // damage. If you prevent damage this way, create a Flurry token. »
    let mut d = deck_card(
        "Toe the Line",
        Color::Red,
        W,
        Types::INSTANT,
        Some(0),
        None,
        None,
    );
    d.resolve = vec![Op::Do(|g, c| {
        g.add_prevention(Prevention {
            id: 0,
            player: c.controller,
            remaining: Some(2),
            arcane_only: false,
            source: None,
            source_filter: None,
            fixed: true,
            on_prevent: Some(|g, p, _| flurry(g, p)),
            until: Until::EndOfTurn,
        })
    })];
    db.add(d);

    // Shove Off (bleu) — Warrior Attack Reaction, coût 1, déf. 3.
    // « Choose a non-equipment card defending a sword attack you control.
    // Return it to its owner's hand. »
    let mut d = deck_card(
        "Shove Off",
        Color::Blue,
        W,
        Types::ATTACK_REACTION,
        Some(1),
        None,
        Some(3),
    );
    d.resolve = vec![
        Op::Ask(|g, c| {
            // CR 7.0.5a : une carte défend tant qu'elle est sur la chaîne, contre
            // l'attaque de son maillon : les cartes des maillons précédents
            // comptent, si l'épée est encore sur ce maillon (notes de Mastery
            // Pack Warrior). Quand l'épée attaque de nouveau, son ancienne
            // attaque-proxy cesse d'exister (CR 1.4.3c) : l'épée a quitté le
            // maillon.
            let options: Vec<Choice> = g
                .combat
                .links
                .iter()
                .filter(|l| {
                    l.attacker == c.controller
                        && l.source
                            .is_some_and(|w| g.def(w).types.has(Types::WEAPON) && is_sword(g, w))
                        && g.is_current(l.attack)
                        && g.obj(l.attack.id).zone == Zone::CombatChain
                })
                .flat_map(|l| l.defenders.iter().copied())
                .filter(|&x| {
                    g.obj(x).zone == Zone::CombatChain && !g.def(x).types.has(Types::EQUIPMENT)
                })
                .map(Choice::Object)
                .collect();
            Some(Ask {
                player: c.controller,
                label: "Carte (hors équipement) qui défend contre votre attaque d'épée",
                options,
            })
        }),
        Op::Do(|g, c| {
            if let Some(Choice::Object(x)) = c.answer {
                g.return_to_hand(x);
            }
        }),
    ];
    db.add(d);
}
