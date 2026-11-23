//! Cartes Guardian du deck Bravo, Flattering Showman.
//!
//! Crush (CR 8.4.2) : « Crush - When this deals 4 or more damage to a hero,
//! [EFFECTS] ». C'est un label d'une capacité déclenchée par un événement de
//! **dégâts** (pas seulement un « hit », CR 8.4.2a). Les cartes avec crush
//! portent aussi le drapeau `Keywords::CRUSH` (« a card with crush », Bravo et
//! Crash and Bash).
//!
//! Clash (CR 8.5.45) : `GameState::clash`.

use super::{card_named, deck_card, equipment, power, target_obj};
use fab_engine::card::{
    AdditionalCost, CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, Types,
};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, PlayerId, TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Bravo, Flattering Showman",
    "Miller's Grindstone",
    "Richter Scale",
    "Steelbraid Buckler",
    "Gauntlet of Boulderhold",
    "Civic Steps",
    "Seismic Surge",
    "Boulder Drop",
    "Buckling Blow",
    "Chokeslam",
    "Clash of Might",
    "Clash of Mountains",
    "Clash of Vigor",
    "Crash and Bash",
    "Crush Confidence",
    "Debilitate",
    "Disable",
    "Fault Line",
    "Flatten the Field",
    "Macho Grande",
    "Sit!",
    "Staunch Response",
    "Test of Vigor",
    "The Suspense is Killing Me",
    "Thunder Quake",
    "Thunk",
];

const GU: Supertypes = Supertypes::GUARDIAN;
const SURGE: &str = "Seismic Surge";

/// Une carte d'action d'attaque Guardian (pas une attaque-proxy).
fn is_guardian_attack_action_card(g: &GameState, id: ObjectId) -> bool {
    super::generic::is_attack_action_card(g, id) && g.def(id).supertypes.has(GU)
}

/// « When this defends ».
fn when_this_defends(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Defend { card, .. } if *card == me)
}

/// « Crush - When this deals 4 or more damage to a hero, … » (CR 8.4.2).
fn crush(script: Vec<Op>) -> TriggerDef {
    TriggerDef {
        when: |_, me, e| {
            matches!(e, Event::Damage { source: Some(s), target_obj: None, amount, .. }
                if *s == me && *amount >= 4)
        },
        anywhere: false,
        targets: vec![],
        script,
    }
}

/// Le héros qui a reçu les dégâts du crush (« they », « that hero »).
fn crushed(c: &Ctx) -> Option<PlayerId> {
    match c.event {
        Some(Event::Damage { target, .. }) => Some(target),
        _ => None,
    }
}

/// Crée `n` tokens `name` pour le joueur `p`.
pub(crate) fn create(g: &mut GameState, p: PlayerId, name: &str, n: u8) {
    let t = card_named(g, name);
    g.create_tokens(p, t, n);
}

/// Clash of Might / Vigor / Mountains, Test of Vigor : « clash with the
/// attacking hero. The winner creates a [token] token. »
pub(crate) fn clash_with_attacker(g: &mut GameState, c: &Ctx, token: &str) {
    let Some(Event::Defend { attack, .. }) = c.event else {
        return;
    };
    let attacker = g.obj(attack).controller;
    if let Some(w) = g.clash(c.controller, attacker) {
        create(g, w, token, 1);
    }
}

/// Une carte d'attaque Guardian avec crush, défense 3. `power` : [rouge, bleu].
fn crush_card(name: &str, color: Color, cost: u8, power: u8, script: Vec<Op>) -> CardDef {
    let mut d = deck_card(
        name,
        color,
        GU,
        Types::ACTION,
        Some(cost),
        Some(power),
        Some(3),
    );
    d.keywords = Keywords::CRUSH;
    d.triggers = vec![crush(script)];
    d
}

/// Choisir une carte de la main de `p` (sans question s'il n'y en a qu'une).
fn ask_hand_card(g: &GameState, p: PlayerId, label: &'static str) -> Option<Ask> {
    let hand = &g.player(p).hand;
    (hand.len() > 1).then(|| Ask {
        player: p,
        label,
        options: hand.iter().map(|&x| Choice::Object(x)).collect(),
    })
}

