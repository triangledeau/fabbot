//! Tests de règles ajoutées pour le deck Arakni : become / return to the
//! brood, traits, marque et touche par un effet, équiper, retrieve, blocage.
//! Un test par article du CR.

mod commun;
use commun::*;
use fab_engine::card::{Color, Traits};
use fab_engine::{Choice, EndReason, Event, Outcome, PlayerId, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);
const ARAKNI: &str = "Arakni, Web of Deceit";
const HUNTSMAN: &str = "Mark of the Huntsman";

/// Arakni (P0) contre Essai Héros (P1) ; P0 garde une option jouable.
fn arakni(arena0: &[&str]) -> Partie {
    let mut t = partie_vraie(
        [ARAKNI, "Essai Héros"],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        arena0,
        &[],
    );
    t.g.player_mut(P0).resource_points = 9;
    t.g.refresh_priority();
    t
}

fn heros(t: &Partie, p: PlayerId) -> String {
    t.g.def(t.g.player(p).hero).name.clone()
}

#[test]
fn cr_2_13_4_un_agent_of_chaos_est_choisi_parmi_toutes_les_cartes_du_trait() {
    let t = arakni(&[]);
    let mut noms: Vec<String> =
        t.g.cards_with_trait(Traits::AGENT_OF_CHAOS)
            .into_iter()
            .map(|c| t.g.db.get(c).name.clone())
            .collect();
    noms.sort();
    assert_eq!(
        noms,
        [
            "Arakni, Black Widow",
            "Arakni, Funnel Web",
            "Arakni, Orb-Weaver",
            "Arakni, Redback",
            "Arakni, Tarantula",
            "Arakni, Trap-Door",
        ]
    );
}

#[test]
fn cr_8_5_25_become_change_les_proprietes_du_heros_mais_pas_son_total_de_vie() {
    let mut t = arakni(&[]);
    t.g.player_mut(P0).life = 13;
    let hero = t.g.player(P0).hero;
    let redback = t.g.db.find_name("Arakni, Redback").unwrap();
    t.g.become_hero(P0, redback);
    assert_eq!(t.g.player(P0).hero, hero, "même objet");
    assert_eq!(heros(&t, P0), "Arakni, Redback");
    assert_eq!(t.vie(P0), 13);
    // La capacité d'Arakni (+1{p} contre un héros marqué) a disparu.
    assert!(t.g.def(hero).aura.is_none());
    assert!(t
        .g
        .turn_events()
        .iter()
        .any(|e| matches!(e, Event::Became { player, card } if *player == P0 && *card == redback)));
}

#[test]
fn cr_8_5_25d_devenir_a_nouveau_la_meme_reference_echoue() {
    let mut t = arakni(&[]);
    let tarantula = t.g.db.find_name("Arakni, Tarantula").unwrap();
    t.g.become_hero(P0, tarantula);
    t.g.become_hero(P0, tarantula);
    let n =
        t.g.turn_events()
            .iter()
            .filter(|e| matches!(e, Event::Became { .. }))
            .count();
    assert_eq!(n, 1);
}

#[test]
fn cr_8_5_52_return_to_the_brood_rend_le_heros_d_origine() {
    let mut t = arakni(&[]);
    let a = t.g.db.find_name("Arakni, Redback").unwrap();
    let b = t.g.db.find_name("Arakni, Tarantula").unwrap();
    t.g.become_hero(P0, a);
    t.g.become_hero(P0, b);
    assert_eq!(heros(&t, P0), "Arakni, Tarantula");
    t.g.return_to_brood(P0);
    assert_eq!(heros(&t, P0), ARAKNI);
    assert!(t.g.became.is_empty());
    // Sans effet « become », return to the brood ne fait rien.
    t.g.return_to_brood(P0);
    assert_eq!(heros(&t, P0), ARAKNI);
}

#[test]
fn cr_9_3_3_une_touche_par_un_effet_retire_la_marque() {
    let mut t = arakni(&[HUNTSMAN]);
    t.g.mark(P1);
    let dague = t.equipement(P0, HUNTSMAN);
    t.g.deal_damage_and_hit(dague, P0, P1, 1);
    assert!(!t.g.player(P1).marked);
    assert!(t.g.turn_events().iter().any(|e| matches!(
        e,
        Event::Hit { attack, marked: true, effect: true, .. } if *attack == dague
    )));
    // Pas de maillon : rien n'est noté comme une attaque qui touche.
    assert!(t.g.combat.links.is_empty());
}

