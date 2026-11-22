//! # fab-engine — le moteur de règles
//!
//! Principe central : le moteur ne sait pas *qui* joue. Il avance tout seul
//! jusqu'à ce qu'un joueur doive faire un choix, puis il s'arrête et expose une
//! [`Decision`]. Le bot ou l'humain répond en donnant l'index d'une option
//! ([`GameState::apply`]).
//!
//! Organisation :
//! - [`card`] : définitions des cartes (données + capacités) ;
//! - [`state`] : l'état de la partie (objets, zones, joueurs, combat) ;
//! - [`flow`] : le déroulement (phases, priorité, couches, combat) ;
//! - [`play`] : jouer une carte, activer une arme, payer les coûts ;
//! - [`script`] : les programmes des capacités (suspendables) ;
//! - [`events`] : événements, journal du tour, déclenchements ;
//! - [`effects`] : primitives d'effet (piocher, dégâts et préventions, détruire…) ;
//! - [`replace`] : effets de remplacement (« instead ») ;
//! - [`invariants`] : contrôles de cohérence utilisés par les tests.

pub mod card;
pub mod decision;
pub mod effects;
pub mod events;
pub mod flow;
pub mod format;
pub mod history;
pub mod ids;
pub mod invariants;
pub mod play;
pub mod replace;
pub mod rng;
pub mod script;
pub mod setup;
pub mod state;
pub mod zones;

pub use card::{
    ActivatedDef, CardDb, CardDef, Color, Delta, Keywords, Speed, Subtypes, Supertypes, TriggerDef,
    Types,
};
pub use decision::{Choice, Decision, DecisionKind};
pub use effects::{DamageEvent, PreventRef};
pub use events::Event;
pub use format::FormatConfig;
pub use history::History;
pub use ids::{CardId, ObjRef, ObjectId, PlayerId};
pub use replace::{Pending, ReplaceKind, Replacement, ReplacementDef, Who};
pub use rng::Rng;
pub use script::{Ask, Ctx, Op, ScriptRef, Target, TargetSpec};
pub use setup::{GameSetup, Loadout, SetupError};
pub use state::{
    ApplyError, CostReduction, EndReason, GameState, Outcome, Prevention, RuleTrigger, Until,
    WeaponEffect, WeaponEffectKind,
};
pub use zones::Zone;
