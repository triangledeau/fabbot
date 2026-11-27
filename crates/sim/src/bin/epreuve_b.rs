//! epreuve-b — regret de B3 contre une recherche bien plus longue
//! (épreuve B, `docs/epreuves.md`).
//!
//! Usage :
//!     cargo run --release -p fab-sim --bin epreuve-b -- [--parties 81]
//!         [--une-sur 10] [--mondes 400] [--simulation b1] [--tours 0]
//!         [--bot b3] [--decks ira,hala,...] [--seed 1] [--threads N]
//!         [--details fichier.tsv]
//!
//! 1. **Banque de positions** : `--parties` parties `--bot` contre `--bot`
//!    (paires de decks parcourues dans l'ordre, 81 = chaque paire une
//!    fois). Parmi les décisions où le bot a vraiment le choix (au moins
//!    deux options différentes), une sur `--une-sur` est gardée, avec le
//!    choix que le bot y a fait. La banque est reproductible : mêmes
//!    graines, même version du code, mêmes positions.
//! 2. **Référence** : pour chaque position, chaque option est notée sur les
//!    **mêmes** `--mondes` mondes imaginés (main adverse et ordre des decks
//!    tirés au hasard selon ce que voit le joueur, comme B2/B3), puis la
//!    partie est finie par `--simulation` pour les deux joueurs (B1 par
//!    défaut), jusqu'à la fin (`--tours 0`) ou pendant N tours puis estimée
//!    par l'évaluation de B3.
//! 3. **Regret** : chances de gagner (référence) de la meilleure option moins
//!    celles de l'option du bot. Pour ne pas surestimer le regret (prendre
//!    le maximum de notes bruitées les gonfle), la meilleure option est
//!    choisie sur les mondes **pairs** et le regret mesuré sur les mondes
//!    **impairs** : il peut donc être négatif sur une position (le bot a
//!    fait mieux que l'option choisie par la moitié des mondes), et sa
//!    moyenne est sans biais. Moyenne par type de décision et par deck,
//!    part des décisions où l'écart est net (plus de 2 erreurs types,
//!    comparaison appariée sur les mêmes mondes).

#[path = "commun/mod.rs"]
mod commun;

use commun::{lire_decks, table_side, Options};
use fab_ai::b3::{Evaluation, Modele};
use fab_ai::determinisation::determiniser;
use fab_ai::side::creer_side;
use fab_ai::tour_exact::decrire_choix;
use fab_ai::{bot_connu, creer_agent, Agent};
use fab_cards::real;
use fab_engine::{Choice, Decision, DecisionKind, FormatConfig, GameState, Outcome, PlayerId, Rng};
use fab_match::{lancer_manche, preparer_manche};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Une position gardée, avec le choix du bot.
struct Position {
    g: GameState,
    choix: usize,
    numero: u32,
}

/// Agent qui garde une décision sur `une_sur` parmi celles qui ont au moins
/// deux options différentes.
struct Banque {
    inner: Box<dyn Agent + Send>,
    une_sur: u64,
    rng: Rng,
    compteur: u32,
    positions: Vec<Position>,
}

impl Agent for Banque {
    fn name(&self) -> &str {
        "banque"
    }
    fn choose(&mut self, g: &GameState, d: &Decision) -> usize {
        let i = self.inner.choose(g, d);
        if options_distinctes(g, d).len() >= 2 {
            self.compteur += 1;
            if self.rng.below(self.une_sur as u32) == 0 {
                self.positions.push(Position {
                    g: g.clone(),
                    choix: i,
                    numero: self.compteur,
                });
            }
        }
        i
    }
}

/// Les options de `d`, une par option différente (deux exemplaires
/// identiques d'une carte en main comptent pour une) : (clé, index).
fn options_distinctes(g: &GameState, d: &Decision) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = Vec::new();
    for (i, &c) in d.options.iter().enumerate() {
        let k = cle(g, c);
        if !v.iter().any(|(x, _)| *x == k) {
            v.push((k, i));
        }
    }
    v
}