/// La carte choisie par [`ask_hand_card`] (ou la seule carte de la main).
fn chosen_hand_card(g: &GameState, c: &Ctx, p: PlayerId) -> Option<ObjectId> {
    match c.answer {
        Some(Choice::Object(x)) if g.obj(x).zone == Zone::Hand => Some(x),
        _ => g.player(p).hand.first().copied(),
    }
}

/// Heave 3 (CR 8.3.18) : « While this is in your hand, at the beginning of
/// your end phase, you may pay 3 and put this face-up into your arsenal. If
/// you do, create 3 Seismic Surge tokens. »
///
/// Capacité déclenchée cachée (CR 5.4.7b). Simplification : le choix de
/// déclencher est posé à la résolution (dans la phase de fin, la pile se
/// résout sans priorité, CR 4.4.2, donc personne ne peut répondre entre les
/// deux). Le déclenchement n'a lieu que si heave est possible (CR 8.3.18b :
/// arsenal vide et 3 ressources payables **sans** pitcher cette carte).
fn heave_3() -> TriggerDef {
    TriggerDef {
        when: |g, me, e| {
            let o = g.obj(me);
            matches!(e, Event::BeginEndPhase(p) if *p == o.owner)
                && o.zone == Zone::Hand
                && can_heave(g, o.owner, me)
        },
        anywhere: true,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                let me = c.source.filter(|&s| g.is_current(s))?.id;
                can_heave(g, c.controller, me).then(|| Ask {
                    player: c.controller,
                    label: "Heave 3 : payer 3 pour mettre la carte en arsenal face visible et créer 3 Seismic Surge ?",
                    options: vec![Choice::Yes, Choice::No],
                })
            }),
            Op::If(|_, c| c.answer == Some(Choice::Yes), 2),
            Op::Do(|g, c| {
                let Some(me) = c.source.filter(|&s| g.is_current(s)) else {
                    return;
                };
                // La carte va d'abord en arsenal : elle ne peut plus servir à
                // payer (le paiement a été vérifié sans elle, CR 8.3.18b).
                g.put_in_arsenal(me.id);
                g.turn_face_up(me.id);
                c.flag = true;
                g.pay(c.controller, 3);
            }),
            Op::Do(|g, c| {
                if c.flag {
                    create(g, c.controller, SURGE, 3);
                }
            }),
        ],
    }
}

fn can_heave(g: &GameState, p: PlayerId, me: ObjectId) -> bool {
    g.player(p).arsenal.is_none() && g.available_resources(p, Some(me)) >= 3
}

pub fn add(db: &mut CardDb) {
    heroes_and_arena(db);
    deck_cards(db);
}

