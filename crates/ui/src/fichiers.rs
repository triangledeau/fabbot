//! Les fichiers de la page web (`crates/ui/web/`), intégrés au programme
//! à la compilation : `fab-web` n'a besoin d'aucun fichier à côté de lui.
//!
//! Un dossier par volet (`commun/`, `accueil/`, `partie/`…, voir
//! `docs/rapatriement-arsenal.md`). **Ajouter un fichier** dans `web/` =
//! ajouter une ligne ici ; le test `tous_les_fichiers_de_web_sont_servis`
//! le rappelle.

/// Un fichier servi tel quel.
pub struct Fichier {
    /// Chemin dans l'adresse, sans le `/` du début (`commun/arsenal.css`).
    pub chemin: &'static str,
    pub type_contenu: &'static str,
    pub octets: &'static [u8],
}

macro_rules! fichier {
    ($chemin:literal, $type:expr) => {
        Fichier {
            chemin: $chemin,
            type_contenu: $type,
            octets: include_bytes!(concat!("../web/", $chemin)),
        }
    };
}

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

/// Tous les fichiers de la page.
pub static FICHIERS: &[Fichier] = &[
    fichier!("index.html", HTML),
    // commun : charte et navigation (lot 0)
    fichier!("commun/arsenal.css", CSS),
    fichier!("commun/navigation.js", JS),
    // commun : images des cartes (lot 1)
    fichier!("commun/cartes.js", JS),
    fichier!("commun/logo_arsenal.webp", "image/webp"),
    // accueil (lot 0)
    fichier!("accueil/accueil.css", CSS),
    fichier!("accueil/accueil.js", JS),
    // partie : l'écran de jeu (lot 0, d'après l'ancienne page unique)
    fichier!("partie/partie.css", CSS),
    fichier!("partie/partie.js", JS),
    // partie : plateau animé (lot 2, d'après finition.js de l'Arsenal JS)
    fichier!("partie/finition.css", CSS),
    fichier!("partie/finition.js", JS),
    // decks maison : import FaBrary et vérification (lot 4a)
    fichier!("decks/decks.css", CSS),
    fichier!("decks/decks.js", JS),
    // éditeur de deck (lot 4b, d'après editeur.js de l'Arsenal JS)
    fichier!("decks/editeur.js", JS),
];

/// Le fichier demandé par `chemin` (`/` = `index.html`).
pub fn trouver(chemin: &str) -> Option<&'static Fichier> {
    let c = chemin.trim_start_matches('/');
    let c = if c.is_empty() { "index.html" } else { c };
    FICHIERS.iter().find(|f| f.chemin == c)
}

/// Le type de contenu attendu pour une extension (sert aux tests).
pub fn type_pour(chemin: &str) -> Option<&'static str> {
    let ext = chemin.rsplit_once('.')?.1;
    Some(match ext {
        "html" => HTML,
        "css" => CSS,
        "js" => JS,
        "webp" => "image/webp",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "json" => "application/json; charset=utf-8",
        _ => return None,
    })
}
