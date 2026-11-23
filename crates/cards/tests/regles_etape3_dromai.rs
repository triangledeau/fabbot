//! Tests des règles ajoutées pour le deck Dromai : cartes sous un permanent
//! (CR 3.0.14), transform (CR 8.5.36), invocations et cartes retournables
//! (CR 8.2.13, 9.1.3), phantasm (CR 8.3.13), fermeture de la chaîne quand
//! l'attaque disparaît (CR 7.7.2c), alliés qui attaquent (CR 1.4.3c, 8.2.8).

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Event, Keywords, ObjectId, PlayerId, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);
const STORM: &str = "Storm of Sandikai";

fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..300 {
        if t.g.is_over() || cond(t) {
            return;
        }
        match t.kind() {
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            k => panic!(
                "décision {k:?} inattendue ; options {:?}",
                t.noms(&t.options())
            ),
        }
    }
    panic!("condition jamais atteinte");
}

fn payer(t: &mut Partie) {
    while !t.g.is_over() && t.g.pending_decision().is_some() && t.kind() == DecisionKind::Pitch {
        let opts = t.options();
        let c = opts
            .iter()
            .copied()
            .find(|c| matches!(c, Choice::Pitch(o) if t.g.def(*o).name == "Essai Rempart"))
            .unwrap_or(opts[0]);
        t.choisir(c);
    }
}

fn creer(t: &mut Partie, p: PlayerId, nom: &str) -> ObjectId {
    let c = t.g.db.find_name(nom).expect("token codé");
    t.g.create_tokens(p, c, 1);
    let o = t.g.last_created[0];
    t.g.refresh_priority();
    o
}

/// Une Aether Ashwing de `p` avec une Ash dessous (phantasm).
fn ashwing_avec_ash(t: &mut Partie, p: PlayerId) -> (ObjectId, ObjectId) {
    let ash = creer(t, p, "Ash");
    let wc = t.g.db.find_name("Aether Ashwing").unwrap();
    let wing = t.g.transform_into_token(p, &[ash], wc).unwrap();
    t.g.refresh_priority();
    (wing, ash)
}

/// P0 (Dromai, Storm of Sandikai) attaque avec ce dragon ; renvoie la proxy.
fn attaquer_dragon(t: &mut Partie, dragon: ObjectId) -> ObjectId {
    passer_jusqua_que(t, |t| {
        t.joueur() == P0 && t.peut(Choice::Activate(dragon, 0))
    });
    t.choisir(Choice::Activate(dragon, 0));
    let i =
        t.g.objects
            .iter()
            .rposition(|o| o.proxy_of == Some(dragon))
            .unwrap();
    let proxy = ObjectId(i as u32);
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(proxy));
    proxy
}

fn defendre_avec(t: &mut Partie, p: PlayerId, cartes: &[ObjectId]) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Defend && t.joueur() == p);
    for &x in cartes {
        t.choisir(Choice::Defend(x));
    }
    if t.kind() == DecisionKind::Defend {
        t.choisir(Choice::Pass);
    }
}

fn dromai(main0: &[(&str, Option<Color>)], main1: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        ["Dromai", "Essai Héros"],
        main0,
        RIEN,
        main1,
        RIEN,
        &[STORM],
        &[],
    )
}

fn evenement(t: &Partie, f: impl Fn(&Event) -> bool) -> bool {
    t.g.turn_events().iter().any(f)
}

// ---------------------------------------------------------------------------
// CR 3.0.14 : cartes sous un permanent ; CR 8.5.36 : transform
// ---------------------------------------------------------------------------

#[test]
fn cr_8_5_36c_transform_en_token_cree_le_token_puis_met_l_objet_dessous() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let (wing, ash) = ashwing_avec_ash(&mut t, P0);
    assert_eq!(t.g.obj(wing).zone, Zone::Permanent);
    assert_eq!(t.g.obj(ash).zone, Zone::Under);
    assert_eq!(t.g.cards_under(wing).collect::<Vec<_>>(), vec![ash]);
    assert!(t.g.check_invariants().is_empty());
}

#[test]
fn cr_8_5_36d_transform_echoue_si_l_objet_n_est_plus_dans_l_arene() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash");
    t.g.destroy(ash);
    let wc = t.g.db.find_name("Aether Ashwing").unwrap();
    assert!(t.g.transform_into_token(P0, &[ash], wc).is_none());
    assert!(t
        .g
        .permanents
        .iter()
        .all(|&o| t.g.def(o).name != "Aether Ashwing"));
}

