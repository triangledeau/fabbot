//! epreuve-a — puzzles à solution exacte (épreuve A, `docs/epreuves.md`).
//!
//! Usage :
//!     cargo run --release -p fab-sim --bin epreuve-a -- [--parties 81]
//!         [--bot b3] [--defense passive,b1] [--budget 200000]
//!         [--decks ira,hala,...] [--seed 1] [--threads N] [--details fichier.tsv]
//!
//! 1. Joue `--parties` parties `--bot` contre `--bot` (B3 par défaut), en
//!    parcourant les paires de decks (81 parties = chaque paire une fois),
//!    et garde la position du début de la phase d'action de chaque tour.
//! 2. Pour chaque position et chaque défense : le solveur exact
//!    (`fab_ai::tour_exact`) énumère toutes les façons de jouer le tour et
//!    trouve le maximum (létal, sinon points de vie retirés) ; puis le bot
//!    joue le même tour, dans le même monde, contre la même défense.
//! 3. Affiche le taux de puzzles résolus au mieux, les létaux trouvés, le
//!    manque à gagner moyen en dégâts, par défense et par deck, et les pires
//!    puzzles (ligne du bot et meilleure ligne en clair).

#[path = "commun/mod.rs"]
mod commun;

use commun::{debut_de_tour, lire_decks, partie_avec_positions, table_side, Options};
use fab_ai::tour_exact::{decrire_ligne, jouer_tour, resoudre, Defense};
use fab_ai::{bot_connu, creer_agent};
use fab_cards::real;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Un puzzle résolu.
#[derive(Clone)]
struct Puzzle {
    graine: u64,
    tour: u32,
    deck: usize,
    adverse: usize,
    defense: Defense,
    vie_adverse: i32,
    complet: bool,
    noeuds: u64,
    max_letal: bool,
    max_degats: i32,
    bot_letal: bool,
    bot_degats: i32,
    ligne_max: String,
    ligne_bot: String,
    secondes_solveur: f64,
}

impl Puzzle {
    fn resolu(&self) -> bool {
        (self.bot_letal, self.bot_degats) >= (self.max_letal, self.max_degats)
            || (self.max_letal && self.bot_letal)
    }
    fn manque(&self) -> i32 {
        if self.max_letal && self.bot_letal {
            0
        } else {
            (self.max_degats - self.bot_degats).max(0)
        }
    }
}

#[derive(Default)]
struct Bilan {
    puzzles: u64,
    complets: u64,
    resolus: u64,
    manque: i64,
    nuls: u64,
    letaux: u64,
    letaux_trouves: u64,
    bot_mieux: u64,
}

impl Bilan {
    fn ajouter(&mut self, p: &Puzzle) {
        self.puzzles += 1;
        if !p.complet {
            return;
        }
        self.complets += 1;
        if p.max_degats <= 0 && !p.max_letal {
            self.nuls += 1;
        }
        if p.resolu() {
            self.resolus += 1;
        }
        // Le bot ne peut pas faire mieux qu'une recherche complète : si
        // c'est le cas, le solveur a un défaut (à signaler).
        if (p.bot_letal, p.bot_degats) > (p.max_letal, p.max_degats) {
            self.bot_mieux += 1;
        }
        self.manque += p.manque() as i64;
        if p.max_letal {
            self.letaux += 1;
            if p.bot_letal {
                self.letaux_trouves += 1;
            }
        }
    }

    fn ligne(&self, nom: &str) -> String {
        let pc = |a: u64, b: u64| {
            if b == 0 {
                "—".to_string()
            } else {
                format!("{:.1} %", 100.0 * a as f64 / b as f64)
            }
        };
        format!(
            "| {nom} | {} | {} | {} | {} | {:.2} | {} / {} ({}) |",
            self.puzzles,
            self.complets,
            self.nuls,
            pc(self.resolus, self.complets),
            self.manque as f64 / self.complets.max(1) as f64,
            self.letaux_trouves,
            self.letaux,
            pc(self.letaux_trouves, self.letaux),
        )
    }
}

