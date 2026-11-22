//! Cartes **fictives** pour tester le moteur de règles.
//!
//! Elles n'existent pas dans le vrai jeu : leurs noms sont en français et
//! préfixés « Essai ». Chacune reproduit un mécanisme rencontré dans les
//! vraies cartes des 9 decks (indiqué en commentaire), pour tester le moteur
//! avant de coder les vraies cartes.

use fab_engine::card::{
    AdditionalCost, CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, Types,
};
use fab_engine::replace::{ReplaceKind, Replacement, ReplacementDef};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, CardId, Choice, Event, GameState, Loadout, ObjectId, PlayerId, Prevention,
    TriggerDef, Until, Zone,
};

fn pitch_of(c: Color) -> u8 {
    match c {
        Color::Red => 1,
        Color::Yellow => 2,
        Color::Blue => 3,
    }
}

fn deck_card(
    name: &str,
    color: Color,
    types: Types,
    cost: u8,
    power: Option<u8>,
    defense: Option<u8>,
) -> CardDef {
    CardDef {
        name: name.into(),
        color: Some(color),
        pitch: Some(pitch_of(color)),
        cost: Some(cost),
        power,
        defense,
        types,
        subtypes: if power.is_some() {
            Subtypes::ATTACK
        } else {
            Subtypes::NONE
        },
        ..Default::default()
    }
}

const COLORS: [Color; 3] = [Color::Red, Color::Yellow, Color::Blue];

fn delta_power(n: i16) -> Delta {
    Delta {
        power: n,
        ..Default::default()
    }
}

/// Le héros adverse du contrôleur.
fn opposing_hero(_: &GameState, c: &Ctx, t: Target) -> bool {
    t == Target::Player(c.controller.opponent())
}

/// N'importe quel héros.
fn any_hero(_: &GameState, _: &Ctx, t: Target) -> bool {
    matches!(t, Target::Player(_))
}

/// L'attaque du maillon actif.
fn active_attack(g: &GameState, _: &Ctx, t: Target) -> bool {
    matches!(t, Target::Object(r) if g.active_attack() == Some(r.id))
}

/// Déclencheur : « when this hits ».
fn when_this_hits(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Hit { attack, .. } if *attack == me)
}

/// Déclencheur : « when this defends ».
fn when_this_defends(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Defend { card, .. } if *card == me)
}

/// Déclencheur : « at the beginning of your action phase ».
fn at_your_action_phase(g: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::BeginActionPhase(p) if *p == g.obj(me).controller)
}

/// Construit la base des cartes d'essai.
pub fn build() -> CardDb {
    let mut db = CardDb::new();
    add_to(&mut db);
    db
}

