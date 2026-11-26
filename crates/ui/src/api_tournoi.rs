//! Arsenal, lot 8 : « Entraîner » dans la page.
//!
//! - **Tournoi** : comme `fab-sim --tournoi`, mais lancé depuis la page, sur
//!   les decks choisis (du projet et maison), dans des fils d'exécution en
//!   arrière-plan. La page lit l'avancement (matrice qui se remplit, score,
//!   Elo, classement des decks) pendant que les parties tournent. Un
//!   tournoi fini est rangé dans `sauvegardes/tournois/` (historique).
//! - **Banc SPRT** (option du tournoi) : le tournoi s'arrête dès que le test
//!   séquentiel tranche entre « A n'a pas d'avance » (Elo ≤ `elo0`) et « A a
//!   au moins `elo1` d'avance ». Statistique seulement, aucune règle du jeu.
//! - **Tâches d'apprentissage** : `fab-sim --apprendre-side` ou
//!   `--apprendre-b3`, lancés comme programmes à part (le `fab-sim` compilé
//!   à côté de `fab-web`), avec leur sortie affichée dans la page.
//!
//! | Route | Rôle |
//! |---|---|
//! | `GET /api/tournoi` | le tournoi en cours ou le dernier lancé |
//! | `POST /api/tournoi/lancer?decks=a,b&bot_a=b1&bot_b=b0&…` | lance un tournoi |
//! | `POST /api/tournoi/arreter` | arrête le tournoi en cours (résultats gardés) |
//! | `GET /api/tournois` | l'historique |
//! | `GET /api/tournoi/historique?nom=x` | un tournoi de l'historique |
//! | `POST /api/tournoi/oublier?nom=x` | retire un tournoi de l'historique |
//! | `GET /api/tache` | la tâche d'apprentissage en cours ou la dernière |
//! | `POST /api/tache/lancer?quoi=side\|b3&parties=n&threads=t` | lance une tâche |
//! | `POST /api/tache/arreter` | arrête la tâche |

use fab_ai::elo::{ecart, ecart_avec_intervalle, Tableau};
use fab_ai::side::{creer_side, TableSide, SIDES};
use fab_ai::{bot_connu, creer_agent, play_out};
use fab_cards::real;
use fab_engine::{CardDb, CardPool, EndReason, FormatConfig, Outcome, PlayerId};
use fab_match::{lancer_manche, preparer_manche};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

type Resultat = (u16, Value);

fn erreur(code: u16, msg: impl Into<String>) -> Resultat {
    (code, json!({ "erreur": msg.into() }))
}

fn db() -> Arc<CardDb> {
    static DB: OnceLock<Arc<CardDb>> = OnceLock::new();
    DB.get_or_init(|| Arc::new(real::build())).clone()
}

// ---------------------------------------------------------------- tournoi

/// Résultats d'une case de la matrice, du point de vue de A.
#[derive(Clone, Copy, Default, Debug)]
pub struct Case {
    pub victoires: u64,
    pub defaites: u64,
    pub nulles: u64,
    pub erreurs: u64,
    pub tours: u64,
}

impl Case {
    fn ajouter(&mut self, c: &Case) {
        self.victoires += c.victoires;
        self.defaites += c.defaites;
        self.nulles += c.nulles;
        self.erreurs += c.erreurs;
        self.tours += c.tours;
    }
    fn jouees(&self) -> u64 {
        self.victoires + self.defaites + self.nulles
    }
    fn points(&self) -> f64 {
        self.victoires as f64 + self.nulles as f64 / 2.0
    }
    fn score(&self) -> Option<f64> {
        (self.jouees() > 0).then(|| self.points() / self.jouees() as f64)
    }
}

/// Les réglages d'un tournoi.
#[derive(Clone, Debug)]
pub struct Reglage {
    pub decks: Vec<String>,
    pub bots: [String; 2],
    pub sides: [String; 2],
    /// Parties par paire de decks (plafond en mode SPRT).
    pub parties: u64,
    pub threads: u64,
    pub graine: u64,
    /// Banc SPRT : `(elo0, elo1)`.
    pub sprt: Option<(f64, f64)>,
}

