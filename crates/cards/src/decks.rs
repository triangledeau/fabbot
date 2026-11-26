//! Les decks du projet : un fichier `NN_nom.txt` par deck dans `data/decks/`
//! (`01_ira.txt` → nom court `ira`).
//!
//! C'est la **seule** liste des decks : le simulateur, l'interface et les
//! tests la lisent ici. Ajouter un deck = poser son fichier dans
//! `data/decks/` une fois toutes ses cartes codées (procédure complète :
//! `docs/nouveau-deck.md`). Un deck en cours de codage attend dans
//! `data/decks/en_cours/`, qui n'est pas lu ici.

use crate::decklist::Decklist;
use crate::real;
use fab_engine::{CardDb, CardPool};
use std::path::PathBuf;

/// Un deck du projet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckDuProjet {
    /// Nom court (`ira`, `hala`…) : le nom du fichier sans le numéro.
    pub nom: String,
    pub fichier: PathBuf,
}

/// Le dossier `data/decks/`.
pub fn dossier() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks")
}

/// Le nom court d'un fichier `NN_nom.txt`, si le nom de fichier a cette forme.
pub fn nom_court(fichier: &str) -> Option<String> {
    let base = fichier.strip_suffix(".txt")?;
    let (num, nom) = base.split_once('_')?;
    (!num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) && !nom.is_empty())
        .then(|| nom.to_lowercase())
}

/// Tous les decks, dans l'ordre des fichiers.
pub fn tous() -> Vec<DeckDuProjet> {
    let mut v: Vec<DeckDuProjet> = std::fs::read_dir(dossier())
        .expect("dossier data/decks")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let f = e.file_name().to_string_lossy().into_owned();
            Some(DeckDuProjet {
                nom: nom_court(&f)?,
                fichier: e.path(),
            })
        })
        .collect();
    v.sort_by(|a, b| a.fichier.cmp(&b.fichier));
    v
}

/// Les noms courts de tous les decks, dans l'ordre des fichiers.
pub fn noms() -> Vec<String> {
    tous().into_iter().map(|d| d.nom).collect()
}

/// Le deck de ce nom court (sans tenir compte des majuscules), ou le seul
/// dont le nom commence ainsi (`dori` → `dorinthea`).
pub fn trouver(nom: &str) -> Result<DeckDuProjet, String> {
    let nom = nom.to_lowercase();
    let tous = tous();
    if let Some(d) = tous.iter().find(|d| d.nom == nom) {
        return Ok(d.clone());
    }
    let debut: Vec<&DeckDuProjet> = tous.iter().filter(|d| d.nom.starts_with(&nom)).collect();
    match debut.as_slice() {
        [d] => Ok((*d).clone()),
        _ => Err(format!(
            "deck inconnu : {nom} (decks : {})",
            tous.iter()
                .map(|d| d.nom.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// La liste (fichier lu) du deck de ce nom.
pub fn liste(nom: &str) -> Result<Decklist, String> {
    let d = trouver(nom)?;
    let texte = std::fs::read_to_string(&d.fichier)
        .map_err(|e| format!("lecture de {} : {e}", d.fichier.display()))?;
    Decklist::parse(&texte).map_err(|e| format!("{} : {e}", d.fichier.display()))
}

/// Le card-pool (55 cartes) du deck de ce nom, avec les vraies cartes.
pub fn charger_pool(db: &CardDb, nom: &str) -> Result<CardPool, String> {
    let l = liste(nom)?;
    real::card_pool(db, &l).map_err(|e| format!("deck {nom} : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_nom_court_est_le_nom_du_fichier_sans_numero() {
        assert_eq!(nom_court("01_ira.txt"), Some("ira".into()));
        assert_eq!(nom_court("10_Kayo.txt"), Some("kayo".into()));
        assert_eq!(nom_court("README.md"), None);
        assert_eq!(nom_court("notes.txt"), None);
        assert_eq!(nom_court("_ira.txt"), None);
    }

    #[test]
    fn un_debut_de_nom_suffit_s_il_est_unique() {
        assert_eq!(trouver("dori").unwrap().nom, "dorinthea");
        assert_eq!(trouver("IRA").unwrap().nom, "ira");
        assert!(trouver("zzz").is_err());
    }
}
