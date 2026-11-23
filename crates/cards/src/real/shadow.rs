//! Cartes Shadow et Brute du deck Levia (et les tokens Gate to i'Arathael et
//! Blasmophet, the Insatiable Hunger).
//!
//! Blood debt (CR 8.3.11) : drapeau `Keywords::BLOOD_DEBT`, géré par le
//! moteur (déclenchement au début de la phase de fin du propriétaire, pour
//! chaque carte publique de sa zone de bannissement).
//!
//! « A card with 6 or more {p} » : dans l'arène, la puissance actuelle ;
//! ailleurs (main, cimetière, pitch, bannissement), la puissance imprimée,
//! car les capacités statiques d'une carte ne fonctionnent pas hors de
//! l'arène (CR 1.7.4). Les événements `Banished` portent déjà cette valeur.

use super::guardian::{clash_with_attacker, create};
use super::{deck_card, equipment, grant, power, target_obj, COLORS};
use fab_engine::card::{CardDb, CardDef, Color, Delta, Keywords, Subtypes, Supertypes, Types};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, PlayerId, ReplaceKind, ReplacementDef, Speed,
    TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Levia",
    "Hell Hammer",
    "Buzzard Helm",
    "Predatory Plating",
    "Skera Strapping",
    "Embraforged Gauntlet",
    "Hex Gauntlet",
    "Hooves of the Shadowbeast",
    "Agile Windup",
    "Boneyard Marauder",
    "Clash of Agility",
    "Cleave the Heavens",
    "Consuming Strength",
    "Countdown to Extinction",
    "Dam the Shadowake",
    "Deadwood Rumbler",
    "Dread Screamer",
    "Endless Maw",
    "Engulfing Shadows",
    "Feasting Shadowbeast",
    "Feeding Frenzy",
    "Goremass Summoning",
    "Graveling Growl",
    "Smashback Alehorn",
    "Soul Harvest",
    "Test of Agility",
    "Vigorous Smashup",
    "Wrecker Romp",
    "Gate to i'Arathael",
    "Blasmophet, the Insatiable Hunger",
];

const SH: Supertypes = Supertypes::SHADOW;
const BR: Supertypes = Supertypes::BRUTE;
const GATE: &str = "Gate to i'Arathael";
const BLASMOPHET: &str = "Blasmophet, the Insatiable Hunger";
const R: Color = Color::Red;
const Y: Color = Color::Yellow;
const B: Color = Color::Blue;

// ----------------------------------------------------------------------
// Aides
// ----------------------------------------------------------------------

/// « When this attacks ».
fn when_this_attacks(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Attack { attack, .. } if *attack == me)
}

/// « When this defends ».
fn when_this_defends(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Defend { card, .. } if *card == me)
}

/// Une carte hors de l'arène (main, cimetière, pitch) a-t-elle 6{p} ou plus ?
/// Puissance imprimée (CR 1.7.4).
fn printed_power6(g: &GameState, id: ObjectId) -> bool {
    g.def(id).power.is_some_and(|p| p >= 6)
}

/// « Draw a card then discard a random card » : renvoie la carte défaussée.
fn draw_then_discard_random(g: &mut GameState, p: PlayerId) -> Option<ObjectId> {
    g.draw(p);
    g.discard_random(p)
}

/// « You control a card with 6 or more {p} » (Predatory Plating) : une carte
/// (pas un token, pas une attaque-proxy) dans l'arène, avec sa puissance
/// actuelle : arme, équipement, carte qui attaque ou qui défend, permanent.
fn controls_power6_card(g: &GameState, p: PlayerId) -> bool {
    (0..g.objects.len() as u32).any(|i| {
        let id = ObjectId(i);
        let o = g.obj(id);
        let d = g.def(id);
        o.zone.in_arena()
            && o.zone != Zone::Hero
            && o.controller == p
            && o.proxy_of.is_none()
            && !d.types.has(Types::TOKEN)
            && d.power.is_some()
            && g.power(id) >= 6
    })
}

/// Le moniker « Blasmophet » (CR 2.7.3 : « Blasmophet, the Insatiable
/// Hunger » a le moniker « Blasmophet »).
fn is_blasmophet(g: &GameState, id: ObjectId) -> bool {
    g.def(id).name.split(',').next() == Some("Blasmophet")
}

/// Les Blasmophet que `p` contrôle (permanents, et son héros).
fn blasmophets(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    g.permanents
        .iter()
        .copied()
        .chain(std::iter::once(g.player(p).hero))
        .filter(|&o| g.obj(o).controller == p && is_blasmophet(g, o))
        .collect()
}

