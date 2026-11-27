//! Cartes Assassin et Chaos du deck Arakni, Web of Deceit, les six Agents of
//! Chaos (demi-héros que le héros peut devenir) et les tokens Graphene
//! Chelicera et Bloodrot Pox.
//!
//! Notions du deck :
//! - **stealth** (CR 8.3.24) : mot-clé sans effet propre, lu par les autres
//!   cartes (`GameState::has_stealth`) ;
//! - **mark** (CR 8.5.50, 9.3) : `GameState::mark` ; la marque disparaît
//!   quand le héros est touché par une source adverse (CR 9.3.3), au moment
//!   de la touche : « when this hits a marked hero » lit `Event::Hit::marked` ;
//! - une **dague** qui inflige des dégâts par un effet « has hit » :
//!   `GameState::deal_damage_and_hit` (événement `Hit` avec `effect`) ;
//! - **become** / **return to the brood** (CR 8.5.25, 8.5.52) :
//!   `GameState::become_random`, `GameState::return_to_brood`.

use super::generic::this_hits;
use super::shadow::{ask_object, picked};
use super::{by_color, card_named, deck_card, equipment, grant, power, target_obj, COLORS};
use fab_engine::card::{
    CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, Traits, Types,
};
use fab_engine::script::{Ask, Ctx, Op, Target, TargetSpec};
use fab_engine::{
    ActivatedDef, Choice, Event, GameState, ObjectId, PlayerId, ReplaceKind, Replacement,
    TriggerDef, Until, Zone,
};

/// Noms des cartes de ce module (pour `docs/cards/STATUS.md`).
pub const NAMES: &[&str] = &[
    "Arakni, Web of Deceit",
    "Arakni, Black Widow",
    "Arakni, Funnel Web",
    "Arakni, Orb-Weaver",
    "Arakni, Redback",
    "Arakni, Tarantula",
    "Arakni, Trap-Door",
    "Danger Digits",
    "Mark of the Huntsman",
    "Prey Spotters",
    "Stalker's Steps",
    "Topsy Turvy",
    "Graphene Chelicera",
    "Bloodrot Pox",
    "Art of Desire: Body",
    "Bite",
    "Concoct Disorder",
    "Double Trouble",
    "Hurl",
    "Infect",
    "Lair of the Spider",
    "Mark of the Black Widow",
    "Mark of the Funnel Web",
    "Mark the Prey",
    "Night's Embrace",
    "Orb-Weaver Spinneret",
    "Pick Up the Point",
    "Razor's Edge",
    "Reaper's Call",
    "Shred",
    "Stains of the Redback",
];

const AS: Supertypes = Supertypes::ASSASSIN;
const CH: Supertypes = Supertypes::CHAOS;
const AN: Supertypes = Supertypes(Supertypes::ASSASSIN.0 | Supertypes::NINJA.0);
const CHELICERA: &str = "Graphene Chelicera";
const POX: &str = "Bloodrot Pox";

// ---------------------------------------------------------------------------
// Aides
// ---------------------------------------------------------------------------

/// « When this attacks ».
fn when_this_attacks(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Attack { attack, .. } if *attack == me)
}

/// « When this attacks a hero ».
pub(crate) fn when_this_attacks_a_hero(_: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::Attack { attack, target_obj: None, .. } if *attack == me)
}

/// « When this hits a hero ».
pub(crate) fn this_hits_a_hero(g: &GameState, me: ObjectId, e: &Event) -> bool {
    this_hits(g, me, e)
        && matches!(
            e,
            Event::Hit {
                target_obj: None,
                ..
            }
        )
}

/// « When this hits a marked hero » : le héros était marqué au moment de la
/// touche (la marque disparaît pendant la touche, CR 9.3.3).
fn this_hits_a_marked_hero(g: &GameState, me: ObjectId, e: &Event) -> bool {
    this_hits_a_hero(g, me, e) && matches!(e, Event::Hit { marked: true, .. })
}

/// « At the beginning of your end phase ».
fn beginning_of_your_end_phase(g: &GameState, me: ObjectId, e: &Event) -> bool {
    matches!(e, Event::BeginEndPhase(p) if *p == g.obj(me).controller)
}

/// Le héros touché (« them ») d'une capacité « when this hits ».
fn hit_hero(c: &Ctx) -> Option<PlayerId> {
    match c.event {
        Some(Event::Hit {
            target,
            target_obj: None,
            ..
        }) => Some(target),
        _ => None,
    }
}

/// Le héros attaqué (« them ») d'une capacité « when this attacks a hero ».
fn attacked_hero(c: &Ctx) -> Option<PlayerId> {
    match c.event {
        Some(Event::Attack {
            target,
            target_obj: None,
            ..
        }) => Some(target),
        _ => None,
    }
}

/// L'objet de la chaîne qui reçoit « the attack gets … » : l'attaque du
/// maillon actif (carte ou attaque-proxy).
fn active_attack_ref(g: &GameState) -> Option<fab_engine::ObjRef> {
    g.active_link().map(|l| l.attack)
}

fn go_again() -> Delta {
    grant(Keywords::GO_AGAIN)
}

/// Une dague (arme ou carte « Dagger Attack »).
fn is_dagger(g: &GameState, id: ObjectId) -> bool {
    g.def(id).subtypes.has(Subtypes::DAGGER)
}

/// Une carte Assassin (hybride comprise, CR 2.14.1b).
fn is_assassin(g: &GameState, id: ObjectId) -> bool {
    g.def(id).supertypes.has(AS)
}

/// Les cartes Assassin de la main de `p`.
fn assassin_cards_in_hand(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    g.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&x| is_assassin(g, x))
        .collect()
}

/// « Mark target opposing hero ».
fn mark_target_hero(g: &mut GameState, c: &mut Ctx) {
    if let Some(Target::Player(p)) = c.target(0) {
        g.mark(p);
    }
}