/// Clé d'une option : sa description, plus l'objet pour un objet de l'arène
/// (deux équipements identiques ne sont pas interchangeables).
fn cle(g: &GameState, c: Choice) -> String {
    use fab_engine::Zone;
    let objet = match c {
        Choice::Play(o)
        | Choice::Activate(o, _)
        | Choice::Pitch(o)
        | Choice::Defend(o)
        | Choice::Arsenal(o)
        | Choice::PitchToBottom(o)
        | Choice::Object(o) => Some(o),
        Choice::Target(fab_engine::Target::Object(r)) => Some(r.id),
        _ => None,
    };
    let mut t = decrire_choix(g, c);
    if let Some(o) = objet {
        let z = g.obj(o).zone;
        if matches!(
            z,
            Zone::Hand | Zone::Deck | Zone::Arsenal | Zone::Pitch | Zone::Graveyard
        ) {
            let _ = write!(t, " [{z:?}, joueur {}]", g.obj(o).owner.0);
        } else {
            let _ = write!(t, " [objet {}]", o.0);
        }
    }
    if let Choice::Prevent(p) = c {
        let _ = write!(t, " {p:?}");
    }
    t
}

/// Type de décision affiché (la priorité est séparée entre son tour et le
/// tour adverse).
fn type_de(g: &GameState, d: &Decision) -> String {
    match d.kind {
        DecisionKind::Priority if g.turn_player == d.player => "priorité (son tour)".into(),
        DecisionKind::Priority => "priorité (tour adverse)".into(),
        DecisionKind::Defend => "défense".into(),
        DecisionKind::Pitch => "pitch".into(),
        DecisionKind::Arsenal => "arsenal".into(),
        DecisionKind::PitchOrder => "ordre du pitch".into(),
        DecisionKind::Target | DecisionKind::AttackTarget => "cible".into(),
        DecisionKind::Prevention => "prévention".into(),
        k => format!("{k:?}"),
    }
}

/// Réglages de la référence.
struct Reference {
    mondes: u32,
    simulation: String,
    tours: u32,
}

/// Note de la référence pour une option : somme des résultats, nombre de
/// mondes, et résultat par monde (pour les comparaisons appariées).
struct Note {
    cle: String,
    index: usize,
    par_monde: Vec<Option<f32>>,
}

impl Note {
    /// Moyenne sur les mondes de rang `parite` (0 : pairs, 1 : impairs).
    fn moyenne(&self, parite: usize) -> f64 {
        let v: Vec<f32> = self
            .par_monde
            .iter()
            .skip(parite)
            .step_by(2)
            .flatten()
            .copied()
            .collect();
        v.iter().map(|&x| x as f64).sum::<f64>() / v.len().max(1) as f64
    }
}

/// Écart apparié (a − b) sur les mondes impairs où les deux options
/// existent : moyenne et erreur type.
fn ecart_apparie(a: &Note, b: &Note) -> (f64, f64) {
    let d: Vec<f64> = a
        .par_monde
        .iter()
        .zip(&b.par_monde)
        .skip(1)
        .step_by(2)
        .filter_map(|(x, y)| Some((*x)? as f64 - (*y)? as f64))
        .collect();
    let n = d.len().max(1) as f64;
    let m = d.iter().sum::<f64>() / n;
    let var = d.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0).max(1.0);
    (m, (var / n).sqrt())
}

