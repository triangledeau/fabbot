//! Les points de décision : tout choix d'un joueur passe par ici.

use crate::ids::{ObjectId, PlayerId};

/// Le type de choix demandé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    /// Le joueur choisisseur désigne le premier joueur (CR 4.1.3).
    FirstPlayer,
    /// Le joueur a la priorité : jouer une carte, activer une capacité, ou passer (CR 1.11).
    Priority,
    /// Payer un coût en ressources : choisir la prochaine carte à pitcher (CR 1.14.2d).
    Pitch,
    /// Déclarer les cartes qui défendent, une par une, puis « Pass » pour terminer (CR 7.3.2).
    Defend,
    /// Fin de tour : mettre une carte en arsenal ou passer (CR 4.4.3b).
    Arsenal,
    /// Fin de tour : choisir la prochaine carte du pitch à mettre sous le deck (CR 4.4.3c).
    PitchOrder,
}

/// Une option proposée au joueur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Passer / terminer / ne rien faire.
    Pass,
    /// Jouer cette carte (depuis la main ou l'arsenal).
    Play(ObjectId),
    /// Activer la capacité d'attaque de cette arme.
    Activate(ObjectId),
    /// Pitcher cette carte de la main.
    Pitch(ObjectId),
    /// Déclarer cette carte (main ou équipement) comme carte qui défend.
    Defend(ObjectId),
    /// Mettre cette carte en arsenal.
    Arsenal(ObjectId),
    /// Mettre cette carte sous le deck maintenant.
    PitchToBottom(ObjectId),
    /// Désigner ce joueur comme premier joueur.
    FirstPlayer(PlayerId),
}

/// Ce que le moteur attend : « le joueur X doit choisir parmi ces options ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub player: PlayerId,
    pub kind: DecisionKind,
    pub options: Vec<Choice>,
}

impl Decision {
    pub fn index_of(&self, choice: Choice) -> Option<usize> {
        self.options.iter().position(|c| *c == choice)
    }
}
