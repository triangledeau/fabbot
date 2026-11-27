//! Les **vraies** cartes des 9 decks (étapes 2 et 3).
//!
//! Textes de référence : fichier officiel LSS, colonne `face_*_true_textbox`
//! (voir `docs/cards/README.md`). Chaque carte est précédée de son texte
//! officiel en commentaire. Les valeurs imprimées (pitch, coût, puissance,
//! défense, vie, intellect) sont vérifiées par le test
//! `crates/cards/tests/cartes_officielles.rs` contre le fichier officiel.
//!
//! Organisation :
//! - [`generic`] : cartes génériques (sans classe) ;
//! - [`warrior`] : cartes Warrior (Hala, Dorinthea) ;
//! - [`ninja`] : cartes Ninja (Ira) ;
//! - [`guardian`] : cartes Guardian (Bravo) ;
//! - [`wizard`] : cartes Wizard, Elemental et Ice (Iyslander) ;
//! - [`runeblade`] : cartes Runeblade, Earth et Elemental (Florian) ;
//! - [`shadow`] : cartes Shadow et Brute (Levia) ;
//! - [`draconic`] : cartes Draconic et Illusionist (Dromai) ;
//! - [`assassin`] : cartes Assassin et Chaos (Arakni) ;
//! - [`lightning`] : cartes Lightning, Elemental et Runeblade (Aurora).

pub mod assassin;
pub mod draconic;
pub mod generic;
pub mod guardian;
pub mod lightning;
pub mod ninja;
pub mod runeblade;
pub mod shadow;
pub mod warrior;
pub mod wizard;

use crate::decklist::{self, Decklist};
use fab_engine::card::{CardDb, CardDef, Color, Delta, Keywords, Subtypes, Supertypes, Types};
use fab_engine::script::{Ctx, Target, TargetSpec};
use fab_engine::{CardId, CardPool, GameState, Loadout, ObjectId};

/// Construit la base des vraies cartes déjà codées.
pub fn build() -> CardDb {
    let mut db = CardDb::new();
    generic::add(&mut db);
    warrior::add(&mut db);
    ninja::add(&mut db);
    guardian::add(&mut db);
    wizard::add(&mut db);
    runeblade::add(&mut db);
    shadow::add(&mut db);
    draconic::add(&mut db);
    assassin::add(&mut db);
    lightning::add(&mut db);
    db
}

/// Noms de toutes les cartes codées (toutes au moins `Tested` dans
/// `docs/cards/STATUS.md` ; seul le propriétaire les passe en `Validated`).
pub fn build_names() -> Vec<&'static str> {
    generic::NAMES
        .iter()
        .chain(warrior::NAMES.iter())
        .chain(ninja::NAMES.iter())
        .chain(guardian::NAMES.iter())
        .chain(wizard::NAMES.iter())
        .chain(runeblade::NAMES.iter())
        .chain(lightning::NAMES.iter())
        .chain(shadow::NAMES.iter())
        .chain(draconic::NAMES.iter())
        .chain(assassin::NAMES.iter())
        .copied()
        .collect()
}

pub(crate) const COLORS: [Color; 3] = [Color::Red, Color::Yellow, Color::Blue];

pub(crate) fn pitch_of(c: Color) -> u8 {
    match c {
        Color::Red => 1,
        Color::Yellow => 2,
        Color::Blue => 3,
    }
}

/// Une carte de deck. `cost = None` : la carte n'a pas de coût (bloc).
pub(crate) fn deck_card(
    name: &str,
    color: Color,
    class: Supertypes,
    types: Types,
    cost: Option<u8>,
    power: Option<u8>,
    defense: Option<u8>,
) -> CardDef {
    CardDef {
        name: name.into(),
        color: Some(color),
        pitch: Some(pitch_of(color)),
        cost,
        power,
        defense,
        types,
        supertypes: class,
        subtypes: if power.is_some() {
            Subtypes::ATTACK
        } else {
            Subtypes::NONE
        },
        ..Default::default()
    }
}

/// Un équipement (hors arme).
pub(crate) fn equipment(name: &str, class: Supertypes, slot: Subtypes, defense: u8) -> CardDef {
    CardDef {
        name: name.into(),
        defense: Some(defense),
        types: Types::EQUIPMENT,
        subtypes: slot,
        supertypes: class,
        ..Default::default()
    }
}

/// Valeur qui dépend de la couleur de la carte : `[rouge, jaune, bleu]`.
pub(crate) fn by_color(g: &GameState, c: &Ctx, v: [i16; 3]) -> i16 {
    match g.db.get(c.card).color {
        Some(Color::Red) | None => v[0],
        Some(Color::Yellow) => v[1],
        Some(Color::Blue) => v[2],
    }
}

pub(crate) fn power(n: i16) -> Delta {
    Delta {
        power: n,
        ..Default::default()
    }
}

pub(crate) fn grant(k: Keywords) -> Delta {
    Delta {
        grants: k,
        ..Default::default()
    }
}

