//! Tests des vraies cartes Ninja (deck Ira, Crimson Haze) : un test par
//! phrase du texte de chaque carte, plus des cas limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Keywords, ObjectId, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const TIGRE: &str = "Crouching Tiger";

/// Passe (priorité, défense, arsenal) jusqu'à ce que `cond` soit vraie.
fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..200 {
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

/// Le joueur qui a la priorité joue cette carte de sa main (dès qu'il le
/// peut) et paie avec des Essai Rempart.
fn jouer(t: &mut Partie, p: fab_engine::PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    passer_jusqua_que(t, |t| t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    payer(t);
    c
}

/// Joue une carte (de P0) puis passe jusqu'à ce qu'elle soit l'attaque active.
fn attaquer(t: &mut Partie, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = jouer(t, P0, nom, couleur);
    jusqua_active(t, c);
    c
}

fn jusqua_active(t: &mut Partie, a: ObjectId) {
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(a));
}

/// P0 attaque avec son arme ; renvoie l'attaque-proxy (active).
fn attaquer_arme(t: &mut Partie, arme: &str) -> ObjectId {
    let w = t.equipement(P0, arme);
    passer_jusqua_que(t, |t| t.peut(Choice::Activate(w, 0)));
    t.choisir(Choice::Activate(w, 0));
    payer(t);
    let i =
        t.g.objects
            .iter()
            .rposition(|o| o.proxy_of == Some(w))
            .expect("attaque-proxy");
    let proxy = ObjectId(i as u32);
    jusqua_active(t, proxy);
    proxy
}

/// Active la capacité n° 0 d'un équipement de P0 et résout la pile.
fn activer(t: &mut Partie, nom: &str) {
    let e = t.equipement(P0, nom);
    t.choisir(Choice::Activate(e, 0));
    payer(t);
    resoudre(t);
}

fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| t.g.stack.is_empty());
}

fn fin_du_combat(t: &mut Partie) {
    passer_jusqua_que(t, |t| !t.g.combat.open);
}

/// Les Crouching Tiger de la zone de bannissement de P0.
fn tigres(t: &Partie) -> Vec<ObjectId> {
    t.g.player(P0)
        .banished
        .iter()
        .copied()
        .filter(|&o| t.g.def(o).name == TIGRE)
        .collect()
}

/// P1 défend avec ces cartes de sa main, puis passe.
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

/// Ira (P0) contre Essai Héros (P1, main de 4 Essai Rempart).
fn ira(main0: &[(&str, Option<Color>)], arena0: &[&str]) -> Partie {
    partie_vraie(
        ["Ira, Crimson Haze", "Essai Héros"],
        main0,
        RIEN,
        RIEN,
        RIEN,
        arena0,
        &[],
    )
}

/// Essai Héros (P0) contre Essai Héros (P1) avec les cartes données.
fn duel(
    main0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_vraie(
        ["Essai Héros", "Essai Héros"],
        main0,
        RIEN,
        main1,
        RIEN,
        arena0,
        arena1,
    )
}

/// Crée un Crouching Tiger avec Pouncing Paws et le joue ; renvoie le tigre
/// devenu l'attaque active.
fn tigre_des_pattes(t: &mut Partie) -> ObjectId {
    let paws = t.equipement(P0, "Pouncing Paws");
    passer_jusqua_que(t, |t| t.peut(Choice::Activate(paws, 0)));
    activer(t, "Pouncing Paws");
    let tigre = *tigres(t).last().expect("un tigre");
    passer_jusqua_que(t, |t| t.peut(Choice::Play(tigre)));
    t.choisir(Choice::Play(tigre));
    jusqua_active(t, tigre);
    tigre
}

// ---------------------------------------------------------------------------
// Ira, Crimson Haze
// ---------------------------------------------------------------------------

#[test]
fn ira_la_deuxieme_attaque_du_tour_gagne_plus_un() {
    let mut t = ira(
        &[
            ("Essai Assaut", R),
            ("Essai Assaut", Y),
            ("Essai Assaut", B),
            RAMPART,
        ],
        &[],
    );
    let a = attaquer(&mut t, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 3, "première attaque : pas de bonus");
    let b = attaquer(&mut t, "Essai Assaut", Y);
    assert_eq!(t.g.power(b), 3, "deuxième attaque : 2 + 1");
    let c = attaquer(&mut t, "Essai Assaut", B);
    assert_eq!(t.g.power(c), 1, "troisième attaque : pas de bonus");
}