/// « As an additional cost to play this, banish 3 random cards from your
/// graveyard » : coût d'effet. `c.flag` : une carte de 6{p} ou plus a été
/// bannie ainsi. Le coût n'est payable que s'il y a 3 cartes au cimetière
/// (CR 1.14.4b, 5.1.8a : voir [`three_in_graveyard`]).
fn banish_3_random(g: &mut GameState, c: &mut Ctx) {
    c.flag = false;
    for _ in 0..3 {
        let gy = &g.player(c.controller).graveyard;
        if gy.is_empty() {
            return;
        }
        let i = g.rng.below(gy.len() as u32) as usize;
        let x = g.player(c.controller).graveyard[i];
        c.flag |= printed_power6(g, x);
        g.banish(c.controller, x, false);
    }
}

/// Condition de jeu des cartes qui bannissent 3 cartes du cimetière.
fn three_in_graveyard(g: &GameState, p: PlayerId, _: ObjectId) -> bool {
    g.player(p).graveyard.len() >= 3
}

/// « If this would be put into your graveyard from anywhere, instead banish
/// it » (Embraforged Gauntlet, Engulfing Shadows).
fn banish_instead_of_graveyard() -> ReplacementDef {
    ReplacementDef {
        kind: ReplaceKind::ToGraveyard,
        anywhere: true,
        optional: false,
        applies: |g, w, p| p.object == w.me && w.me.is_some_and(|me| g.obj(me).owner == p.player),
        apply: |_, _, p| p.zone = Zone::Banished,
    }
}

/// « Banish this from your hand » (coût d'effet d'une capacité de la main).
fn banish_this_from_hand(g: &mut GameState, c: &mut Ctx) {
    if let Some(s) = c.source_id().filter(|&s| g.obj(s).zone == Zone::Hand) {
        g.banish(c.controller, s, false);
    }
}

/// Pose une question « choisir un objet » ; pas de question s'il n'y a
/// qu'un choix possible (voir [`picked`]).
fn ask_object(p: PlayerId, label: &'static str, options: Vec<ObjectId>) -> Option<Ask> {
    (options.len() > 1).then(|| Ask {
        player: p,
        label,
        options: options.into_iter().map(Choice::Object).collect(),
    })
}

/// L'objet choisi par [`ask_object`], ou le seul candidat.
fn picked(c: &Ctx, candidates: &[ObjectId]) -> Option<ObjectId> {
    match c.answer {
        Some(Choice::Object(x)) if candidates.contains(&x) => Some(x),
        _ if candidates.len() == 1 => Some(candidates[0]),
        _ => None,
    }
}

/// Une carte d'attaque Shadow Brute avec blood debt.
fn shadow_brute_attack(
    name: &str,
    color: Color,
    cost: u8,
    power: u8,
    defense: Option<u8>,
) -> CardDef {
    let mut d = deck_card(
        name,
        color,
        SH | BR,
        Types::ACTION,
        Some(cost),
        Some(power),
        defense,
    );
    d.keywords = Keywords::BLOOD_DEBT;
    d
}

/// Puissance imprimée selon la couleur, pour une carte rouge / jaune / bleue.
fn by_index(i: usize, v: [u8; 3]) -> u8 {
    v[i]
}

pub fn add(db: &mut CardDb) {
    hero_and_arena(db);
    deck_cards(db);
    tokens(db);
}

