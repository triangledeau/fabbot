//! Cartes Wizard, Elemental et Ice du deck Iyslander.

use super::{by_color, card_named, deck_card, equipment, grant};
use fab_engine::card::{CardDb, CardDef, Color, Keywords, Subtypes, Supertypes, Types};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, PlayerId, ReplaceKind, Replacement, Speed,
    TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Iyslander",
    "Crucible of Aetherweave",
    "Spellfire Cloak",
    "Aetherstorm Wellingtons",
    "Aethersling",
    "Arcane Twining",
    "Frost Spike",
    "Frosting",
    "Ice Bolt",
    "Ice Eternal",
    "Photon Splicing",
    "Pyroglyphic Protection",
    "Stir the Aetherwinds",
    "Timekeeper's Whim",
    "Turn to Mindfire",
    "Voltic Bolt",
    "Winter's Bite",
    "Frostbite",
    "Ponder",
];

const WIZ: Supertypes = Supertypes::WIZARD;
const ICE_WIZ: Supertypes = Supertypes(Supertypes::ICE.0 | Supertypes::WIZARD.0);
const FROSTBITE: &str = "Frostbite";

/// « any target » : un héros ou un permanent vivant (allié). CR 1.8.5d :
/// seuls les objets qui ont la propriété vie peuvent recevoir des dégâts.
pub(crate) const ANY_TARGET: TargetSpec = TargetSpec {
    label: "cible (héros ou allié)",
    filter: |g, _, t| match t {
        Target::Player(_) => true,
        Target::Object(r) => g.is_living_permanent(r.id),
    },
    optional: false,
};

/// « target hero ».
pub(crate) const TARGET_HERO: TargetSpec = TargetSpec {
    label: "héros",
    filter: |_, _, t| matches!(t, Target::Player(_)),
    optional: false,
};

/// « Deal N arcane damage to [la cible n° 0] » : la source est la carte.
fn arcane_to_target(g: &mut GameState, c: &Ctx, n: i16) {
    if let Some(t) = c.target(0) {
        g.deal_damage_to(c.source_id(), c.controller, t, n as i32, true);
    }
}

/// Une carte Wizard d'action non-attaque.
fn wizard_non_attack_action(g: &GameState, c: ObjectId) -> bool {
    let d = g.def(c);
    d.supertypes.has(Supertypes::WIZARD) && d.is_action() && !d.is_attack_card()
}

/// Nombre de Frostbite que `p` contrôle (zone permanente et zones
/// d'équipement, où Frost Spike peut en créer).
pub(crate) fn frostbites(g: &GameState, p: PlayerId) -> usize {
    g.permanents
        .iter()
        .copied()
        .filter(|&o| g.obj(o).controller == p)
        .chain(g.player(p).equipment())
        .filter(|&o| g.def(o).name == FROSTBITE)
        .count()
}

/// « If this deals damage, you may {t} your hero. If you do, … » : la
/// question, posée seulement si des dégâts ont été infligés et si le héros
/// peut être engagé (CR 8.5.55a : sinon l'effet échoue).
fn ask_tap_hero(g: &GameState, c: &Ctx) -> Option<Ask> {
    let hero = g.player(c.controller).hero;
    (g.last_damage > 0 && !g.obj(hero).tapped).then(|| Ask {
        player: c.controller,
        label: "Engager votre héros ?",
        options: vec![Choice::Yes, Choice::No],
    })
}

/// Réponse à [`ask_tap_hero`] : engage le héros ; renvoie `true` si c'est fait.
fn tap_hero_if_yes(g: &mut GameState, c: &mut Ctx) -> bool {
    if c.answer.take() != Some(Choice::Yes) {
        return false;
    }
    let hero = g.player(c.controller).hero;
    g.tap(hero)
}

/// « Instant -- Discard this: Amp 1 » (Arcane Twining, Photon Splicing).
fn discard_amp_1() -> ActivatedDef {
    ActivatedDef {
        label: "Défausser : amp 1",
        speed: Speed::Instant,
        from_hand: true,
        effect_cost: vec![Op::Do(|g, c| {
            if let Some(s) = c.source_id() {
                g.discard(s);
            }
        })],
        script: vec![Op::Do(|g, c| g.amp(c.controller, 1))],
        ..Default::default()
    }
}