/// Ajoute les cartes d'essai à une base (ex. : base des vraies cartes, pour
/// les tests qui mélangent vraies cartes et cartes d'essai).
pub fn add_to(db: &mut CardDb) {
    let hero = |name: &str| CardDef {
        name: name.into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        ..Default::default()
    };
    db.add(hero("Essai Héros"));
    // Comme Ira : « Your second attack each turn gets +1{p} ».
    let mut ninja = hero("Essai Ninja");
    ninja.supertypes = Supertypes::NINJA;
    ninja.aura = Some(|g, src, target| {
        let me = g.obj(src).controller;
        let is_second = g.obj(target).zone == Zone::CombatChain
            && g.active_attack() == Some(target)
            && g.obj(target).controller == me
            && g.attacks_this_turn(me) == 2;
        if is_second {
            delta_power(1)
        } else {
            Delta::default()
        }
    });
    db.add(ninja);

    for (i, &c) in COLORS.iter().enumerate() {
        let i = i as u8;
        // Attaque simple : coût 1, puissance 4/3/2, défense 3.
        db.add(deck_card(
            "Essai Frappe",
            c,
            Types::ACTION,
            1,
            Some(4 - i),
            Some(3),
        ));
        // Attaque avec go again : coût 0, puissance 3/2/1.
        let mut d = deck_card("Essai Assaut", c, Types::ACTION, 0, Some(3 - i), Some(2));
        d.keywords = Keywords::GO_AGAIN;
        db.add(d);
        // Attaque avec dominate : coût 2, puissance 6/5/4.
        let mut d = deck_card(
            "Essai Écrasement",
            c,
            Types::ACTION,
            2,
            Some(6 - i),
            Some(3),
        );
        d.keywords = Keywords::DOMINATE;
        db.add(d);
        // « When this hits, gain 1{h} ».
        let mut d = deck_card("Essai Morsure", c, Types::ACTION, 1, Some(3 - i), Some(2));
        d.triggers = vec![TriggerDef {
            when: when_this_hits,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
        }];
        db.add(d);
        // Action non-attaque : go again, prochaine attaque +3/+2/+1.
        let mut d = deck_card("Essai Élan", c, Types::ACTION, 0, None, Some(2));
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(match i {
            0 => |g: &mut GameState, c: &mut Ctx| {
                g.next_attack(c.controller, None, delta_power(3), false)
            },
            1 => |g: &mut GameState, c: &mut Ctx| {
                g.next_attack(c.controller, None, delta_power(2), false)
            },
            _ => |g: &mut GameState, c: &mut Ctx| {
                g.next_attack(c.controller, None, delta_power(1), false)
            },
        })];
        db.add(d);
        // Réaction d'attaque : « Target attack gets +3/+2/+1{p} ».
        let mut d = deck_card("Essai Riposte", c, Types::ATTACK_REACTION, 0, None, Some(2));
        d.targets = vec![TargetSpec {
            label: "attaque",
            filter: active_attack,
            optional: false,
        }];
        d.resolve = vec![Op::Do(match i {
            0 => |g: &mut GameState, c: &mut Ctx| buff_target(g, c, 3),
            1 => |g: &mut GameState, c: &mut Ctx| buff_target(g, c, 2),
            _ => |g: &mut GameState, c: &mut Ctx| buff_target(g, c, 1),
        })];
        db.add(d);
        // Réaction de défense : défense 4/3/2.
        db.add(deck_card(
            "Essai Parade",
            c,
            Types::DEFENSE_REACTION,
            0,
            None,
            Some(4 - i),
        ));
        // Instant : 1 dégât au héros adverse ciblé.
        let mut d = deck_card("Essai Étincelle", c, Types::INSTANT, 1, None, None);
        d.targets = vec![TargetSpec {
            label: "héros adverse",
            filter: opposing_hero,
            optional: false,
        }];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(Target::Player(p)) = c.target(0) {
                g.deal_damage(c.source_id(), c.controller, p, 1, false);
            }
        })];
        db.add(d);
    }

    // Comme Hurl / Staunch Response : coût additionnel optionnel.
    let mut d = deck_card("Essai Rage", Color::Red, Types::ACTION, 0, Some(3), Some(2));
    d.additional_cost = Some(AdditionalCost {
        resources: 1,
        optional: true,
    });
    // « If the additional cost is paid, this gets +2{p} » : le paiement est
    // noté par un compteur sur la carte (elle garde son identité sur la pile
    // et la chaîne de combat).
    d.self_mod = Some(|g, me| {
        if g.obj(me).counters == 1 {
            delta_power(2)
        } else {
            Delta::default()
        }
    });
    d.effect_cost = vec![Op::Do(|g, c| {
        if c.optional_paid {
            let me = c.source_id().unwrap();
            g.obj_mut(me).counters = 1;
        }
    })];
    db.add(d);

    // Comme Razor Reflex / Pummel : « Choose 1 ; target attack gets +2{p}, or go again ».
    let mut d = deck_card(
        "Essai Dilemme",
        Color::Red,
        Types::ATTACK_REACTION,
        0,
        None,
        Some(2),
    );
    d.modes = vec!["+2 puissance", "go again"];
    d.targets = vec![TargetSpec {
        label: "attaque",
        filter: active_attack,
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Object(r)) = c.target(0) {
            let delta = if c.mode == 0 {
                delta_power(2)
            } else {
                Delta {
                    grants: Keywords::GO_AGAIN,
                    ..Default::default()
                }
            };
            g.modify(r, delta, Until::EndOfTurn);
        }
    })];
    db.add(d);

    // Comme Wrecker Romp : « As an additional cost to play this, discard a random card ».
    let mut d = deck_card(
        "Essai Sacrifice",
        Color::Red,
        Types::ACTION,
        1,
        Some(7),
        Some(3),
    );
    d.play_condition = Some(|g, p, me| g.player(p).hand.iter().any(|&c| c != me));
    d.effect_cost = vec![Op::Do(|g, c| {
        if let Some(x) = g.discard_random(c.controller) {
            c.picked.push(x);
        }
    })];
    db.add(d);

    // Comme Sirens of Safe Harbor : « When this is put into your graveyard from anywhere, gain 1{h} ».
    let mut d = deck_card(
        "Essai Sirène",
        Color::Blue,
        Types::ACTION,
        2,
        Some(4),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: |g, me, e| matches!(e, Event::ToGraveyard { card, owner, .. } if *card == me && *owner == g.obj(me).owner),
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
    }];
    db.add(d);

    // Comme On the Horizon / Test of Vigor : bloc avec « When this defends, … ».
    let mut d = CardDef {
        name: "Essai Vigie".into(),
        color: Some(Color::Blue),
        pitch: Some(3),
        defense: Some(3),
        types: Types::BLOCK,
        ..Default::default()
    };
    d.triggers = vec![TriggerDef {
        when: when_this_defends,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_life(c.controller, 2))],
    }];
    db.add(d);

    // Comme Sift : « Put up to 2 cards from your hand on the bottom of your deck,
    // then draw that many cards. Go again ».
    let mut d = deck_card("Essai Tri", Color::Blue, Types::ACTION, 0, None, Some(3));
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![
        Op::Ask(|g, c| {
            if c.picked.len() >= 2 || g.player(c.controller).hand.is_empty() {
                return None;
            }
            let mut options: Vec<Choice> = g
                .player(c.controller)
                .hand
                .iter()
                .map(|&x| Choice::Object(x))
                .collect();
            options.push(Choice::Pass);
            Some(Ask {
                player: c.controller,
                label: "Carte de la main à mettre sous le deck (Pass : arrêter)",
                options,
            })
        }),
        Op::Do(|g, c| {
            if let Some(Choice::Object(x)) = c.answer.take() {
                g.put_on_bottom(x);
                c.picked.push(x);
                c.flag = true; // on continue la boucle
            } else {
                c.flag = false;
            }
        }),
        Op::Back(|_, c| c.flag, 2),
        Op::Do(|g, c| {
            for _ in 0..c.picked.len() {
                g.draw(c.controller);
            }
        }),
    ];
    db.add(d);

    // Comme Toe the Line : « The next time you would be dealt damage this turn, prevent 2 of that damage ».
    let mut d = deck_card("Essai Garde", Color::Yellow, Types::INSTANT, 0, None, None);
    d.resolve = vec![Op::Do(|g, c| g.prevent_next(c.controller, 2, false, None))];
    db.add(d);

    // Comme Voltic Bolt (« Deal 5 arcane damage to target hero ») en plus petit.
    let mut d = deck_card("Essai Éclair", Color::Red, Types::ACTION, 1, None, Some(2));
    d.targets = vec![TargetSpec {
        label: "héros",
        filter: any_hero,
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Player(p)) = c.target(0) {
            g.deal_damage(c.source_id(), c.controller, p, 3, true);
        }
    })];
    db.add(d);

    // Comme Arcane Polarity : « Gain 1{h}. If you've been dealt arcane damage this turn, instead gain 4{h} ».
    let mut d = deck_card("Essai Polarité", Color::Red, Types::INSTANT, 0, None, None);
    d.resolve = vec![Op::Do(|g, c| {
        let n = if g.arcane_damage_taken_this_turn(c.controller) > 0 {
            4
        } else {
            1
        };
        g.gain_life(c.controller, n);
    })];
    db.add(d);

    // Comme Timesnap Potion : objet, « Action — Destroy this: Gain 2 action points ».
    let mut d = deck_card("Essai Potion", Color::Blue, Types::ACTION, 0, None, None);
    d.subtypes = Subtypes::ITEM;
    d.activated = vec![ActivatedDef {
        label: "Détruire : 2 points d'action",
        speed: Speed::Action,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| g.gain_action_points(c.controller, 2))],
        ..Default::default()
    }];
    db.add(d);

    // Comme Harvest Season : « Go again. At the beginning of your action phase,
    // destroy this, then gain 3{h} ».
    let mut d = deck_card("Essai Aura", Color::Yellow, Types::ACTION, 1, None, Some(2));
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: at_your_action_phase,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                g.destroy(s.id);
            }
            g.gain_life(c.controller, 3);
        })],
    }];
    db.add(d);
    // Une deuxième aura différente, pour tester l'ordre des déclenchements.
    let mut d = deck_card(
        "Essai Présage",
        Color::Yellow,
        Types::ACTION,
        0,
        None,
        Some(2),
    );
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::GO_AGAIN;
    d.triggers = vec![TriggerDef {
        when: at_your_action_phase,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                g.destroy(s.id);
            }
            g.draw(c.controller);
        })],
    }];
    db.add(d);

    // Bloc : ne peut pas être joué, défense 4 (CR 8.1.12).
    db.add(CardDef {
        name: "Essai Rempart".into(),
        color: Some(Color::Blue),
        pitch: Some(3),
        defense: Some(4),
        types: Types::BLOCK,
        ..Default::default()
    });
    // Ressource : ne peut pas être jouée (CR 8.1.7).
    db.add(CardDef {
        name: "Essai Joyau".into(),
        pitch: Some(3),
        types: Types::RESOURCE,
        ..Default::default()
    });
    let weapon = |name: &str, power: u8, sub: Subtypes, cost: u8, go_again: bool| CardDef {
        name: name.into(),
        power: Some(power),
        types: Types::WEAPON,
        subtypes: sub,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost,
            once_per_turn: true,
            attack: true,
            go_again,
            ..Default::default()
        }],
        ..Default::default()
    };
    // « Once per Turn Action — {r} : Attack », 2H, puissance 3.
    db.add(weapon(
        "Essai Hache",
        3,
        Subtypes::TWO_HAND | Subtypes::AXE,
        1,
        false,
    ));
    // « Once per Turn Action — {r} : Attack. Go again », 1H, puissance 1.
    db.add(weapon(
        "Essai Dague",
        1,
        Subtypes::ONE_HAND | Subtypes::DAGGER,
        1,
        true,
    ));
    let equip = |name: &str, sub: Subtypes, def: u8, kw: Keywords| CardDef {
        name: name.into(),
        defense: Some(def),
        types: Types::EQUIPMENT,
        subtypes: sub,
        keywords: kw,
        ..Default::default()
    };
    db.add(equip(
        "Essai Casque",
        Subtypes::HEAD,
        1,
        Keywords::BLADE_BREAK,
    ));
    db.add(equip(
        "Essai Plastron",
        Subtypes::CHEST,
        2,
        Keywords::BATTLEWORN,
    ));
    db.add(equip(
        "Essai Gantelets",
        Subtypes::ARMS,
        1,
        Keywords::TEMPER,
    ));
    db.add(equip(
        "Essai Bottes",
        Subtypes::LEGS,
        1,
        Keywords::BLADE_BREAK,
    ));
    // Comme Blossom of Spring / Spellfire Cloak : « Instant — Destroy this: Gain {r} ».
    let mut b = equip("Essai Bouclier", Subtypes::CHEST, 1, Keywords::NONE);
    b.activated = vec![ActivatedDef {
        label: "Détruire : gagner 1 ressource",
        speed: Speed::Instant,
        effect_cost: vec![Op::Do(|g, c| g.destroy(c.source_id().unwrap()))],
        script: vec![Op::Do(|g, c| g.gain_resources(c.controller, 1))],
        ..Default::default()
    }];
    db.add(b);
    add_engine_cards(db);
}