fn heroes_and_arena(db: &mut CardDb) {
    // Bravo, Flattering Showman — Guardian Hero - Young, 20 PV, intellect 4.
    // « Action -- {r}{r}, {t}: Turn a face-down card in your arsenal face-up.
    // If it has crush, it gets +2{p} and dominate this turn. Go again »
    db.add(CardDef {
        name: "Bravo, Flattering Showman".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: GU,
        activated: vec![ActivatedDef {
            label: "Retourner la carte de l'arsenal (crush : +2{p} et dominate)",
            speed: Speed::Action,
            cost: 2,
            tap: true,
            go_again: true,
            // L'activation sans carte face cachée serait légale mais sans
            // effet : on ne la propose pas.
            condition: Some(|g, p, _| g.player(p).arsenal.is_some_and(|a| g.obj(a).face_down)),
            script: vec![Op::Do(|g, c| {
                let Some(a) = g.player(c.controller).arsenal else {
                    return;
                };
                if !g.obj(a).face_down {
                    return;
                }
                g.turn_face_up(a);
                if g.keywords(a).has(Keywords::CRUSH) {
                    let d = Delta {
                        power: 2,
                        grants: Keywords::DOMINATE,
                        ..Default::default()
                    };
                    g.modify(g.obj_ref(a), d, Until::EndOfTurn);
                }
            })],
            ..Default::default()
        }],
        ..Default::default()
    });

    // Miller's Grindstone — Guardian Weapon - Hammer (1H), 4{p}.
    // « Once per Turn Action -- {r}{r}{r}: Attack
    // When this hits a hero, clash with them. If you win, destroy the top card
    // of their deck. If they win, put a -1{p} counter on this. »
    db.add(CardDef {
        name: "Miller's Grindstone".into(),
        power: Some(4),
        types: Types::WEAPON,
        subtypes: Subtypes::HAMMER | Subtypes::ONE_HAND,
        supertypes: GU,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 3,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        triggers: vec![TriggerDef {
            // L'événement « hit » d'une attaque-proxy désigne l'arme.
            when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let Some(Event::Hit { target, .. }) = c.event else {
                    return;
                };
                let me = c.controller;
                match g.clash(me, target) {
                    Some(w) if w == me => {
                        if let Some(top) = g.top_of_deck(target) {
                            g.destroy(top);
                        }
                    }
                    Some(_) => {
                        if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                            g.obj_mut(s.id).power_counters -= 1;
                        }
                    }
                    None => {}
                }
            })],
        }],
        ..Default::default()
    });

    // Richter Scale — Guardian Equipment - Chest, déf. 1.
    // « Action -- Destroy this: Create 2 Seismic Surge tokens. Battleworn »
    let mut d = equipment("Richter Scale", GU, Subtypes::CHEST, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.activated = vec![ActivatedDef {
        label: "Détruire : créer 2 Seismic Surge",
        speed: Speed::Action,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| create(g, c.controller, SURGE, 2))],
        ..Default::default()
    }];
    db.add(d);

    // Steelbraid Buckler — Guardian Equipment - Off-Hand, déf. 2. « Temper »
    let mut d = equipment("Steelbraid Buckler", GU, Subtypes::OFF_HAND, 2);
    d.keywords = Keywords::TEMPER;
    db.add(d);

    // Gauntlet of Boulderhold — Guardian Equipment - Arms, déf. 1.
    // « Action -- {r}{r}{r}, destroy this: The next Guardian attack action
    // card you play this turn gets +2{p}. Go again. Battleworn »
    let mut d = equipment("Gauntlet of Boulderhold", GU, Subtypes::ARMS, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.activated = vec![ActivatedDef {
        label: "Détruire : prochaine attaque Guardian +2{p}",
        speed: Speed::Action,
        cost: 3,
        go_again: true,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| {
            g.next_attack(
                c.controller,
                Some(is_guardian_attack_action_card),
                power(2),
                true,
            );
        })],
        ..Default::default()
    }];
    db.add(d);

    // Civic Steps — Guardian Equipment - Legs, déf. 2.
    // « Whenever this defends, create a Quicken token under another hero's
    // control. Temper »
    let mut d = equipment("Civic Steps", GU, Subtypes::LEGS, 2);
    d.keywords = Keywords::TEMPER;
    d.triggers = vec![TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        // Partie à deux : « another hero » est l'adversaire.
        script: vec![Op::Do(|g, c| {
            create(g, c.controller.opponent(), "Quicken", 1);
        })],
    }];
    db.add(d);

    // Seismic Surge — Guardian Token - Aura.
    // « At the beginning of your action phase, destroy this, then your next
    // Guardian attack action card this turn costs {r} less to play. »
    db.add(CardDef {
        name: SURGE.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: GU,
        triggers: vec![TriggerDef {
            when: |g, me, e| matches!(e, Event::BeginActionPhase(p) if *p == g.obj(me).controller),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.destroy(s.id);
                }
                g.next_card_costs_less(c.controller, is_guardian_attack_action_card, 1);
            })],
        }],
        ..Default::default()
    });
}

