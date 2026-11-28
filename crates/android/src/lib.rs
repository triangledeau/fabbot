//! fab-android — le cœur Rust de l'appli Android d'Arsenal.
//!
//! L'appli Kotlin (dossier `android/`) appelle une seule fonction,
//! [`demarrer`] (par JNI : `fr.arsenal.fabbot.Moteur.demarrer(dossier)`),
//! qui lance **le même serveur que `fab-web` sur PC**, mais dans le
//! téléphone, puis affiche `http://127.0.0.1:<port>/` dans une WebView.
//! La page PC est servie telle quelle, avec en plus `mobile.css` et
//! `mobile.js` (dossier `web/`) qui l'adaptent au téléphone.
//!
//! Brief complet et règles : `docs/android.md`. Sur PC, `cargo run -p
//! fab-android --bin apercu` lance la même chose dans le navigateur.

pub mod donnees;
pub mod journal;
pub mod serveur;

#[cfg(target_os = "android")]
mod jni_lien;

use fab_ui::match_web::MatchWeb;
use fab_ui::sauvegarde;
use fab_ui::serveur::{ecouter, graine_horloge, Etat};
use fab_ui::textes::Textes;
use fab_ui::web::Reglages;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Premier port essayé (puis les 19 suivants s'il est pris).
///
/// Un port **fixe** compte : la WebView range les réglages de la page
/// (`localStorage`) par adresse, port compris. Un port tiré au hasard à
/// chaque lancement les ferait oublier.
pub const PORT: u16 = 38_080;

/// Le fichier des textes officiels des cartes dans le dossier de l'appli
/// (copié depuis les fichiers de l'APK par la partie Kotlin).
pub fn chemin_textes(dossier: &Path) -> PathBuf {
    dossier.join("donnees").join("public_card_data.csv")
}

/// Le fichier de la partie en cours, en chemin complet.
pub fn chemin_sauvegarde(dossier: &Path) -> PathBuf {
    dossier.join(sauvegarde::fichier_par_defaut())
}

/// Les réglages du match créé au démarrage, comme `fab-web` sans option :
/// Hala contre Dorinthea, bot B3, 2 manches gagnantes.
pub fn reglages_de_depart() -> Reglages {
    Reglages {
        deck: "hala".into(),
        deck_bot: "dorinthea".into(),
        niveau: "b3".into(),
        graine: graine_horloge(),
        manches: 2,
    }
}

/// Construit l'état du serveur, comme `fab-web` au démarrage : reprise de
/// la partie sauvegardée si elle existe et n'est pas finie, sinon nouveau
/// match ; textes officiels lus dans `textes` s'il existe.
pub fn preparer_etat(dossier: &Path, textes: Option<&Path>) -> Etat {
    let depart = reglages_de_depart();
    let fichier = chemin_sauvegarde(dossier);
    let reprise = if fichier.exists() {
        match sauvegarde::lire(&fichier) {
            Ok(m) if !m.termine() => Some(m),
            Ok(_) => None,
            Err(e) => {
                journal::ecrire(&format!(
                    "Partie sauvegardée illisible ({e}) : nouveau match."
                ));
                None
            }
        }
    } else {
        None
    };
    if reprise.is_some() {
        journal::ecrire("Partie sauvegardée retrouvée : elle reprend.");
    }
    let partie = match reprise {
        Some(m) => Some(m),
        None => match MatchWeb::nouveau(depart.clone()) {
            Ok(m) => Some(m),
            Err(e) => {
                journal::ecrire(&format!("Match impossible à créer : {e}"));
                None
            }
        },
    };
    let textes = textes
        .and_then(|t| std::fs::read_to_string(t).ok())
        .map(|t| Textes::depuis_csv(&t));
    if textes.is_none() {
        journal::ecrire("Textes officiels des cartes absents : la page marche, sans leur texte.");
    }
    Etat {
        partie,
        textes,
        depart,
        sauvegarde: Some(fichier),
    }
}

/// Démarre le serveur et rend aussitôt le numéro de port.
///
/// `dossier` est le dossier privé de l'appli (`context.filesDir` côté
/// Kotlin). Dans l'ordre, **avant** de lancer le moindre fil :
/// 1. les plantages sont notés dans `sauvegardes/plantages.txt` ;
/// 2. `dossier` devient le dossier courant (les sauvegardes, les bugs
///    signalés et les tournois y sont rangés, en chemins relatifs) ;
/// 3. les decks du projet sont installés dans `dossier/decks/` (sans
///    toucher `decks/maison/`) et `FABBOT_DECKS` y renvoie ;
/// 4. le port est réservé.
///
/// Ensuite, un fil à part lit la sauvegarde et les textes des cartes, puis
/// répond aux requêtes. Un second appel (l'activité Android recréée, le
/// processus toujours là) rend le même port sans rien relancer.
pub fn demarrer(dossier: &Path, textes: Option<PathBuf>, port: u16) -> io::Result<u16> {
    static PORT_OUVERT: OnceLock<u16> = OnceLock::new();
    static VERROU: Mutex<()> = Mutex::new(());
    let _garde = VERROU.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(p) = PORT_OUVERT.get() {
        return Ok(*p);
    }

    std::fs::create_dir_all(dossier)?;
    let dossier = dossier.canonicalize()?;
    journal::noter_les_plantages(dossier.join("sauvegardes").join("plantages.txt"));
    std::env::set_current_dir(&dossier)?;
    let decks = dossier.join("decks");
    donnees::installer_decks(&decks)?;
    std::env::set_var("FABBOT_DECKS", &decks);

    let ecoute = ecouter(port)?;
    let port = ecoute.local_addr()?.port();
    journal::ecrire(&format!("Serveur prêt : http://127.0.0.1:{port}/"));
    std::thread::Builder::new()
        .name("serveur".into())
        .spawn(move || {
            let etat = preparer_etat(&dossier, textes.as_deref());
            serveur::servir_toujours(ecoute, Arc::new(Mutex::new(etat)));
        })?;
    let _ = PORT_OUVERT.set(port);
    Ok(port)
}
