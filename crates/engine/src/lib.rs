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
//! - [`effects`] : effets des cartes, modificateurs, déclenchements ;
//! - [`invariants`] : contrôles de cohérence utilisés par les tests.

pub mod card;
pub mod decision;
pub mod effects;
pub mod flow;
pub mod format;
pub mod history;
pub mod ids;
pub mod invariants;
pub mod play;
pub mod rng;
pub mod setup;
pub mod state;
pub mod zones;

pub use card::{CardDb, CardDef, Keywords, Op, Subtypes, Types};
pub use decision::{Choice, Decision, DecisionKind};
pub use format::FormatConfig;
pub use history::History;
pub use ids::{CardId, ObjRef, ObjectId, PlayerId};
pub use rng::Rng;
pub use setup::{GameSetup, Loadout, SetupError};
pub use state::{ApplyError, EndReason, GameState, Outcome};
pub use zones::Zone;