fn deck_cards(db: &mut CardDb) {
    const R: Color = Color::Red;
    const B: Color = Color::Blue;

    // Boulder Drop — Guardian Action - Attack, 7 (rouge) / 5 (bleu), coût 3, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, they put a card
    // from their hand on top of their deck. »
    for (c, p) in [(R, 7), (B, 5)] {
        db.add(crush_card(
            "Boulder Drop",
            c,
            3,
            p,
            vec![
                Op::Ask(|g, c| {
                    ask_hand_card(
                        g,
                        crushed(c)?,
                        "Carte de votre main à mettre sur votre deck",
                    )
                }),
                Op::Do(|g, c| {
                    let Some(p) = crushed(c) else { return };
                    if let Some(x) = chosen_hand_card(g, c, p) {
                        g.put_on_top(x);
                    }
                }),
            ],
        ));
    }

    // Buckling Blow (bleu) — Guardian Action - Attack, 6, coût 4, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, put a -1{d}
    // counter on target equipment they control. »
    let mut d = crush_card("Buckling Blow", B, 4, 6, vec![]);
    d.triggers[0].targets = vec![TargetSpec {
        label: "équipement",
        filter: |g, c, t| {
            let Target::Object(r) = t else { return false };
            g.def(r.id).types.has(Types::EQUIPMENT)
                && g.obj(r.id).zone.in_arena()
                && Some(g.obj(r.id).controller) == crushed(c)
        },
        optional: false,
        in_graveyard: false,
        in_banished: false,
    }];
    d.triggers[0].script = vec![Op::Do(|g, c| {
        if let Some(r) = target_obj(c, 0) {
            g.obj_mut(r.id).defense_counters -= 1;
        }
    })];
    db.add(d);

    // Chokeslam — Guardian Action - Attack, 8 / 6, coût 4, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, attack action
    // cards they control can't gain {p} during their next action phase. »
    for (c, p) in [(R, 8), (B, 6)] {
        db.add(crush_card(
            "Chokeslam",
            c,
            4,
            p,
            vec![Op::Do(|g, c| {
                if let Some(p) = crushed(c) {
                    g.forbid_power_gain_next_action_phase(p);
                }
            })],
        ));
    }

    // Crush Confidence (rouge) — Guardian Action - Attack, 7, coût 3, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, that hero loses
    // all abilities until the end of their next turn. »
    db.add(crush_card(
        "Crush Confidence",
        R,
        3,
        7,
        vec![Op::Do(|g, c| {
            if let Some(p) = crushed(c) {
                let lose = Delta {
                    lose_abilities: true,
                    ..Default::default()
                };
                let until = g.end_of_next_turn(p);
                g.modify(g.obj_ref(g.player(p).hero), lose, until);
            }
        })],
    ));

    // Debilitate — Guardian Action - Attack, 8 / 6, coût 4, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, their first
    // attack during their next turn gets -2{p}. »
    for (c, p) in [(R, 8), (B, 6)] {
        db.add(crush_card(
            "Debilitate",
            c,
            4,
            p,
            vec![Op::Do(|g, c| {
                if let Some(p) = crushed(c) {
                    let turn = g.next_turn_of(p);
                    g.next_attack_on_turn(p, turn, None, power(-2));
                }
            })],
        ));
    }

    // Disable (bleu) — Guardian Action - Attack, 7, coût 5, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, put a card from
    // their arsenal on the bottom of its owner's deck. »
    db.add(crush_card(
        "Disable",
        B,
        5,
        7,
        vec![Op::Do(|g, c| {
            // Un seul emplacement d'arsenal : pas de choix.
            if let Some(a) = crushed(c).and_then(|p| g.player(p).arsenal) {
                g.put_on_bottom(a);
            }
        })],
    ));

    // Fault Line (rouge) — Guardian Action - Attack, 7, coût 3, déf. 3.
    // « If you have a card in your arsenal, this gets +1{p}.
    // Crush - When this deals 4 or more damage to a hero, put all cards in all
    // arsenals on the bottom of their owner's deck. »
    let mut d = crush_card(
        "Fault Line",
        R,
        3,
        7,
        vec![Op::Do(|g, _| {
            for p in PlayerId::BOTH {
                if let Some(a) = g.player(p).arsenal {
                    g.put_on_bottom(a);
                }
            }
        })],
    );
    d.self_mod = Some(|g, me| {
        if g.player(g.obj(me).controller).arsenal.is_some() {
            power(1)
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Flatten the Field (bleu) — Guardian Action - Attack, 6, coût 4, déf. 3.
    // « Crush - When this deals 4 or more damage to a hero, destroy a Seismic
    // Surge token they control. »
    db.add(crush_card(
        "Flatten the Field",
        B,
        4,
        6,
        vec![Op::Do(|g, c| {
            let Some(p) = crushed(c) else { return };
            // Les Seismic Surge sont identiques : pas de choix utile.
            let surge = g
                .permanents
                .iter()
                .copied()
                .find(|&o| g.obj(o).controller == p && g.def(o).name == SURGE);
            if let Some(s) = surge {
                g.destroy(s);
            }
        })],
    ));

    // Macho Grande (bleu) — Guardian Action - Attack, 8, coût 7, déf. 3. « Dominate »
    let mut d = deck_card(
        "Macho Grande",
        B,
        GU,
        Types::ACTION,
        Some(7),
        Some(8),
        Some(3),
    );
    d.keywords = Keywords::DOMINATE;
    db.add(d);

    // Thunder Quake (bleu) — Guardian Action - Attack, 8, coût 6, déf. 3. « Heave 3 »
    let mut d = deck_card(
        "Thunder Quake",
        B,
        GU,
        Types::ACTION,
        Some(6),
        Some(8),
        Some(3),
    );
    d.triggers = vec![heave_3()];
    db.add(d);

    // Thunk (bleu) — Guardian Action - Attack, 6, coût 5, déf. 3.
    // « When you win a clash revealing this, create a Might token. »
    // CR 1.7.4i : fonctionne quand la carte est révélée depuis le deck ;
    // « you » est son propriétaire (CR 1.1.2b, exemple de The Golden Son).
    // Ce n'est pas une carte « avec clash ».
    let mut d = deck_card("Thunk", B, GU, Types::ACTION, Some(5), Some(6), Some(3));
    d.triggers = vec![TriggerDef {
        when: |g, me, e| matches!(e, Event::ClashWon { winner, card } if *card == me && *winner == g.obj(me).owner),
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| create(g, c.controller, "Might", 1))],
    }];
    db.add(d);

    // Clash of Might (bleu) — Brute / Guardian Action - Attack, 4, coût 2, déf. 3.
    // « When this defends, clash with the attacking hero. The winner creates a
    // Might token. »
    let mut d = deck_card(
        "Clash of Might",
        B,
        GU | Supertypes::BRUTE,
        Types::ACTION,
        Some(2),
        Some(4),
        Some(3),
    );
    d.triggers = vec![TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| clash_with_attacker(g, c, "Might"))],
    }];
    db.add(d);

    // Clash of Mountains (bleu) — Guardian Action - Attack, 6, coût 4, déf. 3.
    // « When this defends a Guardian attack, clash with the attacking hero.
    // The winner creates a Seismic Surge token. »
    let mut d = deck_card(
        "Clash of Mountains",
        B,
        GU,
        Types::ACTION,
        Some(4),
        Some(6),
        Some(3),
    );
    d.triggers = vec![TriggerDef {
        // Pour une attaque d'arme, la proxy a la définition de l'arme.
        when: |g, me, e| {
            matches!(e, Event::Defend { card, attack, .. }
                if *card == me && g.def(*attack).supertypes.has(GU))
        },
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| clash_with_attacker(g, c, SURGE))],
    }];
    db.add(d);

    // Clash of Vigor (bleu) — Guardian / Warrior Action - Attack, 4, coût 2, déf. 3.
    // Test of Vigor (rouge) — Guardian / Warrior Block, déf. 4.
    // « When this defends, clash with the attacking hero. The winner creates a
    // Vigor token. »
    let vigor = || TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| clash_with_attacker(g, c, "Vigor"))],
    };
    let gw = GU | Supertypes::WARRIOR;
    let mut d = deck_card(
        "Clash of Vigor",
        B,
        gw,
        Types::ACTION,
        Some(2),
        Some(4),
        Some(3),
    );
    d.triggers = vec![vigor()];
    db.add(d);
    let mut d = deck_card("Test of Vigor", R, gw, Types::BLOCK, None, None, Some(4));
    d.triggers = vec![vigor()];
    db.add(d);

    // Crash and Bash (rouge) — Guardian Block, déf. 4.
    // « When this defends, you may reveal a card with crush from your hand. If
    // you do, create a Seismic Surge token. »
    let mut d = deck_card("Crash and Bash", R, GU, Types::BLOCK, None, None, Some(4));
    d.triggers = vec![TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                let mut options: Vec<Choice> = g
                    .player(c.controller)
                    .hand
                    .iter()
                    .filter(|&&x| g.keywords(x).has(Keywords::CRUSH))
                    .map(|&x| Choice::Object(x))
                    .collect();
                if options.is_empty() {
                    return None;
                }
                options.push(Choice::Pass);
                Some(Ask {
                    player: c.controller,
                    label: "Révéler une carte avec crush pour créer un Seismic Surge ?",
                    options,
                })
            }),
            Op::Do(|g, c| {
                if let Some(Choice::Object(x)) = c.answer {
                    if g.obj(x).zone == Zone::Hand {
                        g.emit(Event::Revealed {
                            player: c.controller,
                            card: x,
                        });
                        create(g, c.controller, SURGE, 1);
                    }
                }
            }),
        ],
    }];
    db.add(d);

    // Sit! (rouge) — Guardian Block, déf. 2.
    // « When this defends a Brute attack, this gets +3{d}. »
    let mut d = deck_card("Sit!", R, GU, Types::BLOCK, None, None, Some(2));
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            matches!(e, Event::Defend { card, attack, .. }
                if *card == me && g.def(*attack).supertypes.has(Supertypes::BRUTE))
        },
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                let d = Delta {
                    defense: 3,
                    ..Default::default()
                };
                g.modify(s, d, Until::EndOfTurn);
            }
        })],
    }];
    db.add(d);

    // Staunch Response (rouge) — Guardian Defense Reaction, coût 2, déf. 7.
    // « As an additional cost to play this, you may pay {r}{r}{r}{r}.
    // If the additional cost is paid, this gets +3{d}. »
    let mut d = deck_card(
        "Staunch Response",
        R,
        GU,
        Types::DEFENSE_REACTION,
        Some(2),
        None,
        Some(7),
    );
    d.additional_cost = Some(AdditionalCost {
        resources: 4,
        optional: true,
    });
    d.resolve = vec![Op::Do(|g, c| {
        if c.optional_paid {
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                let d = Delta {
                    defense: 3,
                    ..Default::default()
                };
                g.modify(s, d, Until::EndOfTurn);
            }
        }
    })];
    db.add(d);

    // The Suspense is Killing Me (bleu) — Guardian Instant - Aura, coût 0, déf. 2.
    // « Suspense. Your first attack each turn gets +1{p}. »
    let mut d = deck_card(
        "The Suspense is Killing Me",
        B,
        GU,
        Types::INSTANT,
        Some(0),
        None,
        Some(2),
    );
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::SUSPENSE;
    d.aura = Some(|g, src, target| {
        let me = g.obj(src).controller;
        // Comme Ira (« your second attack each turn ») : l'attaque active,
        // quand elle est la 1re attaque du joueur ce tour.
        let is_first = g.obj(target).zone == Zone::CombatChain
            && g.active_attack() == Some(target)
            && g.obj(target).controller == me
            && g.attacks_this_turn(me) == 1;
        if is_first {
            power(1)
        } else {
            Delta::default()
        }
    });
    db.add(d);
}