#[test]
fn cr_3_0_14b_une_carte_sous_un_permanent_n_est_pas_dans_l_arene() {
    let mut t = dromai(&[("Sand Cover", R), RAMPART, RAMPART, RAMPART], RIEN);
    let (_, ash) = ashwing_avec_ash(&mut t, P0);
    assert!(!t.g.obj(ash).zone.in_arena());
    // Ce n'est plus « une ash que vous contrôlez » : Sand Cover n'a pas de cible.
    let c = t.en_main(P0, "Sand Cover", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn cr_3_0_14e_les_cartes_dessous_sont_clear_quand_le_permanent_disparait() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let (wing, ash) = ashwing_avec_ash(&mut t, P0);
    t.g.destroy(wing);
    assert_eq!(t.g.obj(wing).zone, Zone::Gone);
    assert_eq!(t.g.obj(ash).zone, Zone::Gone);
    assert!(t.g.sub_cards.is_empty());
    assert!(t.g.check_invariants().is_empty());
}

#[test]
fn cr_8_4_5_material_l_ash_dessous_donne_phantasm_au_permanent() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let (wing, _) = ashwing_avec_ash(&mut t, P0);
    assert!(t.g.keywords(wing).has(Keywords::PHANTASM));
}

// ---------------------------------------------------------------------------
// CR 8.2.13 : invocations ; CR 9.1.3 : cartes retournables
// ---------------------------------------------------------------------------