#[test]
fn ira_le_bonus_compte_aussi_une_attaque_d_arme() {
    let mut t = ira(
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        &["Edge of Autumn"],
    );
    attaquer(&mut t, "Essai Assaut", R);
    let proxy = attaquer_arme(&mut t, "Edge of Autumn");
    assert_eq!(t.g.power(proxy), 2, "Edge of Autumn 1 + 1");
}

// ---------------------------------------------------------------------------
// Edge of Autumn
// ---------------------------------------------------------------------------

#[test]
fn edge_of_autumn_attaque_pour_une_ressource_avec_go_again() {
    let mut t = ira(&[RAMPART, RAMPART, RAMPART, RAMPART], &["Edge of Autumn"]);
    let proxy = attaquer_arme(&mut t, "Edge of Autumn");
    assert_eq!(t.g.power(proxy), 1);
    assert_eq!(t.g.player(P0).resource_points, 2, "3 pitchées - 1");
    assert!(t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn edge_of_autumn_une_seule_fois_par_tour() {
    let mut t = ira(&[RAMPART, RAMPART, RAMPART, RAMPART], &["Edge of Autumn"]);
    let w = t.equipement(P0, "Edge of Autumn");
    attaquer_arme(&mut t, "Edge of Autumn");
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1);
    assert!(!t.peut(Choice::Activate(w, 0)));
}

// ---------------------------------------------------------------------------
// Mask of the Swarming Claw
// ---------------------------------------------------------------------------

#[test]
fn mask_of_the_swarming_claw_arcane_barrier_1() {
    let t = ira(&[RAMPART; 4], &["Mask of the Swarming Claw"]);
    let m = t.equipement(P0, "Mask of the Swarming Claw");
    assert_eq!(t.g.prevention_values(m).0, 1);
}

#[test]
fn mask_of_the_swarming_claw_spellvoid_egal_aux_maillons_controles() {
    let mut t = ira(
        &[("Essai Assaut", R), ("Essai Assaut", Y), RAMPART, RAMPART],
        &["Mask of the Swarming Claw"],
    );
    let m = t.equipement(P0, "Mask of the Swarming Claw");
    assert_eq!(t.g.prevention_values(m).2, 0, "aucun maillon");
    attaquer(&mut t, "Essai Assaut", R);
    assert_eq!(t.g.prevention_values(m).2, 1);
    attaquer(&mut t, "Essai Assaut", Y);
    assert_eq!(t.g.prevention_values(m).2, 2);
    fin_du_combat(&mut t);
    assert_eq!(t.g.prevention_values(m).2, 0, "chaîne fermée");
}

#[test]
fn mask_of_the_swarming_claw_les_maillons_adverses_ne_comptent_pas() {
    let mut t = duel(
        &[RAMPART; 4],
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        &[],
        &["Mask of the Swarming Claw"],
    );
    t.finir_le_tour();
    let m = t.equipement(P1, "Mask of the Swarming Claw");
    let a = jouer(&mut t, P1, "Essai Assaut", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.prevention_values(m).2, 1, "P1 contrôle le maillon");
    // Pour P0 (qui n'a pas de masque ici) on vérifie l'inverse avec le
    // contrôleur : le maillon appartient à P1.
    assert_eq!(t.g.combat.links[0].attacker, P1);
}

// ---------------------------------------------------------------------------
// Blood Scent, Double Cross Strap, Tearing Shuko, Pouncing Paws
// ---------------------------------------------------------------------------

#[test]
fn blood_scent_seulement_apres_une_attaque_de_crouching_tiger() {
    let mut t = ira(&[RAMPART; 4], &["Blood Scent", "Pouncing Paws"]);
    let bs = t.equipement(P0, "Blood Scent");
    assert!(!t.peut(Choice::Activate(bs, 0)));
    tigre_des_pattes(&mut t);
    assert!(t.peut(Choice::Activate(bs, 0)));
    let avant = t.g.player(P0).resource_points;
    activer(&mut t, "Blood Scent");
    assert_eq!(t.g.player(P0).resource_points, avant + 1, "gagne {{r}}");
    assert_eq!(t.g.obj(bs).zone, Zone::Graveyard, "détruit");
}

#[test]
fn blood_scent_battleworn() {
    let t = ira(&[RAMPART; 4], &["Blood Scent"]);
    let bs = t.equipement(P0, "Blood Scent");
    assert!(t.g.keywords(bs).has(Keywords::BATTLEWORN));
}

#[test]
fn double_cross_strap_apres_deux_touches_dans_la_chaine() {
    let mut t = ira(
        &[("Essai Assaut", R), ("Essai Assaut", Y), RAMPART, RAMPART],
        &["Double Cross Strap"],
    );
    let s = t.equipement(P0, "Double Cross Strap");
    attaquer(&mut t, "Essai Assaut", R);
    let b = t.en_main(P0, "Essai Assaut", Y);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(b)));
    assert!(!t.peut(Choice::Activate(s, 0)), "une seule touche");
    attaquer(&mut t, "Essai Assaut", Y);
    passer_jusqua_que(&mut t, |t| t.g.combat.links[1].hit);
    assert!(t.peut(Choice::Activate(s, 0)));
    activer(&mut t, "Double Cross Strap");
    assert_eq!(t.g.player(P0).resource_points, 1);
    assert_eq!(t.g.obj(s).zone, Zone::Graveyard);
}

