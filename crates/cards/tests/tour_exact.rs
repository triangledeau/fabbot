//! Tests du solveur exact d'un tour (`fab_ai::tour_exact`, épreuve A).

mod commun;
use commun::*;
use fab_ai::tour_exact::{jouer_tour, resoudre, Defense};
use fab_ai::{Agent, HeuristicAgent};
use fab_cards::{decks, real};
use fab_engine::{Choice, Decision, DecisionKind, GameState};
use fab_match::{lancer_manche, preparer_manche, SideListe};
use std::sync::Arc;

/// Main de départ de P0 : une attaque sans go again (4), une attaque avec
/// go again (3), deux cartes bleues pour payer.
fn partie_ordre() -> Partie {
    partie(
        &[
            ("Essai Frappe", R),
            ("Essai Assaut", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    )
}

/// Un joueur qui joue toujours l'Essai Frappe d'abord s'il le peut, puis la
/// première option.
struct FrappeDAbord;

impl Agent for FrappeDAbord {
    fn name(&self) -> &str {
        "frappe d'abord"
    }
    fn choose(&mut self, g: &GameState, d: &Decision) -> usize {
        d.options
            .iter()
            .position(|c| matches!(c, Choice::Play(o) if g.def(*o).name == "Essai Frappe"))
            .unwrap_or(0)
    }
}

#[test]
fn le_solveur_joue_l_attaque_avec_go_again_en_premier() {
    let t = partie_ordre();
    assert_eq!(t.kind(), DecisionKind::Priority);
    let sol = resoudre(&t.g, Defense::Passive, 100_000);
    assert!(sol.complet);
    assert!(!sol.meilleur.letal);
    // Assaut (3, go again) puis Frappe (4) payée par un Rempart.
    assert_eq!(sol.meilleur.degats, 7);
    let assaut = t.en_main(P0, "Essai Assaut", R);
    let d = t.g.pending_decision().unwrap();
    assert_eq!(d.options[sol.meilleur.ligne[0]], Choice::Play(assaut));
}

#[test]
fn jouer_la_frappe_d_abord_fait_moins_de_degats() {
    let t = partie_ordre();
    let r = jouer_tour(&t.g, &mut FrappeDAbord, Defense::Passive);
    // La Frappe n'a pas go again : le tour s'arrête après elle.
    assert_eq!(r.degats, 4);
    assert!(!r.letal);
}

#[test]
fn le_solveur_fait_au_moins_aussi_bien_que_b1() {
    let t = partie_ordre();
    let sol = resoudre(&t.g, Defense::Passive, 100_000);
    let r = jouer_tour(&t.g, &mut HeuristicAgent::new(), Defense::Passive);
    assert!(sol.meilleur.degats >= r.degats);
}

#[test]
fn le_letal_est_trouve_quand_l_adversaire_a_peu_de_vie() {
    let mut t = partie_ordre();
    t.g.players[1].life = 7;
    let sol = resoudre(&t.g, Defense::Passive, 100_000);
    assert!(sol.complet);
    assert!(sol.meilleur.letal);
    assert_eq!(sol.meilleur.degats, 7);
    // Avec 8 points de vie, pas de létal.
    let mut t = partie_ordre();
    t.g.players[1].life = 8;
    assert!(!resoudre(&t.g, Defense::Passive, 100_000).meilleur.letal);
}

#[test]
fn la_defense_passive_ne_bloque_pas_et_la_defense_b1_peut_bloquer() {
    let t = partie_ordre();
    let passive = resoudre(&t.g, Defense::Passive, 100_000);
    let b1 = resoudre(&t.g, Defense::B1, 100_000);
    assert!(b1.complet);
    // P1 a quatre Rempart (défense 4) : contre B1, il en reste moins.
    assert!(b1.meilleur.degats <= passive.meilleur.degats);
    // La défense passive répond « Pass » à la déclaration de défense.
    let mut b1_agent = HeuristicAgent::new();
    let d = Decision {
        player: P1,
        kind: DecisionKind::Defend,
        options: vec![
            Choice::Defend(t.en_main(P1, "Essai Rempart", B)),
            Choice::Pass,
        ],
    };
    assert_eq!(Defense::Passive.choisir(&mut b1_agent, &t.g, &d), 1);
}

#[test]
fn la_recherche_s_arrete_au_budget() {
    let t = partie_ordre();
    let sol = resoudre(&t.g, Defense::Passive, 1);
    assert!(!sol.complet);
    assert_eq!(sol.noeuds, 1);
}

#[test]
fn le_solveur_ne_change_pas_la_partie() {
    let t = partie_ordre();
    let avant = format!("{:?}", t.g.pending_decision());
    let vie = t.vie(P1);
    let _ = resoudre(&t.g, Defense::B1, 100_000);
    let _ = jouer_tour(&t.g, &mut HeuristicAgent::new(), Defense::B1);
    assert_eq!(format!("{:?}", t.g.pending_decision()), avant);
    assert_eq!(t.vie(P1), vie);
}

/// Sur de vraies positions (vrais decks, parties B1 contre B1), le solveur
/// complet fait toujours au moins aussi bien que B1 joué dans le même monde.
#[test]
fn sur_de_vraies_positions_le_solveur_complet_bat_ou_egale_b1() {
    let db = Arc::new(real::build());
    let pool = |nom: &str| real::card_pool(&db, &decks::liste(nom).unwrap()).unwrap();
    let (ira, hala) = (pool("ira"), pool("hala"));
    let taille = fab_engine::FormatConfig::silver_age().deck_size as usize;
    let (mut a, mut b) = (SideListe, SideListe);
    let m = preparer_manche(&db, [&ira, &hala], [&mut a, &mut b], P0, taille).unwrap();
    let mut g = lancer_manche(db.clone(), &m, 3, 200).unwrap();
    let mut b1 = HeuristicAgent::new();
    let mut verifiees = 0;
    let mut dernier_tour = u32::MAX;
    while let Some(d) = g.pending_decision().cloned() {
        if d.kind == DecisionKind::Priority
            && d.player == g.turn_player
            && g.action_phase
            && g.stack.is_empty()
            && !g.combat.open
            && g.turn != dernier_tour
        {
            dernier_tour = g.turn;
            for defense in [Defense::Passive, Defense::B1] {
                let sol = resoudre(&g, defense, 50_000);
                let r = jouer_tour(&g, &mut HeuristicAgent::new(), defense);
                if sol.complet {
                    verifiees += 1;
                    assert!(
                        (sol.meilleur.letal, sol.meilleur.degats) >= (r.letal, r.degats),
                        "tour {} ({defense:?}) : solveur {:?}, B1 {:?}",
                        g.turn,
                        (sol.meilleur.letal, sol.meilleur.degats),
                        (r.letal, r.degats)
                    );
                }
            }
        }
        let i = b1.choose(&g, &d);
        g.apply(i).unwrap();
    }
    assert!(verifiees >= 5, "seulement {verifiees} positions vérifiées");
}