/// « Deal N arcane damage to any target » (Frosting, Ice Bolt), N selon la
/// couleur `[rouge, jaune, bleu]`.
fn arcane_spell(
    name: &str,
    class: Supertypes,
    color: Color,
    cost: u8,
    defense: u8,
    target: TargetSpec,
    script: Vec<Op>,
) -> CardDef {
    let mut d = deck_card(
        name,
        color,
        class,
        Types::ACTION,
        Some(cost),
        None,
        Some(defense),
    );
    d.targets = vec![target];
    d.arcane_damage = true;
    d.resolve = script;
    d
}

pub fn add(db: &mut CardDb) {
    // Iyslander — Elemental Wizard Hero - Young, 18{h}, intellect 4.
    // « Essence of Ice
    //   If it's not your turn, you may play blue non-attack action cards from
    //   your arsenal as though they were instants.
    //   Whenever you play an Ice card during an opponent's turn, create a
    //   Frostbite token under their control. »
    // Essence of Ice (CR 8.3.16) ne concerne que la construction du deck.
    db.add(CardDef {
        name: "Iyslander".into(),
        life: Some(18),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: Supertypes::ELEMENTAL | WIZ,
        play_as_instant: Some(|g, me, c| {
            let d = g.def(c);
            g.turn_player != g.obj(me).controller
                && g.obj(c).zone == Zone::Arsenal
                && d.color == Some(Color::Blue)
                && d.is_action()
                && !d.is_attack_card()
        }),
        triggers: vec![TriggerDef {
            when: |g, me, e| match *e {
                Event::Played { player, card, .. } => {
                    player == g.obj(me).controller
                        && g.turn_player != player
                        && g.def(card).supertypes.has(Supertypes::ICE)
                }
                _ => false,
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                // « their » : l'adversaire dont c'est le tour.
                let them = g.turn_player;
                if them != c.controller {
                    let f = card_named(g, FROSTBITE);
                    g.create_tokens(them, f, 1);
                }
            })],
        }],
        ..Default::default()
    });

    // Frostbite — Elemental Token - Aura.
    // « Cards and abilities cost you an additional {r} to play or activate.
    //   When you play a card or activate an ability, destroy this.
    //   At the beginning of your end phase, destroy this. » (CR 8.6.10)
    db.add(CardDef {
        name: FROSTBITE.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        supertypes: Supertypes::ELEMENTAL,
        cost_tax: 1,
        triggers: vec![
            TriggerDef {
                when: |g, me, e| {
                    let me = g.obj(me).controller;
                    matches!(*e, Event::Played { player, .. } | Event::Activated { player, .. } if player == me)
                },
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
            },
            TriggerDef {
                when: |g, me, e| matches!(*e, Event::BeginEndPhase(p) if p == g.obj(me).controller),
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
            },
        ],
        ..Default::default()
    });

    // Ponder — Generic Token - Aura.
    // « At the beginning of your end phase, destroy this and draw a card. »
    db.add(CardDef {
        name: "Ponder".into(),
        types: Types::TOKEN,
        subtypes: Subtypes::AURA,
        triggers: vec![TriggerDef {
            when: |g, me, e| matches!(*e, Event::BeginEndPhase(p) if p == g.obj(me).controller),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                super::generic::destroy_source(g, c);
                g.draw(c.controller);
            })],
        }],
        ..Default::default()
    });

    // Crucible of Aetherweave — Wizard Weapon - Staff (2H).
    // « Once per Turn Instant -- {r}: The next card you play this turn with an
    //   arcane damage effect, instead deals that much arcane damage plus 1. »
    db.add(CardDef {
        name: "Crucible of Aetherweave".into(),
        types: Types::WEAPON,
        subtypes: Subtypes::STAFF | Subtypes::TWO_HAND,
        supertypes: WIZ,
        activated: vec![ActivatedDef {
            label: "{r} : la prochaine carte à dégâts arcaniques inflige +1",
            speed: Speed::Instant,
            cost: 1,
            once_per_turn: true,
            script: vec![Op::Do(|g, c| {
                g.add_replacement(next_card_arcane_plus(
                    c.controller,
                    |g, card| g.def(card).arcane_damage,
                    1,
                ));
            })],
            ..Default::default()
        }],
        ..Default::default()
    });

    // Spellfire Cloak — Wizard Equipment - Chest, déf. 0.
    // « Instant -- Destroy this: Gain {r}. Activate this only during an
    //   opponent's turn.
    //   Arcane Barrier 1 »
    let mut d = equipment("Spellfire Cloak", WIZ, Subtypes::CHEST, 0);
    d.arcane_barrier = 1;
    d.activated = vec![ActivatedDef {
        label: "Détruire : gagner {r}",
        speed: Speed::Instant,
        condition: Some(|g, p, _| g.turn_player != p),
        effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(d);

    // Aetherstorm Wellingtons — Wizard Equipment - Legs, déf. 0.
    // « Arcane Barrier 2 »
    let mut d = equipment("Aetherstorm Wellingtons", WIZ, Subtypes::LEGS, 0);
    d.arcane_barrier = 2;
    db.add(d);

    // Aethersling (rouge) — Wizard Action, coût 1, déf. 2.
    // « Deal 4 arcane damage to any target.
    //   If this deals damage, you may {t} your hero. If you do, this gets go again. »
    db.add(arcane_spell(
        "Aethersling",
        WIZ,
        Color::Red,
        1,
        2,
        ANY_TARGET,
        vec![
            Op::Do(|g, c| arcane_to_target(g, c, 4)),
            Op::Ask(ask_tap_hero),
            Op::Do(|g, c| {
                if tap_hero_if_yes(g, c) {
                    if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                        g.modify(s, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                    }
                }
            }),
        ],
    ));

    // Turn to Mindfire (rouge) — Wizard Action, coût 2, déf. 2.
    // « Deal 5 arcane damage to any target.
    //   If this deals damage, you may {t} your hero. If you do, create a Ponder token. »
    db.add(arcane_spell(
        "Turn to Mindfire",
        WIZ,
        Color::Red,
        2,
        2,
        ANY_TARGET,
        vec![
            Op::Do(|g, c| arcane_to_target(g, c, 5)),
            Op::Ask(ask_tap_hero),
            Op::Do(|g, c| {
                if tap_hero_if_yes(g, c) {
                    let ponder = card_named(g, "Ponder");
                    g.create_tokens(c.controller, ponder, 1);
                }
            }),
        ],
    ));

    // Arcane Twining (bleu) — Wizard Action, coût 0, déf. 3.
    // « Deal 1 arcane damage to any target. Instant -- Discard this: Amp 1 » (3, 2, 1).
    // Photon Splicing (bleu) — Wizard Action, coût 1, déf. 3.
    // « Deal 2 arcane damage to any target. Instant -- Discard this: Amp 1 » (4, 3, 2).
    for (name, cost) in [("Arcane Twining", 0), ("Photon Splicing", 1)] {
        let mut d = arcane_spell(
            name,
            WIZ,
            Color::Blue,
            cost,
            3,
            ANY_TARGET,
            vec![Op::Do(|g, c| {
                let n = if g.db.get(c.card).name == "Arcane Twining" {
                    by_color(g, c, [3, 2, 1])
                } else {
                    by_color(g, c, [4, 3, 2])
                };
                arcane_to_target(g, c, n);
            })],
        );
        d.activated = vec![discard_amp_1()];
        db.add(d);
    }

    // Frosting (bleu) — Ice Wizard Action, coût 0, déf. 3.
    // « Deal 1 arcane damage to any target. » (3, 2, 1)
    db.add(arcane_spell(
        "Frosting",
        ICE_WIZ,
        Color::Blue,
        0,
        3,
        ANY_TARGET,
        vec![Op::Do(|g, c| {
            arcane_to_target(g, c, by_color(g, c, [3, 2, 1]))
        })],
    ));

    // Ice Bolt (rouge, bleu) — Ice Wizard Action, coût 2, déf. 3.
    // « Deal 5 arcane damage to any target. » (5, 4, 3)
    // Voltic Bolt (rouge, bleu) — Wizard Action, coût 2, déf. 3.
    // « Deal 5 arcane damage to target hero. » (5, 4, 3)
    for c in [Color::Red, Color::Blue] {
        db.add(arcane_spell(
            "Ice Bolt",
            ICE_WIZ,
            c,
            2,
            3,
            ANY_TARGET,
            vec![Op::Do(|g, c| {
                arcane_to_target(g, c, by_color(g, c, [5, 4, 3]))
            })],
        ));
        db.add(arcane_spell(
            "Voltic Bolt",
            WIZ,
            c,
            2,
            3,
            TARGET_HERO,
            vec![Op::Do(|g, c| {
                arcane_to_target(g, c, by_color(g, c, [5, 4, 3]))
            })],
        ));
    }

    // Timekeeper's Whim (bleu) — Wizard Action, coût 3, déf. 3.
    // « Deal 3 arcane damage to target hero. (5, 4, 3)
    //   If this was played during an opponent's turn, put it on the bottom of
    //   its owner's deck. »
    db.add(arcane_spell(
        "Timekeeper's Whim",
        WIZ,
        Color::Blue,
        3,
        3,
        TARGET_HERO,
        vec![
            Op::Do(|g, c| arcane_to_target(g, c, by_color(g, c, [5, 4, 3]))),
            Op::Do(|g, c| {
                // La carte se résout pendant le tour où elle a été jouée.
                if g.turn_player != c.controller {
                    if let Some(s) = c
                        .source
                        .filter(|&s| g.is_current(s) && g.obj(s.id).zone == Zone::Stack)
                    {
                        g.put_on_bottom(s.id);
                    }
                }
            }),
        ],
    ));

    // Ice Eternal (bleu) — Elemental Wizard Action, coût XX, déf. 3.
    // « Iyslander Specialization
    //   Ice Fusion
    //   Create X Frostbite tokens under target hero's control. Then if this was
    //   fused, deal arcane damage to that hero equal to the number of
    //   Frostbites they control. »
    // Specialization (CR 8.3.7) ne concerne que la construction du deck.
    let mut d = deck_card(
        "Ice Eternal",
        Color::Blue,
        Supertypes::ELEMENTAL | WIZ,
        Types::ACTION,
        Some(0),
        None,
        Some(3),
    );
    d.x_cost = true;
    d.x_twice = true;
    d.arcane_damage = true;
    d.targets = vec![TARGET_HERO];
    // CR 8.3.17 : « As an additional cost to play this, you may reveal an Ice
    // card from your hand. » Coût d'effet optionnel, payé après les
    // ressources (CR 5.1.9) : seules les cartes encore en main comptent.
    d.effect_cost = vec![
        Op::Ask(|g, c| {
            let ice: Vec<Choice> = g
                .player(c.controller)
                .hand
                .iter()
                .filter(|&&x| g.def(x).supertypes.has(Supertypes::ICE))
                .map(|&x| Choice::Object(x))
                .collect();
            (!ice.is_empty()).then(|| Ask {
                player: c.controller,
                label: "Ice Fusion : révéler une carte Ice de votre main ?",
                options: ice.into_iter().chain([Choice::Pass]).collect(),
            })
        }),
        Op::Do(|g, c| {
            if let Some(Choice::Object(x)) = c.answer.take() {
                g.emit(Event::Revealed {
                    player: c.controller,
                    card: x,
                });
                c.optional_paid = true; // CR 8.3.17a : la carte est fusionnée.
            }
        }),
    ];
    d.resolve = vec![
        Op::Do(|g, c| {
            if let Some(Target::Player(p)) = c.target(0) {
                let f = card_named(g, FROSTBITE);
                g.create_tokens(p, f, c.x);
            }
        }),
        Op::Do(|g, c| {
            if let (true, Some(Target::Player(p))) = (c.optional_paid, c.target(0)) {
                let n = frostbites(g, p) as i32;
                g.deal_damage(c.source_id(), c.controller, p, n, true);
            }
        }),
    ];
    db.add(d);

    // Frost Spike (bleu) — Ice Wizard Instant, coût 0, pas de défense.
    // « Create a Frostbite token in an exposed head, chest, arms, or legs zone.
    //   (A zone is exposed if there is nothing in or equipped to it.) »
    // CR 8.5.2c : le token entre sous le contrôle du joueur à qui appartient la
    // zone. Ce n'est pas une cible : la zone est choisie à la résolution.
    let mut d = deck_card(
        "Frost Spike",
        Color::Blue,
        ICE_WIZ,
        Types::INSTANT,
        Some(0),
        None,
        None,
    );
    d.resolve = vec![
        Op::Ask(|g, c| {
            let zones = g.exposed_zones();
            (!zones.is_empty()).then(|| Ask {
                player: c.controller,
                label: "Zone exposée où créer le Frostbite",
                options: zones.into_iter().map(|(p, z)| Choice::Zone(p, z)).collect(),
            })
        }),
        Op::Do(|g, c| {
            if let Some(Choice::Zone(p, z)) = c.answer.take() {
                let f = card_named(g, FROSTBITE);
                g.create_token_in_zone(p, f, z);
            }
        }),
    ];
    db.add(d);

    // Pyroglyphic Protection (bleu) — Wizard Action - Aura, coût 2, déf. 2.
    // « If you would be dealt arcane damage, prevent 1 of that damage. (3, 2, 1)
    //   At the beginning of your action phase, destroy this. »
    let mut d = deck_card(
        "Pyroglyphic Protection",
        Color::Blue,
        WIZ,
        Types::ACTION,
        Some(2),
        None,
        Some(2),
    );
    d.subtypes = Subtypes::AURA;
    d.prevent_arcane = 1;
    d.triggers = vec![TriggerDef {
        when: |g, me, e| matches!(*e, Event::BeginActionPhase(p) if p == g.obj(me).controller),
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
    }];
    db.add(d);

    // Stir the Aetherwinds (bleu) — Wizard Action, coût 2, déf. 2.
    // « You may play your next Wizard non-attack action card this turn as
    //   though it were an instant. If it has an arcane damage effect, instead
    //   it deals that much arcane damage plus 1. » (3, 2, 1)
    let mut d = deck_card(
        "Stir the Aetherwinds",
        Color::Blue,
        WIZ,
        Types::ACTION,
        Some(2),
        None,
        Some(2),
    );
    d.resolve = vec![Op::Do(|g, c| {
        g.next_card_as_instant(c.controller, wizard_non_attack_action);
        let n = by_color(g, c, [3, 2, 1]) as i32;
        g.add_replacement(next_card_arcane_plus(
            c.controller,
            wizard_non_attack_action,
            n,
        ));
    })];
    db.add(d);

    // Winter's Bite (bleu) — Ice Action, coût 0, déf. 2.
    // « Target hero discards a card unless they pay {r}. ({r}{r}{r}, {r}{r}, {r})
    //   Go again »
    let mut d = deck_card(
        "Winter's Bite",
        Color::Blue,
        Supertypes::ICE,
        Types::ACTION,
        Some(0),
        None,
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.targets = vec![TARGET_HERO];
    d.resolve = vec![
        // Le héros ciblé choisit de payer s'il le peut (et s'il a une carte à défausser).
        Op::Ask(|g, c| {
            let Some(Target::Player(p)) = c.target(0) else {
                return None;
            };
            let n = by_color(g, c, [3, 2, 1]) as u32;
            (!g.player(p).hand.is_empty() && g.available_resources(p, None) >= n).then(|| Ask {
                player: p,
                label: "Payer pour ne pas défausser ?",
                options: vec![Choice::Yes, Choice::No],
            })
        }),
        Op::Do(|g, c| {
            let Some(Target::Player(p)) = c.target(0) else {
                return;
            };
            if c.answer.take() == Some(Choice::Yes) {
                let n = by_color(g, c, [3, 2, 1]) as u8;
                g.pay(p, n);
                c.flag = true;
            }
        }),
        Op::If(|_, c| !c.flag, 2),
        // CR 8.5.5 : le joueur choisit la carte défaussée.
        Op::Ask(|g, c| {
            let Some(Target::Player(p)) = c.target(0) else {
                return None;
            };
            let hand = &g.player(p).hand;
            (hand.len() > 1).then(|| Ask {
                player: p,
                label: "Carte à défausser",
                options: hand.iter().map(|&x| Choice::Object(x)).collect(),
            })
        }),
        Op::Do(|g, c| {
            let Some(Target::Player(p)) = c.target(0) else {
                return;
            };
            let card = match c.answer {
                Some(Choice::Object(x)) if g.obj(x).zone == Zone::Hand => Some(x),
                _ => g.player(p).hand.first().copied(),
            };
            if let Some(x) = card {
                g.discard(x);
            }
        }),
    ];
    db.add(d);
}

/// « The next [carte filtrée] you play this turn … instead deals that much
/// arcane damage plus N » (Crucible of Aetherweave, Stir the Aetherwinds) :
/// la carte est mémorisée quand elle est jouée (`Replacement::bind`), puis
/// ses dégâts arcaniques sont augmentés.
fn next_card_arcane_plus(
    p: PlayerId,
    filter: fn(&GameState, ObjectId) -> bool,
    n: i32,
) -> Replacement {
    Replacement {
        id: 0,
        kind: ReplaceKind::Damage,
        controller: p,
        data: None,
        bind: Some(filter),
        value: n,
        optional: false,
        applies: |_, w, pending| pending.arcane && w.me.is_some() && pending.object == w.me,
        apply: |_, w, pending| pending.amount += w.value,
        until: Until::EndOfTurn,
        uses: None,
    }
}