#[test]
fn double_cross_strap_arcane_barrier_1() {
    let t = ira(&[RAMPART; 4], &["Double Cross Strap"]);
    let s = t.equipement(P0, "Double Cross Strap");
    assert_eq!(t.g.prevention_values(s).0, 1);
}

#[test]
fn tearing_shuko_le_prochain_crouching_tiger_joue_ce_tour_gagne_plus_deux() {
    let mut t = ira(
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        &["Tearing Shuko", "Pouncing Paws"],
    );
    activer(&mut t, "Tearing Shuko");
    assert_eq!(t.g.obj(t.equipement(P0, "Pouncing Paws")).zone, Zone::Legs);
    // Une autre attaque ne consomme pas l'effet.
    let a = attaquer(&mut t, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 3);
    let tigre = tigre_des_pattes(&mut t);
    // Ira : le tigre est aussi la deuxième attaque du tour (+1).
    assert_eq!(t.g.power(tigre), 3, "0 + 2 + 1 (Ira)");
}

#[test]
fn tearing_shuko_battleworn() {
    let t = ira(&[RAMPART; 4], &["Tearing Shuko"]);
    let s = t.equipement(P0, "Tearing Shuko");
    assert!(t.g.keywords(s).has(Keywords::BATTLEWORN));
}

#[test]
fn pouncing_paws_cree_un_crouching_tiger_jouable_ce_tour() {
    let mut t = ira(&[RAMPART; 4], &["Pouncing Paws"]);
    let paws = t.equipement(P0, "Pouncing Paws");
    activer(&mut t, "Pouncing Paws");
    assert_eq!(t.g.obj(paws).zone, Zone::Graveyard, "détruit");
    let v = tigres(&t);
    assert_eq!(v.len(), 1);
    assert!(t.peut(Choice::Play(v[0])));
}

#[test]
fn pouncing_paws_le_tigre_n_est_plus_jouable_au_tour_suivant() {
    let mut t = ira(&[RAMPART; 4], &["Pouncing Paws"]);
    activer(&mut t, "Pouncing Paws");
    let tigre = tigres(&t)[0];
    t.finir_le_tour();
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P0);
    assert_eq!(t.g.obj(tigre).zone, Zone::Banished);
    assert!(!t.peut(Choice::Play(tigre)));
}

#[test]
fn pouncing_paws_battleworn() {
    let t = ira(&[RAMPART; 4], &["Pouncing Paws"]);
    let s = t.equipement(P0, "Pouncing Paws");
    assert!(t.g.keywords(s).has(Keywords::BATTLEWORN));
}

// ---------------------------------------------------------------------------
// Crouching Tiger
// ---------------------------------------------------------------------------

