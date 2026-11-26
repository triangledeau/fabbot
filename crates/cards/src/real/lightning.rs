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
    // CARTES
}
