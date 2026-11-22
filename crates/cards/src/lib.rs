//! # fab-cards — la base de cartes
//!
//! Chaque carte = des **données** (nom, coût, pitch, attaque, défense, types,
//! mots-clés) + un **effet** codé en Rust (étapes 2 et 3) + un **statut de
//! validation** suivi dans `docs/cards/STATUS.md`.
//!
//! On ne cherche pas à « comprendre » automatiquement le texte des cartes :
//! chaque effet est écrit et testé à la main, puis validé par un humain.

pub mod decklist;

use fab_engine::CardId;
use serde::Deserialize;

/// Où en est une carte dans le cycle de vérification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
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

/// Les données imprimées d'une carte.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CardData {
    pub name: String,
    /// Couleur / valeur de pitch (1 = rouge, 2 = jaune, 3 = bleu), si applicable.
    pub pitch: Option<u8>,
    pub cost: Option<u8>,
    pub power: Option<u8>,
    pub defense: Option<u8>,
    pub types: Vec<String>,
    pub keywords: Vec<String>,
    /// Texte officiel (avec errata), recopié pour la relecture.
    pub text: String,
    pub status: CardStatus,
}

/// La base de cartes : un `CardId` est l'index dans `cards`.
#[derive(Clone, Debug, Default)]
pub struct CardDb {
    cards: Vec<CardData>,
}

impl CardDb {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        Ok(CardDb {
            cards: serde_json::from_str(text)?,
        })
    }

    pub fn get(&self, id: CardId) -> Option<&CardData> {
        self.cards.get(id.0 as usize)
    }

    pub fn find(&self, name: &str, pitch: Option<u8>) -> Option<CardId> {
        self.cards
            .iter()
            .position(|c| c.name == name && c.pitch == pitch)
            .map(|i| CardId(i as u16))
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Carte volontairement FICTIVE : sert seulement à tester le chargement.
    const EXEMPLE: &str = r#"[
        {
            "name": "Carte de test",
            "pitch": 1, "cost": 0, "power": 3, "defense": 2,
            "types": ["Generic", "Action", "Attack"],
            "keywords": ["Go again"],
            "text": "Go again (carte fictive)",
            "status": "NotCoded"
        }
    ]"#;

    #[test]
    fn charge_une_base_json() {
        let db = CardDb::from_json(EXEMPLE).unwrap();
        assert_eq!(db.len(), 1);
        let id = db.find("Carte de test", Some(1)).unwrap();
        assert_eq!(db.get(id).unwrap().power, Some(3));
        assert!(db.find("Carte de test", Some(3)).is_none());
    }
}