/// Cible « target attack » : l'attaque du maillon actif.
pub(crate) const TARGET_ATTACK: TargetSpec = TargetSpec {
    label: "attaque",
    filter: |g, _, t| matches!(t, Target::Object(r) if g.active_attack() == Some(r.id)),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target weapon attack » : l'attaque du maillon actif, si c'est une
/// attaque d'arme.
pub(crate) const TARGET_WEAPON_ATTACK: TargetSpec = TargetSpec {
    label: "attaque d'arme",
    filter: |g, _, t| matches!(t, Target::Object(r) if g.active_attack() == Some(r.id) && g.is_weapon_attack(r.id)),
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// Cible « target sword you control » : une épée (pas une attaque-proxy).
pub(crate) const TARGET_SWORD_YOU_CONTROL: TargetSpec = TargetSpec {
    label: "épée",
    filter: |g, c, t| match t {
        Target::Object(r) => {
            let o = g.obj(r.id);
            o.proxy_of.is_none()
                && o.controller == c.controller
                && g.def(r.id).subtypes.has(Subtypes::SWORD)
        }
        Target::Player(_) => false,
    },
    optional: false,
    in_graveyard: false,
    in_banished: false,
};

/// L'objet ciblé n° `i`, s'il est encore là.
pub(crate) fn target_obj(c: &Ctx, i: usize) -> Option<fab_engine::ObjRef> {
    match c.target(i) {
        Some(Target::Object(r)) => Some(r),
        _ => None,
    }
}

/// Une attaque d'épée (attaque-proxy d'une épée).
pub(crate) fn is_sword_attack(g: &GameState, a: ObjectId) -> bool {
    g.is_weapon_attack(a) && g.def(a).subtypes.has(Subtypes::SWORD)
}

/// Une épée (l'arme elle-même, ou sa proxy).
pub(crate) fn is_sword(g: &GameState, o: ObjectId) -> bool {
    g.def(o).subtypes.has(Subtypes::SWORD)
}

/// CR 8.4.3 Reprise : « If the defending hero has defended with a card from
/// their hand this chain link ». Vérifié à la résolution (CR 8.4.3a).
pub(crate) fn reprise(g: &GameState) -> bool {
    let Some(link) = g.active_link() else {
        return false;
    };
    g.turn_events()[link.log_start..].iter().any(|e| {
        matches!(e, fab_engine::Event::Defend { defender, from_hand: true, .. } if *defender == link.target)
    })
}

/// Identifiant d'une carte par son nom (panique si elle n'est pas codée).
pub(crate) fn card_named(g: &GameState, name: &str) -> CardId {
    g.db.find_name(name)
        .unwrap_or_else(|| panic!("carte non codée : {name}"))
}

/// Une carte codée, par nom et couleur (erreur lisible si elle manque).
pub fn find(db: &CardDb, name: &str, color: Option<Color>) -> Result<CardId, String> {
    db.find(name, color)
        .ok_or_else(|| format!("carte non codée : {name} {color:?}"))
}

fn engine_color(c: decklist::Color) -> Color {
    match c {
        decklist::Color::Red => Color::Red,
        decklist::Color::Yellow => Color::Yellow,
        decklist::Color::Blue => Color::Blue,
    }
}

/// Le card-pool d'une liste de deck (CR 4.1, TRP 7.4) : le héros, chaque
/// exemplaire des cartes d'arène et des cartes de deck, dans l'ordre de la liste.
pub fn card_pool(db: &CardDb, list: &Decklist) -> Result<CardPool, String> {
    let hero = find(db, &list.hero, None)?;
    let mut arena = Vec::new();
    for e in &list.arena {
        let id = find(db, &e.name, None)?;
        for _ in 0..e.count {
            arena.push(id);
        }
    }
    let mut deck = Vec::new();
    for e in &list.deck {
        let id = find(db, &e.name, Some(engine_color(e.color)))?;
        for _ in 0..e.count {
            deck.push(id);
        }
    }
    Ok(CardPool { hero, arena, deck })
}

/// Choix « dans l'ordre de la liste » (`CardPool::choix_par_ordre`) : la
/// première carte de chaque zone d'arène qui trouve sa place (les mains
/// d'armes sont respectées, CR 8.2.2b), et les 40 premières cartes du deck.
/// C'est le choix du bot « liste » de la phase de side et celui des tests.
pub fn default_loadout(db: &CardDb, list: &Decklist) -> Result<Loadout, String> {
    Ok(default_loadout_pool(db, &card_pool(db, list)?))
}

/// [`default_loadout`] à partir d'un card-pool.
pub fn default_loadout_pool(db: &CardDb, pool: &CardPool) -> Loadout {
    let taille = fab_engine::FormatConfig::silver_age().deck_size as usize;
    let (arena, deck) = pool.choix_par_ordre(db, taille);
    pool.loadout(db, arena, deck, taille)
        .expect("le choix dans l'ordre de la liste est légal")
}
