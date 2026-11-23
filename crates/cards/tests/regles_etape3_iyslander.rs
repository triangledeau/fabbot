//! Règles ajoutées à l'étape 3 pour le deck Iyslander : jouer une action
//! « as though it were an instant », ordre des modifications de coût, amp,
//! engager un permanent, tokens créés dans une zone d'équipement exposée,
//! prévention statique. Vraies cartes (Iyslander, Frostbite…) et cartes
//! d'essai.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, PreventRef, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);

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

fn iys(main0: &[(&str, Option<Color>)], arena0: &[&str]) -> Partie {
    partie_vraie(
        ["Iyslander", "Essai Héros"],
        main0,
        RIEN,
        &[RAMPART; 4],
        RIEN,
        arena0,
        &[],
    )
}

fn token(t: &Partie, nom: &str) -> fab_engine::CardId {
    t.g.db.find_name(nom).unwrap()
}

#[test]
fn cr_8_1_1d_action_jouee_comme_un_instant_sans_point_d_action_a_tout_moment() {
    let mut t = iys(
        &[
            ("Stir the Aetherwinds", B),
            ("Frosting", B),
            ETINCELLE,
            RAMPART,
        ],
        &[],
    );
    let stir = t.en_main(P0, "Stir the Aetherwinds", B);
    t.choisir(Choice::Play(stir));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Rempart", B)));
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.player(P0).action_points, 0);
    // Une Essai Étincelle sur la pile : l'action se joue quand même (pile non vide).
    let e = t.en_main(P0, "Essai Étincelle", R);
    t.choisir(Choice::Play(e));
    assert!(!t.g.stack.is_empty());
    let f = t.en_main(P0, "Frosting", B);
    assert!(t.peut(Choice::Play(f)));
    t.choisir(Choice::Play(f));
    t.choisir(Choice::Target(Target::Player(P1)));
    assert_eq!(
        t.g.player(P0).action_points,
        0,
        "pas de point d'action dépensé"
    );
}

#[test]
fn cr_5_1_6a_les_augmentations_de_cout_avant_les_reductions() {
    let mut t = iys(&[("Frosting", B), ETINCELLE, RAMPART, RAMPART], &[]);
    let f = t.en_main(P0, "Frosting", B);
    let fb = token(&t, "Frostbite");
    t.g.create_tokens(P0, fb, 1);
    t.g.next_card_costs_less(P0, |_, _| true, 1);
    // 0 + 1 (Frostbite) - 1 = 0. Dans l'autre ordre : max(0, 0 - 1) + 1 = 1.
    assert_eq!(t.g.play_cost(P0, f), 0);
}

#[test]
fn cr_8_5_47_amp_seulement_les_degats_arcaniques_du_joueur_une_fois() {
    let mut t = iys(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &[]);
    t.g.amp(P0, 2);
    t.g.deal_damage(None, P1, P0, 1, true);
    assert_eq!(t.vie(P0), 17, "dégâts de l'adversaire : pas d'amp");
    t.g.deal_damage(None, P0, P1, 1, false);
    assert_eq!(t.vie(P1), 19, "dégâts non arcaniques : pas d'amp");
    t.g.deal_damage(None, P0, P1, 1, true);
    assert_eq!(t.vie(P1), 16, "1 + 2");
    t.g.deal_damage(None, P0, P1, 1, true);
    assert_eq!(t.vie(P1), 15, "amp utilisé");
}

#[test]
fn cr_8_5_55a_engager_un_permanent_deja_engage_echoue() {
    let mut t = iys(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &[]);
    let hero = t.g.player(P0).hero;
    assert!(t.g.tap(hero));
    assert!(!t.g.tap(hero));
    assert!(t.g.obj(hero).tapped);
}

#[test]
fn cr_4_4_3d_le_heros_engage_est_degage_en_fin_de_tour() {
    let mut t = iys(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &[]);
    let hero = t.g.player(P0).hero;
    t.g.tap(hero);
    t.finir_le_tour();
    assert!(!t.g.obj(hero).tapped);
}

#[test]
fn cr_3_0_1a_zone_exposee_sans_rien_dedans_ni_equipe() {
    let t = iys(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &["Nullrune Hood"]);
    let z = t.g.exposed_zones();
    assert!(!z.contains(&(P0, Zone::Head)));
    assert!(z.contains(&(P0, Zone::Chest)));
    assert_eq!(z.len(), 3 + 4);
}

#[test]
fn cr_8_5_2c_token_cree_dans_la_zone_d_un_joueur_sous_son_controle() {
    let mut t = iys(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &[]);
    let fb = token(&t, "Frostbite");
    t.g.create_token_in_zone(P1, fb, Zone::Legs);
    let o = t.g.player(P1).legs.expect("token dans la zone de jambes");
    assert_eq!(t.g.obj(o).controller, P1);
    assert_eq!(t.g.obj(o).zone, Zone::Legs);
    assert!(!t.g.exposed_zones().contains(&(P1, Zone::Legs)));
    // Un token ne peut pas défendre comme un équipement.
    assert!(!t.g.def(o).types.has(fab_engine::Types::EQUIPMENT));
}

#[test]
fn cr_6_4_10_prevention_statique_obligatoire_et_arcane_barrier_au_choix_de_l_ordre() {
    let mut t = iys(
        &[("Pyroglyphic Protection", B), ETINCELLE, RAMPART, RAMPART],
        &["Nullrune Hood"],
    );
    let p = t.en_main(P0, "Pyroglyphic Protection", B);
    t.choisir(Choice::Play(p));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Rempart", B)));
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    let hood = t.equipement(P0, "Nullrune Hood");
    t.g.deal_damage(None, P1, P0, 3, true);
    t.g.refresh_priority();
    assert_eq!(t.kind(), DecisionKind::Prevention);
    let opts = t.options();
    assert!(opts.contains(&Choice::Prevent(PreventRef::Static(p))));
    assert!(opts.contains(&Choice::Prevent(PreventRef::ArcaneBarrier(hood))));
    assert!(
        !opts.contains(&Choice::Pass),
        "une prévention est obligatoire"
    );
    t.choisir(Choice::Prevent(PreventRef::Static(p)));
    // Reste 2 : l'arcane barrier est encore proposée (au choix).
    assert_eq!(
        t.options(),
        vec![
            Choice::Prevent(PreventRef::ArcaneBarrier(hood)),
            Choice::Pass
        ]
    );
    t.choisir(Choice::Pass);
    assert_eq!(t.vie(P0), 16);
}
