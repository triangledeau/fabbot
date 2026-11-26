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
//! `--decks ira hala` : deux vrais decks au choix (nom du fichier dans
//! `data/decks/` sans le numéro, ou son début : `ira`, `hala`, `dori`… ;
//! voir `fab_cards::decks`).
//!
//! `--bots b1 b0` : les bots des joueurs 1 et 2 (par défaut `b0 b0`,
//! aléatoires ; `b1` = heuristique, `b2` = recherche, `b3` = recherche avec
//! l'évaluation apprise ; `b3:fichier` : B3 avec un autre fichier de poids).
//!
//! `--side liste appris` : comment chaque joueur choisit son équipement et
//! ses 40 cartes dans son card-pool de 55 avant chaque partie (vrais decks
//! seulement ; par défaut `appris appris`). `liste` : dans l'ordre de la liste ;
//! `appris` : la table apprise `data/side/b1.txt` ; `hasard` : au hasard.
//!
//! `--apprendre-side` : apprend la table `data/side/b1.txt` (B1 contre B1,
//! `--games` parties par paire de decks avec un choix au hasard contre le
//! choix « liste » ; voir `fab_ai::side`).
//!
//! `--apprendre-b3` : apprend les poids de l'évaluation de B3
//! (`data/b3/evaluation.txt`, ou `--sortie fichier`) sur `--games` parties
//! par paire de decks entre les bots de `--bots` (par défaut `b1 b1` ;
//! `--lambda` : pénalité ridge ; voir `fab_ai::b3`).
//!
//! `--tournoi` : le premier bot de `--bots` contre le second sur **toutes**
//! les paires des vrais decks (81 cases, miroirs compris), `--games`
//! parties par case, en alternant les places et le joueur qui choisit qui
//! commence. Affiche l'écart d'Elo (intervalle à 95 %) et la matrice des
//! scores du premier bot (ligne : son deck ; colonne : le deck adverse).

use fab_ai::elo::{ecart, ecart_avec_intervalle, Tableau};
use fab_ai::side::{creer_side, ecrire_entree, Options, Regression, TableSide, SIDES};
use fab_ai::{bot_connu, creer_agent, play_out, BOTS};
use fab_cards::{decks, real, test_set};
use fab_engine::{
    CardDb, CardPool, EndReason, FormatConfig, GameSetup, GameState, Loadout, Outcome, PlayerId,
};
use fab_match::{lancer_manche, preparer_manche, Manche};
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

