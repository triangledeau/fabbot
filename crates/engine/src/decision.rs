//! Les points de décision : tout choix d'un joueur passe par ici.

use crate::effects::PreventRef;
use crate::ids::{ObjectId, PlayerId};
use crate::script::Target;
use crate::zones::Zone;

/// Le type de choix demandé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    /// Le joueur choisisseur désigne le premier joueur (CR 4.1.3).
    FirstPlayer,
    /// Le joueur a la priorité : jouer une carte, activer une capacité, ou passer (CR 1.11).
    Priority,
    /// Payer un coût additionnel optionnel ? (CR 5.1.3b)
    OptionalCost,
    /// Valeur de X (CR 5.1.3a).
    ChooseX,
    /// Mode d'une capacité modale (CR 1.7.5).
    Mode,
    /// Cible (CR 1.8.5, 5.1.4).
    Target,
    /// Payer un coût en ressources : choisir la prochaine carte à pitcher (CR 1.14.2d).
    Pitch,
    /// Déclarer les cartes qui défendent, une par une, puis « Pass » pour terminer (CR 7.3.2).
    Defend,
    /// Choix demandé par une capacité pendant sa résolution.
    Effect,
    /// Le joueur actif choisit qui empile ses déclenchements en premier (CR 6.6.6b).
    TriggerFirst,
    /// Choisir le prochain déclenchement à empiler (CR 6.6.6b).
    TriggerOrder,
    /// Fin de tour : mettre une carte en arsenal ou passer (CR 4.4.3b).
    Arsenal,
    /// Fin de tour : choisir la prochaine carte du pitch à mettre sous le deck (CR 4.4.3c).
    PitchOrder,
    /// Cible d'une attaque : héros adverse, allié, objet avec spectra (CR 1.4.5).
    AttackTarget,
    /// Le joueur protégé choisit la prochaine prévention à appliquer, ou
    /// « Pass » pour refuser les préventions optionnelles restantes (CR 6.5.5,
    /// arcane barrier, spellvoid).
    Prevention,
    /// Appliquer un remplacement optionnel (« instead you may … ») ? Oui / non.
    Replacement,
}

/// Une option proposée au joueur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Passer / terminer / ne rien faire.
    Pass,
    /// Oui / non.
    Yes,
    No,
    /// Jouer cette carte (depuis la main ou l'arsenal).
    Play(ObjectId),
    /// Activer la capacité n° `i` de cet objet.
    Activate(ObjectId, u8),
    /// Pitcher cette carte de la main.
    Pitch(ObjectId),
    /// Déclarer cette carte (main ou équipement) comme carte qui défend.
    Defend(ObjectId),
    /// Mettre cette carte en arsenal.
    Arsenal(ObjectId),
    /// Mettre cette carte sous le deck maintenant.
    PitchToBottom(ObjectId),
    /// Désigner ce joueur (premier joueur, ordre des déclenchements…).
    Player(PlayerId),
    /// Une cible.
    Target(Target),
    /// Un objet (choix pendant une résolution).
    Object(ObjectId),
    /// Un nombre (X…).
    Number(u8),
    /// Un mode.
    Mode(u8),
    /// Le déclenchement n° `i` en attente.
    Trigger(u8),
    /// Appliquer cette prévention.
    Prevent(PreventRef),
    /// Une zone d'un joueur (ex. Frost Spike : « an exposed head, chest,
    /// arms, or legs zone »).
    Zone(PlayerId, Zone),
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
