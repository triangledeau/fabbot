//! Tests des vraies cartes génériques des decks Hala et Dorinthea : un test
//! par phrase du texte de chaque carte, plus des cas limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, ObjectId, PreventRef, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);

fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..100 {
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

fn jouer(t: &mut Partie, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(t.joueur(), nom, couleur);
    t.choisir(Choice::Play(c));
    c
}

fn pitcher(t: &mut Partie, nom: &str, couleur: Option<Color>) {
    let c = t.en_main(t.joueur(), nom, couleur);
    t.choisir(Choice::Pitch(c));
}

fn defendre_avec(t: &mut Partie, cartes: &[(&str, Option<Color>)]) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Defend);
    for &(n, c) in cartes {
        let x = t.en_main(P1, n, c);
        t.choisir(Choice::Defend(x));
    }
    if t.kind() == DecisionKind::Defend {
        t.choisir(Choice::Pass);
    }
}

fn fin_du_combat(t: &mut Partie) {
    passer_jusqua_que(t, |t| !t.g.combat.open);
}

/// P0 (Essai Héros) commence avec `main0`, P1 avec `main1`.
fn duel(main0: &[(&str, Option<Color>)], main1: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        ["Essai Héros", "Essai Héros"],
        main0,
        RIEN,
        main1,
        RIEN,
        &[],
        &[],
    )
}

#[test]
fn attaques_sans_texte_brutal_assault_et_raging_onslaught() {
    for (nom, c, puissance) in [
        ("Brutal Assault", R, 6),
        ("Brutal Assault", B, 4),
        ("Raging Onslaught", R, 7),
        ("Raging Onslaught", B, 5),
    ] {
        let mut t = duel(&[(nom, c), RAMPART, RAMPART, RAMPART], &[RAMPART; 4]);
        jouer(&mut t, nom, c);
        pitcher(&mut t, "Essai Rempart", B);
        defendre_avec(&mut t, &[]);
        fin_du_combat(&mut t);
        assert_eq!(t.vie(P1), 20 - puissance, "{nom}");
    }
}

#[test]
fn arcane_polarity_gagne_1_sans_degats_arcaniques() {
    let mut t = duel(
        &[("Arcane Polarity", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    t.g.player_mut(P0).life = 15;
    t.g.refresh_priority();
    jouer(&mut t, "Arcane Polarity", R);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.vie(P0), 16);
}

#[test]
fn arcane_polarity_gagne_4_3_2_apres_des_degats_arcaniques() {
    for (c, gain) in [(R, 4), (Y, 3), (B, 2)] {
        let mut t = duel(
            &[("Essai Éclair", R), RAMPART, RAMPART, RAMPART],
            &[("Arcane Polarity", c), RAMPART, RAMPART, RAMPART],
        );
        jouer(&mut t, "Essai Éclair", R);
        // Cible : le héros adverse.
        t.choisir(Choice::Target(Target::Player(P1)));
        pitcher(&mut t, "Essai Rempart", B);
        // P1 répond après les dégâts (pendant le même tour).
        let ap = t.en_main(P1, "Arcane Polarity", c);
        passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(ap)) && t.vie(P1) == 17);
        t.choisir(Choice::Play(ap));
        passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
        assert_eq!(t.vie(P1), 17 + gain);
    }
}

