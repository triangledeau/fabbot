//! Déterminisation : un monde possible, vu par un joueur.
//!
//! Le bot B2 ne connaît pas la main de l'adversaire ni l'ordre des decks. Pour
//! chercher, il imagine un monde possible : une **copie** de la partie où
//! l'information cachée a été redistribuée au hasard. Ce qu'il voit
//! (sa main, les cartes face visible, les tailles des zones) ne change pas.
//!
//! Ce qui est redistribué :
//! - les cartes de l'adversaire dans sa **main**, son **deck** et son
//!   **arsenal face cachée** sont mélangées ensemble puis reposées dans ces
//!   zones (mêmes tailles qu'avant) ;
//! - l'ordre de **mon** deck est mélangé ;
//! - le générateur aléatoire de la copie reçoit une nouvelle graine (sinon
//!   le bot connaîtrait l'avenir : pioches, mélanges, tirages au hasard).
//!
//! Approximations (à affiner plus tard) :
//! - le bot connaît la **liste des 40 cartes** de l'adversaire (comme une
//!   liste de deck publique), pas leur position ;
//! - il oublie les cartes cachées qu'il a vues un instant (cartes révélées
//!   puis reprises en main ou remises dans le deck) ;
//! - les cartes bannies face cachée de l'adversaire ne sont pas mélangées.
//!
//! Les objets échangés sont interchangeables pour le moteur : dans la main,
//! le deck et l'arsenal, une carte est toujours face cachée, sans compteur
//! ni activation (CR 3.0.9 : entrer dans ces zones crée un nouvel objet).

use fab_engine::{GameState, ObjectId, PlayerId, Rng, Zone};

/// Une copie de `g` où l'information cachée pour `moi` est tirée au hasard.
///
/// ⚠️ À appeler quand c'est à `moi` de décider (ou partie finie) : une
/// décision en attente de l'adversaire cite des cartes de sa main, que la
/// redistribution aurait déplacées.
pub fn determiniser(g: &GameState, moi: PlayerId, rng: &mut Rng) -> GameState {
    debug_assert!(
        g.pending.as_ref().is_none_or(|d| d.player == moi),
        "déterminiser pendant une décision de l'adversaire"
    );
    let mut d = g.clone();
    let adv = moi.opponent();

    // Cartes cachées de l'adversaire : main + deck + arsenal face cachée.
    let arsenal_cache = d.player(adv).arsenal.filter(|&a| d.obj(a).face_down);
    let mut cartes: Vec<ObjectId> = Vec::with_capacity(64);
    cartes.extend_from_slice(&d.player(adv).hand);
    cartes.extend_from_slice(&d.player(adv).deck);
    cartes.extend(arsenal_cache);
    // Trier avant de mélanger : le monde imaginé ne dépend alors que de
    // l'ensemble des cartes cachées, pas de leur place réelle (un test le
    // vérifie : B2 fait le même choix quelle que soit la vraie main adverse).
    cartes.sort_unstable();
    rng.shuffle(&mut cartes);
    let nh = d.player(adv).hand.len();
    let nd = d.player(adv).deck.len();
    {
        let ps = d.player_mut(adv);
        ps.hand.clear();
        ps.hand.extend_from_slice(&cartes[..nh]);
        ps.deck.clear();
        ps.deck.extend_from_slice(&cartes[nh..nh + nd]);
        if arsenal_cache.is_some() {
            ps.arsenal = Some(cartes[nh + nd]);
        }
    }
    for (i, &id) in cartes.iter().enumerate() {
        let zone = if i < nh {
            Zone::Hand
        } else if i < nh + nd {
            Zone::Deck
        } else {
            Zone::Arsenal
        };
        let o = d.obj_mut(id);
        o.zone = zone;
        o.face_down = true;
    }

    // Ordre de mon deck.
    let deck = &mut d.player_mut(moi).deck;
    deck.sort_unstable();
    rng.shuffle(deck);

    // Hasard futur inconnu.
    d.rng = Rng::new(rng.next_u64());
    d.record = None;
    d
}

/// Les objets cachés pour `moi` (pour les tests) : main et arsenal face
/// cachée adverses, et les deux decks.
pub fn objets_caches(g: &GameState, moi: PlayerId) -> Vec<ObjectId> {
    let adv = moi.opponent();
    let mut v: Vec<ObjectId> = g.player(adv).hand.clone();
    v.extend_from_slice(&g.player(adv).deck);
    v.extend(g.player(adv).arsenal.filter(|&a| g.obj(a).face_down));
    v.extend_from_slice(&g.player(moi).deck);
    v
}
