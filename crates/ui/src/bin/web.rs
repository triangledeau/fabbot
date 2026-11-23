//! fab-web — jouer contre le bot dans le navigateur.
//!
//! Usage :
//!     cargo run --release -p fab-ui --bin fab-web -- [--deck hala] [--bot dorinthea] [--niveau b1] [--graine N] [--port 8080]
//!
//! Puis ouvrir http://127.0.0.1:8080 dans le navigateur. Le serveur
//! n'écoute que sur cet ordinateur. Ctrl+C dans le terminal pour l'arrêter.

use fab_ui::serveur::{self, graine_horloge, Etat};
use fab_ui::textes::Textes;
use fab_ui::web::{Reglages, Session};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let valeur = |nom: &str| {
        args.iter()
            .position(|a| a == nom)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let deck = valeur("--deck").unwrap_or_else(|| "hala".into());
    let autre = if deck.eq_ignore_ascii_case("hala") {
        "dorinthea"
    } else {
        "hala"
    };
    let reglages = Reglages {
        deck_bot: valeur("--bot").unwrap_or_else(|| autre.into()),
        deck,
        niveau: valeur("--niveau").unwrap_or_else(|| "b1".into()),
        graine: valeur("--graine")
            .and_then(|g| g.parse().ok())
            .unwrap_or_else(graine_horloge),
    };
    let port: u16 = valeur("--port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let session = match Session::nouvelle(reglages.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur : {e}");
            std::process::exit(1);
        }
    };
    let textes = Textes::charger();
    if textes.is_none() {
        println!("Textes officiels des cartes absents : lancez « bash tools/fetch_cards.sh ».");
    }
    println!(
        "fab-web — vous ({}) contre le bot {} ({}), graine {}",
        reglages.deck, reglages.niveau, reglages.deck_bot, reglages.graine
    );
    println!("Ouvrez http://127.0.0.1:{port} dans votre navigateur (Ctrl+C ici pour arrêter).");
    let etat = Etat {
        session: Some(session),
        textes,
        depart: reglages,
    };
    if let Err(e) = serveur::lancer(port, etat) {
        eprintln!("Erreur : impossible d'écouter sur le port {port} : {e}");
        eprintln!("(un autre programme l'utilise peut-être : essayez --port 8081)");
        std::process::exit(1);
    }
}
