//! analyse — statistiques détaillées sur les parties d'un deck.
//!
//! Usage :
//!     cargo run --release -p fab-sim --bin analyse -- <deck> [--bots b3 b3]
//!         [--contre ira,hala,...] [--games 20] [--seed 1] [--threads N]
//!
//! Le deck étudié joue `--games` parties contre chacun des decks de
//! `--contre` (par défaut les 9), en alternant qui choisit le premier
//! joueur. Phase de side apprise des deux côtés (comme le tournoi).
//! Affiche, pour le deck étudié : victoires, durée, dégâts infligés et subis
//! par tour et par source, cartes jouées / pitchées / posées en défense, X
//! choisis, cartes encore en main en fin de tour, et les mêmes chiffres
//! séparés entre parties gagnées et perdues.

use fab_ai::side::{creer_side, TableSide};
use fab_ai::{creer_agent, Agent};
use fab_cards::{decks, real};
use fab_engine::{
    CardDb, CardPool, Choice, Decision, DecisionKind, Event, FormatConfig, GameState, ObjectId,
    Outcome, PlayerId, Zone,
};
use fab_match::{lancer_manche, preparer_manche};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Un agent qui note les décisions de son joueur.
struct Espion {
    inner: Box<dyn Agent + Send>,
    notes: Vec<(u32, String)>,
}

impl Agent for Espion {
    fn name(&self) -> &str {
        "espion"
    }
    fn choose(&mut self, g: &GameState, d: &Decision) -> usize {
        let i = self.inner.choose(g, d);
        let nom = |id: ObjectId| nom_carte(g, id);
        match d.kind {
            DecisionKind::ChooseX => {
                let max = d.options.len() - 1;
                let carte = g.stack.last().map(|l| nom(l.object.id)).unwrap_or_default();
                if let Choice::Number(x) = d.options[i] {
                    self.notes
                        .push((g.turn, format!("X {carte} : {x} (max {max})")));
                }
            }
            DecisionKind::Priority => {
                let mon_tour = g.turn_player == d.player;
                let jouables = d
                    .options
                    .iter()
                    .filter(|c| matches!(c, Choice::Play(_) | Choice::Activate(..)))
                    .count();
                if d.options[i] == Choice::Pass && jouables > 0 && mon_tour && g.stack.is_empty() {
                    self.notes
                        .push((g.turn, "passe alors qu'il peut jouer".into()));
                }
                // Cartes de l'arsenal jouables pendant le tour adverse.
                if !mon_tour {
                    let depuis_arsenal = d.options.iter().any(|c| match c {
                        Choice::Play(id) => g.obj(*id).zone == Zone::Arsenal,
                        _ => false,
                    });
                    if depuis_arsenal {
                        let joue = matches!(d.options[i], Choice::Play(id) if g.obj(id).zone == Zone::Arsenal);
                        self.notes.push((
                            g.turn,
                            format!(
                                "tour adverse, carte d'arsenal jouable : {}",
                                if joue { "jouée" } else { "gardée" }
                            ),
                        ));
                    }
                }
                if let Choice::Play(id) = d.options[i] {
                    let n = nom(id);
                    let de = if g.obj(id).zone == Zone::Arsenal {
                        "arsenal"
                    } else {
                        "main"
                    };
                    let quand = if mon_tour { "mon tour" } else { "tour adverse" };
                    self.notes
                        .push((g.turn, format!("joue {n} depuis {de}, {quand}")));
                    if g.def(id).name == "Ice Eternal" {
                        let ice = g.players[d.player.0 as usize]
                            .hand
                            .iter()
                            .filter(|&&x| {
                                x != id && g.def(x).supertypes.has(fab_engine::Supertypes::ICE)
                            })
                            .count();
                        self.notes.push((
                            g.turn,
                            format!("Ice Eternal jouée avec {ice} carte(s) Ice en main"),
                        ));
                    }
                }
            }
            DecisionKind::Arsenal => {
                if let Choice::Arsenal(id) = d.options[i] {
                    self.notes
                        .push((g.turn, format!("met en arsenal {}", nom(id))));
                } else {
                    self.notes.push((g.turn, "ne met rien en arsenal".into()));
                }
            }
            DecisionKind::OptionalCost | DecisionKind::Effect => {
                let carte = g.stack.last().map(|l| nom(l.object.id)).unwrap_or_default();
                let rep = match d.options[i] {
                    Choice::Pass | Choice::No => "non",
                    _ => "oui",
                };
                self.notes
                    .push((g.turn, format!("question {:?} {carte} : {rep}", d.kind)));
            }
            _ => {}
        }
        i
    }
}