struct Tournoi {
    reglage: Reglage,
    heros: Vec<String>,
    total: u64,
    debut: Instant,
    date: u64,
    fait: AtomicU64,
    arret: AtomicBool,
    cases: Mutex<Vec<Case>>,
    messages: Mutex<Vec<String>>,
    /// Durée (s) une fois tous les fils finis.
    fin: Mutex<Option<f64>>,
    /// Pourquoi le tournoi s'est arrêté avant la fin (bouton, SPRT).
    raison_arret: Mutex<Option<String>>,
    /// Nom du fichier d'historique, une fois écrit.
    fichier: Mutex<Option<String>>,
}

fn courant() -> &'static Mutex<Option<Arc<Tournoi>>> {
    static T: Mutex<Option<Arc<Tournoi>>> = Mutex::new(None);
    &T
}

fn dossier_reglable() -> &'static Mutex<Option<PathBuf>> {
    static D: Mutex<Option<PathBuf>> = Mutex::new(None);
    &D
}

/// Le dossier de l'historique des tournois (`sauvegardes/tournois/` du
/// dossier courant, comme la sauvegarde de la partie).
pub fn dossier_tournois() -> PathBuf {
    dossier_reglable()
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| PathBuf::from("sauvegardes").join("tournois"))
}

/// Change le dossier de l'historique (tests).
pub fn utiliser_dossier_tournois(d: PathBuf) {
    *dossier_reglable().lock().unwrap() = Some(d);
}

/// Bornes du SPRT à 5 % d'erreur de chaque côté : log((1-β)/α).
const BORNE_SPRT: f64 = 2.944_438_979;
/// Pas de conclusion SPRT avant ce nombre de parties (variance trop mal estimée).
const SPRT_MINIMUM: u64 = 30;

/// Log-rapport de vraisemblance du SPRT (approximation normale, comme les
/// bancs d'essai des moteurs d'échecs) pour `points` en `n` parties dont
/// `nulles` nulles, entre les hypothèses `elo0` et `elo1`.
pub fn llr(points: f64, n: f64, nulles: f64, elo0: f64, elo1: f64) -> f64 {
    if n < 1.0 {
        return 0.0;
    }
    let attendu = |e: f64| 1.0 / (1.0 + 10f64.powf(-e / 400.0));
    let (s0, s1) = (attendu(elo0), attendu(elo1));
    let s = points / n;
    let gagnees = points - nulles / 2.0;
    let var = ((gagnees + nulles / 4.0) / n - s * s).max(1e-3);
    (s1 - s0) * (2.0 * points - n * (s0 + s1)) / (2.0 * var)
}

fn sprt_json(r: &Reglage, tot: &Case) -> Value {
    let Some((e0, e1)) = r.sprt else {
        return Value::Null;
    };
    let l = llr(tot.points(), tot.jouees() as f64, tot.nulles as f64, e0, e1);
    let conclusion = if tot.jouees() < SPRT_MINIMUM {
        None
    } else if l >= BORNE_SPRT {
        Some("h1")
    } else if l <= -BORNE_SPRT {
        Some("h0")
    } else {
        None
    };
    json!({ "elo0": e0, "elo1": e1, "llr": l, "borne": BORNE_SPRT, "conclusion": conclusion })
}