fn hero_and_arena(db: &mut CardDb) {
    // Levia — Shadow Brute Hero - Young, 20 PV, intellect 4.
    // « If a card with 6 or more {p} has been put into your banished zone this
    // turn, you don't lose {h} from blood debt during the end phase. »
    db.add(CardDef {
        name: "Levia".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: SH | BR,
        blood_debt_immune: Some(|g, p| g.banished_power6_this_turn(p)),
        ..Default::default()
    });

    // Hell Hammer — Shadow Brute Weapon - Hammer (2H), 6{p}.
    // « Once per Turn Action -- {r}{r}: Attack
    // When the combat chain closes, if this attacked, banish it.
    // Blood Debt »
    db.add(CardDef {
        name: "Hell Hammer".into(),
        power: Some(6),
        types: Types::WEAPON,
        subtypes: Subtypes::TWO_HAND | Subtypes::HAMMER,
        supertypes: SH | BR,
        keywords: Keywords::BLOOD_DEBT,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 2,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        triggers: vec![TriggerDef {
            // CR 7.7.4 : la chaîne est encore là (« if this attacked »).
            when: |g, me, e| {
                matches!(e, Event::CombatChainClosed)
                    && g.combat.links.iter().any(|l| l.source == Some(me))
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.banish(c.controller, s.id, false);
                }
            })],
        }],
        ..Default::default()
    });

    // Buzzard Helm — Brute Equipment - Head, déf. 1.
    // « When this defends, draw a card then discard a random card. If a card
    // with 6 or more {p} is discarded this way, this gets +1{d} this turn.
    // Temper »
    let mut d = equipment("Buzzard Helm", BR, Subtypes::HEAD, 1);
    d.keywords = Keywords::TEMPER;
    d.triggers = vec![TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            let discarded = draw_then_discard_random(g, c.controller);
            if discarded.is_some_and(|x| printed_power6(g, x)) {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    let d = Delta {
                        defense: 1,
                        ..Default::default()
                    };
                    g.modify(s, d, Until::EndOfTurn);
                }
            }
        })],
    }];
    db.add(d);

    // Predatory Plating — Brute Equipment - Chest, déf. 2.
    // « Instant -- Destroy this: Gain {r}. Activate this only if you control a
    // card with 6 or more {p}.
    // Guardwell »
    let mut d = equipment("Predatory Plating", BR, Subtypes::CHEST, 2);
    d.keywords = Keywords::GUARDWELL;
    d.activated = vec![ActivatedDef {
        label: "Détruire : gagner {r}",
        speed: Speed::Instant,
        condition: Some(|g, p, _| controls_power6_card(g, p)),
        effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(d);

    // Skera Strapping — Brute Equipment - Arms, déf. 0.
    // « If you've pitched a card with 6 or more {p} this turn, this gets
    // spellvoid 3. »
    let mut d = equipment("Skera Strapping", BR, Subtypes::ARMS, 0);
    d.self_mod = Some(|g, me| {
        let p = g.obj(me).controller;
        let pitched = g.turn_events().iter().any(|e| {
            matches!(e, Event::Pitched { player, card } if *player == p && printed_power6(g, *card))
        });
        if pitched {
            Delta {
                spellvoid: 3,
                ..Default::default()
            }
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Embraforged Gauntlet — Shadow Equipment - Arms, déf. 2.
    // « If this would be put into your graveyard from anywhere, instead banish it.
    // Temper
    // Blood Debt »
    let mut d = equipment("Embraforged Gauntlet", SH, Subtypes::ARMS, 2);
    d.keywords = Keywords::TEMPER | Keywords::BLOOD_DEBT;
    d.replacements = vec![banish_instead_of_graveyard()];
    db.add(d);

    // Hex Gauntlet — Shadow Brute Equipment - Arms, 6{p}, sans défense.
    // « Instant -- Banish this: Turn a card with blood debt in your banished
    // zone face-down.
    // Blood Debt »
    // Sans propriété défense, il ne peut pas défendre (CR 7.3.2b). Hex
    // Gauntlet lui-même est un choix possible (il est dans la zone de
    // bannissement quand la capacité se résout).
    db.add(CardDef {
        name: "Hex Gauntlet".into(),
        power: Some(6),
        types: Types::EQUIPMENT,
        subtypes: Subtypes::ARMS,
        supertypes: SH | BR,
        keywords: Keywords::BLOOD_DEBT,
        activated: vec![ActivatedDef {
            label: "Bannir : retourner face cachée une carte avec blood debt bannie",
            speed: Speed::Instant,
            effect_cost: vec![Op::Do(|g, c| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.banish(c.controller, s.id, false);
                }
            })],
            script: vec![
                Op::Ask(|g, c| {
                    ask_object(
                        c.controller,
                        "Quelle carte avec blood debt retourner face cachée ?",
                        face_up_blood_debt(g, c.controller),
                    )
                }),
                Op::Do(|g, c| {
                    if let Some(x) = picked(c, &face_up_blood_debt(g, c.controller)) {
                        g.turn_face_down(x);
                    }
                }),
            ],
            ..Default::default()
        }],
        ..Default::default()
    });

    // Hooves of the Shadowbeast — Shadow Brute Equipment - Legs, déf. 1.
    // « Whenever a card with 6 or more {p} is put into your banished zone, you
    // may destroy this. If you do, gain 1 action point.
    // Battleworn »
    let mut d = equipment("Hooves of the Shadowbeast", SH | BR, Subtypes::LEGS, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            matches!(e, Event::Banished { owner, power, .. }
                if *owner == g.obj(me).controller && *power >= 6)
        },
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                c.source.filter(|&s| g.is_current(s))?;
                Some(Ask {
                    player: c.controller,
                    label: "Détruire Hooves of the Shadowbeast pour gagner 1 point d'action ?",
                    options: vec![Choice::Yes, Choice::No],
                })
            }),
            Op::Do(|g, c| {
                if c.answer == Some(Choice::Yes) {
                    if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                        g.destroy(s.id);
                        // CR 1.13.2b : seulement pendant sa phase d'action.
                        g.gain_action_points(c.controller, 1);
                    }
                }
            }),
        ],
    }];
    db.add(d);
}