fn nom_carte(g: &GameState, id: ObjectId) -> String {
    let o = g.obj(id);
    let id = o.proxy_of.unwrap_or(id);
    let d = g.def(id);
    match d.color {
        Some(c) => format!("{} ({:?})", d.name, c).to_lowercase(),
        None => d.name.clone(),
    }
}

#[derive(Default, Clone)]
struct Agreg {
    parties: u64,
    victoires: u64,
    nulles: u64,
    tours_a_moi: u64,
    tours: u64,
    degats_faits: u64,
    degats_faits_arcane: u64,
    degats_subis: u64,
    vie_adverse_fin: i64,
    vie_fin: i64,
    /// Par nom : (fois jouée, dégâts infligés au héros adverse).
    sources: BTreeMap<String, (u64, u64)>,
    subis_par_source: BTreeMap<String, u64>,
    jouees: BTreeMap<String, u64>,
    activees: BTreeMap<String, u64>,
    attaques_par_tour: BTreeMap<String, u64>,
    tours_rien: u64,
    pitchees: BTreeMap<String, u64>,
    defenses_main: BTreeMap<String, u64>,
    defenses_equipement: u64,
    defenses_main_total: u64,
    notes: BTreeMap<String, u64>,
    /// Somme des cartes en main à la fin de mes tours (après l'arsenal).
    main_fin_tour: u64,
    /// Tours où je n'ai joué aucune carte.
    tours_sans_carte: u64,
    /// Nombre de cartes jouées pendant mes tours.
    cartes_mes_tours: u64,
    /// Tours avec au moins deux Ice Eternal.
    double_ice: u64,
    /// Histogramme des dégâts infligés par tour (0, 1-3, 4-6, 7-9, 10+).
    histo: [u64; 5],
}

impl Agreg {
    fn ajouter(&mut self, b: &Agreg) {
        self.parties += b.parties;
        self.victoires += b.victoires;
        self.nulles += b.nulles;
        self.tours_a_moi += b.tours_a_moi;
        self.tours += b.tours;
        self.degats_faits += b.degats_faits;
        self.degats_faits_arcane += b.degats_faits_arcane;
        self.degats_subis += b.degats_subis;
        self.vie_adverse_fin += b.vie_adverse_fin;
        self.vie_fin += b.vie_fin;
        for (k, v) in &b.sources {
            let e = self.sources.entry(k.clone()).or_default();
            e.0 += v.0;
            e.1 += v.1;
        }
        for (m, a) in [
            (&mut self.subis_par_source, &b.subis_par_source),
            (&mut self.jouees, &b.jouees),
            (&mut self.activees, &b.activees),
            (&mut self.attaques_par_tour, &b.attaques_par_tour),
            (&mut self.pitchees, &b.pitchees),
            (&mut self.defenses_main, &b.defenses_main),
            (&mut self.notes, &b.notes),
        ] {
            for (k, v) in a {
                *m.entry(k.clone()).or_default() += v;
            }
        }
        self.defenses_equipement += b.defenses_equipement;
        self.defenses_main_total += b.defenses_main_total;
        self.main_fin_tour += b.main_fin_tour;
        self.tours_sans_carte += b.tours_sans_carte;
        self.tours_rien += b.tours_rien;
        self.cartes_mes_tours += b.cartes_mes_tours;
        self.double_ice += b.double_ice;
        for i in 0..5 {
            self.histo[i] += b.histo[i];
        }
    }
}

