//! Cartes Warrior des decks Hala et Dorinthea.
//!
//! Les cartes sont écrites dans le **langage de cartes**
//! (`crates/cards/cartes/warrior.cartes`, voir `docs/langage-cartes.md`).
//! Ce module ne garde que les morceaux écrits en Rust (lignes `native…` du
//! fichier), chacun précédé du texte officiel qu'il code.

use super::{card_named, grant, is_sword, target_obj};
use crate::langage::{self, Native, ProgramId};
use fab_engine::card::{CardDb, Delta, Keywords, TriggerDef, Types};
use fab_engine::replace::{ReplaceKind, Replacement};
use fab_engine::script::{Ask, Op, Target, TargetSpec};
use fab_engine::{Choice, Event, GameState, PlayerId, Prevention, Until, WeaponEffectKind, Zone};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Hala",
    "Dorinthea",
    "Zenith Blade",
    "Decimator Great Axe",
    "Helm of Unity",
    "Plating of Unity",
    "Gauntlets of Unity",
    "Pillar of Unity",
    "Edge Laden Plate",
    "Refraction Bolters",
    "Flurry",
    "Fatal Engagement",
    "Felling Swing",
    "Hit and Run",
    "Ironsong Response",
    "Overpower",
    "Sharpen Steel",
    "Steelblade Shunt",
    "Display of Craftsmanship",
    "Cut n' Carve",
    "Edict of Steel",
    "Sharp Incline",
    "Swordmaster's Path",
    "Toe the Line",
    "Shove Off",
];

pub fn add(db: &mut CardDb) {
    langage::add(db, ProgramId::Warrior, native);
}

/// Le token Flurry, créé par Edict of Steel et Toe the Line.
fn flurry(g: &mut GameState, p: PlayerId) {
    let f = card_named(g, "Flurry");
    g.create_tokens(p, f, 1);
}

