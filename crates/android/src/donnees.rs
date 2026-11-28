//! Les fichiers embarqués dans l'appli (decks du projet, dos de carte) et
//! leur installation dans le dossier privé de l'appli.

use std::io;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embarques.rs"));

/// Installe les decks du projet dans `dossier` (`<dossier privé>/decks/`).
///
/// Appelée à chaque démarrage : les fichiers du projet sont réécrits (une
/// mise à jour de l'appli apporte ainsi ses decks corrigés), ceux qui ne
/// sont plus dans le projet sont retirés. Le sous-dossier `maison/` (decks
/// d'Antoine) n'est **jamais** touché, seulement créé s'il manque.
pub fn installer_decks(dossier: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dossier.join("maison"))?;
    for entree in std::fs::read_dir(dossier)? {
        let chemin = entree?.path();
        let nom = chemin.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let du_projet = DECKS.iter().any(|(n, _)| *n == nom);
        if chemin.is_file() && nom.ends_with(".txt") && !du_projet {
            std::fs::remove_file(&chemin)?;
        }
    }
    for (nom, contenu) in DECKS {
        let chemin = dossier.join(nom);
        // Pas d'écriture inutile si le fichier est déjà le bon.
        if std::fs::read(&chemin).ok().as_deref() != Some(*contenu) {
            std::fs::write(&chemin, contenu)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_decks_du_projet_sont_embarques() {
        assert!(DECKS.len() >= 9, "{} decks seulement", DECKS.len());
        assert!(DECKS.iter().any(|(n, _)| *n == "02_hala.txt"));
        assert!(DECKS.iter().all(|(n, _)| n.ends_with(".txt")));
    }
}
