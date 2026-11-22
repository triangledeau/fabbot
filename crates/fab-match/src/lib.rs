//! # fab-match — matchs en plusieurs manches et choix du card-pool
//!
//! Rappels des règles officielles (CR = Comprehensive Rules, TRP = Tournament
//! Rules and Policy) :
//! - Un match = une ou plusieurs manches consécutives entre les mêmes joueurs
//!   avec les mêmes decks (CR 4.0.2). Le nombre de manches à gagner dépend du
//!   tournoi, pas du format (TRP 3).
//! - Il n'y a pas de « sideboard » séparé en Flesh and Blood : **avant chaque
//!   manche**, chaque joueur choisit ses cartes d'arène (CR 4.1.4) puis les
//!   40 cartes de son deck (CR 4.1.6) dans son card-pool de 55 cartes. Ce choix
//!   se fait **après** avoir vu le premier joueur et l'équipement adverse
//!   (CR 4.1.3 et 4.1.5). C'est ce que le projet appelle la « phase de side ».
//!
//! TODO(étape 6) : choix de l'équipement et du deck par le bot.

use fab_engine::{Outcome, PlayerId};

/// Score d'un match « premier à N manches gagnées ».
#[derive(Clone, Debug)]
pub struct MatchScore {
    games_to_win: u8,
    wins: [u8; 2],
    results: Vec<Outcome>,
    /// Pour chaque manche jouée : le joueur qui a choisi le premier joueur.
    choosers: Vec<PlayerId>,
}

impl MatchScore {
    /// `games_to_win` : nombre de manches à gagner (dépend du tournoi).
    pub fn new(games_to_win: u8) -> Self {
        assert!(games_to_win > 0);
        MatchScore {
            games_to_win,
            wins: [0, 0],
            results: Vec::new(),
            choosers: Vec::new(),
        }
    }

    /// Qui choisit le premier joueur de la prochaine manche (CR 4.1.3) :
    /// - première manche : un joueur tiré au sort (`random_pick`) ;
    /// - ensuite : le perdant de la manche précédente ;
    /// - si la manche précédente est nulle : le même joueur que la fois précédente.
    pub fn next_chooser(&self, random_pick: PlayerId) -> PlayerId {
        match (self.results.last(), self.choosers.last()) {
            (None, _) => random_pick,
            (Some(Outcome::Winner(w)), _) => w.opponent(),
            (Some(Outcome::Draw), Some(prev)) => *prev,
            (Some(Outcome::Draw), None) => unreachable!("chaque résultat a son choisisseur"),
        }
    }

    /// Enregistre le résultat d'une manche et qui avait choisi le premier joueur.
    pub fn record(&mut self, outcome: Outcome, chooser: PlayerId) {
        assert!(self.winner().is_none(), "le match est déjà terminé");
        if let Outcome::Winner(p) = outcome {
            self.wins[p.index()] += 1;
        }
        self.results.push(outcome);
        self.choosers.push(chooser);
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
    use PlayerId as P;

    #[test]
    fn deux_manches_gagnantes() {
        let mut m = MatchScore::new(2);
        m.record(Outcome::Winner(P::P0), P::P0);
        assert_eq!(m.winner(), None);
        m.record(Outcome::Winner(P::P1), P::P1);
        assert_eq!(m.winner(), None);
        m.record(Outcome::Winner(P::P1), P::P0);
        assert_eq!(m.winner(), Some(P::P1));
        assert_eq!(m.games_played(), 3);
        assert_eq!(m.wins(P::P0), 1);
    }

    // CR 4.1.3
    #[test]
    fn premiere_manche_le_tirage_au_sort_designe_le_choisisseur() {
        let m = MatchScore::new(2);
        assert_eq!(m.next_chooser(P::P1), P::P1);
        assert_eq!(m.next_chooser(P::P0), P::P0);
    }

    // CR 4.1.3
    #[test]
    fn le_perdant_de_la_manche_precedente_choisit() {
        let mut m = MatchScore::new(2);
        m.record(Outcome::Winner(P::P0), P::P0);
        assert_eq!(m.next_chooser(P::P0), P::P1);
    }

    // CR 4.1.3
    #[test]
    fn apres_une_manche_nulle_le_meme_joueur_choisit() {
        let mut m = MatchScore::new(2);
        m.record(Outcome::Draw, P::P1);
        assert_eq!(m.next_chooser(P::P0), P::P1);
    }
}