#[test]
fn chest_puff_si_on_paie_garde_7() {
    let mut t = duel(
        &[("Chest Puff", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    jouer(&mut t, "Chest Puff", R);
    pitcher(&mut t, "Essai Rempart", B);
    // Il reste 1 ressource : le joueur choisit de payer.
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Yes);
    assert_eq!(t.g.player(P0).resource_points, 0);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 13);
}

#[test]
fn chest_puff_sans_payer_perd_1() {
    let mut t = duel(
        &[("Chest Puff", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    jouer(&mut t, "Chest Puff", R);
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 14);
}

#[test]
fn look_tuff_sans_ressource_perd_1_sans_question() {
    let mut t = duel(
        &[("Look Tuff", R), RAMPART, ("Essai Joyau", None), RAMPART],
        &[RAMPART; 4],
    );
    jouer(&mut t, "Look Tuff", R);
    pitcher(&mut t, "Essai Rempart", B);
    // Il reste 2 ressources en main (Rempart, Joyau) : question posée.
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Yes);
    // Payer : il faut pitcher une carte.
    assert_eq!(t.kind(), DecisionKind::Pitch);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 12);
}

#[test]
fn look_tuff_main_vide_perd_1() {
    let mut t = duel(&[("Look Tuff", R), RAMPART], &[RAMPART; 4]);
    // Main de départ : Look Tuff, Rempart, et 2 cartes de remplissage
    // (Rempart). On les retire pour qu'il ne reste rien à pitcher.
    let main: Vec<_> = t.g.player(P0).hand.clone();
    for c in main.into_iter().skip(2) {
        t.g.put_on_bottom(c);
    }
    t.g.refresh_priority();
    jouer(&mut t, "Look Tuff", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 13, "8 - 1");
}

#[test]
fn fyendal_en_attaque_gagne_1_si_moins_de_vie() {
    let mut t = duel(
        &[("Fyendal's Fighting Spirit", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    t.g.player_mut(P0).life = 10;
    t.g.refresh_priority();
    jouer(&mut t, "Fyendal's Fighting Spirit", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P0), 11);
    assert_eq!(t.vie(P1), 13);
}

#[test]
fn fyendal_en_attaque_rien_si_autant_de_vie() {
    let mut t = duel(
        &[("Fyendal's Fighting Spirit", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    jouer(&mut t, "Fyendal's Fighting Spirit", B);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P0), 20);
    assert_eq!(t.vie(P1), 15);
}

#[test]
fn fyendal_en_defense_gagne_1_si_moins_de_vie() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[("Fyendal's Fighting Spirit", Y), RAMPART, RAMPART, RAMPART],
    );
    t.g.player_mut(P1).life = 12;
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[("Fyendal's Fighting Spirit", Y)]);
    fin_du_combat(&mut t);
    // 12 - (4 - 2) + 1.
    assert_eq!(t.vie(P1), 11);
}

#[test]
fn humble_le_heros_touche_perd_ses_capacites_jusqu_a_la_fin_de_son_prochain_tour() {
    let mut t = partie_vraie(
        ["Essai Héros", "Hala"],
        &[("Humble", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Humble", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 14);
    let hala = t.g.player(P1).hero;
    assert!(t.g.lost_abilities(hala));
    t.finir_le_tour();
    // Tour de P1 : Hala ne peut pas aiguiser.
    assert_eq!(t.g.turn_player, P1);
    assert!(!t.peut(Choice::Activate(hala, 0)));
    t.finir_le_tour();
    t.finir_le_tour();
    // Tour suivant de P1 : capacité rendue.
    assert!(!t.g.lost_abilities(hala));
}

#[test]
fn humble_bloque_rien() {
    let mut t = duel(&[("Humble", B), RAMPART, RAMPART, RAMPART], &[RAMPART; 4]);
    jouer(&mut t, "Humble", B);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[RAMPART]);
    fin_du_combat(&mut t);
    let heros = t.g.player(P1).hero;
    assert!(!t.g.lost_abilities(heros));
}

#[test]
fn sirens_gagne_1_quand_elle_va_au_cimetiere() {
    let mut t = duel(
        &[("Sirens of Safe Harbor", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    t.g.player_mut(P0).life = 10;
    t.g.refresh_priority();
    jouer(&mut t, "Sirens of Safe Harbor", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P0), 11);
}

#[test]
fn sirens_en_defense_gagne_1_pour_son_proprietaire() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[("Sirens of Safe Harbor", B), RAMPART, RAMPART, RAMPART],
    );
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[("Sirens of Safe Harbor", B)]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 19, "4 - 2 de défense, +1");
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn sirens_pitchee_ne_va_pas_au_cimetiere() {
    let mut t = duel(
        &[
            ("Essai Frappe", R),
            ("Sirens of Safe Harbor", B),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
    );
    t.g.player_mut(P0).life = 10;
    t.g.refresh_priority();
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Sirens of Safe Harbor", B);
    t.finir_le_tour();
    assert_eq!(t.vie(P0), 10);
}

#[test]
fn lay_low_injouable_si_on_est_marque() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[("Lay Low", Y), RAMPART, RAMPART, RAMPART],
    );
    t.g.mark(P1);
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    let ll = t.en_main(P1, "Lay Low", Y);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(ll)) || !t.g.combat.open);
    assert!(!t.g.combat.open);
}

#[test]
fn lay_low_attaquant_marque_prochaine_attaque_moins_1() {
    let mut t = duel(
        &[("Essai Assaut", R), ("Essai Frappe", R), RAMPART, RAMPART],
        &[("Lay Low", Y), RAMPART, RAMPART, RAMPART],
    );
    t.g.mark(P0);
    jouer(&mut t, "Essai Assaut", R);
    defendre_avec(&mut t, &[]);
    let ll = t.en_main(P1, "Lay Low", Y);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(ll)));
    t.choisir(Choice::Play(ll));
    fin_du_combat(&mut t);
    // Défense 3 contre 3 : rien. Puis l'attaque suivante (go again) a -1.
    assert_eq!(t.vie(P1), 20);
    let f = jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    assert_eq!(t.g.power(f), 3);
}

