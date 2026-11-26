//! Cartes Lightning, Elemental et Runeblade du deck Aurora (essai
//! d'encodage deck par deck, 2026-11-26).

#![allow(unused_imports)]

use super::assassin::this_hits_a_hero;
use super::generic::{destroy_source, when_this_attacks};
use super::{by_color, deck_card, equipment, grant, power, target_obj, COLORS};
use fab_engine::card::{CardDb, CardDef, Color, Delta, Keywords, Subtypes, Supertypes, Types};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, Speed, TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Cloud Cover",
    "Sizzle",
    "Electrostatic Discharge",
    "Stinging Sprite",
    "Lightning Surge",
    "Second Strike",
    "Rush of Power",
    "Visionary of Orbits",
    "Harness Lightning",
    "Static Shock",
    "Path of Same Ends",
    "Twinkle Toes",
    "Star Fall",
    "Helm of Might and Magic",
    "Sigil of Voltaris",
    // NOMS
];

#[allow(dead_code)]
const LIGHTNING: Supertypes = Supertypes::LIGHTNING;
#[allow(dead_code)]
const LIGHTNING_RB: Supertypes = Supertypes(Supertypes::LIGHTNING.0 | Supertypes::RUNEBLADE.0);
#[allow(dead_code)]
const ELEMENTAL_RB: Supertypes = Supertypes(Supertypes::ELEMENTAL.0 | Supertypes::RUNEBLADE.0);