/// Lit les réglages d'un tournoi dans les paramètres de la requête.
pub fn lire_reglage(param: &dyn Fn(&str) -> Option<String>) -> Result<Reglage, String> {
    let decks: Vec<String> = param("decks")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if decks.is_empty() {
        return Err("aucun deck choisi".into());
    }
    let bot = |k: &str, d: &str| -> Result<String, String> {
        let b = param(k).unwrap_or_else(|| d.into()).to_lowercase();
        if bot_connu(&b) {
            Ok(b)
        } else {
            Err(format!("bot inconnu : {b}"))
        }
    };
    let side = |k: &str| -> Result<String, String> {
        let s = param(k).unwrap_or_else(|| "appris".into()).to_lowercase();
        if SIDES.contains(&s.as_str()) {
            Ok(s)
        } else {
            Err(format!(
                "side inconnu : {s} (connus : {})",
                SIDES.join(", ")
            ))
        }
    };
    let nombre = |k: &str, d: u64, max: u64| -> Result<u64, String> {
        match param(k) {
            None => Ok(d),
            Some(v) => match v.parse::<u64>() {
                Ok(n) if (1..=max).contains(&n) => Ok(n),
                _ => Err(format!("{k} : nombre de 1 à {max} attendu, pas « {v} »")),
            },
        }
    };
    let par_defaut = std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(2)
        .saturating_sub(1)
        .max(1);
    let sprt = match param("sprt").filter(|s| !s.is_empty()) {
        None => None,
        Some(v) => {
            let e1: f64 = v
                .parse()
                .ok()
                .filter(|e: &f64| *e > 0.0 && *e <= 400.0)
                .ok_or_else(|| format!("sprt : écart d'Elo de 1 à 400 attendu, pas « {v} »"))?;
            Some((0.0, e1))
        }
    };
    Ok(Reglage {
        decks,
        bots: [bot("bot_a", "b1")?, bot("bot_b", "b0")?],
        sides: [side("side_a")?, side("side_b")?],
        parties: nombre("parties", 10, 100_000)?,
        threads: nombre("threads", par_defaut, 64)?,
        graine: nombre("graine", 1, u64::MAX)?,
        sprt,
    })
}

/// Lance un tournoi en arrière-plan. Refusé si un tournoi tourne déjà.
pub fn lancer(r: Reglage) -> Result<(), String> {
    let mut c = courant().lock().unwrap();
    if c.as_ref().is_some_and(|t| t.fin.lock().unwrap().is_none()) {
        return Err("un tournoi est déjà en cours".into());
    }
    let db = db();
    let mut pools: Vec<CardPool> = Vec::new();
    let mut heros = Vec::new();
    for d in &r.decks {
        let p = crate::charger_pool(&db, d).map_err(|e| format!("deck {d} : {e}"))?;
        heros.push(db.get(p.hero).name.to_string());
        pools.push(p);
    }
    let n = pools.len() as u64;
    let t = Arc::new(Tournoi {
        total: n * n * r.parties,
        heros,
        debut: Instant::now(),
        date: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        fait: AtomicU64::new(0),
        arret: AtomicBool::new(false),
        cases: Mutex::new(vec![Case::default(); (n * n) as usize]),
        messages: Mutex::new(Vec::new()),
        fin: Mutex::new(None),
        raison_arret: Mutex::new(None),
        fichier: Mutex::new(None),
        reglage: r,
    });
    *c = Some(t.clone());
    drop(c);
    std::thread::spawn(move || derouler(&db, &pools, &t));
    Ok(())
}

