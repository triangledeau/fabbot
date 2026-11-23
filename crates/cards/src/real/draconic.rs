//! Cartes Draconic et Illusionist du deck Dromai, ses dragons (faces
//! arrière des invocations) et les tokens Ash, Aether Ashwing et Spectral
//! Shield.
//!
//! - **Ash** (CR 8.6.14) : « Material - While this is under a permanent,
//!   that object has phantasm » (`CardDef::material`).
//! - **Transform** (CR 8.5.36) : `GameState::transform_into_token` (une ash
//!   devient une Aether Ashwing, l'ash passe dessous) et
//!   `GameState::transform_into_card` (Invoke X : la carte devient un
//!   permanent face arrière active, CR 8.2.13a, 8.5.36b, puis l'ash passe
//!   dessous).
//! - **Phantasm** (CR 8.3.13) : géré par le moteur (`RuleTrigger::Phantasm`).
//! - Les dragons n'ont pas de capacité d'attaque imprimée : Storm of Sandikai
//!   leur donne « Once per Turn Action -- 0: Attack » (`CardDef::grants_attack`).
//!   La capacité est écrite dans la définition de chaque dragon, avec la
//!   condition `GameState::attack_granted`.

use super::{card_named, deck_card, grant, target_obj};
use fab_engine::card::{
    CardDb, CardDef, Color, Delta, Flip, Keywords, Subtypes, Supertypes, Types,
};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, CardId, Choice, Event, GameState, ObjectId, PlayerId, ReplaceKind,
    ReplacementDef, Speed, TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Dromai",
    "Storm of Sandikai",
    "Silken Shroud",
    "Smoldering Scales",
    "Wave of Reality",
    "Billowing Mirage",
    "Blaze Headlong",
    "Dunebreaker Cenipai",
    "Dustup",
    "Essence of Ancestry: Body",
    "Invoke Azvolai",
    "Invoke Cromai",
    "Invoke Kyloria",
    "Invoke Miragai",
    "Invoke Nekria",
    "Invoke Ouvia",
    "Invoke Themai",
    "Invoke Yendurai",
    "Passing Mirage",
    "Rake the Embers",
    "Sand Cover",
    "Sweeping Blow",
    "Ash",
    "Aether Ashwing",
    "Spectral Shield",
];

const DI: Supertypes = Supertypes(Supertypes::DRACONIC.0 | Supertypes::ILLUSIONIST.0);
const ILL: Supertypes = Supertypes::ILLUSIONIST;
const ASH: &str = "Ash";
const ASHWING: &str = "Aether Ashwing";
const R: Color = Color::Red;

// ----------------------------------------------------------------------
// Aides
// ----------------------------------------------------------------------

/// « When this attacks ».
fn when_this_attacks(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Attack { attack, .. } if *attack == me)
}

/// « When this is destroyed » : l'objet est alors hors de l'arène (capacité
/// `anywhere`).
fn when_this_destroyed(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Destroyed { card, .. } if *card == me)
}

/// Crée `n` tokens Ash pour `p`.
pub(crate) fn create_ash(g: &mut GameState, p: PlayerId, n: u8) {
    let ash = card_named(g, ASH);
    g.create_tokens(p, ash, n);
}

/// Les ash que `p` contrôle dans l'arène (pas celles qui sont sous un
/// permanent, CR 3.0.14b).
pub(crate) fn ash_you_control(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    g.permanents
        .iter()
        .copied()
        .filter(|&o| g.obj(o).controller == p && g.def(o).subtypes.has(Subtypes::ASH))
        .collect()
}

/// « Transform up to 1 ash you control into an Aether Ashwing » : la
/// question (quelle ash, ou aucune). Ce n'est pas une cible : le choix se
/// fait à la résolution.
fn ask_ash_to_ashwing(g: &GameState, c: &Ctx) -> Option<Ask> {
    let ash = ash_you_control(g, c.controller);
    (!ash.is_empty()).then(|| Ask {
        player: c.controller,
        label: "Transformer une Ash en Aether Ashwing ?",
        options: ash
            .into_iter()
            .map(Choice::Object)
            .chain(std::iter::once(Choice::Pass))
            .collect(),
    })
}