/// Finit la partie (ou N tours) avec l'agent de simulation ; résultat pour `moi`.
fn simuler(
    mut g: GameState,
    moi: PlayerId,
    r: &Reference,
    agents: &mut [Box<dyn Agent + Send>; 2],
    eval: &Evaluation,
) -> f32 {
    let tour0 = g.turn;
    let mut n = 0u32;
    while let Some(d) = g.pending.as_ref() {
        if r.tours > 0 && g.turn >= tour0 + r.tours {
            let v0 = eval.estimer_pour_la_recherche(&g);
            return if moi.0 == 0 { v0 } else { 1.0 - v0 };
        }
        let i = if d.options.len() == 1 {
            0
        } else {
            let d = d.clone();
            agents[d.player.index()].choose(&g, &d)
        };
        let _ = g.apply(i);
        n += 1;
        if n > 200_000 {
            break;
        }
    }
    match g.outcome {
        Some(Outcome::Winner(p)) if p == moi => 1.0,
        Some(Outcome::Winner(_)) => 0.0,
        _ => 0.5,
    }
}

/// Note toutes les options de la position sur les mêmes mondes.
fn noter(p: &Position, r: &Reference, graine: u64, eval: &Evaluation) -> Vec<Note> {
    let d = p.g.pending.clone().expect("décision");
    let moi = d.player;
    let mut notes: Vec<Note> = options_distinctes(&p.g, &d)
        .into_iter()
        .map(|(cle, index)| Note {
            cle,
            index,
            par_monde: Vec::with_capacity(r.mondes as usize),
        })
        .collect();
    let mut rng = Rng::new(graine);
    let mut agents: [Box<dyn Agent + Send>; 2] = [
        creer_agent(&r.simulation, graine ^ 1).expect("bot"),
        creer_agent(&r.simulation, graine ^ 2).expect("bot"),
    ];
    for _ in 0..r.mondes {
        let monde = determiniser(&p.g, moi, &mut rng);
        let dm = monde.pending.clone().expect("décision");
        let opts = options_distinctes(&monde, &dm);
        let graine_monde = rng.next_u64();
        for n in notes.iter_mut() {
            let v = opts.iter().find(|(k, _)| *k == n.cle).map(|&(_, i)| {
                let mut s = monde.clone();
                // Même hasard pour toutes les options de ce monde.
                s.rng = Rng::new(graine_monde);
                let _ = s.apply(i);
                simuler(s, moi, r, &mut agents, eval)
            });
            n.par_monde.push(v);
        }
    }
    notes
}

/// Résultat pour une position.
struct Mesure {
    graine: u64,
    numero: u32,
    deck: usize,
    adverse: usize,
    tour: u32,
    type_decision: String,
    options: usize,
    choix_bot: String,
    meilleur: String,
    regret: f64,
    erreur_type: f64,
    net: bool,
}

#[derive(Default)]
struct Bilan {
    n: u64,
    regret: f64,
    nets: u64,
    meilleurs: u64,
}

impl Bilan {
    fn ajouter(&mut self, m: &Mesure) {
        self.n += 1;
        self.regret += m.regret;
        if m.net {
            self.nets += 1;
        }
        if m.choix_bot == m.meilleur {
            self.meilleurs += 1;
        }
    }
    fn ligne(&self, nom: &str) -> String {
        let n = self.n.max(1) as f64;
        format!(
            "| {nom} | {} | {:.2} | {:.1} % | {:.1} % |",
            self.n,
            100.0 * self.regret / n,
            100.0 * self.nets as f64 / n,
            100.0 * self.meilleurs as f64 / n
        )
    }
}

