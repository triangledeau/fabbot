//! apercu — l'appli Android vue sur PC, dans le navigateur.
//!
//! Usage :
//!     cargo run -p fab-android --bin apercu -- [--dossier target/apercu-android] [--port 38080] [--sans-navigateur]
//!
//! Lance exactement le serveur de l'appli (copie des decks, sauvegarde,
//! `mobile.css` et `mobile.js`), avec comme « dossier privé » un dossier de
//! `target/`. Pour voir la page à la taille du téléphone : outils de
//! développement du navigateur (F12), mode appareil, 412 × 923, rapport
//! de pixels 2,625 (Pixel 9). Voir `docs/android.md`, section 8.5.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let valeur = |nom: &str| {
        args.iter()
            .position(|a| a == nom)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dossier = valeur("--dossier")
        .map(PathBuf::from)
        .unwrap_or_else(|| racine.join("target/apercu-android"));
    // Chemin complet avant que le dossier courant ne change.
    let textes = racine
        .join("data/cards/sources/public_card_data.csv")
        .canonicalize()
        .ok();
    let port = valeur("--port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(fab_android::PORT);

    match fab_android::demarrer(&dossier, textes, port) {
        Ok(port) => {
            let url = format!("http://127.0.0.1:{port}/");
            println!("Aperçu de l'appli Android : {url}");
            println!("Dossier de l'appli : {}", dossier.display());
            println!("Fermez cette fenêtre pour arrêter.");
            if !args.iter().any(|a| a == "--sans-navigateur") {
                fab_ui::serveur::ouvrir_navigateur(&url);
            }
            loop {
                std::thread::park();
            }
        }
        Err(e) => {
            eprintln!("Erreur : démarrage impossible : {e}");
            std::process::exit(1);
        }
    }
}
