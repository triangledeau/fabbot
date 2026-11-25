//! Construction des cartes pour le moteur et exécution des phrases.
//!
//! Les fichiers `.cartes` sont inclus dans le programme et lus une seule fois
//! ([`program`]). Une phrase devient `Op::Run(exec, arg)` : `arg` contient le
//! numéro du fichier (4 bits) et celui de la phrase (12 bits).

use super::*;
use crate::real::{
    card_named, deck_card, equipment, grant, is_sword, is_sword_attack, power, reprise, target_obj,
    TARGET_ATTACK, TARGET_SWORD_YOU_CONTROL, TARGET_WEAPON_ATTACK,
};
use fab_engine::card::{
    ActivatedDef, CardDef, Delta, Keywords, Speed as ESpeed, Subtypes, Supertypes, Types,
};
use fab_engine::script::{Ctx, TargetSpec};
use fab_engine::{Event, Until, WeaponEffectKind, Zone};
use std::sync::OnceLock;

/// Les fichiers de cartes inclus dans le programme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramId {
    Warrior = 0,
}

const SOURCES: [&str; 1] = [include_str!("../../cartes/warrior.cartes")];

static PROGRAMS: [OnceLock<File>; 1] = [OnceLock::new()];

/// Le fichier lu (une seule fois). Panique si le fichier est mal écrit :
/// le test `langage_fichiers_valides` le signale avant.
pub fn program(id: ProgramId) -> &'static File {
    PROGRAMS[id as usize].get_or_init(|| match parse(SOURCES[id as usize]) {
        Ok(f) => f,
        Err(e) => panic!("fichier de cartes {id:?}, {e}"),
    })
}

fn arg(prog: ProgramId, idx: u16) -> u16 {
    assert!(idx < 4096, "trop de phrases dans un fichier");
    ((prog as u16) << 12) | idx
}

fn file_of(arg: u16) -> (&'static File, usize) {
    let prog = match arg >> 12 {
        0 => ProgramId::Warrior,
        n => panic!("fichier de cartes inconnu : {n}"),
    };
    (program(prog), (arg & 0x0fff) as usize)
}

fn color_index(g: &GameState, c: &Ctx) -> usize {
    match g.db.get(c.card).color {
        Some(Color::Red) | None => 0,
        Some(Color::Yellow) => 1,
        Some(Color::Blue) => 2,
    }
}

pub(super) fn add_all(db: &mut CardDb, prog: ProgramId, natives: NativeFn) {
    let file = program(prog);
    for card in &file.cards {
        let colors: Vec<Option<Color>> = if card.colors.is_empty() {
            vec![None]
        } else {
            card.colors.iter().map(|&c| Some(c)).collect()
        };
        for color in colors {
            db.add(build(prog, card, color, natives));
        }
    }
}

/// Classe(s), types et sous-types à partir de la ligne de type officielle.
fn typebox(s: &str) -> (Supertypes, Types, Subtypes) {
    let (left, right) = s.split_once(" - ").unwrap_or((s, ""));
    let mut sup = Supertypes::NONE;
    let mut ty = Types::NONE;
    let mut sub = Subtypes::NONE;
    let words: Vec<&str> = left.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let two = words.get(i + 1).map(|n| format!("{} {}", words[i], n));
        match two.as_deref() {
            Some("Attack Reaction") => {
                ty = ty | Types::ATTACK_REACTION;
                i += 2;
                continue;
            }
            Some("Defense Reaction") => {
                ty = ty | Types::DEFENSE_REACTION;
                i += 2;
                continue;
            }
            _ => {}
        }
        match words[i] {
            "Warrior" => sup = sup | Supertypes::WARRIOR,
            "Action" => ty = ty | Types::ACTION,
            "Instant" => ty = ty | Types::INSTANT,
            "Equipment" => ty = ty | Types::EQUIPMENT,
            "Weapon" => ty = ty | Types::WEAPON,
            "Hero" => ty = ty | Types::HERO,
            "Token" => ty = ty | Types::TOKEN,
            "Block" => ty = ty | Types::BLOCK,
            w => panic!("ligne de type : mot inconnu {w} ({s})"),
        }
        i += 1;
    }
    for w in right.split_whitespace() {
        sub = sub
            | match w {
                "Young" => Subtypes::YOUNG,
                "Head" => Subtypes::HEAD,
                "Chest" => Subtypes::CHEST,
                "Arms" => Subtypes::ARMS,
                "Legs" => Subtypes::LEGS,
                "Aura" => Subtypes::AURA,
                "Attack" => Subtypes::ATTACK,
                "Sword" => Subtypes::SWORD,
                "Axe" => Subtypes::AXE,
                "(2H)" => Subtypes::TWO_HAND,
                "(1H)" => Subtypes::ONE_HAND,
                w => panic!("ligne de type : sous-type inconnu {w} ({s})"),
            };
    }
    (sup, ty, sub)
}