/// Réponse à [`ask_ash_to_ashwing`] : transforme l'ash choisie. Renvoie
/// `true` si une ash a été transformée.
fn ash_to_ashwing(g: &mut GameState, c: &mut Ctx) -> bool {
    match c.answer.take() {
        Some(Choice::Object(a)) if g.obj(a).zone == Zone::Permanent => {
            let wing = card_named(g, ASHWING);
            g.transform_into_token(c.controller, &[a], wing).is_some()
        }
        _ => false,
    }
}

/// Script « transform up to 1 ash you control into an Aether Ashwing ».
fn up_to_one_ashwing() -> Vec<Op> {
    vec![
        Op::Ask(ask_ash_to_ashwing),
        Op::Do(|g, c| {
            ash_to_ashwing(g, c);
        }),
    ]
}

/// « Target ash you control » (Invoke X, Sand Cover).
const TARGET_ASH_YOU_CONTROL: TargetSpec = TargetSpec {
    label: "ash que vous contrôlez",
    filter: |g, c, t| match t {
        Target::Object(r) => {
            let o = g.obj(r.id);
            o.zone == Zone::Permanent
                && o.controller == c.controller
                && g.def(r.id).subtypes.has(Subtypes::ASH)
        }
        Target::Player(_) => false,
    },
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// La capacité que Storm of Sandikai donne aux dragons : « Once per Turn
/// Action -- 0: Attack ».
fn granted_attack() -> ActivatedDef {
    ActivatedDef {
        label: "Attaquer (Storm of Sandikai)",
        speed: Speed::Action,
        once_per_turn: true,
        attack: true,
        condition: Some(|g, _, me| g.attack_granted(me)),
        ..Default::default()
    }
}

/// Un objet qui est ou a été un dragon : un dragon, ou une carte
/// d'invocation revenue face avant (CR 9.1.3b) après avoir été un dragon
/// (seule sa face arrière peut attaquer).
fn was_dragon(g: &GameState, id: ObjectId) -> bool {
    let d = g.def(id);
    d.subtypes.has(Subtypes::DRAGON)
        || d.flip
            .is_some_and(|f| g.db.get(f.back).subtypes.has(Subtypes::DRAGON))
}

/// L'objet `id` (une carte d'attaque ou un allié qui attaque) est-il la
/// première attaque de `p` ce tour qui vérifie `f` ? (« Your first dragon /
/// Illusionist attack each turn ».) L'événement `Attack` désigne la carte
/// ou l'allié qui attaque.
fn is_first_attack(
    g: &GameState,
    p: PlayerId,
    id: ObjectId,
    f: fn(&GameState, ObjectId) -> bool,
) -> bool {
    if g.obj(id).zone != Zone::CombatChain || g.obj(id).proxy_of.is_some() {
        return false;
    }
    g.turn_events().iter().find_map(|e| match *e {
        Event::Attack {
            attack, attacker, ..
        } if attacker == p && f(g, attack) => Some(attack),
        _ => None,
    }) == Some(id)
}

fn loses_phantasm() -> Delta {
    Delta {
        loses: Keywords::PHANTASM,
        ..Default::default()
    }
}

/// « If you've played another red card this turn » (Blaze Headlong) : la
/// couleur de la carte telle qu'elle a été jouée.
fn played_another_red(g: &GameState, p: PlayerId, me: ObjectId) -> bool {
    g.turn_events().iter().any(|e| match *e {
        Event::Played {
            player, card, def, ..
        } => player == p && card != me && g.db.get(def).color == Some(R),
        _ => false,
    })
}

/// Un dragon (face arrière d'une invocation) : allié Draconic Illusionist.
fn dragon(name: &str, power: u8, life: u8) -> CardDef {
    CardDef {
        name: name.into(),
        power: Some(power),
        life: Some(life),
        subtypes: Subtypes::DRAGON | Subtypes::ALLY,
        supertypes: DI,
        activated: vec![granted_attack()],
        ..Default::default()
    }
}

/// Invoke X (rouge) — Draconic Illusionist Action - Invocation, déf. 3.
/// « **Transform** target ash you control into X. **Go again** »
/// Ajoute la face arrière puis la face avant, reliées par `CardDef::flip`.
fn add_invocation(db: &mut CardDb, front_name: &str, cost: u8, mut back: CardDef) {
    let back_id = CardId(db.len() as u16);
    let front_id = CardId(db.len() as u16 + 1);
    let flip = Flip {
        front: front_id,
        back: back_id,
    };
    back.flip = Some(flip);
    db.add(back);
    let mut d = deck_card(front_name, R, DI, Types::ACTION, Some(cost), None, Some(3));
    d.subtypes = Subtypes::INVOCATION;
    d.keywords = Keywords::GO_AGAIN;
    d.flip = Some(flip);
    d.targets = vec![TARGET_ASH_YOU_CONTROL];
    d.resolve = vec![Op::Do(|g, c| {
        // CR 8.5.36b : la carte devient d'abord un permanent (face arrière,
        // CR 8.2.13a), puis l'ash passe dessous. Sans ash légale, la carte
        // entre quand même dans l'arène (CR 5.3.6a, 8.2.13a).
        if let (Some(a), Some(me)) = (target_obj(c, 0), c.source) {
            if g.is_current(me) && g.obj(me.id).zone == Zone::Stack {
                g.transform_into_card(c.controller, &[a.id], me.id);
            }
        }
    })];
    let id = db.add(d);
    debug_assert_eq!(id, front_id);
}

pub fn add(db: &mut CardDb) {
    // Dromai — Draconic Illusionist Hero - Young, 20{h}, intellect 4.
    // « Whenever you pitch a red card, create an Ash token.
    //   If you've played a red card this turn, your dragon attacks get go again. »
    db.add(CardDef {
        name: "Dromai".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: DI,
        triggers: vec![TriggerDef {
            when: |g, me, e| match *e {
                Event::Pitched { player, card } => {
                    player == g.obj(me).controller && g.def(card).color == Some(R)
                }
                _ => false,
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| create_ash(g, c.controller, 1))],
        }],
        aura: Some(|g, me, id| {
            // « Your dragon attacks » : une attaque-proxy d'un dragon.
            let o = g.obj(id);
            let p = g.obj(me).controller;
            let is_dragon_attack = o.proxy_of.is_some_and(|s| {
                g.def(s).subtypes.has(Subtypes::DRAGON) && g.obj(s).controller == p
            });
            if is_dragon_attack && g.played_def_this_turn(p, |d| d.color == Some(R)) {
                grant(Keywords::GO_AGAIN)
            } else {
                Delta::default()
            }
        }),
        ..Default::default()
    });

    // Ash — Draconic Illusionist Token - Ash.
    // « Material - While this is under a permanent, that permanent gets
    //   phantasm. » (CR 8.6.14)
    db.add(CardDef {
        name: ASH.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::ASH,
        supertypes: DI,
        material: Keywords::PHANTASM,
        ..Default::default()
    });

    // Aether Ashwing — Draconic Illusionist Token - Dragon Ally, 1{p} 1{h}.
    // « Arcane Barrier 1 » (CR 8.6.15). Pas de capacité d'attaque imprimée.
    // « You » (arcane barrier) est le contrôleur du token (CR 1.1.2b).
    db.add(CardDef {
        name: ASHWING.into(),
        power: Some(1),
        life: Some(1),
        types: Types::TOKEN,
        subtypes: Subtypes::DRAGON | Subtypes::ALLY,
        supertypes: DI,
        arcane_barrier: 1,
        activated: vec![granted_attack()],
        ..Default::default()
    });

    // Spectral Shield — Illusionist Token - Aura.
    // « Ward 1 » (CR 8.6.8)
    db.add(CardDef {
        name: "Spectral Shield".into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: ILL,
        ward: 1,
        ..Default::default()
    });

    // Storm of Sandikai — Draconic Illusionist Weapon - Scepter (2H).
    // « Dragon allies you control get "Once per Turn Action -- 0: Attack". »
    db.add(CardDef {
        name: "Storm of Sandikai".into(),
        types: Types::WEAPON,
        subtypes: Subtypes::SCEPTER | Subtypes::TWO_HAND,
        supertypes: DI,
        grants_attack: Some(|g, me, ally| {
            let d = g.def(ally);
            d.subtypes.has(Subtypes::DRAGON)
                && d.subtypes.has(Subtypes::ALLY)
                && g.obj(ally).controller == g.obj(me).controller
        }),
        ..Default::default()
    });

    // Silken Shroud — Illusionist Equipment - Head (pas de défense).
    // « When this is destroyed, create a Ponder token.
    //   Ward 1 »
    db.add(CardDef {
        name: "Silken Shroud".into(),
        types: Types::EQUIPMENT,
        subtypes: Subtypes::HEAD,
        supertypes: ILL,
        ward: 1,
        triggers: vec![TriggerDef {
            when: when_this_destroyed,
            anywhere: true,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let ponder = card_named(g, "Ponder");
                g.create_tokens(c.controller, ponder, 1);
            })],
        }],
        ..Default::default()
    });

    // Wave of Reality — Illusionist Equipment - Arms (pas de défense).
    // « When this is destroyed, create a Spectral Shield token.
    //   Ward 1 »
    db.add(CardDef {
        name: "Wave of Reality".into(),
        types: Types::EQUIPMENT,
        subtypes: Subtypes::ARMS,
        supertypes: ILL,
        ward: 1,
        triggers: vec![TriggerDef {
            when: when_this_destroyed,
            anywhere: true,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let s = card_named(g, "Spectral Shield");
                g.create_tokens(c.controller, s, 1);
            })],
        }],
        ..Default::default()
    });

    // Smoldering Scales — Draconic Equipment - Chest, déf. 2.
    // « If one or more Frostbite tokens would be created under your control,
    //   instead you may destroy this.
    //   Guardwell »
    db.add(CardDef {
        name: "Smoldering Scales".into(),
        defense: Some(2),
        types: Types::EQUIPMENT,
        subtypes: Subtypes::CHEST,
        supertypes: Supertypes::DRACONIC,
        keywords: Keywords::GUARDWELL,
        replacements: vec![ReplacementDef {
            kind: ReplaceKind::CreateToken,
            anywhere: false,
            optional: true,
            applies: |g, w, p| {
                p.player == w.controller && p.card.is_some_and(|c| g.db.get(c).name == "Frostbite")
            },
            apply: |g, w, p| {
                // « Instead you may destroy this » : aucun token n'est créé.
                p.amount = 0;
                if let Some(me) = w.me {
                    g.destroy(me);
                }
            },
        }],
        ..Default::default()
    });

    // Billowing Mirage (rouge) — Draconic Illusionist Action - Attack, coût 1, 3{p} 3{d}.
    // « When this attacks, transform up to 1 ash you control into an Aether
    //   Ashwing.
    //   Go again »
    let mut d = deck_card(
        "Billowing Mirage",
        R,
        DI,
        Types::ACTION,
        Some(1),
        Some(3),
        Some(3),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![],
        script: up_to_one_ashwing(),
    }];
    db.add(d);

    // Blaze Headlong (rouge) — Draconic Action - Attack, coût 0, 4{p} 2{d}.
    // « If you've played another red card this turn, this gets go again. »
    let mut d = deck_card(
        "Blaze Headlong",
        R,
        Supertypes::DRACONIC,
        Types::ACTION,
        Some(0),
        Some(4),
        Some(2),
    );
    d.self_mod = Some(|g, me| {
        if played_another_red(g, g.obj(me).controller, me) {
            grant(Keywords::GO_AGAIN)
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Dunebreaker Cenipai (rouge) — Draconic Illusionist Action - Attack, coût 1, 5{p} 3{d}.
    // « Phantasm
    //   When this is destroyed, create an Ash token.
    //   Go again »
    let mut d = deck_card(
        "Dunebreaker Cenipai",
        R,
        DI,
        Types::ACTION,
        Some(1),
        Some(5),
        Some(3),
    );
    d.keywords = Keywords::PHANTASM | Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: when_this_destroyed,
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| create_ash(g, c.controller, 1))],
    }];
    db.add(d);

    // Dustup (rouge) — Draconic Illusionist Action - Attack, coût 0, 4{p} 3{d}.
    // « When this hits, create an Ash token, then transform up to 1 ash you
    //   control into an Aether Ashwing. »
    let mut d = deck_card("Dustup", R, DI, Types::ACTION, Some(0), Some(4), Some(3));
    d.triggers = vec![TriggerDef {
        when: super::generic::this_hits,
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Do(|g, c| create_ash(g, c.controller, 1)),
            Op::Ask(ask_ash_to_ashwing),
            Op::Do(|g, c| {
                ash_to_ashwing(g, c);
            }),
        ],
    }];
    db.add(d);

    // Essence of Ancestry: Body (rouge) — Illusionist Action - Aura, coût 0, déf. 3.
    // « When this leaves the arena, if you control no Illusionist auras, the
    //   next time you would be dealt damage by a red source this turn,
    //   prevent it.
    //   Ward 2 »
    // La condition (« if you control no Illusionist auras ») est vérifiée
    // quand la capacité se déclenche et quand elle se résout.
    let mut d = deck_card(
        "Essence of Ancestry: Body",
        R,
        ILL,
        Types::ACTION,
        Some(0),
        None,
        Some(3),
    );
    d.subtypes = Subtypes::AURA;
    d.ward = 2;
    d.triggers = vec![TriggerDef {
        when: |g, me, e| match *e {
            Event::LeaveArena { card, .. } => {
                card == me && !controls_illusionist_aura(g, g.obj(me).owner)
            }
            _ => false,
        },
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if controls_illusionist_aura(g, c.controller) {
                return;
            }
            g.add_prevention(fab_engine::Prevention {
                id: 0,
                player: c.controller,
                remaining: None,
                arcane_only: false,
                source: None,
                source_filter: Some(|g, s| g.def(s).color == Some(R)),
                fixed: true,
                on_prevent: None,
                until: Until::EndOfTurn,
            });
        })],
    }];
    db.add(d);

    // Passing Mirage (bleu) — Illusionist Action - Aura, coût 0, déf. 2.
    // « Your first Illusionist attack each turn loses and can't gain phantasm.
    //   Spectra »
    let mut d = deck_card(
        "Passing Mirage",
        Color::Blue,
        ILL,
        Types::ACTION,
        Some(0),
        None,
        Some(2),
    );
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::SPECTRA;
    d.aura = Some(|g, me, id| {
        let p = g.obj(me).controller;
        if g.obj(id).controller == p
            && is_first_attack(g, p, id, |g, a| {
                g.supertypes(a).has(Supertypes::ILLUSIONIST)
            })
        {
            loses_phantasm()
        } else {
            Delta::default()
        }
    });
    db.add(d);

    // Rake the Embers (rouge) — Draconic Illusionist Action, coût 1, déf. 2.
    // « Create an Ash token, then transform up to 3 ash you control into
    //   Aether Ashwings.
    //   Go again »
    let mut d = deck_card(
        "Rake the Embers",
        R,
        DI,
        Types::ACTION,
        Some(1),
        None,
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![
        Op::Do(|g, c| {
            create_ash(g, c.controller, 1);
            c.num = 3;
        }),
        Op::Ask(ask_ash_to_ashwing),
        Op::Do(|g, c| {
            if ash_to_ashwing(g, c) {
                c.num -= 1;
            } else {
                c.num = 0;
            }
        }),
        Op::Back(
            |g, c| c.num > 0 && !ash_you_control(g, c.controller).is_empty(),
            2,
        ),
    ];
    db.add(d);

    // Sand Cover (rouge) — Draconic Illusionist Instant, coût 0.
    // « Target ash you control gets ward 4 until end of turn. »
    let mut d = deck_card("Sand Cover", R, DI, Types::INSTANT, Some(0), None, None);
    d.targets = vec![TARGET_ASH_YOU_CONTROL];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(a) = target_obj(c, 0) {
            let ward = Delta {
                ward: 4,
                ..Default::default()
            };
            g.modify(a, ward, Until::EndOfTurn);
        }
    })];
    db.add(d);

    // Sweeping Blow (rouge) — Draconic Illusionist Action - Attack, coût 1, 3{p} 3{d}.
    // « When this attacks, create an Ash token.
    //   Go again »
    let mut d = deck_card(
        "Sweeping Blow",
        R,
        DI,
        Types::ACTION,
        Some(1),
        Some(3),
        Some(3),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| create_ash(g, c.controller, 1))],
    }];
    db.add(d);

    add_dragons(db);
}

