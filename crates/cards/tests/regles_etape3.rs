//! Règles ajoutées à l'étape 3 (deck Ira) : jouer depuis la zone de
//! bannissement, cartes créées avec ephemeral, ambush, capacités accordées,
//! modes dont la cible dépend du mode. Cartes **fictives** (`fab_cards::test_set`).

mod commun;
use commun::*;
use fab_engine::state::CombatStep;
use fab_engine::{Choice, DecisionKind, ObjectId, Zone};

const RAMPART: (&str, Option<fab_engine::card::Color>) = ("Essai Rempart", B);

fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..200 {
        if t.g.is_over() || cond(t) {
            return;
        }
        match t.kind() {
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            k => panic!("décision {k:?} inattendue"),
        }
    }
    panic!("condition jamais atteinte");
}

/// P0 joue Essai Appel : un Essai Fauve est créé dans sa zone de bannissement.
fn appel(t: &mut Partie) -> ObjectId {
    let a = t.en_main(P0, "Essai Appel", B);
    t.choisir(Choice::Play(a));
    passer_jusqua_que(t, |t| t.g.stack.is_empty());
    *t.g.player(P0).banished.last().expect("carte créée")
}

#[test]
fn cr_1_0_2_permission_de_jouer_une_carte_creee_depuis_le_bannissement() {
    let mut t = partie(
        &[("Essai Appel", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f = appel(&mut t);
    assert_eq!(t.g.obj(f).zone, Zone::Banished);
    assert_eq!(t.g.def(f).name, "Essai Fauve");
    assert!(t.peut(Choice::Play(f)));
    t.choisir(Choice::Play(f));
    // Rien d'autre à jouer : le moteur avance seul jusqu'à la chaîne.
    assert_eq!(t.g.obj(f).zone, Zone::CombatChain);
}

#[test]
fn cr_1_0_2_la_permission_ne_dure_que_ce_tour() {
    let mut t = partie(
        &[("Essai Appel", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f = appel(&mut t);
    t.finir_le_tour();
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P0);
    assert!(!t.peut(Choice::Play(f)));
    assert!(t.g.play_permissions.is_empty(), "permission expirée");
}

#[test]
fn cr_8_3_21_ephemeral_retire_de_la_partie_au_lieu_d_aller_au_cimetiere() {
    let mut t = partie(
        &[("Essai Appel", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f = appel(&mut t);
    t.choisir(Choice::Play(f));
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.obj(f).zone, Zone::Gone);
    assert!(!t.g.player(P0).graveyard.contains(&f));
}

#[test]
fn cr_8_3_28_ambush_defendre_depuis_l_arsenal() {
    let mut t = partie(
        &[("Essai Embuscade", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let e = t.en_main(P0, "Essai Embuscade", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Arsenal);
    t.choisir(Choice::Arsenal(e));
    t.finir_le_tour();
    let a = t.en_main(P1, "Essai Assaut", R);
    t.choisir(Choice::Play(a));
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    assert!(t.peut(Choice::Defend(e)));
    t.g.record = Some(Vec::new());
    t.choisir(Choice::Defend(e));
    t.choisir(Choice::Pass);
    // Pas une carte venue de la main (dominate, reprise).
    let defend = t.g.record.as_ref().unwrap().iter().find_map(|ev| match ev {
        fab_engine::Event::Defend {
            card, from_hand, ..
        } if *card == e => Some(*from_hand),
        _ => None,
    });
    assert_eq!(defend, Some(false));
    assert_eq!(t.vie(P0), 20, "défense 3 contre 3");
}

#[test]
fn cr_6_capacite_declenchee_accordee_a_une_attaque() {
    let mut t = partie(
        &[("Essai Frappe", R), ("Essai Héritage", B), RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    let r = t.en_main(P0, "Essai Rempart", B);
    t.choisir(Choice::Pitch(r));
    passer_jusqua_que(&mut t, |t| t.g.combat.step == Some(CombatStep::Reaction));
    let h = t.en_main(P0, "Essai Héritage", B);
    t.choisir(Choice::Play(h));
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 16);
    assert_eq!(t.vie(P0), 21, "« When this hits, gain 1{{h}} »");
}

#[test]
fn cr_6_capacite_accordee_finit_avec_sa_duree() {
    let mut t = partie(
        &[("Essai Frappe", R), ("Essai Héritage", B), RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    let r = t.en_main(P0, "Essai Rempart", B);
    t.choisir(Choice::Pitch(r));
    passer_jusqua_que(&mut t, |t| t.g.combat.step == Some(CombatStep::Reaction));
    let h = t.en_main(P0, "Essai Héritage", B);
    t.choisir(Choice::Play(h));
    passer_jusqua_que(&mut t, |t| {
        !t.g.stack.is_empty() || t.g.granted_triggers.len() == 1
    });
    t.finir_le_tour();
    assert!(t.g.granted_triggers.is_empty());
}

#[test]
fn cr_1_7_5_seuls_les_modes_qui_ont_une_cible_sont_proposes() {
    let mut t = partie(
        &[
            ("Essai Assaut", R),
            ("Essai Alternative", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Essai Dague"],
        &[],
    );
    let a = t.en_main(P0, "Essai Assaut", R);
    t.choisir(Choice::Play(a));
    passer_jusqua_que(&mut t, |t| t.g.combat.step == Some(CombatStep::Reaction));
    let alt = t.en_main(P0, "Essai Alternative", R);
    assert!(t.peut(Choice::Play(alt)));
    t.choisir(Choice::Play(alt));
    assert_eq!(t.kind(), DecisionKind::Mode);
    assert_eq!(t.options(), vec![Choice::Mode(1)], "pas d'attaque d'arme");
    t.choisir(Choice::Mode(1));
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.power(a), 4, "3 + 1");
}

#[test]
fn cr_1_7_5_carte_modale_injouable_si_aucun_mode_n_a_de_cible() {
    let mut t = partie(
        &[("Essai Alternative", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let alt = t.en_main(P0, "Essai Alternative", R);
    assert!(!t.peut(Choice::Play(alt)), "aucune attaque");
    t.finir_le_tour();
}
