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
//! [`preparer_manche`] déroule cette mise en place dans l'ordre officiel :
//! choix du premier joueur (CR 4.1.3), cartes d'arène (CR 4.1.4) choisies
//! sans voir celles de l'adversaire, présentation (CR 4.1.5), puis deck
//! (CR 4.1.6) choisi en connaissant l'arène adverse. Chaque joueur répond à
//! travers le trait [`Side`] (bot ou humain).

use fab_engine::{
    CardDb, CardId, CardPool, Choice, ChoixInvalide, GameSetup, GameState, Loadout, Outcome,
    PlayerId, SetupError,
};
use std::sync::Arc;

/// Ce qu'un joueur sait quand il choisit ses cartes : son card-pool, son
/// héros et celui de l'adversaire (posés face visible, CR 4.1.2), et le
/// premier joueur (CR 4.1.3).
pub struct InfoSide<'a> {
    pub db: &'a CardDb,
    pub moi: PlayerId,
    pub pool: &'a CardPool,
    pub heros_adverse: CardId,
    pub premier: PlayerId,
    /// Taille exacte du deck (TRP 7.4 : 40).
    pub deck_size: usize,
}

/// Une façon de choisir avant la partie (bot ou humain).
pub trait Side {
    /// Nom court (affichages, classements).
    fn nom(&self) -> &str;

    /// CR 4.1.3 : le joueur désigné choisit qui commence. `info.premier` n'a
    /// pas encore de sens ici. Par défaut : commencer soi-même.
    fn premier_joueur(&mut self, info: &InfoSide) -> PlayerId {
        info.moi
    }

    /// CR 4.1.4 : les cartes d'arène, sans connaître celles de l'adversaire.
    fn arene(&mut self, info: &InfoSide) -> Vec<CardId>;

    /// CR 4.1.6 : les cartes du deck, après la présentation des cartes
    /// d'arène des deux joueurs (CR 4.1.5).
    fn deck(&mut self, info: &InfoSide, arene: &[CardId], arene_adverse: &[CardId]) -> Vec<CardId>;
}

/// Choix « dans l'ordre de la liste » (`CardPool::choix_par_ordre`) : la
/// première carte qui trouve sa place dans chaque zone et les 40 premières
/// cartes de deck. Ne tient compte ni de l'adversaire ni du premier joueur.
pub struct SideListe;

impl Side for SideListe {
    fn nom(&self) -> &str {
        "liste"
    }

    fn arene(&mut self, info: &InfoSide) -> Vec<CardId> {
        info.pool.choix_par_ordre(info.db, info.deck_size).0
    }

    fn deck(&mut self, info: &InfoSide, _: &[CardId], _: &[CardId]) -> Vec<CardId> {
        info.pool.choix_par_ordre(info.db, info.deck_size).1
    }
}

/// Le résultat de la mise en place d'une manche.
#[derive(Clone, Debug)]
pub struct Manche {
    pub loadouts: [Loadout; 2],
    /// Joueur qui choisit le premier joueur (CR 4.1.3).
    pub chooser: PlayerId,
    /// Premier joueur choisi.
    pub premier: PlayerId,
}

/// Déroule la mise en place d'une manche (CR 4.1.3 à 4.1.6). Erreur : le
/// joueur dont le choix n'est pas légal, et pourquoi.
pub fn preparer_manche(
    db: &CardDb,
    pools: [&CardPool; 2],
    sides: [&mut dyn Side; 2],
    chooser: PlayerId,
    deck_size: usize,
) -> Result<Manche, (PlayerId, ChoixInvalide)> {
    let mut sides = sides;
    let info = |moi: PlayerId, premier: PlayerId| InfoSide {
        db,
        moi,
        pool: pools[moi.index()],
        heros_adverse: pools[moi.opponent().index()].hero,
        premier,
        deck_size,
    };
    // CR 4.1.3.
    let premier = sides[chooser.index()].premier_joueur(&info(chooser, chooser));
    // CR 4.1.4 : chacun choisit sans voir le choix de l'autre.
    let arenes = PlayerId::BOTH.map(|p| sides[p.index()].arene(&info(p, premier)));
    for p in PlayerId::BOTH {
        pools[p.index()]
            .verifier_arene(db, &arenes[p.index()])
            .map_err(|e| (p, e))?;
    }
    // CR 4.1.5-4.1.6 : les arènes sont présentées, puis chacun choisit son deck.
    let decks = PlayerId::BOTH.map(|p| {
        sides[p.index()].deck(
            &info(p, premier),
            &arenes[p.index()],
            &arenes[p.opponent().index()],
        )
    });
    let mut loadouts = Vec::new();
    for p in PlayerId::BOTH {
        let lo = pools[p.index()]
            .loadout(
                db,
                arenes[p.index()].clone(),
                decks[p.index()].clone(),
                deck_size,
            )
            .map_err(|e| (p, e))?;
        loadouts.push(lo);
    }
    Ok(Manche {
        loadouts: loadouts.try_into().unwrap(),
        chooser,
        premier,
    })
}

/// Crée la partie d'une manche préparée et répond à la décision « qui
/// commence » avec le premier joueur déjà choisi (CR 4.1.3 : ce choix
/// précède celui des cartes).
pub fn lancer_manche(
    db: Arc<CardDb>,
    manche: &Manche,
    seed: u64,
    max_turns: u32,
) -> Result<GameState, SetupError> {
    let setup = GameSetup {
        players: manche.loadouts.clone(),
        chooser: manche.chooser,
        seed,
        shuffle: true,
        max_turns,
    };
    let mut g = GameState::new(db, setup)?;
    let d = g
        .pending_decision()
        .expect("décision « qui commence »")
        .clone();
    let i = d
        .index_of(Choice::Player(manche.premier))
        .expect("premier joueur proposé");
    g.apply(i).expect("premier joueur légal");
    Ok(g)
}

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

    /// Les manches jouées : résultat et joueur qui avait choisi le premier joueur.
    pub fn historique(&self) -> Vec<(Outcome, PlayerId)> {
        self.results
            .iter()
            .copied()
            .zip(self.choosers.iter().copied())
            .collect()
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
