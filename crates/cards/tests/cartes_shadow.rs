//! Tests des vraies cartes du deck Levia (Shadow, Brute et tokens) : un test
//! par phrase du texte de chaque carte, plus des cas limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Event, Keywords, ObjectId, PlayerId, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const LEVIA: &str = "Levia";
const HEROS: &str = "Essai Héros";
const GATE: &str = "Gate to i'Arathael";
const BLASMOPHET: &str = "Blasmophet, the Insatiable Hunger";
/// Carte de puissance 7 (≥ 6), coût 1.
const GROWL_R: (&str, Option<Color>) = ("Graveling Growl", R);
/// Instant toujours jouable (avec un Rempart à pitcher) : le joueur garde
/// une option, le moteur ne passe pas tout seul à la fin du tour.
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);

/// Passe (priorité, défense, arsenal) jusqu'à ce que `cond` soit vraie.
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

/// Paie le coût en cours en pitchant des Essai Rempart (sinon la première carte).
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

fn jouer_obj(t: &mut Partie, p: PlayerId, c: ObjectId) {
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    payer(t);
}

/// Le joueur joue cette carte de sa main dès qu'il le peut, et paie.
fn jouer(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    jouer_obj(t, p, c);
    c
}

fn jusqua_active(t: &mut Partie, a: ObjectId) {
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(a));
}

/// Joue une carte d'attaque puis passe jusqu'à ce qu'elle soit l'attaque active.
fn attaquer(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = jouer(t, p, nom, couleur);
    jusqua_active(t, c);
    c
}

fn activer(t: &mut Partie, p: PlayerId, src: ObjectId, i: u8) {
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Activate(src, i)));
    t.choisir(Choice::Activate(src, i));
    payer(t);
}

/// Le joueur attaque avec son arme ; renvoie l'attaque-proxy (active).
fn attaquer_arme(t: &mut Partie, p: PlayerId, arme: &str) -> ObjectId {
    let w = t.equipement(p, arme);
    activer(t, p, w, 0);
    let i =
        t.g.objects
            .iter()
            .rposition(|o| o.proxy_of == Some(w))
            .expect("attaque-proxy");
    let proxy = ObjectId(i as u32);
    jusqua_active(t, proxy);
    proxy
}

fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.g.stack.is_empty() && t.g.pending_triggers.is_empty() && t.g.tasks.is_empty()
    });
}

fn fin_du_combat(t: &mut Partie) {
    passer_jusqua_que(t, |t| !t.g.combat.open);
}

/// Le joueur défend avec ces cartes (main ou équipement), puis passe.
fn defendre_avec(t: &mut Partie, p: PlayerId, cartes: &[ObjectId]) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Defend && t.joueur() == p);
    for &x in cartes {
        t.choisir(Choice::Defend(x));
    }
    if t.kind() == DecisionKind::Defend {
        t.choisir(Choice::Pass);
    }
}

/// Crée `n` tokens pour le joueur.
fn creer(t: &mut Partie, p: PlayerId, nom: &str, n: u8) {
    let c = t.g.db.find_name(nom).expect("token codé");
    t.g.create_tokens(p, c, n);
    t.g.refresh_priority();
}

/// Permanents de ce nom contrôlés par le joueur.
fn jetons_obj(t: &Partie, p: PlayerId, nom: &str) -> Vec<ObjectId> {
    t.g.permanents
        .iter()
        .copied()
        .filter(|&o| t.g.obj(o).controller == p && t.g.def(o).name == nom)
        .collect()
}

fn jetons(t: &Partie, p: PlayerId, nom: &str) -> usize {
    jetons_obj(t, p, nom).len()
}

/// Une carte du deck du joueur, par nom et couleur.
fn du_deck(t: &Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    *t.g.player(p)
        .deck
        .iter()
        .find(|&&c| t.g.def(c).name == nom && t.g.def(c).color == couleur)
        .unwrap_or_else(|| panic!("{nom} {couleur:?} absente du deck de {p:?}"))
}

/// Met au cimetière une carte du deck (préparation d'un test).
fn au_cimetiere(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = du_deck(t, p, nom, couleur);
    t.g.put_in_graveyard(c);
    t.g.refresh_priority();
    c
}

/// Bannit (face visible) une carte du deck (préparation d'un test), par
/// l'adversaire `par` (pour ne pas compter comme « you've banished »).
fn bannir_par(
    t: &mut Partie,
    par: PlayerId,
    p: PlayerId,
    nom: &str,
    couleur: Option<Color>,
) -> ObjectId {
    let c = du_deck(t, p, nom, couleur);
    t.g.banish(par, c, false);
    t.g.refresh_priority();
    c
}

/// Avance jusqu'au tour `n`, une fois ses déclenchements de début résolus.
fn debut_du_tour(t: &mut Partie, n: u32) {
    while t.g.turn < n {
        t.finir_le_tour();
    }
    passer_jusqua_que(t, |t| {
        t.g.turn != n || (t.g.stack.is_empty() && t.g.pending_triggers.is_empty())
    });
    assert_eq!(t.g.turn, n);
}

/// Répond aux questions des capacités (`DecisionKind::Effect`) par `c`.
fn repondre(t: &mut Partie, c: Choice) {
    assert_eq!(t.kind(), DecisionKind::Effect, "{:?}", t.noms(&t.options()));
    t.choisir(c);
}

/// Passe jusqu'à une question d'une capacité.
fn jusqua_question(t: &mut Partie) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Effect);
}

/// Levia (P0) contre Essai Héros (P1).
fn levia(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    arena0: &[&str],
) -> Partie {
    partie_vraie([LEVIA, HEROS], main0, suite0, RIEN, RIEN, arena0, &[])
}

