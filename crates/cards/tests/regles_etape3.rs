//! Règles ajoutées à l'étape 3 (deck Ira) : jouer depuis la zone de
//! bannissement, cartes créées avec ephemeral, ambush, capacités accordées,
//! modes dont la cible dépend du mode ; pour le deck Bravo : clash, réductions
//! de coût, « loses and can't gain ». Cartes **fictives** (`fab_cards::test_set`).

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

// ---------------------------------------------------------------------------
// Ajouts du deck Bravo : clash, réductions de coût, « loses and can't gain »
// ---------------------------------------------------------------------------

/// Partie où le dessus du deck de P0 est `haut0` et celui de P1 `haut1`
/// (`None` : un Essai Rempart, sans puissance).
fn clash_entre(
    haut0: Option<(&str, Option<fab_engine::card::Color>)>,
    haut1: Option<(&str, Option<fab_engine::card::Color>)>,
) -> Partie {
    let s0: Vec<_> = haut0.into_iter().collect();
    let s1: Vec<_> = haut1.into_iter().collect();
    partie(&[RAMPART; 4], &s0, &[RAMPART; 4], &s1, &[], &[])
}

#[test]
fn cr_8_5_45_le_clash_est_gagne_par_la_plus_grande_puissance() {
    let mut t = clash_entre(Some(("Essai Frappe", R)), Some(("Essai Frappe", B)));
    let (h0, h1) = (t.g.top_of_deck(P0).unwrap(), t.g.top_of_deck(P1).unwrap());
    assert_eq!(t.g.clash(P0, P1), Some(P0), "4 contre 2");
    assert_eq!(t.g.clash(P1, P0), Some(P0), "l'ordre ne compte pas");
    // CR 8.5.17c : révéler ne déplace pas les cartes.
    assert_eq!(t.g.top_of_deck(P0), Some(h0));
    assert_eq!(t.g.top_of_deck(P1), Some(h1));
    // CR 8.5.45a : le gagnant gagne « en révélant cette carte ».
    assert!(t.g.turn_events().iter().any(|e| matches!(
        e,
        fab_engine::Event::ClashWon { winner, card } if *winner == P0 && *card == h0
    )));
}

#[test]
fn cr_8_5_45b_une_carte_sans_puissance_perd_contre_une_carte_qui_en_a() {
    let mut t = clash_entre(None, Some(("Essai Frappe", B)));
    assert_eq!(
        t.g.clash(P0, P1),
        Some(P1),
        "Rempart (sans puissance) contre 2"
    );
}

#[test]
fn cr_8_5_45b_un_deck_vide_ne_revele_rien_et_perd() {
    let mut t = clash_entre(None, Some(("Essai Frappe", B)));
    t.g.player_mut(P1).deck.clear();
    t.g.player_mut(P0).deck.clear();
    let haut = t.en_main(P0, "Essai Rempart", B);
    // P0 n'a plus qu'une carte sans puissance, P1 rien : personne ne gagne.
    t.g.put_on_top(haut);
    assert_eq!(t.g.clash(P0, P1), None);
}

#[test]
fn cr_8_5_45c_a_puissance_egale_personne_ne_gagne() {
    let mut t = clash_entre(Some(("Essai Frappe", R)), Some(("Essai Frappe", R)));
    assert_eq!(t.g.clash(P0, P1), None);
    let mut t = clash_entre(None, None);
    assert_eq!(t.g.clash(P0, P1), None, "aucune puissance des deux côtés");
}

#[test]
fn cr_8_5_45d_il_faut_deux_heros_differents() {
    let mut t = clash_entre(Some(("Essai Frappe", R)), None);
    assert_eq!(t.g.clash(P0, P0), None);
}

#[test]
fn cr_5_1_6_reduction_de_la_prochaine_carte_qui_verifie_le_filtre() {
    let mut t = partie(
        &[
            ("Essai Écrasement", R),
            ("Essai Frappe", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    // « Your next [carte avec dominate] this turn costs {r} less », deux fois.
    let filtre = |g: &fab_engine::GameState, c: ObjectId| {
        g.def(c).keywords.has(fab_engine::Keywords::DOMINATE)
    };
    t.g.next_card_costs_less(P0, filtre, 1);
    t.g.next_card_costs_less(P0, filtre, 1);
    let e = t.en_main(P0, "Essai Écrasement", R);
    let f = t.en_main(P0, "Essai Frappe", R);
    assert_eq!(t.g.play_cost(P0, e), 0, "2 - 1 - 1");
    assert_eq!(t.g.play_cost(P0, f), 1, "hors filtre");
    assert_eq!(t.g.play_cost(P1, e), 2, "réductions de P0 seulement");
    t.g.refresh_priority();
    t.choisir(Choice::Play(e));
    assert!(
        t.g.cost_reductions.is_empty(),
        "les deux réductions sont utilisées"
    );
}

#[test]
fn cr_8_3_4_perdre_et_ne_plus_pouvoir_gagner_dominate() {
    let mut t = partie(
        &[("Essai Écrasement", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let e = t.en_main(P0, "Essai Écrasement", R);
    let r = t.g.obj_ref(e);
    let perd = fab_engine::Delta {
        loses: fab_engine::Keywords::DOMINATE,
        ..Default::default()
    };
    t.g.modify(r, perd, fab_engine::Until::EndOfTurn);
    assert!(!t.g.keywords(e).has(fab_engine::Keywords::DOMINATE));
    let gagne = fab_engine::Delta {
        grants: fab_engine::Keywords::DOMINATE | fab_engine::Keywords::GO_AGAIN,
        ..Default::default()
    };
    t.g.modify(r, gagne, fab_engine::Until::EndOfTurn);
    let k = t.g.keywords(e);
    assert!(!k.has(fab_engine::Keywords::DOMINATE));
    assert!(
        k.has(fab_engine::Keywords::GO_AGAIN),
        "les autres mots-clés restent"
    );
}
