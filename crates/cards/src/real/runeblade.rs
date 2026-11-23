//! Cartes Runeblade, Earth et Elemental du deck Florian.

use super::generic::{destroy_source, is_attack_action_card, this_hits};
use super::{by_color, card_named, deck_card, equipment, power, target_obj, COLORS};
use fab_engine::card::{
    CardDb, CardDef, Color, Delta, Keywords, Split, Subtypes, Supertypes, Types,
};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, CardId, Choice, Event, GameState, ObjectId, PlayerId, ReplaceKind,
    ReplacementDef, Speed, TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Florian",
    "Nebula Blade",
    "Scepter of Pain",
    "Annals of Sutcliffe",
    "Arcane Lantern",
    "Bloodied Oval",
    "Crown of Dichotomy",
    "Helm of Lignum Vitae",
    "Plume of Evergrowth",
    "Blossom of Spring",
    "Unyielding Grip",
    "Well Grounded",
    "Amplify the Arknight",
    "Arcane Seeds // Life",
    "Autumn's Touch",
    "Chorus of Rotwood",
    "Colors of Aria",
    "Fruits of the Forest",
    "Harvest Season",
    "Oath of the Arknight",
    "Read the Runes",
    "Reduce to Runechant",
    "Rootbound Carapace",
    "Runeblood Incantation",
    "Sigil of Silphidae",
    "Runechant",
    "Embodiment of Earth",
];

const G: Supertypes = Supertypes::NONE;
const RB: Supertypes = Supertypes::RUNEBLADE;
const EARTH: Supertypes = Supertypes::EARTH;
const EARTH_RB: Supertypes = Supertypes(Supertypes::EARTH.0 | Supertypes::RUNEBLADE.0);
pub(crate) const RUNECHANT: &str = "Runechant";
const EMBODIMENT: &str = "Embodiment of Earth";

// ---------------------------------------------------------------------------
// Outils communs
// ---------------------------------------------------------------------------

