//! Paramètres du format de jeu, lus depuis `config/silver_age.toml`.
//!
//! Les valeurs sont `Option` tant qu'elles n'ont pas été vérifiées sur le
//! document officiel de LSS : on préfère « inconnu » à « faux ».

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct FormatConfig {
    pub name: String,
    /// `true` seulement quand chaque valeur a été vérifiée sur la source officielle.
    pub verified: bool,
    /// Lien ou référence du document officiel utilisé.
    pub source: Option<String>,
    pub deck_size: Option<u16>,
    pub card_pool_max: Option<u16>,
    pub max_copies: Option<u8>,
    pub allowed_rarities: Option<Vec<String>>,
    pub hero_type: Option<String>,
    pub games_to_win_match: Option<u8>,
}

impl FormatConfig {
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_fichier_du_depot_se_lit() {
        let text = include_str!("../../../config/silver_age.toml");
        let cfg = FormatConfig::from_toml(text).expect("silver_age.toml invalide");
        assert_eq!(cfg.name, "Silver Age");
    }
}
