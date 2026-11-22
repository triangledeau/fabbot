//! # fab-engine — le moteur de règles
//!
//! Principe central : le moteur ne sait pas *qui* joue. Il avance tout seul
//! jusqu'à ce qu'un joueur doive faire un choix, puis il s'arrête et expose une
//! [`Decision`]. Le bot ou l'humain répond en donnant l'index d'une option.
//!
//! Ce découpage permet :
//! - au bot et à l'humain d'utiliser exactement la même interface ;
//! - l'annulation illimitée (voir [`history::History`]) ;
//! - de rejouer une partie à l'identique à partir de la graine et des choix.
//!
//! État actuel : squelette (étape 0). Les règles arrivent à l'étape 1.

pub mod decision;
pub mod format;
pub mod game;
pub mod history;
pub mod ids;
pub mod rng;
pub mod zones;

pub use decision::{Choice, Decision, DecisionKind};
pub use format::FormatConfig;
pub use game::{GameState, Outcome};
pub use history::History;
pub use ids::{CardId, ObjectId, PlayerId};
pub use rng::Rng;
pub use zones::Zone;