/// Essai Héros (P0) attaque Levia (P1) au tour 1 avec Essai Frappe (4{p}).
fn levia_defend(
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena1: &[&str],
) -> (Partie, ObjectId) {
    let mut t = partie_vraie(
        [HEROS, LEVIA],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        main1,
        suite1,
        &[],
        arena1,
    );
    let a = attaquer(&mut t, P0, "Essai Frappe", R);
    (t, a)
}

/// Vie perdue par le joueur entre maintenant et le tour suivant.
fn perte_en_fin_de_tour(t: &mut Partie, p: PlayerId) -> i32 {
    let avant = t.vie(p);
    t.finir_le_tour();
    avant - t.vie(p)
}

// ---------------------------------------------------------------------------
// Levia
// ---------------------------------------------------------------------------

#[test]
fn levia_perd_1_pv_par_carte_avec_blood_debt_bannie_sans_carte_de_6_bannie_ce_tour() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", Y), ("Feeding Frenzy", B)],
        &[],
    );
    // Bannies par l'adversaire au tour 1 : 5{p} et 4{p}, avec blood debt.
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", Y);
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 2);
}

#[test]
fn levia_ne_perd_pas_de_pv_de_blood_debt_si_une_carte_de_6_a_ete_bannie_ce_tour() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", R), ("Feeding Frenzy", B)],
        &[],
    );
    // Même bannie par l'adversaire : « put into your banished zone ».
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", R); // 6{p}
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 0);
}

#[test]
fn levia_une_carte_de_6_bannie_au_tour_precedent_ne_protege_pas() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", R)],
        &[],
    );
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", R);
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 0); // tour 1
    t.finir_le_tour(); // tour 2 (P1) : pas de blood debt pour Levia
    assert_eq!(t.g.turn, 3);
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 1); // tour 3
}

#[test]
fn levia_une_carte_de_6_bannie_dans_la_zone_adverse_ne_protege_pas() {
    let mut t = partie_vraie(
        [LEVIA, HEROS],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[GROWL_R],
        &[],
        &[],
    );
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    // Levia bannit une carte de 7{p}… de la zone de l'adversaire.
    let x = du_deck(&t, P1, "Graveling Growl", R);
    t.g.banish(P0, x, false);
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 1);
}

// ---------------------------------------------------------------------------
// Équipement
// ---------------------------------------------------------------------------

#[test]
fn hell_hammer_attaque_pour_2_ressources_une_fois_par_tour() {
    let mut t = levia(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hell Hammer"],
    );
    let w = t.equipement(P0, "Hell Hammer");
    let proxy = attaquer_arme(&mut t, P0, "Hell Hammer");
    assert_eq!(t.g.power(proxy), 6);
    assert_eq!(t.g.player(P0).resource_points, 1); // un Rempart (3) pour 2
                                                   // « Once per Turn » : plus d'activation possible ce tour.
    assert_eq!(t.g.obj(w).activations[0], 1);
    assert!(!t
        .g
        .legal_priority_options(P0)
        .contains(&Choice::Activate(w, 0)));
}

#[test]
fn hell_hammer_est_bannie_a_la_fermeture_de_la_chaine_si_elle_a_attaque() {
    let mut t = levia(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hell Hammer"],
    );
    let w = t.equipement(P0, "Hell Hammer");
    attaquer_arme(&mut t, P0, "Hell Hammer");
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(w).zone, Zone::Banished);
    // 6{p} : compte comme une carte de 6 bannie (Levia ne perd rien ce tour).
    assert!(t.g.banished_power6_this_turn(P0));
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 0);
}

#[test]
fn hell_hammer_reste_equipee_si_elle_n_a_pas_attaque() {
    let mut t = levia(
        &[("Feeding Frenzy", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hell Hammer"],
    );
    let w = t.equipement(P0, "Hell Hammer");
    attaquer(&mut t, P0, "Feeding Frenzy", B);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(w).zone, Zone::Weapon);
}

#[test]
fn hell_hammer_blood_debt_dans_la_zone_de_bannissement() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hell Hammer"],
    );
    attaquer_arme(&mut t, P0, "Hell Hammer");
    fin_du_combat(&mut t);
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 0); // bannie ce tour (6{p})
    t.finir_le_tour();
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 1); // tour 3 : blood debt
}

#[test]
fn buzzard_helm_en_defendant_pioche_puis_defausse_et_gagne_1_avec_une_carte_de_6() {
    let (mut t, _) = levia_defend(
        &[GROWL_R, GROWL_R, GROWL_R, GROWL_R],
        &[GROWL_R],
        &["Buzzard Helm"],
    );
    let h = t.equipement(P1, "Buzzard Helm");
    defendre_avec(&mut t, P1, &[h]);
    resoudre(&mut t);
    // Pioche 1, défausse 1 au hasard : toujours une carte de 7{p} ici.
    assert_eq!(t.g.player(P1).hand.len(), 4);
    assert_eq!(t.g.player(P1).graveyard.len(), 1);
    // Essai Frappe (4) contre 1 + 1 : 2 dégâts.
    assert_eq!(t.vie(P1), 20 - 2);
}

#[test]
fn buzzard_helm_une_carte_de_moins_de_6_defaussee_ne_donne_rien() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &["Buzzard Helm"],
    );
    let h = t.equipement(P1, "Buzzard Helm");
    defendre_avec(&mut t, P1, &[h]);
    resoudre(&mut t);
    // Le Rempart défaussé, puis le casque détruit par temper.
    assert_eq!(t.vie(P1), 20 - 3);
}

#[test]
fn buzzard_helm_temper() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &["Buzzard Helm"],
    );
    let h = t.equipement(P1, "Buzzard Helm");
    defendre_avec(&mut t, P1, &[h]);
    fin_du_combat(&mut t);
    // Défense 1, -1 : 0, détruit.
    assert_eq!(t.g.obj(h).zone, Zone::Graveyard);
}

