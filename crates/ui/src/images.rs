//! Image officielle et nom français de chaque carte codée (lot 1 du plan de
//! rapatriement, `docs/rapatriement-arsenal.md`).
//!
//! La table `data/cards/images.tsv` est produite par
//! `tools/images_cartes.py` à partir du fichier officiel LSS ; elle ne
//! contient que des codes d'impression (`FR_SBR016`) : la page charge les
//! images en ligne, sur le serveur d'images de LSS. Une carte absente de la
//! table garde son cadre dessiné.

use fab_engine::{CardDef, Color};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Le fichier, intégré au programme.
const TABLE: &str = include_str!("../../../data/cards/images.tsv");

/// Adresse d'une image, en ajoutant la taille (`normal` ou `large`) et le
/// code : `{IMAGES}normal/FR_SBR016.webp`.
pub const IMAGES: &str = "https://legendstory-production-s3-public.s3.amazonaws.com/media/cards/";

/// Ce que la table dit d'une carte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Image {
    /// Code de l'image (`FR_` au début = version française).
    pub code: &'static str,
    /// Nom imprimé sur la version française (vide si la carte n'a pas de VF).
    pub nom_fr: &'static str,
}

fn couleur(c: Option<Color>) -> &'static str {
    match c {
        Some(Color::Red) => "Red",
        Some(Color::Yellow) => "Yellow",
        Some(Color::Blue) => "Blue",
        None => "",
    }
}

/// Clé de la table : `nom<TAB>couleur`.
fn table() -> &'static HashMap<String, Image> {
    static T: OnceLock<HashMap<String, Image>> = OnceLock::new();
    T.get_or_init(|| {
        TABLE
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| {
                let mut c = l.split('\t');
                let (nom, coul, code) = (c.next()?, c.next()?, c.next()?);
                let nom_fr = c.next().unwrap_or("");
                Some((format!("{nom}\t{coul}"), Image { code, nom_fr }))
            })
            .collect()
    })
}

/// L'image d'une carte (par son nom anglais et sa couleur).
pub fn image(d: &CardDef) -> Option<Image> {
    image_par_nom(&d.name, d.color)
}

/// L'image d'une carte par son nom anglais et sa couleur.
pub fn image_par_nom(nom: &str, c: Option<Color>) -> Option<Image> {
    table().get(&format!("{nom}\t{}", couleur(c))).copied()
}

/// Les champs `image` (code) et `nom_fr` à ajouter au JSON d'une carte
/// (`null` / absents si la carte n'est pas dans la table).
pub fn champs(d: &CardDef) -> (Option<&'static str>, Option<&'static str>) {
    match image(d) {
        Some(i) => (Some(i.code), (!i.nom_fr.is_empty()).then_some(i.nom_fr)),
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_carte_avec_vf_a_son_image_francaise_et_son_nom_francais() {
        let i = image_par_nom("Chokeslam", Some(Color::Red)).expect("dans la table");
        assert!(i.code.starts_with("FR_"), "{}", i.code);
        assert_eq!(i.nom_fr, "Étouffclac");
    }

    #[test]
    fn la_couleur_distingue_les_cartes_de_meme_nom() {
        let r = image_par_nom("Chokeslam", Some(Color::Red)).unwrap();
        let b = image_par_nom("Chokeslam", Some(Color::Blue)).unwrap();
        assert_ne!(r.code, b.code);
        assert!(image_par_nom("Chokeslam", Some(Color::Yellow)).is_none());
    }

    #[test]
    fn une_carte_inconnue_n_a_pas_d_image() {
        assert!(image_par_nom("Carte qui n'existe pas", None).is_none());
    }

    #[test]
    fn les_codes_ne_contiennent_que_des_caracteres_sans_danger() {
        // Le code finit dans une adresse : lettres, chiffres, `_` et `-` seulement.
        for i in table().values() {
            assert!(
                !i.code.is_empty()
                    && i.code
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "code d'image suspect : {:?}",
                i.code
            );
        }
    }
}
