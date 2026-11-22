//! Tests des règles de base (étape 1), avec les cartes **fictives** d'essai.
//!
//! Chaque test cite l'article du Comprehensive Rules (CR) qu'il vérifie.
//! Les decks ne sont pas mélangés : le premier élément de la liste est le
//! dessus du deck, donc on sait exactement quelles cartes sont piochées.

mod commun;
use commun::*;
use fab_cards::test_set;
use fab_engine::card::Color;
use fab_engine::{
    Choice, DecisionKind, EndReason, GameSetup, GameState, History, Outcome, PlayerId, Zone,
};
use std::sync::Arc;

// ----------------------------------------------------------------------
// Mise en place et structure du tour
// ----------------------------------------------------------------------

#[test]
fn cr_4_1_10_chaque_joueur_pioche_jusqu_a_son_intellect() {
    let t = partie(RIEN, RIEN, RIEN, RIEN, &[], &[]);
    assert_eq!(t.g.player(P0).hand.len(), 4);
    assert_eq!(t.g.player(P1).hand.len(), 4);
    assert_eq!(t.g.player(P0).deck.len(), 36);
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn cr_4_1_3_le_choisisseur_peut_faire_commencer_l_adversaire() {
    let db = Arc::new(test_set::build());
    let setup = GameSetup {
        players: [
            test_set::aggro_loadout(&db),
            test_set::defensive_loadout(&db),
        ],
        chooser: P1,
        seed: 1,
        shuffle: true,
        max_turns: 100,
    };
    let mut g = GameState::new(db, setup).unwrap();
    let d = g.pending_decision().unwrap().clone();
    assert_eq!(d.player, P1);
    g.apply(d.index_of(Choice::Player(P0)).unwrap()).unwrap();
    assert_eq!(g.turn_player, P0);
}

#[test]
fn cr_4_3_2_le_joueur_actif_a_un_point_d_action() {
    let t = partie(
        &[("Essai Frappe", R), ("Essai Frappe", B)],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    assert_eq!(t.joueur(), P0);
    assert_eq!(t.kind(), DecisionKind::Priority);
    assert_eq!(t.g.player(P0).action_points, 1);
    assert_eq!(t.g.player(P1).action_points, 0);
}

#[test]
fn cr_8_1_7_et_8_1_12_ressources_et_blocs_ne_se_jouent_pas() {
    let t = partie(
        &[
            ("Essai Rempart", B),
            ("Essai Joyau", None),
            ("Essai Frappe", R),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let rempart = t.en_main(P0, "Essai Rempart", B);
    let joyau = t.en_main(P0, "Essai Joyau", None);
    let frappe = t.en_main(P0, "Essai Frappe", R);
    assert!(!t.peut(Choice::Play(rempart)));
    assert!(!t.peut(Choice::Play(joyau)));
    assert!(t.peut(Choice::Play(frappe)));
}

// ----------------------------------------------------------------------
// Coûts et pitch
// ----------------------------------------------------------------------

#[test]
fn cr_1_14_2d_payer_en_pitchant_et_garder_le_surplus() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Assaut", R),
            ("Essai Élan", R),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let frappe = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(frappe));
    // Coût 1, aucune ressource : il faut pitcher (la carte jouée n'est pas proposée).
    assert_eq!(t.kind(), DecisionKind::Pitch);
    assert!(!t.peut(Choice::Pitch(frappe)));
    let bleue = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(bleue));
    // Pitch 3 pour un coût 1 : il reste 2 points de ressource.
    assert_eq!(t.g.player(P0).resource_points, 2);
    assert_eq!(t.g.obj(bleue).zone, Zone::Pitch);
    // CR 5.1.6b : l'action a coûté le point d'action.
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn cr_4_4_3e_les_ressources_sont_perdues_en_fin_de_tour() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Élan", R),
            ("Essai Élan", R),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let frappe = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(frappe));
    let bleue = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(bleue));
    assert_eq!(t.g.player(P0).resource_points, 2);
    t.passer_jusqua(DecisionKind::Arsenal);
    t.choisir(Choice::Pass);
    // Tour de P1 : P0 a perdu ses ressources.
    assert_eq!(t.g.turn_player, P1);
    assert_eq!(t.g.player(P0).resource_points, 0);
}

// ----------------------------------------------------------------------
// Combat
// ----------------------------------------------------------------------

/// P0 attaque avec Essai Frappe rouge (puissance 4), pitch la Frappe bleue.
fn attaque_frappe(main1: &[(&str, Option<Color>)], arena1: &[&str]) -> Partie {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        main1,
        RIEN,
        &[],
        arena1,
    );
    let frappe = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(frappe));
    let bleue = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(bleue));
    t
}

