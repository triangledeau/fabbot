//! Tests des vraies cartes Warrior (decks Hala et Dorinthea) : un test par
//! phrase du texte de chaque carte, plus des cas limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Keywords, ObjectId, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);

/// Passe (priorité, défense, arsenal) jusqu'à ce que `cond` soit vraie.
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

/// Passe jusqu'à ce que le choix soit proposé.
fn passer_jusqua_option(t: &mut Partie, c: Choice) {
    passer_jusqua_que(t, |t| t.peut(c));
}

/// Joue une carte de la main de P0 ou P1 (selon qui a la décision).
fn jouer(t: &mut Partie, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(t.joueur(), nom, couleur);
    t.choisir(Choice::Play(c));
    c
}

/// Pitche une carte de la main du joueur qui doit payer.
fn pitcher(t: &mut Partie, nom: &str, couleur: Option<Color>) {
    let c = t.en_main(t.joueur(), nom, couleur);
    t.choisir(Choice::Pitch(c));
}

/// Les deux joueurs passent jusqu'à ce que la pile soit vide.
fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| t.g.stack.is_empty());
}

/// La dernière attaque-proxy créée pour cette arme (pile, file ou chaîne).
fn derniere_proxy(t: &Partie, arme: ObjectId) -> ObjectId {
    let i =
        t.g.objects
            .iter()
            .rposition(|o| o.proxy_of == Some(arme))
            .expect("une attaque de cette arme");
    ObjectId(i as u32)
}

fn hala(main0: &[(&str, Option<Color>)], arena0: &[&str]) -> Partie {
    partie_vraie(
        ["Hala", "Essai Héros"],
        main0,
        RIEN,
        RIEN,
        RIEN,
        arena0,
        &[],
    )
}

// ---------------------------------------------------------------------------
// Hala
// ---------------------------------------------------------------------------

#[test]
fn hala_aiguise_une_epee_pour_trois_ressources_avec_go_again() {
    let mut t = hala(&[RAMPART, RAMPART, RAMPART, RAMPART], &["Zenith Blade"]);
    let heros = t.g.player(P0).hero;
    let epee = t.equipement(P0, "Zenith Blade");
    t.choisir(Choice::Activate(heros, 0));
    pitcher(&mut t, "Essai Rempart", B);
    assert!(t.g.obj(heros).tapped, "coût {{t}}");
    resoudre(&mut t);
    // CR 8.5.58 : un compteur +1{p}.
    assert_eq!(t.g.obj(epee).power_counters, 1);
    assert_eq!(t.g.power(epee), 4);
    // Go again : le point d'action dépensé est rendu.
    assert_eq!(t.g.player(P0).action_points, 1);
    assert!(t.g.sharpened_this_turn(epee));
}

#[test]
fn hala_sharpen_les_compteurs_partent_au_debut_de_la_phase_de_fin() {
    let mut t = hala(&[RAMPART, RAMPART, RAMPART, RAMPART], &["Zenith Blade"]);
    let heros = t.g.player(P0).hero;
    let epee = t.equipement(P0, "Zenith Blade");
    t.choisir(Choice::Activate(heros, 0));
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    t.finir_le_tour();
    assert_eq!(t.g.obj(epee).power_counters, 0);
    assert_eq!(t.g.power(epee), 3);
}

#[test]
fn hala_ne_peut_pas_aiguiser_sans_epee() {
    let t = hala(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &["Decimator Great Axe"],
    );
    let heros = t.g.player(P0).hero;
    assert!(!t.peut(Choice::Activate(heros, 0)));
}

// ---------------------------------------------------------------------------
// Zenith Blade, Edict of Steel, Flurry
// ---------------------------------------------------------------------------

