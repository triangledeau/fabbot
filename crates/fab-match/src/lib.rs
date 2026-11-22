//! # fab-match — matchs en plusieurs manches et phase de side
//!
//! TODO(étape 6) :
//! - règle officielle pour savoir qui commence chaque manche (à vérifier) ;
//! - phase de side : application des guides de side calculés par simulation ;
//! - gestion des égalités (Draw) selon les règles de tournoi.

use fab_engine::{Outcome, PlayerId};

/// Score d'un match « au meilleur de N manches ».
#[derive(Clone, Debug)]
pub struct MatchScore {
    games_to_win: u8,
    wins: [u8; 2],
    results: Vec<Outcome>,
}

impl MatchScore {
    /// `games_to_win` : 2 pour un match en 2 manches gagnantes.
    pub fn new(games_to_win: u8) -> Self {
        assert!(games_to_win > 0);
        MatchScore {
            games_to_win,
            wins: [0, 0],
            results: Vec::new(),
        }
    }

    /// Enregistre le résultat d'une manche.
    pub fn record(&mut self, outcome: Outcome) {
        assert!(self.winner().is_none(), "le match est déjà terminé");
        if let Outcome::Winner(p) = outcome {
            self.wins[p.index()] += 1;
        }
        self.results.push(outcome);
    }

    pub fn wins(&self, p: PlayerId) -> u8 {
        self.wins[p.index()]
    }

    pub fn games_played(&self) -> usize {
        self.results.len()
    }

    /// Le vainqueur du match, s'il est décidé.
    pub fn winner(&self) -> Option<PlayerId> {
        [PlayerId::P0, PlayerId::P1]
            .into_iter()
            .find(|p| self.wins[p.index()] >= self.games_to_win)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deux_manches_gagnantes() {
        let mut m = MatchScore::new(2);
        m.record(Outcome::Winner(PlayerId::P0));
        assert_eq!(m.winner(), None);
        m.record(Outcome::Winner(PlayerId::P1));
        assert_eq!(m.winner(), None);
        m.record(Outcome::Winner(PlayerId::P1));
        assert_eq!(m.winner(), Some(PlayerId::P1));
        assert_eq!(m.games_played(), 3);
        assert_eq!(m.wins(PlayerId::P0), 1);
    }
}
