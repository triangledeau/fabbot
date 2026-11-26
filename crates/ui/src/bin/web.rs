//! fab-web — jouer contre le bot dans le navigateur.
//!
//! Usage :
//!     cargo run --release -p fab-ui --bin fab-web -- [--deck hala] [--bot dorinthea] [--niveau b1] [--graine N] [--manches 2] [--port 8080]
//!
//! Le navigateur s'ouvre tout seul sur la page du jeu (`--sans-navigateur`
//! pour l'éviter). Si le port 8080 est pris, le suivant libre est utilisé.
//! La partie en cours est sauvegardée à chaque coup dans
//! `sauvegardes/en-cours.json` et reprise au lancement suivant
//! (`--nouvelle-partie` pour l'ignorer). Le serveur n'écoute que sur cet ordinateur. Le plus simple sous Windows :
//! double-cliquer sur `Jouer.bat` à la racine du projet.

use fab_ui::match_web::MatchWeb;
use fab_ui::sauvegarde;
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

    // Lot 3 : la partie en cours est écrite à chaque coup dans
    // sauvegardes/en-cours.json ; au lancement, on la reprend (sauf
    // `--nouvelle-partie`, ou si le match était fini).
    let fichier = sauvegarde::fichier_par_defaut();
    let reprise = if args.iter().any(|a| a == "--nouvelle-partie") || !fichier.exists() {
        None
    } else {
        match sauvegarde::lire(&fichier) {
            Ok(m) if !m.termine() => Some(m),
            Ok(_) => None,
            Err(e) => {
                println!("Partie sauvegardée illisible ({e}) : nouveau match.");
                None
            }
        }
    };
    if reprise.is_some() {
        println!(
            "Partie sauvegardée retrouvée ({}) : elle reprend là où vous l'aviez laissée.",
            fichier.display()
        );
    }
    let partie = match reprise
        .map(Ok)
        .unwrap_or_else(|| MatchWeb::nouveau(reglages.clone()))
    {
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
        partie.reglages.deck,
        partie.reglages.niveau,
        partie.reglages.deck_bot,
        partie.reglages.graine
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
        sauvegarde: Some(fichier),
    };
    if !args.iter().any(|a| a == "--sans-navigateur") {
        serveur::ouvrir_navigateur(&url);
    }
    serveur::servir_toujours(ecoute, etat);
}