/// Une carte de token (CR 8.1.8).
fn token(name: &str, sub: Subtypes) -> CardDef {
    CardDef {
        name: name.into(),
        types: Types::TOKEN,
        subtypes: sub,
        ..Default::default()
    }
}

fn card_named(g: &GameState, name: &str) -> CardId {
    g.db.find_name(name)
        .unwrap_or_else(|| panic!("carte d'essai inconnue : {name}"))
}

/// « If this would be put into your graveyard from anywhere, instead banish it »
/// (Embraforged Gauntlet, Engulfing Shadows).
fn banish_instead_of_graveyard() -> ReplacementDef {
    ReplacementDef {
        kind: ReplaceKind::ToGraveyard,
        anywhere: true,
        optional: false,
        applies: |g, w, p| p.object == w.me && w.me.is_some_and(|me| g.obj(me).owner == p.player),
        apply: |_, _, p| p.zone = Zone::Banished,
    }
}

/// Cartes d'essai de la fin de l'étape 1 : préventions au choix, remplacements,
/// alliés et spectra, perte des capacités, déclenchements de fermeture.
fn add_engine_cards(db: &mut CardDb) {
    let equip = |name: &str, sub: Subtypes, def: u8| CardDef {
        name: name.into(),
        defense: Some(def),
        types: Types::EQUIPMENT,
        subtypes: sub,
        ..Default::default()
    };
    // Comme Nullrune Hood : « Arcane Barrier 1 ».
    let mut d = equip("Essai Capuche", Subtypes::HEAD, 1);
    d.arcane_barrier = 1;
    db.add(d);
    // Comme Mask of the Swarming Claw : « Arcane Barrier 1 » et « Spellvoid » (ici 2).
    let mut d = equip("Essai Masque", Subtypes::HEAD, 0);
    d.arcane_barrier = 1;
    d.spellvoid = 2;
    db.add(d);
    // Comme Silken Shroud : « When this is destroyed, … » (ici : gain 1{h}) et « Ward 1 ».
    let mut d = equip("Essai Voile", Subtypes::HEAD, 0);
    d.ward = 1;
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Destroyed { card, .. } if *card == me),
        anywhere: true,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
    }];
    db.add(d);
    // Comme Embraforged Gauntlet : temper + « If this would be put into your
    // graveyard from anywhere, instead banish it ».
    let mut d = equip("Essai Gantelet Sombre", Subtypes::ARMS, 1);
    d.keywords = Keywords::TEMPER;
    d.replacements = vec![banish_instead_of_graveyard()];
    db.add(d);
    // Comme Smoldering Scales : « If one or more Frostbite tokens would be
    // created under your control, instead you may destroy this. »
    let mut d = equip("Essai Écailles", Subtypes::CHEST, 1);
    d.replacements = vec![ReplacementDef {
        kind: ReplaceKind::CreateToken,
        anywhere: false,
        optional: true,
        applies: |g, w, p| {
            p.player == w.controller && p.card.is_some_and(|c| g.db.get(c).name == "Essai Givre")
        },
        apply: |g, w, p| {
            if let Some(me) = w.me {
                g.destroy(me);
            }
            p.amount = 0;
        },
    }];
    db.add(d);
    // Comme Hell Hammer : « Once per Turn Action — {r}{r}: Attack. When the
    // combat chain closes, if this attacked, banish it. »
    db.add(CardDef {
        name: "Essai Marteau".into(),
        power: Some(4),
        types: Types::WEAPON,
        subtypes: Subtypes::TWO_HAND | Subtypes::HAMMER,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 2,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        triggers: vec![TriggerDef {
            when: |g, me, e| {
                matches!(e, Event::CombatChainClosed)
                    && g.combat.links.iter().any(|l| l.source == Some(me))
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                    g.banish(s.id, false);
                }
            })],
        }],
        ..Default::default()
    });
    // Comme Florian : « If you would create 1 or more aura tokens, instead
    // create that many plus 1 of each of those tokens » (sans sa condition).
    db.add(CardDef {
        name: "Essai Florian".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        replacements: vec![ReplacementDef {
            kind: ReplaceKind::CreateToken,
            anywhere: false,
            optional: false,
            applies: |g, w, p| {
                p.player == w.controller
                    && p.card
                        .is_some_and(|c| g.db.get(c).subtypes.has(Subtypes::AURA))
            },
            apply: |_, _, p| p.amount += 1,
        }],
        ..Default::default()
    });

    // Tokens.
    db.add(token("Essai Rune", Subtypes::AURA)); // comme Runechant (sans capacité)
    db.add(token("Essai Givre", Subtypes::AURA)); // comme Frostbite (sans capacité)
    let mut d = token("Essai Écran", Subtypes::AURA); // comme Spectral Shield
    d.ward = 1;
    db.add(d);
    db.add(token("Essai Cendre", Subtypes::ASH)); // comme Ash
                                                  // Comme un dragon de Dromai : allié 3{p} 2{h}, « Once per Turn Action — 0: Attack ».
    let mut d = token("Essai Dragon", Subtypes::ALLY | Subtypes::DRAGON);
    d.power = Some(3);
    d.life = Some(2);
    d.activated = vec![ActivatedDef {
        label: "Attaquer",
        speed: Speed::Action,
        once_per_turn: true,
        attack: true,
        ..Default::default()
    }];
    db.add(d);

    // Comme Read the Runes : « Create a Runechant token », go again.
    let mut d = deck_card(
        "Essai Incantation",
        Color::Yellow,
        Types::ACTION,
        0,
        None,
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![Op::Do(|g, c| {
        let rune = card_named(g, "Essai Rune");
        g.create_tokens(c.controller, rune, 1);
    })];
    db.add(d);
    // Comme Frost Spike / Iyslander : « Create a Frostbite token under target hero's control ».
    let mut d = deck_card("Essai Gel", Color::Blue, Types::INSTANT, 0, None, None);
    d.targets = vec![TargetSpec {
        label: "héros",
        filter: any_hero,
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Player(p)) = c.target(0) {
            let givre = card_named(g, "Essai Givre");
            g.create_tokens(p, givre, 1);
        }
    })];
    db.add(d);
    // Comme Invoke … / Billowing Mirage : crée un dragon allié, go again.
    let mut d = deck_card(
        "Essai Invocation",
        Color::Red,
        Types::ACTION,
        0,
        None,
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![Op::Do(|g, c| {
        let dragon = card_named(g, "Essai Dragon");
        g.create_tokens(c.controller, dragon, 1);
    })];
    db.add(d);
    // Comme Dromai (pitch rouge) : crée une Ash, go again.
    let mut d = deck_card("Essai Braise", Color::Red, Types::ACTION, 0, None, Some(2));
    d.keywords = Keywords::GO_AGAIN;
    d.resolve = vec![Op::Do(|g, c| {
        let cendre = card_named(g, "Essai Cendre");
        g.create_tokens(c.controller, cendre, 1);
    })];
    db.add(d);
    // Comme Passing Mirage : aura avec « Spectra », go again.
    let mut d = deck_card("Essai Mirage", Color::Blue, Types::ACTION, 0, None, Some(2));
    d.subtypes = Subtypes::AURA;
    d.keywords = Keywords::GO_AGAIN | Keywords::SPECTRA;
    db.add(d);
    // Comme Sand Cover : « Target ash you control gets ward 4 until end of turn ».
    let mut d = deck_card(
        "Essai Couverture",
        Color::Red,
        Types::INSTANT,
        0,
        None,
        None,
    );
    d.targets = vec![TargetSpec {
        label: "cendre",
        filter: |g, c, t| {
            matches!(t, Target::Object(r) if g.def(r.id).subtypes.has(Subtypes::ASH)
                && g.obj(r.id).controller == c.controller)
        },
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Object(r)) = c.target(0) {
            let ward = Delta {
                ward: 4,
                ..Default::default()
            };
            g.modify(r, ward, Until::EndOfTurn);
        }
    })];
    db.add(d);
    // Comme Toe the Line : « The next time you would be dealt damage this turn,
    // prevent 2 of that damage. If you prevent damage this way, … » (ici : gain 1{h}).
    let mut d = deck_card(
        "Essai Parapet",
        Color::Yellow,
        Types::INSTANT,
        0,
        None,
        None,
    );
    d.resolve = vec![Op::Do(|g, c| {
        g.add_prevention(Prevention {
            id: 0,
            player: c.controller,
            remaining: Some(2),
            arcane_only: false,
            source: None,
            source_filter: None,
            fixed: true,
            on_prevent: Some(|g, p, _| g.gain_life(p, 1)),
            until: Until::EndOfTurn,
        })
    })];
    db.add(d);
    // Comme Engulfing Shadows : réaction de défense, « If this would be put into
    // your graveyard from anywhere, instead banish it ».
    let mut d = deck_card(
        "Essai Ombre",
        Color::Red,
        Types::DEFENSE_REACTION,
        0,
        None,
        Some(3),
    );
    d.replacements = vec![banish_instead_of_graveyard()];
    db.add(d);
    // Comme Crouching Tiger : « Ephemeral » (« If this would be put into a
    // graveyard from anywhere, instead remove it from the game »), go again.
    let mut d = deck_card(
        "Essai Tigre",
        Color::Red,
        Types::ACTION,
        0,
        Some(1),
        Some(2),
    );
    d.keywords = Keywords::GO_AGAIN;
    d.replacements = vec![ReplacementDef {
        kind: ReplaceKind::ToGraveyard,
        anywhere: true,
        optional: false,
        applies: |_, w, p| p.object == w.me,
        apply: |_, _, p| p.zone = Zone::Gone,
    }];
    db.add(d);
    // Comme Humble : « When this hits a hero, that hero loses all abilities
    // until the end of their next turn ».
    let mut d = deck_card(
        "Essai Humilité",
        Color::Red,
        Types::ACTION,
        0,
        Some(3),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(Event::Hit { target, .. }) = c.event {
                let hero = g.player(target).hero;
                let lose = Delta {
                    lose_abilities: true,
                    ..Default::default()
                };
                let until = g.end_of_next_turn(target);
                g.modify(g.obj_ref(hero), lose, until);
            }
        })],
    }];
    db.add(d);
    // Comme Debilitate (sans crush) : « their first attack during their next
    // turn gets -2{p} ».
    let mut d = deck_card(
        "Essai Affaiblissement",
        Color::Red,
        Types::ACTION,
        0,
        Some(3),
        Some(2),
    );
    d.triggers = vec![TriggerDef {
        when: |_, me, e| matches!(e, Event::Hit { attack, target_obj: None, .. } if *attack == me),
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(Event::Hit { target, .. }) = c.event {
                let turn = g.next_turn_of(target);
                g.next_attack_on_turn(target, turn, None, delta_power(-2));
            }
        })],
    }];
    db.add(d);
    // Comme Stir the Aetherwinds (partie remplacement) : « la prochaine fois
    // qu'une de vos sources infligerait des dégâts arcaniques ce tour, elle
    // en inflige 3 de plus à la place ».
    let mut d = deck_card(
        "Essai Attisement",
        Color::Red,
        Types::INSTANT,
        0,
        None,
        None,
    );
    d.resolve = vec![Op::Do(|g, c| {
        g.add_replacement(Replacement {
            id: 0,
            kind: ReplaceKind::Damage,
            controller: c.controller,
            data: None,
            optional: false,
            applies: |_, w, p| p.arcane && p.player == w.controller,
            apply: |_, _, p| p.amount += 3,
            until: Until::EndOfTurn,
            uses: Some(1),
        })
    })];
    db.add(d);
    // Comme Topsy Turvy (partie effet) : « Until end of turn, if one or more
    // cards would be put on top of a deck, instead they're put on the bottom ».
    let mut d = deck_card(
        "Essai Renversement",
        Color::Blue,
        Types::INSTANT,
        0,
        None,
        None,
    );
    d.resolve = vec![Op::Do(|g, c| {
        g.add_replacement(Replacement {
            id: 0,
            kind: ReplaceKind::ToDeckTop,
            controller: c.controller,
            data: None,
            optional: false,
            applies: |_, _, _| true,
            apply: |_, _, p| p.on_top = false,
            until: Until::EndOfTurn,
            uses: None,
        })
    })];
    db.add(d);
    add_step3_cards(db);
}