#[test]
fn zenith_blade_premiere_attaque_go_again_si_aiguisee() {
    let mut t = hala(
        &[("Edict of Steel", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    let epee = t.equipement(P0, "Zenith Blade");
    jouer(&mut t, "Edict of Steel", R);
    resoudre(&mut t);
    t.choisir(Choice::Activate(epee, 0));
    pitcher(&mut t, "Essai Rempart", B);
    let proxy = derniere_proxy(&t, epee);
    assert!(t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    assert_eq!(t.g.power(proxy), 4, "3 + compteur");
}

#[test]
fn zenith_blade_sans_sharpen_pas_de_go_again() {
    let mut t = hala(&[RAMPART, RAMPART, RAMPART, RAMPART], &["Zenith Blade"]);
    let epee = t.equipement(P0, "Zenith Blade");
    t.choisir(Choice::Activate(epee, 0));
    pitcher(&mut t, "Essai Rempart", B);
    let proxy = derniere_proxy(&t, epee);
    assert!(!t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    assert_eq!(t.g.power(proxy), 3);
}

#[test]
fn edict_of_steel_rouge_cree_un_flurry_et_a_go_again() {
    let mut t = hala(
        &[("Edict of Steel", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Edict of Steel", R);
    resoudre(&mut t);
    let flurry: Vec<_> =
        t.g.permanents
            .iter()
            .filter(|&&o| t.g.def(o).name == "Flurry")
            .collect();
    assert_eq!(flurry.len(), 1);
    assert_eq!(t.g.obj(*flurry[0]).controller, P0);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn edict_of_steel_bleu_sans_trois_compteurs_pas_de_flurry() {
    let mut t = hala(
        &[("Edict of Steel", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Edict of Steel", B);
    resoudre(&mut t);
    assert!(t.g.permanents.is_empty());
    let epee = t.equipement(P0, "Zenith Blade");
    assert_eq!(t.g.obj(epee).power_counters, 1);
}

#[test]
fn edict_of_steel_jaune_avec_deux_compteurs_cree_un_flurry() {
    let mut t = hala(
        &[
            ("Edict of Steel", R),
            ("Edict of Steel", Y),
            RAMPART,
            RAMPART,
        ],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Edict of Steel", R);
    resoudre(&mut t);
    jouer(&mut t, "Edict of Steel", Y);
    resoudre(&mut t);
    let n =
        t.g.permanents
            .iter()
            .filter(|&&o| t.g.def(o).name == "Flurry")
            .count();
    assert_eq!(n, 2);
}

#[test]
fn flurry_detruit_quand_on_active_une_attaque_d_arme_et_permet_deux_attaques() {
    let mut t = hala(
        &[("Edict of Steel", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    let epee = t.equipement(P0, "Zenith Blade");
    jouer(&mut t, "Edict of Steel", R);
    resoudre(&mut t);
    let flurry = t.g.permanents[0];
    t.choisir(Choice::Activate(epee, 0));
    pitcher(&mut t, "Essai Rempart", B);
    // Le déclenchement de Flurry se résout avant l'attaque.
    passer_jusqua_que(&mut t, |t| t.g.obj(flurry).zone == Zone::Gone);
    // Après la première attaque (go again grâce au sharpen), une deuxième
    // attaque avec la même arme est possible.
    passer_jusqua_option(&mut t, Choice::Activate(epee, 0));
    t.choisir(Choice::Activate(epee, 0));
    let proxy = derniere_proxy(&t, epee);
    // Deuxième attaque de l'arme : pas de go again de Zenith Blade.
    assert!(!t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    // Pas de troisième attaque.
    resoudre(&mut t);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert!(!t.peut(Choice::Activate(epee, 0)));
}

#[test]
fn sans_flurry_une_seule_attaque_par_tour() {
    let mut t = hala(
        &[("Edict of Steel", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    let epee = t.equipement(P0, "Zenith Blade");
    jouer(&mut t, "Edict of Steel", B);
    resoudre(&mut t);
    t.choisir(Choice::Activate(epee, 0));
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open && t.joueur() == P0);
    // L'épée aiguisée a eu go again : il reste un point d'action, mais la
    // limite « Once per Turn » est atteinte.
    assert_eq!(t.g.player(P0).action_points, 1);
    t.g.player_mut(P0).resource_points = 3;
    t.g.refresh_priority();
    assert!(!t.peut(Choice::Activate(epee, 0)));
}

/// P1 défend avec ces cartes de sa main (déclarées une par une), puis passe.
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

/// P0 attaque avec son arme (en pitchant les cartes données).
fn attaquer(t: &mut Partie, arme: &str, pitch: &[(&str, Option<Color>)]) -> ObjectId {
    let w = t.equipement(P0, arme);
    t.choisir(Choice::Activate(w, 0));
    for &(n, c) in pitch {
        pitcher(t, n, c);
    }
    w
}

fn nombre_de_flurry(t: &Partie, p: fab_engine::PlayerId) -> usize {
    t.g.permanents
        .iter()
        .filter(|&&o| t.g.def(o).name == "Flurry" && t.g.obj(o).controller == p)
        .count()
}

// ---------------------------------------------------------------------------
// Cut n' Carve, Sharp Incline, Swordmaster's Path
// ---------------------------------------------------------------------------

#[test]
fn cut_n_carve_rouge_la_prochaine_attaque_de_l_epee_a_dominate() {
    let mut t = hala(
        &[("Cut n' Carve", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Cut n' Carve", R);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    let proxy = derniere_proxy(&t, epee);
    assert!(t.g.keywords(proxy).has(Keywords::DOMINATE));
    assert_eq!(t.g.obj(epee).power_counters, 1, "sharpen");
}

#[test]
fn cut_n_carve_bleu_sans_trois_compteurs_pas_de_dominate() {
    let mut t = hala(
        &[("Cut n' Carve", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Cut n' Carve", B);
    resoudre(&mut t);
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    let proxy = derniere_proxy(&t, epee);
    assert!(!t.g.keywords(proxy).has(Keywords::DOMINATE));
}

#[test]
fn cut_n_carve_dominate_seulement_pour_l_attaque_de_cette_epee() {
    let mut t = hala(
        &[("Cut n' Carve", R), ("Essai Frappe", R), RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Cut n' Carve", R);
    resoudre(&mut t);
    let f = jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    assert!(!t.g.keywords(f).has(Keywords::DOMINATE));
    // L'effet attend toujours l'attaque de l'épée.
    assert_eq!(t.g.next_attack_effects.len(), 1);
}

#[test]
fn sharp_incline_rouge_la_prochaine_attaque_de_l_epee_coute_r_de_moins() {
    let mut t = hala(
        &[("Sharp Incline", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Sharp Incline", R);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    let epee = t.equipement(P0, "Zenith Blade");
    assert_eq!(t.g.activation_cost(epee, 0), 0);
    t.choisir(Choice::Activate(epee, 0));
    // Aucune carte à pitcher : l'attaque est gratuite.
    assert_eq!(t.g.player(P0).hand.len(), 3);
    // La réduction est utilisée.
    assert_eq!(t.g.activation_cost(epee, 0), 1);
}

#[test]
fn sharp_incline_jaune_sans_deux_compteurs_pas_de_reduction() {
    let mut t = hala(
        &[("Sharp Incline", Y), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Sharp Incline", Y);
    resoudre(&mut t);
    let epee = t.equipement(P0, "Zenith Blade");
    assert_eq!(t.g.obj(epee).power_counters, 1);
    assert_eq!(t.g.activation_cost(epee, 0), 1);
}

#[test]
fn swordmasters_path_le_prochain_sharpen_compte_double() {
    let mut t = hala(
        &[
            ("Swordmaster's Path", R),
            ("Edict of Steel", Y),
            ("Edict of Steel", R),
            RAMPART,
        ],
        &["Zenith Blade"],
    );
    let epee = t.equipement(P0, "Zenith Blade");
    jouer(&mut t, "Swordmaster's Path", R);
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    // Edict jaune : sharpen deux fois (2 compteurs) -> Flurry.
    jouer(&mut t, "Edict of Steel", Y);
    resoudre(&mut t);
    assert_eq!(t.g.obj(epee).power_counters, 2);
    assert_eq!(nombre_de_flurry(&t, P0), 1);
    // Seulement la prochaine fois : le sharpen suivant est normal.
    jouer(&mut t, "Edict of Steel", R);
    resoudre(&mut t);
    assert_eq!(t.g.obj(epee).power_counters, 3);
}

#[test]
fn swordmasters_path_la_prochaine_attaque_d_epee_gagne_3() {
    let mut t = hala(
        &[("Swordmaster's Path", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Swordmaster's Path", R);
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    // Il reste 2 ressources : l'épée attaque sans pitcher.
    let epee = attaquer(&mut t, "Zenith Blade", &[]);
    assert_eq!(t.g.power(derniere_proxy(&t, epee)), 6);
}

#[test]
fn swordmasters_path_bleu_donne_1_et_ne_touche_pas_une_attaque_de_carte() {
    let mut t = hala(
        &[
            ("Swordmaster's Path", B),
            ("Essai Frappe", R),
            RAMPART,
            RAMPART,
        ],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Swordmaster's Path", B);
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    let f = jouer(&mut t, "Essai Frappe", R);
    assert_eq!(t.g.power(f), 4, "pas une attaque d'épée");
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.next_attack_effects.len(), 1);
}

// ---------------------------------------------------------------------------
// Réactions d'attaque
// ---------------------------------------------------------------------------

#[test]
fn display_of_craftsmanship_plus_4_et_compteur_si_l_arme_est_aiguisee() {
    let mut t = hala(
        &[
            ("Edict of Steel", B),
            ("Display of Craftsmanship", R),
            RAMPART,
            RAMPART,
        ],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Edict of Steel", B);
    resoudre(&mut t);
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    let d = t.en_main(P0, "Display of Craftsmanship", R);
    passer_jusqua_option(&mut t, Choice::Play(d));
    t.choisir(Choice::Play(d));
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    // 3 + 1 (sharpen) + 4 + 1 (nouveau compteur sur l'arme) = 9.
    assert_eq!(t.g.obj(epee).power_counters, 2);
    assert_eq!(t.vie(P1), 11);
    t.finir_le_tour();
    assert_eq!(t.g.obj(epee).power_counters, 0, "tous les +1 retirés");
}

#[test]
fn display_of_craftsmanship_sans_sharpen_pas_de_compteur() {
    let mut t = hala(
        &[("Display of Craftsmanship", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    let d = t.en_main(P0, "Display of Craftsmanship", B);
    passer_jusqua_option(&mut t, Choice::Play(d));
    t.choisir(Choice::Play(d));
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(epee).power_counters, 0);
    assert_eq!(t.vie(P1), 15, "3 + 2");
}

#[test]
fn display_of_craftsmanship_ne_cible_pas_une_attaque_de_carte() {
    let mut t = hala(
        &[
            ("Essai Frappe", R),
            ("Display of Craftsmanship", B),
            RAMPART,
            RAMPART,
        ],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    let d = t.en_main(P0, "Display of Craftsmanship", B);
    // La carte n'est jamais jouable pendant le combat.
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(d)) || !t.g.combat.open);
    assert!(!t.g.combat.open);
    assert_eq!(t.g.obj(d).zone, Zone::Hand);
}

#[test]
fn ironsong_response_avec_reprise_plus_3() {
    let mut t = hala(
        &[("Ironsong Response", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[RAMPART]);
    jouer(&mut t, "Ironsong Response", R);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 18, "3 + 3 contre 4");
}

#[test]
fn ironsong_response_sans_reprise_rien() {
    let mut t = hala(
        &[("Ironsong Response", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[]);
    jouer(&mut t, "Ironsong Response", R);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn ironsong_response_reprise_avec_un_equipement_seul_ne_compte_pas() {
    let mut t = partie_vraie(
        ["Hala", "Essai Héros"],
        &[("Ironsong Response", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Zenith Blade"],
        &["Essai Casque"],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let casque = t.equipement(P1, "Essai Casque");
    t.choisir(Choice::Defend(casque));
    t.choisir(Choice::Pass);
    jouer(&mut t, "Ironsong Response", B);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 18, "3 contre 1, sans bonus");
}

#[test]
fn overpower_plus_4_sans_reprise_plus_6_avec() {
    // Sans défense : 3 + 4 = 7. Avec un bloc de 4 : 3 + 6 - 4 = 5.
    for (defense, vie) in [(false, 13), (true, 15)] {
        let mut t = hala(
            &[("Overpower", R), RAMPART, RAMPART, RAMPART],
            &["Zenith Blade"],
        );
        attaquer(&mut t, "Zenith Blade", &[RAMPART]);
        defendre_avec(&mut t, if defense { &[RAMPART] } else { &[] });
        jouer(&mut t, "Overpower", R);
        pitcher(&mut t, "Essai Rempart", B);
        passer_jusqua_que(&mut t, |t| !t.g.combat.open);
        assert_eq!(t.vie(P1), vie);
    }
}

#[test]
fn overpower_bleu_plus_2_ou_plus_4() {
    // Sans défense : 3 + 2 = 5. Avec un bloc de 4 : 3 + 4 - 4 = 3.
    for (defense, vie) in [(false, 15), (true, 17)] {
        let mut t = hala(
            &[("Overpower", B), RAMPART, RAMPART, RAMPART],
            &["Zenith Blade"],
        );
        attaquer(&mut t, "Zenith Blade", &[RAMPART]);
        defendre_avec(&mut t, if defense { &[RAMPART] } else { &[] });
        jouer(&mut t, "Overpower", B);
        pitcher(&mut t, "Essai Rempart", B);
        passer_jusqua_que(&mut t, |t| !t.g.combat.open);
        assert_eq!(t.vie(P1), vie);
    }
}

#[test]
fn fatal_engagement_seulement_si_une_carte_d_attaque_defend() {
    let mut t = hala(
        &[("Fatal Engagement", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    // Un bloc qui défend ne suffit pas : la carte n'est jamais jouable.
    defendre_avec(&mut t, &[RAMPART]);
    let fe = t.en_main(P0, "Fatal Engagement", R);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(fe)) || !t.g.combat.open);
    assert!(!t.g.combat.open);

    let mut t = partie_vraie(
        ["Hala", "Essai Héros"],
        &[("Fatal Engagement", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Zenith Blade"],
        &[],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[("Essai Frappe", R)]);
    // Coût 2 : payé avec les ressources restantes.
    jouer(&mut t, "Fatal Engagement", R);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 15, "3 + 5 contre 3");
}

#[test]
fn shove_off_renvoie_une_carte_qui_defend_dans_la_main() {
    let mut t = hala(
        &[("Shove Off", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[RAMPART]);
    let rempart = t.g.combat.links[0].defenders[0];
    jouer(&mut t, "Shove Off", B);
    // Coût 1 : payé avec les ressources restantes du pitch.
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Object(rempart));
    assert_eq!(t.g.obj(rempart).zone, Zone::Hand);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 17, "plus de défense : 3 dégâts");
}

#[test]
fn shove_off_ne_renvoie_pas_un_equipement() {
    let mut t = partie_vraie(
        ["Hala", "Essai Héros"],
        &[("Shove Off", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Zenith Blade"],
        &["Essai Bouclier"],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let bouclier = t.equipement(P1, "Essai Bouclier");
    t.choisir(Choice::Defend(bouclier));
    t.choisir(Choice::Pass);
    let so = t.en_main(P0, "Shove Off", B);
    passer_jusqua_option(&mut t, Choice::Play(so));
    t.choisir(Choice::Play(so));
    // Aucun choix possible : rien ne se passe.
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(bouclier).zone, Zone::Chest);
    assert_eq!(t.vie(P1), 18);
}

// ---------------------------------------------------------------------------
// Actions Warrior
// ---------------------------------------------------------------------------

#[test]
fn sharpen_steel_prochaine_attaque_d_arme_plus_3_et_go_again() {
    let mut t = hala(
        &[("Sharpen Steel", R), ("Essai Frappe", R), RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Sharpen Steel", R);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    // Une attaque de carte ne consomme pas l'effet.
    let f = jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    assert_eq!(t.g.power(f), 4);
    assert_eq!(t.g.next_attack_effects.len(), 1);
}

#[test]
fn sharpen_steel_s_applique_a_l_attaque_de_l_arme() {
    let mut t = hala(
        &[("Sharpen Steel", Y), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Sharpen Steel", Y);
    resoudre(&mut t);
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    assert_eq!(t.g.power(derniere_proxy(&t, epee)), 5);
    // « Sharpen Steel » ne fait pas un sharpen (CR 8.5.58).
    assert!(!t.g.sharpened_this_turn(epee));
}

#[test]
fn hit_and_run_donne_go_again_a_la_prochaine_attaque_d_arme() {
    let mut t = hala(
        &[("Hit and Run", R), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Hit and Run", R);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    // Pas d'attaque d'arme avant : pas de +3.
    assert_eq!(t.g.next_attack_effects.len(), 1);
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    let proxy = derniere_proxy(&t, epee);
    assert!(t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    assert_eq!(t.g.power(proxy), 3);
}

#[test]
fn hit_and_run_apres_une_attaque_d_arme_la_prochaine_attaque_gagne_3() {
    let mut t = hala(
        &[
            ("Edict of Steel", B),
            ("Hit and Run", R),
            ("Essai Frappe", R),
            RAMPART,
        ],
        &["Zenith Blade"],
    );
    // Épée aiguisée : sa première attaque a go again.
    jouer(&mut t, "Edict of Steel", B);
    resoudre(&mut t);
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open && t.joueur() == P0);
    jouer(&mut t, "Hit and Run", R);
    resoudre(&mut t);
    let f = jouer(&mut t, "Essai Frappe", R);
    assert_eq!(t.g.power(f), 4 + 3);
}

#[test]
fn felling_swing_prochaine_attaque_de_hache_plus_6() {
    let mut t = partie_vraie(
        ["Dorinthea", "Essai Héros"],
        &[("Felling Swing", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Decimator Great Axe"],
        &[],
    );
    jouer(&mut t, "Felling Swing", R);
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    let hache = attaquer(&mut t, "Decimator Great Axe", &[RAMPART]);
    assert_eq!(t.g.power(derniere_proxy(&t, hache)), 4 + 6);
}

#[test]
fn felling_swing_ne_touche_pas_une_epee() {
    let mut t = hala(
        &[("Felling Swing", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade"],
    );
    jouer(&mut t, "Felling Swing", B);
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    let epee = attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    assert_eq!(t.g.power(derniere_proxy(&t, epee)), 3);
}

// ---------------------------------------------------------------------------
// Réactions de défense, instants
// ---------------------------------------------------------------------------

#[test]
fn steelblade_shunt_inflige_1_si_elle_defend_une_attaque_d_arme() {
    let mut t = partie_vraie(
        ["Hala", "Hala"],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Steelblade Shunt", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Zenith Blade"],
        &[],
    );
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[]);
    let s = t.en_main(P1, "Steelblade Shunt", R);
    passer_jusqua_option(&mut t, Choice::Play(s));
    t.choisir(Choice::Play(s));
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P0), 19);
    assert_eq!(t.vie(P1), 20, "défense 6");
}

#[test]
fn steelblade_shunt_contre_une_attaque_de_carte_rien() {
    let mut t = partie_vraie(
        ["Essai Héros", "Hala"],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Steelblade Shunt", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    defendre_avec(&mut t, &[]);
    let s = t.en_main(P1, "Steelblade Shunt", B);
    passer_jusqua_option(&mut t, Choice::Play(s));
    t.choisir(Choice::Play(s));
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P0), 20);
    assert_eq!(t.vie(P1), 20, "défense 4 contre 4");
}

#[test]
fn toe_the_line_previent_2_et_cree_un_flurry() {
    let mut t = partie_vraie(
        ["Essai Héros", "Hala"],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Toe the Line", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    let toe = t.en_main(P1, "Toe the Line", R);
    passer_jusqua_option(&mut t, Choice::Play(toe));
    t.choisir(Choice::Play(toe));
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 18, "4 - 2 prévenus");
    assert_eq!(nombre_de_flurry(&t, P1), 1);
}

#[test]
fn toe_the_line_sans_degats_pas_de_flurry() {
    let mut t = partie_vraie(
        ["Hala", "Essai Héros"],
        &[("Toe the Line", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    jouer(&mut t, "Toe the Line", R);
    resoudre(&mut t);
    t.finir_le_tour();
    assert_eq!(nombre_de_flurry(&t, P0), 0);
    assert!(t.g.preventions.is_empty(), "« this turn »");
}

// ---------------------------------------------------------------------------
// Équipements Warrior
// ---------------------------------------------------------------------------

#[test]
fn edge_laden_plate_seulement_apres_avoir_aiguise_une_epee() {
    let mut t = hala(
        &[("Edict of Steel", B), RAMPART, RAMPART, RAMPART],
        &["Zenith Blade", "Edge Laden Plate"],
    );
    let plastron = t.equipement(P0, "Edge Laden Plate");
    assert!(!t.peut(Choice::Activate(plastron, 0)));
    jouer(&mut t, "Edict of Steel", B);
    resoudre(&mut t);
    assert!(t.peut(Choice::Activate(plastron, 0)));
    t.choisir(Choice::Activate(plastron, 0));
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).resource_points, 1);
    assert_eq!(t.g.obj(plastron).zone, Zone::Graveyard);
}

#[test]
fn refraction_bolters_detruites_l_attaque_d_arme_gagne_go_again() {
    let mut t = hala(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &["Zenith Blade", "Refraction Bolters"],
    );
    let bottes = t.equipement(P0, "Refraction Bolters");
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[]);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Yes);
    assert_eq!(t.g.obj(bottes).zone, Zone::Graveyard);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn refraction_bolters_on_peut_refuser() {
    let mut t = hala(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &["Zenith Blade", "Refraction Bolters"],
    );
    let bottes = t.equipement(P0, "Refraction Bolters");
    attaquer(&mut t, "Zenith Blade", &[RAMPART]);
    defendre_avec(&mut t, &[]);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(bottes).zone, Zone::Legs);
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn unity_plus_1_en_defendant_avec_une_carte_de_la_main_puis_temper() {
    let mut t = partie_vraie(
        ["Essai Héros", "Hala"],
        &[("Essai Écrasement", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &["Helm of Unity"],
    );
    jouer(&mut t, "Essai Écrasement", R);
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let casque = t.equipement(P1, "Helm of Unity");
    t.choisir(Choice::Defend(casque));
    let r = t.en_main(P1, "Essai Rempart", B);
    // Dominate : plus d'autre carte possible, la déclaration se termine seule.
    t.choisir(Choice::Defend(r));
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    // 6 contre 2 (casque +1) + 4 : aucun dégât (sans Unity : 1).
    assert_eq!(t.vie(P1), 20);
    // Temper : -1 compteur ; 1 - 1 + 1 (jusqu'à la fin du tour) = 1, pas détruit.
    assert_eq!(t.g.obj(casque).zone, Zone::Head);
    assert_eq!(t.g.defense(casque), 1);
    t.finir_le_tour();
    assert_eq!(t.g.defense(casque), 0);
}

#[test]
fn unity_seul_pas_de_bonus_et_temper_le_detruit() {
    let mut t = partie_vraie(
        ["Essai Héros", "Hala"],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &["Pillar of Unity"],
    );
    jouer(&mut t, "Essai Frappe", R);
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let jambes = t.equipement(P1, "Pillar of Unity");
    t.choisir(Choice::Defend(jambes));
    t.choisir(Choice::Pass);
    assert_eq!(t.g.defense(jambes), 1);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(jambes).zone, Zone::Graveyard);
}

// ---------------------------------------------------------------------------
// Decimator Great Axe, Dorinthea
// ---------------------------------------------------------------------------

fn dorinthea(main0: &[(&str, Option<Color>)], main1: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        ["Dorinthea", "Essai Héros"],
        main0,
        RIEN,
        main1,
        RIEN,
        &["Decimator Great Axe"],
        &["Essai Casque"],
    )
}

#[test]
fn decimator_divise_par_deux_la_defense_de_base_arrondie_au_superieur() {
    let mut t = dorinthea(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
    );
    attaquer(&mut t, "Decimator Great Axe", &[RAMPART]);
    defendre_avec(&mut t, &[("Essai Frappe", R)]);
    // Défense 3 -> 2 (3/2 arrondi au supérieur) : 4 - 2 = 2 dégâts.
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 18);
}

#[test]
fn decimator_cible_au_choix_parmi_les_cartes_qui_defendent() {
    let mut t = dorinthea(
        &[("Felling Swing", B), RAMPART, RAMPART, RAMPART],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
    );
    // Hache à 4 + 4 = 8.
    jouer(&mut t, "Felling Swing", B);
    pitcher(&mut t, "Essai Rempart", B);
    resoudre(&mut t);
    attaquer(&mut t, "Decimator Great Axe", &[RAMPART]);
    defendre_avec(&mut t, &[("Essai Frappe", R), RAMPART]);
    // Le déclenchement ne se produit qu'une fois (première carte), avec une cible au choix.
    assert_eq!(t.kind(), DecisionKind::Target);
    let rempart = t.g.combat.links[0].defenders[1];
    let cible = t
        .options()
        .into_iter()
        .find(|c| matches!(c, Choice::Target(fab_engine::Target::Object(r)) if r.id == rempart))
        .unwrap();
    t.choisir(cible);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    // Rempart 4 -> 2, Frappe 3 : 8 - 5 = 3 dégâts.
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn decimator_defendue_par_un_equipement_seul_ne_declenche_pas() {
    let mut t = dorinthea(&[RAMPART, RAMPART, RAMPART, RAMPART], &[RAMPART; 4]);
    attaquer(&mut t, "Decimator Great Axe", &[RAMPART]);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let casque = t.equipement(P1, "Essai Casque");
    t.choisir(Choice::Defend(casque));
    t.choisir(Choice::Pass);
    assert!(t.g.stack.is_empty() && t.g.pending_triggers.is_empty());
    assert_eq!(t.g.defense(casque), 1);
}

#[test]
fn dorinthea_premiere_touche_d_arme_une_attaque_de_plus() {
    let mut t = dorinthea(
        &[("Hit and Run", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    let hache = t.equipement(P0, "Decimator Great Axe");
    jouer(&mut t, "Hit and Run", B);
    resoudre(&mut t);
    attaquer(&mut t, "Decimator Great Axe", &[RAMPART]);
    defendre_avec(&mut t, &[]);
    // L'attaque touche : Dorinthea permet une attaque de plus avec la hache.
    passer_jusqua_option(&mut t, Choice::Activate(hache, 0));
    assert_eq!(t.vie(P1), 16);
    t.choisir(Choice::Activate(hache, 0));
    pitcher(&mut t, "Essai Rempart", B);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.vie(P1), 12);
    // Pas de troisième attaque (seulement la première touche du tour).
    t.g.player_mut(P0).action_points = 1;
    t.g.player_mut(P0).resource_points = 3;
    t.g.refresh_priority();
    assert!(!t.peut(Choice::Activate(hache, 0)));
}

#[test]
fn dorinthea_une_attaque_bloquee_ne_donne_rien() {
    let mut t = dorinthea(
        &[("Hit and Run", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
    );
    let hache = t.equipement(P0, "Decimator Great Axe");
    jouer(&mut t, "Hit and Run", B);
    resoudre(&mut t);
    attaquer(&mut t, "Decimator Great Axe", &[RAMPART]);
    // Deux blocs : 2 (divisé) + 4, l'attaque ne touche pas.
    defendre_avec(&mut t, &[RAMPART, RAMPART]);
    let premier = t.options()[0];
    t.choisir(premier); // cible de la Decimator
    passer_jusqua_que(&mut t, |t| !t.g.combat.open && t.joueur() == P0);
    assert_eq!(t.vie(P1), 20);
    // Go again (Hit and Run), ressources disponibles, mais pas de 2e attaque.
    assert_eq!(t.g.player(P0).action_points, 1);
    t.g.player_mut(P0).resource_points = 3;
    t.g.refresh_priority();
    assert!(!t.peut(Choice::Activate(hache, 0)));
}