fn main() {
    let o = Options::lire();
    let parties = o.nombre("--parties", 81);
    let seed0 = o.nombre("--seed", 1);
    let budget = o.nombre("--budget", 200_000);
    let threads = o.threads();
    let bot = o.texte("--bot").unwrap_or("b3").to_lowercase();
    if !bot_connu(&bot) {
        eprintln!("bot inconnu « {bot} »");
        std::process::exit(2);
    }
    let defenses: Vec<Defense> = o
        .texte("--defense")
        .unwrap_or("passive,b1")
        .split(',')
        .map(|s| {
            Defense::depuis_nom(s.trim()).unwrap_or_else(|| {
                eprintln!("défense inconnue « {s} » (connues : passive, b1)");
                std::process::exit(2)
            })
        })
        .collect();
    let db = Arc::new(real::build());
    let table = table_side(&db);
    let decks = lire_decks(&db, o.texte("--decks"));
    let n = decks.len() as u64;
    println!(
        "epreuve-a — puzzles exacts : {parties} parties {bot} contre {bot} ({n} decks), \
         défenses {}, budget {budget} noeuds par puzzle, {threads} threads, graine {seed0}",
        defenses
            .iter()
            .map(|d| d.nom())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let debut = Instant::now();
    let suivant = AtomicU64::new(0);
    let erreurs = AtomicU64::new(0);
    let fini = AtomicU64::new(0);
    let tous: Mutex<Vec<Puzzle>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            let (db, table, decks, bot, defenses) = (&db, &table, &decks, &bot, &defenses);
            let (suivant, erreurs, fini, tous) = (&suivant, &erreurs, &fini, &tous);
            s.spawn(move || loop {
                let k = suivant.fetch_add(1, Ordering::Relaxed);
                if k >= parties {
                    break;
                }
                let paire = k % (n * n);
                let (i, j) = ((paire / n) as usize, (paire % n) as usize);
                let graine = seed0.wrapping_mul(1_000_003).wrapping_add(k);
                let pools = [&decks[i].pool, &decks[j].pool];
                let positions = match partie_avec_positions(
                    db,
                    table,
                    pools,
                    bot,
                    graine,
                    [debut_de_tour(), debut_de_tour()],
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("ERREUR partie graine {graine} : {e}");
                        erreurs.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                };
                let mut v = Vec::new();
                for g in &positions {
                    let actif = g.turn_player;
                    let (deck, adverse) = if actif.0 == 0 { (i, j) } else { (j, i) };
                    for &defense in defenses {
                        let t = Instant::now();
                        let sol = resoudre(g, defense, budget);
                        let secondes_solveur = t.elapsed().as_secs_f64();
                        let graine_bot = graine ^ ((g.turn as u64) << 32) ^ 0xB07;
                        let mut agent = creer_agent(bot, graine_bot).expect("bot");
                        let r = jouer_tour(g, &mut *agent, defense);
                        v.push(Puzzle {
                            graine,
                            tour: g.turn,
                            deck,
                            adverse,
                            defense,
                            vie_adverse: g.player(actif.opponent()).life,
                            complet: sol.complet,
                            noeuds: sol.noeuds,
                            max_letal: sol.meilleur.letal,
                            max_degats: sol.meilleur.degats,
                            bot_letal: r.letal,
                            bot_degats: r.degats,
                            ligne_max: decrire_ligne(g, &sol.meilleur.ligne, defense),
                            ligne_bot: decrire_ligne(g, &r.ligne, defense),
                            secondes_solveur,
                        });
                    }
                }
                tous.lock().unwrap().extend(v);
                let f = fini.fetch_add(1, Ordering::Relaxed) + 1;
                if f % 10 == 0 || f == parties {
                    eprintln!(
                        "… {f}/{parties} parties, {:.0} s",
                        debut.elapsed().as_secs_f64()
                    );
                }
            });
        }
    });
    let mut tous = tous.into_inner().unwrap();
    tous.sort_by_key(|p| (p.graine, p.tour, p.defense.nom()));
    let secs = debut.elapsed().as_secs_f64();

    let mut sortie = String::new();
    let _ = writeln!(
        sortie,
        "\nDurée : {secs:.0} s, {} puzzles, erreurs : {}",
        tous.len(),
        erreurs.load(Ordering::Relaxed)
    );
    let entete = "| | puzzles | complets | max = 0 | résolus au mieux | manque moyen (PV) | létaux trouvés |\n|---|---|---|---|---|---|---|";
    for &defense in &defenses {
        let ps: Vec<&Puzzle> = tous.iter().filter(|p| p.defense == defense).collect();
        let mut total = Bilan::default();
        let mut par_deck: Vec<Bilan> = (0..decks.len()).map(|_| Bilan::default()).collect();
        for p in &ps {
            total.ajouter(p);
            par_deck[p.deck].ajouter(p);
        }
        let temps: f64 = ps.iter().map(|p| p.secondes_solveur).sum();
        let noeuds_max = ps.iter().map(|p| p.noeuds).max().unwrap_or(0);
        let _ = writeln!(
            sortie,
            "\n## Défense {} (solveur : {:.1} s en tout, au plus {noeuds_max} noeuds pour un puzzle)\n",
            defense.nom(),
            temps
        );
        let _ = writeln!(sortie, "{entete}");
        let _ = writeln!(sortie, "{}", total.ligne("**tous**"));
        for (k, b) in par_deck.iter().enumerate() {
            let _ = writeln!(sortie, "{}", b.ligne(&decks[k].nom));
        }
        if total.bot_mieux > 0 {
            let _ = writeln!(
                sortie,
                "\n⚠️ {} puzzles où le bot fait mieux que le solveur complet : défaut du solveur à chercher.",
                total.bot_mieux
            );
        }
        // Létaux manqués, puis les plus gros manques.
        let mut manques: Vec<&&Puzzle> = ps.iter().filter(|p| p.complet && !p.resolu()).collect();
        manques.sort_by_key(|p| {
            (
                std::cmp::Reverse(p.max_letal && !p.bot_letal),
                std::cmp::Reverse(p.manque()),
            )
        });
        let _ = writeln!(
            sortie,
            "\nPires puzzles (létaux manqués d'abord), au plus 15 :"
        );
        for p in manques.iter().take(15) {
            let res = |l: bool, d: i32| {
                if l {
                    format!("létal ({d} PV)")
                } else {
                    format!("{d} PV")
                }
            };
            let _ = writeln!(
                sortie,
                "- graine {} tour {} : {} contre {} (vie adverse {}) : meilleur {}, bot {}\n  - meilleure ligne : {}\n  - ligne du bot : {}",
                p.graine,
                p.tour,
                decks[p.deck].nom,
                decks[p.adverse].nom,
                p.vie_adverse,
                res(p.max_letal, p.max_degats),
                res(p.bot_letal, p.bot_degats),
                p.ligne_max,
                p.ligne_bot
            );
        }
    }
    print!("{sortie}");

    if let Some(chemin) = o.texte("--details") {
        let mut f = std::fs::File::create(chemin).expect("fichier --details");
        let _ = writeln!(
            f,
            "graine\ttour\tdeck\tadverse\tdefense\tvie_adverse\tcomplet\tnoeuds\tmax_letal\tmax_degats\tbot_letal\tbot_degats\tligne_max\tligne_bot"
        );
        for p in &tous {
            let _ = writeln!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                p.graine,
                p.tour,
                decks[p.deck].nom,
                decks[p.adverse].nom,
                p.defense.nom(),
                p.vie_adverse,
                p.complet,
                p.noeuds,
                p.max_letal,
                p.max_degats,
                p.bot_letal,
                p.bot_degats,
                p.ligne_max,
                p.ligne_bot
            );
        }
        println!("\nDétails écrits dans {chemin}");
    }
    if erreurs.load(Ordering::Relaxed) > 0 {
        std::process::exit(1);
    }
}