/// Mécanismes ajoutés à l'étape 3 (deck Ira).
fn add_step3_cards(db: &mut CardDb) {
    // Comme Crouching Tiger : carte créée (pas un token), « Ephemeral. Go again ».
    db.add(CardDef {
        name: "Essai Fauve".into(),
        cost: Some(0),
        power: Some(1),
        types: Types::ACTION,
        subtypes: Subtypes::ATTACK,
        keywords: Keywords::EPHEMERAL | Keywords::GO_AGAIN,
        ..Default::default()
    });
    // Comme Pouncing Paws : « Create a Crouching Tiger in your banished zone.
    // You may play it this turn. »
    let mut d = deck_card("Essai Appel", Color::Blue, Types::INSTANT, 0, None, None);
    d.resolve = vec![Op::Do(|g, c| {
        let f = card_named(g, "Essai Fauve");
        let id = g.create_card_banished(c.controller, f);
        g.permit_play(c.controller, id, g.turn);
    })];
    db.add(d);
    // Comme Tiger Eye Reflex : « Ambush ».
    let mut d = deck_card(
        "Essai Embuscade",
        Color::Blue,
        Types::BLOCK,
        0,
        None,
        Some(3),
    );
    d.cost = None;
    d.keywords = Keywords::AMBUSH;
    db.add(d);
    // Comme Legacy of Ikaru : « Target attack gets "When this hits, gain 1{h}." »
    let mut d = deck_card(
        "Essai Héritage",
        Color::Blue,
        Types::ATTACK_REACTION,
        0,
        None,
        Some(3),
    );
    d.targets = vec![TargetSpec {
        label: "attaque",
        filter: active_attack,
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        if let Some(Target::Object(r)) = c.target(0) {
            g.grant_trigger(r, c.card, 0, Until::EndOfTurn);
        }
    })];
    d.granted = vec![TriggerDef {
        when: when_this_hits,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.gain_life(c.controller, 1))],
    }];
    db.add(d);
    // Comme Razor Reflex : « Choose 1 ; target weapon attack gets +3{p} ;
    // or target attack action card gets +1{p} » (cible selon le mode).
    let mut d = deck_card(
        "Essai Alternative",
        Color::Red,
        Types::ATTACK_REACTION,
        0,
        None,
        Some(2),
    );
    d.modes = vec!["attaque d'arme +3", "carte d'attaque +1"];
    d.targets = vec![TargetSpec {
        label: "attaque",
        filter: |g, c, t| {
            let Target::Object(r) = t else { return false };
            g.active_attack() == Some(r.id) && (g.is_weapon_attack(r.id) == (c.mode == 0))
        },
        optional: false,
    }];
    d.resolve = vec![Op::Do(|g, c| {
        let n = if c.mode == 0 { 3 } else { 1 };
        buff_target(g, c, n);
    })];
    db.add(d);
}

