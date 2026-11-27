//! Définitions des cartes : données imprimées + capacités.
//!
//! Une [`CardDef`] décrit une carte « en général » (ex. : telle carte, version
//! rouge). Les exemplaires en jeu sont des objets ([`crate::state::Object`]) qui
//! pointent vers leur définition par un [`CardId`].
//!
//! Les capacités sont de petits programmes ([`Op`], voir [`crate::script`])
//! écrits carte par carte dans `fab-cards`.

use crate::events::Event;
use crate::ids::{CardId, ObjectId, PlayerId};
use crate::replace::ReplacementDef;
use crate::script::{Op, TargetSpec};
use crate::state::GameState;
use std::ops::BitOr;

macro_rules! flags {
    ($(#[$m:meta])* $name:ident($t:ty) { $($(#[$fm:meta])* $flag:ident = $bit:expr),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub struct $name(pub $t);
        impl $name {
            pub const NONE: $name = $name(0);
            $($(#[$fm])* pub const $flag: $name = $name(1 << $bit);)*
            /// Contient tous les drapeaux de `other` (et `other` n'est pas vide).
            #[inline]
            pub fn has(self, other: $name) -> bool {
                self.0 & other.0 == other.0 && other.0 != 0
            }
            /// Contient au moins un des drapeaux de `other`.
            #[inline]
            pub fn any(self, other: $name) -> bool {
                self.0 & other.0 != 0
            }
            #[inline]
            pub fn without(self, other: $name) -> $name {
                $name(self.0 & !other.0)
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
    /// Sous-types (CR 8.2) et sous-types d'arme.
    Subtypes(u64) {
        ATTACK = 0, ONE_HAND = 1, TWO_HAND = 2, HEAD = 3, CHEST = 4, ARMS = 5,
        LEGS = 6, OFF_HAND = 7, AURA = 8, ITEM = 9, YOUNG = 10, ALLY = 11,
        LANDMARK = 12, AFFLICTION = 13, ASH = 14, INVOCATION = 15, CONSTRUCT = 16,
        QUIVER = 17, FIGMENT = 18, ARROW = 19, TRAP = 20,
        SWORD = 21, AXE = 22, DAGGER = 23, HAMMER = 24, CLUB = 25, STAFF = 26,
        SCEPTER = 27, BOOK = 28, DRAGON = 29, DEMON = 30, DISEASE = 31,
    }
}

flags! {
    /// Supertypes : classes et talents (CR 2.11), pour les filtres
    /// « attaque Ninja », « carte Earth », etc.
    Supertypes(u32) {
        NINJA = 0, WARRIOR = 1, GUARDIAN = 2, WIZARD = 3, RUNEBLADE = 4,
        BRUTE = 5, ILLUSIONIST = 6, ASSASSIN = 7, SHADOW = 8, EARTH = 9,
        ICE = 10, LIGHTNING = 11, ELEMENTAL = 12, DRACONIC = 13, CHAOS = 14,
    }
}

flags! {
    /// Mots-clés de capacité gérés directement par le moteur (CR 8.3).
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
        /// CR 8.3.34
        GUARDWELL = 5,
        /// CR 8.3.24
        STEALTH = 6,
        /// CR 8.3.13
        PHANTASM = 7,
        /// CR 8.3.22
        OVERPOWER = 8,
        /// CR 8.3.14 : « this can be attacked » + « when this becomes the
        /// target of an attack, destroy this ».
        SPECTRA = 9,
        /// CR 8.3.28 : « While this is in your arsenal, you may defend with it. »
        AMBUSH = 10,
        /// CR 8.3.21 : « If this would be put into a graveyard from anywhere,
        /// instead remove it from the game. »
        EPHEMERAL = 11,
        /// CR 8.4.2 : crush est un **label** (« Crush - When this deals 4 or
        /// more damage, … ») et non un mot-clé ; ce drapeau dit seulement que
        /// la carte « a crush » (Bravo, Crash and Bash). La capacité elle-même
        /// est une capacité déclenchée de la carte.
        CRUSH = 12,
        /// CR 8.3.42 : « This enters the arena with 2 suspense counters »,
        /// « At the start of your turn, remove a suspense counter from this »,
        /// « When this has no suspense counters, destroy it ».
        SUSPENSE = 13,
        /// CR 8.3.11 : « While this is in your banished zone, at the beginning
        /// of your end phase, lose 1{h}. » Seulement si la carte est publique
        /// (face visible) dans la zone de bannissement (CR 8.3.11a).
        BLOOD_DEBT = 14,
    }
}

flags! {
    /// Traits (CR 2.13) : mots non fonctionnels qui désignent un groupe de
    /// cartes (CR 2.13.4, « a random Agent of Chaos »).
    Traits(u8) {
        AGENT_OF_CHAOS = 0,
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

impl Subtypes {
    /// Sous-types qui font d'une carte de deck un permanent (CR 1.3.3).
    pub const PERMANENT: Subtypes = Subtypes(
        Subtypes::AFFLICTION.0
            | Subtypes::ALLY.0
            | Subtypes::ASH.0
            | Subtypes::AURA.0
            | Subtypes::CONSTRUCT.0
            | Subtypes::FIGMENT.0
            | Subtypes::INVOCATION.0
            | Subtypes::ITEM.0
            | Subtypes::LANDMARK.0,
    );
}

/// Couleur (barre de couleur) d'une carte de deck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Color {
    Red,
    Yellow,
    Blue,
}

/// Vitesse d'une capacité activée (le TYPE de CR 5.2.1b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speed {
    Action,
    Instant,
    AttackReaction,
    DefenseReaction,
}

/// Une capacité activée : « [LIMIT] TYPE — COÛT : EFFETS » (CR 5.2).
#[derive(Clone, Debug)]
pub struct ActivatedDef {
    /// Texte court pour l'interface.
    pub label: &'static str,
    pub speed: Speed,
    /// Coût en ressources.
    pub cost: u8,
    /// Coût {t} : engager la source.
    pub tap: bool,
    /// « Once per Turn ».
    pub once_per_turn: bool,
    /// La source doit être dans la main (ex. « Instant — Discard this »).
    pub from_hand: bool,
    /// Capacité « Attack » : crée une attaque-proxy (CR 1.4.3, 8.3.1).
    pub attack: bool,
    /// La couche activée (ou l'attaque-proxy) a go again.
    pub go_again: bool,
    /// Condition d'activation (CR 5.2.1e).
    pub condition: Option<fn(&GameState, PlayerId, ObjectId) -> bool>,
    pub targets: Vec<TargetSpec>,
    /// Coûts d'effet (ex. « destroy this », « discard this »), payés à CR 5.1.9.
    pub effect_cost: Vec<Op>,
    /// Effets de résolution.
    pub script: Vec<Op>,
}

impl Default for ActivatedDef {
    fn default() -> Self {
        ActivatedDef {
            label: "",
            speed: Speed::Action,
            cost: 0,
            tap: false,
            once_per_turn: false,
            from_hand: false,
            attack: false,
            go_again: false,
            condition: None,
            targets: Vec::new(),
            effect_cost: Vec::new(),
            script: Vec::new(),
        }
    }
}

/// Une capacité déclenchée (CR 6.6).
#[derive(Clone, Debug)]
pub struct TriggerDef {
    /// La condition de déclenchement : `me` est l'objet qui porte la capacité.
    pub when: fn(&GameState, ObjectId, &Event) -> bool,
    /// Le déclenchement fonctionne aussi hors de l'arène (ex. « when this is
    /// put into your graveyard »). Par défaut : seulement dans l'arène et sur la pile.
    pub anywhere: bool,
    pub targets: Vec<TargetSpec>,
    pub script: Vec<Op>,
}

/// Modification appliquée par un effet (statique ou continu).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Delta {
    pub power: i16,
    pub defense: i16,
    pub grants: Keywords,
    /// « gets arcane barrier N / ward N / spellvoid N » (ex. Sand Cover,
    /// Skera Strapping, Mask of the Swarming Claw).
    pub arcane_barrier: u8,
    pub ward: u8,
    pub spellvoid: u8,
    /// « loses all abilities » (ex. Humble, Crush Confidence).
    pub lose_abilities: bool,
    /// « loses and can't gain [mots-clés] » (ex. Unflinching Foothold :
    /// dominate). Retirés même s'ils sont accordés par un autre effet.
    pub loses: Keywords,
}

/// Coût additionnel en ressources (CR 5.4.4a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdditionalCost {
    pub resources: u8,
    pub optional: bool,
}

/// Carte double (split-card, CR 9.2) : les définitions de la carte entière
/// et de chacun de ses côtés. Chaque définition concernée porte la même
/// information. La carte entière a les propriétés des deux côtés (CR 9.2.2) ;
/// sur la pile, elle n'a que celles du côté choisi (CR 9.2.3), ou des deux si
/// elle est fusionnée (meld, CR 8.3.38c).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Split {
    pub whole: CardId,
    pub left: CardId,
    pub right: CardId,
    /// La définition « fusionnée » (les deux côtés), si la carte a meld.
    pub melded: Option<CardId>,
}

/// Carte retournable (flip-card, CR 9.1.3 : invocation, figment, construct) :
/// les définitions de ses deux faces. Les deux définitions portent la même
/// information. Hors de l'arène, la face avant est active (CR 9.1.3a-b) ;
/// une invocation qui se résout entre dans l'arène face arrière active
/// (CR 8.2.13a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flip {
    pub front: CardId,
    pub back: CardId,
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
    pub supertypes: Supertypes,
    pub keywords: Keywords,
    /// Traits (CR 2.13), ex. Agent of Chaos.
    pub traits: Traits,
    /// Le coût contient X (CR 5.1.3a).
    pub x_cost: bool,
    /// Coût imprimé « XX » : le coût de base est X+X (CR 2.1, exemple de
    /// Spark of Genius ; ex. Ice Eternal).
    pub x_twice: bool,
    /// La carte a un effet de dégâts arcaniques (« the next card you play this
    /// turn with an arcane damage effect », Crucible of Aetherweave).
    pub arcane_damage: bool,
    /// Modes : « Choose 1 » (CR 1.7.5) ; libellés des modes.
    pub modes: Vec<&'static str>,
    /// Cibles déclarées en jouant la carte (CR 5.1.4).
    pub targets: Vec<TargetSpec>,
    pub additional_cost: Option<AdditionalCost>,
    /// Coûts d'effet additionnels (ex. « banish 3 random cards from your graveyard »).
    pub effect_cost: Vec<Op>,
    /// Capacités de résolution (CR 5.3.4).
    pub resolve: Vec<Op>,
    pub triggers: Vec<TriggerDef>,
    /// Capacités déclenchées que cette carte **accorde** à d'autres objets
    /// (« target attack gets "When this hits, …" »), voir
    /// `GameState::grant_trigger`. Elles ne fonctionnent pas sur la carte elle-même.
    pub granted: Vec<TriggerDef>,
    pub activated: Vec<ActivatedDef>,
    /// « Play this only if … » (CR 5.1.5).
    pub play_condition: Option<fn(&GameState, PlayerId, ObjectId) -> bool>,
    /// Modification du coût (négatif = réduction), CR 5.1.6.
    pub cost_mod: Option<fn(&GameState, PlayerId, ObjectId) -> i16>,
    /// Effets statiques sur la carte elle-même (« this gets +1{p} if … »).
    pub self_mod: Option<fn(&GameState, ObjectId) -> Delta>,
    /// Effets statiques d'une arme sur ses attaques (« this card's attacks get
    /// +1{p} and go again »), appliqués en continu à l'attaque-proxy. Argument :
    /// l'arme.
    pub attack_mod: Option<fn(&GameState, ObjectId) -> Delta>,
    /// Effets statiques sur d'autres objets, actifs quand la source est dans
    /// l'arène (héros, équipements, auras). Arguments : source, objet modifié.
    pub aura: Option<fn(&GameState, ObjectId, ObjectId) -> Delta>,
    /// Valeurs imprimées des préventions au choix (CR 8.3.8, 8.3.20, 8.3.15).
    /// 0 = la carte n'a pas la capacité.
    pub arcane_barrier: u8,
    pub ward: u8,
    pub spellvoid: u8,
    /// Effets de remplacement statiques (« If … would …, instead … », CR 6.4).
    pub replacements: Vec<ReplacementDef>,
    /// « Cards and abilities cost you an additional {r} to play or activate »
    /// (Frostbite, CR 8.6.10) : surcoût pour le contrôleur de ce permanent.
    pub cost_tax: u8,
    /// « If you would be dealt arcane damage, prevent N of that damage »
    /// (Pyroglyphic Protection) : prévention statique, obligatoire, à chaque
    /// événement de dégâts arcaniques (CR 6.4.10).
    pub prevent_arcane: u8,
    /// « You may play [cartes] as though they were instants » (Iyslander) :
    /// capacité statique du héros, des équipements ou des permanents.
    /// Arguments : source de la capacité, carte que son contrôleur veut jouer.
    pub play_as_instant: Option<fn(&GameState, ObjectId, ObjectId) -> bool>,
    /// Carte double (CR 9.2) : voir [`Split`].
    pub split: Option<Split>,
    /// « This enters the arena with N verse counters » (Runeblood Incantation).
    pub enter_counters: u8,
    /// « While this is face-up in any zone, it's Earth, Ice, and Lightning »
    /// (Colors of Aria) : supertypes ajoutés tant que la carte est face visible.
    pub face_up_supertypes: Supertypes,
    /// « If …, you don't lose {h} from blood debt during the end phase »
    /// (Levia) : capacité statique du héros, lue quand le déclenchement de
    /// blood debt se résout (CR 8.3.11). Argument : le joueur du héros.
    pub blood_debt_immune: Option<fn(&GameState, PlayerId) -> bool>,
    /// « Once per turn, you may play [carte filtrée] from your banished
    /// zone » (Blasmophet, the Insatiable Hunger) : permission statique d'un
    /// permanent, une fois par tour pour chaque objet qui l'a. Arguments :
    /// la source, la carte à jouer.
    pub play_from_banished: Option<fn(&GameState, ObjectId, ObjectId) -> bool>,
    /// Carte retournable (CR 9.1.3) : voir [`Flip`].
    pub flip: Option<Flip>,
    /// « Material - While this is under a permanent, that permanent gets
    /// [mots-clés] » (CR 8.4.5, ex. Ash : phantasm).
    pub material: Keywords,
    /// « [Alliés filtrés] you control get "Once per Turn Action -- 0:
    /// Attack" » (Storm of Sandikai) : capacité statique d'une arme, d'un
    /// équipement, du héros ou d'un permanent. Arguments : la source, l'allié.
    /// L'allié porte la capacité d'attaque dans sa définition (`activated`),
    /// avec la condition [`GameState::attack_granted`].
    pub grants_attack: Option<fn(&GameState, ObjectId, ObjectId) -> bool>,
    /// « If this would be dealt damage, remove a counter from it to prevent N
    /// of that damage » (Yendurai) : prévention d'un allié, valeur N.
    pub counter_prevention: u8,
    /// « Opponents can't play cards or activate abilities during your turn »
    /// (Themai).
    pub silences_opponents: bool,
    /// « [Objets filtrés] cost you {r} less to activate » (Arakni,
    /// Orb-Weaver : les Graphene Chelicerae) : capacité statique du héros.
    /// Arguments : le héros, l'objet dont on active une capacité. Renvoie la
    /// réduction (CR 5.1.6).
    pub activation_discount: Option<fn(&GameState, ObjectId, ObjectId) -> u8>,
}

impl CardDef {
    pub fn is_attack_card(&self) -> bool {
        self.subtypes.has(Subtypes::ATTACK)
    }

    /// Une carte d'action (type Action) — CR 8.1.1.
    pub fn is_action(&self) -> bool {
        self.types.has(Types::ACTION)
    }

    /// Valeur de pitch, 0 si la carte n'en a pas (CR 1.8.7a).
    pub fn pitch_value(&self) -> u8 {
        self.pitch.unwrap_or(0)
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

    /// Premier exemplaire portant ce nom, quelle que soit la couleur.
    pub fn find_name(&self, name: &str) -> Option<CardId> {
        self.defs
            .iter()
            .position(|d| d.name == name)
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
        assert_eq!(t.without(Types::ACTION), Types::INSTANT);
    }
}