/// Cible « target opposing hero ».
pub(crate) const TARGET_OPPOSING_HERO: TargetSpec = TargetSpec {
    label: "héros adverse",
    filter: |_, c, t| matches!(t, Target::Player(p) if p != c.controller),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target hero ».
const TARGET_HERO: TargetSpec = TargetSpec {
    label: "héros",
    filter: |_, _, t| matches!(t, Target::Player(_)),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Une dague (l'arme, pas une attaque-proxy) que le joueur `c.controller`
/// contrôle dans l'arène.
fn dagger_you_control(g: &GameState, c: &Ctx, t: Target) -> bool {
    match t {
        Target::Object(r) => {
            let o = g.obj(r.id);
            o.proxy_of.is_none()
                && o.controller == c.controller
                && o.zone.in_arena()
                && g.def(r.id).types.has(Types::WEAPON)
                && is_dagger(g, r.id)
        }
        Target::Player(_) => false,
    }
}

/// Cible « target dagger you control ».
const TARGET_DAGGER: TargetSpec = TargetSpec {
    label: "dague",
    filter: dagger_you_control,
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « you may have target dagger you control … » (Bite).
const TARGET_DAGGER_OPTIONAL: TargetSpec = TargetSpec {
    optional: true,
    ..TARGET_DAGGER
};

/// Cible « target dagger you control that isn't on the active chain link »
/// (Danger Digits) : ni l'arme de l'attaque active, ni l'attaque elle-même.
const TARGET_DAGGER_NOT_ACTIVE: TargetSpec = TargetSpec {
    label: "dague hors du maillon actif",
    filter: |g, c, t| {
        dagger_you_control(g, c, t)
            && match (t, g.active_link()) {
                (Target::Object(r), Some(l)) => l.source != Some(r.id) && l.attack.id != r.id,
                _ => true,
            }
    },
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target attack with stealth » : l'attaque active, si elle a stealth.
const TARGET_STEALTH_ATTACK: TargetSpec = TargetSpec {
    label: "attaque avec stealth",
    filter: |g, _, t| matches!(t, Target::Object(r) if g.active_attack() == Some(r.id) && g.has_stealth(r.id)),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target attack action card with stealth ».
const TARGET_STEALTH_ATTACK_CARD: TargetSpec = TargetSpec {
    label: "carte d'action d'attaque avec stealth",
    filter: |g, _, t| {
        matches!(t, Target::Object(r) if g.active_attack() == Some(r.id)
            && super::generic::is_attack_action_card(g, r.id)
            && g.has_stealth(r.id))
    },
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target Assassin attack ».
const TARGET_ASSASSIN_ATTACK: TargetSpec = TargetSpec {
    label: "attaque Assassin",
    filter: |g, _, t| matches!(t, Target::Object(r) if g.active_attack() == Some(r.id) && is_assassin(g, r.id)),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target dagger attack ».
const TARGET_DAGGER_ATTACK: TargetSpec = TargetSpec {
    label: "attaque de dague",
    filter: |g, _, t| matches!(t, Target::Object(r) if g.active_attack() == Some(r.id) && is_dagger(g, r.id)),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target card defending an Assassin attack » : une carte qui
/// défend sur le maillon actif, dont l'attaque est Assassin.
const TARGET_DEFENDING_ASSASSIN: TargetSpec = TargetSpec {
    label: "carte qui défend contre une attaque Assassin",
    filter: |g, _, t| match (t, g.active_link()) {
        (Target::Object(r), Some(l)) => l.defenders.contains(&r.id) && is_assassin(g, l.attack.id),
        _ => false,
    },
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// « Target dagger you control deals 1 damage to [héros]. If damage is
/// dealt this way, the dagger has hit. Destroy the dagger. » La dague est la
/// cible n° 0 ; le héros est donné par [`struck_hero`]. Deux instructions :
/// les dégâts peuvent attendre un choix de prévention.
fn dagger_strikes() -> [Op; 2] {
    [
        Op::Do(|g, c| {
            let dagger = target_obj(c, 0).filter(|&r| g.is_current(r));
            if let (Some(d), Some(h)) = (dagger, struck_hero(g, c)) {
                g.deal_damage_and_hit(d.id, c.controller, h, 1);
            }
        }),
        Op::Do(|g, c| {
            if let Some(d) = target_obj(c, 0).filter(|&r| g.is_current(r)) {
                g.destroy(d.id);
            }
        }),
    ]
}

/// Le héros visé par [`dagger_strikes`] : la cible n° 1 (Hurl : « target
/// hero »), sinon le héros attaqué (Bite : « them »), sinon le héros qui
/// défend sur le maillon actif (Danger Digits).
fn struck_hero(g: &GameState, c: &Ctx) -> Option<PlayerId> {
    match c.target(1) {
        Some(Target::Player(p)) => Some(p),
        _ => attacked_hero(c).or_else(|| g.active_link().map(|l| l.target)),
    }
}

/// « Discard an Assassin card » (coût des Agents of Chaos).
fn discard_an_assassin_card() -> Vec<Op> {
    vec![
        Op::Ask(|g, c| {
            ask_object(
                c.controller,
                "Défausser quelle carte Assassin ?",
                assassin_cards_in_hand(g, c.controller),
            )
        }),
        Op::Do(|g, c| {
            let v = assassin_cards_in_hand(g, c.controller);
            if let Some(x) = picked(c, &v) {
                g.discard(x);
            }
        }),
    ]
}

fn has_assassin_card(g: &GameState, p: PlayerId, _: ObjectId) -> bool {
    !assassin_cards_in_hand(g, p).is_empty()
}

/// « Equip a Graphene Chelicera token. »
fn equip_chelicera(g: &mut GameState, p: PlayerId) {
    let c = card_named(g, CHELICERA);
    g.equip_weapon_token(p, c);
}

/// « Your next attack with stealth this turn gets +N{p}. »
fn next_stealth_attack(g: &mut GameState, p: PlayerId, n: i16) {
    g.next_attack(p, Some(|g, a| g.has_stealth(a)), power(n), false);
}

/// Carte d'attaque Assassin avec stealth, coût 0, déf. 3 (puissance 3/2/1).
fn stealth_attack(name: &str, color: Color) -> CardDef {
    let mut d = deck_card(
        name,
        color,
        AS,
        Types::ACTION,
        Some(0),
        Some(pitch_power(color)),
        Some(3),
    );
    d.keywords = Keywords::STEALTH;
    d
}

/// Puissance 3 / 2 / 1 selon la couleur.
fn pitch_power(c: Color) -> u8 {
    match c {
        Color::Red => 3,
        Color::Yellow => 2,
        Color::Blue => 1,
    }
}

/// « At the beginning of your end phase, return to the brood » (Agents of Chaos).
fn return_to_the_brood() -> TriggerDef {
    TriggerDef {
        when: beginning_of_your_end_phase,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| g.return_to_brood(c.controller))],
    }
}

/// Un Agent of Chaos : Chaos Assassin Demi-Hero, vie « * » (CR 1.12.3,
/// codée 0 : la vie du joueur ne change pas, CR 8.5.25e), intellect 4.
fn agent(name: &str) -> CardDef {
    CardDef {
        name: name.into(),
        life: Some(0),
        intellect: Some(4),
        types: Types::DEMI_HERO,
        supertypes: CH | AS,
        traits: Traits::AGENT_OF_CHAOS,
        triggers: vec![return_to_the_brood()],
        ..Default::default()
    }
}

/// « Once per Turn Attack Reaction -- Discard an Assassin card: Target
/// [cible] gets +3{p}. … » (Black Widow, Funnel Web, Redback, Tarantula).
fn agent_reaction(label: &'static str, target: TargetSpec, script: Vec<Op>) -> ActivatedDef {
    ActivatedDef {
        label,
        speed: Speed::AttackReaction,
        once_per_turn: true,
        condition: Some(has_assassin_card),
        targets: vec![target],
        effect_cost: discard_an_assassin_card(),
        script,
        ..Default::default()
    }
}

/// « Target … gets +3{p} » sur la cible n° 0.
fn plus_three(g: &mut GameState, c: &mut Ctx) {
    if let Some(r) = target_obj(c, 0) {
        g.modify(r, power(3), Until::EndOfTurn);
    }
}

/// Black Widow / Funnel Web : « If it has stealth, it gets "When this hits
/// a hero, …" » (capacité n° 0 de `granted` de l'Agent).
fn plus_three_and_grant(g: &mut GameState, c: &mut Ctx) {
    plus_three(g, c);
    if let Some(r) = target_obj(c, 0).filter(|&r| g.is_current(r)) {
        if g.has_stealth(r.id) {
            g.grant_trigger(r, c.card, 0, Until::EndOfTurn);
        }
    }
}

/// « They banish a card from their hand » : le héros touché choisit.
fn they_banish_from_hand() -> Vec<Op> {
    vec![
        Op::Ask(|g, c| {
            let p = hit_hero(c)?;
            ask_object(
                p,
                "Bannir quelle carte de votre main ?",
                g.player(p).hand.clone(),
            )
        }),
        Op::Do(|g, c| {
            if let Some(p) = hit_hero(c) {
                let hand = g.player(p).hand.clone();
                if let Some(x) = picked(c, &hand) {
                    g.banish(p, x, false);
                }
            }
        }),
    ]
}

/// « Banish a card in their arsenal ».
fn banish_their_arsenal(g: &mut GameState, c: &mut Ctx) {
    if let Some(p) = hit_hero(c) {
        if let Some(x) = g.player(p).arsenal {
            g.banish(c.controller, x, false);
        }
    }
}

/// « When this hits a hero, mark them. »
fn hit_mark() -> TriggerDef {
    TriggerDef {
        when: this_hits_a_hero,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(p) = hit_hero(c) {
                g.mark(p);
            }
        })],
    }
}

/// Nombre de réactions d'attaque jouées ou activées par `p` sur le maillon
/// actif (Double Trouble).
pub(crate) fn attack_reactions_this_link(g: &GameState, p: PlayerId) -> usize {
    let Some(link) = g.active_link() else {
        return 0;
    };
    g.turn_events()[link.log_start..]
        .iter()
        .filter(|e| match **e {
            Event::Played { player, def, .. } => {
                player == p && g.db.get(def).types.has(Types::ATTACK_REACTION)
            }
            Event::Activated {
                player,
                source,
                ability,
            } => {
                player == p
                    && g.def(source)
                        .activated
                        .get(ability as usize)
                        .is_some_and(|a| a.speed == Speed::AttackReaction)
            }
            _ => false,
        })
        .count()
}

/// Double Trouble : « If you've played or activated 2 or more attack
/// reactions this chain link » (seulement quand elle est l'attaque active).
fn double_trouble_on(g: &GameState, me: ObjectId) -> bool {
    g.active_attack() == Some(me) && attack_reactions_this_link(g, g.obj(me).controller) >= 2
}

// ---------------------------------------------------------------------------
// Cartes
// ---------------------------------------------------------------------------

pub fn add(db: &mut CardDb) {
    hero_and_agents(db);
    arena(db);
    deck_cards(db);
}

fn hero_and_agents(db: &mut CardDb) {
    // Arakni, Web of Deceit — Chaos Assassin Hero - Young, 20 PV, intellect 4.
    // « Your attacks with **stealth** that are attacking a **marked** hero
    // get +1{p} and "When this hits, this gets **go again**."
    // At the beginning of your end phase, if an opponent is **marked**, you
    // become a random Agent of Chaos. »
    db.add(CardDef {
        name: "Arakni, Web of Deceit".into(),
        life: Some(20),
        intellect: Some(4),
        types: Types::HERO,
        subtypes: Subtypes::YOUNG,
        supertypes: CH | AS,
        aura: Some(|g, src, t| {
            // Seule l'attaque active « is attacking » (CR 7.0.3b). La
            // lecture de stealth est imprimée (voir `has_stealth`).
            let me = g.obj(src).controller;
            match g.active_link() {
                Some(l)
                    if l.attack.id == t
                        && l.attacker == me
                        && l.target_obj.is_none()
                        && g.player(l.target).marked
                        && g.has_stealth(t) =>
                {
                    power(1)
                }
                _ => Delta::default(),
            }
        }),
        triggers: vec![
            // « When this hits, this gets go again » : capacité accordée à
            // l'attaque tant qu'elle attaque un héros marqué ; elle se
            // déclenche à la touche, pendant laquelle la marque disparaît
            // (CR 9.3.3) : on lit `marked` de l'événement. Codée comme une
            // capacité du héros, avec le même effet sur l'attaque.
            TriggerDef {
                when: |g, me, e| match *e {
                    Event::Hit {
                        attack,
                        attacker,
                        target_obj: None,
                        marked: true,
                        effect: false,
                        ..
                    } => attacker == g.obj(me).controller && g.has_stealth(attack),
                    _ => false,
                },
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, _| {
                    if let Some(a) = active_attack_ref(g) {
                        g.modify(a, go_again(), Until::EndOfTurn);
                    }
                })],
            },
            // « If an opponent is marked » : vérifié au déclenchement et à
            // la résolution (condition « if » d'une capacité déclenchée).
            TriggerDef {
                when: |g, me, e| {
                    beginning_of_your_end_phase(g, me, e)
                        && g.player(g.obj(me).controller.opponent()).marked
                },
                anywhere: false,
                targets: vec![],
                script: vec![Op::Do(|g, c| {
                    if g.player(c.controller.opponent()).marked {
                        g.become_random(c.controller, Traits::AGENT_OF_CHAOS);
                    }
                })],
            },
        ],
        ..Default::default()
    });

    // Arakni, Black Widow — Chaos Assassin Demi-Hero, vie *, intellect 4.
    // « Once per Turn Attack Reaction -- Discard an Assassin card: Target
    // Assassin attack gets +3{p}. If it has **stealth**, it gets "When this
    // hits a hero, they banish a card from their hand."
    // At the beginning of your end phase, **return to the brood**. »
    let mut d = agent("Arakni, Black Widow");
    d.activated = vec![agent_reaction(
        "Défausser une carte Assassin : +3{p}",
        TARGET_ASSASSIN_ATTACK,
        vec![Op::Do(plus_three_and_grant)],
    )];
    d.granted = vec![TriggerDef {
        when: this_hits_a_hero,
        anywhere: false,
        targets: vec![],
        script: they_banish_from_hand(),
    }];
    db.add(d);

    // Arakni, Funnel Web — Chaos Assassin Demi-Hero, vie *, intellect 4.
    // « Once per Turn Attack Reaction -- Discard an Assassin card: Target
    // Assassin attack gets +3{p}. If it has **stealth**, it gets "When this
    // hits a hero, banish a card in their arsenal."
    // At the beginning of your end phase, **return to the brood**. »
    let mut d = agent("Arakni, Funnel Web");
    d.activated = vec![agent_reaction(
        "Défausser une carte Assassin : +3{p}",
        TARGET_ASSASSIN_ATTACK,
        vec![Op::Do(plus_three_and_grant)],
    )];
    d.granted = vec![TriggerDef {
        when: this_hits_a_hero,
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(banish_their_arsenal)],
    }];
    db.add(d);

    // Arakni, Orb-Weaver — Chaos Assassin Demi-Hero, vie *, intellect 4.
    // « Graphene Chelicerae cost you {r} less to activate.
    // **Once per Turn Instant** -- Discard an Assassin card: Equip a Graphene
    // Chelicera token. Your next attack with **stealth** this turn gets +3{p}.
    // At the beginning of your end phase, **return to the brood**. »
    let mut d = agent("Arakni, Orb-Weaver");
    d.activation_discount = Some(|g, _, obj| u8::from(g.def(obj).name == CHELICERA));
    d.activated = vec![ActivatedDef {
        label: "Défausser une carte Assassin : Graphene Chelicera",
        speed: Speed::Instant,
        once_per_turn: true,
        condition: Some(has_assassin_card),
        effect_cost: discard_an_assassin_card(),
        script: vec![Op::Do(|g, c| {
            equip_chelicera(g, c.controller);
            next_stealth_attack(g, c.controller, 3);
        })],
        ..Default::default()
    }];
    db.add(d);

    // Arakni, Redback — Chaos Assassin Demi-Hero, vie *, intellect 4.
    // « Once per Turn Attack Reaction -- Discard an Assassin card: Target
    // Assassin attack gets +3{p}. If it has **stealth**, it gets **go again**.
    // At the beginning of your end phase, **return to the brood**. »
    let mut d = agent("Arakni, Redback");
    d.activated = vec![agent_reaction(
        "Défausser une carte Assassin : +3{p}",
        TARGET_ASSASSIN_ATTACK,
        vec![Op::Do(|g, c| {
            plus_three(g, c);
            if let Some(r) = target_obj(c, 0).filter(|&r| g.is_current(r)) {
                if g.has_stealth(r.id) {
                    g.modify(r, go_again(), Until::EndOfTurn);
                }
            }
        })],
    )];
    db.add(d);

    // Arakni, Tarantula — Chaos Assassin Demi-Hero, vie *, intellect 4.
    // « Whenever a dagger you own hits a hero, they lose 1{h}.
    // **Once per Turn Attack Reaction** -- Discard an Assassin card: Target
    // dagger attack gets +3{p}.
    // At the beginning of your end phase, **return to the brood**. »
    let mut d = agent("Arakni, Tarantula");
    d.triggers.push(TriggerDef {
        // Pour une attaque-proxy, l'événement « hit » désigne l'arme ; une
        // dague qui touche par un effet (Bite, Hurl…) aussi.
        when: |g, me, e| match *e {
            Event::Hit {
                attack,
                target_obj: None,
                ..
            } => is_dagger(g, attack) && g.obj(attack).owner == g.obj(me).controller,
            _ => false,
        },
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(p) = hit_hero(c) {
                g.lose_life(p, 1);
            }
        })],
    });
    d.activated = vec![agent_reaction(
        "Défausser une carte Assassin : +3{p} (dague)",
        TARGET_DAGGER_ATTACK,
        vec![Op::Do(plus_three)],
    )];
    db.add(d);

    // Arakni, Trap-Door — Chaos Assassin Demi-Hero, vie *, intellect 4.
    // « When you become this, you may search your deck for a card, banish
    // it face-down, then shuffle. If it's a trap, you may play it until the
    // start of your next turn.
    // At the beginning of your end phase, **return to the brood**. »
    let mut d = agent("Arakni, Trap-Door");
    d.triggers.push(TriggerDef {
        when: |g, me, e| {
            matches!(*e, Event::Became { player, card }
                if player == g.obj(me).controller && card == g.obj(me).card)
        },
        anywhere: false,
        targets: vec![],
        script: vec![
            Op::Ask(|g, c| {
                (!g.player(c.controller).deck.is_empty()).then(|| Ask {
                    player: c.controller,
                    label: "Chercher une carte dans votre deck (bannie face cachée) ?",
                    options: vec![Choice::Yes, Choice::No],
                })
            }),
            Op::Do(|_, c| c.flag = c.answer.take() == Some(Choice::Yes)),
            Op::If(|_, c| c.flag, 3),
            Op::Ask(|g, c| {
                let deck = g.player(c.controller).deck.clone();
                ask_object(c.controller, "Quelle carte bannir face cachée ?", deck)
            }),
            Op::Do(|g, c| {
                let deck = g.player(c.controller).deck.clone();
                if let Some(x) = picked(c, &deck) {
                    g.banish(c.controller, x, true);
                    if g.def(x).subtypes.has(Subtypes::TRAP) {
                        // « Until the start of your next turn » : ce tour et
                        // le tour de l'adversaire.
                        let t = g.turn;
                        g.permit_play_face_down(c.controller, x, t);
                        g.permit_play_face_down(c.controller, x, t + 1);
                    }
                }
            }),
            Op::Do(|g, c| g.shuffle_deck(c.controller)),
        ],
    });
    db.add(d);
}