fn buff_target(g: &mut GameState, c: &mut Ctx, n: i16) {
    if let Some(Target::Object(r)) = c.target(0) {
        g.modify(r, delta_power(n), Until::EndOfTurn);
    }
}

/// Raccourci : identifiant d'une carte d'essai (panique si elle n'existe pas).
pub fn id(db: &CardDb, name: &str, color: Option<Color>) -> CardId {
    db.find(name, color)
        .unwrap_or_else(|| panic!("carte d'essai inconnue : {name} {color:?}"))
}

/// Deck d'essai « agressif » : 40 cartes, Essai Hache + 4 équipements.
pub fn aggro_loadout(db: &CardDb) -> Loadout {
    let mut deck = Vec::new();
    for c in COLORS {
        for (name, n) in [
            ("Essai Frappe", 3),
            ("Essai Assaut", 3),
            ("Essai Écrasement", 2),
            ("Essai Morsure", 2),
            ("Essai Élan", 1),
            ("Essai Riposte", 1),
            ("Essai Étincelle", 1),
        ] {
            for _ in 0..n {
                deck.push(id(db, name, Some(c)));
            }
        }
    }
    deck.truncate(34);
    for (name, c) in [
        ("Essai Rage", Color::Red),
        ("Essai Dilemme", Color::Red),
        ("Essai Sacrifice", Color::Red),
        ("Essai Potion", Color::Blue),
        ("Essai Aura", Color::Yellow),
        ("Essai Présage", Color::Yellow),
    ] {
        deck.push(id(db, name, Some(c)));
    }
    Loadout {
        hero: id(db, "Essai Ninja", None),
        arena: vec![
            id(db, "Essai Hache", None),
            id(db, "Essai Casque", None),
            id(db, "Essai Plastron", None),
            id(db, "Essai Gantelets", None),
            id(db, "Essai Bottes", None),
        ],
        deck,
    }
}