/// « target opposing hero ».
const TARGET_OPPOSING_HERO: TargetSpec = TargetSpec {
    label: "héros adverse",
    filter: |_, c, t| matches!(t, Target::Player(p) if p != c.controller),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// « any opposing target » : le héros adverse ou un allié qu'un adversaire
/// contrôle (CR 1.8.5d : seuls les objets qui ont la vie reçoivent des dégâts).
const ANY_OPPOSING_TARGET: TargetSpec = TargetSpec {
    label: "cible adverse (héros ou allié)",
    filter: |g, c, t| match t {
        Target::Player(p) => p != c.controller,
        Target::Object(r) => g.is_living_permanent(r.id) && g.obj(r.id).controller != c.controller,
    },
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Une carte Earth (Colors of Aria l'est tant qu'elle est face visible).
pub(crate) fn is_earth(g: &GameState, id: ObjectId) -> bool {
    g.supertypes(id).has(Supertypes::EARTH)
}

/// Nombre de cartes Earth dans la zone de bannissement de `p`.
/// TODO(règles) : une carte bannie face cachée compte-t-elle ? On lit pour
/// l'instant son texte imprimé (question posée au propriétaire).
pub(crate) fn earth_banished(g: &GameState, p: PlayerId) -> usize {
    g.player(p)
        .banished
        .iter()
        .filter(|&&c| is_earth(g, c))
        .count()
}

/// « If there are 4 or more Earth cards in your banished zone ».
fn four_earth_banished(g: &GameState, p: PlayerId) -> bool {
    earth_banished(g, p) >= 4
}

/// Nombre de Runechant que `p` contrôle.
pub(crate) fn runechants(g: &GameState, p: PlayerId) -> usize {
    g.permanents
        .iter()
        .filter(|&&o| g.obj(o).controller == p && g.def(o).name == RUNECHANT)
        .count()
}

/// « Create N Runechant tokens » (un seul événement de création, CR 8.5.2).
fn create_runechants(g: &mut GameState, p: PlayerId, n: u8) {
    let r = card_named(g, RUNECHANT);
    g.create_tokens(p, r, n);
}

/// « This costs {r} less to play for each Runechant you control. »
fn less_per_runechant(g: &GameState, p: PlayerId, _: ObjectId) -> i16 {
    -(runechants(g, p) as i16)
}

/// Une carte de `p` dans son cimetière (cible « from your graveyard »).
fn in_your_graveyard(g: &GameState, c: &Ctx, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.zone == Zone::Graveyard && o.owner == c.controller
}

fn defense(n: i16) -> Delta {
    Delta {
        defense: n,
        ..Default::default()
    }
}

/// L'objet est une carte qui défend sur la chaîne de combat.
fn is_defending(g: &GameState, id: ObjectId) -> bool {
    g.obj(id).zone == Zone::CombatChain && g.combat.links.iter().any(|l| l.defenders.contains(&id))
}

// ---------------------------------------------------------------------------
// Decompose (CR 8.4.14) : « You may banish 2 Earth cards and an action card
// from your graveyard. If you do, [EFFECTS]. »
// ---------------------------------------------------------------------------

/// Les cartes déjà choisies (`picked`) peuvent-elles être complétées, avec
/// d'autres cartes du cimetière de `p`, en 3 cartes distinctes : 2 cartes
/// Earth et une carte d'action ? (Une carte Earth d'action peut tenir l'un
/// ou l'autre rôle, pas les deux.)
fn decompose_possible(g: &GameState, p: PlayerId, picked: &[ObjectId]) -> bool {
    if picked.len() > 3 {
        return false;
    }
    let (mut earth, mut earth_action, mut action_only) = (0, 0, 0);
    for &x in &g.player(p).graveyard {
        if picked.contains(&x) {
            continue;
        }
        let (e, a) = (is_earth(g, x), g.def(x).is_action());
        earth += usize::from(e);
        earth_action += usize::from(e && a);
        action_only += usize::from(a && !e);
    }
    // Chaque carte choisie reçoit un rôle : Earth (bit à 1) ou action.
    (0..1u32 << picked.len()).any(|mask| {
        let mut e_used = 0;
        for (i, &x) in picked.iter().enumerate() {
            let as_earth = mask & (1 << i) != 0;
            let ok = if as_earth {
                is_earth(g, x)
            } else {
                g.def(x).is_action()
            };
            if !ok {
                return false;
            }
            e_used += usize::from(as_earth);
        }
        let a_used = picked.len() - e_used;
        if e_used > 2 || a_used > 1 {
            return false;
        }
        let (e, a) = (2 - e_used, 1 - a_used);
        if a == 0 {
            earth >= e
        } else {
            (action_only >= 1 && earth >= e) || (earth_action >= 1 && earth > e)
        }
    })
}

/// Les instructions de decompose. Ensuite, `c.flag` dit si les 3 cartes
/// ont été bannies (« If you do »).
fn decompose() -> Vec<Op> {
    vec![
        Op::Ask(|g, c| {
            decompose_possible(g, c.controller, &[]).then(|| Ask {
                player: c.controller,
                label:
                    "Decompose : bannir 2 cartes Earth et une carte d'action de votre cimetière ?",
                options: vec![Choice::Yes, Choice::No],
            })
        }),
        Op::Do(|_, c| {
            c.flag = c.answer.take() == Some(Choice::Yes);
            c.picked.clear();
        }),
        Op::If(|_, c| c.flag, 3),
        Op::Ask(|g, c| {
            let options: Vec<Choice> = g
                .player(c.controller)
                .graveyard
                .iter()
                .copied()
                .filter(|x| !c.picked.contains(x))
                .filter(|&x| {
                    let mut v = c.picked.clone();
                    v.push(x);
                    decompose_possible(g, c.controller, &v)
                })
                .map(Choice::Object)
                .collect();
            (!options.is_empty()).then_some(Ask {
                player: c.controller,
                label: "Decompose : carte à bannir de votre cimetière",
                options,
            })
        }),
        Op::Do(|_, c| match c.answer.take() {
            Some(Choice::Object(x)) => c.picked.push(x),
            _ => c.flag = false,
        }),
        Op::Back(|_, c| c.flag && c.picked.len() < 3, 2),
        Op::Do(|g, c| {
            c.flag = c.flag && c.picked.len() == 3;
            if c.flag {
                for x in c.picked.clone() {
                    if g.obj(x).zone == Zone::Graveyard {
                        g.banish(c.controller, x, false);
                    }
                }
            }
        }),
    ]
}

// ---------------------------------------------------------------------------
// Les cartes
// ---------------------------------------------------------------------------

pub fn add(db: &mut CardDb) {
    hero_and_arena(db);
    deck_cards(db);
    arcane_seeds_life(db);
    tokens(db);
}

fn hero_and_arena(db: &mut CardDb) {
    // Florian — Elemental Runeblade Hero - Young, 20 PV, intellect 4.
    // « If there are 4 or more Earth cards in your banished zone, Florian
    //   gets "If you would create 1 or more aura tokens, instead create that
    //   many plus 1 of each of those tokens."
    //   Essence of Earth »
    // Essence of Earth (CR 8.3.16) ne concerne que la construction du deck.
    // « You would create » : le joueur qui crée les tokens (`Pending::by`),
    // pas celui qui les contrôle (CR 8.5.2c, ex. un Frostbite créé chez
    // Florian par Iyslander n'est pas concerné).
    db.add(CardDef {
        name: "Florian".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: Supertypes::ELEMENTAL | RB,
        replacements: vec![ReplacementDef {
            kind: ReplaceKind::CreateToken,
            anywhere: false,
            optional: false,
            applies: |g, w, p| {
                p.by == w.controller
                    && four_earth_banished(g, w.controller)
                    && p.card.is_some_and(|c| {
                        let d = g.db.get(c);
                        d.types.has(Types::TOKEN) && d.subtypes.has(Subtypes::AURA)
                    })
            },
            apply: |_, _, p| p.amount += 1,
        }],
        ..Default::default()
    });

    // Nebula Blade — Runeblade Weapon - Sword (2H), 1{p}.
    // « Once per Turn Action -- {r}{r}: Attack
    //   When this hits, create a Runechant token.
    //   If you've played a non-attack action card this turn, this gets +3{p}. »
    db.add(CardDef {
        name: "Nebula Blade".into(),
        power: Some(1),
        types: Types::WEAPON,
        subtypes: Subtypes::SWORD | Subtypes::TWO_HAND,
        supertypes: RB,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 2,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        triggers: vec![TriggerDef {
            when: this_hits,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| create_runechants(g, c.controller, 1))],
        }],
        self_mod: Some(|g, me| {
            // Une carte double jouée d'un seul côté n'a que ce côté (CR 9.2.3).
            let p = g.obj(me).controller;
            if g.played_def_this_turn(p, |d| d.is_action() && !d.is_attack_card()) {
                power(3)
            } else {
                Delta::default()
            }
        }),
        ..Default::default()
    });

    // Scepter of Pain — Runeblade Weapon - Scepter (1H).
    // « Once per Turn Action -- {r}{r}: Deal 1 arcane damage to any opposing
    //   target. Create a Runechant token for each damage dealt this way. »
    db.add(CardDef {
        name: "Scepter of Pain".into(),
        types: Types::WEAPON,
        subtypes: Subtypes::SCEPTER | Subtypes::ONE_HAND,
        supertypes: RB,
        arcane_damage: true,
        activated: vec![ActivatedDef {
            label: "1 dégât arcanique à une cible adverse",
            speed: Speed::Action,
            cost: 2,
            once_per_turn: true,
            targets: vec![ANY_OPPOSING_TARGET],
            script: vec![
                Op::Do(|g, c| {
                    g.last_damage = 0;
                    if let Some(t) = c.target(0) {
                        g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                    }
                }),
                // `last_damage` est lu à l'instruction suivante (préventions).
                Op::Do(|g, c| {
                    let n = g.last_damage.max(0) as u8;
                    if n > 0 {
                        create_runechants(g, c.controller, n);
                    }
                }),
            ],
            ..Default::default()
        }],
        ..Default::default()
    });

    // Annals of Sutcliffe — Runeblade Weapon - Book (2H).
    // « Once per Turn Action -- {r}{r}{r}: Draw a card. If an attack action
    //   card and a non-attack action card were pitched this way, create a
    //   Runechant token. »
    // « Pitched this way » : les cartes pitchées pour payer ce coût, c.-à-d.
    // les événements `Pitched` depuis l'activation (`Ctx::log_start`). Le coût
    // d'effet (vide) sert à les lire juste après le paiement (CR 5.1.9).
    db.add(CardDef {
        name: "Annals of Sutcliffe".into(),
        types: Types::WEAPON,
        subtypes: Subtypes::BOOK | Subtypes::TWO_HAND,
        supertypes: RB,
        activated: vec![ActivatedDef {
            label: "Piocher une carte",
            speed: Speed::Action,
            cost: 3,
            once_per_turn: true,
            effect_cost: vec![Op::Do(|g, c| {
                let (mut attack, mut non_attack) = (false, false);
                for e in &g.turn_events()[c.log_start as usize..] {
                    if let Event::Pitched { player, card } = *e {
                        let d = g.def(card);
                        if player == c.controller && d.is_action() {
                            attack |= d.is_attack_card();
                            non_attack |= !d.is_attack_card();
                        }
                    }
                }
                c.flag = attack && non_attack;
            })],
            script: vec![Op::Do(|g, c| {
                g.draw(c.controller);
                if c.flag {
                    create_runechants(g, c.controller, 1);
                }
            })],
            ..Default::default()
        }],
        ..Default::default()
    });

    // Arcane Lantern — Generic Equipment - Off-Hand, pas de défense.
    // « Arcane Barrier 1 »
    let mut d = equipment("Arcane Lantern", G, Subtypes::OFF_HAND, 0);
    d.defense = None;
    d.arcane_barrier = 1;
    db.add(d);

    // Bloodied Oval — Generic Equipment - Off-Hand, déf. *.
    // « This card's {d} is equal to the number of opponents with greater {h}
    //   than you. Blade Break »
    let mut d = equipment("Bloodied Oval", G, Subtypes::OFF_HAND, 0);
    d.keywords = Keywords::BLADE_BREAK;
    d.self_mod = Some(|g, me| {
        let p = g.obj(me).owner;
        let n = PlayerId::BOTH
            .into_iter()
            .filter(|&o| o != p && g.player(o).life > g.player(p).life)
            .count();
        defense(n as i16)
    });
    db.add(d);

    // Crown of Dichotomy — Runeblade Equipment - Head, déf. 0.
    // « Action -- {r}, destroy this: Put target Runeblade attack action card
    //   and target Runeblade non-attack action card from your graveyard on top
    //   of your deck in any order.
    //   Arcane Barrier 1 »
    let mut d = equipment("Crown of Dichotomy", RB, Subtypes::HEAD, 0);
    d.arcane_barrier = 1;
    d.activated = vec![ActivatedDef {
        label: "Détruire : 2 cartes Runeblade du cimetière sur le deck",
        speed: Speed::Action,
        cost: 1,
        targets: vec![
            TargetSpec {
                label: "carte d'attaque Runeblade de votre cimetière",
                filter: |g, c, t| match t {
                    Target::Object(r) => {
                        in_your_graveyard(g, c, r.id)
                            && g.supertypes(r.id).has(RB)
                            && is_attack_action_card(g, r.id)
                    }
                    Target::Player(_) => false,
                },
                optional: false,
                in_graveyard: true,
                in_banished: false,
            },
            TargetSpec {
                label: "carte d'action non-attaque Runeblade de votre cimetière",
                filter: |g, c, t| match t {
                    Target::Object(r) => {
                        let d = g.def(r.id);
                        in_your_graveyard(g, c, r.id)
                            && g.supertypes(r.id).has(RB)
                            && d.is_action()
                            && !d.is_attack_card()
                    }
                    Target::Player(_) => false,
                },
                optional: false,
                in_graveyard: true,
                in_banished: false,
            },
        ],
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![
            // « in any order » : le joueur choisit la carte qui finit sur le dessus.
            Op::Ask(|_, c| {
                let (Some(a), Some(b)) = (target_obj(c, 0), target_obj(c, 1)) else {
                    return None;
                };
                Some(Ask {
                    player: c.controller,
                    label: "Carte à mettre sur le dessus du deck (l'autre juste en dessous)",
                    options: vec![Choice::Object(a.id), Choice::Object(b.id)],
                })
            }),
            Op::Do(|g, c| {
                let mut cards: Vec<ObjectId> = (0..2)
                    .filter_map(|i| target_obj(c, i))
                    .map(|r| r.id)
                    .collect();
                if let Some(Choice::Object(top)) = c.answer {
                    // La carte du dessus est mise en dernier.
                    cards.sort_by_key(|&x| x == top);
                }
                for x in cards {
                    if g.obj(x).zone == Zone::Graveyard {
                        g.put_on_top(x);
                    }
                }
            }),
        ],
        ..Default::default()
    }];
    db.add(d);

    // Helm of Lignum Vitae — Earth Equipment - Head, déf. 1.
    // « If there are 4 or more Earth cards in your banished zone, this gets
    //   +1{d}. Blade Break »
    let mut d = equipment("Helm of Lignum Vitae", EARTH, Subtypes::HEAD, 1);
    d.keywords = Keywords::BLADE_BREAK;
    d.self_mod = Some(|g, me| {
        if four_earth_banished(g, g.obj(me).owner) {
            defense(1)
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Plume of Evergrowth — Earth Equipment - Head, déf. 0.
    // « Instant -- {r}{r}{r}, destroy this: Return target Earth action card or
    //   Earth instant card from your graveyard to your hand. »
    let mut d = equipment("Plume of Evergrowth", EARTH, Subtypes::HEAD, 0);
    d.activated = vec![ActivatedDef {
        label: "Détruire : une carte Earth du cimetière en main",
        speed: Speed::Instant,
        cost: 3,
        targets: vec![TargetSpec {
            label: "carte d'action ou instant Earth de votre cimetière",
            filter: |g, c, t| match t {
                Target::Object(r) => {
                    in_your_graveyard(g, c, r.id)
                        && is_earth(g, r.id)
                        && g.def(r.id).types.any(Types::ACTION | Types::INSTANT)
                }
                Target::Player(_) => false,
            },
            optional: false,
            in_graveyard: true,
            in_banished: false,
        }],
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                if g.obj(r.id).zone == Zone::Graveyard {
                    g.return_to_hand(r.id);
                }
            }
        })],
        ..Default::default()
    }];
    db.add(d);

    // Blossom of Spring — Generic Equipment - Chest, déf. 0.
    // « Action -- Destroy this: Gain {r}. Go again »
    let mut d = equipment("Blossom of Spring", G, Subtypes::CHEST, 0);
    d.activated = vec![ActivatedDef {
        label: "Détruire : gagner {r}",
        speed: Speed::Action,
        go_again: true,
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(d);

    // Unyielding Grip — Generic Equipment - Arms, déf. 0.
    // « If you have no cards in your hand, this gets +3{d}. Blade Break »
    let mut d = equipment("Unyielding Grip", G, Subtypes::ARMS, 0);
    d.keywords = Keywords::BLADE_BREAK;
    d.self_mod = Some(|g, me| {
        if g.player(g.obj(me).owner).hand.is_empty() {
            defense(3)
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Well Grounded — Earth Equipment - Legs, déf. 0.
    // « Instant -- Destroy this: Prevent the next 2 damage that would be dealt
    //   to you this turn. Activate this only if there are 4 or more Earth
    //   cards in your banished zone. »
    let mut d = equipment("Well Grounded", EARTH, Subtypes::LEGS, 0);
    d.activated = vec![ActivatedDef {
        label: "Détruire : prévenir les 2 prochains dégâts",
        speed: Speed::Instant,
        condition: Some(|g, p, _| four_earth_banished(g, p)),
        effect_cost: vec![Op::Do(|g, c| destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.prevent_next(c.controller, 2, false, None))],
        ..Default::default()
    }];
    db.add(d);
}

fn deck_cards(db: &mut CardDb) {
    // Amplify the Arknight — Runeblade Action - Attack, 6/5/4, coût 3, déf. 3.
    // « This costs {r} less to play for each Runechant you control. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Amplify the Arknight",
            c,
            RB,
            Types::ACTION,
            Some(3),
            Some(6 - i as u8),
            Some(3),
        );
        d.cost_mod = Some(less_per_runechant);
        db.add(d);
    }

    // Autumn's Touch — Earth Action - Attack, 7/6/5, coût 3, déf. 3. (Sans texte.)
    for (i, c) in COLORS.into_iter().enumerate() {
        db.add(deck_card(
            "Autumn's Touch",
            c,
            EARTH,
            Types::ACTION,
            Some(3),
            Some(7 - i as u8),
            Some(3),
        ));
    }

    // Chorus of Rotwood (rouge) — Earth Runeblade Action, coût 1, déf. 2.
    // « Decompose - You may banish 2 Earth cards and an action card from your
    //   graveyard. If you do, create an Embodiment of Earth token.
    //   Create 3 Runechant tokens. »
    let mut d = deck_card(
        "Chorus of Rotwood",
        Color::Red,
        EARTH_RB,
        Types::ACTION,
        Some(1),
        None,
        Some(2),
    );
    d.resolve = decompose();
    d.resolve.push(Op::Do(|g, c| {
        if c.flag {
            let e = card_named(g, EMBODIMENT);
            g.create_tokens(c.controller, e, 1);
        }
    }));
    d.resolve
        .push(Op::Do(|g, c| create_runechants(g, c.controller, 3)));
    db.add(d);

    // Colors of Aria (rouge) — Elemental Action - Attack, 7, coût 3, déf. 3.
    // « While this is face-up in any zone, it's Earth, Ice, and Lightning. »
    let mut d = deck_card(
        "Colors of Aria",
        Color::Red,
        Supertypes::ELEMENTAL,
        Types::ACTION,
        Some(3),
        Some(7),
        Some(3),
    );
    d.face_up_supertypes = Supertypes::EARTH | Supertypes::ICE | Supertypes::LIGHTNING;
    db.add(d);

    // Fruits of the Forest — Earth Action - Attack, 7/6/5, coût 3, déf. 2.
    // « Instant -- Discard this: Gain 2{h} »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Fruits of the Forest",
            c,
            EARTH,
            Types::ACTION,
            Some(3),
            Some(7 - i as u8),
            Some(2),
        );
        d.activated = vec![ActivatedDef {
            label: "Défausser : gagner 2 PV",
            speed: Speed::Instant,
            from_hand: true,
            effect_cost: vec![Op::Do(|g, c| {
                if let Some(s) = c.source_id().filter(|&s| g.obj(s).zone == Zone::Hand) {
                    g.discard(s);
                }
            })],
            script: vec![Op::Do(|g, c| g.gain_life(c.controller, 2))],
            ..Default::default()
        }];
        db.add(d);
    }

    // Harvest Season — Earth Action - Aura, coût 1, déf. 2.
    // « Go again
    //   At the beginning of your action phase, destroy this, then gain 3{h}
    //   (jaune : 2{h} ; bleu : 1{h}). »
    for c in COLORS {
        let mut d = deck_card(
            "Harvest Season",
            c,
            EARTH,
            Types::ACTION,
            Some(1),
            None,
            Some(2),
        );
        d.subtypes = Subtypes::AURA;
        d.keywords = Keywords::GO_AGAIN;
        d.triggers = vec![TriggerDef {
            when: beginning_of_your_action_phase,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                destroy_source(g, c);
                let n = by_color(g, c, [3, 2, 1]) as u16;
                g.gain_life(c.controller, n);
            })],
        }];
        db.add(d);
    }

    // Oath of the Arknight — Runeblade Action, coût 2, déf. 3.
    // « Your next Runeblade attack this turn gets +3{p} (jaune : +2 ; bleu : +1).
    //   Create a Runechant token.
    //   Go again »
    for c in COLORS {
        let mut d = deck_card(
            "Oath of the Arknight",
            c,
            RB,
            Types::ACTION,
            Some(2),
            None,
            Some(3),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 1]);
            g.next_attack(
                c.controller,
                Some(|g, a| g.supertypes(a).has(Supertypes::RUNEBLADE)),
                power(n),
                false,
            );
            create_runechants(g, c.controller, 1);
        })];
        db.add(d);
    }

    // Read the Runes — Runeblade Action, coût 0, déf. 2.
    // « Create 3 Runechant tokens. » (jaune : 2 ; bleu : « a Runechant token »)
    for c in COLORS {
        let mut d = deck_card(
            "Read the Runes",
            c,
            RB,
            Types::ACTION,
            Some(0),
            None,
            Some(2),
        );
        d.resolve = vec![Op::Do(|g, c| {
            let n = by_color(g, c, [3, 2, 1]) as u8;
            create_runechants(g, c.controller, n);
        })];
        db.add(d);
    }

    // Reduce to Runechant — Runeblade Defense Reaction, coût 1, déf. 4/3/2.
    // « This costs {r} less to play for each Runechant you control.
    //   Create a Runechant token. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Reduce to Runechant",
            c,
            RB,
            Types::DEFENSE_REACTION,
            Some(1),
            None,
            Some(4 - i as u8),
        );
        d.cost_mod = Some(less_per_runechant);
        d.resolve = vec![Op::Do(|g, c| create_runechants(g, c.controller, 1))];
        db.add(d);
    }

    // Rootbound Carapace — Earth Defense Reaction, coût 0, déf. 3/2/1.
    // « Decompose - You may banish 2 Earth cards and an action card from your
    //   graveyard. If you do, this gets +1{d}. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Rootbound Carapace",
            c,
            EARTH,
            Types::DEFENSE_REACTION,
            Some(0),
            None,
            Some(3 - i as u8),
        );
        d.resolve = decompose();
        d.resolve.push(Op::Do(|g, c| {
            if let Some(s) = c.source.filter(|&s| c.flag && g.is_current(s)) {
                // Pas de durée écrite : tant que la carte existe (elle
                // devient un nouvel objet en quittant la chaîne, CR 3.0.9).
                g.modify(s, defense(1), Until::EndOfTurn);
            }
        }));
        db.add(d);
    }

    // Runeblood Incantation — Runeblade Action - Aura, coût 1, déf. 2.
    // « Go again
    //   This enters the arena with 3 verse counters (jaune : 2 ; bleu : a).
    //   At the beginning of your action phase, remove a verse counter from
    //   this. If you do, create a Runechant token. Otherwise, destroy this. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Runeblood Incantation",
            c,
            RB,
            Types::ACTION,
            Some(1),
            None,
            Some(2),
        );
        d.subtypes = Subtypes::AURA;
        d.keywords = Keywords::GO_AGAIN;
        d.enter_counters = 3 - i as u8;
        d.triggers = vec![TriggerDef {
            when: beginning_of_your_action_phase,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let Some(s) = c.source.filter(|&s| g.is_current(s)) else {
                    return;
                };
                if g.obj(s.id).counters > 0 {
                    g.obj_mut(s.id).counters -= 1;
                    create_runechants(g, c.controller, 1);
                } else {
                    g.destroy(s.id);
                }
            })],
        }];
        db.add(d);
    }

    // Sigil of Silphidae (bleu) — Runeblade Action - Aura, coût 0, déf. 3.
    // « Go again
    //   When this enters or leaves the arena, you may banish another aura from
    //   your graveyard. If you do, deal 1 arcane damage to target hero.
    //   At the beginning of your action phase, destroy this. »
    // La cible est déclarée quand la couche déclenchée est mise sur la pile
    // (CR 6.6.6a) ; le choix de bannir se fait à la résolution.
    let mut d = deck_card(
        "Sigil of Silphidae",
        Color::Blue,
        RB,
        Types::ACTION,
        Some(0),
        None,
        Some(3),
    );
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::GO_AGAIN;
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
            script: vec![
                Op::Ask(|g, c| {
                    let me = c.source_id();
                    let options: Vec<Choice> = g
                        .player(c.controller)
                        .graveyard
                        .iter()
                        .copied()
                        .filter(|&x| Some(x) != me && g.def(x).subtypes.has(Subtypes::AURA))
                        .map(Choice::Object)
                        .chain(std::iter::once(Choice::Pass))
                        .collect();
                    (options.len() > 1).then_some(Ask {
                        player: c.controller,
                        label: "Bannir une autre aura de votre cimetière (1 dégât arcanique) ?",
                        options,
                    })
                }),
                Op::Do(|g, c| {
                    g.last_damage = 0;
                    if let Some(Choice::Object(x)) = c.answer.take() {
                        if g.obj(x).zone == Zone::Graveyard {
                            g.banish(c.controller, x, false);
                            if let Some(t) = c.target(0) {
                                g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                            }
                        }
                    }
                }),
            ],
        },
        TriggerDef {
            when: beginning_of_your_action_phase,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| destroy_source(g, c))],
        },
    ];
    db.add(d);
}