fn arena(db: &mut CardDb) {
    // Danger Digits — Assassin / Ninja Equipment - Arms, déf. 0.
    // « **Attack Reaction** -- Destroy this: Target dagger you control that
    // isn't on the active chain link deals 1 damage to the defending hero.
    // If damage is dealt this way, the dagger has hit. Destroy the dagger. »
    let mut d = equipment("Danger Digits", AN, Subtypes::ARMS, 0);
    let [strike, destroy] = dagger_strikes();
    d.activated = vec![ActivatedDef {
        label: "Détruire : une dague inflige 1 dégât",
        speed: Speed::AttackReaction,
        targets: vec![TARGET_DAGGER_NOT_ACTIVE],
        effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
        script: vec![strike, destroy],
        ..Default::default()
    }];
    db.add(d);

    // Mark of the Huntsman — Assassin Weapon - Dagger (1H), 1{p}.
    // « **Once per Turn Action** -- {r}{r}: **Attack**. **Go again**
    // When this hits a hero, you may choose to destroy this and **mark** them.
    // If this is attacking a **marked** hero, this gets +1{p}. »
    db.add(CardDef {
        name: "Mark of the Huntsman".into(),
        power: Some(1),
        types: Types::WEAPON,
        subtypes: Subtypes::DAGGER | Subtypes::ONE_HAND,
        supertypes: AS,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 2,
            once_per_turn: true,
            attack: true,
            go_again: true,
            ..Default::default()
        }],
        self_mod: Some(|g, me| match g.attacked_hero(me) {
            Some(h) if g.player(h).marked => power(1),
            _ => Delta::default(),
        }),
        triggers: vec![TriggerDef {
            when: this_hits_a_hero,
            anywhere: false,
            targets: vec![],
            // TODO(règles) : si la dague a déjà été détruite (Bite, Hurl,
            // Danger Digits la détruisent juste après la touche), le joueur
            // peut-il quand même marquer le héros ? Choix provisoire : oui
            // (« destroy this » échoue, « mark them » est fait).
            script: vec![
                Op::Ask(|_, c| {
                    hit_hero(c).map(|_| Ask {
                        player: c.controller,
                        label: "Détruire Mark of the Huntsman et marquer le héros ?",
                        options: vec![Choice::Yes, Choice::No],
                    })
                }),
                Op::Do(|g, c| {
                    if c.answer.take() == Some(Choice::Yes) {
                        if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                            g.destroy(s.id);
                        }
                        if let Some(p) = hit_hero(c) {
                            g.mark(p);
                        }
                    }
                }),
            ],
        }],
        ..Default::default()
    });

    // Prey Spotters — Assassin Equipment - Head, déf. 1.
    // « **Attack Reaction** -- Destroy this: **Mark** target opposing hero.
    // **Battleworn** »
    let mut d = equipment("Prey Spotters", AS, Subtypes::HEAD, 1);
    d.keywords = Keywords::BATTLEWORN;
    d.activated = vec![ActivatedDef {
        label: "Détruire : marquer le héros adverse",
        speed: Speed::AttackReaction,
        targets: vec![TARGET_OPPOSING_HERO],
        effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
        script: vec![Op::Do(mark_target_hero)],
        ..Default::default()
    }];
    db.add(d);

    // Stalker's Steps — Assassin Equipment - Legs, déf. 0.
    // « **Attack Reaction** -- Destroy this: Target attack with stealth gets
    // **go again**. **Arcane Barrier 1** »
    let mut d = equipment("Stalker's Steps", AS, Subtypes::LEGS, 0);
    d.arcane_barrier = 1;
    d.activated = vec![ActivatedDef {
        label: "Détruire : go again",
        speed: Speed::AttackReaction,
        targets: vec![TARGET_STEALTH_ATTACK],
        effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
        script: vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                g.modify(r, go_again(), Until::EndOfTurn);
            }
        })],
        ..Default::default()
    }];
    db.add(d);

    // Topsy Turvy — Chaos Equipment - Head, déf. 0.
    // « **Instant** -- Destroy this: Until end of turn, if one or more cards
    // would be put on top of a deck, instead they're put on the bottom.
    // **Arcane Barrier 1** »
    let mut d = equipment("Topsy Turvy", CH, Subtypes::HEAD, 0);
    d.arcane_barrier = 1;
    d.activated = vec![ActivatedDef {
        label: "Détruire : dessus du deck → dessous",
        speed: Speed::Instant,
        effect_cost: vec![Op::Do(|g, c| super::generic::destroy_source(g, c))],
        script: vec![Op::Do(|g, c| {
            g.add_replacement(Replacement {
                id: 0,
                kind: ReplaceKind::ToDeckTop,
                controller: c.controller,
                data: None,
                bind: None,
                value: 0,
                optional: false,
                applies: |_, _, _| true,
                apply: |_, _, p| p.on_top = false,
                until: Until::EndOfTurn,
                uses: None,
            })
        })],
        ..Default::default()
    }];
    db.add(d);

    // Graphene Chelicera — Assassin Token Weapon - Dagger (1H), 1{p}.
    // « **Stealth**
    // **Once per Turn Action** -- {r}: **Attack**
    // When this attacks a **marked** hero, the attack gets **go again**. »
    db.add(CardDef {
        name: CHELICERA.into(),
        power: Some(1),
        types: Types::TOKEN | Types::WEAPON,
        subtypes: Subtypes::DAGGER | Subtypes::ONE_HAND,
        supertypes: AS,
        keywords: Keywords::STEALTH,
        activated: vec![ActivatedDef {
            label: "Attaquer",
            speed: Speed::Action,
            cost: 1,
            once_per_turn: true,
            attack: true,
            ..Default::default()
        }],
        triggers: vec![TriggerDef {
            when: |g, me, e| {
                matches!(*e, Event::Attack { attack, target, target_obj: None, .. }
                    if attack == me && g.player(target).marked)
            },
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, _| {
                if let Some(a) = active_attack_ref(g) {
                    g.modify(a, go_again(), Until::EndOfTurn);
                }
            })],
        }],
        ..Default::default()
    });

    // Bloodrot Pox — Generic Token - Disease Aura.
    // « At the beginning of your end phase, destroy this, then it deals 2
    // damage to you unless you pay {r}{r}{r}. »
    db.add(CardDef {
        name: POX.into(),
        types: Types::TOKEN,
        subtypes: Subtypes::DISEASE | Subtypes::AURA,
        triggers: vec![TriggerDef {
            when: beginning_of_your_end_phase,
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Do(|g, c| {
                    if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                        g.destroy(s.id);
                    }
                }),
                // Le joueur ne choisit que s'il peut payer.
                Op::Ask(|g, c| {
                    (g.available_resources(c.controller, None) >= 3).then(|| Ask {
                        player: c.controller,
                        label: "Payer 3 ressources ? Sinon Bloodrot Pox vous inflige 2 dégâts.",
                        options: vec![Choice::Yes, Choice::No],
                    })
                }),
                Op::Do(|g, c| {
                    if c.answer.take() == Some(Choice::Yes) {
                        g.pay(c.controller, 3);
                    } else {
                        let src = c.source_id();
                        g.deal_damage(src, c.controller, c.controller, 2, false);
                    }
                }),
            ],
        }],
        ..Default::default()
    });
}

