//! fab-sim — simulateur de parties.
//!
//! Usage (à partir de l'étape 1) :
//!     cargo run --release -p fab-sim -- --games 10000
//!
//! TODO(étape 3) : parties en parallèle sur tous les cœurs, matrice 9×9,
//! mesure des parties par seconde.

use std::thread::available_parallelism;

fn main() {
    let threads = available_parallelism().map(|n| n.get()).unwrap_or(1);
    println!("fab-sim — squelette (étape 0)");
    println!("Threads disponibles sur cette machine : {threads}");
    println!("Le moteur de règles n'est pas encore implémenté (étape 1).");
}