#[test]
fn predatory_plating_detruire_gagne_1_ressource_avec_une_carte_de_6() {
    let mut t = levia(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hell Hammer", "Predatory Plating"],
    );
    let pl = t.equipement(P0, "Predatory Plating");
    activer(&mut t, P0, pl, 0);
    resoudre(&mut t);
    assert_eq!(t.g.obj(pl).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).resource_points, 1);
}

#[test]
fn predatory_plating_indisponible_sans_carte_de_6() {
    let t = levia(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Predatory Plating"],
    );
    let pl = t.equipement(P0, "Predatory Plating");
    assert!(!t.peut(Choice::Activate(pl, 0)));
}

#[test]
fn predatory_plating_une_carte_d_attaque_de_6_sur_la_chaine_compte() {
    let mut t = levia(
        &[("Feeding Frenzy", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Predatory Plating"],
    );
    let pl = t.equipement(P0, "Predatory Plating");
    attaquer(&mut t, P0, "Feeding Frenzy", R);
    assert!(t.peut(Choice::Activate(pl, 0)));
}

#[test]
fn predatory_plating_guardwell() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Predatory Plating"],
    );
    let pl = t.equipement(P1, "Predatory Plating");
    defendre_avec(&mut t, P1, &[pl]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.defense(pl), 0);
    assert_eq!(t.g.obj(pl).zone, Zone::Chest);
}

#[test]
fn skera_strapping_spellvoid_3_apres_avoir_pitche_une_carte_de_6() {
    let mut t = levia(
        &[("Feeding Frenzy", B), GROWL_R, GROWL_R, RAMPART],
        RIEN,
        &["Skera Strapping"],
    );
    let s = t.equipement(P0, "Skera Strapping");
    assert_eq!(t.g.prevention_values(s), (0, 0, 0));
    // Feeding Frenzy (coût 2) payé avec deux Graveling Growl (7{p}).
    let c = t.en_main(P0, "Feeding Frenzy", B);
    jouer_obj_pitch(&mut t, P0, c, "Graveling Growl");
    assert_eq!(t.g.prevention_values(s), (0, 0, 3));
}

/// Joue `c` en pitchant d'abord les cartes nommées `nom`.
fn jouer_obj_pitch(t: &mut Partie, p: PlayerId, c: ObjectId, nom: &str) {
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    while t.g.pending_decision().is_some() && t.kind() == DecisionKind::Pitch {
        let opts = t.options();
        let x = opts
            .iter()
            .copied()
            .find(|c| matches!(c, Choice::Pitch(o) if t.g.def(*o).name == nom))
            .unwrap_or(opts[0]);
        t.choisir(x);
    }
}

#[test]
fn skera_strapping_sans_pitch_de_6_pas_de_spellvoid() {
    let mut t = levia(
        &[("Feeding Frenzy", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Skera Strapping"],
    );
    let s = t.equipement(P0, "Skera Strapping");
    jouer(&mut t, P0, "Feeding Frenzy", B);
    assert_eq!(t.g.prevention_values(s), (0, 0, 0));
}

#[test]
fn embraforged_gauntlet_est_banni_au_lieu_d_aller_au_cimetiere() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Embraforged Gauntlet"],
    );
    let g = t.equipement(P1, "Embraforged Gauntlet");
    t.g.destroy(g);
    assert_eq!(t.g.obj(g).zone, Zone::Banished);
}

#[test]
fn embraforged_gauntlet_temper() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Embraforged Gauntlet"],
    );
    let g = t.equipement(P1, "Embraforged Gauntlet");
    defendre_avec(&mut t, P1, &[g]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.defense(g), 1);
    assert_eq!(t.g.obj(g).zone, Zone::Arms);
}

#[test]
fn embraforged_gauntlet_blood_debt_une_fois_banni() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Embraforged Gauntlet"],
    );
    let g = t.equipement(P0, "Embraforged Gauntlet");
    t.g.destroy(g);
    t.g.refresh_priority();
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 1);
}

#[test]
fn hex_gauntlet_se_bannit_et_retourne_face_cachee_une_carte_avec_blood_debt() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
        &["Hex Gauntlet"],
    );
    let f = bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    let h = t.equipement(P0, "Hex Gauntlet");
    activer(&mut t, P0, h, 0);
    resoudre_jusqua_question(&mut t);
    // Deux choix : Feeding Frenzy, ou Hex Gauntlet lui-même (banni, blood debt).
    assert!(t.peut(Choice::Object(f)) && t.peut(Choice::Object(h)));
    repondre(&mut t, Choice::Object(f));
    resoudre(&mut t);
    assert_eq!(t.g.obj(h).zone, Zone::Banished);
    assert!(t.g.obj(f).face_down);
    assert!(!t.g.obj(h).face_down);
}

/// Passe jusqu'à une question d'une capacité (ou la fin de la pile).
fn resoudre_jusqua_question(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.kind() == DecisionKind::Effect || (t.g.stack.is_empty() && t.g.tasks.is_empty())
    });
}

#[test]
fn hex_gauntlet_une_carte_face_cachee_n_a_plus_blood_debt() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
        &["Hex Gauntlet"],
    );
    let f = bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    let h = t.equipement(P0, "Hex Gauntlet");
    activer(&mut t, P0, h, 0);
    resoudre_jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(f));
    resoudre(&mut t);
    // Hex Gauntlet (6{p}) a été mis dans la zone de bannissement ce tour :
    // Levia ne perd rien ce tour. Au tour 3, seul Hex Gauntlet (face visible) compte.
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 0);
    t.finir_le_tour();
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 1);
}

