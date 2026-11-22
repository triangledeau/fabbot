//! Cartes Ninja du deck Ira, Crimson Haze.
//!
//! Combo (CR 8.4.1) : « Combo - If [NAMES] was the last attack this combat
//! chain, [EFFECTS] ». Voir [`last_attack_before`] pour la lecture retenue de
//! « the last attack this combat chain ».

use super::generic::this_hits;
use super::{by_color, card_named, deck_card, equipment, grant, power, target_obj, COLORS};
use fab_engine::card::{
    CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, Types,
};
use fab_engine::script::{Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, CardId, Event, GameState, ObjectId, PlayerId, TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Ira, Crimson Haze",
    "Edge of Autumn",
    "Mask of the Swarming Claw",
    "Blood Scent",
    "Double Cross Strap",
    "Tearing Shuko",
    "Pouncing Paws",
    "Crouching Tiger",
    "Aspect of Tiger: Body",
    "Bittering Thorns",
    "Blessing of Qi",
    "Flying Kick",
    "Growl",
    "Untamed",
    "Legacy of Ikaru",
    "Pouncing Qi",
    "Qi Unleashed",
    "Seek Vengeance",
    "Tiger Eye Reflex",
    "Feign Vengeance",
];

const N: Supertypes = Supertypes::NINJA;
const TIGER: &str = "Crouching Tiger";

/// « The last attack this combat chain », vu depuis l'attaque `me` : l'attaque
/// du maillon **précédent** celui de `me` (définition de la carte, CR 7.0.3c :
/// dernier état connu). Si `me` n'est pas encore sur la chaîne (sur la pile),
/// l'attaque du dernier maillon.
///
/// TODO(règles) : lecture retenue, car l'attaque elle-même ne peut pas être
/// « the last attack » de son propre combo (sinon Seek Vengeance ne
/// fonctionnerait jamais). Même lecture pour la capacité accordée par Legacy
/// of Ikaru. À confirmer par le propriétaire.
pub(crate) fn last_attack_before(g: &GameState, me: ObjectId) -> Option<CardId> {
    let links = &g.combat.links;
    match links.iter().position(|l| l.attack.id == me) {
        Some(k) => k.checked_sub(1).map(|j| links[j].card),
        None => links.last().map(|l| l.card),
    }
}

/// Combo : « If [name] was the last attack this combat chain ».
fn combo(g: &GameState, me: ObjectId, name: &str) -> bool {
    last_attack_before(g, me).is_some_and(|c| g.db.get(c).name == name)
}

fn is_tiger(g: &GameState, id: ObjectId) -> bool {
    g.def(id).name == TIGER
}

/// « When this attacks ».
fn when_this_attacks(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Attack { attack, .. } if *attack == me)
}

/// « Create a Crouching Tiger in your banished zone. You may play it this
/// turn » (`next_turn = false`) ou « during your next turn » (`true`). Renvoie
/// la carte créée.
fn create_tiger(g: &mut GameState, p: PlayerId, next_turn: bool) -> ObjectId {
    let tiger = card_named(g, TIGER);
    let id = g.create_card_banished(p, tiger);
    let turn = if next_turn { g.next_turn_of(p) } else { g.turn };
    g.permit_play(p, id, turn);
    id
}

/// Le joueur a-t-il attaqué avec un Crouching Tiger ce tour ?
fn attacked_with_tiger_this_turn(g: &GameState, p: PlayerId) -> bool {
    g.turn_events().iter().any(|e| {
        matches!(e, Event::Attack { attack, attacker, .. } if *attacker == p && is_tiger(g, *attack))
    })
}

/// Nombre de maillons de cette chaîne de combat dont l'attaque de `p` a touché.
fn hits_this_chain(g: &GameState, p: PlayerId) -> usize {
    g.combat
        .links
        .iter()
        .filter(|l| l.attacker == p && l.hit)
        .count()
}

/// Un équipement Ninja « Instant -- Destroy this: … » avec battleworn.
fn destroy_this_equipment(
    name: &str,
    slot: Subtypes,
    defense: u8,
    label: &'static str,
    condition: Option<fn(&GameState, PlayerId, ObjectId) -> bool>,
    script: Vec<Op>,
) -> CardDef {
    let mut d = equipment(name, N, slot, defense);
    d.keywords = Keywords::BATTLEWORN;
    d.activated = vec![ActivatedDef {
        label,
        speed: Speed::Instant,
        condition,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script,
        ..Default::default()
    }];
    d
}

/// Growl, Untamed : « When this attacks, the next Crouching Tiger you play
/// this combat chain gets +1{p}. »
fn next_tiger_this_chain_plus_one() -> TriggerDef {
    TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            g.next_attack_this_chain(c.controller, Some(is_tiger), power(1));
        })],
    }
}

