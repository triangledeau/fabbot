//! Les points de décision : tout choix d'un joueur passe par ici.

use crate::ids::{ObjectId, PlayerId};

/// Le type de choix demandé. La liste sera complétée au fil des étapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    /// Phase d'action : jouer une carte, activer une capacité, ou passer.
    Action,
    /// Choisir quelle(s) carte(s) pitcher pour payer un coût.
    Pitch,
    /// Choisir les cartes et équipements qui défendent.
    Defend,
    /// Fenêtre de réaction / priorité.
    Priority,
    /// Choisir une carte à mettre en arsenal en fin de tour.
    Arsenal,
    /// Choisir une cible.
    Target,
    /// Ordonner des déclenchements simultanés.
    OrderTriggers,
    /// Choix entre plusieurs modes d'une carte.
    Mode,
    /// Phase de side entre deux manches.
    Sideboard,
}

/// Une option proposée au joueur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    Pass,
    Object(ObjectId),
    Objects(Vec<ObjectId>),
    Number(u32),
}

/// Ce que le moteur attend : « le joueur X doit choisir parmi ces options ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub player: PlayerId,
    pub kind: DecisionKind,
    pub options: Vec<Choice>,
}

impl Decision {
    /// Une décision n'a de sens que s'il existe au moins une option.
    /// Quand il n'en existe qu'une, le moteur peut la prendre tout seul
    /// (ex. : passer la priorité sans aucune action légale).
    pub fn is_forced(&self) -> bool {
        self.options.len() == 1
    }
}