pub fn add(db: &mut CardDb) {
    let _ = &db;
    // Cloud Cover (bleu, rouge, jaune) — Lightning Instant, coût 0.
    // « The next time you would be dealt damage this turn, prevent 1 of that
    //   damage. » (bleu : 1 ; rouge : 3 ; jaune : 2)
    // Prévention « fixe » (CR 6.4.10i) : ne vaut que pour un seul événement de dégâts.
    for c in COLORS {
        let mut d = deck_card(
            "Cloud Cover",
            c,
            LIGHTNING,
            Types::INSTANT,
            Some(0),
            None,
            None,
        );
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 1]);
            g.prevent_next_time(c.controller, Some(n as i32));
        })];
        db.add(d);
    }

    // Sizzle (rouge, jaune) — Lightning Action, coût 0, déf. 2.
    // « Your next Lightning or Elemental attack this turn gets +3{p}
    //   (jaune : +2).
    //   Go again »
    // Pas de version bleue : la valeur bleue de by_color n'est jamais utilisée.
    for c in [Color::Red, Color::Yellow] {
        let mut d = deck_card(
            "Sizzle",
            c,
            LIGHTNING,
            Types::ACTION,
            Some(0),
            None,
            Some(2),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 0]);
            g.next_attack(
                c.controller,
                Some(|g, a| {
                    g.supertypes(a).has(Supertypes::LIGHTNING)
                        || g.supertypes(a).has(Supertypes::ELEMENTAL)
                }),
                power(n),
                false,
            );
        })];
        db.add(d);
    }

    // Electrostatic Discharge (bleu, rouge, jaune) — Lightning Instant, coût 0.
    // « The next attack action card you play this turn with cost 1 or less
    //   gets +1{p} (rouge : +3 ; jaune : +2). »
    for c in COLORS {
        let mut d = deck_card(
            "Electrostatic Discharge",
            c,
            LIGHTNING,
            Types::INSTANT,
            Some(0),
            None,
            None,
        );
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 1]);
            g.next_attack(
                c.controller,
                Some(|g, a| {
                    super::generic::is_attack_action_card(g, a) && g.def(a).cost.unwrap_or(0) <= 1
                }),
                power(n),
                true,
            );
        })];
        db.add(d);
    }

    // Stinging Sprite (bleu, rouge, jaune) — Lightning Runeblade Action - Attack,
    // coût 0, déf. 2.
    // « When this attacks or defends, deal 1 arcane damage to target hero. »
    // (puissance : bleu 1 ; rouge 3 ; jaune 2)
    for (c, p) in [(Color::Blue, 1), (Color::Red, 3), (Color::Yellow, 2)] {
        let mut d = deck_card(
            "Stinging Sprite",
            c,
            LIGHTNING_RB,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(2),
        );
        d.arcane_damage = true;
        d.triggers = vec![TriggerDef {
            when: |g, me, e| {
                super::generic::when_this_attacks(g, me, e)
                    || super::generic::when_this_defends(g, me, e)
            },
            anywhere: false,
            targets: vec![super::wizard::TARGET_HERO],
            script: vec![Op::Do(|g, c| {
                if let Some(t) = c.target(0) {
                    g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                }
            })],
        }];
        db.add(d);
    }

    // Lightning Surge (bleu, rouge, jaune) — Lightning Action - Attack, coût 0,
    // défense 2 ; puissance bleu 2, rouge 4, jaune 3.
    // « If this was played from arsenal, it gets go again. »
    for (c, p) in [(Color::Blue, 2), (Color::Red, 4), (Color::Yellow, 3)] {
        let mut d = deck_card(
            "Lightning Surge",
            c,
            LIGHTNING,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(2),
        );
        d.self_mod = Some(|g, me| {
            if joue_depuis_arsenal(g, me) {
                grant(Keywords::GO_AGAIN)
            } else {
                Delta::default()
            }
        });
        db.add(d);
    }

    // Second Strike (bleu, rouge, jaune) — Lightning Action - Attack, coût 0,
    // déf. 2. Puissance : bleu 1 ; rouge 3 ; jaune 2.
    // « When this attacks, if you've dealt damage this turn, this gets +1{p}
    //   and **go again**. »
    // La condition est vérifiée au déclenchement (quand l'attaque est déclarée),
    // comme le texte le demande.
    for (c, p) in [(Color::Blue, 1), (Color::Red, 3), (Color::Yellow, 2)] {
        let mut d = deck_card(
            "Second Strike",
            c,
            LIGHTNING,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(2),
        );
        d.triggers = vec![TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if a_inflige_des_degats(g, c.controller) {
                    if let Some(s) = c.source {
                        g.modify(
                            s,
                            Delta {
                                power: 1,
                                grants: Keywords::GO_AGAIN,
                                ..Default::default()
                            },
                            Until::EndOfTurn,
                        );
                    }
                }
            })],
        }];
        db.add(d);
    }

    // Rush of Power (bleu, rouge, jaune) — Lightning Runeblade Action - Attack,
    // coût 0, déf. 3. Puissance : bleu 1 ; rouge 3 ; jaune 2.
    // « **Quickstrike** - If this has go again, it gets +1{p}.
    //   When this hits a hero, deal 1 arcane damage to them. »
    // TODO(règles) : Quickstrike n'est pas un mot-clé du moteur. Le +1{p} est
    // calculé au moment où l'attaque est déclarée (comme Second Strike), car
    // « go again » peut être accordé avant l'attaque ; à confirmer dans le CR.
    for (c, p) in [(Color::Blue, 1), (Color::Red, 3), (Color::Yellow, 2)] {
        let mut d = deck_card(
            "Rush of Power",
            c,
            LIGHTNING_RB,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(3),
        );
        d.arcane_damage = true;
        d.triggers = vec![
            TriggerDef {
                when: when_this_attacks,
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| {
                    if let Some(s) = c.source {
                        if g.keywords(s.id).has(Keywords::GO_AGAIN) {
                            g.modify(
                                s,
                                Delta {
                                    power: 1,
                                    ..Default::default()
                                },
                                Until::EndOfTurn,
                            );
                        }
                    }
                })],
            },
            TriggerDef {
                when: |g, me, e| super::assassin::this_hits_a_hero(g, me, e),
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| {
                    if let Some(Event::Hit { target, .. }) = c.event {
                        g.deal_damage_to(
                            c.source_id(),
                            c.controller,
                            Target::Player(target),
                            1,
                            true,
                        );
                    }
                })],
            },
        ];
        db.add(d);
    }

    // Visionary of Orbits (rouge) — Lightning Action - Attack, coût 3,
    // 7{p} 2{d}.
    // « When this hits, you may put an instant card from your graveyard on
    //   the bottom of your deck. »
    let mut d = deck_card(
        "Visionary of Orbits",
        Color::Red,
        LIGHTNING,
        Types::ACTION,
        Some(3),
        Some(7),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: super::generic::this_hits,
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                let mut options: Vec<Choice> = g
                    .player(c.controller)
                    .graveyard
                    .iter()
                    .filter(|&&x| g.def(x).types.any(Types::INSTANT))
                    .map(|&x| Choice::Object(x))
                    .collect();
                if options.is_empty() {
                    return None;
                }
                options.push(Choice::Pass);
                Some(Ask {
                    player: c.controller,
                    label: "Carte instant du cimetière sous le deck (Pass : aucune)",
                    options,
                })
            }),
            Op::Do(|g, c| {
                if let Some(Choice::Object(x)) = c.answer.take() {
                    if g.obj(x).zone == Zone::Graveyard && g.obj(x).owner == c.controller {
                        g.put_on_bottom(x);
                    }
                }
            }),
        ],
    }];
    db.add(d);

    // Harness Lightning (rouge, jaune) — Elemental Runeblade Action, coût 0,
    // déf. 2. Pas de puissance.
    // « **Lightning Flow** - If you've played a Lightning card this turn,
    //   deal 3 arcane damage to target hero. » (rouge : 3 ; jaune : 2)
    // La carte elle-même n'est pas une carte Lightning : seule une autre carte
    // Lightning jouée ce tour compte.
    for c in [Color::Red, Color::Yellow] {
        let mut d = deck_card(
            "Harness Lightning",
            c,
            ELEMENTAL_RB,
            Types::ACTION,
            Some(0),
            None,
            Some(2),
        );
        d.targets = vec![super::wizard::TARGET_HERO];
        d.arcane_damage = true;
        d.resolve = vec![Op::Do(|g, c| {
            if a_joue_une_carte_lightning(g, c.controller) {
                if let Some(t) = c.target(0) {
                    let n = by_color(g, c, [3, 2, 0]);
                    g.deal_damage_to(c.source_id(), c.controller, t, n as i32, true);
                }
            }
        })];
        db.add(d);
    }

    // Static Shock (rouge, jaune) — Elemental Runeblade Action - Attack, coût 0,
    // déf. 3. Puissance : rouge 4 ; jaune 3.
    // « **Lightning Flow** - When this hits a hero, if you've played a Lightning
    //   card this turn, deal 1 arcane damage to them. »
    // La carte elle-même n'est pas une carte Lightning : seule une autre carte
    // Lightning jouée ce tour compte.
    for (c, p) in [(Color::Red, 4), (Color::Yellow, 3)] {
        let mut d = deck_card(
            "Static Shock",
            c,
            ELEMENTAL_RB,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(3),
        );
        d.arcane_damage = true;
        d.triggers = vec![TriggerDef {
            when: this_hits_a_hero,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if a_joue_une_carte_lightning(g, c.controller) {
                    if let Some(t) = hit_hero(c) {
                        g.deal_damage_to(c.source_id(), c.controller, Target::Player(t), 1, true);
                    }
                }
            })],
        }];
        db.add(d);
    }

    // Path of Same Ends (bleu, rouge, jaune) — Lightning Runeblade Action - Attack,
    // coût 0, déf. 2. Puissance : bleu 1 ; rouge 3 ; jaune 2.
    // « When this attacks a hero, deal 1 arcane damage to them. If damage is dealt
    //   this way, this gets **go again**.
    //   **Instant** -- {r}: This gets **go again**. »
    // « them » = le héros attaqué (événement `Attack`, cible sans objet).
    for (c, p) in [(Color::Blue, 1), (Color::Red, 3), (Color::Yellow, 2)] {
        let mut d = deck_card(
            "Path of Same Ends",
            c,
            LIGHTNING_RB,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(2),
        );
        d.arcane_damage = true;
        d.triggers = vec![TriggerDef {
            when: super::assassin::when_this_attacks_a_hero,
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Do(|g, c| {
                    g.last_damage = 0;
                    if let Some(Event::Attack { target, .. }) = c.event {
                        g.deal_damage_to(
                            c.source_id(),
                            c.controller,
                            Target::Player(target),
                            1,
                            true,
                        );
                    }
                }),
                // `last_damage` est lu à l'instruction suivante.
                Op::Do(|g, c| {
                    if g.last_damage > 0 {
                        if let Some(s) = c.source {
                            g.modify(s, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                        }
                    }
                }),
            ],
        }];
        // TODO(règles) : test de la capacité Instant non écrit : dans le harnais,
        // P0 n'a pas de fenêtre de priorité où la capacité est proposée après la
        // déclaration (Defend, puis fin de chaîne). À vérifier (CR 7 / priorité).
        // La condition ci-dessous limite l'usage à la chaîne de combat.
        d.activated = vec![ActivatedDef {
            label: "{r} : go again",
            speed: Speed::Instant,
            cost: 1,
            condition: Some(|g, _, me| g.obj(me).zone == Zone::CombatChain),
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source {
                    g.modify(s, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                }
            })],
            ..Default::default()
        }];
        db.add(d);
    }

    // Twinkle Toes — Lightning Equipment - Legs, déf. 0, sans couleur.
    // « Instant -- Destroy this: Prevent the next 2 damage that would be dealt
    //   to you this turn. Activate this only if you've played an instant card
    //   this turn. »
    let mut d = equipment("Twinkle Toes", LIGHTNING, Subtypes::LEGS, 0);
    d.activated = vec![ActivatedDef {
        label: "Détruire : prévenir les 2 prochains dégâts",
        speed: Speed::Instant,
        condition: Some(|g, p, _| a_joue_un_instant_ce_tour(g, p)),
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.prevent_next(c.controller, 2, false, None))],
        ..Default::default()
    }];
    db.add(d);

    // Star Fall — Lightning Runeblade Weapon - Sword (2H), 1{p}, sans couleur.
    // « Once per Turn Action -- {r}: **Attack**
    //   If you've played a Lightning card this turn, this card's attacks get +1{p}
    //   and **go again**. »
    db.add(CardDef {
        name: "Star Fall".into(),
        power: Some(1),
        types: Types::WEAPON,
        subtypes: Subtypes::SWORD | Subtypes::TWO_HAND,
        supertypes: LIGHTNING_RB,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 1,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        self_mod: Some(|g, me| {
            // « this card's attacks » : le bonus porte sur l'arme (et son attaque-proxy).
            if a_joue_une_carte_lightning(g, g.obj(me).controller) {
                power(1)
            } else {
                Delta::default()
            }
        }),
        // Le « go again » d'une arme ne passe pas par `self_mod` (il n'atteint
        // pas la proxy d'attaque) : il est accordé à l'attaque elle-même quand
        // elle est déclarée, comme pour Graphene Chelicera (assassin.rs).
        // TODO(règles) : la condition « a joué une carte Lightning ce tour » est
        // évaluée à la déclaration de l'attaque. Une carte Lightning jouée ensuite
        // (en chaîne, avant la résolution) ne donnerait pas le go again ici. À
        // vérifier dans le CR (`docs/rules/sources/regles-cr.txt`).
        triggers: vec![TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if a_joue_une_carte_lightning(g, c.controller) {
                    if let Some(a) = g.active_link().map(|l| l.attack) {
                        g.modify(a, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                    }
                }
            })],
        }],
        ..Default::default()
    });

    // Helm of Might and Magic — Runeblade Equipment - Head, déf. 1 (sans couleur).
    // « Blade Break
    // Spellvoid 1 »
    // Spellvoid 1 : « If you would be dealt arcane damage, you may destroy this
    // to prevent 1 of that damage. » (CR 8.3.15)
    let mut d = equipment(
        "Helm of Might and Magic",
        Supertypes::RUNEBLADE,
        Subtypes::HEAD,
        1,
    );
    d.keywords = Keywords::BLADE_BREAK;
    d.spellvoid = 1;
    db.add(d);

    // Sigil of Voltaris (bleu) — Lightning Runeblade Instant - Aura, coût 0,
    // sans déf. (carte d'arène : pas de défense).
    // « When this enters or leaves the arena, deal 1 arcane damage to target hero.
    //   At the beginning of your action phase, destroy this. »
    let mut d = deck_card(
        "Sigil of Voltaris",
        Color::Blue,
        LIGHTNING_RB,
        Types::INSTANT,
        Some(0),
        None,
        None,
    );
    d.subtypes = Subtypes::AURA;
    d.arcane_damage = true;
    d.triggers = vec![
        TriggerDef {
            when: |g, me, e| match *e {
                Event::EnterArena { card } => card == me && g.obj(me).zone == Zone::Permanent,
                Event::LeaveArena { card, .. } => card == me,
                _ => false,
            },
            // « leaves the arena » : la carte est alors au cimetière.
            anywhere: true,
            targets: vec![super::wizard::TARGET_HERO],
            script: vec![Op::Do(|g, c| {
                if let Some(t) = c.target(0) {
                    g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                }
            })],
        },
        TriggerDef {
            when: |g, me, e| matches!(*e, Event::BeginActionPhase(p) if p == g.obj(me).controller),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| destroy_source(g, c))],
        },
    ];
    db.add(d);

    // CARTES
}

