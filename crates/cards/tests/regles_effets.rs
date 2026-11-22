//! Tests du système d'effets (suite de l'étape 1) : capacités activées,
//! permanents, coûts optionnels et d'effet, modes, cibles, choix pendant la
//! résolution, déclenchements et leur ordre, prévention, dégâts arcaniques,
//! effets statiques de héros. Cartes **fictives** (`fab_cards::test_set`).

mod commun;
use commun::*;
use fab_engine::script::Target;
use fab_engine::{Choice, DecisionKind, Zone};

#[test]
fn cr_5_2_objet_permanent_et_capacite_activee_avec_cout_d_effet() {
    let mut t = partie(
        &[
            ("Essai Potion", B),
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
    let potion = t.en_main(P0, "Essai Potion", B);
    t.choisir(Choice::Play(potion));
    // CR 5.3.6a : un objet (item) entre dans l'arène comme permanent.
    assert_eq!(t.g.obj(potion).zone, Zone::Permanent);
    t.finir_le_tour();
    t.finir_le_tour();
    // Tour 3 : « Action — Destroy this: Gain 2 action points ».
    assert_eq!(t.g.turn_player, P0);
    assert!(t.peut(Choice::Activate(potion, 0)));
    t.choisir(Choice::Activate(potion, 0));
    // Coût : 1 point d'action (capacité de type action) ; effet : +2.
    assert_eq!(t.g.player(P0).action_points, 2);
    assert_eq!(t.g.obj(potion).zone, Zone::Graveyard);
}

#[test]
fn cr_8_1_6a_capacite_instant_pendant_le_tour_adverse() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &["Essai Bouclier"],
    );
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    // P1 a la priorité pendant l'étape des couches : il peut activer son équipement.
    assert_eq!(t.joueur(), P1);
    let bouclier = t.equipement(P1, "Essai Bouclier");
    t.choisir(Choice::Activate(bouclier, 0));
    assert_eq!(t.g.player(P1).resource_points, 1);
    assert_eq!(t.g.obj(bouclier).zone, Zone::Graveyard);
    assert!(t.g.player(P1).chest.is_none());
}