/// Comment les joueurs commencent la partie : avec des `Loadout` fixés
/// (decks d'essai), ou par la phase de side dans leur card-pool.
#[derive(Clone, Copy)]
enum Depart<'a> {
    Fixes(&'a [Loadout; 2]),
    Pools([&'a CardPool; 2], [&'a str; 2], &'a Option<Arc<TableSide>>),
}

/// Joue une partie ; renvoie la partie finie, la manche préparée (phase de
/// side) et le nombre de décisions.
fn jouer(
    db: &Arc<CardDb>,
    depart: Depart,
    bots: [&str; 2],
    chooser: PlayerId,
    seed: u64,
    check: bool,
) -> Result<(GameState, Option<Manche>, u32), String> {
    let (mut g, manche) = preparer(db, depart, chooser, seed)?;
    let mut a = creer_agent(bots[0], seed ^ 0xA5A5).expect("bot");
    let mut b = creer_agent(bots[1], seed ^ 0x5A5A).expect("bot");
    let d = play_out(&mut g, [&mut *a, &mut *b], check)?;
    Ok((g, manche, d))
}

/// Met en place une partie (avec la phase de side pour de vrais decks).
fn preparer(
    db: &Arc<CardDb>,
    depart: Depart,
    chooser: PlayerId,
    seed: u64,
) -> Result<(GameState, Option<Manche>), String> {
    Ok(match depart {
        Depart::Fixes(lo) => {
            let setup = GameSetup {
                players: lo.clone(),
                chooser,
                seed,
                shuffle: true,
                max_turns: 200,
            };
            (
                GameState::new(db.clone(), setup).expect("mise en place"),
                None,
            )
        }
        Depart::Pools(pools, sides, table) => {
            let mut s0 = creer_side(sides[0], db, table.clone(), seed ^ 0x51DE).expect("side");
            let mut s1 = creer_side(sides[1], db, table.clone(), seed ^ 0xE51D).expect("side");
            let taille = FormatConfig::silver_age().deck_size as usize;
            let m = preparer_manche(db, pools, [&mut *s0, &mut *s1], chooser, taille)
                .map_err(|(p, e)| format!("choix du joueur {} : {e:?}", p.0 + 1))?;
            let g = lancer_manche(db.clone(), &m, seed, 200).expect("mise en place");
            (g, Some(m))
        }
    })
}

/// Joue une partie et range son résultat dans `st`, du point de vue des
/// places (joueur 0 / joueur 1).
fn une_partie(
    db: &Arc<CardDb>,
    depart: Depart,
    bots: [&str; 2],
    chooser: PlayerId,
    seed: u64,
    check: bool,
    st: &mut Stats,
) {
    match jouer(db, depart, bots, chooser, seed, check) {
        Ok((g, _, d)) => {
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

/// Lit un vrai deck de `data/decks/` par son nom court ou le début de son
/// nom (`ira`, `hala`… ; voir `fab_cards::decks`).
fn lire_deck(db: &CardDb, nom: &str) -> (String, CardPool) {
    let list = decks::liste(nom).unwrap_or_else(|e| panic!("{e}"));
    let pool = real::card_pool(db, &list).unwrap_or_else(|e| panic!("deck {nom} : {e}"));
    (list.hero.clone(), pool)
}

/// Les noms courts de tous les decks de `data/decks/`, dans l'ordre des fichiers.
fn tous_les_decks() -> Vec<String> {
    decks::noms()
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
    let bots: [String; 2] = match args.iter().position(|a| a == "--bots") {
        Some(i) => [
            args.get(i + 1).expect("--bots a b").to_lowercase(),
            args.get(i + 2).expect("--bots a b").to_lowercase(),
        ],
        None => ["b0".to_string(), "b0".to_string()],
    };
    for b in &bots {
        if !bot_connu(b) {
            eprintln!("bot inconnu « {b} » (connus : {}, b2-N)", BOTS.join(", "));
            std::process::exit(2);
        }
    }
    let bots = [bots[0].as_str(), bots[1].as_str()];
    let sides: [String; 2] = match args.iter().position(|a| a == "--side") {
        Some(i) => [
            args.get(i + 1).expect("--side a b").to_lowercase(),
            args.get(i + 2).expect("--side a b").to_lowercase(),
        ],
        None => ["appris".to_string(), "appris".to_string()],
    };
    for s in &sides {
        if !SIDES.contains(&s.as_str()) {
            eprintln!("side inconnu « {s} » (connus : {})", SIDES.join(", "));
            std::process::exit(2);
        }
    }
    let sides = [sides[0].as_str(), sides[1].as_str()];

    if args.iter().any(|a| a == "--apprendre-b3") {
        let sortie = args
            .iter()
            .position(|a| a == "--sortie")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../data/b3/evaluation.txt")
                    .to_string_lossy()
                    .into_owned()
            });
        let lambda = args
            .iter()
            .position(|a| a == "--lambda")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(LAMBDA_B3);
        // Par défaut, les parties d'apprentissage sont jouées par B1.
        let bots = if args.iter().any(|a| a == "--bots") {
            bots
        } else {
            ["b1", "b1"]
        };
        apprendre_b3(games, threads, seed0, bots, &sortie, lambda);
        return;
    }
    if args.iter().any(|a| a == "--apprendre-side") {
        apprendre_side(games, threads, seed0);
        return;
    }
    if args.iter().any(|a| a == "--tournoi") {
        // `--avec levia` : seulement les paires où l'un des decks est Levia.
        let avec = args
            .iter()
            .position(|a| a == "--avec")
            .map(|i| args.get(i + 1).expect("--avec deck").to_lowercase());
        tournoi(games, threads, check, seed0, bots, sides, avec.as_deref());
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
    let table = table_side(&db, sides);
    let mut pools = None;
    let (noms, lo) = if let Some(choisis) = &choisis {
        let (n0, p0) = lire_deck(&db, &choisis[0]);
        let (n1, p1) = lire_deck(&db, &choisis[1]);
        // Les loadouts fixés ne servent pas : la phase de side choisit.
        let lo = [
            real::default_loadout_pool(&db, &p0),
            real::default_loadout_pool(&db, &p1),
        ];
        pools = Some([p0, p1]);
        ([n0, n1], lo)
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
    let (nom0, nom1) = if vrais {
        (
            format!("{} ({}, side {})", noms[0], bots[0], sides[0]),
            format!("{} ({}, side {})", noms[1], bots[1], sides[1]),
        )
    } else {
        (
            format!("{} ({})", noms[0], bots[0]),
            format!("{} ({})", noms[1], bots[1]),
        )
    };
    println!("fab-sim — {games} parties, {threads} threads, vérification : {check}");
    let start = Instant::now();
    let per = games.div_ceil(threads);
    let total = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (db, lo, pools, table) = (db.clone(), &lo, &pools, &table);
                s.spawn(move || {
                    let mut st = Stats::default();
                    let from = t * per;
                    let to = ((t + 1) * per).min(games);
                    let depart = match pools {
                        Some([p0, p1]) => Depart::Pools([p0, p1], sides, table),
                        None => Depart::Fixes(lo),
                    };
                    for i in from..to {
                        let seed = seed0.wrapping_mul(1_000_003).wrapping_add(i);
                        let chooser = PlayerId((i % 2) as u8);
                        une_partie(&db, depart, bots, chooser, seed, check, &mut st);
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

/// `--tournoi` : `bots[0]` (A) contre `bots[1]` (B) sur les 81 paires de decks
/// (avec `avec` : seulement les 17 paires où l'un des deux decks est celui-là).
fn tournoi(
    games: u64,
    threads: u64,
    check: bool,
    seed0: u64,
    bots: [&str; 2],
    sides: [&str; 2],
    avec: Option<&str>,
) {
    let db = Arc::new(real::build());
    let table = table_side(&db, sides);
    let decks: Vec<(String, CardPool)> =
        tous_les_decks().iter().map(|n| lire_deck(&db, n)).collect();
    let n = decks.len();
    let noms_decks = tous_les_decks();
    let garder = |i: usize| avec.is_none_or(|a| noms_decks[i].to_lowercase().contains(a));
    if avec.is_some() && !(0..n).any(garder) {
        eprintln!("--avec : aucun deck de ce nom");
        std::process::exit(2);
    }
    let cases: Vec<usize> = (0..n * n)
        .filter(|&c| garder(c / n) || garder(c % n))
        .collect();
    let total_jobs = cases.len() as u64 * games;
    println!(
        "fab-sim — tournoi {} (A, side {}) contre {} (B, side {}) : {} paires de decks, \
         {games} parties par paire ({total_jobs} parties), {threads} threads, vérification : {check}",
        bots[0],
        sides[0],
        bots[1],
        sides[1],
        cases.len()
    );
    let start = Instant::now();
    let next = AtomicU64::new(0);
    // Pour chaque case (deck de A, deck de B) : stats du point de vue de A
    // (A est toujours « joueur 0 » dans ces stats, quelle que soit sa place).
    let par_case: Vec<Stats> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (db, decks, next, table, cases) = (db.clone(), &decks, &next, &table, &cases);
                s.spawn(move || {
                    let mut v = vec![Stats::default(); n * n];
                    loop {
                        let job = next.fetch_add(1, Ordering::Relaxed);
                        if job >= total_jobs {
                            break;
                        }
                        let case = cases[(job / games) as usize];
                        let k = job % games;
                        let (da, db_) = (case / n, case % n);
                        // Alterner la place de A et le joueur qui choisit.
                        let a_place = (k % 2) as usize;
                        let chooser = PlayerId(((k / 2) % 2) as u8);
                        let seed = seed0
                            .wrapping_mul(1_000_003)
                            .wrapping_add(case as u64 * games + k);
                        let mut pools = [&decks[da].1, &decks[db_].1];
                        let mut b = bots;
                        let mut sd = sides;
                        if a_place == 1 {
                            pools.swap(0, 1);
                            b.swap(0, 1);
                            sd.swap(0, 1);
                        }
                        let mut st = Stats::default();
                        let depart = Depart::Pools(pools, sd, table);
                        une_partie(&db, depart, b, chooser, seed, check, &mut st);
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
    let noms = tous_les_decks();
    print!("{:>10}", "");
    for d in &noms {
        print!("{:>7}", court(d));
    }
    println!("{:>8}", "moy.");
    for (i, d) in noms.iter().enumerate() {
        print!("{:>10}", court(d));
        let mut ligne = Stats::default();
        for j in 0..n {
            let c = &par_case[i * n + j];
            ligne.add(c);
            if c.played() == 0 {
                print!("{:>7}", "-");
            } else {
                print!("{:>7.0}", 100.0 * c.points0() / c.played() as f64);
            }
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

/// La table apprise, lue une seule fois si un joueur s'en sert.
fn table_side(db: &CardDb, sides: [&str; 2]) -> Option<Arc<TableSide>> {
    sides
        .iter()
        .any(|s| *s == "appris" || *s == "general")
        .then(|| {
            Arc::new(TableSide::lire(db, fab_ai::side::TABLE_B1).expect("data/side/b1.txt valide"))
        })
}

/// `--apprendre-side` : pour chaque paire (deck A, deck B), `games` parties
/// B1 contre B1 où A tire son équipement et ses 40 cartes au hasard et B
/// garde le choix « liste ». Régression du score de A sur ses choix, puis
/// écriture de `data/side/b1.txt` (voir `fab_ai::side`).
fn apprendre_side(games: u64, threads: u64, seed0: u64) {
    let db = Arc::new(real::build());
    let decks: Vec<(String, CardPool)> =
        tous_les_decks().iter().map(|n| lire_deck(&db, n)).collect();
    let opts: Vec<Options> = decks.iter().map(|(_, p)| Options::new(&db, p)).collect();
    let n = decks.len();
    let total_jobs = (n * n) as u64 * games;
    let taille = FormatConfig::silver_age().deck_size as usize;
    println!(
        "fab-sim — apprentissage du side : {n}×{n} paires, {games} parties par paire \
         ({total_jobs} parties), {threads} threads"
    );
    let start = Instant::now();
    let next = AtomicU64::new(0);
    let vide = || -> Vec<Regression> {
        (0..n * n)
            .map(|c| Regression::new(opts[c / n].nb_variables()))
            .collect()
    };
    let regs: Vec<Regression> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (db, decks, next, opts) = (db.clone(), &decks, &next, &opts);
                s.spawn(move || {
                    let mut v = vide();
                    let mut erreurs = 0;
                    loop {
                        let job = next.fetch_add(1, Ordering::Relaxed);
                        if job >= total_jobs {
                            break;
                        }
                        let case = (job / games) as usize;
                        let k = job % games;
                        let (da, db_) = (case / n, case % n);
                        let a_place = (k % 2) as usize;
                        let chooser = PlayerId(((k / 2) % 2) as u8);
                        // Graines différentes de celles des tournois.
                        let seed = (seed0 ^ 0x5EED_51DE)
                            .wrapping_mul(1_000_003)
                            .wrapping_add(job);
                        let mut pools = [&decks[da].1, &decks[db_].1];
                        let mut sd = ["hasard", "liste"];
                        if a_place == 1 {
                            pools.swap(0, 1);
                            sd.swap(0, 1);
                        }
                        let depart = Depart::Pools(pools, sd, &None);
                        match jouer(&db, depart, ["b1", "b1"], chooser, seed, false) {
                            Ok((g, Some(m), _)) => {
                                let a = PlayerId(a_place as u8);
                                let y = match g.outcome {
                                    Some(Outcome::Winner(w)) if w == a => 1.0,
                                    Some(Outcome::Winner(_)) => 0.0,
                                    _ => 0.5,
                                };
                                let lo = &m.loadouts[a_place];
                                let x = opts[da].variables(&lo.arena, &lo.deck, m.premier == a);
                                v[case].ajouter(&x, y);
                            }
                            _ => erreurs += 1,
                        }
                    }
                    if erreurs > 0 {
                        eprintln!("{erreurs} parties en erreur (ignorées)");
                    }
                    v
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .fold(vide(), |mut a, b| {
                for (x, y) in a.iter_mut().zip(&b) {
                    x.fusionner(y);
                }
                a
            })
    });
    println!("Durée : {:.1} s", start.elapsed().as_secs_f64());

    // Régression « générale » de chaque deck (toutes paires), puis chaque
    // paire tirée vers elle.
    let mut texte = String::from(
        "# Choix appris (phase de side des bots B1/B2) : voir crates/ai/src/side.rs.\n\
         # Produit par : cargo run --release -p fab-sim -- --apprendre-side\n",
    );
    texte.push_str(&format!(
        "# {games} parties par paire de decks (B1 contre B1, choix au hasard contre le choix « liste »).\n\
         # Entre crochets : écart estimé de chances de gagner par exemplaire (points de %),\n\
         # par rapport à la meilleure option de la zone ou à la moins bonne carte gardée.\n",
    ));
    for i in 0..n {
        let mut gen = Regression::new(opts[i].nb_variables());
        for j in 0..n {
            gen.fusionner(&regs[i * n + j]);
        }
        let zero = vec![0.0; opts[i].nb_variables()];
        let beta_gen = gen.resoudre(LAMBDA_GENERAL, &zero);
        texte.push_str(&format!("\n# ===== {} =====\n", decks[i].0));
        texte.push_str(&ecrire_entree(
            &db,
            &opts[i],
            &decks[i].1,
            None,
            &beta_gen,
            taille,
        ));
        for j in 0..n {
            let beta = regs[i * n + j].resoudre(LAMBDA_PAIRE, &beta_gen);
            texte.push_str(&ecrire_entree(
                &db,
                &opts[i],
                &decks[i].1,
                Some(decks[j].1.hero),
                &beta,
                taille,
            ));
        }
    }
    let chemin =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/side/b1.txt");
    std::fs::write(&chemin, texte).expect("écriture de data/side/b1.txt");
    println!("Table écrite : data/side/b1.txt (recompiler pour s'en servir)");
}

/// Pénalité « ridge » de l'apprentissage de B3 (vers 0).
const LAMBDA_B3: f64 = 10.0;

/// `--apprendre-b3` : `games` parties par paire de decks (81 paires) entre
/// `bots[0]` et `bots[1]` (choix de side appris), une position au début de
/// chaque tour, avec le résultat final. Une partie sur 10 est gardée de côté
/// pour mesurer la qualité des prédictions (perte logarithmique : plus elle
/// est petite, mieux c'est ; 0,693 = prédire toujours 50 %), comparée à la
/// formule fixe de B2. Écrit les poids dans `sortie`.
fn apprendre_b3(games: u64, threads: u64, seed0: u64, bots: [&str; 2], sortie: &str, lambda: f64) {
    use fab_ai::b3::{ecrire_modele, Apprentissage};
    let db = Arc::new(real::build());
    let table = table_side(&db, ["appris", "appris"]);
    let decks: Vec<(String, CardPool)> =
        tous_les_decks().iter().map(|n| lire_deck(&db, n)).collect();
    let n = decks.len();
    let total_jobs = (n * n) as u64 * games;
    println!(
        "fab-sim — apprentissage de B3 : {n}×{n} paires, {games} parties par paire \
         ({total_jobs} parties, {} contre {}), {threads} threads",
        bots[0], bots[1]
    );
    let start = Instant::now();
    let next = AtomicU64::new(0);
    // (apprentissage, test, perte de la formule de B2 sur le test, erreurs)
    let morceaux: Vec<(Apprentissage, Apprentissage, f64, u64)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let (db, decks, next, table) = (db.clone(), &decks, &next, &table);
                s.spawn(move || {
                    let mut app = Apprentissage::new();
                    let mut test = Apprentissage::new();
                    let mut perte_b2 = 0.0f64;
                    let mut erreurs = 0;
                    let mut positions: Vec<GameState> = Vec::new();
                    loop {
                        let job = next.fetch_add(1, Ordering::Relaxed);
                        if job >= total_jobs {
                            break;
                        }
                        let case = (job / games) as usize;
                        let k = job % games;
                        let (da, db_) = (case / n, case % n);
                        let chooser = PlayerId((k % 2) as u8);
                        // Graines différentes de celles des tournois.
                        let seed = (seed0 ^ 0xB3B3_0E7A)
                            .wrapping_mul(1_000_003)
                            .wrapping_add(job);
                        let pools = [&decks[da].1, &decks[db_].1];
                        let depart = Depart::Pools(pools, ["appris", "appris"], table);
                        let Ok((mut g, _)) = preparer(&db, depart, chooser, seed) else {
                            erreurs += 1;
                            continue;
                        };
                        let mut a = creer_agent(bots[0], seed ^ 0xA5A5).expect("bot");
                        let mut b = creer_agent(bots[1], seed ^ 0x5A5A).expect("bot");
                        positions.clear();
                        let mut tour = 0;
                        let mut ok = true;
                        let mut decisions = 0u32;
                        while let Some(d) = g.pending.as_ref() {
                            if g.turn != tour {
                                tour = g.turn;
                                let mut copie = g.clone();
                                copie.record = None;
                                positions.push(copie);
                            }
                            let d = d.clone();
                            let i = if d.player.0 == 0 {
                                a.choose(&g, &d)
                            } else {
                                b.choose(&g, &d)
                            };
                            decisions += 1;
                            if g.apply(i).is_err() || decisions > 200_000 {
                                ok = false;
                                break;
                            }
                        }
                        if !ok {
                            erreurs += 1;
                            continue;
                        }
                        let y = match g.outcome {
                            Some(Outcome::Winner(PlayerId::P0)) => 1.0,
                            Some(Outcome::Winner(_)) => 0.0,
                            _ => 0.5,
                        };
                        let garde = job % 10 == 7;
                        for p in &positions {
                            if garde {
                                test.ajouter(p, y);
                                let q =
                                    (fab_ai::b2::estimer(p, 1.5) as f64).clamp(1e-9, 1.0 - 1e-9);
                                perte_b2 -= y as f64 * q.ln() + (1.0 - y as f64) * (1.0 - q).ln();
                            } else {
                                app.ajouter(p, y);
                            }
                        }
                    }
                    (app, test, perte_b2, erreurs)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut app = Apprentissage::new();
    let mut test = Apprentissage::new();
    let mut perte_b2 = 0.0;
    let mut erreurs = 0;
    for (a, t, p, e) in &morceaux {
        app.fusionner(a);
        test.fusionner(t);
        perte_b2 += p;
        erreurs += e;
    }
    drop(morceaux);
    println!(
        "Parties : {:.1} s, {} positions d'apprentissage, {} de test, {erreurs} parties en erreur",
        start.elapsed().as_secs_f64(),
        app.nb_positions(),
        test.nb_positions()
    );
    let perte_b2 = perte_b2 / test.nb_positions().max(1) as f64;
    println!(
        "Perte logarithmique (plus petit = mieux, 0,693 = toujours 50 %) ; \
         formule fixe de B2 sur le test : {perte_b2:.4}"
    );
    // Comparaison sur les mêmes parties : sans les caractéristiques du
    // contenu de la main (comme la 2e génération).
    {
        let garder = |c| !Apprentissage::est_contenu_de_la_main(c);
        let (poids, perte_app) = app.filtrer(garder).resoudre(lambda);
        let perte_test = test.filtrer(garder).perte(&poids);
        let chemin = sortie.replace(".txt", "_sans_main.txt");
        println!(
            "  sans le contenu de la main, λ = {lambda} : apprentissage {perte_app:.4}, \
             test {perte_test:.4} → {chemin}"
        );
        let entete = format!(
            "# Sans les caractéristiques du contenu de la main, mêmes parties que {sortie}.\n\
             # Perte logarithmique sur des parties de test : {perte_test:.4}.\n"
        );
        std::fs::write(&chemin, ecrire_modele(&db, &poids, &entete)).expect("écriture des poids");
    }
    // Plusieurs pénalités : on garde celle qui prédit le mieux les parties
    // de test (le contenu de la main ajoute beaucoup de poids).
    let mut meilleur: Option<(f64, Vec<_>, f64)> = None;
    for l in [lambda / 4.0, lambda, lambda * 4.0] {
        let (poids, perte_app) = app.resoudre(l);
        let perte_test = test.perte(&poids);
        println!("  avec le contenu de la main, λ = {l} : apprentissage {perte_app:.4}, test {perte_test:.4}");
        if meilleur.as_ref().is_none_or(|m| perte_test < m.2) {
            meilleur = Some((l, poids, perte_test));
        }
    }
    let (lambda, poids, perte_test) = meilleur.expect("au moins une pénalité");
    let entete = format!(
        "# Évaluation apprise de B3 (voir crates/ai/src/b3.rs) : une ligne « caractéristique = poids ».\n\
         # Chances de gagner du joueur 0 = σ(note₀ − note₁), note = Σ poids × valeur.\n\
         # Produit par : cargo run --release -p fab-sim -- --apprendre-b3 --games {games} --bots {} {}\n\
         # {} positions (début de chaque tour) de {total_jobs} parties ({} contre {}, side appris), λ = {lambda}.\n\
         # Perte logarithmique sur des parties de test : {perte_test:.4} (formule fixe de B2 : {perte_b2:.4}).\n",
        bots[0],
        bots[1],
        app.nb_positions(),
        bots[0],
        bots[1],
    );
    std::fs::write(sortie, ecrire_modele(&db, &poids, &entete)).expect("écriture des poids");
    println!("Poids écrits : {sortie} (recompiler pour que « b3 » s'en serve)");
}

/// Pénalité « ridge » de l'estimation générale (vers 0).
const LAMBDA_GENERAL: f64 = 100.0;
/// Pénalité de l'estimation d'une paire (vers l'estimation générale),
/// en « parties » : plus elle est grande, plus la paire suit le général.
const LAMBDA_PAIRE: f64 = 1_000.0;
