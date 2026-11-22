//! Harnais commun aux tests de règles : préparer une partie avec des decks
//! dans un ordre connu, faire des choix par leur nom, lire l'état.
#![allow(dead_code, unused_imports)]

use fab_cards::test_set::{self, id};
use fab_engine::card::{CardDb, Color};
use fab_engine::{
    Choice, DecisionKind, EndReason, GameSetup, GameState, History, Loadout, ObjectId, Outcome,
    PlayerId, Zone,
};
use std::sync::Arc;

pub const R: Option<Color> = Some(Color::Red);
pub const Y: Option<Color> = Some(Color::Yellow);
pub const B: Option<Color> = Some(Color::Blue);
pub const P0: PlayerId = PlayerId::P0;
pub const P1: PlayerId = PlayerId::P1;

pub struct Partie {
    pub g: GameState,
}

/// Une partie où P0 commence. `main0` / `main1` : les 4 premières cartes de
/// chaque deck (la main de départ), `suite0` / `suite1` : le reste du deck.
pub fn partie(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_heros(
        ["Essai Héros", "Essai Héros"],
        main0,
        suite0,
        main1,
        suite1,
        arena0,
        arena1,
    )
}

/// Comme [`partie`], en choisissant les héros.
pub fn partie_heros(
    heros: [&str; 2],
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_db(
        Arc::new(test_set::build()),
        heros,
        main0,
        suite0,
        main1,
        suite1,
        arena0,
        arena1,
    )
}

/// Base des vraies cartes + cartes d'essai (pour les adversaires et le remplissage).
pub fn base_vraie() -> CardDb {
    let mut db = fab_cards::real::build();
    test_set::add_to(&mut db);
    db
}

/// Comme [`partie_heros`], avec les vraies cartes (et les cartes d'essai).
pub fn partie_vraie(
    heros: [&str; 2],
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_db(
        Arc::new(base_vraie()),
        heros,
        main0,
        suite0,
        main1,
        suite1,
        arena0,
        arena1,
    )
}

#[allow(clippy::too_many_arguments)]
fn partie_db(
    db: Arc<CardDb>,
    heros: [&str; 2],
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    let loadout = |hero: &str,
                   main: &[(&str, Option<Color>)],
                   suite: &[(&str, Option<Color>)],
                   arena: &[&str],
                   db: &CardDb| {
        let mut deck: Vec<_> = main
            .iter()
            .chain(suite)
            .map(|&(n, c)| id(db, n, c))
            .collect();
        // Remplissage neutre : des Essai Rempart (blocs, injouables).
        while deck.len() < 40 {
            deck.push(id(db, "Essai Rempart", B));
        }
        Loadout {
            hero: id(db, hero, None),
            arena: arena.iter().map(|n| id(db, n, None)).collect(),
            deck,
        }
    };
    let setup = GameSetup {
        players: [
            loadout(heros[0], main0, suite0, arena0, &db),
            loadout(heros[1], main1, suite1, arena1, &db),
        ],
        chooser: P0,
        seed: 42,
        shuffle: false,
        max_turns: 100,
    };
    let mut g = GameState::new(db, setup).unwrap();
    // CR 4.1.3 : P0 choisit de commencer.
    let i = g
        .pending_decision()
        .unwrap()
        .index_of(Choice::Player(P0))
        .unwrap();
    g.apply(i).unwrap();
    Partie { g }
}

impl Partie {
    pub fn kind(&self) -> DecisionKind {
        self.g
            .pending_decision()
            .expect("une décision est attendue")
            .kind
    }

    pub fn joueur(&self) -> PlayerId {
        self.g
            .pending_decision()
            .expect("une décision est attendue")
            .player
    }

    pub fn options(&self) -> Vec<Choice> {
        self.g
            .pending_decision()
            .map(|d| d.options.clone())
            .unwrap_or_default()
    }

    pub fn noms(&self, choices: &[Choice]) -> Vec<String> {
        choices
            .iter()
            .map(|c| match c {
                Choice::Play(o)
                | Choice::Activate(o, _)
                | Choice::Pitch(o)
                | Choice::Defend(o)
                | Choice::Arsenal(o)
                | Choice::PitchToBottom(o) => {
                    format!("{c:?}={}", self.g.def(*o).name)
                }
                _ => format!("{c:?}"),
            })
            .collect()
    }

    pub fn choisir(&mut self, c: Choice) {
        let d = self
            .g
            .pending_decision()
            .expect("une décision est attendue")
            .clone();
        let i = d.index_of(c).unwrap_or_else(|| {
            panic!(
                "{c:?} indisponible pour {:?} ({:?}) ; options : {:?}",
                d.player,
                d.kind,
                self.noms(&d.options)
            )
        });
        self.g.apply(i).unwrap();
    }

    /// Une carte de la main du joueur, par nom et couleur (pas encore
    /// déclarée comme carte qui défend).
    pub fn en_main(&self, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
        *self
            .g
            .player(p)
            .hand
            .iter()
            .find(|&&c| {
                self.g.def(c).name == nom
                    && self.g.def(c).color == couleur
                    && !self.g.declared_defenders.contains(&c)
            })
            .unwrap_or_else(|| panic!("{nom} {couleur:?} absente de la main de {p:?}"))
    }

    pub fn equipement(&self, p: PlayerId, nom: &str) -> ObjectId {
        let ps = self.g.player(p);
        ps.equipment()
            .chain(ps.weapons.iter().copied())
            .find(|&c| self.g.def(c).name == nom)
            .unwrap_or_else(|| panic!("{nom} absent de l'équipement de {p:?}"))
    }

    pub fn vie(&self, p: PlayerId) -> i32 {
        self.g.player(p).life
    }

    pub fn peut(&self, c: Choice) -> bool {
        self.options().contains(&c)
    }

    /// Passe la priorité jusqu'à ce que la décision attendue ne soit plus une priorité.
    pub fn passer_jusqua(&mut self, kind: DecisionKind) {
        for _ in 0..50 {
            if self.g.is_over() || self.kind() == kind {
                return;
            }
            match self.kind() {
                DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                    self.choisir(Choice::Pass)
                }
                k => panic!("décision {k:?} inattendue en attendant {kind:?}"),
            }
        }
        panic!("{kind:?} jamais atteinte");
    }
}

pub const RIEN: &[(&str, Option<Color>)] = &[];

impl Partie {
    /// Passe la priorité (et les décisions simples) jusqu'au tour suivant.
    pub fn finir_le_tour(&mut self) {
        let tour = self.g.turn;
        for _ in 0..200 {
            if self.g.is_over() || self.g.turn != tour {
                return;
            }
            let premier = self.options()[0];
            match self.kind() {
                DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                    self.choisir(Choice::Pass)
                }
                _ => self.choisir(premier),
            }
        }
        panic!("le tour ne se termine pas");
    }
}