fn target_spec(t: TargetKind) -> TargetSpec {
    match t {
        TargetKind::Attack => TARGET_ATTACK,
        TargetKind::WeaponAttack => TARGET_WEAPON_ATTACK,
        TargetKind::SwordYouControl => TARGET_SWORD_YOU_CONTROL,
    }
}

fn engine_keyword(k: &Keyword) -> Keywords {
    match k {
        Keyword::GoAgain => Keywords::GO_AGAIN,
        Keyword::Temper => Keywords::TEMPER,
        Keyword::Battleworn => Keywords::BATTLEWORN,
        Keyword::Dominate => Keywords::DOMINATE,
        Keyword::Specialization(_) => Keywords::NONE,
    }
}

/// Instructions d'un bloc de phrases.
fn ops(prog: ProgramId, file: &File, lines: &[Line], natives: NativeFn, card: &str) -> Vec<Op> {
    let mut out = Vec::new();
    for l in lines {
        let body: Vec<Op> = match &file.stmts[l.stmt as usize] {
            Stmt::Native { name, .. } => match natives(name) {
                Some(Native::Ops(v)) => v,
                _ => panic!("{card} : morceau Rust `{name}` introuvable (Ops)"),
            },
            _ => vec![Op::Run(exec, arg(prog, l.stmt))],
        };
        if let Some(c) = l.cond {
            out.push(Op::IfArg(check, arg(prog, c), body.len() as u8));
        }
        out.extend(body);
    }
    out
}

/// Libellé (français) d'une capacité activée, pour l'interface.
fn activated_label(a: &Activated, file: &File) -> &'static str {
    if a.attack {
        return "Attaquer";
    }
    match a.lines.first().map(|l| &file.stmts[l.stmt as usize]) {
        Some(Stmt::SharpenTarget) => "Aiguiser une épée",
        Some(Stmt::GainResources(_)) if a.destroy_this => "Détruire : gagner 1 ressource",
        _ => "Activer",
    }
}

fn build(prog: ProgramId, card: &CardSpec, color: Option<Color>, natives: NativeFn) -> CardDef {
    let file = program(prog);
    let ci = match color {
        Some(Color::Yellow) => 1,
        Some(Color::Blue) => 2,
        _ => 0,
    };
    let (sup, ty, sub) = typebox(&card.typebox);
    let st = card.stats;
    let v = |x: Option<ByColor>| x.map(|a| a[ci] as u8);
    let mut d = if let Some(c) = color {
        let mut d = deck_card(
            &card.name,
            c,
            sup,
            ty,
            v(st.cost),
            v(st.power),
            v(st.defense),
        );
        d.subtypes = d.subtypes | sub;
        d
    } else if ty.has(Types::EQUIPMENT) {
        equipment(&card.name, sup, sub, v(st.defense).unwrap_or(0))
    } else {
        CardDef {
            name: card.name.clone(),
            power: v(st.power),
            defense: v(st.defense),
            types: ty,
            subtypes: sub,
            supertypes: sup,
            ..Default::default()
        }
    };
    d.life = st.life;
    d.intellect = st.intellect;
    if let Some(t) = card.target {
        d.targets = vec![target_spec(t)];
    }
    for item in &card.items {
        match item {
            Item::Keyword(k) => d.keywords = d.keywords | engine_keyword(k),
            Item::PlayOnlyIf(PlayCond::AttackActionDefending) => {
                d.play_condition = Some(|g, _, _| {
                    g.active_link().is_some_and(|l| {
                        l.defenders.iter().any(|&x| {
                            let def = g.def(x);
                            g.obj(x).zone == Zone::CombatChain
                                && def.types.has(Types::ACTION)
                                && def.subtypes.has(Subtypes::ATTACK)
                        })
                    })
                })
            }
            Item::Resolve(lines) => d.resolve = ops(prog, file, lines, natives, &card.name),
            Item::Activated(a) => d.activated.push(ActivatedDef {
                label: activated_label(a, file),
                speed: match a.speed {
                    Speed::Action => ESpeed::Action,
                    Speed::Instant => ESpeed::Instant,
                },
                cost: a.cost,
                tap: a.tap,
                once_per_turn: a.once_per_turn,
                attack: a.attack,
                go_again: a.go_again,
                condition: a.only_if.map(|c| match c {
                    ActivateCond::SharpenedSwordThisTurn => {
                        (|g, p, _| g.player_sharpened_this_turn(p, is_sword))
                            as fn(&GameState, fab_engine::PlayerId, ObjectId) -> bool
                    }
                }),
                targets: a.target.map(target_spec).into_iter().collect(),
                effect_cost: if a.destroy_this {
                    vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))]
                } else {
                    vec![]
                },
                script: ops(prog, file, &a.lines, natives, &card.name),
                ..Default::default()
            }),
            Item::Trigger { event, lines, .. } => d.triggers.push(TriggerDef {
                when: match event {
                    EventKind::DefendsWithHandCard => |_, me, e| {
                        matches!(e, Event::Defend { card, with_hand_card: true, .. } if *card == me)
                    },
                },
                anywhere: false,
                targets: vec![],
                script: ops(prog, file, lines, natives, &card.name),
            }),
            Item::NativeTrigger { name, .. } => match natives(name) {
                Some(Native::Trigger(t)) => d.triggers.push(t),
                _ => panic!("{} : morceau Rust `{name}` introuvable (Trigger)", card.name),
            },
            Item::NativeStatic { name, .. } => match natives(name) {
                Some(Native::Static(f)) => d.aura = Some(f),
                _ => panic!("{} : morceau Rust `{name}` introuvable (Static)", card.name),
            },
        }
    }
    d
}

