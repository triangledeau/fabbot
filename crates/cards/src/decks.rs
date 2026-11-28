//! Les decks du projet : un fichier `NN_nom.txt` par deck dans `data/decks/`
//! (`01_ira.txt` → nom court `ira`).
//!
//! C'est la **seule** liste des decks : le simulateur, l'interface et les
//! tests la lisent ici. Ajouter un deck = poser son fichier dans
//! `data/decks/` une fois toutes ses cartes codées (procédure complète :
//! `docs/nouveau-deck.md`). Un deck en cours de codage attend dans
//! `data/decks/en_cours/`, qui n'est pas lu ici.
//!
//! Les **decks maison** (`data/decks/maison/<nom>.txt`, lot 4a du
//! rapatriement d'Arsenal) sont ceux que le propriétaire importe ou construit
//! dans la page. Ils sont lus au moment où on les demande (pas besoin de
//! recompiler), jouables dans `fab-web` dès que toutes leurs cartes sont
//! codées, mais pas comptés dans [`tous`] : le tournoi et les tables
//! apprises restent sur les decks du projet.

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
    /// Deck maison (`data/decks/maison/`) plutôt que deck du projet.
    pub maison: bool,
}

/// Le dossier `data/decks/`.
///
/// Si la variable d'environnement `FABBOT_DECKS` existe et n'est pas vide,
/// c'est ce chemin qui est pris (l'appli Android y copie les decks, car le
/// chemin de compilation n'existe pas sur le téléphone ; voir
/// `docs/android.md`, section 5.1). Sinon, rien ne change.
pub fn dossier() -> PathBuf {
    dossier_selon(std::env::var_os("FABBOT_DECKS"))
}

/// Le dossier des decks selon la valeur de `FABBOT_DECKS` (séparé de
/// [`dossier`] pour être testé sans toucher l'environnement du processus).
fn dossier_selon(variable: Option<std::ffi::OsString>) -> PathBuf {
    match variable {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks"),
    }
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
                maison: false,
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

/// Le dossier des decks maison, `data/decks/maison/`.
pub fn dossier_maison() -> PathBuf {
    dossier().join("maison")
}

/// Un nom de deck maison valable : 1 à 40 caractères parmi `a-z`, `0-9`
/// et `-`, et pas déjà le nom d'un deck du projet.
pub fn verifier_nom_maison(nom: &str) -> Result<(), String> {
    if nom.is_empty()
        || nom.len() > 40
        || !nom
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!(
            "nom de deck invalide : « {nom} » (lettres minuscules sans accent, chiffres et tirets)"
        ));
    }
    if noms().iter().any(|n| n == nom) {
        return Err(format!("« {nom} » est déjà le nom d'un deck du projet"));
    }
    Ok(())
}

/// Les decks maison, par ordre alphabétique (dossier absent = aucun).
pub fn maison() -> Vec<DeckDuProjet> {
    let Ok(lecture) = std::fs::read_dir(dossier_maison()) else {
        return Vec::new();
    };
    let mut v: Vec<DeckDuProjet> = lecture
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let f = e.file_name().to_string_lossy().into_owned();
            let nom = f.strip_suffix(".txt")?.to_string();
            verifier_nom_maison(&nom).ok()?;
            Some(DeckDuProjet {
                nom,
                fichier: e.path(),
                maison: true,
            })
        })
        .collect();
    v.sort_by(|a, b| a.nom.cmp(&b.nom));
    v
}

/// Enregistre un deck maison (le remplace s'il existe déjà).
pub fn enregistrer_maison(nom: &str, liste: &Decklist) -> Result<DeckDuProjet, String> {
    verifier_nom_maison(nom)?;
    let d = dossier_maison();
    std::fs::create_dir_all(&d).map_err(|e| format!("création de {} : {e}", d.display()))?;
    let fichier = d.join(format!("{nom}.txt"));
    let tmp = d.join(format!(".{nom}.txt.tmp"));
    std::fs::write(&tmp, liste.to_text())
        .and_then(|_| std::fs::rename(&tmp, &fichier))
        .map_err(|e| format!("écriture de {} : {e}", fichier.display()))?;
    Ok(DeckDuProjet {
        nom: nom.to_string(),
        fichier,
        maison: true,
    })
}

/// Les decks maison archivés (`data/decks/maison/archives/`) : ni listés ni
/// jouables, mais gardés.
pub fn archives_maison() -> Vec<String> {
    let Ok(lecture) = std::fs::read_dir(dossier_maison().join("archives")) else {
        return Vec::new();
    };
    let mut v: Vec<String> = lecture
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let f = e.file_name().to_string_lossy().into_owned();
            f.strip_suffix(".txt").map(String::from)
        })
        .filter(|n| verifier_nom_maison(n).is_ok())
        .collect();
    v.sort();
    v
}

fn deplacer(de: PathBuf, vers: PathBuf) -> Result<(), String> {
    if !de.is_file() {
        return Err(format!("introuvable : {}", de.display()));
    }
    if vers.exists() {
        return Err(format!("existe déjà : {}", vers.display()));
    }
    if let Some(p) = vers.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("création de {} : {e}", p.display()))?;
    }
    std::fs::rename(&de, &vers).map_err(|e| format!("déplacement de {} : {e}", de.display()))
}

/// Range un deck maison dans `archives/` (il n'est plus proposé).
pub fn archiver_maison(nom: &str) -> Result<(), String> {
    verifier_nom_maison(nom)?;
    let d = dossier_maison();
    deplacer(
        d.join(format!("{nom}.txt")),
        d.join("archives").join(format!("{nom}.txt")),
    )
}

/// Ressort un deck maison des archives.
pub fn restaurer_maison(nom: &str) -> Result<(), String> {
    verifier_nom_maison(nom)?;
    let d = dossier_maison();
    deplacer(
        d.join("archives").join(format!("{nom}.txt")),
        d.join(format!("{nom}.txt")),
    )
}

/// Supprime pour de bon un deck maison archivé (on archive d'abord : une
/// fausse manœuvre ne perd rien).
pub fn supprimer_archive_maison(nom: &str) -> Result<(), String> {
    verifier_nom_maison(nom)?;
    let f = dossier_maison().join("archives").join(format!("{nom}.txt"));
    std::fs::remove_file(&f).map_err(|e| format!("suppression de {} : {e}", f.display()))
}

/// Le deck de ce nom court (sans tenir compte des majuscules), ou le seul
/// dont le nom commence ainsi (`dori` → `dorinthea`). Un deck maison est
/// trouvé par son nom exact.
pub fn trouver(nom: &str) -> Result<DeckDuProjet, String> {
    let nom = nom.to_lowercase();
    let tous = tous();
    if let Some(d) = tous.iter().find(|d| d.nom == nom) {
        return Ok(d.clone());
    }
    if let Some(d) = maison().into_iter().find(|d| d.nom == nom) {
        return Ok(d);
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
    fn sans_fabbot_decks_le_dossier_des_decks_ne_change_pas() {
        let attendu = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks");
        assert_eq!(dossier_selon(None), attendu);
        assert_eq!(dossier_selon(Some("".into())), attendu);
    }

    #[test]
    fn fabbot_decks_remplace_le_dossier_des_decks() {
        assert_eq!(
            dossier_selon(Some("/donnees/decks".into())),
            PathBuf::from("/donnees/decks")
        );
    }

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