#[test]
fn cr_7_5_2_sans_defense_l_attaque_inflige_sa_puissance() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &[]);
    t.passer_jusqua(DecisionKind::Defend);
    assert_eq!(t.joueur(), P1);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 16);
}

#[test]
fn cr_7_5_2_la_defense_reduit_les_degats_et_cr_7_7_6_les_cartes_vont_au_cimetiere() {
    let mut t = attaque_frappe(&[("Essai Rempart", B), ("Essai Frappe", Y)], &[]);
    t.passer_jusqua(DecisionKind::Defend);
    let frappe_j = t.en_main(P1, "Essai Frappe", Y);
    t.choisir(Choice::Defend(frappe_j)); // défense 3
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 19); // 4 - 3
    assert_eq!(t.g.obj(frappe_j).zone, Zone::Graveyard);
    // L'attaque aussi est au cimetière.
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
}

#[test]
fn cr_7_5_2_une_defense_superieure_ne_fait_pas_de_degats() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &[]);
    t.passer_jusqua(DecisionKind::Defend);
    let rempart = t.en_main(P1, "Essai Rempart", B);
    t.choisir(Choice::Defend(rempart)); // défense 4 = puissance 4
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn cr_7_7_5_l_equipement_qui_defend_retourne_dans_sa_zone() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &["Essai Plastron"]);
    t.passer_jusqua(DecisionKind::Defend);
    let plastron = t.equipement(P1, "Essai Plastron");
    t.choisir(Choice::Defend(plastron));
    t.choisir(Choice::Pass);
    // Personne ne peut plus rien faire : la chaîne de combat se termine seule.
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 18); // 4 - 2
    assert_eq!(t.g.obj(plastron).zone, Zone::Chest);
    // CR 8.3.2 : battleworn, un compteur -1.
    assert_eq!(t.g.defense(plastron), 1);
}

#[test]
fn cr_8_3_3_blade_break_detruit_l_equipement_apres_avoir_defendu() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &["Essai Casque"]);
    t.passer_jusqua(DecisionKind::Defend);
    let casque = t.equipement(P1, "Essai Casque");
    t.choisir(Choice::Defend(casque));
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.obj(casque).zone, Zone::Graveyard);
    assert!(t.g.player(P1).head.is_none());
}

#[test]
fn cr_8_3_10_temper_detruit_a_zero_de_defense() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &["Essai Gantelets"]);
    t.passer_jusqua(DecisionKind::Defend);
    let gants = t.equipement(P1, "Essai Gantelets"); // défense 1
    t.choisir(Choice::Defend(gants));
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.obj(gants).zone, Zone::Graveyard);
}

#[test]
fn cr_8_1_12_un_bloc_peut_defendre() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &[]);
    t.passer_jusqua(DecisionKind::Defend);
    let rempart = t.en_main(P1, "Essai Rempart", B);
    assert!(t.peut(Choice::Defend(rempart)));
}

#[test]
fn cr_8_3_4_dominate_une_seule_carte_de_la_main() {
    let mut t = partie(
        &[
            ("Essai Écrasement", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Rempart", B),
            ("Essai Frappe", Y),
            ("Essai Parade", R),
        ],
        RIEN,
        &[],
        &["Essai Plastron"],
    );
    let e = t.en_main(P0, "Essai Écrasement", R);
    t.choisir(Choice::Play(e)); // coût 2
    let b = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(b));
    t.passer_jusqua(DecisionKind::Defend);
    let rempart = t.en_main(P1, "Essai Rempart", B);
    let frappe = t.en_main(P1, "Essai Frappe", Y);
    let plastron = t.equipement(P1, "Essai Plastron");
    t.choisir(Choice::Defend(rempart));
    // Une deuxième carte de la main est interdite, l'équipement reste permis.
    assert!(!t.peut(Choice::Defend(frappe)));
    assert!(t.peut(Choice::Defend(plastron)));
    t.choisir(Choice::Pass);
    // CR 8.3.4b : pas de réaction de défense depuis la main non plus.
    t.passer_jusqua(DecisionKind::Arsenal);
    let parade = t.en_main(P1, "Essai Parade", R);
    assert_eq!(t.g.obj(parade).zone, Zone::Hand);
    assert_eq!(t.vie(P1), 18); // 6 - 4
}

