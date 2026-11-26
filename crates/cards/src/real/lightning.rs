//! Cartes Lightning, Elemental et Runeblade du deck Aurora (essai
//! d'encodage deck par deck, 2026-11-26).

#![allow(unused_imports)]

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

    // CARTES
}
