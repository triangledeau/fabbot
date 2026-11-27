//! Le niveau réfléchi (`fab_ai::reflechi`) sur une vraie partie : temps
//! prévu respecté, plusieurs fils, avancement suivi, « Joue maintenant ».

use fab_ai::reflechi::{ReflechiAgent, ReglagesReflechi};
use fab_ai::{creer_agent, Agent, HeuristicAgent};
use fab_cards::decklist::Decklist;
use fab_cards::real;
use fab_engine::{DecisionKind, GameSetup, GameState, PlayerId};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Une vraie partie (Hala contre Dorinthea… les deux premiers decks),
/// avancée par B1 jusqu'à une décision de priorité de P0 avec au moins
/// deux options distinctes.
fn position() -> GameState {
    let db = Arc::new(real::build());
    let decks: Vec<_> = fab_cards::decks::tous()
        .into_iter()
        .take(2)
        .map(|d| {
            let list = Decklist::parse(&std::fs::read_to_string(d.fichier).unwrap()).unwrap();
            real::default_loadout(&db, &list).unwrap()
        })
        .collect();
    let setup = GameSetup {
        players: [decks[0].clone(), decks[1].clone()],
        chooser: PlayerId::P0,
        seed: 3,
        shuffle: true,
        max_turns: 200,
    };
    let mut g = GameState::new(db, setup).unwrap();
    let mut b1 = HeuristicAgent::new();
    while let Some(d) = g.pending_decision().cloned() {
        if d.player == PlayerId::P0
            && d.kind == DecisionKind::Priority
            && fab_ai::b2::cles_des_options(&g, &d).len() >= 3
        {
            return g;
        }
        let i = b1.choose(&g, &d);
        g.apply(i).unwrap();
    }
    panic!("pas de décision de priorité à plusieurs options");
}

fn reglages(max_ms: u64) -> ReglagesReflechi {
    ReglagesReflechi {
        fils: 2,
        duree_max: Duration::from_millis(max_ms),
        duree_min: Duration::from_millis(100),
        par_option: Duration::from_secs(10),
        arret_anticipe: false,
        ..ReglagesReflechi::default()
    }
}

#[test]
fn reflechi_respecte_le_temps_prevu_et_suit_son_avancement() {
    let g = position();
    let d = g.pending_decision().unwrap().clone();
    let mut bot = ReflechiAgent::new(1, reglages(600));
    let suivi = bot.suivi();
    let debut = Instant::now();
    let i = bot.choose(&g, &d);
    let duree = debut.elapsed();
    assert!(i < d.options.len());
    assert!(duree >= Duration::from_millis(600), "{duree:?}");
    assert!(duree < Duration::from_millis(1500), "{duree:?}");
    let p = suivi.progression();
    assert!(!p.en_cours);
    assert_eq!(p.numero, 1);
    assert_eq!(p.fils, 2);
    assert_eq!(p.prevu, Duration::from_millis(600));
    assert!(p.iterations >= 32, "{}", p.iterations);
    assert!(p.options.len() >= 3);
    // Les options sont rangées de la plus visitée à la moins visitée, et
    // l'option jouée est la première.
    assert!(p.options.windows(2).all(|w| w[0].visites >= w[1].visites));
    assert_eq!(p.options[0].index, i);
    assert!(p.options.iter().all(|o| (0.0..=1.0).contains(&o.chances)));
}

#[test]
fn joue_maintenant_arrete_la_reflexion() {
    let g = position();
    let d = g.pending_decision().unwrap().clone();
    let mut bot = ReflechiAgent::new(2, reglages(60_000));
    let suivi = bot.suivi();
    let debut = Instant::now();
    let i = std::thread::scope(|s| {
        let h = s.spawn(|| bot.choose(&g, &d));
        // On attend que la réflexion ait commencé.
        while !suivi.progression().en_cours {
            std::thread::yield_now();
        }
        std::thread::sleep(Duration::from_millis(300));
        assert!(suivi.progression().en_cours);
        suivi.jouer_maintenant();
        h.join().unwrap()
    });
    assert!(i < d.options.len());
    assert!(debut.elapsed() < Duration::from_secs(5));
    assert!(!suivi.progression().en_cours);
}

#[test]
fn une_seule_option_distincte_ne_fait_pas_reflechir() {
    let g = position();
    let mut d = g.pending_decision().unwrap().clone();
    d.options.truncate(1);
    let mut bot = ReflechiAgent::new(3, reglages(60_000));
    let debut = Instant::now();
    assert_eq!(bot.choose(&g, &d), 0);
    assert!(debut.elapsed() < Duration::from_millis(50));
    assert_eq!(bot.suivi().progression().numero, 0);
}

#[test]
fn le_niveau_reflechi_se_cree_par_son_nom_et_a_un_suivi() {
    assert!(creer_agent("reflechi", 1).unwrap().suivi().is_some());
    assert!(creer_agent("reflechi-30s", 1).unwrap().suivi().is_some());
    assert!(creer_agent("b3", 1).unwrap().suivi().is_none());
}