#[test]
fn cr_8_2_13a_une_invocation_entre_dans_l_arene_face_arriere_active() {
    let mut t = dromai(&[("Invoke Yendurai", R), ETINCELLE, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash");
    let c = t.en_main(P0, "Invoke Yendurai", R);
    assert_eq!(t.g.def(c).name, "Invoke Yendurai");
    t.choisir(Choice::Play(c));
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.def(c).name, "Yendurai");
    assert_eq!(t.g.obj(c).zone, Zone::Permanent);
    // « Enters the arena with an endurance counter » : la face arrière.
    assert_eq!(t.g.obj(c).counters, 1);
    // CR 8.5.36b : la carte est devenue un permanent, l'ash est dessous.
    assert_eq!(t.g.cards_under(c).collect::<Vec<_>>(), vec![ash]);
}

#[test]
fn cr_8_5_36b_l_ash_de_l_invocation_est_deja_dessous_quand_ouvia_entre() {
    let mut t = dromai(&[("Invoke Ouvia", R), ETINCELLE, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash");
    let c = t.en_main(P0, "Invoke Ouvia", R);
    t.choisir(Choice::Play(c));
    payer(&mut t);
    // Ouvia entre : sa capacité n'a aucune ash à transformer (pas de question).
    passer_jusqua_que(&mut t, |t| {
        t.g.stack.is_empty() && t.g.pending_triggers.is_empty()
    });
    assert_eq!(t.g.obj(ash).zone, Zone::Under);
    assert!(evenement(
        &t,
        |e| matches!(e, Event::EnterArena { card } if *card == c)
    ));
}

#[test]
fn cr_9_1_3b_une_carte_retournable_hors_de_l_arene_a_sa_face_avant() {
    let mut t = dromai(&[("Invoke Themai", R), ETINCELLE, RAMPART, RAMPART], RIEN);
    creer(&mut t, P0, "Ash");
    let c = t.en_main(P0, "Invoke Themai", R);
    t.choisir(Choice::Play(c));
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.def(c).name, "Themai");
    t.g.return_to_hand(c);
    assert_eq!(t.g.obj(c).zone, Zone::Hand);
    assert_eq!(t.g.def(c).name, "Invoke Themai");
    assert!(t.g.sub_cards.is_empty());
}

#[test]
fn cr_5_3_5a_une_invocation_devenue_permanent_donne_quand_meme_go_again() {
    let mut t = dromai(&[("Invoke Miragai", R), ETINCELLE, RAMPART, RAMPART], RIEN);
    creer(&mut t, P0, "Ash");
    let c = t.en_main(P0, "Invoke Miragai", R);
    t.choisir(Choice::Play(c));
    payer(&mut t);
    assert_eq!(t.g.player(P0).action_points, 0);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.player(P0).action_points, 1);
}

// ---------------------------------------------------------------------------
// CR 8.3.13 : phantasm ; CR 7.7.2c : l'attaque disparaît
// ---------------------------------------------------------------------------

#[test]
fn cr_8_3_13_phantasm_detruit_l_allie_defendu_par_une_attaque_de_6_et_la_chaine_se_ferme() {
    let mut t = dromai(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Essai Écrasement", R), RAMPART, RAMPART, RAMPART],
    );
    let (wing, ash) = ashwing_avec_ash(&mut t, P0);
    attaquer_dragon(&mut t, wing);
    let e = t.en_main(P1, "Essai Écrasement", R);
    defendre_avec(&mut t, P1, &[e]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(wing).zone, Zone::Gone);
    assert_eq!(t.g.obj(ash).zone, Zone::Gone);
    // CR 7.7.2c : détruite avant les dégâts, la chaîne se ferme sans résolution.
    assert!(evenement(&t, |e| matches!(e, Event::CombatChainClosed)));
    assert!(!evenement(&t, |e| matches!(
        e,
        Event::ChainLinkResolved { .. }
    )));
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn cr_8_3_13_phantasm_pas_declenche_par_une_attaque_de_5() {
    let mut t = dromai(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Essai Écrasement", Y), RAMPART, RAMPART, RAMPART],
    );
    let (wing, _) = ashwing_avec_ash(&mut t, P0);
    attaquer_dragon(&mut t, wing);
    let e = t.en_main(P1, "Essai Écrasement", Y);
    defendre_avec(&mut t, P1, &[e]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(wing).zone, Zone::Permanent);
}

#[test]
fn cr_8_3_13_phantasm_pas_declenche_par_une_carte_qui_n_est_pas_une_attaque() {
    let mut t = dromai(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
    );
    let (wing, _) = ashwing_avec_ash(&mut t, P0);
    attaquer_dragon(&mut t, wing);
    let r = t.en_main(P1, "Essai Rempart", B);
    defendre_avec(&mut t, P1, &[r]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(wing).zone, Zone::Permanent);
}

#[test]
fn cr_8_3_13_sans_phantasm_rien_ne_se_passe() {
    let mut t = dromai(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Essai Écrasement", R), RAMPART, RAMPART, RAMPART],
    );
    let wing = creer(&mut t, P0, "Aether Ashwing");
    attaquer_dragon(&mut t, wing);
    let e = t.en_main(P1, "Essai Écrasement", R);
    defendre_avec(&mut t, P1, &[e]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(wing).zone, Zone::Permanent);
}

#[test]
fn cr_1_4_3c_un_allie_detruit_pendant_son_attaque_l_attaque_cesse_d_exister() {
    let mut t = dromai(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
    );
    let wing = creer(&mut t, P0, "Aether Ashwing");
    attaquer_dragon(&mut t, wing);
    t.g.destroy(wing);
    t.g.refresh_priority();
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 20);
    assert!(!evenement(&t, |e| matches!(e, Event::Hit { .. })));
}

// ---------------------------------------------------------------------------
// CR 8.2.8 : alliés
// ---------------------------------------------------------------------------

#[test]
fn cr_8_2_8_un_allie_qui_attaque_peut_recevoir_des_degats_et_mourir() {
    let mut t = dromai(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
    );
    let wing = creer(&mut t, P0, "Aether Ashwing");
    attaquer_dragon(&mut t, wing);
    assert!(t.g.is_living_permanent(wing));
    t.g.deal_damage_to(None, P1, Target::Object(t.g.obj_ref(wing)), 1, false);
    t.g.refresh_priority();
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    // 1{h} : il meurt (CR 1.10.2b), l'attaque disparaît (CR 7.7.2c).
    assert_eq!(t.g.obj(wing).zone, Zone::Gone);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn cr_8_2_8_un_allie_qui_attaque_revient_dans_la_zone_permanente() {
    let mut t = dromai(&[RAMPART, RAMPART, RAMPART, RAMPART], RIEN);
    let wing = creer(&mut t, P0, "Aether Ashwing");
    attaquer_dragon(&mut t, wing);
    assert_eq!(t.g.obj(wing).zone, Zone::CombatChain);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(wing).zone, Zone::Permanent);
    assert_eq!(t.g.obj(wing).controller, P0);
    assert_eq!(t.vie(P1), 19);
}