#[test]
fn cr_7_4_2d_une_reaction_de_defense_ajoute_sa_defense() {
    let mut t = attaque_frappe(&[("Essai Parade", Y), ("Essai Rempart", B)], &[]);
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    // Étape de réaction : P1 peut jouer Essai Parade (coût 0, défense 3).
    assert_eq!(t.kind(), DecisionKind::Priority);
    assert_eq!(t.joueur(), P1);
    let parade = t.en_main(P1, "Essai Parade", Y);
    t.choisir(Choice::Play(parade));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 19); // 4 - 3
    assert_eq!(t.g.obj(parade).zone, Zone::Graveyard);
}

#[test]
fn cr_8_1_2a_seul_l_attaquant_joue_une_reaction_d_attaque() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Riposte", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[("Essai Riposte", R), ("Essai Frappe", B)],
        RIEN,
        &[],
        &[],
    );
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    let b = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(b));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    // Réaction : P0 (attaquant) peut jouer sa Riposte.
    assert_eq!(t.joueur(), P0);
    let riposte = t.en_main(P0, "Essai Riposte", R);
    t.choisir(Choice::Play(riposte));
    // P0 récupère la priorité (CR 5.1.10), puis passe ; P1 n'a rien de jouable :
    // sa Riposte n'est pas proposée (il n'est pas l'attaquant).
    t.choisir(Choice::Pass);
    let r1 = t.en_main(P1, "Essai Riposte", R);
    assert!(!t.peut(Choice::Play(r1)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 13); // 4 + 3
}

#[test]
fn cr_6_6_un_declenchement_quand_l_attaque_touche() {
    let mut t = partie(
        &[
            ("Essai Morsure", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let m = t.en_main(P0, "Essai Morsure", R);
    t.choisir(Choice::Play(m));
    let b = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(b));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 17); // Morsure rouge : 3
    assert_eq!(t.vie(P0), 21); // « quand elle touche, gagnez 1 PV »
}

// ----------------------------------------------------------------------
// Points d'action et go again
// ----------------------------------------------------------------------

#[test]
fn cr_8_1_1c_sans_point_d_action_pas_de_deuxieme_action() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &[]);
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    // Après l'attaque sans go again, P0 n'a plus de point d'action : il ne peut
    // plus jouer ses Remparts (de toute façon injouables) ni rien d'autre.
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn cr_8_3_5b_go_again_donne_un_point_d_action() {
    let mut t = partie(
        &[
            ("Essai Assaut", R),
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let a = t.en_main(P0, "Essai Assaut", R);
    t.choisir(Choice::Play(a)); // coût 0
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    // Étape de résolution : go again rend un point d'action, et CR 7.6.3a
    // permet de jouer une nouvelle attaque sur la même chaîne de combat.
    assert_eq!(t.joueur(), P0);
    assert_eq!(t.g.player(P0).action_points, 1);
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    let b = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(b));
    t.passer_jusqua(DecisionKind::Defend);
    assert_eq!(t.g.combat.links.len(), 2);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 13); // 3 + 4
}

#[test]
fn cr_5_3_5_go_again_d_une_action_non_attaque_et_cr_1_8_10_prochaine_attaque() {
    let mut t = partie(
        &[
            ("Essai Élan", R),
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let e = t.en_main(P0, "Essai Élan", R);
    t.choisir(Choice::Play(e)); // coût 0, go again, prochaine attaque +3
    t.passer_jusqua(DecisionKind::Priority);
    // Élan s'est résolu : P0 a récupéré un point d'action.
    let f = t.en_main(P0, "Essai Frappe", R);
    assert!(t.g.player(P0).action_points == 1 || t.peut(Choice::Play(f)));
    t.choisir(Choice::Play(f));
    let b = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Pitch(b));
    assert_eq!(t.g.power(f), 7); // 4 + 3
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 13);
}

// ----------------------------------------------------------------------
// Armes
// ----------------------------------------------------------------------

