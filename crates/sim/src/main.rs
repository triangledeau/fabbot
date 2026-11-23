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
//! `--decks ira hala` : deux vrais decks au choix (début du nom de fichier
//! dans `data/decks/`, sans le numéro : `ira`, `hala`, `dorinthea`…).
//!
//! `--bots b1 b0` : les bots des joueurs 1 et 2 (par défaut `b0 b0`,
//! aléatoires ; `b1` = heuristique).
//!
//! `--tournoi` : le premier bot de `--bots` contre le second sur **toutes**
//! les paires des 9 vrais decks (81 cases, miroirs compris), `--games`
//! parties par case, en alternant les places et le joueur qui choisit qui
//! commence. Affiche l'écart d'Elo (intervalle à 95 %) et la matrice des
//! scores du premier bot (ligne : son deck ; colonne : le deck adverse).

use fab_ai::elo::{ecart, ecart_avec_intervalle, Tableau};
use fab_ai::{creer_agent, play_out, BOTS};
use fab_cards::decklist::Decklist;
use fab_cards::{real, test_set};
use fab_engine::{CardDb, EndReason, GameSetup, GameState, Loadout, Outcome, PlayerId};
use std::sync::atomic::{AtomicU64, Ordering};
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

impl Stats {
    fn add(&mut self, b: &Stats) {
        self.games += b.games;
        self.wins[0] += b.wins[0];
        self.wins[1] += b.wins[1];
        self.draws_life += b.draws_life;
        self.draws_limit += b.draws_limit;
        self.draws_stalemate += b.draws_stalemate;
        self.turns += b.turns;
        self.decisions += b.decisions;
        self.errors += b.errors;
    }

    fn draws(&self) -> u64 {
        self.draws_life + self.draws_limit + self.draws_stalemate
    }

    /// Points du joueur 0 (nulle = ½), parties en erreur exclues.
    fn points0(&self) -> f64 {
        self.wins[0] as f64 + self.draws() as f64 / 2.0
    }

    fn played(&self) -> u64 {
        self.games - self.errors
    }
}