/// « At the beginning of your action phase ».
fn beginning_of_your_action_phase(g: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(*e, Event::BeginActionPhase(p) if p == g.obj(me).controller)
}

/// Arcane Seeds // Life (rouge) — carte double (split-card, CR 9.2) avec meld.
/// Côté gauche : Arcane Seeds — Runeblade Action, coût 0.
///   « Meld. Create a Runechant token. Create a Runechant token. Go again »
/// Côté droit : Life — Earth Instant, coût 0.
///   « Meld. Gain 1{h} »
/// Pitch 1, déf. 3. Meld (CR 8.3.38) : « You may pay twice the base cost of
/// this to play both halves of this. » La carte fusionnée a les propriétés des
/// deux côtés (CR 8.3.38c) et se résout d'abord avec Life, puis avec Arcane
/// Seeds (CR 5.3.4d).
fn arcane_seeds_life(db: &mut CardDb) {
    let base = db.len() as u16;
    let split = Split {
        whole: CardId(base),
        left: CardId(base + 1),
        right: CardId(base + 2),
        melded: Some(CardId(base + 3)),
    };
    // Deux créations distinctes (deux phrases) : avec Florian, chacune est
    // remplacée séparément.
    let seeds: Vec<Op> = vec![
        Op::Do(|g, c| create_runechants(g, c.controller, 1)),
        Op::Do(|g, c| create_runechants(g, c.controller, 1)),
    ];
    let life: Vec<Op> = vec![Op::Do(|g, c| g.gain_life(c.controller, 1))];
    let side = |name: &str, types: Types, supertypes: Supertypes| CardDef {
        name: name.into(),
        color: Some(Color::Red),
        pitch: Some(1),
        cost: Some(0),
        defense: Some(3),
        types,
        supertypes,
        split: Some(split),
        ..Default::default()
    };
    // La carte entière (main, deck, cimetière…) : les deux côtés (CR 9.2.2).
    let mut whole = side(
        "Arcane Seeds // Life",
        Types::ACTION | Types::INSTANT,
        EARTH_RB,
    );
    whole.keywords = Keywords::GO_AGAIN;
    // Libellés des façons de la jouer (interface, CR 5.1.2c).
    whole.modes = vec!["Arcane Seeds", "Life", "Meld : les deux côtés"];
    assert_eq!(db.add(whole), split.whole);
    let mut left = side("Arcane Seeds", Types::ACTION, RB);
    left.keywords = Keywords::GO_AGAIN;
    left.resolve = seeds;
    assert_eq!(db.add(left), split.left);
    let mut right = side("Life", Types::INSTANT, EARTH);
    right.resolve = life.clone();
    assert_eq!(db.add(right), split.right);
    // Fusionnée : coût = deux fois le coût de base (0), CR 8.3.38b. Première
    // résolution : le côté droit (Life).
    let mut melded = side(
        "Arcane Seeds // Life",
        Types::ACTION | Types::INSTANT,
        EARTH_RB,
    );
    melded.keywords = Keywords::GO_AGAIN;
    melded.resolve = life;
    assert_eq!(db.add(melded), split.melded.unwrap());
}