#[test]
fn cr_5_1_3b_cout_additionnel_optionnel_paye() {
    let mut t = partie(
        &[
            ("Essai Rage", R),
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
    let rage = t.en_main(P0, "Essai Rage", R);
    t.choisir(Choice::Play(rage));
    assert_eq!(t.kind(), DecisionKind::OptionalCost);
    t.choisir(Choice::Yes);
    // Coût 0 + 1 optionnel : il faut pitcher.
    assert_eq!(t.kind(), DecisionKind::Pitch);
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 15); // 3 + 2
}

#[test]
fn cr_5_1_3b_cout_additionnel_optionnel_refuse() {
    let mut t = partie(
        &[
            ("Essai Rage", R),
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
    let rage = t.en_main(P0, "Essai Rage", R);
    t.choisir(Choice::Play(rage));
    t.choisir(Choice::No);
    // Coût 0 : rien à pitcher, la partie avance jusqu'à la défense.
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.player(P0).hand.len(), 3);
}

#[test]
fn cr_1_7_5_mode_choisi_et_cible_unique() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Dilemme", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    let d = t.en_main(P0, "Essai Dilemme", R);
    t.choisir(Choice::Play(d));
    assert_eq!(t.kind(), DecisionKind::Mode);
    t.choisir(Choice::Mode(1)); // go again
                                // Une seule cible légale (l'attaque active) : choisie automatiquement.
                                // Plus personne n'a rien à jouer : la partie avance jusqu'à la fin du tour.
    assert_eq!(t.kind(), DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 16);
    // Le go again donné par Dilemme a rendu un point d'action (perdu en fin de tour).
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn cr_5_1_9_cout_d_effet_defausser_au_hasard() {
    let mut t = partie(
        &[
            ("Essai Sacrifice", R),
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
    let s = t.en_main(P0, "Essai Sacrifice", R);
    t.choisir(Choice::Play(s));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    // Coût d'effet : une carte au hasard parmi les 2 Remparts restants.
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
    assert_eq!(t.g.player(P0).hand.len(), 1);
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 13);
}

#[test]
fn cr_1_7_4i_declenchement_hors_arene_quand_la_carte_va_au_cimetiere() {
    let mut t = attaque_frappe(&[("Essai Sirène", B), ("Essai Rempart", B)]);
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Defend(t.en_main(P1, "Essai Sirène", B))); // défense 2
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    // 4 - 2 = 2 dégâts, puis la Sirène va au cimetière : +1 PV.
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn cr_7_0_5a_declenchement_quand_la_carte_defend() {
    let mut t = attaque_frappe(&[("Essai Vigie", B), ("Essai Rempart", B)]);
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Defend(t.en_main(P1, "Essai Vigie", B))); // défense 3
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 21); // 20 - 1 + 2
}

#[test]
fn cr_1_8_6_choix_pendant_la_resolution() {
    let mut t = partie(
        &[
            ("Essai Tri", B),
            ("Essai Frappe", R),
            ("Essai Frappe", Y),
            ("Essai Rempart", B),
        ],
        &[("Essai Assaut", R), ("Essai Assaut", Y)],
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let tri = t.en_main(P0, "Essai Tri", B);
    let fr = t.en_main(P0, "Essai Frappe", R);
    let fy = t.en_main(P0, "Essai Frappe", Y);
    t.choisir(Choice::Play(tri));
    assert_eq!(t.kind(), DecisionKind::Effect);
    t.choisir(Choice::Object(fr));
    assert_eq!(t.kind(), DecisionKind::Effect);
    t.choisir(Choice::Object(fy));
    // « jusqu'à 2 » atteint : on pioche 2 cartes.
    let noms: Vec<String> =
        t.g.player(P0)
            .hand
            .iter()
            .map(|&c| t.g.def(c).name.clone())
            .collect();
    assert!(noms.contains(&"Essai Assaut".to_string()));
    assert_eq!(t.g.player(P0).hand.len(), 3);
    assert_eq!(t.g.obj(fr).zone, Zone::Deck);
    // Go again.
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn cr_1_7_4_effet_statique_du_heros_deuxieme_attaque() {
    let mut t = partie_heros(
        ["Essai Ninja", "Essai Héros"],
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
    t.choisir(Choice::Play(t.en_main(P0, "Essai Assaut", R)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    let f = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(f));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    assert_eq!(t.g.power(f), 5); // 4 + 1 : deuxième attaque du tour
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 12); // 3 + 5
}

#[test]
fn cr_6_4_prevention_des_prochains_degats() {
    let mut t = attaque_frappe(&[("Essai Garde", Y), ("Essai Rempart", B)]);
    assert_eq!(t.joueur(), P1);
    t.choisir(Choice::Play(t.en_main(P1, "Essai Garde", Y)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 18); // 4 - 2 prévenus
}

#[test]
fn cr_8_5_3_degats_arcaniques_cible_au_choix_et_journal_du_tour() {
    let mut t = partie(
        &[
            ("Essai Éclair", R),
            ("Essai Frappe", B),
            ("Essai Polarité", R),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    t.choisir(Choice::Play(t.en_main(P0, "Essai Éclair", R)));
    // « target hero » : les deux héros sont des cibles légales.
    assert_eq!(t.kind(), DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(P0)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    // CR 5.1.10 : P0 récupère la priorité ; Éclair n'est pas encore résolu.
    assert_eq!(t.vie(P0), 20);
    t.choisir(Choice::Pass);
    assert_eq!(t.vie(P0), 17);
    // « If you've been dealt arcane damage this turn, instead gain 4 ».
    t.choisir(Choice::Play(t.en_main(P0, "Essai Polarité", R)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P0), 21);
}

#[test]
fn cr_6_6_6b_le_joueur_choisit_l_ordre_de_ses_declenchements() {
    let mut t = partie(
        &[
            ("Essai Aura", Y),
            ("Essai Présage", Y),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let aura = t.en_main(P0, "Essai Aura", Y);
    let presage = t.en_main(P0, "Essai Présage", Y);
    t.choisir(Choice::Play(aura));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.choisir(Choice::Play(presage));
    assert_eq!(t.g.obj(aura).zone, Zone::Permanent);
    assert_eq!(t.g.obj(presage).zone, Zone::Permanent);
    t.finir_le_tour();
    t.finir_le_tour();
    // Début de la phase d'action de P0 : deux déclenchements différents.
    assert_eq!(t.kind(), DecisionKind::TriggerOrder);
    assert_eq!(t.joueur(), P0);
    let main_avant = t.g.player(P0).hand.len();
    let premier = t.options()[0];
    t.choisir(premier);
    // Les deux déclenchements se résolvent ; P0 n'a ensuite rien à jouer.
    assert_eq!(t.kind(), DecisionKind::Arsenal);
    assert_eq!(t.vie(P0), 23);
    assert_eq!(t.g.obj(aura).zone, Zone::Graveyard);
    assert_eq!(t.g.obj(presage).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).hand.len(), main_avant + 1);
}

/// P0 attaque avec Essai Frappe rouge (puissance 4), pitch la Frappe bleue.
fn attaque_frappe(main1: &[(&str, Option<fab_engine::card::Color>)]) -> Partie {
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
        &[],
    );
    let frappe = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(frappe));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t
}
