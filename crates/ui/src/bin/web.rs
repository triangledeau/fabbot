//! fab-web — jouer contre le bot dans le navigateur.
//!
//! Usage :
//!     cargo run --release -p fab-ui --bin fab-web -- [--deck hala] [--bot dorinthea] [--niveau b1] [--graine N] [--manches 2] [--port 8080]
//!
//! Le navigateur s'ouvre tout seul sur la page du jeu (`--sans-navigateur`
//! pour l'éviter). Si le port 8080 est pris, le suivant libre est utilisé.
//! Le serveur n'écoute que sur cet ordinateur. Le plus simple sous Windows :
//! double-cliquer sur `Jouer.bat` à la racine du projet.

use fab_ui::match_web::MatchWeb;
use fab_ui::serveur::{self, graine_horloge, Etat};
use fab_ui::textes::Textes;
use fab_ui::web::Reglages;

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
        manches: valeur("--manches")
            .and_then(|g| g.parse().ok())
            .unwrap_or(2),
    };
    let port: u16 = valeur("--port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let partie = match MatchWeb::nouveau(reglages.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur : {e}");
            std::process::exit(1);
        }
    };
    let textes = Textes::charger();
    if textes.is_none() {
        println!(
            "Textes officiels des cartes absents : la page marche, mais sans le texte des cartes."
        );
    }
    println!(
        "fab-web — vous ({}) contre le bot {} ({}), graine {}",
        reglages.deck, reglages.niveau, reglages.deck_bot, reglages.graine
    );
    let ecoute = match serveur::ecouter(port) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Erreur : aucun port libre à partir de {port} : {e}");
            std::process::exit(1);
        }
    };
    let port = ecoute.local_addr().map(|a| a.port()).unwrap_or(port);
    let url = format!("http://127.0.0.1:{port}");
    println!();
    println!("Le jeu est prêt : {url}");
    println!("(le navigateur s'ouvre tout seul ; sinon, copiez cette adresse dans le navigateur)");
    println!();
    println!(
        "Laissez cette fenêtre ouverte pendant que vous jouez. Fermez-la pour arrêter le jeu."
    );
    let etat = Etat {
        partie: Some(partie),
        textes,
        depart: reglages,
    };
    if !args.iter().any(|a| a == "--sans-navigateur") {
        serveur::ouvrir_navigateur(&url);
    }
    serveur::servir_toujours(ecoute, etat);
}