#[test]
fn cr_9_3_3_des_degats_sans_touche_ne_retirent_pas_la_marque() {
    let mut t = arakni(&[]);
    t.g.mark(P1);
    t.g.deal_damage(None, P0, P1, 1, false);
    assert!(t.g.player(P1).marked);
}

#[test]
fn cr_8_5_41_equiper_un_token_arme_sans_main_libre_echoue() {
    let mut t = arakni(&[HUNTSMAN, HUNTSMAN]);
    let c = t.g.db.find_name("Graphene Chelicera").unwrap();
    assert!(t.g.equip_weapon_token(P0, c).is_none());
    assert_eq!(t.g.player(P0).weapons.len(), 2);
    // Avec une main libre, le token est équipé dans une zone d'arme.
    let d = t.g.player(P0).weapons[0];
    t.g.destroy(d);
    let tok = t.g.equip_weapon_token(P0, c).expect("équipé");
    assert_eq!(t.g.obj(tok).zone, Zone::Weapon);
}

#[test]
fn cr_8_5_51a_retrieve_impossible_sans_main_libre() {
    let mut t = arakni(&[HUNTSMAN, HUNTSMAN]);
    let d = t.g.player(P0).weapons[0];
    t.g.destroy(d);
    let c = t.g.db.find_name("Graphene Chelicera").unwrap();
    t.g.equip_weapon_token(P0, c).expect("équipé");
    assert!(!t.g.equip_from_graveyard(d));
    assert_eq!(t.g.obj(d).zone, Zone::Graveyard);
}

#[test]
fn cr_8_5_51_retrieve_equipe_la_carte_comme_un_nouvel_objet() {
    let mut t = arakni(&[HUNTSMAN]);
    let d = t.g.player(P0).weapons[0];
    t.g.destroy(d);
    let avant = t.g.obj_ref(d);
    assert!(t.g.equip_from_graveyard(d));
    assert_eq!(t.g.obj(d).zone, Zone::Weapon);
    assert!(!t.g.is_current(avant), "CR 3.0.9 : nouvel objet");
    assert_eq!(t.g.player(P0).weapons, vec![d]);
}

#[test]
fn cr_4_5_4d_blocage_decks_et_mains_vides_et_vies_inchangees() {
    let mut t = arakni(&[]);
    for p in PlayerId::BOTH {
        let ps = t.g.player_mut(p);
        ps.deck.clear();
        ps.hand.clear();
    }
    for _ in 0..20 {
        if t.g.is_over() {
            break;
        }
        t.finir_le_tour();
    }
    assert_eq!(t.g.outcome, Some(Outcome::Draw));
    assert_eq!(t.g.end_reason, Some(EndReason::Stalemate));
    assert!(t.g.turn < 10, "tour {}", t.g.turn);
}

#[test]
fn cr_7_2_3_the_next_time_you_attack_s_applique_a_l_evenement_d_attaque() {
    // Public Bounty : la cible (héros marqué) est connue à l'attaque.
    let mut t = partie_vraie(
        [ARAKNI, "Essai Héros"],
        &[
            ("Public Bounty", R),
            ("Pursue to the Edge of Oblivion", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &[],
    );
    t.g.player_mut(P0).resource_points = 9;
    t.g.refresh_priority();
    let pb = t.en_main(P0, "Public Bounty", R);
    t.choisir(Choice::Play(pb));
    while t.g.pending_decision().is_some() && !t.g.stack.is_empty() {
        t.choisir(Choice::Pass);
    }
    let a = t.en_main(P0, "Pursue to the Edge of Oblivion", R);
    t.choisir(Choice::Play(a));
    // Sur la pile, l'attaque n'a pas encore le bonus.
    assert_eq!(t.g.power(a), 4);
    while t.g.active_attack() != Some(a) {
        t.choisir(Choice::Pass);
    }
    assert_eq!(t.g.power(a), 7);
}
