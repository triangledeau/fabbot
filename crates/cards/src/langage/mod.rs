//! Le **langage de cartes** (prototype, 2026-11-25).
//!
//! Une carte s'écrit en quelques lignes de données (`crates/cards/cartes/*.cartes`)
//! au lieu de code Rust. Le même texte sert à trois choses :
//! 1. **construire** la carte pour le moteur ([`add`]) : chaque phrase devient
//!    une instruction [`Op::Run`] / [`Op::IfArg`] qui pointe vers une table ;
//! 2. **réécrire** la carte en anglais ([`render`]) pour la comparer au texte
//!    officiel (outil `cargo run -p fab-cards --bin langage -- verifier`) ;
//! 3. servir de cible à un script ou à un modèle qui encode les cartes
//!    automatiquement (voir `docs/langage-cartes.md`).
//!
//! Le vocabulaire est **fermé** : une phrase qui n'existe pas dans le langage
//! s'écrit `native NOM "texte"`, avec du Rust dans le module de la classe.

pub mod deviner;
pub mod officiel;
mod parse;
mod render;
mod run;

pub use parse::{parse, ParseError};
pub use render::{normalize_official, render};

use fab_engine::card::{CardDb, Color, TriggerDef};
use fab_engine::script::Op;
use fab_engine::{GameState, ObjectId};

/// Un fichier `.cartes` lu.
#[derive(Debug, Default)]
pub struct File {
    pub cards: Vec<CardSpec>,
    /// Toutes les phrases (instructions) du fichier : [`Op::Run`] y renvoie par index.
    pub stmts: Vec<Stmt>,
    /// Toutes les conditions du fichier : [`Op::IfArg`] y renvoie par index.
    pub conds: Vec<Cond>,
}

/// Une carte : nom, couleurs, valeurs imprimées et capacités dans l'ordre du texte.
#[derive(Debug, Default)]
pub struct CardSpec {
    pub name: String,
    /// Vide = carte sans couleur (héros, équipement, token).
    pub colors: Vec<Color>,
    /// Ligne de type officielle, telle qu'imprimée (« Warrior Weapon - Sword (2H) »).
    pub typebox: String,
    pub stats: Stats,
    /// Cible déclarée en jouant la carte (une seule dans ce prototype).
    pub target: Option<TargetKind>,
    pub items: Vec<Item>,
    /// Ligne du fichier où commence la carte (messages d'erreur).
    pub line: usize,
}

/// Une valeur qui peut dépendre de la couleur : `[rouge, jaune, bleu]`.
pub type ByColor = [i16; 3];

#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    pub cost: Option<ByColor>,
    pub power: Option<ByColor>,
    pub defense: Option<ByColor>,
    pub life: Option<u8>,
    pub intellect: Option<u8>,
}

/// Un morceau du texte de la carte, dans l'ordre.
#[derive(Debug)]
pub enum Item {
    Keyword(Keyword),
    PlayOnlyIf(PlayCond),
    /// Effets à la résolution (index dans [`File::stmts`]).
    Resolve(Vec<Line>),
    Activated(Activated),
    Trigger {
        label: TriggerLabel,
        event: EventKind,
        lines: Vec<Line>,
    },
    NativeTrigger {
        name: String,
        text: String,
    },
    NativeStatic {
        name: String,
        text: String,
    },
}

/// Une ligne d'effet : une phrase, éventuellement sous condition.
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub cond: Option<u16>,
    pub stmt: u16,
}

#[derive(Debug)]
pub struct Activated {
    pub speed: Speed,
    pub cost: u8,
    pub tap: bool,
    pub once_per_turn: bool,
    pub go_again: bool,
    pub attack: bool,
    pub destroy_this: bool,
    pub only_if: Option<ActivateCond>,
    pub target: Option<TargetKind>,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Action,
    Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    GoAgain,
    Temper,
    Battleworn,
    Dominate,
    /// « Hala Specialization » : ne concerne que la construction du deck.
    Specialization(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Attack,
    WeaponAttack,
    SwordYouControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackFilter {
    Any,
    Weapon,
    Axe,
    Sword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCond {
    AttackActionDefending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivateCond {
    SharpenedSwordThisTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerLabel {
    Unity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    DefendsWithHandCard,
}

/// Les conditions d'une ligne `if … then …`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    AttackedWithWeaponThisTurn,
    Reprise,
    TargetCountersAtLeast(ByColor),
    DefendsWeaponAttack,
}

/// Les phrases du langage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// « Your next [filtre] attack this turn gets +N{p} / go again. »
    NextAttack {
        filter: AttackFilter,
        gets: Gets,
    },
    /// « Target … gets +N{p}. » (avec « Reprise - … instead it gets +M{p}. »)
    TargetGets {
        power: ByColor,
        instead_if_reprise: Option<ByColor>,
    },
    /// « Sharpen target sword you control. »
    SharpenTarget,
    /// « its next attack this turn gets … » (l'arme ciblée).
    TargetNextAttackGets(Keyword),
    /// « your next attack with it this turn costs {r} less to activate »
    TargetNextAttackCostsLess(u8),
    CreateToken(String),
    DealDamageAttackingHero(u8),
    GainResources(u8),
    /// « this gets +N{d} until end of turn »
    ThisGetsDefense(i16),
    /// Morceau écrit en Rust.
    Native {
        name: String,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gets {
    Power(ByColor),
    Keyword(Keyword),
}

/// Ce que le module Rust d'une classe fournit pour les lignes `native…`.
pub enum Native {
    Ops(Vec<Op>),
    Trigger(TriggerDef),
    Static(fn(&GameState, ObjectId, ObjectId) -> fab_engine::Delta),
}

/// Une fonction qui donne le morceau Rust d'un nom `native`.
pub type NativeFn = fn(&str) -> Option<Native>;

/// Ajoute à la base toutes les cartes d'un fichier déjà lu.
///
/// `file` doit vivre aussi longtemps que le programme (les instructions y
/// renvoient par index) : voir [`run::program`].
pub fn add(db: &mut CardDb, prog: run::ProgramId, natives: NativeFn) {
    run::add_all(db, prog, natives)
}

pub use run::{program, ProgramId};