/// Le corps du tournoi : `threads` fils prennent les parties une à une.
/// Les parties sont numérotées tour par tour (une partie de chaque paire,
/// puis la deuxième de chaque paire…) pour que la matrice se remplisse
/// partout à la fois, et qu'un arrêt en cours (bouton, SPRT) donne des
/// résultats équilibrés entre les paires.
fn derouler(db: &Arc<CardDb>, pools: &[CardPool], t: &Arc<Tournoi>) {
    let r = &t.reglage;
    let n = pools.len();
    let cases = (n * n) as u64;
    let suivant = AtomicU64::new(0);
    let table: Option<Arc<TableSide>> = r
        .sides
        .iter()
        .any(|s| s == "appris" || s == "general")
        .then(|| {
            Arc::new(TableSide::lire(db, fab_ai::side::TABLE_B1).expect("data/side/b1.txt valide"))
        });
    let taille = FormatConfig::silver_age().deck_size as usize;
    std::thread::scope(|s| {
        for _ in 0..r.threads {
            let (suivant, table) = (&suivant, &table);
            s.spawn(move || loop {
                if t.arret.load(Ordering::Relaxed) {
                    break;
                }
                let job = suivant.fetch_add(1, Ordering::Relaxed);
                if job >= t.total {
                    break;
                }
                let case = (job % cases) as usize;
                let k = job / cases;
                let (da, dbb) = (case / n, case % n);
                // Alterner la place de A et le joueur qui choisit (CR 4.1).
                let a_place = (k % 2) as usize;
                let chooser = PlayerId(((k / 2) % 2) as u8);
                let seed = r.graine.wrapping_mul(1_000_003).wrapping_add(job);
                let mut p = [&pools[da], &pools[dbb]];
                let mut b = [r.bots[0].as_str(), r.bots[1].as_str()];
                let mut sd = [r.sides[0].as_str(), r.sides[1].as_str()];
                if a_place == 1 {
                    p.swap(0, 1);
                    b.swap(0, 1);
                    sd.swap(0, 1);
                }
                let res = (|| -> Result<_, String> {
                    let mut s0 = creer_side(sd[0], db, table.clone(), seed ^ 0x51DE)
                        .ok_or("side inconnu")?;
                    let mut s1 = creer_side(sd[1], db, table.clone(), seed ^ 0xE51D)
                        .ok_or("side inconnu")?;
                    let m = preparer_manche(db, p, [&mut *s0, &mut *s1], chooser, taille)
                        .map_err(|(j, e)| format!("choix du joueur {} : {e:?}", j.0 + 1))?;
                    let mut g =
                        lancer_manche(db.clone(), &m, seed, 200).map_err(|e| format!("{e:?}"))?;
                    let mut x = creer_agent(b[0], seed ^ 0xA5A5).ok_or("bot inconnu")?;
                    let mut y = creer_agent(b[1], seed ^ 0x5A5A).ok_or("bot inconnu")?;
                    play_out(&mut g, [&mut *x, &mut *y], false)?;
                    Ok(g)
                })();
                let mut c = Case::default();
                match res {
                    Ok(g) => {
                        c.tours = g.turn as u64;
                        match g.outcome {
                            Some(Outcome::Winner(w)) if w.index() == a_place => c.victoires = 1,
                            Some(Outcome::Winner(_)) => c.defaites = 1,
                            _ => c.nulles = 1,
                        }
                        if g.end_reason == Some(EndReason::TurnLimit) {
                            let mut m = t.messages.lock().unwrap();
                            if m.len() < 5 {
                                m.push(format!(
                                    "graine {seed} : nulle à la limite de 200 tours ({} contre {})",
                                    r.decks[da], r.decks[dbb]
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        c.erreurs = 1;
                        let mut m = t.messages.lock().unwrap();
                        if m.len() < 5 {
                            m.push(format!("ERREUR graine {seed} : {e}"));
                        }
                    }
                }
                let mut v = t.cases.lock().unwrap();
                v[case].ajouter(&c);
                t.fait.fetch_add(1, Ordering::Relaxed);
                if r.sprt.is_some() {
                    let mut tot = Case::default();
                    for x in v.iter() {
                        tot.ajouter(x);
                    }
                    if let Some(cl) = sprt_json(r, &tot)["conclusion"].as_str() {
                        if !t.arret.swap(true, Ordering::Relaxed) {
                            *t.raison_arret.lock().unwrap() = Some(format!("SPRT : {cl}"));
                        }
                    }
                }
            });
        }
    });
    *t.fin.lock().unwrap() = Some(t.debut.elapsed().as_secs_f64());
    // Historique (un échec d'écriture n'empêche pas de lire le résultat).
    let v = etat_json(t);
    let nom = format!("tournoi-{}", t.date);
    let d = dossier_tournois();
    let ecrit = std::fs::create_dir_all(&d).and_then(|_| {
        std::fs::write(
            d.join(format!("{nom}.json")),
            serde_json::to_string_pretty(&v).unwrap_or_default(),
        )
    });
    match ecrit {
        Ok(()) => *t.fichier.lock().unwrap() = Some(nom),
        Err(e) => t
            .messages
            .lock()
            .unwrap()
            .push(format!("historique non écrit ({}) : {e}", d.display())),
    }
}

fn case_json(c: &Case) -> Value {
    json!({
        "v": c.victoires, "d": c.defaites, "n": c.nulles, "e": c.erreurs,
        "score": c.score(),
    })
}

/// L'état d'un tournoi (en cours ou fini) pour la page.
fn etat_json(t: &Tournoi) -> Value {
    let r = &t.reglage;
    let n = r.decks.len();
    let cases = t.cases.lock().unwrap().clone();
    let mut tot = Case::default();
    for c in &cases {
        tot.ajouter(c);
    }
    let matrice: Vec<Vec<Value>> = (0..n)
        .map(|i| (0..n).map(|j| case_json(&cases[i * n + j])).collect())
        .collect();
    let somme = |f: &dyn Fn(usize) -> usize| {
        let mut s = Case::default();
        for k in 0..n {
            s.ajouter(&cases[f(k)]);
        }
        s
    };
    let lignes: Vec<Value> = (0..n).map(|i| case_json(&somme(&|j| i * n + j))).collect();
    let colonnes: Vec<Value> = (0..n).map(|j| case_json(&somme(&|i| i * n + j))).collect();
    // Classement des decks : Elo (Bradley-Terry) d'après les paires de
    // decks différents, centré sur la moyenne. Il a du sens quand A et B
    // sont le même bot ; sinon il mêle force du deck et force du bot.
    let noms: Vec<&str> = r.decks.iter().map(|s| s.as_str()).collect();
    let mut tab = Tableau::new(&noms);
    for i in 0..n {
        for j in 0..n {
            let c = &cases[i * n + j];
            if i != j && c.jouees() > 0 {
                tab.points[i][j] += c.points();
                tab.points[j][i] += c.jouees() as f64 - c.points();
                tab.parties[i][j] += c.jouees() as f64;
                tab.parties[j][i] += c.jouees() as f64;
            }
        }
    }
    let elo = if n > 1 { tab.elo() } else { vec![0.0] };
    let moy = elo.iter().sum::<f64>() / elo.len().max(1) as f64;
    let mut classement: Vec<Value> = (0..n)
        .map(|i| json!({ "deck": r.decks[i], "heros": t.heros[i], "elo": elo[i] - moy }))
        .collect();
    classement.sort_by(|a, b| {
        b["elo"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["elo"].as_f64().unwrap_or(0.0))
    });
    let fin = *t.fin.lock().unwrap();
    let secondes = fin.unwrap_or_else(|| t.debut.elapsed().as_secs_f64());
    let jouees = tot.jouees() as f64;
    let (e, bas, haut) = ecart_avec_intervalle(tot.points(), jouees, tot.nulles as f64);
    let fait = t.fait.load(Ordering::Relaxed);
    json!({
        "en_cours": fin.is_none(),
        "date": t.date,
        "fichier": *t.fichier.lock().unwrap(),
        "arret": *t.raison_arret.lock().unwrap(),
        "reglage": {
            "decks": r.decks, "heros": t.heros, "bots": r.bots, "sides": r.sides,
            "parties": r.parties, "threads": r.threads, "graine": r.graine,
            "sprt": r.sprt.map(|(a, b)| json!([a, b])),
        },
        "fait": fait,
        "total": t.total,
        "secondes": secondes,
        "parties_par_s": if secondes > 0.0 { fait as f64 / secondes } else { 0.0 },
        "matrice": matrice,
        "lignes": lignes,
        "colonnes": colonnes,
        "classement": classement,
        "total_a": {
            "v": tot.victoires, "d": tot.defaites, "n": tot.nulles, "e": tot.erreurs,
            "score": tot.score(),
            "elo": (jouees > 0.0).then_some(e),
            "elo_bas": (jouees > 0.0).then_some(bas),
            "elo_haut": (jouees > 0.0).then_some(haut),
            "tours_moyens": if jouees > 0.0 { tot.tours as f64 / jouees } else { 0.0 },
        },
        "elo_ligne": lignes_elo(&cases, n),
        "sprt": sprt_json(r, &tot),
        "messages": *t.messages.lock().unwrap(),
    })
}

fn lignes_elo(cases: &[Case], n: usize) -> Vec<Option<f64>> {
    (0..n)
        .map(|i| {
            let mut s = Case::default();
            for j in 0..n {
                s.ajouter(&cases[i * n + j]);
            }
            s.score().map(ecart)
        })
        .collect()
}

/// L'état du tournoi courant (`None` : aucun tournoi lancé depuis le
/// démarrage de `fab-web`).
pub fn etat() -> Option<Value> {
    courant().lock().unwrap().as_ref().map(|t| etat_json(t))
}

fn nom_historique_valide(nom: &str) -> bool {
    !nom.is_empty()
        && nom.len() <= 60
        && nom
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn historique() -> Vec<Value> {
    let Ok(l) = std::fs::read_dir(dossier_tournois()) else {
        return Vec::new();
    };
    let mut v: Vec<Value> = l
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let p = e.path();
            let nom = p.file_stem()?.to_str()?.to_string();
            if p.extension()? != "json" || !nom_historique_valide(&nom) {
                return None;
            }
            let t: Value = serde_json::from_str(&std::fs::read_to_string(&p).ok()?).ok()?;
            Some(json!({
                "nom": nom,
                "date": t["date"],
                "reglage": t["reglage"],
                "fait": t["fait"],
                "total_a": t["total_a"],
                "arret": t["arret"],
            }))
        })
        .collect();
    v.sort_by_key(|x| std::cmp::Reverse(x["date"].as_u64().unwrap_or(0)));
    v
}

// ---------------------------------------------------------------- tâches

struct Tache {
    quoi: String,
    commande: String,
    debut: Instant,
    journal: Arc<Mutex<Vec<String>>>,
    enfant: Mutex<Child>,
    code: Mutex<Option<Option<i32>>>,
}

fn tache_courante() -> &'static Mutex<Option<Arc<Tache>>> {
    static T: Mutex<Option<Arc<Tache>>> = Mutex::new(None);
    &T
}

/// Le `fab-sim` compilé à côté de ce programme (`target/release/`).
fn chemin_fab_sim() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let nom = if cfg!(windows) {
        "fab-sim.exe"
    } else {
        "fab-sim"
    };
    let p = exe.parent()?.join(nom);
    p.exists().then_some(p)
}

fn garder(journal: &Mutex<Vec<String>>, ligne: String) {
    let mut j = journal.lock().unwrap();
    j.push(ligne);
    if j.len() > 400 {
        j.drain(..100);
    }
}

fn lancer_tache(quoi: &str, parties: u64, threads: u64) -> Result<(), String> {
    let mut c = tache_courante().lock().unwrap();
    if let Some(t) = c.as_ref() {
        if t.code.lock().unwrap().is_none()
            && t.enfant.lock().unwrap().try_wait().ok().flatten().is_none()
        {
            return Err("une tâche est déjà en cours".into());
        }
    }
    let option = match quoi {
        "side" => "--apprendre-side",
        "b3" => "--apprendre-b3",
        _ => return Err(format!("tâche inconnue : {quoi} (side ou b3)")),
    };
    let exe = chemin_fab_sim().ok_or(
        "fab-sim introuvable à côté de fab-web : compilez-le avec \
         « cargo build --release -p fab-sim »",
    )?;
    let args = [
        option.to_string(),
        "--games".into(),
        parties.to_string(),
        "--threads".into(),
        threads.to_string(),
    ];
    let mut enfant = Command::new(&exe)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("lancement de {} : {e}", exe.display()))?;
    let journal = Arc::new(Mutex::new(Vec::new()));
    let sortie = enfant.stdout.take();
    let erreurs = enfant.stderr.take();
    for flux in [
        sortie.map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
        erreurs.map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
    ]
    .into_iter()
    .flatten()
    {
        let j = journal.clone();
        std::thread::spawn(move || {
            for l in BufReader::new(flux).lines().map_while(Result::ok) {
                garder(&j, l);
            }
        });
    }
    *c = Some(Arc::new(Tache {
        quoi: quoi.into(),
        commande: format!("fab-sim {}", args.join(" ")),
        debut: Instant::now(),
        journal,
        enfant: Mutex::new(enfant),
        code: Mutex::new(None),
    }));
    Ok(())
}

