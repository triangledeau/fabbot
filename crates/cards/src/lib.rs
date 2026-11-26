//! # fab-cards — la base de cartes
//!
//! - [`decklist`] : lecture et vérification des listes de `data/decks/` ;
//! - [`decks`] : la liste des decks du projet (un fichier par deck) et les decks maison ;
//! - [`import`] : lecture d'une liste libre (export texte de FaBrary) ;
//! - [`real`] : les vraies cartes des 9 decks (étape 2 : Hala et Dorinthea) ;
//! - [`test_set`] : cartes **fictives** servant à tester les règles du moteur
//!   (étape 1), avant le codage des vraies cartes (étapes 2 et 3).
//!
//! Chaque vraie carte aura : ses données (fichier officiel LSS), ses effets
//! codés à la main, et un statut de validation (`docs/cards/STATUS.md`).

pub mod decklist;
pub mod decks;
pub mod import;
pub mod langage;
pub mod real;
pub mod test_set;

/// Où en est une carte dans le cycle de vérification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CardStatus {
    /// Données saisies, effet pas encore codé.
    NotCoded,
    /// Effet codé.
    Coded,
    /// Tests écrits et au vert.
    Tested,
    /// Relue et validée par le propriétaire du projet.
    Validated,
}
