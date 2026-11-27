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
    // commun : fonds illustrés, tirés au hasard à chaque page
    fichier!("commun/fonds.js", JS),
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
    // impression des proxies (lot 5)
    fichier!("impression/impression.css", CSS),
    fichier!("impression/impression.js", JS),
    // extras de la partie : conseil, évaluation, Bot contre Bot, bug (lot 9)
    fichier!("partie/extras.css", CSS),
    fichier!("partie/extras.js", JS),
    // apprendre : didacticiel des règles (phase 2)
    fichier!("apprendre/apprendre.css", CSS),
    fichier!("apprendre/apprendre.js", JS),
    fichier!("apprendre/lecons.js", JS),
    // entraîner : tournoi, SPRT, apprentissage (lot 8)
    fichier!("entrainer/entrainer.css", CSS),
    fichier!("entrainer/entrainer.js", JS),
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

/// L'outil « Proxies de Rathe » (impression complète), s'il a été copié dans
/// `data/outils/proxies-de-rathe.html`. Il n'est pas dans le dépôt : il
/// embarque une base de cartes (éditions, liens d'images) qu'on ne publie
/// pas (dépôt public). Voir `data/outils/README.md`.
pub fn chemin_proxies() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/outils/proxies-de-rathe.html")
}

/// Le vrai dos de carte, s'il a été posé dans `data/outils/` sous le nom
/// `dos-de-carte.webp`, `.png` ou `.jpg` (image de LSS : hors du dépôt
/// public). Renvoie le contenu et son type.
pub fn dos_de_carte() -> Option<(Vec<u8>, &'static str)> {
    let dossier = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/outils");
    [
        ("webp", "image/webp"),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
    ]
    .iter()
    .find_map(|(ext, t)| {
        std::fs::read(dossier.join(format!("dos-de-carte.{ext}")))
            .ok()
            .map(|c| (c, *t))
    })
}

/// Le contenu de l'outil prêt à être intégré à Arsenal : le mode intégré
/// (`window.ARSENAL_IMPRESSION`) est posé s'il ne l'est pas déjà.
pub fn outil_proxies() -> Option<Vec<u8>> {
    std::fs::read_to_string(chemin_proxies())
        .ok()
        .map(|t| mode_integre(&t).into_bytes())
}

/// Pose `window.ARSENAL_IMPRESSION=1` au début de la page de l'outil (comme
/// le faisait build.py de l'Arsenal JS), sauf s'il y est déjà.
pub fn mode_integre(t: &str) -> String {
    if t.contains("window.ARSENAL_IMPRESSION=1") {
        return t.to_string();
    }
    let marque = "<script>window.ARSENAL_IMPRESSION=1;</script>";
    match t.find("<head>") {
        Some(i) => format!("{}{marque}{}", &t[..i + 6], &t[i + 6..]),
        None => format!("{marque}{t}"),
    }
}
