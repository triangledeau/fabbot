//! B1 et les sorts arcaniques d'Iyslander contre l'arcane barrier adverse :
//! jouer les sorts qui la passent, les renforcer (Crucible of Aetherweave,
//! amp), garder une carte Ice bleue en arsenal pour le tour adverse.
//! Mesures : `docs/analyses/2026-11-26-iyslander-hala-ira-b3.md`.

mod commun;
use commun::*;
use fab_ai::{Agent, HeuristicAgent};
use fab_engine::{Choice, DecisionKind};

fn choix_b1(t: &Partie) -> Choice {
    let d = t.g.pending_decision().unwrap().clone();
    let i = HeuristicAgent::new().choose(&t.g, &d);
    d.options[i]
}

/// Arcane Barrier 2 + Arcane Barrier 1 : AB3 en tout.
const AB3: &[&str] = &["Aetherstorm Wellingtons", "Spellfire Cloak"];

fn iyslander(
    main: &[(&str, Option<fab_engine::card::Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_vraie(
        ["Iyslander", "Hala"],
        main,
        RIEN,
        RIEN,
        RIEN,
        arena0,
        arena1,
    )
}

#[test]
fn b1_joue_le_sort_qui_passe_l_arcane_barrier() {
    // Frosting bleu (1) est arrêté par AB3, Ice Bolt rouge (5) en passe 2.
    let t = iyslander(
        &[
            ("Frosting", B),
            ("Ice Bolt", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        &[],
        AB3,
    );
    let bolt = t.en_main(P0, "Ice Bolt", R);
    assert_eq!(choix_b1(&t), Choice::Play(bolt));
}

#[test]
fn b1_garde_un_sort_entierement_arrete_par_l_arcane_barrier() {
    let t = iyslander(
        &[
            ("Frosting", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        &[],
        AB3,
    );
    assert_eq!(choix_b1(&t), Choice::Pass);
}

#[test]
fn b1_active_le_crucible_avant_son_sort() {
    let t = iyslander(
        &[
            ("Ice Bolt", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        &["Crucible of Aetherweave"],
        &[],
    );
    let crucible = t.equipement(P0, "Crucible of Aetherweave");
    assert_eq!(choix_b1(&t), Choice::Activate(crucible, 0));
}

#[test]
fn b1_n_active_pas_le_crucible_sans_sort_a_renforcer() {
    let t = iyslander(
        &[
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        &["Crucible of Aetherweave"],
        &[],
    );
    assert_eq!(choix_b1(&t), Choice::Pass);
}

#[test]
fn b1_defausse_arcane_twining_pour_qu_ice_bolt_passe_l_arcane_barrier() {
    // Ice Bolt bleu (3) contre AB3 : 0 dégât ; avec amp 1, 1 dégât passe.
    let t = iyslander(
        &[
            ("Ice Bolt", B),
            ("Arcane Twining", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        &[],
        AB3,
    );
    let twining = t.en_main(P0, "Arcane Twining", B);
    assert_eq!(choix_b1(&t), Choice::Activate(twining, 0));
}

#[test]
fn b1_met_une_carte_ice_bleue_en_arsenal_puis_la_joue_au_tour_adverse() {
    // Ice Bolt rouge vaut plus en main, mais Frosting bleu joué depuis
    // l'arsenal au tour adverse crée une gelure chez l'adversaire.
    let mut t = iyslander(
        &[
            ("Frosting", B),
            ("Ice Bolt", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        &[],
        AB3,
    );
    t.passer_jusqua(DecisionKind::Arsenal);
    let frosting = t.en_main(P0, "Frosting", B);
    assert_eq!(choix_b1(&t), Choice::Arsenal(frosting));
    t.choisir(Choice::Arsenal(frosting));
    // Tour de P1 : il passe jusqu'à ce que P0 puisse jouer Frosting.
    for _ in 0..50 {
        if t.joueur() == P0 && t.peut(Choice::Play(frosting)) {
            break;
        }
        let premier = t.options()[0];
        let c = if t.peut(Choice::Pass) {
            Choice::Pass
        } else {
            premier
        };
        t.choisir(c);
    }
    assert_ne!(t.g.turn_player, P0);
    assert_eq!(choix_b1(&t), Choice::Play(frosting));
}