/// Les cartes avec blood debt, face visible, de la zone de bannissement de `p`.
fn face_up_blood_debt(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    g.player(p)
        .banished
        .iter()
        .copied()
        .filter(|&x| !g.obj(x).face_down && g.def(x).keywords.has(Keywords::BLOOD_DEBT))
        .collect()
}

fn deck_cards(db: &mut CardDb) {
    // Boneyard Marauder (rouge) — Shadow Brute Action - Attack, 6, coût 1, déf. 3.
    // « As an additional cost to play this, banish 3 random cards from your
    // graveyard.
    // Blood Debt »
    let mut d = shadow_brute_attack("Boneyard Marauder", R, 1, 6, Some(3));
    d.play_condition = Some(three_in_graveyard);
    d.effect_cost = vec![Op::Do(banish_3_random)];
    db.add(d);

    // Dread Screamer — Shadow Brute Action - Attack, 6 / 5 / 4, coût 2, déf. 3.
    // « As an additional cost to play this, banish 3 random cards from your
    // graveyard.
    // If a card with 6 or more {p} is banished this way, this gets go again.
    // Blood Debt »
    // Endless Maw — Shadow Brute Action - Attack, 6 / 5 / 4, coût 3, déf. 3.
    // « … If a card with 6 or more {p} is banished this way, this gets +3{p}. … »
    for (i, &color) in COLORS.iter().enumerate() {
        let p = by_index(i, [6, 5, 4]);
        let mut d = shadow_brute_attack("Dread Screamer", color, 2, p, Some(3));
        d.play_condition = Some(three_in_graveyard);
        d.effect_cost = vec![Op::Do(|g, c| {
            banish_3_random(g, c);
            if c.flag {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.modify(s, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                }
            }
        })];
        db.add(d);

        let mut d = shadow_brute_attack("Endless Maw", color, 3, p, Some(3));
        d.play_condition = Some(three_in_graveyard);
        d.effect_cost = vec![Op::Do(|g, c| {
            banish_3_random(g, c);
            if c.flag {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.modify(s, power(3), Until::EndOfTurn);
                }
            }
        })];
        db.add(d);
    }

    // Cleave the Heavens — Shadow Brute Action - Attack, 6 / 5 / 4, coût 2, déf. 2.
    // « Instant -- Banish this from your hand: Create a Gate to i'Arathael token.
    // Blood Debt »
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = shadow_brute_attack(
            "Cleave the Heavens",
            color,
            2,
            by_index(i, [6, 5, 4]),
            Some(2),
        );
        d.activated = vec![ActivatedDef {
            label: "Bannir de la main : créer un Gate to i'Arathael",
            speed: Speed::Instant,
            from_hand: true,
            effect_cost: vec![Op::Do(banish_this_from_hand)],
            script: vec![Op::Do(|g, c| create(g, c.controller, GATE, 1))],
            ..Default::default()
        }];
        db.add(d);
    }

    // Countdown to Extinction — Shadow Action - Attack, 6 / 5 / 4, coût 3, déf. 3.
    // « When this attacks, create a Gate to i'Arathael token.
    // When this hits, you may search your deck for a Darkest Hour, banish it,
    // then shuffle.
    // Blood Debt »
    // Aucun de nos decks ne contient Darkest Hour : la recherche ne trouve
    // rien, mais le deck est mélangé si le joueur cherche.
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Countdown to Extinction",
            color,
            SH,
            Types::ACTION,
            Some(3),
            Some(by_index(i, [6, 5, 4])),
            Some(3),
        );
        d.keywords = Keywords::BLOOD_DEBT;
        d.triggers = vec![
            TriggerDef {
                when: when_this_attacks,
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| create(g, c.controller, GATE, 1))],
            },
            TriggerDef {
                when: super::generic::this_hits,
                anywhere: false,
                targets: vec![],
                script: vec![
                    Op::Ask(|_, c| {
                        Some(Ask {
                            player: c.controller,
                            label: "Chercher un Darkest Hour dans votre deck (puis mélanger) ?",
                            options: vec![Choice::Yes, Choice::No],
                        })
                    }),
                    Op::If(|_, c| c.answer == Some(Choice::Yes), 2),
                    Op::Ask(|g, c| {
                        ask_object(
                            c.controller,
                            "Quel Darkest Hour bannir ?",
                            darkest_hours(g, c.controller),
                        )
                    }),
                    Op::Do(|g, c| {
                        if let Some(x) = picked(c, &darkest_hours(g, c.controller)) {
                            g.banish(c.controller, x, false);
                        }
                        g.shuffle_deck(c.controller);
                    }),
                ],
            },
        ];
        db.add(d);
    }

    // Dam the Shadowake — Shadow Block, déf. 4 / 3 / 2.
    // « When this defends a Shadow hero's attack, create a Gate to i'Arathael
    // token. »
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Dam the Shadowake",
            color,
            SH,
            Types::BLOCK,
            None,
            None,
            Some(by_index(i, [4, 3, 2])),
        );
        d.triggers = vec![TriggerDef {
            // L'attaque d'un héros Shadow : le héros du joueur qui attaque.
            when: |g, me, e| {
                matches!(e, Event::Defend { card, attack, .. } if *card == me && {
                    let hero = g.player(g.obj(*attack).controller).hero;
                    g.supertypes(hero).has(SH)
                })
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| create(g, c.controller, GATE, 1))],
        }];
        db.add(d);
    }

    // Deadwood Rumbler — Shadow Brute Action - Attack, 8 / 7 / 6, coût 3, sans défense.
    // « When this attacks, draw a card then discard a random card. If a card
    // with 6 or more {p} is discarded this way, banish a card from a graveyard.
    // Blood Debt »
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = shadow_brute_attack("Deadwood Rumbler", color, 3, by_index(i, [8, 7, 6]), None);
        d.triggers = vec![TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Do(|g, c| {
                    let x = draw_then_discard_random(g, c.controller);
                    c.flag = x.is_some_and(|x| printed_power6(g, x));
                }),
                Op::If(|_, c| c.flag, 2),
                Op::Ask(|g, c| {
                    ask_object(
                        c.controller,
                        "Quelle carte bannir d'un cimetière ?",
                        all_graveyards(g),
                    )
                }),
                Op::Do(|g, c| {
                    if let Some(x) = picked(c, &all_graveyards(g)) {
                        g.banish(c.controller, x, false);
                    }
                }),
            ],
        }];
        db.add(d);
    }

    // Feasting Shadowbeast — Shadow Brute Action - Attack, 6 / 5 / 4, coût 2, déf. 3.
    // « When this attacks, banish the top card of your deck.
    // If you've banished a card with 6 or more {p} this turn, this gets +2{p}.
    // Blood Debt »
    // Feeding Frenzy — même chose, « … this gets go again. »
    for (i, &color) in COLORS.iter().enumerate() {
        let p = by_index(i, [6, 5, 4]);
        for name in ["Feasting Shadowbeast", "Feeding Frenzy"] {
            let mut d = shadow_brute_attack(name, color, 2, p, Some(3));
            d.triggers = vec![TriggerDef {
                when: when_this_attacks,
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| {
                    if let Some(x) = g.top_of_deck(c.controller) {
                        g.banish(c.controller, x, false);
                    }
                })],
            }];
            type SelfMod = fn(&GameState, ObjectId) -> Delta;
            let f: SelfMod = if name == "Feasting Shadowbeast" {
                |g, me| {
                    if g.you_banished_power6_this_turn(g.obj(me).controller) {
                        power(2)
                    } else {
                        Delta::default()
                    }
                }
            } else {
                |g, me| {
                    if g.you_banished_power6_this_turn(g.obj(me).controller) {
                        grant(Keywords::GO_AGAIN)
                    } else {
                        Delta::default()
                    }
                }
            };
            d.self_mod = Some(f);
            db.add(d);
        }
    }

    // Graveling Growl — Shadow Brute Action - Attack, 7 / 6 / 5, coût 1, déf. 3.
    // « Play this only if a card with 6 or more {p} has been put into your
    // banished zone this turn.
    // Blood Debt »
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d =
            shadow_brute_attack("Graveling Growl", color, 1, by_index(i, [7, 6, 5]), Some(3));
        d.play_condition = Some(|g, p, _| g.banished_power6_this_turn(p));
        db.add(d);
    }

    // Agile Windup — Brute / Warrior Action - Attack, 7 / 6 / 5, coût 3, déf. 2.
    // « Instant -- Discard this: Create an Agility token. »
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Agile Windup",
            color,
            BR | Supertypes::WARRIOR,
            Types::ACTION,
            Some(3),
            Some(by_index(i, [7, 6, 5])),
            Some(2),
        );
        d.activated = vec![ActivatedDef {
            label: "Défausser : créer un Agility",
            speed: Speed::Instant,
            from_hand: true,
            effect_cost: vec![Op::Do(|g, c| {
                if let Some(s) = c.source_id().filter(|&s| g.obj(s).zone == Zone::Hand) {
                    g.discard(s);
                }
            })],
            script: vec![Op::Do(|g, c| create(g, c.controller, "Agility", 1))],
            ..Default::default()
        }];
        db.add(d);
    }

    // Clash of Agility — Brute / Warrior Action - Attack, 6 / 5 / 4, coût 2, déf. 3.
    // Test of Agility (rouge) — Brute / Warrior Block, déf. 4.
    // « When this defends, clash with the attacking hero. The winner creates
    // an Agility token. »
    let agility = || TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| clash_with_attacker(g, c, "Agility"))],
    };
    let bw = BR | Supertypes::WARRIOR;
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Clash of Agility",
            color,
            bw,
            Types::ACTION,
            Some(2),
            Some(by_index(i, [6, 5, 4])),
            Some(3),
        );
        d.triggers = vec![agility()];
        db.add(d);
    }
    let mut d = deck_card("Test of Agility", R, bw, Types::BLOCK, None, None, Some(4));
    d.triggers = vec![agility()];
    db.add(d);

    // Vigorous Smashup — Brute Action - Attack, 7 / 6 / 5, coût 3, déf. 3.
    // « When this defends, clash with the attacking hero. The winner creates a
    // Vigor token. You may put your revealed card on the bottom of its owner's
    // deck. »
    // La carte révélée reste sur le deck (CR 8.5.17c) : elle est encore sur le
    // dessus du deck quand le joueur choisit.
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Vigorous Smashup",
            color,
            BR,
            Types::ACTION,
            Some(3),
            Some(by_index(i, [7, 6, 5])),
            Some(3),
        );
        d.triggers = vec![TriggerDef {
            when: when_this_defends,
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Do(|g, c| {
                    c.picked = g.top_of_deck(c.controller).into_iter().collect();
                    clash_with_attacker(g, c, "Vigor");
                }),
                Op::Ask(|g, c| {
                    let x = *c.picked.first()?;
                    (g.top_of_deck(c.controller) == Some(x)).then(|| Ask {
                        player: c.controller,
                        label: "Mettre votre carte révélée sous votre deck ?",
                        options: vec![Choice::Yes, Choice::No],
                    })
                }),
                Op::Do(|g, c| {
                    if c.answer == Some(Choice::Yes) {
                        if let Some(&x) = c.picked.first() {
                            g.put_on_bottom(x);
                        }
                    }
                }),
            ],
        }];
        db.add(d);
    }

    // Wrecker Romp — Brute Action - Attack, 8 / 7 / 6, coût 2, déf. 3.
    // « As an additional cost to play this, discard a random card. »
    // Coût impayable sans autre carte en main (CR 1.14.4b, 5.1.8a).
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Wrecker Romp",
            color,
            BR,
            Types::ACTION,
            Some(2),
            Some(by_index(i, [8, 7, 6])),
            Some(3),
        );
        d.play_condition = Some(|g, p, me| g.player(p).hand.iter().any(|&c| c != me));
        d.effect_cost = vec![Op::Do(|g, c| {
            g.discard_random(c.controller);
        })];
        db.add(d);
    }

    // Consuming Strength (jaune) — Shadow Brute Action - Attack, 6, coût 2, déf. 3.
    // « Play this only if you control a Blasmophet.
    // Instant -- {r}, banish this from your hand: Your next attack this turn
    // gets +2{p}.
    // Blood Debt »
    let mut d = shadow_brute_attack("Consuming Strength", Y, 2, 6, Some(3));
    d.play_condition = Some(|g, p, _| !blasmophets(g, p).is_empty());
    d.activated = vec![ActivatedDef {
        label: "{r}, bannir de la main : votre prochaine attaque a +2{p}",
        speed: Speed::Instant,
        cost: 1,
        from_hand: true,
        effect_cost: vec![Op::Do(banish_this_from_hand)],
        script: vec![Op::Do(|g, c| {
            g.next_attack(c.controller, None, power(2), false);
        })],
        ..Default::default()
    }];
    db.add(d);

    // Engulfing Shadows (jaune) — Shadow Defense Reaction, coût 0, déf. 4.
    // « If this would be put into your graveyard from anywhere, instead banish it.
    // Blood Debt »
    let mut d = deck_card(
        "Engulfing Shadows",
        Y,
        SH,
        Types::DEFENSE_REACTION,
        Some(0),
        None,
        Some(4),
    );
    d.keywords = Keywords::BLOOD_DEBT;
    d.replacements = vec![banish_instead_of_graveyard()];
    db.add(d);

    // Goremass Summoning (bleu) — Shadow Brute Action, coût 2, déf. 3.
    // « If you've banished a card with 6 or more {p} this turn, create a
    // Blasmophet, the Insatiable Hunger token.
    // Go again »
    // Unique (CR 8.3.48a) : si le joueur contrôle alors deux Blasmophet, il
    // en garde un (son choix) et l'autre est retiré (« cleared ») comme action
    // d'état, juste après la résolution.
    let mut d = deck_card(
        "Goremass Summoning",
        B,
        SH | BR,
        Types::ACTION,
        Some(2),
        None,
        Some(3),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![
        Op::Do(|g, c| {
            if g.you_banished_power6_this_turn(c.controller) {
                create(g, c.controller, BLASMOPHET, 1);
            }
        }),
        Op::Ask(|g, c| {
            let v = blasmophets(g, c.controller);
            (v.len() >= 2).then(|| Ask {
                player: c.controller,
                label: "Unique : quel Blasmophet garder ?",
                options: v.into_iter().map(Choice::Object).collect(),
            })
        }),
        Op::Do(|g, c| {
            let v = blasmophets(g, c.controller);
            if v.len() >= 2 {
                let keep = picked(c, &v).unwrap_or(v[0]);
                for x in v {
                    if x != keep && g.obj(x).zone == Zone::Permanent {
                        g.clear(x);
                    }
                }
            }
        }),
    ];
    db.add(d);

    // Smashback Alehorn (bleu) — Brute Action, coût 0, déf. 2.
    // « Create an Agility and a Might token.
    // Go again »
    let mut d = deck_card(
        "Smashback Alehorn",
        B,
        BR,
        Types::ACTION,
        Some(0),
        None,
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![Op::Do(|g, c| {
        create(g, c.controller, "Agility", 1);
        create(g, c.controller, "Might", 1);
    })];
    db.add(d);

    // Soul Harvest (bleu) — Shadow Action - Attack, 6, coût 6, déf. 3.
    // « Legendary Levia Specialization
    // As an additional cost to play this, banish 6 cards from your graveyard.
    // This gets +1{p} for each card with blood debt banished this way.
    // When this hits a hero, banish all cards in their soul. They lose {h}
    // equal to the number of cards banished this way. »
    // Legendary / Specialization : règles de construction du deck. Aucun
    // héros de nos 9 decks n'a d'âme (soul, cartes Light) : le moteur n'a pas
    // de zone d'âme, la capacité « when this hits » ne bannit donc rien.
    let mut d = deck_card(
        "Soul Harvest",
        B,
        SH,
        Types::ACTION,
        Some(6),
        Some(6),
        Some(3),
    );
    d.play_condition = Some(|g, p, _| g.player(p).graveyard.len() >= 6);
    d.effect_cost = vec![
        Op::Ask(|g, c| {
            let gy = &g.player(c.controller).graveyard;
            // Plus de choix si toutes les cartes restantes doivent partir.
            (gy.len() > 6 - c.picked.len()).then(|| Ask {
                player: c.controller,
                label: "Soul Harvest : quelle carte bannir de votre cimetière ?",
                options: gy.iter().map(|&x| Choice::Object(x)).collect(),
            })
        }),
        Op::Do(|g, c| {
            let gy = g.player(c.controller).graveyard.clone();
            let x = match c.answer.take() {
                Some(Choice::Object(x)) if gy.contains(&x) => Some(x),
                _ => gy.first().copied(),
            };
            match x {
                Some(x) => {
                    g.banish(c.controller, x, false);
                    c.picked.push(x);
                }
                None => c.flag = true, // cimetière vide (ne devrait pas arriver)
            }
        }),
        Op::Back(|_, c| !c.flag && c.picked.len() < 6, 2),
        Op::Do(|g, c| {
            let n = c
                .picked
                .iter()
                .filter(|&&x| g.def(x).keywords.has(Keywords::BLOOD_DEBT))
                .count() as i16;
            if n > 0 {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.modify(s, power(n), Until::EndOfTurn);
                }
            }
        }),
    ];
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            super::generic::this_hits(g, me, e)
                && matches!(
                    e,
                    Event::Hit {
                        target_obj: None,
                        ..
                    }
                )
        },
        anywhere: false,
        targets: vec![],
        // Âme toujours vide dans nos decks : 0 carte bannie, 0 PV perdu.
        script: vec![],
    }];
    db.add(d);
}

