//! L'état de la partie.
//!
//! Squelette : la structure réelle (zones, vie, ressources, chaîne de combat,
//! couches, effets continus) sera construite à l'étape 1.

use crate::decision::Decision;
use crate::ids::PlayerId;
use crate::rng::Rng;

/// Fin de partie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Winner(PlayerId),
    Draw,
}

/// Erreur renvoyée quand on donne une réponse impossible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    NoPendingDecision,
    InvalidChoice { index: usize, available: usize },
}

/// L'état complet d'une partie. Il doit rester **peu coûteux à copier**
/// (`Clone`), car l'annulation et la recherche du bot en font beaucoup.
#[derive(Clone, Debug)]
pub struct GameState {
    pub rng: Rng,
    pub turn: u32,
    pub active_player: PlayerId,
    pub pending: Option<Decision>,
    pub outcome: Option<Outcome>,
}

impl GameState {
    /// Nouvelle partie à partir d'une graine (même graine ⇒ même partie
    /// si les joueurs font les mêmes choix).
    pub fn new(seed: u64) -> Self {
        GameState {
            rng: Rng::new(seed),
            turn: 0,
            active_player: PlayerId::P0,
            pending: None,
            outcome: None,
        }
    }

    /// La décision en attente, s'il y en a une.
    pub fn pending_decision(&self) -> Option<&Decision> {
        self.pending.as_ref()
    }

    pub fn is_over(&self) -> bool {
        self.outcome.is_some()
    }

    /// Répond à la décision en attente en choisissant l'option `index`,
    /// puis fait avancer la partie jusqu'à la prochaine décision.
    pub fn apply(&mut self, index: usize) -> Result<(), ApplyError> {
        let decision = self.pending.as_ref().ok_or(ApplyError::NoPendingDecision)?;
        if index >= decision.options.len() {
            return Err(ApplyError::InvalidChoice {
                index,
                available: decision.options.len(),
            });
        }
        // TODO(étape 1) : appliquer le choix et faire avancer les règles.
        self.pending = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::{Choice, DecisionKind};

    #[test]
    fn un_choix_hors_limites_est_refuse() {
        let mut g = GameState::new(1);
        g.pending = Some(Decision {
            player: PlayerId::P0,
            kind: DecisionKind::Action,
            options: vec![Choice::Pass],
        });
        assert_eq!(
            g.apply(3),
            Err(ApplyError::InvalidChoice {
                index: 3,
                available: 1
            })
        );
        assert!(g.apply(0).is_ok());
        assert_eq!(g.apply(0), Err(ApplyError::NoPendingDecision));
    }
}
