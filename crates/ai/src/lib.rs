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

/// Joue une partie jusqu'au bout entre deux agents.
///
/// Si `check` est vrai, vérifie la cohérence de l'état après chaque décision
/// et renvoie une erreur au premier problème (utilisé par les tests massifs).
pub fn play_out(
    g: &mut GameState,
    agents: [&mut dyn Agent; 2],
    check: bool,
) -> Result<u32, String> {
    let [a0, a1] = agents;
    let mut decisions = 0u32;
    while let Some(d) = g.pending_decision() {
        let d = d.clone();
        let agent: &mut dyn Agent = if d.player.0 == 0 { &mut *a0 } else { &mut *a1 };
        let i = agent.choose(g, &d);
        g.apply(i).map_err(|e| format!("{e:?}"))?;
        decisions += 1;
        if check {
            let errs = g.check_invariants();
            if !errs.is_empty() {
                return Err(format!(
                    "après la décision {decisions} ({:?}) : {errs:?}",
                    d.options[i]
                ));
            }
        }
        if decisions > 200_000 {
            return Err("partie anormalement longue".into());
        }
    }
    Ok(decisions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fab_engine::{Choice, DecisionKind, PlayerId};

    #[test]
    fn le_bot_aleatoire_choisit_toujours_une_option_valide() {
        let db = std::sync::Arc::new(fab_engine::CardDb::new());
        let _ = db;
        let decision = Decision {
            player: PlayerId::P0,
            kind: DecisionKind::Priority,
            options: vec![Choice::Pass, Choice::Pass, Choice::Pass],
        };
        let mut bot = RandomAgent::new(9);
        for _ in 0..1000 {
            let i = bot.rng.below(decision.options.len() as u32) as usize;
            assert!(i < 3);
        }
    }
}
