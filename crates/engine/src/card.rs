//! Définitions des cartes : données imprimées + capacités.
//!
//! Une [`CardDef`] décrit une carte « en général » (ex. : telle carte, version
//! rouge). Les exemplaires en jeu sont des objets ([`crate::state::Object`]) qui
//! pointent vers leur définition par un [`CardId`].
//!
//! Les capacités sont décrites par de petites listes d'instructions ([`Op`]),
//! interprétées par le moteur ([`crate::effects`]). Les instructions seront
//! enrichies à l'étape 2, carte par carte.

use crate::ids::CardId;
use std::ops::BitOr;

macro_rules! flags {
    ($(#[$m:meta])* $name:ident($t:ty) { $($(#[$fm:meta])* $flag:ident = $bit:expr),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub struct $name(pub $t);
        impl $name {
            pub const NONE: $name = $name(0);
            $($(#[$fm])* pub const $flag: $name = $name(1 << $bit);)*
            #[inline]
            pub fn has(self, other: $name) -> bool {
                self.0 & other.0 == other.0 && other.0 != 0
            }
            #[inline]
            pub fn any(self, other: $name) -> bool {
                self.0 & other.0 != 0
            }
        }
        impl BitOr for $name {
            type Output = $name;
            fn bitor(self, o: $name) -> $name {
                $name(self.0 | o.0)
            }
        }
    };
}

flags! {
    /// Types de carte (CR 8.1).
    Types(u16) {
        ACTION = 0, ATTACK_REACTION = 1, DEFENSE_REACTION = 2, INSTANT = 3,
        EQUIPMENT = 4, WEAPON = 5, HERO = 6, RESOURCE = 7, TOKEN = 8, BLOCK = 9,
        MENTOR = 10, DEMI_HERO = 11,
    }
}

flags! {
    /// Sous-types (CR 8.2).
    Subtypes(u32) {
        ATTACK = 0, ONE_HAND = 1, TWO_HAND = 2, HEAD = 3, CHEST = 4, ARMS = 5,
        LEGS = 6, OFF_HAND = 7, AURA = 8, ITEM = 9, YOUNG = 10, ALLY = 11,
        LANDMARK = 12, AFFLICTION = 13, ASH = 14, INVOCATION = 15, CONSTRUCT = 16,
        QUIVER = 17, FIGMENT = 18, ARROW = 19,
    }
}

flags! {
    /// Mots-clés de capacité gérés par le moteur (CR 8.3).
    Keywords(u32) {
        /// CR 8.3.5
        GO_AGAIN = 0,
        /// CR 8.3.4
        DOMINATE = 1,
        /// CR 8.3.3
        BLADE_BREAK = 2,
        /// CR 8.3.2
        BATTLEWORN = 3,
        /// CR 8.3.10
        TEMPER = 4,
    }
}

impl Types {
    /// CR 1.3.2c : cartes de deck.
    pub fn is_deck_card(self) -> bool {
        self.any(
            Types::ACTION
                | Types::ATTACK_REACTION
                | Types::DEFENSE_REACTION
                | Types::INSTANT
                | Types::MENTOR
                | Types::RESOURCE
                | Types::BLOCK,
        )
    }
}

/// Couleur (barre de couleur) d'une carte de deck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Color {
    Red,
    Yellow,
    Blue,
}

/// Une instruction d'effet.
///
/// Étape 1 : instructions simples, sans choix du joueur. Les instructions
/// avec choix (cibles, modes, « vous pouvez ») arrivent à l'étape 2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// Le contrôleur pioche N cartes (CR 8.5.6).
    Draw(u8),
    /// Le contrôleur gagne N points de vie (CR 8.5.7).
    GainLife(u8),
    /// Inflige N dégâts au héros adverse (CR 8.5.3), la source est cette carte.
    DamageOpposingHero(u8),
    /// L'attaque de la chaîne active gagne +N de puissance jusqu'à la fin du tour.
    ActiveAttackGetsPower(i8),
    /// « Votre prochaine attaque ce tour gagne +N » (CR 1.8.10).
    NextAttackGetsPower(i8),
}

/// Capacité d'attaque d'une arme : « [Once per Turn] Action — {coût} : Attack ».
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponAttack {
    pub cost: u8,
    pub once_per_turn: bool,
    /// L'attaque-proxy a « go again » (ex. « Attack. Go again. »).
    pub go_again: bool,
}

/// Définition complète d'une carte.
#[derive(Clone, Debug, Default)]
pub struct CardDef {
    pub name: String,
    pub color: Option<Color>,
    /// Valeurs imprimées ; `None` = la carte n'a pas la propriété (CR 2.x).
    pub pitch: Option<u8>,
    pub cost: Option<u8>,
    pub power: Option<u8>,
    pub defense: Option<u8>,
    pub life: Option<u8>,
    pub intellect: Option<u8>,
    pub types: Types,
    pub subtypes: Subtypes,
    pub keywords: Keywords,
    pub weapon_attack: Option<WeaponAttack>,
    /// Capacités de résolution (CR 5.3.4).
    pub on_resolve: Vec<Op>,
    /// « When this attacks, … » (CR 7.2.3).
    pub on_attack: Vec<Op>,
    /// « When this hits, … » (CR 7.5.5).
    pub on_hit: Vec<Op>,
}

impl CardDef {
    pub fn is_attack_card(&self) -> bool {
        self.subtypes.has(Subtypes::ATTACK)
    }

    /// Une carte d'action (type Action) — CR 8.1.1.
    pub fn is_action(&self) -> bool {
        self.types.has(Types::ACTION)
    }
}

/// La base des cartes utilisées dans une partie. Immuable pendant la partie,
/// partagée entre toutes les copies d'un état.
#[derive(Clone, Debug, Default)]
pub struct CardDb {
    defs: Vec<CardDef>,
}

impl CardDb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, def: CardDef) -> CardId {
        let id = CardId(self.defs.len() as u16);
        self.defs.push(def);
        id
    }

    #[inline]
    pub fn get(&self, id: CardId) -> &CardDef {
        &self.defs[id.0 as usize]
    }

    pub fn find(&self, name: &str, color: Option<Color>) -> Option<CardId> {
        self.defs
            .iter()
            .position(|d| d.name == name && d.color == color)
            .map(|i| CardId(i as u16))
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drapeaux() {
        let t = Types::ACTION | Types::INSTANT;
        assert!(t.has(Types::ACTION));
        assert!(!t.has(Types::WEAPON));
        assert!(!t.has(Types::NONE));
        assert!(t.is_deck_card());
        assert!(!Types::EQUIPMENT.is_deck_card());
    }
}