/// Joue une partie et range son résultat dans `st`, du point de vue des
/// places (joueur 0 / joueur 1).
#[allow(clippy::too_many_arguments)]
fn une_partie(
    db: &Arc<CardDb>,
    lo: [Loadout; 2],
    bots: [&str; 2],
    chooser: PlayerId,
    seed: u64,
    check: bool,
    st: &mut Stats,
) {
    let setup = GameSetup {
        players: lo,
        chooser,
        seed,
        shuffle: true,
        max_turns: 200,
    };
    let mut g = GameState::new(db.clone(), setup).expect("mise en place");
    let mut a = creer_agent(bots[0], seed ^ 0xA5A5).expect("bot");
    let mut b = creer_agent(bots[1], seed ^ 0x5A5A).expect("bot");
    match play_out(&mut g, [&mut *a, &mut *b], check) {
        Ok(d) => {
            st.decisions += d as u64;
            st.turns += g.turn as u64;
            match (g.outcome, g.end_reason) {
                (Some(Outcome::Winner(p)), _) => st.wins[p.index()] += 1,
                (Some(Outcome::Draw), Some(EndReason::TurnLimit)) => st.draws_limit += 1,
                (Some(Outcome::Draw), Some(EndReason::Stalemate)) => st.draws_stalemate += 1,
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

/// Lit un vrai deck de `data/decks/` par le début de son nom (`ira`, `hala`…).
fn lire_deck(db: &CardDb, nom: &str) -> (String, Loadout) {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks");
    let fichier = std::fs::read_dir(&dir)
        .expect("dossier data/decks")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|f| {
            f.ends_with(".txt")
                && f.split_once('_')
                    .is_some_and(|(_, reste)| reste.starts_with(&nom.to_lowercase()))
        })
        .unwrap_or_else(|| panic!("aucun deck « {nom} » dans data/decks"));
    let text = std::fs::read_to_string(dir.join(&fichier)).expect("liste de deck");
    let list = Decklist::parse(&text).expect("liste valide");
    let lo = real::default_loadout(db, &list).unwrap_or_else(|e| panic!("{fichier} : {e}"));
    (list.hero.clone(), lo)
}

/// Les 9 decks, dans l'ordre des fichiers.
const DECKS: [&str; 9] = [
    "ira",
    "hala",
    "dorinthea",
    "bravo",
    "iyslander",
    "florian",
    "levia",
    "dromai",
    "arakni",
];

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
    let bots: [String; 2] = match args.iter().position(|a| a == "--bots") {
        Some(i) => [
            args.get(i + 1).expect("--bots a b").to_lowercase(),
            args.get(i + 2).expect("--bots a b").to_lowercase(),
        ],
        None => ["b0".to_string(), "b0".to_string()],
    };
    for b in &bots {
        if !BOTS.contains(&b.as_str()) {
            eprintln!("bot inconnu « {b} » (connus : {})", BOTS.join(", "));
            std::process::exit(2);
        }
    }
    let bots = [bots[0].as_str(), bots[1].as_str()];

    if args.iter().any(|a| a == "--tournoi") {
        tournoi(games, threads, check, seed0, bots);
        return;
    }

    // Vrais decks : `--vrais` (Hala contre Dorinthea) ou `--decks a b`.
    let choisis: Option<[String; 2]> = match args.iter().position(|a| a == "--decks") {
        Some(i) => Some([
            args.get(i + 1).expect("--decks a b").clone(),
            args.get(i + 2).expect("--decks a b").clone(),
        ]),
        None if args.iter().any(|a| a == "--vrais") => {
            Some(["hala".to_string(), "dorinthea".to_string()])
        }
        None => None,
    };
    let vrais = choisis.is_some();
    let db = Arc::new(if vrais {
        real::build()
    } else {
        test_set::build()
    });
    let (noms, lo) = if let Some(choisis) = &choisis {
        let (n0, l0) = lire_deck(&db, &choisis[0]);
        let (n1, l1) = lire_deck(&db, &choisis[1]);
        ([n0, n1], [l0, l1])
    } else if args.iter().any(|a| a == "--mecanismes") {
        (
            [
                "deck mécanismes 1".to_string(),
                "deck mécanismes 2".to_string(),
            ],
            test_set::mechanics_loadouts(&db),
        )
    } else {
        (
            ["deck agressif".to_string(), "deck défensif".to_string()],
            [
                test_set::aggro_loadout(&db),
                test_set::defensive_loadout(&db),
            ],
        )
    };
    let (nom0, nom1) = (
        format!("{} ({})", noms[0], bots[0]),
        format!("{} ({})", noms[1], bots[1]),
    );
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
                        let chooser = PlayerId((i % 2) as u8);
                        une_partie(&db, lo.clone(), bots, chooser, seed, check, &mut st);
                    }
                    st
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .fold(Stats::default(), |mut a, b| {
                a.add(&b);
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

/// `--tournoi` : `bots[0]` (A) contre `bots[1]` (B) sur les 81 paires de decks.
fn tournoi(games: u64, threads: u64, check: bool, seed0: u64, bots: [&str; 2]) {
    let db = Arc::new(real::build());
    let decks: Vec<(String, Loadout)> = DECKS.iter().map(|n| lire_deck(&db, n)).collect();
    let n = decks.len();
    let cases = (n * n) as u64;
    let total_jobs = cases * games;
    println!(
        "fab-sim — tournoi {} (A) contre {} (B) : {n}×{n} paires de decks, {games} parties par paire \
         ({total_jobs} parties), {threads} threads, vérification : {check}",
        bots[0], bots[1]
    );
    let start = Instant::now();
    let next = AtomicU64::new(0);
    // Pour chaque case (deck de A, deck de B) : stats du point de vue de A
    // (A est toujours « joueur 0 » dans ces stats, quelle que soit sa place).
    let par_case: Vec<Stats> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (db, decks, next) = (db.clone(), &decks, &next);
                s.spawn(move || {
                    let mut v = vec![Stats::default(); n * n];
                    loop {
                        let job = next.fetch_add(1, Ordering::Relaxed);
                        if job >= total_jobs {
                            break;
                        }
                        let case = (job / games) as usize;
                        let k = job % games;
                        let (da, db_) = (case / n, case % n);
                        // Alterner la place de A et le joueur qui choisit.
                        let a_place = (k % 2) as usize;
                        let chooser = PlayerId(((k / 2) % 2) as u8);
                        let seed = seed0.wrapping_mul(1_000_003).wrapping_add(job);
                        let mut lo = [decks[da].1.clone(), decks[db_].1.clone()];
                        let mut b = bots;
                        if a_place == 1 {
                            lo.swap(0, 1);
                            b.swap(0, 1);
                        }
                        let mut st = Stats::default();
                        une_partie(&db, lo, b, chooser, seed, check, &mut st);
                        if a_place == 1 {
                            st.wins.swap(0, 1);
                        }
                        v[case].add(&st);
                    }
                    v
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).fold(
            vec![Stats::default(); n * n],
            |mut a, b| {
                for (x, y) in a.iter_mut().zip(&b) {
                    x.add(y);
                }
                a
            },
        )
    });
    let secs = start.elapsed().as_secs_f64();
    let mut total = Stats::default();
    for c in &par_case {
        total.add(c);
    }
    println!(
        "Durée : {secs:.1} s — {:.0} parties/s",
        total.games as f64 / secs
    );
    let joues = total.played() as f64;
    println!(
        "Résultat de {} : {} victoires, {} défaites, {} nulles (blocage {}, limite de tours {}, 2 héros à 0 {}) sur {} parties",
        bots[0],
        total.wins[0],
        total.wins[1],
        total.draws(),
        total.draws_stalemate,
        total.draws_limit,
        total.draws_life,
        total.played()
    );
    let (e, lo, hi) = ecart_avec_intervalle(total.points0(), joues, total.draws() as f64);
    println!(
        "Score de {} : {:.1} % → écart d'Elo {:+.0} (intervalle à 95 % : {:+.0} à {:+.0})",
        bots[0],
        100.0 * total.points0() / joues.max(1.0),
        e,
        lo,
        hi
    );
    let mut t = Tableau::new(&[bots[1], bots[0]]);
    t.points[1][0] = total.points0();
    t.points[0][1] = joues - total.points0();
    t.parties[0][1] = joues;
    t.parties[1][0] = joues;
    let elo = t.elo();
    println!(
        "Classement Elo (référence {} = 0) : {} {:+.0}",
        bots[1], bots[0], elo[1]
    );

    // Matrice : score de A (en %) avec le deck de la ligne contre B avec le
    // deck de la colonne.
    let court = |s: &str| s.chars().take(6).collect::<String>();
    println!();
    println!(
        "Score de {} (%), ligne = son deck, colonne = deck de {} :",
        bots[0], bots[1]
    );
    print!("{:>10}", "");
    for d in DECKS {
        print!("{:>7}", court(d));
    }
    println!("{:>8}", "moy.");
    for (i, d) in DECKS.iter().enumerate() {
        print!("{:>10}", court(d));
        let mut ligne = Stats::default();
        for j in 0..n {
            let c = &par_case[i * n + j];
            ligne.add(c);
            print!("{:>7.0}", 100.0 * c.points0() / c.played().max(1) as f64);
        }
        let s = ligne.points0() / ligne.played().max(1) as f64;
        println!("{:>7.0} ({:+.0} Elo)", 100.0 * s, ecart(s));
    }
    print!("{:>10}", "moy.");
    for j in 0..n {
        let mut col = Stats::default();
        for i in 0..n {
            col.add(&par_case[i * n + j]);
        }
        print!(
            "{:>7.0}",
            100.0 * col.points0() / col.played().max(1) as f64
        );
    }
    println!();
    println!("Erreurs : {}", total.errors);
    if total.errors > 0 {
        std::process::exit(1);
    }
}
