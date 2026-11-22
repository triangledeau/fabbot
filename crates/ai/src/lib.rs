//! # fab-ai — les bots
//!
//! Tous les joueurs (bots et humain) implémentent [`Agent`] : ils reçoivent
//! l'état et la décision en attente, et renvoient l'index de l'option choisie.
//!
//! Paliers prévus :
//! - B0 [`RandomAgent`] : aléatoire, sert de référence et de testeur de règles ;
//! - B1 heuristique (étape 4) ;
//! - B2 ISMCTS (étape 4) ;
//! - B3 évaluation apprise par auto-jeu (étape 7).

use fab_engine::{Decision, GameState, Rng};

pub trait Agent {
    /// Nom affiché dans les classements Elo et les journaux.
    fn name(&self) -> &str;

    /// Renvoie l'index de l'option choisie dans `decision.options`.
    ///
    /// ⚠️ Un bot ne doit utiliser que l'information à laquelle son joueur a
    /// droit (pas la main adverse ni l'ordre du deck). Le filtrage de
    /// l'information cachée sera ajouté à l'étape 4.
    fn choose(&mut self, state: &GameState, decision: &Decision) -> usize;
}

/// B0 : choisit uniformément parmi les options légales.
pub struct RandomAgent {
    rng: Rng,
}

impl RandomAgent {
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            rng: Rng::new(seed),
        }
    }
}

impl Agent for RandomAgent {
    fn name(&self) -> &str {
        "B0-aleatoire"
    }

    fn choose(&mut self, _state: &GameState, decision: &Decision) -> usize {
        self.rng.below(decision.options.len() as u32) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fab_engine::{Choice, DecisionKind, PlayerId};

    #[test]
    fn le_bot_aleatoire_choisit_toujours_une_option_valide() {
        let state = GameState::new(0);
        let decision = Decision {
            player: PlayerId::P0,
            kind: DecisionKind::Action,
            options: vec![Choice::Pass, Choice::Number(1), Choice::Number(2)],
        };
        let mut bot = RandomAgent::new(9);
        for _ in 0..1000 {
            assert!(bot.choose(&state, &decision) < 3);
        }
    }
}