fn deck_cards(db: &mut CardDb) {
    // Art of Desire: Body (rouge) — Assassin Action - Attack, 3{p}, coût 0, déf. 3.
    // « **Stealth**
    // When this hits a hero, banish the top card of their deck.
    // Whenever this banishes a red card, draw a card and gain 1{h}. »
    let mut d = stealth_attack("Art of Desire: Body", Color::Red);
    d.triggers = vec![
        TriggerDef {
            when: this_hits_a_hero,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let Some(p) = hit_hero(c) else { return };
                let Some(top) = g.top_of_deck(p) else { return };
                g.banish(c.controller, top, false);
                // « Whenever this banishes a red card » : capacité
                // déclenchée distincte (n° 1).
                if g.def(top).color == Some(Color::Red) {
                    if let Some(s) = c.source_id() {
                        g.queue_trigger(s, 1);
                    }
                }
            })],
        },
        TriggerDef {
            when: |_, _, _| false,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                g.draw(c.controller);
                g.gain_life(c.controller, 1);
            })],
        },
    ];
    db.add(d);

    // Bite — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // When this attacks a hero, you may have target dagger you control deal
    // 1 damage to them. If damage is dealt this way, the dagger has hit.
    // Destroy the dagger. »
    // « You may » : la cible est facultative ; sans cible, rien ne se passe.
    for &color in &COLORS {
        let mut d = stealth_attack("Bite", color);
        let [strike, destroy] = dagger_strikes();
        d.triggers = vec![TriggerDef {
            when: when_this_attacks_a_hero,
            anywhere: false,
            targets: vec![TARGET_DAGGER_OPTIONAL],
            script: vec![strike, destroy],
        }];
        db.add(d);
    }

    // Concoct Disorder — Chaos Action - Attack, 4 / 3 / 2, coût 1, déf. 3.
    // « When this attacks, each hero puts the top card of their deck
    // face-down into their arsenal. If 2 or more cards are put into arsenals
    // this way, this gets **go again**. »
    // Un héros dont l'arsenal est occupé (ou le deck vide) n'y met rien.
    for (i, &color) in COLORS.iter().enumerate() {
        let mut d = deck_card(
            "Concoct Disorder",
            color,
            CH,
            Types::ACTION,
            Some(1),
            Some([4, 3, 2][i]),
            Some(3),
        );
        d.triggers = vec![TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let mut n = 0;
                // Le joueur actif d'abord (CR 1.9.2c).
                for p in [g.turn_player, g.turn_player.opponent()] {
                    if g.player(p).arsenal.is_some() {
                        continue;
                    }
                    if let Some(top) = g.top_of_deck(p) {
                        g.put_in_arsenal(top);
                        n += 1;
                    }
                }
                if n >= 2 {
                    if let Some(s) = c.source.filter(|&s| g.is_current(s)) {
                        g.modify(s, go_again(), Until::EndOfTurn);
                    }
                }
            })],
        }];
        db.add(d);
    }

    // Double Trouble — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // If you've played or activated 2 or more attack reactions this chain
    // link, this gets +2{p} and "When this hits a hero, banish the top 2
    // cards of their deck." »
    for &color in &COLORS {
        let mut d = stealth_attack("Double Trouble", color);
        d.self_mod = Some(|g, me| {
            if double_trouble_on(g, me) {
                power(2)
            } else {
                Delta::default()
            }
        });
        d.triggers = vec![TriggerDef {
            when: |g, me, e| this_hits_a_hero(g, me, e) && double_trouble_on(g, me),
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                let Some(p) = hit_hero(c) else { return };
                for _ in 0..2 {
                    if let Some(top) = g.top_of_deck(p) {
                        g.banish(c.controller, top, false);
                    }
                }
            })],
        }];
        db.add(d);
    }

    // Hurl — Assassin / Ninja Action - Attack, 3 / 2 / 1, coût 0, déf. 2.
    // « As an additional cost to play this, you may pay {r}.
    // If the additional cost is paid, this gets "When this attacks, target
    // dagger you control deals 1 damage to target hero. If damage is dealt
    // this way, the dagger has hit. Destroy the dagger."
    // **Go again** »
    for &color in &COLORS {
        let mut d = deck_card(
            "Hurl",
            color,
            AN,
            Types::ACTION,
            Some(0),
            Some(pitch_power(color)),
            Some(2),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.additional_cost = Some(fab_engine::card::AdditionalCost {
            resources: 1,
            optional: true,
        });
        // La capacité est accordée à la carte quand le coût est payé
        // (l'attaque garde son identité de la pile à la chaîne de combat).
        d.effect_cost = vec![Op::Do(|g, c| {
            if c.optional_paid {
                if let Some(s) = c.source {
                    g.grant_trigger(s, c.card, 0, Until::EndOfTurn);
                }
            }
        })];
        let [strike, destroy] = dagger_strikes();
        d.granted = vec![TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![TARGET_DAGGER, TARGET_HERO],
            script: vec![strike, destroy],
        }];
        db.add(d);
    }

    // Infect — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // When this hits a hero, create a Bloodrot Pox token under their control. »
    for &color in &COLORS {
        let mut d = stealth_attack("Infect", color);
        d.triggers = vec![TriggerDef {
            when: this_hits_a_hero,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(|g, c| {
                if let Some(p) = hit_hero(c) {
                    let pox = card_named(g, POX);
                    g.create_tokens_by(c.controller, p, pox, 1);
                }
            })],
        }];
        db.add(d);
    }

    // Lair of the Spider (rouge) — Assassin / Ninja Defense Reaction - Trap,
    // coût 0, déf. 3.
    // « When this defends an attack with **go again**, **mark** the attacking hero. »
    let mut d = deck_card(
        "Lair of the Spider",
        Color::Red,
        AN,
        Types::DEFENSE_REACTION,
        Some(0),
        None,
        Some(3),
    );
    d.subtypes = Subtypes::TRAP;
    d.triggers = vec![TriggerDef {
        when: |g, me, e| {
            matches!(*e, Event::Defend { card, attack, .. }
                if card == me && g.keywords(attack).has(Keywords::GO_AGAIN))
        },
        anywhere: false,
        targets: vec![],
        script: vec![Op::Do(|g, c| {
            if let Some(Event::Defend { attack, .. }) = c.event {
                let attacker = g.obj(attack).controller;
                g.mark(attacker);
            }
        })],
    }];
    db.add(d);

    // Mark of the Black Widow — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // When this hits a **marked** hero, they banish a card from their hand. »
    for &color in &COLORS {
        let mut d = stealth_attack("Mark of the Black Widow", color);
        d.triggers = vec![TriggerDef {
            when: this_hits_a_marked_hero,
            anywhere: false,
            targets: vec![],
            script: they_banish_from_hand(),
        }];
        db.add(d);
    }

    // Mark of the Funnel Web — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // When this hits a **marked** hero, banish a card in their arsenal. »
    for &color in &COLORS {
        let mut d = stealth_attack("Mark of the Funnel Web", color);
        d.triggers = vec![TriggerDef {
            when: this_hits_a_marked_hero,
            anywhere: false,
            targets: vec![],
            script: vec![Op::Do(banish_their_arsenal)],
        }];
        db.add(d);
    }

    // Mark the Prey — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // When this hits a hero, **mark** them. »
    for &color in &COLORS {
        let mut d = stealth_attack("Mark the Prey", color);
        d.triggers = vec![hit_mark()];
        db.add(d);
    }

    // Night's Embrace (bleu) — Assassin Attack Reaction, coût 0, déf. 3.
    // « Your attacks with stealth get +1{p} this turn. »
    let mut d = deck_card(
        "Night's Embrace",
        Color::Blue,
        AS,
        Types::ATTACK_REACTION,
        Some(0),
        None,
        Some(3),
    );
    d.resolve = vec![Op::Do(|g, c| {
        g.buff_attacks_this_turn(c.controller, |g, a| g.has_stealth(a), power(1));
    })];
    db.add(d);

    // Orb-Weaver Spinneret — Assassin Action, coût 0, déf. 3.
    // « Equip a Graphene Chelicera token.
    // Your next attack with **stealth** this turn gets +3{p} / +2{p} / +1{p}.
    // **Go again** »
    for &color in &COLORS {
        let mut d = deck_card(
            "Orb-Weaver Spinneret",
            color,
            AS,
            Types::ACTION,
            Some(0),
            None,
            Some(3),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.resolve = vec![Op::Do(|g, c| {
            equip_chelicera(g, c.controller);
            let n = by_color(g, c, [3, 2, 1]);
            next_stealth_attack(g, c.controller, n);
        })];
        db.add(d);
    }

    // Pick Up the Point — Assassin / Ninja Action - Attack, 3 / 2 / 1, coût 0, déf. 2.
    // « When this attacks, you may **retrieve** a dagger from your graveyard.
    // **Go again** »
    // Retrieve (CR 8.5.51) : payer {r} pour équiper la carte ; seulement si
    // elle peut être équipée (une main libre) et si le joueur peut payer.
    for &color in &COLORS {
        let mut d = deck_card(
            "Pick Up the Point",
            color,
            AN,
            Types::ACTION,
            Some(0),
            Some(pitch_power(color)),
            Some(2),
        );
        d.keywords = Keywords::GO_AGAIN;
        d.triggers = vec![TriggerDef {
            when: when_this_attacks,
            anywhere: false,
            targets: vec![],
            script: vec![
                Op::Ask(|g, c| {
                    let v = retrievable_daggers(g, c.controller);
                    if v.is_empty() {
                        return None;
                    }
                    let mut options: Vec<Choice> = v.into_iter().map(Choice::Object).collect();
                    options.push(Choice::Pass);
                    Some(Ask {
                        player: c.controller,
                        label: "Payer {r} pour équiper une dague de votre cimetière (retrieve) ?",
                        options,
                    })
                }),
                Op::Do(|g, c| {
                    if let Some(Choice::Object(x)) = c.answer {
                        if retrievable_daggers(g, c.controller).contains(&x) {
                            c.picked = vec![x];
                            g.pay(c.controller, 1);
                        }
                    }
                }),
                Op::Do(|g, c| {
                    if let Some(&x) = c.picked.first() {
                        g.equip_from_graveyard(x);
                    }
                }),
            ],
        }];
        db.add(d);
    }

    // Razor's Edge — Assassin Attack Reaction, coût 0, déf. 3.
    // « Target attack action card with **stealth** gets +3{p} / +2{p} / +1{p}. »
    for &color in &COLORS {
        let mut d = deck_card(
            "Razor's Edge",
            color,
            AS,
            Types::ATTACK_REACTION,
            Some(0),
            None,
            Some(3),
        );
        d.targets = vec![TARGET_STEALTH_ATTACK_CARD];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let n = by_color(g, c, [3, 2, 1]);
                g.modify(r, power(n), Until::EndOfTurn);
            }
        })];
        db.add(d);
    }

    // Reaper's Call — Assassin Action - Attack, 3 / 2 / 1, coût 0, déf. 3.
    // « **Stealth**
    // **Instant** -- Discard this: **Mark** target opposing hero. »
    for &color in &COLORS {
        let mut d = stealth_attack("Reaper's Call", color);
        d.activated = vec![ActivatedDef {
            label: "Défausser : marquer le héros adverse",
            speed: Speed::Instant,
            from_hand: true,
            targets: vec![TARGET_OPPOSING_HERO],
            effect_cost: vec![Op::Do(|g, c| {
                if let Some(s) = c.source_id().filter(|&s| g.obj(s).zone == Zone::Hand) {
                    g.discard(s);
                }
            })],
            script: vec![Op::Do(mark_target_hero)],
            ..Default::default()
        }];
        db.add(d);
    }

    // Shred — Assassin Attack Reaction, coût 0, déf. 3.
    // « Target card defending an Assassin attack gets -4{d} / -3{d} / -2{d}
    // this combat chain. »
    for &color in &COLORS {
        let mut d = deck_card(
            "Shred",
            color,
            AS,
            Types::ATTACK_REACTION,
            Some(0),
            None,
            Some(3),
        );
        d.targets = vec![TARGET_DEFENDING_ASSASSIN];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let n = by_color(g, c, [4, 3, 2]);
                let delta = Delta {
                    defense: -n,
                    ..Default::default()
                };
                g.modify(r, delta, Until::CombatChainCloses);
            }
        })];
        db.add(d);
    }

    // Stains of the Redback — Assassin Attack Reaction, coût 1, déf. 3.
    // « If the defending hero is **marked**, this costs {r} less to play.
    // Target attack with **stealth** gets +3{p} / +2{p} / +1{p} and **go again**. »
    for &color in &COLORS {
        let mut d = deck_card(
            "Stains of the Redback",
            color,
            AS,
            Types::ATTACK_REACTION,
            Some(1),
            None,
            Some(3),
        );
        d.cost_mod = Some(|g, _, _| match g.active_link() {
            Some(l) if g.player(l.target).marked => -1,
            _ => 0,
        });
        d.targets = vec![TARGET_STEALTH_ATTACK];
        d.resolve = vec![Op::Do(|g, c| {
            if let Some(r) = target_obj(c, 0) {
                let n = by_color(g, c, [3, 2, 1]);
                let delta = Delta {
                    power: n,
                    grants: Keywords::GO_AGAIN,
                    ..Default::default()
                };
                g.modify(r, delta, Until::EndOfTurn);
            }
        })];
        db.add(d);
    }
}

/// Les dagues du cimetière de `p` que le joueur peut « retrieve » : une main
/// libre (CR 8.5.51a) et {r} disponible.
fn retrievable_daggers(g: &GameState, p: PlayerId) -> Vec<ObjectId> {
    if g.available_resources(p, None) < 1 {
        return Vec::new();
    }
    g.player(p)
        .graveyard
        .iter()
        .copied()
        .filter(|&x| is_dagger(g, x) && g.can_equip_weapon(p, g.obj(x).card))
        .collect()
}
