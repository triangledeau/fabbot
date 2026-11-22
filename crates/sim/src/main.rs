//! fab-sim — simulateur de parties.
//!
//! Usage :
//!     cargo run --release -p fab-sim -- --games 100000 [--check] [--threads N] [--mecanismes]
//!
//! `--mecanismes` : decks d'essai qui utilisent préventions au choix,
//! remplacements, alliés, spectra… (voir `test_set::mechanics_loadouts`).
//!
//! `--vrais` : les vrais decks Hala contre Dorinthea (`data/decks/`), avec un
//! choix provisoire d'équipement et de 40 cartes (`real::default_loadout`).
//!
//! Étape 1 : parties « bot aléatoire contre bot aléatoire » avec les decks
//! d'essai (cartes fictives). Sert à mesurer la vitesse du moteur et, avec
//! `--check`, à vérifier la cohérence de l'état après chaque décision.

use fab_ai::{play_out, RandomAgent};
use fab_cards::decklist::Decklist;
use fab_cards::{real, test_set};
use fab_engine::{EndReason, GameSetup, GameState, Outcome, PlayerId};
use std::sync::Arc;
use std::time::Instant;

#[derive(Default, Clone, Copy)]
struct Stats {
    games: u64,
    wins: [u64; 2],
    draws_life: u64,
    draws_limit: u64,
    draws_stalemate: u64,
    turns: u64,
    decisions: u64,
    errors: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |name: &str, default: u64| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    };
    let games = get("--games", 10_000);
    let threads = get(
        "--threads",
        std::thread::available_parallelism()
            .map(|n| n.get() as u64)
            .unwrap_or(1),
    )
    .max(1);
    let check = args.iter().any(|a| a == "--check");
    let seed0 = get("--seed", 1);

    let vrais = args.iter().any(|a| a == "--vrais");
    let db = Arc::new(if vrais {
        real::build()
    } else {
        test_set::build()
    });
    let (nom0, nom1) = if vrais {
        ("Hala", "Dorinthea")
    } else {
        ("deck agressif", "deck défensif")
    };
    let lo = if vrais {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks");
        let lire = |f: &str| {
            let text = std::fs::read_to_string(dir.join(f)).expect("liste de deck");
            let list = Decklist::parse(&text).expect("liste valide");
            real::default_loadout(&db, &list).expect("cartes codées")
        };
        [lire("02_hala.txt"), lire("03_dorinthea.txt")]
    } else if args.iter().any(|a| a == "--mecanismes") {
        test_set::mechanics_loadouts(&db)
    } else {
        [
            test_set::aggro_loadout(&db),
            test_set::defensive_loadout(&db),
        ]
    };
    println!("fab-sim — {games} parties, {threads} threads, vérification : {check}");
    let start = Instant::now();
    let per = games.div_ceil(threads);
    let total = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let db = db.clone();
                let lo = lo.clone();
                s.spawn(move || {
                    let mut st = Stats::default();
                    let from = t * per;
                    let to = ((t + 1) * per).min(games);
                    for i in from..to {
                        let seed = seed0.wrapping_mul(1_000_003).wrapping_add(i);
                        let setup = GameSetup {
                            players: lo.clone(),
                            chooser: PlayerId((i % 2) as u8),
                            seed,
                            shuffle: true,
                            max_turns: 200,
                        };
                        let mut g = GameState::new(db.clone(), setup).expect("mise en place");
                        let mut a = RandomAgent::new(seed ^ 0xA5A5);
                        let mut b = RandomAgent::new(seed ^ 0x5A5A);
                        match play_out(&mut g, [&mut a, &mut b], check) {
                            Ok(d) => {
                                st.decisions += d as u64;
                                st.turns += g.turn as u64;
                                match (g.outcome, g.end_reason) {
                                    (Some(Outcome::Winner(p)), _) => st.wins[p.index()] += 1,
                                    (Some(Outcome::Draw), Some(EndReason::TurnLimit)) => {
                                        st.draws_limit += 1
                                    }
                                    _ => st.draws_life += 1,
                                }
                            }
                            Err(e) => {
                                st.errors += 1;
                                if st.errors <= 3 {
                                    eprintln!("ERREUR graine {seed} : {e}");
                                }
                            }
                        }
                        st.games += 1;
                    }
                    st
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .fold(Stats::default(), |mut a, b| {
                a.games += b.games;
                a.wins[0] += b.wins[0];
                a.wins[1] += b.wins[1];
                a.draws_life += b.draws_life;
                a.draws_limit += b.draws_limit;
                a.draws_stalemate += b.draws_stalemate;
                a.turns += b.turns;
                a.decisions += b.decisions;
                a.errors += b.errors;
                a
            })
    });
    let secs = start.elapsed().as_secs_f64();
    let g = total.games.max(1) as f64;
    println!(
        "Durée : {secs:.2} s — {:.0} parties/s",
        total.games as f64 / secs
    );
    println!(
        "Victoires : {nom0} {:.1} %, {nom1} {:.1} % — nulles : 2 héros à 0 {}, blocage {}, limite de tours {}",
        100.0 * total.wins[0] as f64 / g,
        100.0 * total.wins[1] as f64 / g,
        total.draws_life,
        total.draws_stalemate,
        total.draws_limit
    );
    println!(
        "Moyennes : {:.1} tours, {:.0} décisions par partie",
        total.turns as f64 / g,
        total.decisions as f64 / g
    );
    println!("Erreurs : {}", total.errors);
    if total.errors > 0 {
        std::process::exit(1);
    }
}