#[test]
fn lay_low_attaquant_non_marque_rien() {
    let mut t = duel(
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        &[("Lay Low", Y), RAMPART, RAMPART, RAMPART],
    );
    jouer(&mut t, "Essai Assaut", R);
    defendre_avec(&mut t, &[]);
    let ll = t.en_main(P1, "Lay Low", Y);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(ll)));
    t.choisir(Choice::Play(ll));
    fin_du_combat(&mut t);
    assert!(t.g.next_attack_effects.is_empty());
}

#[test]
fn on_the_horizon_bloc_qui_defend_et_se_declenche() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[("On the Horizon", R), RAMPART, RAMPART, RAMPART],
    );
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    let oh = t.en_main(P1, "On the Horizon", R);
    assert!(t.g.def(oh).cost.is_none(), "un bloc n'a pas de coût");
    defendre_avec(&mut t, &[("On the Horizon", R)]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 20, "défense 4");
    // Regarder ne pioche pas et ne déplace rien.
    assert_eq!(t.g.player(P1).deck.len(), 36);
    let horizon =
        t.g.turn_events()
            .iter()
            .any(|e| matches!(e, fab_engine::Event::Defend { card, .. } if *card == oh));
    assert!(horizon);
}

#[test]
fn sift_met_jusqu_a_4_cartes_sous_le_deck_puis_pioche_autant() {
    let mut t = duel(
        &[
            ("Sift", R),
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            RAMPART,
        ],
        &[RAMPART; 4],
    );
    jouer(&mut t, "Sift", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    let f1 = t.en_main(P0, "Essai Frappe", R);
    let f2 = t.en_main(P0, "Essai Frappe", B);
    t.choisir(Choice::Object(f1));
    t.choisir(Choice::Object(f2));
    t.choisir(Choice::Pass);
    assert_eq!(t.g.obj(f1).zone, Zone::Deck);
    assert_eq!(t.g.player(P0).deck[0], f2, "sous le deck");
    assert_eq!(t.g.player(P0).hand.len(), 3, "2 piochées");
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn sift_bleu_au_plus_2_cartes() {
    let mut t = duel(&[("Sift", B), RAMPART, RAMPART, RAMPART], &[RAMPART; 4]);
    jouer(&mut t, "Sift", B);
    for _ in 0..2 {
        passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
        let c = t.g.player(P0).hand[0];
        t.choisir(Choice::Object(c));
    }
    // Plus de question après 2 cartes.
    assert_ne!(t.kind(), DecisionKind::Effect);
    assert_eq!(t.g.player(P0).hand.len(), 3);
}

#[test]
fn nullrune_arcane_barrier_1() {
    for casque in [
        "Nullrune Hood",
        "Nullrune Robe",
        "Nullrune Gloves",
        "Nullrune Boots",
    ] {
        let mut t = partie_vraie(
            ["Essai Héros", "Essai Héros"],
            &[("Essai Éclair", R), RAMPART, RAMPART, RAMPART],
            RIEN,
            &[RAMPART; 4],
            RIEN,
            &[],
            &[casque],
        );
        let n = t.equipement(P1, casque);
        assert_eq!(t.g.defense(n), 0);
        jouer(&mut t, "Essai Éclair", R);
        t.choisir(Choice::Target(Target::Player(P1)));
        pitcher(&mut t, "Essai Rempart", B);
        passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
        t.choisir(Choice::Prevent(PreventRef::ArcaneBarrier(n)));
        pitcher(&mut t, "Essai Rempart", B);
        assert_eq!(t.vie(P1), 18, "{casque} : 3 - 1");
    }
}

#[test]
fn blade_beckoner_boots_plus_1_contre_une_attaque_d_arme() {
    let mut t = partie_vraie(
        ["Hala", "Essai Héros"],
        &[RAMPART; 4],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &["Zenith Blade"],
        &["Blade Beckoner Boots"],
    );
    let bottes = t.equipement(P1, "Blade Beckoner Boots");
    let epee = t.equipement(P0, "Zenith Blade");
    t.choisir(Choice::Activate(epee, 0));
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    t.choisir(Choice::Defend(bottes));
    t.choisir(Choice::Pass);
    fin_du_combat(&mut t);
    // 3 contre 2 : 1 dégât. Guardwell : -1 compteur par point de défense
    // (2 au moment de la fermeture) -> défense 0 ensuite.
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.defense(bottes), 0);
    assert_eq!(t.g.obj(bottes).zone, Zone::Legs, "guardwell ne détruit pas");
}

#[test]
fn blade_beckoner_boots_contre_une_attaque_de_carte_pas_de_bonus() {
    let mut t = partie_vraie(
        ["Essai Héros", "Essai Héros"],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &["Blade Beckoner Boots"],
    );
    let bottes = t.equipement(P1, "Blade Beckoner Boots");
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    t.choisir(Choice::Defend(bottes));
    t.choisir(Choice::Pass);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
}
