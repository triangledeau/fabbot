//! Déterminisation : un monde possible, vu par un joueur.
//!
//! Le bot B2 ne connaît pas la main de l'adversaire ni l'ordre des decks. Pour
//! chercher, il imagine un monde possible : une **copie** de la partie où
//! l'information cachée a été tirée au hasard. Ce qu'il voit (sa main, les
//! cartes face visible, les tailles des zones) ne change pas.
//!
//! Ce qui est imaginé :
//! - les cartes cachées de l'adversaire (sa **main**, son **deck** et son
//!   **arsenal face cachée**) sont tirées au hasard dans son **card-pool**
//!   (`GameState::card_pools`, les cartes de deck de ses 55), moins les
//!   cartes de son deck que le bot a déjà vues (jouées, pitchées, au
//!   cimetière, bannies…). Le bot ne sait donc pas quelles 40 cartes
//!   l'adversaire a choisies (choix du propriétaire, 2026-11-23 ; plus tard :
//!   la fréquence des cartes dans les decks du méta) ;
//! - l'ordre de **mon** deck est mélangé ;
//! - le générateur aléatoire de la copie reçoit une nouvelle graine (sinon
//!   le bot connaîtrait l'avenir : pioches, mélanges, tirages au hasard).
//!
//! Approximations (à affiner plus tard) :
//! - il oublie les cartes cachées qu'il a vues un instant (cartes révélées
//!   puis reprises en main ou remises dans le deck) ;
//! - les cartes bannies face cachée de l'adversaire comptent comme vues
//!   (elles ne sont pas imaginées) ;
//! - si l'adversaire a déjà montré plus de cartes que son card-pool n'en
//!   permet (cas anormal), on redistribue ses vraies cartes cachées.
//!
//! Changer l'identité d'une carte cachée est sans risque pour le moteur :
//! dans la main, le deck et l'arsenal, une carte est toujours face cachée,
//! sans compteur ni activation (CR 3.0.9 : entrer dans ces zones crée un
//! nouvel objet), et `GameState::set_card` tient à jour les listes de
//! capacités.

use fab_engine::{CardId, GameState, ObjectId, PlayerId, Rng};

/// Une copie de `g` où l'information cachée pour `moi` est tirée au hasard.
///
/// ⚠️ À appeler quand c'est à `moi` de décider (ou partie finie) : une
/// décision en attente de l'adversaire cite des cartes de sa main, que le
/// tirage aurait changées.
pub fn determiniser(g: &GameState, moi: PlayerId, rng: &mut Rng) -> GameState {
    debug_assert!(
        g.pending.as_ref().is_none_or(|d| d.player == moi),
        "déterminiser pendant une décision de l'adversaire"
    );
    let mut d = g.clone();
    let adv = moi.opponent();

    // Cartes cachées de l'adversaire : main + deck + arsenal face cachée.
    let caches = caches_adverses(&d, adv);
    // Les cartes possibles : son card-pool, moins les cartes de son deck
    // déjà vues. On part d'une liste triée : le monde imaginé ne dépend que
    // de ce que voit le bot (un test le vérifie : B2 fait le même choix
    // quelle que soit la vraie main adverse).
    let mut possibles: Vec<CardId> = d.card_pools[adv.index()].clone();
    possibles.sort_unstable();
    let (debut, fin) = d.deck_objects[adv.index()];
    let mut coherent = true;
    for i in debut..fin {
        let id = ObjectId(i);
        if caches.binary_search(&id).is_ok() {
            continue;
        }
        let vue = face_avant(&d, d.obj(id).card);
        match possibles.binary_search(&vue) {
            Ok(k) => {
                possibles.remove(k);
            }
            Err(_) => coherent = false,
        }
    }
    if !coherent || possibles.len() < caches.len() {
        // Cas anormal : on redistribue les vraies cartes cachées.
        possibles = caches.iter().map(|&id| d.obj(id).card).collect();
        possibles.sort_unstable();
    }
    rng.shuffle(&mut possibles);
    for (&id, &carte) in caches.iter().zip(&possibles) {
        if d.obj(id).card != carte {
            d.set_card(id, carte);
        }
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

/// La carte « telle qu'imprimée » : face avant d'une carte retournable
/// (CR 9.1.3), carte entière d'une carte double (CR 9.2).
fn face_avant(g: &GameState, c: CardId) -> CardId {
    let def = g.db.get(c);
    if let Some(f) = def.flip {
        f.front
    } else if let Some(sp) = def.split {
        sp.whole
    } else {
        c
    }
}

/// Main, deck et arsenal face cachée de `adv`, triés.
fn caches_adverses(g: &GameState, adv: PlayerId) -> Vec<ObjectId> {
    let ps = g.player(adv);
    let mut v: Vec<ObjectId> = Vec::with_capacity(64);
    v.extend_from_slice(&ps.hand);
    v.extend_from_slice(&ps.deck);
    v.extend(ps.arsenal.filter(|&a| g.obj(a).face_down));
    v.sort_unstable();
    v
}

/// Les objets cachés pour `moi` (pour les tests) : main et arsenal face
/// cachée adverses, et les deux decks.
pub fn objets_caches(g: &GameState, moi: PlayerId) -> Vec<ObjectId> {
    let mut v = caches_adverses(g, moi.opponent());
    v.extend_from_slice(&g.player(moi).deck);
    v
}