// ----------------------------------------------------------------------
// Exécution
// ----------------------------------------------------------------------

fn attack_filter(f: AttackFilter) -> Option<fn(&GameState, ObjectId) -> bool> {
    match f {
        AttackFilter::Any => None,
        AttackFilter::Weapon => Some(|g, a| g.is_weapon_attack(a)),
        AttackFilter::Axe => Some(|g, a| g.def(a).subtypes.has(Subtypes::AXE)),
        AttackFilter::Sword => Some(is_sword_attack),
    }
}

fn keyword_delta(k: &Keyword) -> Delta {
    grant(engine_keyword(k))
}

/// Exécute la phrase `arg`.
fn exec(g: &mut GameState, c: &mut Ctx, arg: u16) {
    let (file, i) = file_of(arg);
    let ci = color_index(g, c);
    let p = c.controller;
    match &file.stmts[i] {
        Stmt::NextAttack { filter, gets } => {
            let delta = match gets {
                Gets::Power(v) => power(v[ci]),
                Gets::Keyword(k) => keyword_delta(k),
            };
            g.next_attack(p, attack_filter(*filter), delta, false);
        }
        Stmt::TargetGets {
            power: v,
            instead_if_reprise,
        } => {
            if let Some(r) = target_obj(c, 0) {
                let n = match instead_if_reprise {
                    Some(m) if reprise(g) => m[ci],
                    _ => v[ci],
                };
                g.modify(r, power(n), Until::EndOfTurn);
            }
        }
        Stmt::SharpenTarget => {
            if let Some(r) = target_obj(c, 0) {
                g.sharpen(p, r.id);
            }
        }
        Stmt::TargetNextAttackGets(k) => {
            if let Some(r) = target_obj(c, 0) {
                g.next_attack_with(p, r, keyword_delta(k));
            }
        }
        Stmt::TargetNextAttackCostsLess(n) => {
            if let Some(r) = target_obj(c, 0) {
                g.add_weapon_effect(r, WeaponEffectKind::CostLess(*n));
            }
        }
        Stmt::CreateToken(name) => {
            let t = card_named(g, name);
            g.create_tokens(p, t, 1);
        }
        Stmt::DealDamageAttackingHero(n) => {
            if let Some(attacker) = g.active_link().map(|l| l.attacker) {
                g.deal_damage(c.source_id(), p, attacker, *n as i32, false);
            }
        }
        Stmt::GainResources(n) => g.gain_resources(p, *n),
        Stmt::ThisGetsDefense(n) => {
            if let Some(s) = c.source {
                let d = Delta {
                    defense: *n,
                    ..Default::default()
                };
                g.modify(s, d, Until::EndOfTurn);
            }
        }
        Stmt::Native { .. } => unreachable!("un morceau natif est compilé à part"),
    }
}

/// Évalue la condition `arg`.
fn check(g: &GameState, c: &Ctx, arg: u16) -> bool {
    let (file, i) = file_of(arg);
    let ci = color_index(g, c);
    match file.conds[i] {
        Cond::AttackedWithWeaponThisTurn => {
            let p = c.controller;
            // L'événement « attack » d'une attaque-proxy désigne l'arme.
            g.turn_events().iter().any(|e| {
                matches!(e, Event::Attack { attack, attacker, .. }
                    if *attacker == p && g.def(*attack).types.has(Types::WEAPON))
            })
        }
        Cond::Reprise => reprise(g),
        Cond::TargetCountersAtLeast(v) => target_obj(c, 0)
            .filter(|&r| g.is_current(r))
            .is_some_and(|r| g.obj(r.id).power_counters as i16 >= v[ci]),
        Cond::DefendsWeaponAttack => {
            // La carte va défendre l'attaque du maillon actif (CR 5.3.2b,
            // 7.4.2d) : condition vérifiée à la résolution (CR 5.3.4).
            g.active_link()
                .is_some_and(|l| l.source.is_some_and(|w| g.def(w).types.has(Types::WEAPON)))
        }
    }
}