fn lire_deck(db: &CardDb, nom: &str) -> (String, CardPool) {
    let list = decks::liste(nom).unwrap_or_else(|e| panic!("{e}"));
    let pool = real::card_pool(db, &list).unwrap_or_else(|e| panic!("deck {nom} : {e}"));
    (list.hero.clone(), pool)
}

/// Joue une partie ; le deck étudié est le joueur `moi`.
#[allow(clippy::too_many_arguments)]
fn partie(
    db: &Arc<CardDb>,
    pools: [&CardPool; 2],
    bots: [&str; 2],
    table: &Arc<TableSide>,
    chooser: PlayerId,
    seed: u64,
    moi: usize,
) -> Option<Agreg> {
    let side_moi = std::env::var("SIDE_ETUDIE").unwrap_or_else(|_| "appris".into());
    let side_adv = std::env::var("SIDE_ADVERSE").unwrap_or_else(|_| "appris".into());
    let sides = if moi == 0 {
        [side_moi.as_str(), side_adv.as_str()]
    } else {
        [side_adv.as_str(), side_moi.as_str()]
    };
    let mut s0 = creer_side(sides[0], db, Some(table.clone()), seed ^ 0x51DE)?;
    let mut s1 = creer_side(sides[1], db, Some(table.clone()), seed ^ 0xE51D)?;
    let taille = FormatConfig::silver_age().deck_size as usize;
    let m = preparer_manche(db, pools, [&mut *s0, &mut *s1], chooser, taille).ok()?;
    let mut g = lancer_manche(db.clone(), &m, seed, 200).ok()?;
    g.record = Some(Vec::new());
    // Le premier « début de tour » a eu lieu avant l'enregistrement.
    let premier = g.turn_player.0 as usize == moi;
    let mut a = Espion {
        inner: creer_agent(bots[0], seed ^ 0xA5A5)?,
        notes: Vec::new(),
    };
    let mut b = Espion {
        inner: creer_agent(bots[1], seed ^ 0x5A5A)?,
        notes: Vec::new(),
    };
    let mut fin_de_tour_main: Vec<u64> = Vec::new();
    let mut tour_vu = 0;
    let mut n = 0;
    while let Some(d) = g.pending_decision() {
        let d = d.clone();
        if g.turn != tour_vu {
            tour_vu = g.turn;
        }
        let i = if d.player.0 == 0 {
            a.choose(&g, &d)
        } else {
            b.choose(&g, &d)
        };
        let tour_avant = g.turn;
        let joueur_avant = g.turn_player;
        let main_avant = g.players[moi].hand.len();
        g.apply(i).ok()?;
        if g.turn != tour_avant && joueur_avant.0 as usize == moi {
            // Approximation : la main juste avant la fin du tour.
            fin_de_tour_main.push(main_avant as u64);
        }
        n += 1;
        if n > 200_000 {
            return None;
        }
    }
    let mut r = Agreg {
        parties: 1,
        ..Default::default()
    };
    match g.outcome {
        Some(Outcome::Winner(p)) if p.0 as usize == moi => r.victoires = 1,
        Some(Outcome::Winner(_)) => {}
        _ => r.nulles = 1,
    }
    r.tours = g.turn as u64;
    r.vie_fin = g.players[moi].life as i64;
    r.vie_adverse_fin = g.players[1 - moi].life as i64;
    r.main_fin_tour = fin_de_tour_main.iter().sum();
    let espion = if moi == 0 { &a } else { &b };
    for (_, n) in &espion.notes {
        let cle = if let Some(p) = n.find(" (max") {
            // X : regrouper par (carte, X, max)
            n[..p].to_string() + &n[p..]
        } else {
            n.clone()
        };
        *r.notes.entry(cle).or_default() += 1;
    }
    // Parcours des événements.
    let ev = g.record.take().unwrap_or_default();
    let mut mon_tour = premier;
    let mut degats_tour = 0u64;
    let mut cartes_tour = 0u64;
    let mut ice_tour = 0u64;
    let mut act_tour = 0u64;
    let mut att_tour = 0u64;
    let fin_tour = |r: &mut Agreg, degats: u64, cartes: u64, ice: u64, act: u64, att: u64| {
        r.tours_a_moi += 1;
        if cartes + act == 0 {
            r.tours_rien += 1;
        }
        *r.attaques_par_tour
            .entry(format!("{att} attaque(s)"))
            .or_default() += 1;
        let k = match degats {
            0 => 0,
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            _ => 4,
        };
        r.histo[k] += 1;
        if cartes == 0 {
            r.tours_sans_carte += 1;
        }
        r.cartes_mes_tours += cartes;
        if ice >= 2 {
            r.double_ice += 1;
        }
    };
    for e in ev {
        match e {
            Event::StartOfTurn(p) => {
                if mon_tour {
                    fin_tour(
                        &mut r,
                        degats_tour,
                        cartes_tour,
                        ice_tour,
                        act_tour,
                        att_tour,
                    );
                }
                mon_tour = p.0 as usize == moi;
                degats_tour = 0;
                cartes_tour = 0;
                ice_tour = 0;
                act_tour = 0;
                att_tour = 0;
            }
            Event::Played { player, def, .. } if player.0 as usize == moi => {
                let d = db.get(def);
                let nom = match d.color {
                    Some(c) => format!("{} ({:?})", d.name, c).to_lowercase(),
                    None => d.name.clone(),
                };
                if d.name == "Ice Eternal" && mon_tour {
                    ice_tour += 1;
                }
                *r.jouees.entry(nom).or_default() += 1;
                if mon_tour {
                    cartes_tour += 1;
                }
            }
            Event::Activated {
                player,
                source,
                ability,
            } if player.0 as usize == moi => {
                let lab = g
                    .def(source)
                    .activated
                    .get(ability as usize)
                    .map(|a| a.label)
                    .unwrap_or("?");
                let quand = if mon_tour { "mon tour" } else { "tour adverse" };
                *r.activees
                    .entry(format!("{} : {lab} ({quand})", nom_carte(&g, source)))
                    .or_default() += 1;
                if mon_tour {
                    act_tour += 1;
                }
            }
            Event::Attack { attacker, .. } if attacker.0 as usize == moi && mon_tour => {
                att_tour += 1;
            }
            Event::Pitched { player, card } if player.0 as usize == moi => {
                *r.pitchees.entry(nom_carte(&g, card)).or_default() += 1;
            }
            Event::Defend {
                card,
                defender,
                from_hand,
                ..
            } if defender.0 as usize == moi => {
                if from_hand {
                    r.defenses_main_total += 1;
                    *r.defenses_main.entry(nom_carte(&g, card)).or_default() += 1;
                } else {
                    r.defenses_equipement += 1;
                }
            }
            Event::Damage {
                source,
                by,
                target,
                target_obj: None,
                amount,
                arcane,
            } => {
                let nom = source.map(|s| nom_carte(&g, s)).unwrap_or("?".into());
                if by.0 as usize == moi && target.0 as usize != moi {
                    r.degats_faits += amount as u64;
                    if arcane {
                        r.degats_faits_arcane += amount as u64;
                    }
                    r.sources.entry(nom).or_default().1 += amount as u64;
                    if mon_tour {
                        degats_tour += amount as u64;
                    }
                } else if target.0 as usize == moi {
                    r.degats_subis += amount as u64;
                    *r.subis_par_source.entry(nom).or_default() += amount as u64;
                }
            }
            _ => {}
        }
    }
    if mon_tour {
        fin_tour(
            &mut r,
            degats_tour,
            cartes_tour,
            ice_tour,
            act_tour,
            att_tour,
        );
    }
    for (k, v) in &r.jouees {
        r.sources.entry(k.clone()).or_default().0 += v;
    }
    Some(r)
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
    let deck = args.get(1).expect("deck étudié").clone();
    let games = get("--games", 20);
    let seed0 = get("--seed", 1);
    let threads = get(
        "--threads",
        std::thread::available_parallelism()
            .map(|n| n.get() as u64)
            .unwrap_or(1),
    );
    let bots: [String; 2] = match args.iter().position(|a| a == "--bots") {
        Some(i) => [args[i + 1].clone(), args[i + 2].clone()],
        None => ["b3".into(), "b3".into()],
    };
    let contre: Vec<String> = match args.iter().position(|a| a == "--contre") {
        Some(i) => args[i + 1].split(',').map(String::from).collect(),
        None => decks::noms(),
    };
    let db = Arc::new(real::build());
    let table = Arc::new(TableSide::lire(&db, fab_ai::side::TABLE_B1).expect("table"));
    let (heros, pool) = lire_deck(&db, &deck);
    let adversaires: Vec<(String, String, CardPool)> = contre
        .iter()
        .map(|n| {
            let (h, p) = lire_deck(&db, n);
            (n.clone(), h, p)
        })
        .collect();
    let mut taches = Vec::new();
    for (k, _) in adversaires.iter().enumerate() {
        for j in 0..games {
            taches.push((k, j));
        }
    }
    let taches = Arc::new(Mutex::new(taches));
    let resultats: Arc<Mutex<Vec<(usize, Agreg)>>> = Arc::new(Mutex::new(Vec::new()));
    let erreurs = Arc::new(Mutex::new(0u64));
    std::thread::scope(|s| {
        for _ in 0..threads {
            let taches = taches.clone();
            let resultats = resultats.clone();
            let erreurs = erreurs.clone();
            let (db, table, pool, adversaires, bots) = (&db, &table, &pool, &adversaires, &bots);
            s.spawn(move || loop {
                let Some((k, j)) = taches.lock().unwrap().pop() else {
                    break;
                };
                let seed = seed0 * 1_000_003 + (k as u64) * 10_007 + j;
                // Alterner la place du deck étudié et qui choisit.
                let moi = (j % 2) as usize;
                let chooser = PlayerId(((j / 2) % 2) as u8);
                let p_adv = &adversaires[k].2;
                let pools = if moi == 0 {
                    [pool, p_adv]
                } else {
                    [p_adv, pool]
                };
                let b = if moi == 0 {
                    [bots[0].as_str(), bots[1].as_str()]
                } else {
                    [bots[1].as_str(), bots[0].as_str()]
                };
                match partie(db, pools, b, table, chooser, seed, moi) {
                    Some(r) => resultats.lock().unwrap().push((k, r)),
                    None => *erreurs.lock().unwrap() += 1,
                }
            });
        }
    });
    let res = resultats.lock().unwrap();
    println!(
        "Analyse de {heros} ({}) contre {} decks, {} parties par deck, bots {} (étudié) contre {}, erreurs {}",
        deck,
        adversaires.len(),
        games,
        bots[0],
        bots[1],
        erreurs.lock().unwrap()
    );
    println!("\n## Par adversaire");
    println!("| adversaire | parties | victoires % | tours | dégâts faits/partie | subis/partie | vie adverse en fin | ma vie en fin |");
    println!("|---|---|---|---|---|---|---|---|");
    let mut tot = Agreg::default();
    let mut gagnees = Agreg::default();
    let mut perdues = Agreg::default();
    for (k, (nom, _, _)) in adversaires.iter().enumerate() {
        let mut a = Agreg::default();
        for (kk, r) in res.iter() {
            if *kk == k {
                a.ajouter(r);
                if r.victoires == 1 {
                    gagnees.ajouter(r);
                } else {
                    perdues.ajouter(r);
                }
            }
        }
        let n = a.parties.max(1) as f64;
        println!(
            "| {nom} | {} | {:.0} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |",
            a.parties,
            100.0 * (a.victoires as f64 + a.nulles as f64 / 2.0) / n,
            a.tours as f64 / n,
            a.degats_faits as f64 / n,
            a.degats_subis as f64 / n,
            a.vie_adverse_fin as f64 / n,
            a.vie_fin as f64 / n
        );
        tot.ajouter(&a);
    }
    for (titre, a) in [
        ("Toutes les parties", &tot),
        ("Parties gagnées", &gagnees),
        ("Parties perdues ou nulles", &perdues),
    ] {
        resume(titre, a);
    }
    let n = tot.parties.max(1) as f64;
    println!("\n## Dégâts infligés au héros adverse par source (toutes parties)");
    println!("| source | fois jouée /partie | dégâts /partie | dégâts par carte jouée |");
    println!("|---|---|---|---|");
    let mut v: Vec<_> = tot.sources.iter().collect();
    v.sort_by_key(|x| std::cmp::Reverse(x.1 .1));
    for (k, (j, d)) in v {
        println!(
            "| {k} | {:.2} | {:.2} | {} |",
            *j as f64 / n,
            *d as f64 / n,
            if *j > 0 {
                format!("{:.2}", *d as f64 / *j as f64)
            } else {
                "—".into()
            }
        );
    }
    liste("Capacités activées", &tot.activees, n, 30);
    liste(
        "Attaques par tour à moi (nombre de tours)",
        &tot.attaques_par_tour,
        n,
        10,
    );
    liste("Dégâts subis par source", &tot.subis_par_source, n, 25);
    liste("Cartes pitchées", &tot.pitchees, n, 40);
    liste(
        "Cartes de la main posées en défense",
        &tot.defenses_main,
        n,
        40,
    );
    liste("Décisions notées", &tot.notes, n, 120);
}