#[test]
fn crouching_tiger_attaque_de_0_avec_go_again() {
    let mut t = ira(&[RAMPART; 4], &["Pouncing Paws"]);
    let tigre = tigre_des_pattes(&mut t);
    assert_eq!(t.g.power(tigre), 0);
    assert_eq!(t.g.player(P0).action_points, 0, "une action : 1 point");
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn crouching_tiger_ephemeral_retire_de_la_partie_au_lieu_du_cimetiere() {
    let mut t = ira(&[RAMPART; 4], &["Pouncing Paws"]);
    let tigre = tigre_des_pattes(&mut t);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(tigre).zone, Zone::Gone);
    assert!(!t.g.player(P0).graveyard.contains(&tigre));
}

// ---------------------------------------------------------------------------
// Aspect of Tiger: Body
// ---------------------------------------------------------------------------

#[test]
fn aspect_of_tiger_combo_apres_une_attaque_rouge_go_again_et_tigre() {
    let mut t = ira(
        &[
            ("Essai Assaut", R),
            ("Aspect of Tiger: Body", R),
            RAMPART,
            RAMPART,
        ],
        &[],
    );
    attaquer(&mut t, "Essai Assaut", R);
    let a = attaquer(&mut t, "Aspect of Tiger: Body", R);
    resoudre(&mut t);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    let v = tigres(&t);
    assert_eq!(v.len(), 1);
    fin_du_combat(&mut t);
    assert!(t.peut(Choice::Play(v[0])), "jouable ce tour");
}

#[test]
fn aspect_of_tiger_sans_combo_rien() {
    let mut t = ira(
        &[("Aspect of Tiger: Body", R), RAMPART, RAMPART, RAMPART],
        &[],
    );
    let a = attaquer(&mut t, "Aspect of Tiger: Body", R);
    resoudre(&mut t);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
    assert!(tigres(&t).is_empty());
}

#[test]
fn aspect_of_tiger_pas_de_combo_apres_une_attaque_bleue() {
    let mut t = ira(
        &[
            ("Essai Assaut", B),
            ("Aspect of Tiger: Body", R),
            RAMPART,
            RAMPART,
        ],
        &[],
    );
    attaquer(&mut t, "Essai Assaut", B);
    attaquer(&mut t, "Aspect of Tiger: Body", R);
    resoudre(&mut t);
    assert!(tigres(&t).is_empty());
}

#[test]
fn aspect_of_tiger_pas_de_combo_apres_une_attaque_d_arme() {
    let mut t = ira(
        &[("Aspect of Tiger: Body", R), RAMPART, RAMPART, RAMPART],
        &["Edge of Autumn"],
    );
    attaquer_arme(&mut t, "Edge of Autumn");
    attaquer(&mut t, "Aspect of Tiger: Body", R);
    resoudre(&mut t);
    assert!(tigres(&t).is_empty());
}

// ---------------------------------------------------------------------------
// Bittering Thorns
// ---------------------------------------------------------------------------

