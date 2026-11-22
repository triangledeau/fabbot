//! Paramètres du format de jeu, lus depuis `config/silver_age.toml`.
//!
//! Source : Tournament Rules and Policy, section 7.4 (Silver Age Format).
//! Le nombre de manches d'un match n'est **pas** fixé par le format : il dépend
//! du tournoi (TRP 3), il est donc réglé dans `fab-match`, pas ici.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct FormatConfig {
    pub name: String,
    /// `true` seulement quand chaque valeur a été vérifiée sur la source officielle.
    pub verified: bool,
    /// Référence du document officiel utilisé et date de consultation.
    pub source: String,
    /// Sous-type obligatoire du héros (TRP 7.4 : « young »).
    pub hero_subtype: String,
    /// Nombre maximal de cartes d'arène + cartes de deck dans le card-pool.
    pub card_pool_max: u16,
    /// Taille exacte du deck au début de la partie.
    pub deck_size: u16,
    /// Copies maximales d'une carte unique (unicité = nom + pitch, CR 2.7.1).
    pub max_copies: u8,
    /// Copies maximales d'une carte « restricted » (TRP 7).
    pub max_copies_restricted: u8,
    /// Raretés légales (TRP 7.4).
    pub legal_rarities: Vec<String>,
    /// Cartes bannies (« Card Legality Policy »).
    pub banned: Vec<String>,
    /// Cartes restreintes à `max_copies_restricted` exemplaire(s).
    pub restricted: Vec<String>,
    /// Héros « mis au banc » pour la saison en cours.
    pub benched_heroes: Vec<String>,
    /// `true` quand les listes banned/restricted/benched ont été vérifiées.
    pub legality_list_verified: bool,
}

impl FormatConfig {
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Nombre maximal d'exemplaires autorisés pour une carte de ce nom.
    pub fn copies_allowed(&self, card_name: &str) -> u8 {
        if self.banned.iter().any(|n| n == card_name) {
            0
        } else if self.restricted.iter().any(|n| n == card_name) {
            self.max_copies_restricted
        } else {
            self.max_copies
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn silver_age() -> FormatConfig {
        let text = include_str!("../../../config/silver_age.toml");
        FormatConfig::from_toml(text).expect("silver_age.toml invalide")
    }

    #[test]
    fn le_fichier_silver_age_correspond_au_trp_7_4() {
        let cfg = silver_age();
        assert_eq!(cfg.name, "Silver Age");
        assert!(cfg.verified);
        assert_eq!(cfg.hero_subtype, "Young");
        assert_eq!(cfg.card_pool_max, 55);
        assert_eq!(cfg.deck_size, 40);
        assert_eq!(cfg.max_copies, 2);
        assert_eq!(cfg.legal_rarities, vec!["Basic", "Common", "Rare"]);
    }

    #[test]
    fn la_liste_officielle_silver_age_est_renseignee() {
        let cfg = silver_age();
        assert!(cfg.legality_list_verified);
        assert_eq!(cfg.banned.len(), 40);
        assert_eq!(cfg.copies_allowed("Flic Flak"), 0);
        assert_eq!(
            cfg.benched_heroes,
            vec!["Chane", "Briar", "Oldhim", "Oscilio"]
        );
    }

    #[test]
    fn cartes_bannies_et_restreintes() {
        let mut cfg = silver_age();
        cfg.banned = vec!["Carte bannie fictive".into()];
        cfg.restricted = vec!["Carte restreinte fictive".into()];
        assert_eq!(cfg.copies_allowed("Carte bannie fictive"), 0);
        assert_eq!(cfg.copies_allowed("Carte restreinte fictive"), 1);
        assert_eq!(cfg.copies_allowed("Carte normale"), 2);
    }
}
