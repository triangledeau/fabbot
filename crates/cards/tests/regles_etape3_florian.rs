//! Tests de règles ajoutées pour le deck Florian : cartes doubles et meld
//! (CR 9.2, 8.3.38, 5.3.4d), cibles dans un cimetière (CR 1.8.5a), créateur
//! d'un token (CR 8.5.2c), decompose (CR 8.4.14).

mod commun;
use commun::*;
use fab_engine::card::{Color, Supertypes, Types};
use fab_engine::{Choice, DecisionKind, Event, ObjectId, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);
const SEEDS: &str = "Arcane Seeds // Life";

fn florian(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    arena0: &[&str],
) -> Partie {
    partie_vraie(
        ["Florian", "Essai Héros"],
        main0,
        suite0,
        main1,
        RIEN,
        arena0,
        &[],
    )
}

fn jetons(t: &Partie, nom: &str) -> usize {
    t.g.permanents
        .iter()
        .filter(|&&o| t.g.obj(o).controller == P0 && t.g.def(o).name == nom)
        .count()
}

fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..300 {
        if t.g.is_over() || cond(t) {
            return;
        }
        t.choisir(Choice::Pass);
    }
    panic!("condition jamais atteinte");
}

fn du_deck(t: &Partie, nom: &str, couleur: Option<Color>) -> ObjectId {
    *t.g.player(P0)
        .deck
        .iter()
        .find(|&&c| t.g.def(c).name == nom && t.g.def(c).color == couleur)
        .expect("carte dans le deck")
}

#[test]
fn cr_5_1_2c_le_joueur_choisit_un_cote_ou_les_deux_avec_meld() {
    let mut t = florian(&[(SEEDS, R), ETINCELLE, RAMPART, RAMPART], RIEN, RIEN, &[]);
    let c = t.en_main(P0, SEEDS, R);
    t.choisir(Choice::Play(c));
    assert_eq!(t.kind(), DecisionKind::Mode);
    assert_eq!(
        t.options(),
        vec![Choice::Mode(0), Choice::Mode(1), Choice::Mode(2)]
    );
}