fn main() {
    let o = Options::lire();
    let parties = o.nombre("--parties", 81);
    let une_sur = o.nombre("--une-sur", 10).max(1);
    let seed0 = o.nombre("--seed", 1);
    let threads = o.threads();
    let bot = o.texte("--bot").unwrap_or("b3").to_lowercase();
    let reference = Reference {
        mondes: o.nombre("--mondes", 400).max(2) as u32,
        simulation: o.texte("--simulation").unwrap_or("b1").to_lowercase(),
        tours: o.nombre("--tours", 0) as u32,
    };
    for b in [&bot, &reference.simulation] {
        if !bot_connu(b) {
            eprintln!("bot inconnu « {b} »");
            std::process::exit(2);
        }
    }
    let db = Arc::new(real::build());
    let eval = Modele::par_defaut().indexer(&db);
    let table = table_side(&db);
    let decks = lire_decks(&db, o.texte("--decks"));
    let n = decks.len() as u64;
    println!(
        "epreuve-b — regret de {bot} : {parties} parties {bot} contre {bot} ({n} decks), \
         une décision sur {une_sur} gardée ; référence : {} mondes par position, \
         parties simulées par {} {}, {threads} threads, graine {seed0}",
        reference.mondes,
        reference.simulation,
        if reference.tours == 0 {
            "jusqu'à la fin".to_string()
        } else {
            format!("pendant {} tours puis évaluation de B3", reference.tours)
        }
    );
    let debut = Instant::now();
    let suivant = AtomicU64::new(0);
    let erreurs = AtomicU64::new(0);
    let fini = AtomicU64::new(0);
    let tous: Mutex<Vec<Mesure>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            let (db, table, decks, bot, reference, eval) =
                (&db, &table, &decks, &bot, &reference, &eval);
            let (suivant, erreurs, fini, tous) = (&suivant, &erreurs, &fini, &tous);
            s.spawn(move || loop {
                let k = suivant.fetch_add(1, Ordering::Relaxed);
                if k >= parties {
                    break;
                }
                let paire = k % (n * n);
                let (i, j) = ((paire / n) as usize, (paire % n) as usize);
                let graine = seed0.wrapping_mul(1_000_003).wrapping_add(k);
                let positions = match banque(
                    db,
                    table,
                    [&decks[i].pool, &decks[j].pool],
                    bot,
                    graine,
                    une_sur,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("ERREUR partie graine {graine} : {e}");
                        erreurs.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                };
                let mut v = Vec::new();
                for p in &positions {
                    let d = p.g.pending.as_ref().expect("décision");
                    let moi = d.player;
                    let graine_ref = graine ^ ((p.numero as u64) << 24) ^ ((moi.0 as u64) << 60);
                    let notes = noter(p, reference, graine_ref, eval);
                    let cle_bot = cle(&p.g, d.options[p.choix]);
                    let Some(bot_note) = notes.iter().find(|n| n.cle == cle_bot) else {
                        continue;
                    };
                    let meilleure = notes
                        .iter()
                        .max_by(|a, b| a.moyenne(0).total_cmp(&b.moyenne(0)))
                        .expect("options");
                    let (regret, et) = ecart_apparie(meilleure, bot_note);
                    let (deck, adverse) = if moi.0 == 0 { (i, j) } else { (j, i) };
                    v.push(Mesure {
                        graine,
                        numero: p.numero,
                        deck,
                        adverse,
                        tour: p.g.turn,
                        type_decision: type_de(&p.g, d),
                        options: notes.len(),
                        choix_bot: decrire_choix(&p.g, d.options[p.choix]),
                        meilleur: decrire_choix(&p.g, d.options[meilleure.index]),
                        regret,
                        erreur_type: et,
                        net: regret > 2.0 * et && regret > 0.0,
                    });
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
    tous.sort_by_key(|m| (m.graine, m.numero, m.deck));
    let secs = debut.elapsed().as_secs_f64();

    let mut sortie = String::new();
    let _ = writeln!(
        sortie,
        "\nDurée : {secs:.0} s, {} positions, erreurs : {}\n",
        tous.len(),
        erreurs.load(Ordering::Relaxed)
    );
    let entete = "| | positions | regret moyen (points de %) | écart net | bot = meilleure option |\n|---|---|---|---|---|";
    let mut total = Bilan::default();
    let mut par_type: BTreeMap<String, Bilan> = BTreeMap::new();
    let mut par_deck: Vec<Bilan> = (0..decks.len()).map(|_| Bilan::default()).collect();
    for m in &tous {
        total.ajouter(m);
        par_type
            .entry(m.type_decision.clone())
            .or_default()
            .ajouter(m);
        par_deck[m.deck].ajouter(m);
    }
    let _ = writeln!(sortie, "## Par type de décision\n\n{entete}");
    let _ = writeln!(sortie, "{}", total.ligne("**toutes**"));
    let mut types: Vec<(&String, &Bilan)> = par_type.iter().collect();
    types.sort_by(|a, b| b.1.regret.total_cmp(&a.1.regret));
    for (t, b) in types {
        let _ = writeln!(sortie, "{}", b.ligne(t));
    }
    let _ = writeln!(sortie, "\n## Par deck (du joueur qui décide)\n\n{entete}");
    for (k, b) in par_deck.iter().enumerate() {
        let _ = writeln!(sortie, "{}", b.ligne(&decks[k].nom));
    }
    let mut pires: Vec<&Mesure> = tous.iter().filter(|m| m.net).collect();
    pires.sort_by(|a, b| b.regret.total_cmp(&a.regret));
    let _ = writeln!(sortie, "\n## Plus gros écarts nets (au plus 20)\n");
    for m in pires.iter().take(20) {
        let _ = writeln!(
            sortie,
            "- graine {} décision {} tour {} : {} contre {}, {} ({} options) : bot « {} », référence « {} », regret {:.1} points (± {:.1})",
            m.graine,
            m.numero,
            m.tour,
            decks[m.deck].nom,
            decks[m.adverse].nom,
            m.type_decision,
            m.options,
            m.choix_bot,
            m.meilleur,
            100.0 * m.regret,
            200.0 * m.erreur_type
        );
    }
    print!("{sortie}");

    if let Some(chemin) = o.texte("--details") {
        let mut f = std::fs::File::create(chemin).expect("fichier --details");
        let _ = writeln!(
            f,
            "graine\tdecision\ttour\tdeck\tadverse\ttype\toptions\tchoix_bot\tmeilleur\tregret\terreur_type\tnet"
        );
        for m in &tous {
            let _ = writeln!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{}",
                m.graine,
                m.numero,
                m.tour,
                decks[m.deck].nom,
                decks[m.adverse].nom,
                m.type_decision,
                m.options,
                m.choix_bot,
                m.meilleur,
                m.regret,
                m.erreur_type,
                m.net
            );
        }
        println!("\nDétails écrits dans {chemin}");
    }
    if erreurs.load(Ordering::Relaxed) > 0 {
        std::process::exit(1);
    }
}

/// Joue une partie et garde la banque de positions des deux joueurs.
fn banque(
    db: &Arc<fab_engine::CardDb>,
    table: &Arc<fab_ai::side::TableSide>,
    pools: [&fab_engine::CardPool; 2],
    bot: &str,
    graine: u64,
    une_sur: u64,
) -> Result<Vec<Position>, String> {
    let chooser = PlayerId((graine % 2) as u8);
    let mut s0 = creer_side("appris", db, Some(table.clone()), graine ^ 0x51DE).expect("side");
    let mut s1 = creer_side("appris", db, Some(table.clone()), graine ^ 0xE51D).expect("side");
    let taille = FormatConfig::silver_age().deck_size as usize;
    let m = preparer_manche(db, pools, [&mut *s0, &mut *s1], chooser, taille)
        .map_err(|(p, e)| format!("choix du joueur {} : {e:?}", p.0 + 1))?;
    let mut g = lancer_manche(db.clone(), &m, graine, 200).map_err(|e| format!("{e:?}"))?;
    let nouveau = |k: u64| Banque {
        inner: creer_agent(bot, graine ^ k).expect("bot"),
        une_sur,
        rng: Rng::new(graine ^ k ^ 0xBA4C),
        compteur: 0,
        positions: Vec::new(),
    };
    let mut a = nouveau(0xA5A5);
    let mut b = nouveau(0x5A5A);
    fab_ai::play_out(&mut g, [&mut a, &mut b], false)?;
    let mut v = a.positions;
    v.append(&mut b.positions);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(v: &[f32]) -> Note {
        Note {
            cle: String::new(),
            index: 0,
            par_monde: v.iter().map(|&x| Some(x)).collect(),
        }
    }

    #[test]
    fn la_meilleure_option_est_choisie_sur_les_mondes_pairs() {
        // Mondes pairs : 1, 1 ; impairs : 0, 0.
        let a = note(&[1.0, 0.0, 1.0, 0.0]);
        assert_eq!(a.moyenne(0), 1.0);
        assert_eq!(a.moyenne(1), 0.0);
    }

    #[test]
    fn l_ecart_apparie_ne_compte_que_les_mondes_impairs() {
        let a = note(&[0.0, 1.0, 0.0, 1.0, 0.0, 0.0]);
        let b = note(&[1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
        let (m, et) = ecart_apparie(&a, &b);
        // Mondes impairs : écarts 1, 1, 0.
        assert!((m - 2.0 / 3.0).abs() < 1e-9);
        assert!(et > 0.0);
    }

    #[test]
    fn l_ecart_apparie_ignore_les_mondes_ou_une_option_manque() {
        let mut a = note(&[0.0, 1.0, 0.0, 1.0]);
        a.par_monde[1] = None;
        let b = note(&[0.0, 0.0, 0.0, 0.0]);
        let (m, _) = ecart_apparie(&a, &b);
        assert_eq!(m, 1.0);
    }

    #[test]
    fn deux_exemplaires_identiques_sont_une_seule_option() {
        let db = Arc::new(real::build());
        let decks = lire_decks(&db, Some("ira,hala"));
        let table = table_side(&db);
        let positions =
            banque(&db, &table, [&decks[0].pool, &decks[1].pool], "b1", 5, 1).expect("partie");
        assert!(!positions.is_empty());
        for p in &positions {
            let d = p.g.pending.as_ref().unwrap();
            let opts = options_distinctes(&p.g, d);
            assert!(opts.len() >= 2);
            // Le choix du bot a toujours une clé parmi les options.
            let k = cle(&p.g, d.options[p.choix]);
            assert!(opts.iter().any(|(x, _)| *x == k));
            // Deux options de même clé : même carte, même zone.
            for (i, &c) in d.options.iter().enumerate() {
                let (_, j) = opts.iter().find(|(x, _)| *x == cle(&p.g, c)).unwrap();
                if let (Choice::Play(a), Choice::Play(b)) = (c, d.options[*j]) {
                    assert_eq!(p.g.obj(a).card, p.g.obj(b).card, "option {i}");
                    assert_eq!(p.g.obj(a).zone, p.g.obj(b).zone);
                }
            }
        }
    }

    #[test]
    fn la_reference_donne_une_note_par_monde_et_par_option() {
        let db = Arc::new(real::build());
        let decks = lire_decks(&db, Some("ira,hala"));
        let table = table_side(&db);
        let eval = Modele::par_defaut().indexer(&db);
        let positions =
            banque(&db, &table, [&decks[0].pool, &decks[1].pool], "b1", 5, 20).expect("partie");
        let r = Reference {
            mondes: 6,
            simulation: "b1".into(),
            tours: 0,
        };
        let p = &positions[0];
        let notes = noter(p, &r, 3, &eval);
        assert!(notes.len() >= 2);
        for n in &notes {
            assert_eq!(n.par_monde.len(), 6);
            for v in n.par_monde.iter().flatten() {
                assert!((0.0..=1.0).contains(v));
            }
        }
        // Même graine : mêmes notes (reproductible).
        let encore = noter(p, &r, 3, &eval);
        for (a, b) in notes.iter().zip(&encore) {
            assert_eq!(a.par_monde, b.par_monde);
        }
    }
}
