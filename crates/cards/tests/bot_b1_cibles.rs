//! B1 : choix de la cible d'une attaque (héros adverse ou allié adverse,
//! CR 1.4.5). Cas mesurés : les dragons de Dromai.

mod commun;
use commun::*;
use fab_ai::{Agent, HeuristicAgent};
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, ObjectId, Target};

const JACK: (&str, Option<Color>) = ("Jack Be Quick", R); // coût 0, 3{p}
const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);

/// P0 joue Jack Be Quick ; P1 contrôle l'allié `allie` : un token, ou le
/// dragon d'une invocation (« Invoke X », mise dans l'arène face dragon).
/// Renvoie la cible choisie par B1 et l'objet allié.
fn cible_choisie(allie: &str, vie_p1: Option<i32>) -> (Target, ObjectId) {
    let token = !allie.starts_with("Invoke");
    let main1: &[(&str, Option<Color>)] = if token { &[] } else { &[(allie, R)] };
    let mut t = partie_vraie(
        ["Dromai", "Dromai"],
        &[JACK, RAMPART, RAMPART, RAMPART],
        &[],
        main1,
        &[],
        &[],
        &[],
    );
    let a = if token {
        let c = t.g.db.find_name(allie).expect("token codé");
        t.g.create_tokens(P1, c, 1);
        t.g.last_created[0]
    } else {
        let c = t.en_main(P1, allie, R);
        t.g.transform_into_card(P1, &[], c);
        c
    };
    if let Some(v) = vie_p1 {
        t.g.players[P1.index()].life = v;
    }
    t.g.refresh_priority();
    let jack = t.en_main(P0, JACK.0, JACK.1);
    t.choisir(Choice::Play(jack));
    assert_eq!(t.kind(), DecisionKind::AttackTarget);
    let d = t.g.pending_decision().unwrap().clone();
    let i = HeuristicAgent::new().choose(&t.g, &d);
    match d.options[i] {
        Choice::Target(x) => (x, a),
        c => panic!("option inattendue {c:?}"),
    }
}

#[test]
fn b1_attaque_un_dragon_qu_il_peut_detruire_et_qui_frappe_fort() {
    // Kyloria : 4{p}, 2{h}. 3 dégâts le détruisent ; 2 × 4 > 3.
    let (cible, kyloria) = cible_choisie("Invoke Kyloria", None);
    assert!(matches!(cible, Target::Object(r) if r.id == kyloria));
}

#[test]
fn b1_vise_le_heros_si_le_dragon_frappe_peu() {
    // Aether Ashwing : 1{p}, 1{h} ; 2 × 1 < 3 dégâts au héros.
    let (cible, _) = cible_choisie("Aether Ashwing", None);
    assert_eq!(cible, Target::Player(P1));
}

#[test]
fn b1_vise_le_heros_si_l_attaque_ne_peut_pas_detruire_le_dragon() {
    // Nekria : 4{p}, 7{h} ; 3 dégâts ne suffisent pas.
    let (cible, _) = cible_choisie("Invoke Nekria", None);
    assert_eq!(cible, Target::Player(P1));
}

#[test]
fn b1_vise_le_heros_si_l_attaque_peut_le_tuer() {
    let (cible, _) = cible_choisie("Invoke Kyloria", Some(3));
    assert_eq!(cible, Target::Player(P1));
}