/// Tokens Runechant et Embodiment of Earth.
fn tokens(db: &mut CardDb) {
    // Runechant — Runeblade Token - Aura.
    // « When you play an attack action card or activate a weapon attack,
    //   destroy this and deal 1 arcane damage to target opposing hero. »
    db.add(CardDef {
        name: RUNECHANT.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: RB,
        arcane_damage: true,
        triggers: vec![TriggerDef {
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
                        let d = g.def(source);
                        player == me
                            && d.types.has(Types::WEAPON)
                            && d.activated[ability as usize].attack
                    }
                    _ => false,
                }
            },
            anywhere: false,
            targets: vec![TARGET_OPPOSING_HERO],
            script: vec![Op::Do(|g, c| {
                destroy_source(g, c);
                if let Some(t) = c.target(0) {
                    g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                }
            })],
        }],
        ..Default::default()
    });

    // Embodiment of Earth — Elemental Token - Aura.
    // « Non-attack action cards you control get +1{d} while defending.
    //   At the beginning of your action phase, destroy this. »
    db.add(CardDef {
        name: EMBODIMENT.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: Supertypes::ELEMENTAL,
        aura: Some(|g, src, x| {
            let d = g.def(x);
            if g.obj(x).controller == g.obj(src).controller
                && d.is_action()
                && !d.is_attack_card()
                && is_defending(g, x)
            {
                defense(1)
            } else {
                Delta::default()
            }
        }),
        triggers: vec![TriggerDef {
            when: beginning_of_your_action_phase,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| destroy_source(g, c))],
        }],
        ..Default::default()
    });
}