/// Deck d'essai « défensif » : réactions de défense, blocs, Essai Dague.
pub fn defensive_loadout(db: &CardDb) -> Loadout {
    let mut deck = Vec::new();
    for c in COLORS {
        for (name, n) in [
            ("Essai Frappe", 2),
            ("Essai Assaut", 2),
            ("Essai Morsure", 2),
            ("Essai Parade", 3),
            ("Essai Élan", 1),
        ] {
            for _ in 0..n {
                deck.push(id(db, name, Some(c)));
            }
        }
    }
    for (name, c, n) in [
        ("Essai Rempart", Some(Color::Blue), 1),
        ("Essai Joyau", None, 1),
        ("Essai Sirène", Some(Color::Blue), 2),
        ("Essai Vigie", Some(Color::Blue), 2),
        ("Essai Tri", Some(Color::Blue), 1),
        ("Essai Garde", Some(Color::Yellow), 1),
        ("Essai Éclair", Some(Color::Red), 1),
        ("Essai Polarité", Some(Color::Red), 1),
    ] {
        for _ in 0..n {
            deck.push(id(db, name, c));
        }
    }
    assert_eq!(deck.len(), 40);
    Loadout {
        hero: id(db, "Essai Héros", None),
        arena: vec![
            id(db, "Essai Dague", None),
            id(db, "Essai Casque", None),
            id(db, "Essai Bouclier", None),
            id(db, "Essai Gantelets", None),
            id(db, "Essai Bottes", None),
        ],
        deck,
    }
}