/// Le héros touché (« them ») d'une capacité « when this hits a hero ».
fn hit_hero(c: &Ctx) -> Option<fab_engine::PlayerId> {
    match c.event {
        Some(Event::Hit {
            target,
            target_obj: None,
            ..
        }) => Some(target),
        _ => None,
    }
}

/// « If you've played a Lightning card this turn » : `p` a joué ce tour une
/// carte de supertype Lightning.
fn a_joue_une_carte_lightning(g: &GameState, p: fab_engine::PlayerId) -> bool {
    g.played_def_this_turn(p, |d| d.supertypes.has(Supertypes::LIGHTNING))
}

/// « If you've played an instant card this turn » : `p` a joué ce tour une
/// carte de type Instant.
fn a_joue_un_instant_ce_tour(g: &GameState, p: fab_engine::PlayerId) -> bool {
    g.played_def_this_turn(p, |d| d.types.has(Types::INSTANT))
}

/// « If you've dealt damage this turn » : `p` a infligé des dégâts ce tour
/// (un événement `Damage` n'existe que si des dégâts ont été réellement infligés).
fn a_inflige_des_degats(g: &GameState, p: fab_engine::PlayerId) -> bool {
    g.turn_events()
        .iter()
        .any(|e| matches!(e, Event::Damage { by, .. } if *by == p))
}

/// « If this was played from arsenal » : la carte a été jouée depuis l'arsenal
/// (dernier événement `Played` de cette carte, ce tour).
fn joue_depuis_arsenal(g: &fab_engine::GameState, me: ObjectId) -> bool {
    if !matches!(g.obj(me).zone, Zone::Stack | Zone::CombatChain) {
        return false;
    }
    let from = g.turn_events().iter().rev().find_map(|e| match *e {
        Event::Played { card, from, .. } if card == me => Some(from),
        _ => None,
    });
    from == Some(Zone::Arsenal)
}