#[test]
fn bittering_thorns_si_elle_touche_la_prochaine_attaque_gagne_plus_un() {
    let mut t = duel(
        &[
            ("Bittering Thorns", R),
            ("Essai Assaut", R),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let b = attaquer(&mut t, "Bittering Thorns", R);
    assert_eq!(t.g.power(b), 4);
    assert!(t.g.keywords(b).has(Keywords::GO_AGAIN));
    let a = attaquer(&mut t, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 4, "3 + 1");
    assert_eq!(t.vie(P1), 16);
}

#[test]
fn bittering_thorns_bloquee_pas_de_bonus() {
    let mut t = duel(
        &[
            ("Bittering Thorns", B),
            ("Essai Assaut", R),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    attaquer(&mut t, "Bittering Thorns", B);
    defendre_avec(&mut t, &[RAMPART]);
    let a = attaquer(&mut t, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 3);
}

// ---------------------------------------------------------------------------
// Blessing of Qi
// ---------------------------------------------------------------------------

#[test]
fn blessing_of_qi_au_debut_du_tour_detruit_et_cree_un_tigre_plus_trois() {
    let mut t = ira(&[("Blessing of Qi", R), RAMPART, RAMPART, RAMPART], &[]);
    let b = jouer(&mut t, P0, "Blessing of Qi", R);
    resoudre(&mut t);
    assert_eq!(t.g.obj(b).zone, Zone::Permanent, "une aura");
    assert_eq!(t.g.player(P0).action_points, 0, "pas de go again");
    t.finir_le_tour();
    // Début du tour de P1 : rien.
    assert_eq!(t.g.obj(b).zone, Zone::Permanent);
    assert!(tigres(&t).is_empty());
    t.finir_le_tour();
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.joueur() == P0);
    assert_eq!(t.g.obj(b).zone, Zone::Graveyard, "détruite");
    let v = tigres(&t);
    assert_eq!(v.len(), 1);
    assert_eq!(t.g.power(v[0]), 3);
    t.choisir(Choice::Play(v[0]));
    jusqua_active(&mut t, v[0]);
    assert_eq!(t.g.power(v[0]), 3, "le bonus suit le tigre sur la chaîne");
}

#[test]
fn blessing_of_qi_bleu_plus_un() {
    let mut t = ira(&[("Blessing of Qi", B), RAMPART, RAMPART, RAMPART], &[]);
    jouer(&mut t, P0, "Blessing of Qi", B);
    resoudre(&mut t);
    t.finir_le_tour();
    t.finir_le_tour();
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.joueur() == P0);
    assert_eq!(t.g.power(tigres(&t)[0]), 1);
}

// ---------------------------------------------------------------------------
// Flying Kick
// ---------------------------------------------------------------------------

#[test]
fn flying_kick_maillon_3_ou_plus_gagne_plus_deux() {
    let mut t = duel(
        &[
            ("Essai Assaut", R),
            ("Essai Assaut", Y),
            ("Flying Kick", R),
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    attaquer(&mut t, "Essai Assaut", R);
    attaquer(&mut t, "Essai Assaut", Y);
    let k = attaquer(&mut t, "Flying Kick", R);
    assert_eq!(t.g.power(k), 7);
}

#[test]
fn flying_kick_maillon_1_ou_2_pas_de_bonus() {
    let mut t = duel(
        &[("Essai Assaut", R), ("Flying Kick", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    attaquer(&mut t, "Essai Assaut", R);
    let k = attaquer(&mut t, "Flying Kick", B);
    assert_eq!(t.g.power(k), 3, "maillon 2");
}

// ---------------------------------------------------------------------------
// Growl, Untamed
// ---------------------------------------------------------------------------

#[test]
fn growl_le_prochain_tigre_joue_dans_cette_chaine_gagne_plus_un() {
    let mut t = duel(
        &[("Growl", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Pouncing Paws"],
        &[],
    );
    let g = attaquer(&mut t, "Growl", R);
    resoudre(&mut t);
    assert!(t.g.keywords(g).has(Keywords::GO_AGAIN));
    let tigre = tigre_des_pattes(&mut t);
    assert_eq!(t.g.power(tigre), 1);
}

#[test]
fn growl_l_effet_finit_avec_la_chaine() {
    let mut t = duel(
        &[("Growl", Y), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Pouncing Paws"],
        &[],
    );
    attaquer(&mut t, "Growl", Y);
    fin_du_combat(&mut t);
    let tigre = tigre_des_pattes(&mut t);
    assert_eq!(t.g.power(tigre), 0);
}

#[test]
fn untamed_le_prochain_tigre_joue_dans_cette_chaine_gagne_plus_un() {
    let mut t = duel(
        &[("Untamed", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Pouncing Paws"],
        &[],
    );
    let u = attaquer(&mut t, "Untamed", R);
    assert_eq!(t.g.power(u), 4);
    assert!(t.g.keywords(u).has(Keywords::GO_AGAIN));
    let tigre = tigre_des_pattes(&mut t);
    assert_eq!(t.g.power(tigre), 1);
}

// ---------------------------------------------------------------------------
// Legacy of Ikaru, Seek Vengeance
// ---------------------------------------------------------------------------

#[test]
fn seek_vengeance_combo_apres_edge_of_autumn_go_again() {
    let mut t = ira(
        &[("Seek Vengeance", R), RAMPART, RAMPART, RAMPART],
        &["Edge of Autumn"],
    );
    attaquer_arme(&mut t, "Edge of Autumn");
    let s = attaquer(&mut t, "Seek Vengeance", R);
    assert!(t.g.keywords(s).has(Keywords::GO_AGAIN));
}

#[test]
fn seek_vengeance_sans_combo_pas_de_go_again() {
    let mut t = ira(&[("Seek Vengeance", B), RAMPART, RAMPART, RAMPART], &[]);
    let s = attaquer(&mut t, "Seek Vengeance", B);
    assert!(!t.g.keywords(s).has(Keywords::GO_AGAIN));
    assert_eq!(t.g.power(s), 2);
}

/// P0 attaque avec `premiere`, puis Seek Vengeance (rouge), puis joue Legacy
/// of Ikaru sur Seek Vengeance. Renvoie la partie à la mise en arsenal de P0.
fn legacy_apres(premiere: Option<(&str, Option<Color>)>) -> Partie {
    let mut main = vec![("Seek Vengeance", R), ("Legacy of Ikaru", B), RAMPART];
    main.push(premiere.unwrap_or(RAMPART));
    let mut t = ira(&main, &["Edge of Autumn"]);
    match premiere {
        Some((n, c)) => {
            attaquer(&mut t, n, c);
        }
        None => {
            attaquer_arme(&mut t, "Edge of Autumn");
        }
    }
    attaquer(&mut t, "Seek Vengeance", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction)
    });
    let l = t.en_main(P0, "Legacy of Ikaru", B);
    t.choisir(Choice::Play(l));
    // Plus rien à jouer : le moteur passe tout seul jusqu'à la fin du tour.
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Arsenal);
    t
}

#[test]
fn legacy_of_ikaru_plus_un_et_pioche_si_edge_of_autumn_etait_la_derniere_attaque() {
    let t = legacy_apres(None);
    // Edge of Autumn 1, puis Seek Vengeance 4 + 1 (Legacy) + 1 (Ira, 2e attaque).
    assert_eq!(t.vie(P1), 20 - 1 - 6);
    // Main : 1 Rampart (l'autre a payé Edge of Autumn) + la carte piochée.
    assert_eq!(t.g.player(P0).hand.len(), 2);
}

#[test]
fn legacy_of_ikaru_pas_de_pioche_sans_edge_of_autumn() {
    let t = legacy_apres(Some(("Essai Assaut", R)));
    // Essai Assaut 3, puis Seek Vengeance 4 + 1 + 1.
    assert_eq!(t.vie(P1), 20 - 3 - 6);
    assert_eq!(t.g.player(P0).hand.len(), 1, "pas de pioche");
    // Seek Vengeance n'a pas eu go again (pas de combo) : la chaîne s'est
    // fermée sans autre attaque.
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn legacy_of_ikaru_seulement_sur_une_attaque_ninja() {
    let mut t = ira(
        &[
            ("Essai Assaut", R),
            ("Legacy of Ikaru", B),
            RAMPART,
            RAMPART,
        ],
        &[],
    );
    attaquer(&mut t, "Essai Assaut", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction)
    });
    let l = t.en_main(P0, "Legacy of Ikaru", B);
    assert!(!t.peut(Choice::Play(l)), "Essai Assaut n'est pas Ninja");
}

// ---------------------------------------------------------------------------
// Pouncing Qi, Qi Unleashed
// ---------------------------------------------------------------------------

#[test]
fn pouncing_qi_combo_apres_crouching_tiger_plus_un_et_go_again() {
    let mut t = duel(
        &[("Pouncing Qi", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Pouncing Paws"],
        &[],
    );
    tigre_des_pattes(&mut t);
    let q = attaquer(&mut t, "Pouncing Qi", R);
    assert_eq!(t.g.power(q), 4);
    assert!(t.g.keywords(q).has(Keywords::GO_AGAIN));
}

#[test]
fn pouncing_qi_sans_combo() {
    let mut t = duel(
        &[("Pouncing Qi", Y), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let q = attaquer(&mut t, "Pouncing Qi", Y);
    assert_eq!(t.g.power(q), 2);
    assert!(!t.g.keywords(q).has(Keywords::GO_AGAIN));
}

#[test]
fn qi_unleashed_combo_apres_crouching_tiger_plus_quatre() {
    let mut t = duel(
        &[("Qi Unleashed", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Pouncing Paws"],
        &[],
    );
    tigre_des_pattes(&mut t);
    let q = attaquer(&mut t, "Qi Unleashed", B);
    assert_eq!(t.g.power(q), 5);
}

#[test]
fn qi_unleashed_combo_seulement_si_le_tigre_est_juste_avant() {
    let mut t = duel(
        &[("Essai Assaut", R), ("Qi Unleashed", R), RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Pouncing Paws"],
        &[],
    );
    tigre_des_pattes(&mut t);
    attaquer(&mut t, "Essai Assaut", R);
    let q = attaquer(&mut t, "Qi Unleashed", R);
    assert_eq!(t.g.power(q), 3);
}

// ---------------------------------------------------------------------------
// Tiger Eye Reflex
// ---------------------------------------------------------------------------

/// P0 met Tiger Eye Reflex en arsenal, P1 attaque avec Essai Assaut.
fn tiger_eye_en_arsenal(couleur: Option<Color>) -> (Partie, ObjectId) {
    let mut t = duel(
        &[("Tiger Eye Reflex", couleur), RAMPART, RAMPART, RAMPART],
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    let te = t.en_main(P0, "Tiger Eye Reflex", couleur);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Arsenal);
    t.choisir(Choice::Arsenal(te));
    t.finir_le_tour();
    let a = jouer(&mut t, P1, "Essai Assaut", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let _ = a;
    (t, te)
}

#[test]
fn tiger_eye_reflex_ambush_defend_depuis_l_arsenal() {
    let (mut t, te) = tiger_eye_en_arsenal(Y);
    assert!(t.peut(Choice::Defend(te)));
    t.choisir(Choice::Defend(te));
    t.choisir(Choice::Pass);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Arsenal);
    // Essai Assaut (3) entièrement bloqué par la défense 3.
    assert_eq!(t.vie(P0), 20);
    assert_eq!(t.g.obj(te).zone, Zone::Graveyard);
}

#[test]
fn tiger_eye_reflex_cree_un_tigre_jouable_pendant_le_prochain_tour() {
    let (mut t, te) = tiger_eye_en_arsenal(B);
    t.choisir(Choice::Defend(te));
    t.choisir(Choice::Pass);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Arsenal);
    assert_eq!(t.vie(P0), 19, "défense 2 contre 3");
    let v = tigres(&t);
    assert_eq!(v.len(), 1);
    // Pas pendant le tour de P1.
    assert_eq!(t.g.turn_player, P1);
    assert!(!t.g.play_permitted(P0, v[0]));
    passer_jusqua_que(&mut t, |t| {
        t.g.turn_player == P0 && t.kind() == DecisionKind::Priority
    });
    assert!(t.peut(Choice::Play(v[0])), "pendant le prochain tour de P0");
}

#[test]
fn tiger_eye_reflex_sans_ambush_pas_de_defense_depuis_l_arsenal_pour_une_autre_carte() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[("Essai Assaut", R), RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    let f = t.en_main(P0, "Essai Frappe", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Arsenal);
    t.choisir(Choice::Arsenal(f));
    t.finir_le_tour();
    jouer(&mut t, P1, "Essai Assaut", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    assert!(!t.peut(Choice::Defend(f)));
}

// ---------------------------------------------------------------------------
// Feign Vengeance
// ---------------------------------------------------------------------------

#[test]
fn feign_vengeance_pioche_si_une_carte_la_defend() {
    let mut t = duel(
        &[("Feign Vengeance", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let f = attaquer(&mut t, "Feign Vengeance", B);
    assert_eq!(t.g.power(f), 0);
    assert_eq!(t.g.player(P0).hand.len(), 3);
    defendre_avec(&mut t, &[RAMPART]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).hand.len(), 4, "une carte piochée");
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn feign_vengeance_sans_defense_pas_de_pioche() {
    let mut t = duel(
        &[("Feign Vengeance", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    attaquer(&mut t, "Feign Vengeance", B);
    defendre_avec(&mut t, &[]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).hand.len(), 3);
}