/// Decks d'essai qui utilisent les mécanismes de la fin de l'étape 1
/// (préventions au choix, remplacements, alliés, spectra, perte des
/// capacités, fermeture de chaîne), pour les tests aléatoires massifs.
pub fn mechanics_loadouts(db: &CardDb) -> [Loadout; 2] {
    let fill = |list: &[(&str, Color, usize)]| {
        let mut deck = Vec::new();
        for &(name, c, n) in list {
            for _ in 0..n {
                deck.push(id(db, name, Some(c)));
            }
        }
        while deck.len() < 40 {
            deck.push(id(db, "Essai Frappe", Some(Color::Blue)));
        }
        deck.truncate(40);
        deck
    };
    let p0 = fill(&[
        ("Essai Incantation", Color::Yellow, 3),
        ("Essai Invocation", Color::Red, 3),
        ("Essai Gel", Color::Blue, 3),
        ("Essai Humilité", Color::Red, 3),
        ("Essai Affaiblissement", Color::Red, 3),
        ("Essai Tigre", Color::Red, 3),
        ("Essai Attisement", Color::Red, 2),
        ("Essai Éclair", Color::Red, 3),
        ("Essai Renversement", Color::Blue, 2),
        ("Essai Frappe", Color::Red, 4),
        ("Essai Assaut", Color::Red, 4),
        ("Essai Riposte", Color::Red, 2),
    ]);
    let p1 = fill(&[
        ("Essai Ombre", Color::Red, 3),
        ("Essai Mirage", Color::Blue, 3),
        ("Essai Braise", Color::Red, 3),
        ("Essai Couverture", Color::Red, 3),
        ("Essai Parapet", Color::Yellow, 3),
        ("Essai Gel", Color::Blue, 2),
        ("Essai Invocation", Color::Red, 3),
        ("Essai Éclair", Color::Red, 3),
        ("Essai Garde", Color::Yellow, 2),
        ("Essai Parade", Color::Red, 3),
        ("Essai Morsure", Color::Red, 4),
        ("Essai Assaut", Color::Yellow, 4),
    ]);
    [
        Loadout {
            hero: id(db, "Essai Florian", None),
            arena: vec![
                id(db, "Essai Marteau", None),
                id(db, "Essai Masque", None),
                id(db, "Essai Écailles", None),
                id(db, "Essai Gantelet Sombre", None),
                id(db, "Essai Bottes", None),
            ],
            deck: p0,
        },
        Loadout {
            hero: id(db, "Essai Ninja", None),
            arena: vec![
                id(db, "Essai Dague", None),
                id(db, "Essai Voile", None),
                id(db, "Essai Plastron", None),
                id(db, "Essai Gantelets", None),
            ],
            deck: p1,
        },
    ]
}

/// Tous les joueurs du jeu d'essai (pour les tests).
pub const PLAYERS: [PlayerId; 2] = PlayerId::BOTH;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_decks_d_essai_font_40_cartes() {
        let db = build();
        assert_eq!(aggro_loadout(&db).deck.len(), 40);
        assert_eq!(defensive_loadout(&db).deck.len(), 40);
        for l in mechanics_loadouts(&db) {
            assert_eq!(l.deck.len(), 40);
        }
    }
}