#[test]
fn hex_gauntlet_6_de_puissance_sans_defense_ne_peut_pas_defendre() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hex Gauntlet"],
    );
    let h = t.equipement(P1, "Hex Gauntlet");
    assert_eq!(t.g.power(h), 6);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    assert!(!t.peut(Choice::Defend(h)));
}

#[test]
fn hooves_une_carte_de_6_bannie_permet_de_detruire_pour_1_point_d_action() {
    let mut t = levia(
        &[("Cleave the Heavens", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hooves of the Shadowbeast"],
    );
    let hv = t.equipement(P0, "Hooves of the Shadowbeast");
    let c = t.en_main(P0, "Cleave the Heavens", R);
    activer(&mut t, P0, c, 0); // bannit Cleave the Heavens (6{p})
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    resoudre(&mut t);
    assert_eq!(t.g.obj(hv).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).action_points, 2);
}

#[test]
fn hooves_le_joueur_peut_refuser() {
    let mut t = levia(
        &[("Cleave the Heavens", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hooves of the Shadowbeast"],
    );
    let hv = t.equipement(P0, "Hooves of the Shadowbeast");
    let c = t.en_main(P0, "Cleave the Heavens", R);
    activer(&mut t, P0, c, 0);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.g.obj(hv).zone, Zone::Legs);
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn hooves_une_carte_de_moins_de_6_ne_declenche_rien() {
    let mut t = levia(
        &[("Cleave the Heavens", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hooves of the Shadowbeast"],
    );
    let c = t.en_main(P0, "Cleave the Heavens", B);
    activer(&mut t, P0, c, 0); // 4{p}
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).banished.len(), 1);
    assert!(t.g.stack.is_empty());
    assert_eq!(jetons(&t, P0, GATE), 1);
}

#[test]
fn hooves_battleworn() {
    let (mut t, _) = levia_defend(
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Hooves of the Shadowbeast"],
    );
    let hv = t.equipement(P1, "Hooves of the Shadowbeast");
    defendre_avec(&mut t, P1, &[hv]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.defense(hv), 0);
    assert_eq!(t.g.obj(hv).zone, Zone::Legs);
}

// ---------------------------------------------------------------------------
// Coûts « banish 3 random cards from your graveyard »
// ---------------------------------------------------------------------------

/// Levia avec `carte` en main et trois cartes `cim` au cimetière.
fn avec_cimetiere(carte: (&str, Option<Color>), cim: (&str, Option<Color>)) -> Partie {
    let mut t = levia(&[carte, RAMPART, RAMPART, RAMPART], &[cim, cim, cim], &[]);
    for _ in 0..3 {
        au_cimetiere(&mut t, P0, cim.0, cim.1);
    }
    t
}

#[test]
fn boneyard_marauder_bannit_3_cartes_du_cimetiere_en_coût_additionnel() {
    let mut t = avec_cimetiere(("Boneyard Marauder", R), RAMPART);
    let a = attaquer(&mut t, P0, "Boneyard Marauder", R);
    assert_eq!(t.g.player(P0).graveyard.len(), 0);
    assert_eq!(t.g.player(P0).banished.len(), 3);
    assert_eq!(t.g.power(a), 6);
}

#[test]
fn boneyard_marauder_injouable_avec_moins_de_3_cartes_au_cimetiere() {
    let mut t = levia(
        &[("Boneyard Marauder", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART, RAMPART],
        &[],
    );
    au_cimetiere(&mut t, P0, "Essai Rempart", B);
    au_cimetiere(&mut t, P0, "Essai Rempart", B);
    let c = t.en_main(P0, "Boneyard Marauder", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn boneyard_marauder_blood_debt() {
    let t = levia(
        &[("Boneyard Marauder", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let c = t.en_main(P0, "Boneyard Marauder", R);
    assert!(t.g.keywords(c).has(Keywords::BLOOD_DEBT));
}

#[test]
fn dread_screamer_go_again_si_une_carte_de_6_est_bannie_ainsi() {
    let mut t = avec_cimetiere(("Dread Screamer", R), GROWL_R);
    let a = attaquer(&mut t, P0, "Dread Screamer", R);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn dread_screamer_sans_carte_de_6_bannie_pas_de_go_again() {
    let mut t = avec_cimetiere(("Dread Screamer", R), RAMPART);
    let a = attaquer(&mut t, P0, "Dread Screamer", R);
    assert_eq!(t.g.player(P0).banished.len(), 3);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn endless_maw_plus_3_si_une_carte_de_6_est_bannie_ainsi() {
    let mut t = avec_cimetiere(("Endless Maw", Y), GROWL_R);
    let a = attaquer(&mut t, P0, "Endless Maw", Y);
    assert_eq!(t.g.power(a), 5 + 3);
}

#[test]
fn endless_maw_sans_carte_de_6_bannie_pas_de_bonus() {
    let mut t = avec_cimetiere(("Endless Maw", Y), RAMPART);
    let a = attaquer(&mut t, P0, "Endless Maw", Y);
    assert_eq!(t.g.power(a), 5);
}

// ---------------------------------------------------------------------------
// Gate to i'Arathael, Cleave the Heavens, Countdown to Extinction, Dam the Shadowake
// ---------------------------------------------------------------------------

#[test]
fn cleave_the_heavens_bannir_de_la_main_cree_un_gate() {
    let mut t = levia(
        &[("Cleave the Heavens", Y), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let c = t.en_main(P0, "Cleave the Heavens", Y);
    activer(&mut t, P0, c, 0);
    assert_eq!(t.g.obj(c).zone, Zone::Banished);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, GATE), 1);
    // Blood debt (5{p}, rien de 6 banni) : 1 PV perdu en fin de tour.
    assert_eq!(perte_en_fin_de_tour(&mut t, P0), 1);
}

#[test]
fn gate_permet_de_jouer_ce_tour_une_action_avec_blood_debt_bannie() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
        &[],
    );
    let f = bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    creer(&mut t, P0, GATE, 1);
    assert!(!t.peut(Choice::Play(f)));
    let gate = jetons_obj(&t, P0, GATE)[0];
    activer(&mut t, P0, gate, 0); // {r} : un Rempart pitché
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, GATE), 0);
    jouer_obj(&mut t, P0, f);
    assert_eq!(t.g.obj(f).zone, Zone::Stack);
}

#[test]
fn gate_la_permission_ne_dure_que_ce_tour() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
        &[],
    );
    let f = bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    creer(&mut t, P0, GATE, 1);
    let gate = jetons_obj(&t, P0, GATE)[0];
    activer(&mut t, P0, gate, 0);
    resoudre(&mut t);
    debut_du_tour(&mut t, 3);
    assert!(!t.g.legal_priority_options(P0).contains(&Choice::Play(f)));
}

#[test]
fn gate_cible_seulement_une_action_avec_blood_debt_face_visible() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART, ("Feeding Frenzy", B)],
        &[],
    );
    // Un Rempart banni (sans blood debt) et un Feeding Frenzy face cachée.
    bannir_par(&mut t, P1, P0, "Essai Rempart", B);
    let f = du_deck(&t, P0, "Feeding Frenzy", B);
    t.g.banish(P1, f, true);
    creer(&mut t, P0, GATE, 1);
    let gate = jetons_obj(&t, P0, GATE)[0];
    assert!(!t.peut(Choice::Activate(gate, 0)));
}

#[test]
fn countdown_to_extinction_en_attaquant_cree_un_gate() {
    let mut t = levia(
        &[("Countdown to Extinction", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    attaquer(&mut t, P0, "Countdown to Extinction", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, GATE), 1);
}

#[test]
fn countdown_to_extinction_en_touchant_propose_de_chercher_un_darkest_hour() {
    let mut t = levia(
        &[("Countdown to Extinction", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    attaquer(&mut t, P0, "Countdown to Extinction", R);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    resoudre(&mut t);
    // Aucun Darkest Hour dans le deck : rien n'est banni.
    assert_eq!(t.g.player(P0).banished.len(), 0);
    assert_eq!(t.vie(P1), 20 - 6);
}

#[test]
fn dam_the_shadowake_defendre_l_attaque_d_un_heros_shadow_cree_un_gate() {
    let mut t = partie_vraie(
        [LEVIA, HEROS],
        &[("Feeding Frenzy", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Dam the Shadowake", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    attaquer(&mut t, P0, "Feeding Frenzy", B);
    let d = t.en_main(P1, "Dam the Shadowake", R);
    defendre_avec(&mut t, P1, &[d]);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P1, GATE), 1);
}

#[test]
fn dam_the_shadowake_l_attaque_d_un_heros_non_shadow_ne_cree_rien() {
    let (mut t, _) = levia_defend(
        &[("Dam the Shadowake", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let d = t.en_main(P1, "Dam the Shadowake", B);
    defendre_avec(&mut t, P1, &[d]);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P1, GATE), 0);
    assert_eq!(t.g.defense(d), 2);
}

// ---------------------------------------------------------------------------
// Deadwood Rumbler, Feasting Shadowbeast, Feeding Frenzy, Graveling Growl
// ---------------------------------------------------------------------------

#[test]
fn deadwood_rumbler_pioche_defausse_et_bannit_une_carte_d_un_cimetiere_si_6() {
    // Main après avoir joué : 3 Graveling Growl ; pioche un Graveling Growl.
    let mut t = levia(
        &[("Deadwood Rumbler", B), GROWL_R, GROWL_R, GROWL_R],
        &[GROWL_R, GROWL_R],
        &[],
    );
    let x = au_cimetiere(&mut t, P0, "Graveling Growl", R);
    let c = t.en_main(P0, "Deadwood Rumbler", B);
    jouer_obj_pitch(&mut t, P0, c, "Graveling Growl"); // coût 3 : 3 Graveling Growl
    jusqua_question(&mut t);
    // Choix d'une carte d'un cimetière : la défaussée ou celle d'avant.
    repondre(&mut t, Choice::Object(x));
    resoudre(&mut t);
    assert_eq!(t.g.obj(x).zone, Zone::Banished);
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
    // Bannie par Levia, 7{p} : « you've banished a card with 6 or more ».
    assert!(t.g.you_banished_power6_this_turn(P0));
}

#[test]
fn deadwood_rumbler_une_carte_de_moins_de_6_defaussee_ne_bannit_rien() {
    let mut t = levia(
        &[("Deadwood Rumbler", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[],
    );
    attaquer(&mut t, P0, "Deadwood Rumbler", B);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
    assert_eq!(t.g.player(P0).banished.len(), 0);
}

#[test]
fn deadwood_rumbler_sans_defense_ne_peut_pas_defendre() {
    let (mut t, _) = levia_defend(
        &[("Deadwood Rumbler", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let d = t.en_main(P1, "Deadwood Rumbler", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    assert!(!t.peut(Choice::Defend(d)));
}

#[test]
fn feasting_shadowbeast_bannit_le_dessus_du_deck_et_gagne_2_si_6() {
    let mut t = levia(
        &[("Feasting Shadowbeast", Y), RAMPART, RAMPART, RAMPART],
        &[GROWL_R],
        &[],
    );
    let top = du_deck(&t, P0, "Graveling Growl", R);
    let a = attaquer(&mut t, P0, "Feasting Shadowbeast", Y);
    resoudre(&mut t);
    assert_eq!(t.g.obj(top).zone, Zone::Banished);
    assert_eq!(t.g.power(a), 5 + 2);
}

#[test]
fn feasting_shadowbeast_sans_carte_de_6_bannie_pas_de_bonus() {
    let mut t = levia(
        &[("Feasting Shadowbeast", Y), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[],
    );
    let a = attaquer(&mut t, P0, "Feasting Shadowbeast", Y);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).banished.len(), 1);
    assert_eq!(t.g.power(a), 5);
}

#[test]
fn feasting_shadowbeast_une_carte_bannie_par_l_adversaire_ne_compte_pas() {
    let mut t = levia(
        &[("Feasting Shadowbeast", Y), RAMPART, RAMPART, RAMPART],
        &[RAMPART, GROWL_R],
        &[],
    );
    bannir_par(&mut t, P1, P0, "Graveling Growl", R);
    let a = attaquer(&mut t, P0, "Feasting Shadowbeast", Y);
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 5);
}

#[test]
fn feeding_frenzy_bannit_le_dessus_du_deck_et_gagne_go_again_si_6() {
    let mut t = levia(
        &[("Feeding Frenzy", B), RAMPART, RAMPART, RAMPART],
        &[GROWL_R],
        &[],
    );
    let a = attaquer(&mut t, P0, "Feeding Frenzy", B);
    resoudre(&mut t);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn feeding_frenzy_sans_carte_de_6_bannie_pas_de_go_again() {
    let mut t = levia(
        &[("Feeding Frenzy", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[],
    );
    let a = attaquer(&mut t, P0, "Feeding Frenzy", B);
    resoudre(&mut t);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn graveling_growl_injouable_sans_carte_de_6_bannie_ce_tour() {
    let t = levia(&[GROWL_R, RAMPART, RAMPART, RAMPART], RIEN, &[]);
    let c = t.en_main(P0, "Graveling Growl", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn graveling_growl_jouable_apres_une_carte_de_6_mise_dans_votre_zone_de_bannissement() {
    let mut t = levia(
        &[GROWL_R, ETINCELLE, RAMPART, RAMPART],
        &[("Feeding Frenzy", R)],
        &[],
    );
    bannir_par(&mut t, P1, P0, "Feeding Frenzy", R);
    let c = t.en_main(P0, "Graveling Growl", R);
    assert!(t.peut(Choice::Play(c)));
}

// ---------------------------------------------------------------------------
// Agility, Might, Vigor : Agile Windup, Clash / Test of Agility,
// Vigorous Smashup, Smashback Alehorn
// ---------------------------------------------------------------------------

#[test]
fn agile_windup_defausser_cree_un_agility() {
    let mut t = levia(&[("Agile Windup", R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    let c = t.en_main(P0, "Agile Windup", R);
    activer(&mut t, P0, c, 0);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Agility"), 1);
}

#[test]
fn agility_au_debut_de_votre_tour_se_detruit_et_la_prochaine_attaque_a_go_again() {
    let mut t = partie_vraie(
        [LEVIA, HEROS],
        &[ETINCELLE, ("Feeding Frenzy", B), RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P0, "Agility", 1);
    t.finir_le_tour();
    assert_eq!(jetons(&t, P0, "Agility"), 1); // pas au tour adverse
    debut_du_tour(&mut t, 3);
    assert_eq!(jetons(&t, P0, "Agility"), 0);
    let a = attaquer(&mut t, P0, "Feeding Frenzy", B);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn clash_of_agility_en_defendant_le_gagnant_du_clash_cree_un_agility() {
    // Dessus du deck de Levia : Graveling Growl (7) contre Rempart (sans {p}).
    let (mut t, _) = levia_defend(
        &[("Clash of Agility", B), RAMPART, RAMPART, RAMPART],
        &[GROWL_R],
        &[],
    );
    let d = t.en_main(P1, "Clash of Agility", B);
    defendre_avec(&mut t, P1, &[d]);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P1, "Agility"), 1);
    assert_eq!(jetons(&t, P0, "Agility"), 0);
}

#[test]
fn test_of_agility_en_defendant_le_gagnant_du_clash_cree_un_agility() {
    // Dessus du deck de Levia : Rempart (sans {p}) ; attaquant : Essai Frappe (4).
    let mut t = partie_vraie(
        [HEROS, LEVIA],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[("Essai Frappe", Y)],
        &[("Test of Agility", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    attaquer(&mut t, P0, "Essai Frappe", R);
    let d = t.en_main(P1, "Test of Agility", R);
    defendre_avec(&mut t, P1, &[d]);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Agility"), 1);
    assert_eq!(jetons(&t, P1, "Agility"), 0);
}

#[test]
fn vigorous_smashup_le_gagnant_cree_un_vigor_et_on_peut_mettre_sa_carte_sous_le_deck() {
    let (mut t, _) = levia_defend(
        &[("Vigorous Smashup", R), RAMPART, RAMPART, RAMPART],
        &[GROWL_R],
        &[],
    );
    let top = du_deck(&t, P1, "Graveling Growl", R);
    let d = t.en_main(P1, "Vigorous Smashup", R);
    defendre_avec(&mut t, P1, &[d]);
    jusqua_question(&mut t);
    assert_eq!(jetons(&t, P1, "Vigor"), 1);
    repondre(&mut t, Choice::Yes);
    resoudre(&mut t);
    assert_eq!(t.g.player(P1).deck[0], top);
    assert_ne!(t.g.top_of_deck(P1), Some(top));
}

#[test]
fn vigorous_smashup_on_peut_garder_sa_carte_sur_le_deck() {
    let (mut t, _) = levia_defend(
        &[("Vigorous Smashup", R), RAMPART, RAMPART, RAMPART],
        &[GROWL_R],
        &[],
    );
    let top = du_deck(&t, P1, "Graveling Growl", R);
    let d = t.en_main(P1, "Vigorous Smashup", R);
    defendre_avec(&mut t, P1, &[d]);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.g.top_of_deck(P1), Some(top));
}

#[test]
fn smashback_alehorn_cree_un_agility_et_un_might_avec_go_again() {
    let mut t = levia(
        &[("Smashback Alehorn", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Smashback Alehorn", B);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Agility"), 1);
    assert_eq!(jetons(&t, P0, "Might"), 1);
    assert_eq!(t.g.player(P0).action_points, 1);
}

// ---------------------------------------------------------------------------
// Wrecker Romp, Engulfing Shadows, Soul Harvest
// ---------------------------------------------------------------------------

#[test]
fn wrecker_romp_defausse_une_carte_au_hasard_en_cout_additionnel() {
    let mut t = levia(&[("Wrecker Romp", R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    let a = attaquer(&mut t, P0, "Wrecker Romp", R);
    // Un Rempart pitché, un défaussé.
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
    assert_eq!(t.g.player(P0).hand.len(), 1);
    assert_eq!(t.g.power(a), 8);
}

#[test]
fn wrecker_romp_injouable_sans_autre_carte_en_main() {
    let mut t = levia(&[("Wrecker Romp", R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    // Il ne reste que Wrecker Romp en main, avec des ressources.
    t.g.player_mut(P0).resource_points = 2;
    for _ in 0..3 {
        let r = t.en_main(P0, "Essai Rempart", B);
        t.g.put_in_graveyard(r);
    }
    t.g.refresh_priority();
    let c = t.en_main(P0, "Wrecker Romp", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn engulfing_shadows_est_bannie_au_lieu_d_aller_au_cimetiere() {
    let (mut t, _) = levia_defend(
        &[("Engulfing Shadows", Y), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let d = t.en_main(P1, "Engulfing Shadows", Y);
    passer_jusqua_que(&mut t, |t| t.joueur() == P1 && t.peut(Choice::Play(d)));
    t.choisir(Choice::Play(d));
    payer(&mut t);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(d).zone, Zone::Banished);
    assert!(t.g.keywords(d).has(Keywords::BLOOD_DEBT));
}

#[test]
fn soul_harvest_bannit_6_cartes_et_gagne_1_par_carte_avec_blood_debt() {
    let mut t = levia(
        &[("Soul Harvest", B), RAMPART, RAMPART, RAMPART],
        &[
            ("Feeding Frenzy", B),
            ("Feeding Frenzy", Y),
            RAMPART,
            RAMPART,
            RAMPART,
            RAMPART,
            RAMPART,
        ],
        &[],
    );
    let ff = au_cimetiere(&mut t, P0, "Feeding Frenzy", B);
    let fy = au_cimetiere(&mut t, P0, "Feeding Frenzy", Y);
    for _ in 0..5 {
        au_cimetiere(&mut t, P0, "Essai Rempart", B);
    }
    t.g.player_mut(P0).resource_points = 3; // coût 6 : 3 + un Rempart
    t.g.refresh_priority();
    let c = t.en_main(P0, "Soul Harvest", B);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    payer(&mut t);
    // 7 cartes : le joueur choisit les 6 à bannir, une par une.
    let mut gardee = None;
    while t.kind() == DecisionKind::Effect {
        let opts = t.options();
        // Garde un Rempart au cimetière.
        let rempart =
            t.g.player(P0)
                .graveyard
                .iter()
                .copied()
                .find(|&x| t.g.def(x).name == "Essai Rempart");
        let x = opts
            .iter()
            .copied()
            .find(|o| matches!(o, Choice::Object(x) if Some(*x) != rempart))
            .unwrap();
        gardee = rempart;
        t.choisir(x);
    }
    assert_eq!(t.g.player(P0).graveyard, vec![gardee.unwrap()]);
    assert_eq!(t.g.obj(ff).zone, Zone::Banished);
    assert_eq!(t.g.obj(fy).zone, Zone::Banished);
    assert_eq!(t.g.power(c), 6 + 2);
}

#[test]
fn soul_harvest_injouable_avec_moins_de_6_cartes_au_cimetiere() {
    let mut t = levia(
        &[("Soul Harvest", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 5],
        &[],
    );
    for _ in 0..5 {
        au_cimetiere(&mut t, P0, "Essai Rempart", B);
    }
    t.g.player_mut(P0).resource_points = 6;
    t.g.refresh_priority();
    let c = t.en_main(P0, "Soul Harvest", B);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn soul_harvest_en_touchant_un_heros_sans_ame_ne_fait_rien_perdre_de_plus() {
    let mut t = levia(
        &[("Soul Harvest", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 6],
        &[],
    );
    for _ in 0..6 {
        au_cimetiere(&mut t, P0, "Essai Rempart", B);
    }
    t.g.player_mut(P0).resource_points = 3;
    t.g.refresh_priority();
    attaquer(&mut t, P0, "Soul Harvest", B);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 20 - 6);
}

// ---------------------------------------------------------------------------
// Goremass Summoning, Blasmophet, Consuming Strength
// ---------------------------------------------------------------------------

#[test]
fn goremass_summoning_cree_un_blasmophet_si_vous_avez_banni_une_carte_de_6() {
    let mut t = levia(
        &[
            ("Goremass Summoning", B),
            ("Cleave the Heavens", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    let c = t.en_main(P0, "Cleave the Heavens", R);
    activer(&mut t, P0, c, 0); // Levia bannit une carte de 6{p}
    resoudre(&mut t);
    jouer(&mut t, P0, "Goremass Summoning", B);
    resoudre(&mut t);
    let b = jetons_obj(&t, P0, BLASMOPHET);
    assert_eq!(b.len(), 1);
    assert_eq!(t.g.power(b[0]), 6);
    assert_eq!(t.g.life_total(b[0]), Some(6));
    // Go again.
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn goremass_summoning_sans_carte_de_6_bannie_ne_cree_rien() {
    let mut t = levia(
        &[("Goremass Summoning", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Goremass Summoning", B);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, BLASMOPHET), 0);
}

#[test]
fn blasmophet_unique_le_joueur_garde_un_seul_blasmophet() {
    let mut t = levia(
        &[
            ("Goremass Summoning", B),
            ("Cleave the Heavens", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    creer(&mut t, P0, BLASMOPHET, 1);
    let ancien = jetons_obj(&t, P0, BLASMOPHET)[0];
    let c = t.en_main(P0, "Cleave the Heavens", R);
    activer(&mut t, P0, c, 0);
    resoudre(&mut t);
    jouer(&mut t, P0, "Goremass Summoning", B);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(ancien));
    resoudre(&mut t);
    assert_eq!(jetons_obj(&t, P0, BLASMOPHET), vec![ancien]);
}

#[test]
fn blasmophet_une_fois_par_tour_jouer_une_action_avec_blood_debt_bannie() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B), ("Feeding Frenzy", Y)],
        &[],
    );
    let f1 = bannir_par(&mut t, P1, P0, "Feeding Frenzy", B);
    let f2 = bannir_par(&mut t, P1, P0, "Feeding Frenzy", Y);
    assert!(!t.peut(Choice::Play(f1)));
    creer(&mut t, P0, BLASMOPHET, 1);
    assert!(t.peut(Choice::Play(f1)) && t.peut(Choice::Play(f2)));
    jouer_obj(&mut t, P0, f1);
    fin_du_combat(&mut t);
    // Une seule fois par tour (et Feeding Frenzy a go again ? non : rien banni de 6).
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    assert!(!t.peut(Choice::Play(f2)));
}

#[test]
fn blasmophet_ne_permet_pas_de_jouer_une_carte_sans_blood_debt() {
    let mut t = levia(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Agile Windup", B)],
        &[],
    );
    let a = bannir_par(&mut t, P1, P0, "Agile Windup", B);
    creer(&mut t, P0, BLASMOPHET, 1);
    assert!(!t.peut(Choice::Play(a)));
}

#[test]
fn blasmophet_en_fin_de_phase_detruit_sans_carte_blood_debt_bannie_ce_tour() {
    let mut t = levia(&[RAMPART, RAMPART, RAMPART, RAMPART], RIEN, &[]);
    creer(&mut t, P0, BLASMOPHET, 1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    // « You may banish a card from your hand » : le joueur refuse.
    repondre(&mut t, Choice::Pass);
    assert_eq!(jetons(&t, P0, BLASMOPHET), 0);
}

#[test]
fn blasmophet_bannir_une_carte_avec_blood_debt_de_la_main_le_garde() {
    let mut t = levia(
        &[("Feeding Frenzy", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    creer(&mut t, P0, BLASMOPHET, 1);
    let f = t.en_main(P0, "Feeding Frenzy", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    repondre(&mut t, Choice::Object(f));
    assert_eq!(t.g.obj(f).zone, Zone::Banished);
    assert_eq!(jetons(&t, P0, BLASMOPHET), 1);
}

#[test]
fn blasmophet_se_declenche_aussi_en_fin_de_tour_adverse() {
    let mut t = levia(
        &[("Cleave the Heavens", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    creer(&mut t, P0, BLASMOPHET, 1);
    // Tour 1 : Levia bannit une carte avec blood debt, Blasmophet reste.
    let c = t.en_main(P0, "Cleave the Heavens", B);
    activer(&mut t, P0, c, 0);
    resoudre(&mut t);
    t.finir_le_tour();
    assert_eq!(jetons(&t, P0, BLASMOPHET), 1);
    // Tour 2 (P1) : rien banni, Blasmophet est détruit.
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P0);
    repondre(&mut t, Choice::Pass);
    assert_eq!(jetons(&t, P0, BLASMOPHET), 0);
}

#[test]
fn consuming_strength_injouable_sans_blasmophet() {
    let mut t = levia(
        &[("Consuming Strength", Y), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let c = t.en_main(P0, "Consuming Strength", Y);
    assert!(!t.peut(Choice::Play(c)));
    creer(&mut t, P0, BLASMOPHET, 1);
    assert!(t.peut(Choice::Play(c)));
}

#[test]
fn consuming_strength_bannir_de_la_main_pour_1_ressource_la_prochaine_attaque_a_plus_2() {
    let mut t = levia(
        &[
            ("Consuming Strength", Y),
            ("Feeding Frenzy", B),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    let c = t.en_main(P0, "Consuming Strength", Y);
    activer(&mut t, P0, c, 0);
    assert_eq!(t.g.obj(c).zone, Zone::Banished);
    assert_eq!(t.g.player(P0).resource_points, 2);
    resoudre(&mut t);
    let a = attaquer(&mut t, P0, "Feeding Frenzy", B);
    assert_eq!(t.g.power(a), 4 + 2);
}

#[test]
fn les_cartes_bannies_gardent_trace_de_qui_les_a_bannies() {
    let mut t = levia(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &[GROWL_R], &[]);
    let x = bannir_par(&mut t, P1, P0, "Graveling Growl", R);
    let ev =
        t.g.turn_events()
            .iter()
            .find_map(|e| match *e {
                Event::Banished {
                    card,
                    by,
                    owner,
                    power,
                    ..
                } if card == x => Some((by, owner, power)),
                _ => None,
            })
            .unwrap();
    assert_eq!(ev, (P1, P0, 7));
    let _ = Target::Player(P0);
}