pub fn add(db: &mut CardDb) {
    heroes_and_arena(db);
    deck_cards(db);
}

fn heroes_and_arena(db: &mut CardDb) {
    // Ira, Crimson Haze — Ninja Hero - Young, 20 PV, intellect 4.
    // « Your second attack each turn gets +1{p}. »
    db.add(CardDef {
        name: "Ira, Crimson Haze".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: N,
        aura: Some(|g, src, target| {
            let me = g.obj(src).controller;
            // L'attaque qui est devenue la 2e attaque du joueur ce tour : elle
            // est l'attaque active quand le compteur vaut 2 (les suivantes
            // deviennent actives à leur tour).
            let is_second = g.obj(target).zone == Zone::CombatChain
                && g.active_attack() == Some(target)
                && g.obj(target).controller == me
                && g.attacks_this_turn(me) == 2;
            if is_second {
                power(1)
            } else {
                Delta::default()
            }
        }),
        ..Default::default()
    });

    // Edge of Autumn — Ninja Weapon - Sword (2H), 1{p}.
    // « Once per Turn Action -- {r}: Attack. Go again »
    db.add(CardDef {
        name: "Edge of Autumn".into(),
        power: Some(1),
        types: Types::WEAPON,
        subtypes: Subtypes::SWORD | Subtypes::TWO_HAND,
        supertypes: N,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 1,
            once_per_turn: true,
            attack: true,
            go_again: true,
            ..Default::default()
        }],
        ..Default::default()
    });

    // Mask of the Swarming Claw — Ninja Equipment - Head, déf. 0.
    // « Arcane Barrier 1. Spellvoid X, where X is the number of chain links
    // you control. »
    let mut d = equipment("Mask of the Swarming Claw", N, Subtypes::HEAD, 0);
    d.arcane_barrier = 1;
    d.self_mod = Some(|g, me| {
        let p = g.obj(me).controller;
        let x = g.combat.links.iter().filter(|l| l.attacker == p).count();
        Delta {
            spellvoid: x.min(u8::MAX as usize) as u8,
            ..Default::default()
        }
    });
    db.add(d);

    // Blood Scent — Ninja Equipment - Chest, déf. 1.
    // « Instant -- Destroy this: Gain {r}. Activate this only if you've
    // attacked with a Crouching Tiger this turn. Battleworn »
    db.add(destroy_this_equipment(
        "Blood Scent",
        Subtypes::CHEST,
        1,
        "Détruire : gagner 1 ressource",
        Some(|g, p, _| attacked_with_tiger_this_turn(g, p)),
        vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
    ));

    // Double Cross Strap — Ninja Equipment - Chest, déf. 0.
    // « Instant -- Destroy this: Gain {r}. Activate this only if you've hit 2
    // or more times this combat chain. Arcane Barrier 1 »
    let mut d = equipment("Double Cross Strap", N, Subtypes::CHEST, 0);
    d.arcane_barrier = 1;
    d.activated = vec![ActivatedDef {
        label: "Détruire : gagner 1 ressource",
        speed: Speed::Instant,
        condition: Some(|g, p, _| hits_this_chain(g, p) >= 2),
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(d);

    // Tearing Shuko — Ninja Equipment - Arms, déf. 1.
    // « Instant -- Destroy this: The next Crouching Tiger you play this turn
    // gets +2{p}. Battleworn »
    db.add(destroy_this_equipment(
        "Tearing Shuko",
        Subtypes::ARMS,
        1,
        "Détruire : prochain Crouching Tiger +2{p}",
        None,
        vec![Op::Do(|g, c| {
            g.next_attack(c.controller, Some(is_tiger), power(2), true);
        })],
    ));

    // Pouncing Paws — Ninja Equipment - Legs, déf. 1.
    // « Instant -- Destroy this: Create a Crouching Tiger in your banished
    // zone. You may play it this turn. Battleworn »
    db.add(destroy_this_equipment(
        "Pouncing Paws",
        Subtypes::LEGS,
        1,
        "Détruire : créer un Crouching Tiger",
        None,
        vec![Op::Do(|g, c| {
            create_tiger(g, c.controller, false);
        })],
    ));

    // Crouching Tiger — Ninja Action - Attack, 0{p}, coût 0 (sans pitch ni
    // défense). « Ephemeral. Go again ». Ce n'est pas un token (ligne de type
    // officielle) : une carte créée, retirée de la partie au lieu d'aller au
    // cimetière (CR 8.3.21).
    db.add(CardDef {
        name: TIGER.into(),
        cost: Some(0),
        power: Some(0),
        types: Types::ACTION,
        subtypes: Subtypes::ATTACK,
        supertypes: N,
        keywords: Keywords::EPHEMERAL | Keywords::GO_AGAIN,
        ..Default::default()
    });
}

fn deck_cards(db: &mut CardDb) {
    // Aspect of Tiger: Body (rouge) — Ninja Action - Attack, 3, coût 0, déf. 3.
    // « Combo - When this attacks, if a red attack action card was the last
    // attack this combat chain, this gets go again and create a Crouching
    // Tiger in your banished zone. You may play it this turn. »
    let mut d = deck_card(
        "Aspect of Tiger: Body",
        Color::Red,
        N,
        Types::ACTION,
        Some(0),
        Some(3),
        Some(3),
    );
    d.triggers = vec![TriggerDef {
        when: when_this_attacks,
        anywhere: false,
        targets: vec![],
        // CR 6.6.1c : la condition est vérifiée à la résolution.
        script: vec![Op::Do(|g, c| {
            let Some(me) = c.source else { return };
            let red_attack_card = last_attack_before(g, me.id).is_some_and(|card| {
                let d = g.db.get(card);
                d.color == Some(Color::Red) && d.is_action() && d.is_attack_card()
            });
            if red_attack_card {
                g.modify(me, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                create_tiger(g, c.controller, false);
            }
        })],
    }];
    db.add(d);

    // Bittering Thorns — Ninja Action - Attack, 4/3/2, coût 1, déf. 2.
    // « When this hits, your next attack this turn gets +1{p}. Go again »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Bittering Thorns",
            c,
            N,
            Types::ACTION,
            Some(1),
            Some(4 - i as u8),
            Some(2),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.triggers = vec![TriggerDef {
            when: this_hits,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                g.next_attack(c.controller, None, power(1), false)
            })],
        }];
        db.add(d);
    }

    // Blessing of Qi — Ninja Action - Aura, coût 1, déf. 3.
    // « At the start of your turn, destroy this, then create a Crouching Tiger
    // in your banished zone. It gets +3{p} and you may play it this turn. »
    // (+3 / +2 / +1)
    for c in COLORS {
        let mut d = deck_card(
            "Blessing of Qi",
            c,
            N,
            Types::ACTION,
            Some(1),
            None,
            Some(3),
        );
        d.subtypes = Subtypes::AURA;
        d.triggers = vec![TriggerDef {
            when: |g, me, e| matches!(e, Event::StartOfTurn(p) if *p == g.obj(me).controller),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c: &mut Ctx| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.destroy(s.id);
                }
                let tiger = create_tiger(g, c.controller, false);
                let bonus = by_color(g, c, [3, 2, 1]);
                // Sans durée écrite ; le tigre ne peut être joué que ce tour.
                g.modify(g.obj_ref(tiger), power(bonus), Until::EndOfTurn);
            })],
        }];
        db.add(d);
    }

    // Flying Kick — Ninja Action - Attack, 5/4/3, coût 2, déf. 3.
    // « If this was played as chain link 3 or higher, it gets +2{p}. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Flying Kick",
            c,
            N,
            Types::ACTION,
            Some(2),
            Some(5 - i as u8),
            Some(3),
        );
        d.self_mod = Some(|g, me| {
            // CR 7.0.3a : l'attaque devient le maillon N+1.
            let links = &g.combat.links;
            let number = match links.iter().position(|l| l.attack.id == me) {
                Some(k) => k + 1,
                None if g.obj(me).zone == Zone::Stack => links.len() + 1,
                None => 0,
            };
            if number >= 3 {
                power(2)
            } else {
                Delta::default()
            }
        });
        db.add(d);
    }

    // Growl (rouge, jaune) — Ninja Action - Attack, 3/2, coût 0, déf. 2.
    // Untamed — Ninja Action - Attack, 4/3/2, coût 1, déf. 2.
    // « When this attacks, the next Crouching Tiger you play this combat chain
    // gets +1{p}. Go again »
    for (i, c) in COLORS.into_iter().enumerate() {
        let i = i as u8;
        for (name, cost, base) in [("Growl", 0, 3), ("Untamed", 1, 4)] {
            if name == "Growl" && c == Color::Blue {
                continue; // Growl n'existe qu'en rouge et jaune.
            }
            let mut d = deck_card(
                name,
                c,
                N,
                Types::ACTION,
                Some(cost),
                Some(base - i),
                Some(2),
            );
            d.keywords = Keywords::GO_AGAIN;
            d.triggers = vec![next_tiger_this_chain_plus_one()];
            db.add(d);
        }
    }

    // Legacy of Ikaru (bleu) — Ninja Attack Reaction, coût 0, déf. 3.
    // « Target Ninja attack gets +1{p} and "When this hits, if Edge of Autumn
    // was the last attack this combat chain, draw a card." »
    let mut d = deck_card(
        "Legacy of Ikaru",
        Color::Blue,
        N,
        Types::ATTACK_REACTION,
        Some(0),
        None,
        Some(3),
    );
    d.targets = vec![TargetSpec {
        label: "attaque Ninja",
        filter: |g, _, t| {
            matches!(t, Target::Object(r) if g.active_attack() == Some(r.id)
                && g.def(r.id).supertypes.has(Supertypes::NINJA))
        },
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(r) = target_obj(c, 0) {
            g.modify(r, power(1), Until::EndOfTurn);
            g.grant_trigger(r, c.card, 0, Until::EndOfTurn);
        }
    })];
    d.granted = vec![TriggerDef {
        when: this_hits,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if c.source.is_some_and(|s| combo(g, s.id, "Edge of Autumn")) {
                g.draw(c.controller);
            }
        })],
    }];
    db.add(d);

    // Pouncing Qi — Ninja Action - Attack, 3/2/1, coût 0, déf. 3.
    // « Combo - If Crouching Tiger was the last attack this combat chain, this
    // gets +1{p} and go again. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Pouncing Qi",
            c,
            N,
            Types::ACTION,
            Some(0),
            Some(3 - i as u8),
            Some(3),
        );
        d.self_mod = Some(|g, me| {
            if on_chain_or_stack(g, me) && combo(g, me, TIGER) {
                Delta {
                    power: 1,
                    grants: Keywords::GO_AGAIN,
                    ..Default::default()
                }
            } else {
                Delta::default()
            }
        });
        db.add(d);
    }

    // Qi Unleashed — Ninja Action - Attack, 3/2/1, coût 2, déf. 3.
    // « Combo - If Crouching Tiger was the last attack this combat chain, this
    // gets +4{p}. »
    for (i, c) in COLORS.into_iter().enumerate() {
        let mut d = deck_card(
            "Qi Unleashed",
            c,
            N,
            Types::ACTION,
            Some(2),
            Some(3 - i as u8),
            Some(3),
        );
        d.self_mod = Some(|g, me| {
            if on_chain_or_stack(g, me) && combo(g, me, TIGER) {
                power(4)
            } else {
                Delta::default()
            }
        });
        db.add(d);
    }

    // Seek Vengeance (rouge, bleu) — Ninja Action - Attack, 4 / 2, coût 0, déf. 3.
    // « Combo - If Edge of Autumn was the last attack this combat chain, this
    // gets go again. »
    for (c, p) in [(Color::Red, 4), (Color::Blue, 2)] {
        let mut d = deck_card(
            "Seek Vengeance",
            c,
            N,
            Types::ACTION,
            Some(0),
            Some(p),
            Some(3),
        );
        d.self_mod = Some(|g, me| {
            if on_chain_or_stack(g, me) && combo(g, me, "Edge of Autumn") {
                grant(Keywords::GO_AGAIN)
            } else {
                Delta::default()
            }
        });
        db.add(d);
    }

    // Tiger Eye Reflex (jaune, bleu) — Ninja Block, déf. 3 / 2.
    // « Ambush. When this defends, create a Crouching Tiger in your banished
    // zone. You may play it during your next turn. »
    for (c, def) in [(Color::Yellow, 3), (Color::Blue, 2)] {
        let mut d = deck_card(
            "Tiger Eye Reflex",
            c,
            N,
            Types::BLOCK,
            None,
            None,
            Some(def),
        );
        d.keywords = Keywords::AMBUSH;
        d.triggers = vec![TriggerDef {
            when: |_, me, e| matches!(e, Event::Defend { card, .. } if *card == me),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                create_tiger(g, c.controller, true);
            })],
        }];
        db.add(d);
    }

    // Feign Vengeance (bleu) — Ninja Action - Attack, 0, coût 0, déf. 3.
    // « When this chain link resolves, if there is a card defending this, draw
    // a card. Go again »
    let mut d = deck_card(
        "Feign Vengeance",
        Color::Blue,
        N,
        Types::ACTION,
        Some(0),
        Some(0),
        Some(3),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::ChainLinkResolved { attack, .. } if *attack == me),
        anywhere: false,
        targets: vec![],
        // CR 7.0.5a : une carte défend tant qu'elle est sur la chaîne.
        script: vec![Op::Do(|g, c| {
            let Some(me) = c.source_id() else { return };
            let defended = g.combat.links.iter().any(|l| {
                l.attack.id == me
                    && l.defenders
                        .iter()
                        .any(|&d| g.obj(d).zone == Zone::CombatChain)
            });
            if defended {
                g.draw(c.controller);
            }
        })],
    }];
    db.add(d);
}

/// Un combo ne s'applique qu'à une attaque sur la pile ou la chaîne.
fn on_chain_or_stack(g: &GameState, me: ObjectId) -> bool {
    matches!(g.obj(me).zone, Zone::CombatChain | Zone::Stack)
}
