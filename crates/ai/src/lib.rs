//! # fab-ai — les bots
//!
//! Tous les joueurs (bots et humain) implémentent [`Agent`] : ils reçoivent
//! l'état et la décision en attente, et renvoient l'index de l'option choisie.
//!
//! Paliers prévus :
//! - B0 [`RandomAgent`] : aléatoire, sert de référence et de testeur de règles ;
//! - B1 [`HeuristicAgent`] : heuristique simple, sans recherche ;
//! - B2 [`IsmctsAgent`] : recherche ISMCTS (mondes imaginés + parties simulées par B1) ;
//! - B3 évaluation apprise par auto-jeu (étape 7).

use fab_engine::{Decision, GameState, Rng};

pub mod b1;
pub mod b2;
pub mod determinisation;
pub mod elo;
pub mod side;
pub mod vue;

pub use b1::HeuristicAgent;
pub use b2::IsmctsAgent;

/// Les bots disponibles, par leur nom court (`b0`, `b1`, `b2`). B2 accepte
/// aussi un nombre d'itérations par décision : `b2-1000`.
pub const BOTS: [&str; 3] = ["b0", "b1", "b2"];

/// Crée un bot à partir de son nom court ; `graine` règle son hasard.
///
/// Variantes de B2 (voir [`b2::ReglagesB2::depuis_nom`]) : `b2-1000`
/// (1000 itérations par décision), `b2-500ms` (une demi-seconde par
/// décision), `b2-200t4` (parties simulées arrêtées après 4 tours),
/// `b2-200a` (parties simulées par B0).
pub fn creer_agent(nom: &str, graine: u64) -> Option<Box<dyn Agent + Send>> {
    let nom = nom.to_lowercase();
    match nom.as_str() {
        "b0" => Some(Box::new(RandomAgent::new(graine))),
        "b1" => Some(Box::new(HeuristicAgent::new())),
        "b2" => Some(Box::new(IsmctsAgent::new(graine))),
        _ => {
            let reglages = b2::ReglagesB2::depuis_nom(nom.strip_prefix("b2-")?)?;
            Some(Box::new(IsmctsAgent::avec(graine, reglages)))
        }
    }
}

/// Le nom désigne-t-il un bot connu ?
pub fn bot_connu(nom: &str) -> bool {
    creer_agent(nom, 0).is_some()
}

pub trait Agent {
    /// Nom affiché dans les classements Elo et les journaux.
    fn name(&self) -> &str;

    /// Renvoie l'index de l'option choisie dans `decision.options`.
    ///
    /// ⚠️ Un bot ne doit utiliser que l'information à laquelle son joueur a
    /// droit (pas la main adverse ni l'ordre du deck) : les bots heuristiques
    /// ne lisent `state` qu'à travers une [`vue::Vue`].
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
