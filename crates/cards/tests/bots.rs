//! Tests des bots (étape 4) : B1 heuristique et la `Vue` (information
//! d'un joueur), avec les cartes d'essai et les 9 vrais decks.

mod commun;
use commun::*;
use fab_ai::vue::Vue;
use fab_ai::{creer_agent, play_out, Agent, HeuristicAgent, BOTS};
use fab_cards::decklist::Decklist;
use fab_cards::real;
use fab_engine::{Choice, DecisionKind, GameSetup, GameState};
use std::sync::Arc;

/// Le choix de B1 pour la décision en attente.
fn choix_b1(t: &Partie) -> Choice {
    let d = t.g.pending_decision().expect("décision").clone();
    let i = HeuristicAgent::new().choose(&t.g, &d);
    d.options[i]
}

// ----------------------------------------------------------------------
// La vue : un bot ne voit que l'information de son joueur
// ----------------------------------------------------------------------

#[test]
fn la_vue_montre_ma_main() {
    let t = partie(&[("Essai Frappe", R)], RIEN, RIEN, RIEN, &[], &[]);
    let v = Vue::new(&t.g, P0);
    let c = t.en_main(P0, "Essai Frappe", R);
    assert_eq!(v.carte(c).name, "Essai Frappe");
    assert_eq!(v.ma_main().len(), 4);
}

#[test]
#[should_panic(expected = "carte cachée")]
fn la_vue_refuse_de_lire_la_main_adverse() {
    let t = partie(RIEN, RIEN, &[("Essai Frappe", R)], RIEN, &[], &[]);
    let v = Vue::new(&t.g, P0);
    let c = t.en_main(P1, "Essai Frappe", R);
    assert!(!v.visible(c));
    v.carte(c);
}

#[test]
fn la_vue_cache_le_deck_meme_a_son_proprietaire() {
    let t = partie(RIEN, RIEN, RIEN, RIEN, &[], &[]);
    let v = Vue::new(&t.g, P0);
    let dessus = *t.g.player(P0).deck.last().unwrap();
    assert!(!v.visible(dessus));
    // Mais on connaît la taille des mains.
    assert_eq!(v.taille_main(P1), 4);
}

// ----------------------------------------------------------------------
// B1 à son tour
// ----------------------------------------------------------------------

#[test]
fn b1_joue_d_abord_l_attaque_avec_go_again() {
    // Essai Frappe rouge : 4 puissance sans go again ; Essai Assaut rouge :
    // 3 puissance avec go again. Jouer l'Assaut d'abord garde l'action.
    let t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Assaut", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    assert_eq!(t.kind(), DecisionKind::Priority);
    let assaut = t.en_main(P0, "Essai Assaut", R);
    assert_eq!(choix_b1(&t), Choice::Play(assaut));
}

#[test]
fn b1_joue_la_plus_grosse_attaque_sans_go_again() {
    let t = partie(
        &[
            ("Essai Frappe", B),
            ("Essai Frappe", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let rouge = t.en_main(P0, "Essai Frappe", R);
    assert_eq!(choix_b1(&t), Choice::Play(rouge));
}

#[test]
fn b1_pitche_au_plus_juste() {
    // Essai Écrasement rouge coûte 2 : l'Assaut jaune (pitch 2) le paie
    // exactement, la Frappe bleue (pitch 3) gaspillerait 1 ressource.
    let mut t = partie(
        &[
            ("Essai Écrasement", R),
            ("Essai Frappe", B),
            ("Essai Assaut", Y),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let e = t.en_main(P0, "Essai Écrasement", R);
    t.choisir(Choice::Play(e));
    assert_eq!(t.kind(), DecisionKind::Pitch);
    let jaune = t.en_main(P0, "Essai Assaut", Y);
    assert_eq!(choix_b1(&t), Choice::Pitch(jaune));
}

// ----------------------------------------------------------------------
// B1 en défense
// ----------------------------------------------------------------------

/// P0 attaque avec la carte donnée (payée avec des Rempart) ; P1 (main de
/// Rempart, défense 4) doit déclarer ses défenseurs.
fn attaque_contre_p1(attaque: (&str, Option<fab_engine::card::Color>), vie_p1: i32) -> Partie {
    let mut t = partie(
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
        &[],
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

#[test]
fn b1_defend_contre_une_attaque_mortelle() {
    // 4 puissance contre 3 PV : il faut défendre.
    let t = attaque_contre_p1(("Essai Frappe", R), 3);
    assert!(matches!(choix_b1(&t), Choice::Defend(_)));
}

#[test]
fn b1_defend_quand_une_carte_empeche_assez_de_degats() {
    // 4 puissance, un Rempart (défense 4, aucune valeur en attaque) l'arrête.
    let t = attaque_contre_p1(("Essai Frappe", R), 20);
    assert!(matches!(choix_b1(&t), Choice::Defend(_)));
}

#[test]
fn b1_ne_gache_pas_une_carte_pour_une_petite_attaque() {
    // Essai Assaut bleu : 1 puissance, pas la peine de défendre à 20 PV.
    let t = attaque_contre_p1(("Essai Assaut", B), 20);
    assert_eq!(choix_b1(&t), Choice::Pass);
}

#[test]
fn b1_arrete_de_defendre_quand_l_attaque_est_bloquee() {
    let mut t = attaque_contre_p1(("Essai Frappe", R), 20);
    let c = choix_b1(&t);
    t.choisir(c);
    assert_eq!(t.kind(), DecisionKind::Defend);
    assert_eq!(choix_b1(&t), Choice::Pass);
}

// ----------------------------------------------------------------------
// Parties complètes
// ----------------------------------------------------------------------

#[test]
fn les_bots_se_creent_par_leur_nom() {
    for nom in BOTS {
        assert!(creer_agent(nom, 1).is_some(), "{nom}");
    }
    assert!(creer_agent("b9", 1).is_none());
}

#[test]
fn b1_joue_des_parties_completes_avec_les_9_decks_sans_erreur() {
    // Toutes les décisions passent par la Vue : une lecture d'information
    // cachée ferait paniquer le test. `play_out` vérifie aussi la cohérence
    // de l'état après chaque décision.
    let db = Arc::new(real::build());
    let fichiers: Vec<_> = fab_cards::decks::tous()
        .into_iter()
        .map(|d| d.fichier)
        .collect();
    assert!(fichiers.len() >= 9);
    let decks: Vec<_> = fichiers
        .iter()
        .map(|f| {
            let list = Decklist::parse(&std::fs::read_to_string(f).unwrap()).unwrap();
            real::default_loadout(&db, &list).unwrap()
        })
        .collect();
    for (i, a) in decks.iter().enumerate() {
        for (j, b) in decks.iter().enumerate() {
            let setup = GameSetup {
                players: [a.clone(), b.clone()],
                chooser: P0,
                seed: (i * 9 + j) as u64,
                shuffle: true,
                max_turns: 200,
            };
            let mut g = GameState::new(db.clone(), setup).unwrap();
            let mut x = HeuristicAgent::new();
            let mut y = creer_agent(if j % 2 == 0 { "b1" } else { "b0" }, 7).unwrap();
            play_out(&mut g, [&mut x, &mut *y], true)
                .unwrap_or_else(|e| panic!("decks {i} contre {j} : {e}"));
            assert!(g.is_over());
        }
    }
}
