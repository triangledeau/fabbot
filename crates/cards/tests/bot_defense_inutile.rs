//! Les bots ne défendent pas avec un équipement qui n'ajoute aucune défense
//! (ex. les Nullrune : défense 0, « Arcane Barrier 1 », rien à faire dans
//! un blocage). Signalé par le propriétaire le 2026-11-27 : contre une
//! Chute d'étoile (3), le bot avait posé une carte + Nullrune Gloves +
//! Nullrune Hood.

mod commun;
use commun::*;
use fab_ai::b2::{cles_des_options, ReglagesB2};
use fab_ai::{equipement_sans_defense, Agent, HeuristicAgent, IsmctsAgent};
use fab_engine::{Choice, DecisionKind};

const NULLRUNE: [&str; 2] = ["Nullrune Hood", "Nullrune Gloves"];

/// P0 attaque avec la carte donnée (payée avec un Rempart) ; P1, qui porte
/// la capuche et les gants Nullrune, doit déclarer ses défenseurs.
fn attaque_contre_p1(attaque: (&str, Option<fab_engine::card::Color>), vie_p1: i32) -> Partie {
    let mut t = partie_vraie(
        ["Essai Héros", "Essai Héros"],
        &[
            attaque,
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &NULLRUNE,
    );
    t.g.player_mut(P1).life = vie_p1;
    let a = t.en_main(P0, attaque.0, attaque.1);
    t.choisir(Choice::Play(a));
    if t.kind() == DecisionKind::Pitch {
        let r = t.en_main(P0, "Essai Rempart", B);
        t.choisir(Choice::Pitch(r));
    }
    t.passer_jusqua(DecisionKind::Defend);
    assert_eq!(t.joueur(), P1);
    t
}

fn est_nullrune(t: &Partie, c: Choice) -> bool {
    matches!(c, Choice::Defend(o) if t.g.def(o).name.starts_with("Nullrune"))
}

#[test]
fn les_equipements_nullrune_sont_reconnus_sans_defense() {
    let t = attaque_contre_p1(("Essai Frappe", R), 20);
    for nom in NULLRUNE {
        let e = t.equipement(P1, nom);
        // Le moteur garde le droit de défendre avec (c'est légal)…
        assert!(t.peut(Choice::Defend(e)), "{nom}");
        // … mais il n'apporte rien.
        assert!(equipement_sans_defense(&t.g, e), "{nom}");
    }
    let rempart = t.en_main(P1, "Essai Rempart", B);
    assert!(!equipement_sans_defense(&t.g, rempart));
}

#[test]
fn la_recherche_n_essaie_pas_de_defendre_avec_un_equipement_sans_defense() {
    // B2, B3 et le niveau réfléchi ne considèrent que ces options.
    let t = attaque_contre_p1(("Essai Frappe", R), 20);
    let d = t.g.pending_decision().unwrap().clone();
    let options: Vec<Choice> = cles_des_options(&t.g, &d)
        .iter()
        .map(|&(_, i)| d.options[i])
        .collect();
    assert!(options.contains(&Choice::Pass));
    assert!(options.iter().any(|&c| matches!(c, Choice::Defend(_))));
    assert!(
        !options.iter().any(|&c| est_nullrune(&t, c)),
        "{:?}",
        t.noms(&options)
    );
}

#[test]
fn b3_ne_defend_jamais_avec_un_equipement_sans_defense() {
    let mut t = attaque_contre_p1(("Essai Frappe", R), 20);
    for graine in 0..4 {
        let mut bot = IsmctsAgent::avec(
            graine,
            ReglagesB2 {
                iterations: 60,
                ..ReglagesB2::b3()
            },
        );
        // Toute la déclaration des défenseurs, jusqu'au « Pass ».
        let mut u = Partie { g: t.g.clone() };
        while u.g.pending_decision().map(|d| d.kind) == Some(DecisionKind::Defend) {
            let d = u.g.pending_decision().unwrap().clone();
            let c = d.options[bot.choose(&u.g, &d)];
            assert!(!est_nullrune(&u, c), "graine {graine} : {:?}", u.noms(&[c]));
            u.choisir(c);
            if c == Choice::Pass {
                break;
            }
        }
    }
    t.choisir(Choice::Pass);
}

#[test]
fn b1_ne_defend_pas_avec_un_equipement_sans_defense_meme_face_a_une_attaque_mortelle() {
    // Essai Écrasement rouge : 6 puissance, dominate (une seule carte de la
    // main). À 2 PV, après un Rempart (4), il reste 2 dégâts mortels : les
    // Nullrune n'y changent rien.
    let mut t = attaque_contre_p1(("Essai Écrasement", R), 2);
    let rempart = t.en_main(P1, "Essai Rempart", B);
    t.choisir(Choice::Defend(rempart));
    assert_eq!(t.kind(), DecisionKind::Defend);
    let d = t.g.pending_decision().unwrap().clone();
    assert!(d.options.iter().any(|&c| est_nullrune(&t, c)));
    let c = d.options[HeuristicAgent::new().choose(&t.g, &d)];
    assert_eq!(c, Choice::Pass, "{:?}", t.noms(&[c]));
}