/// Les Darkest Hour du deck de `p`.
fn darkest_hours(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    g.player(p)
        .deck
        .iter()
        .copied()
        .filter(|&x| g.def(x).name == "Darkest Hour")
        .collect()
}

/// Les cartes de tous les cimetières (« a card from a graveyard »).
fn all_graveyards(g: &GameState) -> Vec<ObjectId> {
    PlayerId::BOTH
        .iter()
        .flat_map(|&p| g.player(p).graveyard.iter().copied())
        .collect()
}

/// Cible « target action card with blood debt from your banished zone ».
const TARGET_BLOOD_DEBT_ACTION_BANISHED: TargetSpec = TargetSpec {
    label: "carte d'action avec blood debt bannie",
    filter: |g, c, t| match t {
        Target::Object(r) => {
            let o = g.obj(r.id);
            let d = g.def(r.id);
            o.zone == Zone::Banished
                && !o.face_down
                && o.owner == c.controller
                && d.types.has(Types::ACTION)
                && d.keywords.has(Keywords::BLOOD_DEBT)
        }
        Target::Player(_) => false,
    },
    optional: false,
    in_graveyard: false,
    in_banished: true,
};

fn tokens(db: &mut CardDb) {
    // Gate to i'Arathael — Shadow Token - Aura.
    // « Instant -- {r}, destroy this: You may play target action card with
    // blood debt from your banished zone this turn. »
    db.add(CardDef {
        name: GATE.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: SH,
        activated: vec![ActivatedDef {
            label: "{r}, détruire : jouer une action avec blood debt bannie ce tour",
            speed: Speed::Instant,
            cost: 1,
            targets: vec![TARGET_BLOOD_DEBT_ACTION_BANISHED],
            effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
            script: vec![Op::Do(|g, c| {
                if let Some(r) = target_obj(c, 0) {
                    let turn = g.turn;
                    g.permit_play(c.controller, r.id, turn);
                }
            })],
            ..Default::default()
        }],
        ..Default::default()
    });

    // Blasmophet, the Insatiable Hunger — Shadow Token - Demon Ally, 6{p}, 6 PV.
    // « Unique
    // Once per turn, you may play an action card with blood debt from your
    // banished zone.
    // At the beginning of each end phase, you may banish a card from your
    // hand. Then if you haven't banished a card with blood debt this turn,
    // destroy this. »
    // Pas de capacité « Attack » : ce Blasmophet ne peut pas attaquer (on
    // peut l'attaquer, c'est un allié). Unique : voir Goremass Summoning.
    db.add(CardDef {
        name: BLASMOPHET.into(),
        power: Some(6),
        life: Some(6),
        types: Types::TOKEN,
        subtypes: Subtypes::ALLY | Subtypes::DEMON,
        supertypes: SH,
        play_from_banished: Some(|g, _, card| {
            let d = g.def(card);
            d.types.has(Types::ACTION) && d.keywords.has(Keywords::BLOOD_DEBT)
        }),
        triggers: vec![TriggerDef {
            when: |_, _, e| matches!(e, Event::BeginEndPhase(_)),
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Ask(|g, c| {
                    let hand = &g.player(c.controller).hand;
                    if hand.is_empty() {
                        return None;
                    }
                    let mut options: Vec<Choice> =
                        hand.iter().map(|&x| Choice::Object(x)).collect();
                    options.push(Choice::Pass);
                    Some(Ask {
                        player: c.controller,
                        label: "Blasmophet : bannir une carte de votre main ?",
                        options,
                    })
                }),
                Op::Do(|g, c| {
                    if let Some(Choice::Object(x)) = c.answer {
                        if g.obj(x).zone == Zone::Hand {
                            g.banish(c.controller, x, false);
                        }
                    }
                    if !g.you_banished_blood_debt_this_turn(c.controller) {
                        super::generic::destroy_source(g, c);
                    }
                }),
            ],
        }],
        ..Default::default()
    });
}