fn etat_tache() -> Value {
    let c = tache_courante().lock().unwrap();
    let Some(t) = c.as_ref() else {
        return json!({ "aucune": true, "fab_sim": chemin_fab_sim().is_some() });
    };
    let mut code = t.code.lock().unwrap();
    if code.is_none() {
        if let Ok(Some(s)) = t.enfant.lock().unwrap().try_wait() {
            *code = Some(s.code());
        }
    }
    json!({
        "quoi": t.quoi,
        "commande": t.commande,
        "en_cours": code.is_none(),
        "code": code.flatten(),
        "secondes": t.debut.elapsed().as_secs_f64(),
        "journal": *t.journal.lock().unwrap(),
        "fab_sim": true,
    })
}

// ---------------------------------------------------------------- routes

/// Les routes « Entraîner » ; `None` si le chemin n'est pas à ce module.
pub fn repondre(
    methode: &str,
    chemin: &str,
    param: &dyn Fn(&str) -> Option<String>,
) -> Option<Resultat> {
    Some(match (methode, chemin) {
        ("GET", "/api/tournoi") => (200, etat().unwrap_or_else(|| json!({ "aucun": true }))),
        ("POST", "/api/tournoi/lancer") => match lire_reglage(param).and_then(lancer) {
            Ok(()) => (200, etat().unwrap_or(Value::Null)),
            Err(e) if e.contains("déjà") => erreur(409, e),
            Err(e) => erreur(400, e),
        },
        ("POST", "/api/tournoi/arreter") => match courant().lock().unwrap().as_ref() {
            Some(t) => {
                if !t.arret.swap(true, Ordering::Relaxed) {
                    *t.raison_arret.lock().unwrap() = Some("arrêté depuis la page".into());
                }
                (200, json!({ "ok": true }))
            }
            None => erreur(400, "aucun tournoi en cours"),
        },
        ("GET", "/api/tournois") => (
            200,
            json!({ "tournois": historique(), "dossier": dossier_tournois() }),
        ),
        ("GET", "/api/tournoi/historique") | ("POST", "/api/tournoi/oublier") => {
            let nom = param("nom").unwrap_or_default();
            if !nom_historique_valide(&nom) {
                return Some(erreur(400, format!("nom invalide : {nom}")));
            }
            let p = dossier_tournois().join(format!("{nom}.json"));
            if methode == "POST" {
                match std::fs::remove_file(&p) {
                    Ok(()) => (200, json!({ "ok": true })),
                    Err(e) => erreur(404, format!("{nom} : {e}")),
                }
            } else {
                match std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                {
                    Some(v) => (200, v),
                    None => erreur(404, format!("tournoi inconnu : {nom}")),
                }
            }
        }
        ("GET", "/api/tache") => (200, etat_tache()),
        ("POST", "/api/tache/lancer") => {
            let n = |k: &str, d: u64| {
                param(k)
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&x| (1..=10_000_000).contains(&x))
                    .unwrap_or(d)
            };
            let quoi = param("quoi").unwrap_or_default();
            match lancer_tache(&quoi, n("parties", 200), n("threads", 4)) {
                Ok(()) => (200, etat_tache()),
                Err(e) if e.contains("déjà") => erreur(409, e),
                Err(e) => erreur(400, e),
            }
        }
        ("POST", "/api/tache/arreter") => match tache_courante().lock().unwrap().as_ref() {
            Some(t) => {
                let _ = t.enfant.lock().unwrap().kill();
                (200, json!({ "ok": true }))
            }
            None => erreur(400, "aucune tâche en cours"),
        },
        _ => return None,
    })
}