/// Les morceaux écrits en Rust, par nom.
fn native(name: &str) -> Option<Native> {
    Some(match name {
        // Dorinthea : « The first time your weapon attack hits each turn, you
        // may attack an additional time with that weapon this turn. »
        "dorinthea" => Native::Trigger(TriggerDef {
            when: |g, me, e| {
                let p = g.obj(me).controller;
                let weapon_hit = |e: &Event| {
                    matches!(e, Event::Hit { attack, attacker, .. }
                        if *attacker == p && g.def(*attack).types.has(Types::WEAPON))
                };
                // CR 6.6.5d : seulement la première fois du tour (l'événement
                // courant est déjà dans le journal).
                weapon_hit(e) && g.turn_events().iter().filter(|x| weapon_hit(x)).count() == 1
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                // « you may » : une permission, pas de question (CR 5.2.3c).
                if let Some(Event::Hit { attack, .. }) = c.event {
                    let w = g.obj_ref(attack);
                    g.add_weapon_effect(w, WeaponEffectKind::AdditionalAttack);
                }
            })],
        }),

        // Zenith Blade : « If this has been sharpened this turn, its first
        // attack this turn gets go again. »
        "zenith_blade" => Native::Static(|g, src, target| {
            let o = g.obj(target);
            // `counters` d'une proxy : numéro de l'attaque de l'arme ce tour.
            if o.proxy_of == Some(src) && o.counters == 1 && g.sharpened_this_turn(src) {
                grant(Keywords::GO_AGAIN)
            } else {
                Delta::default()
            }
        }),

        // Decimator Great Axe : « The first time this is defended by a
        // non-equipment card each turn, halve the base {d} of target defending
        // card, rounded up, until end of turn. »
        "decimator" => Native::Trigger(TriggerDef {
            when: |g, me, e| {
                let defended_by_card = |e: &Event| {
                    matches!(e, Event::Defend { card, attack, .. }
                        if g.obj(*attack).proxy_of == Some(me)
                            && !g.def(*card).types.has(Types::EQUIPMENT))
                };
                defended_by_card(e)
                    && g.turn_events()
                        .iter()
                        .filter(|x| defended_by_card(x))
                        .count()
                        == 1
            },
            anywhere: false,
            targets: vec![TargetSpec {
                label: "carte qui défend",
                // CR 7.0.5a : une carte défend tant qu'elle est sur la chaîne.
                // Notes de Dusk till Dawn : n'importe quelle carte qui défend sur
                // la chaîne, pas seulement sur ce maillon, équipements compris ;
                // jusqu'à la fin du tour (Errata Bulletin 7).
                filter: |g, _, t| {
                    matches!(t, Target::Object(r) if g.obj(r.id).zone == Zone::CombatChain
                        && g.combat.links.iter().any(|l| l.defenders.contains(&r.id)))
                },
                optional: false,
                in_graveyard: false,
                in_banished: false,
            }],
            script: vec![Op::Do(|g, c| {
                if let Some(r) = target_obj(c, 0) {
                    // Défense de base divisée par 2, arrondie au supérieur :
                    // on retire la moitié arrondie à l'inférieur.
                    // TODO(règles) : équivalent tant qu'aucun effet ne change la
                    // défense de base (aucun dans les 9 decks).
                    let base = g.def(r.id).defense.unwrap_or(0) as i16;
                    let d = Delta {
                        defense: -(base / 2),
                        ..Default::default()
                    };
                    g.modify(r, d, Until::EndOfTurn);
                }
            })],
        }),

        // Refraction Bolters : « When a weapon attack you control hits, you
        // may destroy this. If you do, the attack gets go again. »
        "refraction_bolters" => Native::Trigger(TriggerDef {
            when: |g, me, e| {
                matches!(e, Event::Hit { attack, attacker, .. }
                    if *attacker == g.obj(me).controller && g.def(*attack).types.has(Types::WEAPON))
            },
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Ask(|g, c| {
                    c.source.filter(|&s| g.is_current(s)).map(|_| Ask {
                        player: c.controller,
                        label: "Détruire cette carte pour donner go again à l'attaque ?",
                        options: vec![Choice::Yes, Choice::No],
                    })
                }),
                Op::If(|_, c| c.answer == Some(Choice::Yes), 1),
                Op::Do(|g, c| {
                    g.destroy(c.source_id().unwrap());
                    // « the attack » : l'attaque (proxy) de l'arme qui a touché.
                    if let Some(Event::Hit { attack, .. }) = c.event {
                        let link = g
                            .combat
                            .links
                            .iter()
                            .rev()
                            .find(|l| l.source == Some(attack));
                        if let Some(a) = link.map(|l| l.attack) {
                            g.modify(a, grant(Keywords::GO_AGAIN), Until::EndOfTurn);
                        }
                    }
                }),
            ],
        }),

        // Flurry : « When you activate a weapon attack, destroy this and you
        // may attack with the weapon twice this turn. »
        "flurry" => Native::Trigger(TriggerDef {
            when: |g, me, e| match *e {
                Event::Activated {
                    player,
                    source,
                    ability,
                } => {
                    let def = g.def(source);
                    player == g.obj(me).controller
                        && def.types.has(Types::WEAPON)
                        && def.activated[ability as usize].attack
                }
                _ => false,
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.destroy(s.id);
                }
                if let Some(Event::Activated { source, .. }) = c.event {
                    // CR 5.2.3e : fixe la limite à 2.
                    let w = g.obj_ref(source);
                    g.add_weapon_effect(w, WeaponEffectKind::AttackTwice);
                }
            })],
        }),

        // Display of Craftsmanship : « If the weapon has been sharpened this
        // turn, put a +1{p} counter on it. »
        "display_counter" => Native::Ops(vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                if let Some(w) = g.obj(r.id).proxy_of {
                    if g.sharpened_this_turn(w) && g.obj(w).zone.in_arena() {
                        // Ce compteur est retiré au début de la phase de fin
                        // par le sharpen de l'arme (« remove all +1{p} counters »).
                        g.obj_mut(w).power_counters += 1;
                    }
                }
            }
        })]),

        // Swordmaster's Path : « The next time you would sharpen a sword this
        // turn, instead sharpen it an additional time. »
        "swordmaster_path" => Native::Ops(vec![Op::Do(|g, c| {
            g.add_replacement(Replacement {
                id: 0,
                kind: ReplaceKind::Sharpen,
                controller: c.controller,
                data: None,
                bind: None,
                value: 0,
                optional: false,
                applies: |g, w, p| {
                    p.player == w.controller && p.object.is_some_and(|o| is_sword(g, o))
                },
                apply: |_, _, p| p.amount += 1,
                until: Until::EndOfTurn,
                uses: Some(1),
            });
        })]),

        // Toe the Line : « The next time you would be dealt damage this turn,
        // prevent 2 of that damage. If you prevent damage this way, create a
        // Flurry token. »
        "toe_the_line" => Native::Ops(vec![Op::Do(|g, c| {
            g.add_prevention(Prevention {
                id: 0,
                player: c.controller,
                remaining: Some(2),
                arcane_only: false,
                source: None,
                source_filter: None,
                fixed: true,
                on_prevent: Some(|g, p, _| flurry(g, p)),
                until: Until::EndOfTurn,
            })
        })]),

        // Shove Off : « Choose a non-equipment card defending a sword attack
        // you control. Return it to its owner's hand. »
        "shove_off" => Native::Ops(vec![
            Op::Ask(|g, c| {
                // CR 7.0.5a : une carte défend tant qu'elle est sur la chaîne, contre
                // l'attaque de son maillon : les cartes des maillons précédents
                // comptent, si l'épée est encore sur ce maillon (notes de Mastery
                // Pack Warrior). Quand l'épée attaque de nouveau, son ancienne
                // attaque-proxy cesse d'exister (CR 1.4.3c) : l'épée a quitté le
                // maillon.
                let options: Vec<Choice> = g
                    .combat
                    .links
                    .iter()
                    .filter(|l| {
                        l.attacker == c.controller
                            && l.source.is_some_and(|w| {
                                g.def(w).types.has(Types::WEAPON) && is_sword(g, w)
                            })
                            && g.is_current(l.attack)
                            && g.obj(l.attack.id).zone == Zone::CombatChain
                    })
                    .flat_map(|l| l.defenders.iter().copied())
                    .filter(|&x| {
                        g.obj(x).zone == Zone::CombatChain && !g.def(x).types.has(Types::EQUIPMENT)
                    })
                    .map(Choice::Object)
                    .collect();
                Some(Ask {
                    player: c.controller,
                    label: "Carte (hors équipement) qui défend contre votre attaque d'épée",
                    options,
                })
            }),
            Op::Do(|g, c| {
                if let Some(Choice::Object(x)) = c.answer {
                    g.return_to_hand(x);
                }
            }),
        ]),

        _ => return None,
    })
}
