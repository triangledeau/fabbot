//! fab-ui — jouer contre le bot dans le terminal.
//!
//! Usage :
//!     cargo run --release -p fab-ui -- [--deck hala|dorinthea] [--bot hala|dorinthea] [--graine N]
//!
//! Par défaut : vous jouez Hala, le bot joue Dorinthea, graine tirée de l'heure.
//! Le bot est B0 (choix au hasard parmi les options légales).

use fab_ai::RandomAgent;
use fab_engine::PlayerId;
use fab_ui::jeu::{Fin, Interface};
use fab_ui::textes::Textes;
use std::io::{self, Write};

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
    let deck_bot = valeur("--bot").unwrap_or_else(|| autre.into());
    let graine = valeur("--graine")
        .and_then(|g| g.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1)
        });

    let mut g = match fab_ui::nouvelle_partie(&deck, &deck_bot, graine) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Erreur : {e}");
            std::process::exit(1);
        }
    };
    let textes = Textes::charger();
    println!("fab-ui — vous ({deck}) contre le bot B0 aléatoire ({deck_bot}), graine {graine}");
    println!("(rejouer la même partie : --graine {graine})");
    if textes.is_none() {
        println!("Textes officiels des cartes absents : lancez « bash tools/fetch_cards.sh » pour la commande « t ».");
    }
    let mut bot = RandomAgent::new(graine ^ 0x5A5A);
    let mut ui = Interface {
        humain: PlayerId::P0,
        bot: &mut bot,
        textes: textes.as_ref(),
    };
    let stdin = io::stdin();
    let mut entree = stdin.lock();
    let mut sortie = io::stdout();
    match ui.jouer(&mut g, &mut entree, &mut sortie) {
        Ok(Fin::Terminee) => {}
        Ok(Fin::Abandon) => println!("Partie abandonnée."),
        Err(e) => {
            let _ = sortie.flush();
            eprintln!("Erreur : {e}");
            std::process::exit(1);
        }
    }
}