fn resume(titre: &str, a: &Agreg) {
    let n = a.parties.max(1) as f64;
    let t = a.tours_a_moi.max(1) as f64;
    println!("\n## {titre} ({} parties)", a.parties);
    println!(
        "- victoires {:.1} % ; tours par partie {:.1} (dont {:.1} à moi)",
        100.0 * (a.victoires as f64 + a.nulles as f64 / 2.0) / n,
        a.tours as f64 / n,
        t / n
    );
    println!(
        "- dégâts infligés par tour à moi : {:.2} (dont arcane {:.2}) ; subis par partie {:.1}",
        a.degats_faits as f64 / t,
        a.degats_faits_arcane as f64 / t,
        a.degats_subis as f64 / n
    );
    println!(
        "- répartition des dégâts de mes tours : 0 : {:.0} %, 1-3 : {:.0} %, 4-6 : {:.0} %, 7-9 : {:.0} %, 10+ : {:.0} %",
        100.0 * a.histo[0] as f64 / t,
        100.0 * a.histo[1] as f64 / t,
        100.0 * a.histo[2] as f64 / t,
        100.0 * a.histo[3] as f64 / t,
        100.0 * a.histo[4] as f64 / t
    );
    println!(
        "- cartes jouées par tour à moi {:.2} ; tours sans aucune carte jouée {:.0} % ; main en fin de tour {:.2}",
        a.cartes_mes_tours as f64 / t,
        100.0 * a.tours_sans_carte as f64 / t,
        a.main_fin_tour as f64 / t
    );
    println!(
        "- tours où je ne fais rien (ni carte ni capacité) : {:.0} %",
        100.0 * a.tours_rien as f64 / t
    );
    println!(
        "- défense : cartes de la main {:.2} par partie, équipements {:.2} par partie ; tours avec 2 Ice Eternal : {}",
        a.defenses_main_total as f64 / n,
        a.defenses_equipement as f64 / n,
        a.double_ice
    );
}

fn liste(titre: &str, m: &BTreeMap<String, u64>, n: f64, max: usize) {
    println!("\n## {titre} (par partie)");
    let mut v: Vec<_> = m.iter().collect();
    v.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for (k, c) in v.into_iter().take(max) {
        println!("- {k} : {:.2}", *c as f64 / n);
    }
}