/// Le joueur contrôle-t-il une aura Illusionist ?
fn controls_illusionist_aura(g: &GameState, p: PlayerId) -> bool {
    g.permanents.iter().any(|&o| {
        g.obj(o).controller == p
            && g.def(o).subtypes.has(Subtypes::AURA)
            && g.supertypes(o).has(Supertypes::ILLUSIONIST)
    })
}

/// « Any target » sauf celle déjà choisie pour la première cible (« up to
/// any 2 targets » : deux cibles différentes).
const SECOND_ANY_TARGET: TargetSpec = TargetSpec {
    label: "deuxième cible (héros ou allié)",
    filter: |g, c, t| (super::wizard::ANY_TARGET.filter)(g, c, t) && c.target(0) != Some(t),
    optional: true,
    in_graveyard: false,
    in_banished: false,
};

/// Les huit invocations et leurs dragons.
fn add_dragons(db: &mut CardDb) {
    // Invoke Azvolai — coût 0. Azvolai : Dragon Ally, 2{p} 3{h}.
    // « Whenever Azvolai attacks, you may have him deal 1 arcane damage to up
    //   to any 2 targets. »
    // Les cibles sont déclarées quand la capacité se déclenche (CR 6.6.6a) ;
    // le « you may » est demandé à la résolution.
    let mut d = dragon("Azvolai", 2, 3);
    d.arcane_damage = true;
    let first = TargetSpec {
        optional: true,
        ..super::wizard::ANY_TARGET
    };
    d.triggers = vec![TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![first, SECOND_ANY_TARGET],
        script: vec![
            Op::Ask(|_, c| {
                (c.target(0).is_some() || c.target(1).is_some()).then(|| Ask {
                    player: c.controller,
                    label: "Azvolai inflige 1 dégât arcanique à ses cibles ?",
                    options: vec![Choice::Yes, Choice::No],
                })
            }),
            Op::Do(|g, c| {
                c.flag = c.answer == Some(Choice::Yes);
                if let (true, Some(t)) = (c.flag, c.target(0)) {
                    g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                }
            }),
            Op::Do(|g, c| {
                if let (true, Some(t)) = (c.flag, c.target(1)) {
                    g.deal_damage_to(c.source_id(), c.controller, t, 1, true);
                }
            }),
        ],
    }];
    add_invocation(db, "Invoke Azvolai", 0, d);

    // Invoke Cromai — coût 0. Cromai : Dragon Ally, 3{p} 2{h}.
    // « Once per turn, when Cromai attacks or leaves the arena, gain 1 action
    //   point. »
    // « Once per turn » : la capacité ne se déclenche pas en quittant l'arène
    // si Cromai a déjà attaqué ce tour (elle s'est alors déclenchée).
    let mut d = dragon("Cromai", 3, 2);
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            let attacks = |g: &GameState| {
                g.turn_events()
                    .iter()
                    .filter(|x| matches!(x, Event::Attack { attack, .. } if *attack == me))
                    .count()
            };
            match *e {
                Event::Attack { attack, .. } => attack == me && attacks(g) == 1,
                Event::LeaveArena { card, .. } => card == me && attacks(g) == 0,
                _ => false,
            }
        },
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_action_points(c.controller, 1))],
    }];
    add_invocation(db, "Invoke Cromai", 0, d);

    // Invoke Kyloria — coût 1. Kyloria : Dragon Ally, 4{p} 2{h}.
    // « Whenever Kyloria hits a hero, gain control of an item they control.
    //   Otherwise, draw a card. »
    let mut d = dragon("Kyloria", 4, 2);
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                let items = items_of(g, hit_hero(c)?);
                (items.len() > 1).then(|| Ask {
                    player: c.controller,
                    label: "Objet (item) dont vous gagnez le contrôle",
                    options: items.into_iter().map(Choice::Object).collect(),
                })
            }),
            Op::Do(|g, c| {
                let Some(them) = hit_hero(c) else { return };
                let items = items_of(g, them);
                let pick = match c.answer {
                    Some(Choice::Object(x)) if items.contains(&x) => Some(x),
                    _ => items.first().copied(),
                };
                match pick {
                    Some(x) => g.gain_control(c.controller, x),
                    None => {
                        g.draw(c.controller);
                    }
                }
            }),
        ],
    }];
    add_invocation(db, "Invoke Kyloria", 1, d);

    // Invoke Miragai — coût 1. Miragai : Dragon Ally, 2{p} 4{h}.
    // « Your first dragon attack each turn loses and can't gain phantasm. »
    let mut d = dragon("Miragai", 2, 4);
    d.aura = Some(|g, me, id| {
        let p = g.obj(me).controller;
        if g.obj(id).controller == p && is_first_attack(g, p, id, was_dragon) {
            loses_phantasm()
        } else {
            Delta::default()
        }
    });
    add_invocation(db, "Invoke Miragai", 1, d);

    // Invoke Nekria — coût 3. Nekria : Dragon Ally, 4{p} 7{h}.
    // « Whenever Nekria deals or is dealt damage, put a -1{h} counter on her
    //   and create an Ash token. »
    let mut d = dragon("Nekria", 4, 7);
    d.triggers = vec![TriggerDef {
        when: |_, me, e| match *e {
            Event::Damage {
                source, target_obj, ..
            } => source == Some(me) || target_obj == Some(me),
            _ => false,
        },
        // Le compteur n'est mis que si Nekria est encore là ; l'Ash est créée
        // dans tous les cas (CR 1.7.1a).
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(me) = c.source.filter(|&s| g.is_current(s)) {
                g.add_life_counters(me.id, -1);
            }
            create_ash(g, c.controller, 1);
        })],
    }];
    add_invocation(db, "Invoke Nekria", 3, d);

    // Invoke Ouvia — coût 2. Ouvia : Dragon Ally, 1{p} 6{h}.
    // « At the start of your turn or when Ouvia enters the arena, transform
    //   up to 1 ash you control into an Aether Ashwing. »
    let mut d = dragon("Ouvia", 1, 6);
    d.triggers = vec![TriggerDef {
        when: |g, me, e| match *e {
            Event::StartOfTurn(p) => p == g.obj(me).controller,
            Event::EnterArena { card } => card == me,
            _ => false,
        },
        anywhere: false,
        targets: vec![],
        script: up_to_one_ashwing(),
    }];
    add_invocation(db, "Invoke Ouvia", 2, d);

    // Invoke Themai — coût 2. Themai : Dragon Ally, 3{p} 4{h}.
    // « Opponents can't play cards or activate abilities during your turn. »
    let mut d = dragon("Themai", 3, 4);
    d.silences_opponents = true;
    add_invocation(db, "Invoke Themai", 2, d);

    // Invoke Yendurai — coût 1. Yendurai : Dragon Ally, 3{p} 3{h}.
    // « Yendurai enters the arena with an endurance counter on him.
    //   If Yendurai would be dealt damage, remove an endurance counter from
    //   him to prevent 3 of that damage. »
    let mut d = dragon("Yendurai", 3, 3);
    d.enter_counters = 1;
    d.counter_prevention = 3;
    add_invocation(db, "Invoke Yendurai", 1, d);
}

/// Le héros touché (« they ») d'une capacité « when … hits a hero ».
fn hit_hero(c: &Ctx) -> Option<PlayerId> {
    match c.event {
        Some(Event::Hit { target, .. }) => Some(target),
        _ => None,
    }
}

/// Les objets (items) que `p` contrôle.
fn items_of(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    g.permanents
        .iter()
        .copied()
        .filter(|&o| g.obj(o).controller == p && g.def(o).subtypes.has(Subtypes::ITEM))
        .collect()
}