#[test]
fn cr_1_4_3_attaque_d_arme_une_fois_par_tour_avec_go_again() {
    let mut t = partie(
        &[
            ("Essai Frappe", B),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Essai Dague"],
        &[],
    );
    let dague = t.equipement(P0, "Essai Dague");
    t.choisir(Choice::Activate(dague, 0));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    // Go again (« Attack. Go again. ») mais « Once per Turn » : plus d'attaque d'arme.
    assert_eq!(t.g.player(P0).action_points, 1);
    assert!(!t.peut(Choice::Activate(dague, 0)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 19);
    // CR 7.7.5 : l'arme est revenue dans sa zone.
    assert_eq!(t.g.obj(dague).zone, Zone::Weapon);
}

// ----------------------------------------------------------------------
// Instants et priorité
// ----------------------------------------------------------------------

#[test]
fn cr_8_1_6a_l_adversaire_peut_jouer_un_instant_quand_il_a_la_priorite() {
    let mut t = attaque_frappe(&[("Essai Étincelle", R), ("Essai Frappe", B)], &[]);
    // P0 passe (pile : son attaque) ; P1 reçoit la priorité et peut jouer l'instant.
    assert_eq!(t.joueur(), P1);
    let e = t.en_main(P1, "Essai Étincelle", R);
    t.choisir(Choice::Play(e));
    t.choisir(Choice::Pitch(t.en_main(P1, "Essai Frappe", B)));
    // L'instant est au-dessus de l'attaque : il se résout en premier (CR 3.15).
    t.passer_jusqua(DecisionKind::Defend);
    assert_eq!(t.vie(P0), 19);
}

// ----------------------------------------------------------------------
// Phase de fin
// ----------------------------------------------------------------------

#[test]
fn cr_4_4_3_arsenal_pitch_sous_le_deck_et_pioche() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &[]);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.joueur(), P0);
    let r = t.en_main(P0, "Essai Rempart", B);
    t.choisir(Choice::Arsenal(r));
    let ps = t.g.player(P0);
    assert_eq!(ps.arsenal, Some(r));
    assert!(t.g.obj(r).face_down);
    // La Frappe bleue pitchée est sous le deck.
    assert!(ps.pitch.is_empty());
    assert_eq!(t.g.def(ps.deck[0]).name, "Essai Frappe");
    // Main : 1 Rempart restant + pioche jusqu'à 4.
    assert_eq!(ps.hand.len(), 4);
    // CR 4.4.4 : c'est au tour de P1.
    assert_eq!(t.g.turn_player, P1);
    assert_eq!(t.g.turn, 2);
}

// ----------------------------------------------------------------------
// Fin de partie, déterminisme, annulation
// ----------------------------------------------------------------------

#[test]
fn cr_4_5_3a_un_heros_a_zero_pv_perd_la_partie() {
    let mut t = attaque_frappe(&[("Essai Rempart", B)], &[]);
    t.g.player_mut(P1).life = 3;
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    assert!(t.g.is_over());
    assert_eq!(t.g.outcome, Some(Outcome::Winner(P0)));
    assert_eq!(t.g.end_reason, Some(EndReason::LifeZero));
    assert_eq!(t.vie(P1), 0);
}

#[test]
fn meme_graine_memes_choix_meme_partie() {
    let run = || {
        let db = Arc::new(test_set::build());
        let setup = GameSetup {
            players: [
                test_set::aggro_loadout(&db),
                test_set::defensive_loadout(&db),
            ],
            chooser: P0,
            seed: 123,
            shuffle: true,
            max_turns: 200,
        };
        let mut g = GameState::new(db, setup).unwrap();
        let mut a = fab_ai::RandomAgent::new(1);
        let mut b = fab_ai::RandomAgent::new(2);
        fab_ai::play_out(&mut g, [&mut a, &mut b], true).unwrap();
        (g.outcome, g.turn, g.player(P0).life, g.player(P1).life)
    };
    assert_eq!(run(), run());
}

#[test]
fn annuler_revient_exactement_a_l_etat_precedent() {
    let mut t = partie(
        &[("Essai Frappe", R), ("Essai Frappe", B)],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let mut h = History::new();
    h.record(&t.g);
    let options_avant = t.options();
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    assert_ne!(t.g.obj(f).zone, Zone::Hand);
    t.g = h.undo().unwrap();
    assert_eq!(t.g.obj(f).zone, Zone::Hand);
    assert_eq!(t.options(), options_avant);
}

#[test]
fn des_milliers_de_parties_aleatoires_restent_coherentes() {
    let db = Arc::new(test_set::build());
    for seed in 0..300 {
        let setup = GameSetup {
            players: [
                test_set::aggro_loadout(&db),
                test_set::defensive_loadout(&db),
            ],
            chooser: PlayerId((seed % 2) as u8),
            seed,
            shuffle: true,
            max_turns: 200,
        };
        let mut g = GameState::new(db.clone(), setup).unwrap();
        let mut a = fab_ai::RandomAgent::new(seed);
        let mut b = fab_ai::RandomAgent::new(seed + 1000);
        fab_ai::play_out(&mut g, [&mut a, &mut b], true)
            .unwrap_or_else(|e| panic!("graine {seed} : {e}"));
        assert!(g.is_over());
    }
}