#[test]
fn cr_5_1_2c_seuls_les_cotes_jouables_sont_proposes() {
    // Pendant le tour adverse, seul Life (instant) est jouable : ni Arcane
    // Seeds ni la version fusionnée, qui sont des actions (CR 8.1.1a).
    let mut t = florian(&[(SEEDS, R), RAMPART, RAMPART, RAMPART], RIEN, RIEN, &[]);
    t.finir_le_tour();
    let c = t.en_main(P0, SEEDS, R);
    passer_jusqua_que(&mut t, |t| t.joueur() == P0 && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    assert_ne!(t.kind(), DecisionKind::Mode, "un seul côté possible");
    let played = t.g.turn_events().iter().rev().find_map(|e| match *e {
        Event::Played { def, .. } => Some(def),
        _ => None,
    });
    assert_eq!(t.g.db.get(played.unwrap()).name, "Life");
}

#[test]
fn cr_9_2_3_sur_la_pile_une_carte_double_n_a_que_le_cote_choisi() {
    let mut t = florian(&[(SEEDS, R), ETINCELLE, RAMPART, RAMPART], RIEN, RIEN, &[]);
    let c = t.en_main(P0, SEEDS, R);
    t.choisir(Choice::Play(c));
    t.choisir(Choice::Mode(1));
    assert_eq!(t.g.obj(c).zone, Zone::Stack);
    let d = t.g.def(c);
    assert_eq!(d.name, "Life");
    assert!(d.types.has(Types::INSTANT) && !d.types.has(Types::ACTION));
    assert!(!t.g.supertypes(c).has(Supertypes::RUNEBLADE));
    // Un instant ne coûte pas de point d'action.
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn cr_9_2_2_hors_de_la_pile_une_carte_double_a_les_proprietes_des_deux_cotes() {
    let mut t = florian(&[(SEEDS, R), ETINCELLE, RAMPART, RAMPART], RIEN, RIEN, &[]);
    let c = t.en_main(P0, SEEDS, R);
    t.choisir(Choice::Play(c));
    t.choisir(Choice::Mode(0));
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    let d = t.g.def(c);
    assert!(d.types.has(Types::ACTION) && d.types.has(Types::INSTANT));
    assert!(t
        .g
        .supertypes(c)
        .has(Supertypes::EARTH | Supertypes::RUNEBLADE));
}

#[test]
fn cr_8_3_38_carte_fusionnee_action_et_instant_coute_un_point_d_action() {
    let mut t = florian(&[(SEEDS, R), ETINCELLE, RAMPART, RAMPART], RIEN, RIEN, &[]);
    let c = t.en_main(P0, SEEDS, R);
    t.choisir(Choice::Play(c));
    t.choisir(Choice::Mode(2));
    let d = t.g.def(c);
    // CR 8.3.38c : les deux noms et les types des deux côtés.
    assert!(d.types.has(Types::ACTION) && d.types.has(Types::INSTANT));
    assert!(t
        .g
        .supertypes(c)
        .has(Supertypes::EARTH | Supertypes::RUNEBLADE));
    // CR 8.3.38b : deux fois le coût de base (0) ; CR 8.1.1c : 1 point d'action.
    assert_eq!(t.g.player(P0).action_points, 0);
    assert_eq!(t.g.player(P0).resource_points, 0);
}

#[test]
fn cr_5_3_4d_fusionnee_le_cote_droit_puis_priorite_puis_le_cote_gauche() {
    let mut t = florian(&[(SEEDS, R), ETINCELLE, RAMPART, RAMPART], RIEN, RIEN, &[]);
    let c = t.en_main(P0, SEEDS, R);
    t.choisir(Choice::Play(c));
    t.choisir(Choice::Mode(2));
    t.choisir(Choice::Pass); // l'adversaire passe tout seul
                             // Première résolution : Life (côté droit) seulement.
    assert_eq!(t.vie(P0), 21);
    assert_eq!(jetons(&t, "Runechant"), 0);
    assert_eq!(t.g.obj(c).zone, Zone::Stack);
    assert_eq!((t.kind(), t.joueur()), (DecisionKind::Priority, P0));
    assert_eq!(t.g.player(P0).action_points, 0, "pas encore de go again");
    t.choisir(Choice::Pass);
    // Seconde résolution : Arcane Seeds (côté gauche), puis go again.
    assert_eq!(t.vie(P0), 21);
    assert_eq!(jetons(&t, "Runechant"), 2);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn cr_1_8_5a_une_cible_dans_votre_cimetiere_mais_pas_dans_celui_de_l_adversaire() {
    // Plume of Evergrowth : « target Earth action card … from your graveyard ».
    let mut t = partie_vraie(
        ["Florian", "Essai Héros"],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Autumn's Touch", R)],
        &[("Autumn's Touch", Y)],
        RIEN,
        &["Plume of Evergrowth"],
        &[],
    );
    // Dans la main de départ de l'adversaire.
    let adverse = t.en_main(P1, "Autumn's Touch", Y);
    t.g.put_in_graveyard(adverse);
    t.g.refresh_priority();
    let p = t.equipement(P0, "Plume of Evergrowth");
    assert!(
        !t.peut(Choice::Activate(p, 0)),
        "seule cible : chez l'adversaire"
    );
    let mien = du_deck(&t, "Autumn's Touch", R);
    t.g.put_in_graveyard(mien);
    t.g.refresh_priority();
    t.choisir(Choice::Activate(p, 0));
    // Une seule cible légale : déclarée d'office ; on paie {r}{r}{r}.
    while t.kind() == DecisionKind::Pitch {
        let o = t.options()[0];
        t.choisir(o);
    }
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.obj(mien).zone, Zone::Hand);
    assert_eq!(t.g.obj(adverse).zone, Zone::Graveyard);
}

#[test]
fn cr_8_5_2c_le_createur_d_un_token_n_est_pas_forcement_son_controleur() {
    let mut t = florian(
        &[RAMPART; 4],
        &[
            ("Autumn's Touch", R),
            ("Autumn's Touch", R),
            ("Autumn's Touch", Y),
            ("Autumn's Touch", B),
        ],
        RIEN,
        &[],
    );
    for (n, c) in [
        ("Autumn's Touch", R),
        ("Autumn's Touch", R),
        ("Autumn's Touch", Y),
        ("Autumn's Touch", B),
    ] {
        let x = du_deck(&t, n, c);
        t.g.banish(x, false);
    }
    let rc = t.g.db.find_name("Runechant").unwrap();
    t.g.create_tokens_by(P1, P0, rc, 2);
    assert_eq!(jetons(&t, "Runechant"), 2, "Florian ne crée pas ces tokens");
}

#[test]
fn cr_8_4_14_decompose_bannit_trois_cartes_distinctes() {
    // Une carte Earth d'action ne peut pas compter deux fois.
    let mut t = florian(
        &[("Chorus of Rotwood", R), RAMPART, RAMPART, RAMPART],
        &[("Harvest Season", R), ("Autumn's Touch", R)],
        RIEN,
        &[],
    );
    for (n, c) in [("Harvest Season", R), ("Autumn's Touch", R)] {
        let x = du_deck(&t, n, c);
        t.g.put_in_graveyard(x);
    }
    t.g.refresh_priority();
    let c = t.en_main(P0, "Chorus of Rotwood", R);
    t.choisir(Choice::Play(c));
    while t.kind() == DecisionKind::Pitch {
        let o = t.options()[0];
        t.choisir(o);
    }
    // 2 cartes seulement au cimetière : pas de question decompose.
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert!(t.g.player(P0).banished.is_empty());
    assert_eq!(jetons(&t, "Runechant"), 3);
}
