//! Tests des vraies cartes du deck Dromai (Draconic, Illusionist, dragons,
//! tokens Ash / Aether Ashwing / Spectral Shield, et cartes génériques du
//! deck) : un test par phrase du texte de chaque carte, plus des cas limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{
    Choice, DecisionKind, Event, Keywords, ObjectId, PlayerId, PreventRef, Target, Zone,
};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const DROMAI: &str = "Dromai";
const HEROS: &str = "Essai Héros";
const STORM: &str = "Storm of Sandikai";
/// Instant toujours jouable (avec un Rempart à pitcher) : le joueur garde
/// une option, le moteur ne passe pas tout seul.
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);
/// Carte d'attaque non-Illusionist de 7{p} (déf. 3) : défend contre phantasm.
const GROWL: (&str, Option<Color>) = ("Graveling Growl", R);

// ---------------------------------------------------------------------------
// Aides
// ---------------------------------------------------------------------------

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

/// La dernière attaque-proxy créée pour `src`.
fn proxy_de(t: &Partie, src: ObjectId) -> ObjectId {
    let i =
        t.g.objects
            .iter()
            .rposition(|o| o.proxy_of == Some(src))
            .expect("attaque-proxy");
    ObjectId(i as u32)
}

/// Le joueur attaque avec ce dragon ; renvoie l'attaque-proxy (active).
fn attaquer_dragon(t: &mut Partie, p: PlayerId, dragon: ObjectId) -> ObjectId {
    activer(t, p, dragon, 0);
    let proxy = proxy_de(t, dragon);
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
fn creer(t: &mut Partie, p: PlayerId, nom: &str, n: u8) -> Vec<ObjectId> {
    let c = t.g.db.find_name(nom).expect("token codé");
    t.g.create_tokens(p, c, n);
    let v = t.g.last_created.clone();
    t.g.refresh_priority();
    v
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

/// Répond aux questions des capacités (`DecisionKind::Effect`) par `c`.
fn repondre(t: &mut Partie, c: Choice) {
    assert_eq!(t.kind(), DecisionKind::Effect, "{:?}", t.noms(&t.options()));
    t.choisir(c);
}

/// Passe jusqu'à une question d'une capacité.
fn jusqua_question(t: &mut Partie) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Effect);
}

/// Dromai (P0, avec Storm of Sandikai) contre Essai Héros (P1).
fn dromai(main0: &[(&str, Option<Color>)], suite0: &[(&str, Option<Color>)]) -> Partie {
    dromai_contre(main0, suite0, RIEN, RIEN)
}

fn dromai_contre(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
) -> Partie {
    partie_vraie([DROMAI, HEROS], main0, suite0, main1, suite1, &[STORM], &[])
}

/// Crée une Ash puis joue cette invocation (qui la vise) et la résout ;
/// renvoie l'objet (le dragon).
fn invoquer(t: &mut Partie, nom: &str) -> ObjectId {
    creer(t, P0, "Ash", 1);
    let c = jouer(t, P0, nom, R);
    resoudre(t);
    c
}

/// Les points d'action du joueur.
fn pa(t: &Partie, p: PlayerId) -> u8 {
    t.g.player(p).action_points
}

// ---------------------------------------------------------------------------
// Dromai
// ---------------------------------------------------------------------------

#[test]
fn dromai_pitcher_une_carte_rouge_cree_une_ash() {
    // Rake the Embers (coût 1) payé en pitchant Blaze Headlong (rouge).
    let mut t = dromai(
        &[
            ("Rake the Embers", R),
            ("Blaze Headlong", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
    );
    let c = t.en_main(P0, "Rake the Embers", R);
    t.choisir(Choice::Play(c));
    let blaze = t.en_main(P0, "Blaze Headlong", R);
    t.choisir(Choice::Pitch(blaze));
    // L'Ash du pitch est créée tout de suite (avant la résolution de Rake).
    resoudre_sans_question(&mut t);
    assert!(t
        .g
        .turn_events()
        .iter()
        .any(|e| matches!(e, Event::Pitched { card, .. } if *card == blaze)));
    // Rake crée une Ash de plus : 2 Ash, puis les questions de transform.
    assert_eq!(compter_ash_et_ashwings(&t), 2);
}

/// Résout la pile en refusant les transformations (Pass).
fn resoudre_sans_question(t: &mut Partie) {
    for _ in 0..100 {
        if t.g.stack.is_empty() && t.g.pending_triggers.is_empty() && t.g.tasks.is_empty() {
            return;
        }
        match t.kind() {
            DecisionKind::Effect => t.choisir(Choice::Pass),
            _ => t.choisir(Choice::Pass),
        }
    }
}

/// Ash dans l'arène + Ash sous un permanent.
fn compter_ash_et_ashwings(t: &Partie) -> usize {
    jetons(t, P0, "Ash") + t.g.sub_cards.len()
}

#[test]
fn dromai_pitcher_une_carte_bleue_ne_cree_pas_d_ash() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    jouer(&mut t, P0, "Essai Étincelle", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Ash"), 0);
}

#[test]
fn dromai_attaque_de_dragon_a_go_again_si_une_carte_rouge_a_ete_jouee() {
    let mut t = dromai(&[("Invoke Cromai", R), RAMPART, RAMPART, RAMPART], RIEN);
    // Invoke Cromai (rouge) a été jouée ce tour.
    let cromai = invoquer(&mut t, "Invoke Cromai");
    let a = attaquer_dragon(&mut t, P0, cromai);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn dromai_attaque_de_dragon_sans_go_again_sans_carte_rouge_jouee() {
    let mut t = dromai(&[RAMPART, RAMPART, RAMPART, RAMPART], RIEN);
    let wing = creer(&mut t, P0, "Aether Ashwing", 1)[0];
    let a = attaquer_dragon(&mut t, P0, wing);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn dromai_une_carte_rouge_jouee_par_l_adversaire_ne_compte_pas() {
    let mut t = dromai_contre(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    let wing = creer(&mut t, P0, "Aether Ashwing", 1)[0];
    // P1 joue une Essai Étincelle (rouge) pendant le tour de P0.
    t.choisir(Choice::Pass);
    jouer(&mut t, P1, "Essai Étincelle", R);
    resoudre(&mut t);
    let a = attaquer_dragon(&mut t, P0, wing);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

// ---------------------------------------------------------------------------
// Storm of Sandikai
// ---------------------------------------------------------------------------

#[test]
fn storm_of_sandikai_donne_aux_dragons_une_attaque_par_tour() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let wing = creer(&mut t, P0, "Aether Ashwing", 1)[0];
    assert!(t.peut(Choice::Activate(wing, 0)));
    attaquer_dragon(&mut t, P0, wing);
    fin_du_combat(&mut t);
    // Once per Turn : plus d'attaque ce tour (même avec un point d'action).
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    assert!(!t.peut(Choice::Activate(wing, 0)));
}

#[test]
fn sans_storm_of_sandikai_un_dragon_ne_peut_pas_attaquer() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let wing = creer(&mut t, P0, "Aether Ashwing", 1)[0];
    assert!(!t.peut(Choice::Activate(wing, 0)));
}

#[test]
fn storm_of_sandikai_ne_donne_pas_d_attaque_aux_allies_non_dragons() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    assert!(t.g.def(ash).activated.is_empty());
    // Un dragon de l'adversaire n'a pas l'attaque de la Storm de P0.
    let wing1 = creer(&mut t, P1, "Aether Ashwing", 1)[0];
    assert!(!t.g.attack_granted(wing1));
}

// ---------------------------------------------------------------------------
// Ash, Aether Ashwing, Spectral Shield
// ---------------------------------------------------------------------------

#[test]
fn ash_sous_un_permanent_lui_donne_phantasm() {
    let mut t = dromai(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    let wing_card = t.g.db.find_name("Aether Ashwing").unwrap();
    let wing = t.g.transform_into_token(P0, &[ash], wing_card).unwrap();
    assert!(t.g.keywords(wing).has(Keywords::PHANTASM));
    // L'Ash elle-même, dans l'arène, ne donne rien à personne.
    let ash2 = creer(&mut t, P0, "Ash", 1)[0];
    assert!(!t.g.keywords(ash2).has(Keywords::PHANTASM));
    let autre = creer(&mut t, P0, "Aether Ashwing", 1)[0];
    assert!(!t.g.keywords(autre).has(Keywords::PHANTASM));
}

#[test]
fn aether_ashwing_attaque_avec_1_de_puissance() {
    let mut t = dromai(&[RAMPART, RAMPART, RAMPART, RAMPART], RIEN);
    let wing = creer(&mut t, P0, "Aether Ashwing", 1)[0];
    let a = attaquer_dragon(&mut t, P0, wing);
    assert_eq!(t.g.power(a), 1);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 19);
    // L'allié revient dans la zone permanente.
    assert_eq!(t.g.obj(wing).zone, Zone::Permanent);
}

#[test]
fn aether_ashwing_arcane_barrier_1_protege_son_controleur() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Éclair", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[STORM],
    );
    let wing = creer(&mut t, P1, "Aether Ashwing", 1)[0];
    let c = t.en_main(P0, "Essai Éclair", R);
    t.choisir(Choice::Play(c));
    t.choisir(Choice::Target(Target::Player(P1)));
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
    assert_eq!(t.joueur(), P1);
    assert!(t.peut(Choice::Prevent(PreventRef::ArcaneBarrier(wing))));
}

#[test]
fn spectral_shield_ward_1() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let s = creer(&mut t, P1, "Spectral Shield", 1)[0];
    attaquer(&mut t, P0, "Essai Frappe", R);
    fin_du_combat(&mut t);
    // 4 dégâts, 1 prévenu par le ward (le token est détruit).
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(s).zone, Zone::Gone);
}

// ---------------------------------------------------------------------------
// Équipements
// ---------------------------------------------------------------------------

/// Essai Héros (P0) attaque Dromai (P1, avec ces équipements) avec Essai Frappe (4{p}).
fn dromai_attaque(arena1: &[&str], main1: &[(&str, Option<Color>)]) -> Partie {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        main1,
        RIEN,
        &[],
        arena1,
    );
    attaquer(&mut t, P0, "Essai Frappe", R);
    t
}

#[test]
fn silken_shroud_ward_1_puis_cree_un_ponder() {
    let mut t = dromai_attaque(&["Silken Shroud"], RIEN);
    let shroud = t.equipement(P1, "Silken Shroud");
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(shroud).zone, Zone::Graveyard);
    assert_eq!(jetons(&t, P1, "Ponder"), 1);
}

#[test]
fn wave_of_reality_ward_1_puis_cree_un_spectral_shield() {
    let mut t = dromai_attaque(&["Wave of Reality"], RIEN);
    let wave = t.equipement(P1, "Wave of Reality");
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(wave).zone, Zone::Graveyard);
    assert_eq!(jetons(&t, P1, "Spectral Shield"), 1);
}

#[test]
fn smoldering_scales_peut_etre_detruite_au_lieu_de_recevoir_un_frostbite() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Smoldering Scales"],
        &[],
    );
    let scales = t.equipement(P0, "Smoldering Scales");
    let fb = t.g.db.find_name("Frostbite").unwrap();
    t.g.create_tokens(P0, fb, 1);
    assert_eq!(t.kind(), DecisionKind::Replacement);
    t.choisir(Choice::Yes);
    assert_eq!(jetons(&t, P0, "Frostbite"), 0);
    assert_eq!(t.g.obj(scales).zone, Zone::Graveyard);
}

#[test]
fn smoldering_scales_le_joueur_peut_garder_l_equipement_et_recevoir_le_frostbite() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Smoldering Scales"],
        &[],
    );
    let scales = t.equipement(P0, "Smoldering Scales");
    let fb = t.g.db.find_name("Frostbite").unwrap();
    t.g.create_tokens(P0, fb, 1);
    t.choisir(Choice::No);
    assert_eq!(jetons(&t, P0, "Frostbite"), 1);
    assert_eq!(t.g.obj(scales).zone, Zone::Chest);
    // Un Frostbite créé chez l'adversaire n'est pas concerné.
    t.g.create_tokens(P1, fb, 1);
    assert_eq!(jetons(&t, P1, "Frostbite"), 1);
}

#[test]
fn smoldering_scales_guardwell() {
    let mut t = dromai_attaque(&["Smoldering Scales"], RIEN);
    let scales = t.equipement(P1, "Smoldering Scales");
    defendre_avec(&mut t, P1, &[scales]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 18);
    assert_eq!(t.g.defense(scales), 0);
}

// ---------------------------------------------------------------------------
// Cartes d'attaque Draconic / Illusionist
// ---------------------------------------------------------------------------

#[test]
fn billowing_mirage_transforme_une_ash_en_aether_ashwing_quand_elle_attaque() {
    let mut t = dromai(&[("Billowing Mirage", R), RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    jouer(&mut t, P0, "Billowing Mirage", R);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(ash));
    let wings = jetons_obj(&t, P0, "Aether Ashwing");
    assert_eq!(wings.len(), 1);
    assert_eq!(t.g.obj(ash).zone, Zone::Under);
    assert!(t.g.keywords(wings[0]).has(Keywords::PHANTASM));
}

#[test]
fn billowing_mirage_peut_ne_rien_transformer_et_a_go_again() {
    let mut t = dromai(&[("Billowing Mirage", R), RAMPART, RAMPART, RAMPART], RIEN);
    creer(&mut t, P0, "Ash", 1);
    let c = jouer(&mut t, P0, "Billowing Mirage", R);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Pass);
    assert_eq!(jetons(&t, P0, "Aether Ashwing"), 0);
    assert_eq!(jetons(&t, P0, "Ash"), 1);
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn billowing_mirage_sans_ash_pas_de_question() {
    let mut t = dromai(&[("Billowing Mirage", R), RAMPART, RAMPART, RAMPART], RIEN);
    let c = attaquer(&mut t, P0, "Billowing Mirage", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
}

#[test]
fn blaze_headlong_a_go_again_si_une_autre_carte_rouge_a_ete_jouee() {
    let mut t = dromai(
        &[("Blaze Headlong", R), ("Sand Cover", R), RAMPART, RAMPART],
        RIEN,
    );
    creer(&mut t, P0, "Ash", 1);
    jouer(&mut t, P0, "Sand Cover", R);
    resoudre(&mut t);
    let c = jouer(&mut t, P0, "Blaze Headlong", R);
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn blaze_headlong_seule_n_a_pas_go_again() {
    let mut t = dromai(&[("Blaze Headlong", R), RAMPART, RAMPART, RAMPART], RIEN);
    let c = jouer(&mut t, P0, "Blaze Headlong", R);
    assert!(!t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn dunebreaker_cenipai_a_phantasm_et_go_again() {
    let mut t = dromai(
        &[("Dunebreaker Cenipai", R), RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    let c = attaquer(&mut t, P0, "Dunebreaker Cenipai", R);
    assert!(t.g.keywords(c).has(Keywords::PHANTASM));
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 15);
}

#[test]
fn dunebreaker_cenipai_detruite_par_phantasm_cree_une_ash() {
    let mut t = dromai_contre(
        &[("Dunebreaker Cenipai", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[GROWL, RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    let c = attaquer(&mut t, P0, "Dunebreaker Cenipai", R);
    let growl = t.en_main(P1, "Graveling Growl", R);
    defendre_avec(&mut t, P1, &[growl]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    assert_eq!(t.vie(P1), 20);
    assert_eq!(jetons(&t, P0, "Ash"), 1);
}

#[test]
fn dustup_quand_elle_touche_cree_une_ash_puis_peut_la_transformer() {
    let mut t = dromai(&[("Dustup", R), RAMPART, RAMPART, RAMPART], RIEN);
    attaquer(&mut t, P0, "Dustup", R);
    jusqua_question(&mut t);
    // L'Ash vient d'être créée : elle peut devenir une Aether Ashwing.
    let ash = jetons_obj(&t, P0, "Ash")[0];
    repondre(&mut t, Choice::Object(ash));
    assert_eq!(jetons(&t, P0, "Aether Ashwing"), 1);
    assert_eq!(jetons(&t, P0, "Ash"), 0);
    assert_eq!(t.vie(P1), 16);
}

#[test]
fn dustup_qui_ne_touche_pas_ne_cree_rien() {
    let mut t = dromai_contre(
        &[("Dustup", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    attaquer(&mut t, P0, "Dustup", R);
    let r = t.en_main(P1, "Essai Rempart", B);
    defendre_avec(&mut t, P1, &[r]);
    fin_du_combat(&mut t);
    assert_eq!(jetons(&t, P0, "Ash"), 0);
}

#[test]
fn sweeping_blow_cree_une_ash_quand_elle_attaque_et_a_go_again() {
    let mut t = dromai(&[("Sweeping Blow", R), RAMPART, RAMPART, RAMPART], RIEN);
    let c = attaquer(&mut t, P0, "Sweeping Blow", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Ash"), 1);
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
}

// ---------------------------------------------------------------------------
// Actions non-attaque, auras, instants
// ---------------------------------------------------------------------------

#[test]
fn rake_the_embers_cree_une_ash_puis_transforme_jusqu_a_3_ash() {
    let mut t = dromai(&[("Rake the Embers", R), RAMPART, RAMPART, RAMPART], RIEN);
    creer(&mut t, P0, "Ash", 3);
    jouer(&mut t, P0, "Rake the Embers", R);
    for _ in 0..3 {
        jusqua_question(&mut t);
        let a = jetons_obj(&t, P0, "Ash")[0];
        repondre(&mut t, Choice::Object(a));
    }
    resoudre(&mut t);
    // 4 Ash : 3 transformées, 1 reste.
    assert_eq!(jetons(&t, P0, "Aether Ashwing"), 3);
    assert_eq!(jetons(&t, P0, "Ash"), 1);
    // Go again : le point d'action est revenu.
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn rake_the_embers_le_joueur_peut_s_arreter() {
    let mut t = dromai(&[("Rake the Embers", R), RAMPART, RAMPART, RAMPART], RIEN);
    creer(&mut t, P0, "Ash", 1);
    jouer(&mut t, P0, "Rake the Embers", R);
    jusqua_question(&mut t);
    let a = jetons_obj(&t, P0, "Ash")[0];
    repondre(&mut t, Choice::Object(a));
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Pass);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Aether Ashwing"), 1);
    assert_eq!(jetons(&t, P0, "Ash"), 1);
}

#[test]
fn sand_cover_donne_ward_4_a_une_ash() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Sand Cover", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let ash = creer(&mut t, P1, "Ash", 1)[0];
    attaquer(&mut t, P0, "Essai Frappe", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction) && t.joueur() == P1
    });
    jouer(&mut t, P1, "Sand Cover", R);
    // P1 n'a plus rien à jouer : la carte se résout, puis les dégâts.
    // Les 4 dégâts sont prévenus en détruisant l'Ash.
    assert_eq!(t.vie(P1), 20);
    assert_eq!(t.g.obj(ash).zone, Zone::Gone);
}

#[test]
fn sand_cover_ne_vise_que_vos_ash() {
    let mut t = dromai(&[("Sand Cover", R), RAMPART, RAMPART, RAMPART], RIEN);
    creer(&mut t, P1, "Ash", 1);
    let c = t.en_main(P0, "Sand Cover", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn essence_of_ancestry_body_ward_2() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Essence of Ancestry: Body", R)],
        &[],
        &[],
    );
    let e = mettre_en_jeu(&mut t, P1, "Essence of Ancestry: Body", R);
    attaquer(&mut t, P0, "Essai Frappe", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 18);
    assert_eq!(t.g.obj(e).zone, Zone::Graveyard);
}

/// Retire de la main ou du deck du joueur la première carte qui vérifie `f`.
fn retirer(t: &mut Partie, p: PlayerId, f: impl Fn(&Partie, ObjectId) -> bool) -> ObjectId {
    let ps = t.g.player(p);
    if let Some(i) = ps.hand.iter().position(|&c| f(t, c)) {
        return t.g.player_mut(p).hand.remove(i);
    }
    let i = ps
        .deck
        .iter()
        .position(|&c| f(t, c))
        .expect("carte absente de la main et du deck");
    t.g.player_mut(p).deck.remove(i)
}

/// Met en jeu (zone permanente) une carte de la main ou du deck du joueur.
fn mettre_en_jeu(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = retirer(t, p, |t, c| {
        t.g.def(c).name == nom && t.g.def(c).color == couleur
    });
    t.g.obj_mut(c).zone = Zone::Permanent;
    t.g.obj_mut(c).face_down = false;
    t.g.permanents.push(c);
    t.g.refresh_priority();
    c
}

#[test]
fn essence_of_ancestry_body_en_quittant_l_arene_previent_les_prochains_degats_d_une_source_rouge() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Assaut", R), ("Essai Frappe", R), RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Essence of Ancestry: Body", R)],
        &[],
        &[],
    );
    mettre_en_jeu(&mut t, P1, "Essence of Ancestry: Body", R);
    // Essai Assaut (rouge, 3, go again) : ward 2 → 1 dégât, l'aura quitte
    // l'arène.
    let a = attaquer(&mut t, P0, "Essai Assaut", R);
    passer_jusqua_que(&mut t, |t| t.g.active_attack() != Some(a));
    assert_eq!(t.vie(P1), 19);
    // Essai Frappe (rouge, 4) : tous ses dégâts sont prévenus.
    attaquer(&mut t, P0, "Essai Frappe", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn essence_of_ancestry_body_rien_si_une_autre_aura_illusionist_est_controlee() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Essence of Ancestry: Body", R)],
        &[],
        &[],
    );
    mettre_en_jeu(&mut t, P1, "Essence of Ancestry: Body", R);
    // Un Spectral Shield (aura Illusionist) reste après la destruction.
    // Le ward 2 de l'Essence et le ward 1 du Shield : 4 - 2 = 2, l'ordre
    // est au choix du joueur ; il détruit l'Essence seulement.
    creer(&mut t, P1, "Spectral Shield", 1);
    attaquer(&mut t, P0, "Essai Frappe", R);
    // Deux wards obligatoires : l'ordre compte (le Shield est-il encore là
    // quand l'Essence quitte l'arène ?), le joueur le choisit (CR 6.5.5).
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
    let e = *t
        .options()
        .iter()
        .find(|c| matches!(c, Choice::Prevent(PreventRef::Ward(o)) if t.g.def(*o).name == "Essence of Ancestry: Body"))
        .unwrap();
    t.choisir(e);
    fin_du_combat(&mut t);
    // 4 - 2 - 1 = 1 dégât ; pas de prévention créée par l'Essence (le Shield
    // était encore là quand elle a quitté l'arène).
    assert_eq!(t.vie(P1), 19);
    assert!(t.g.preventions.is_empty());
}

#[test]
fn passing_mirage_premiere_attaque_illusionist_perd_phantasm() {
    let mut t = dromai_contre(
        &[
            ("Dunebreaker Cenipai", R),
            ("Dunebreaker Cenipai", R),
            RAMPART,
            RAMPART,
        ],
        &[("Passing Mirage", B)],
        &[GROWL, RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    mettre_en_jeu(&mut t, P0, "Passing Mirage", B);
    let c = attaquer(&mut t, P0, "Dunebreaker Cenipai", R);
    assert!(!t.g.keywords(c).has(Keywords::PHANTASM));
    let growl = t.en_main(P1, "Graveling Growl", R);
    defendre_avec(&mut t, P1, &[growl]);
    fin_du_combat(&mut t);
    // Pas détruite par phantasm : 5 contre 3, 2 dégâts.
    assert_eq!(t.vie(P1), 18);
    // La deuxième attaque Illusionist du tour garde phantasm.
    let c2 = attaquer(&mut t, P0, "Dunebreaker Cenipai", R);
    assert!(t.g.keywords(c2).has(Keywords::PHANTASM));
}

#[test]
fn passing_mirage_spectra_detruite_quand_elle_est_attaquee() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Passing Mirage", B)],
        &[],
        &[],
    );
    let m = mettre_en_jeu(&mut t, P1, "Passing Mirage", B);
    let c = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(c));
    let r = t.g.obj_ref(m);
    t.choisir(Choice::Target(Target::Object(r)));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(t.g.obj(m).zone, Zone::Graveyard);
}

// ---------------------------------------------------------------------------
// Invocations et dragons
// ---------------------------------------------------------------------------

#[test]
fn invoke_transforme_l_ash_ciblee_en_dragon_avec_phantasm_et_go_again() {
    let mut t = dromai(&[("Invoke Azvolai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    let c = jouer(&mut t, P0, "Invoke Azvolai", R);
    resoudre(&mut t);
    assert_eq!(t.g.obj(c).zone, Zone::Permanent);
    assert_eq!(t.g.def(c).name, "Azvolai");
    assert_eq!(t.g.life_total(c), Some(3));
    assert_eq!(t.g.power(c), 2);
    assert_eq!(t.g.obj(ash).zone, Zone::Under);
    assert!(t.g.keywords(c).has(Keywords::PHANTASM));
    // Go again (imprimé sur Invoke Azvolai).
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn invoke_sans_ash_ne_peut_pas_etre_jouee() {
    let t = dromai(&[("Invoke Azvolai", R), ETINCELLE, RAMPART, RAMPART], RIEN);
    let c = t.en_main(P0, "Invoke Azvolai", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn invoke_dont_l_ash_a_disparu_entre_quand_meme_dans_l_arene() {
    // Essai Étincelle : P0 garde la priorité avec l'invocation sur la pile.
    let mut t = dromai_contre(
        &[("Invoke Cromai", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    let c = t.en_main(P0, "Invoke Cromai", R);
    passer_jusqua_que(&mut t, |t| t.joueur() == P0 && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    payer(&mut t);
    assert_eq!(t.g.obj(c).zone, Zone::Stack);
    // L'Ash disparaît avant la résolution.
    t.g.destroy(ash);
    t.g.refresh_priority();
    resoudre(&mut t);
    assert_eq!(t.g.obj(c).zone, Zone::Permanent);
    assert_eq!(t.g.def(c).name, "Cromai");
    assert!(!t.g.keywords(c).has(Keywords::PHANTASM));
}

#[test]
fn un_dragon_detruit_redevient_invoke_au_cimetiere_et_son_ash_disparait() {
    let mut t = dromai(&[("Invoke Cromai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    let c = jouer(&mut t, P0, "Invoke Cromai", R);
    resoudre(&mut t);
    t.g.destroy(c);
    t.g.refresh_priority();
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    assert_eq!(t.g.def(c).name, "Invoke Cromai");
    assert_eq!(t.g.obj(ash).zone, Zone::Gone);
    assert!(t.g.sub_cards.is_empty());
}

#[test]
fn azvolai_quand_il_attaque_inflige_1_degat_arcanique_a_2_cibles() {
    let mut t = dromai(&[("Invoke Azvolai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let az = invoquer(&mut t, "Invoke Azvolai");
    let wing1 = creer(&mut t, P1, "Aether Ashwing", 1)[0];
    activer(&mut t, P0, az, 0);
    // Cible de l'attaque : le héros adverse.
    if t.kind() == DecisionKind::AttackTarget {
        t.choisir(Choice::Target(Target::Player(P1)));
    }
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(P1)));
    let r = t.g.obj_ref(wing1);
    t.choisir(Choice::Target(Target::Object(r)));
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    // L'Ashwing de P1 (arcane barrier 1) protège son héros : P1 refuse de payer.
    if t.kind() == DecisionKind::Prevention {
        t.choisir(Choice::Pass);
    }
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.obj(wing1).zone, Zone::Gone);
}

#[test]
fn azvolai_le_joueur_peut_refuser() {
    let mut t = dromai(&[("Invoke Azvolai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let az = invoquer(&mut t, "Invoke Azvolai");
    activer(&mut t, P0, az, 0);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(P1)));
    if t.kind() == DecisionKind::Target {
        t.choisir(Choice::Pass);
    }
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn cromai_quand_il_attaque_gagne_1_point_d_action() {
    let mut t = dromai(&[("Invoke Cromai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let cromai = invoquer(&mut t, "Invoke Cromai");
    assert_eq!(pa(&t, P0), 1);
    attaquer_dragon(&mut t, P0, cromai);
    resoudre(&mut t);
    // 1 point d'action dépensé pour attaquer, 1 regagné.
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn cromai_quand_il_quitte_l_arene_gagne_1_point_d_action() {
    let mut t = dromai(&[("Invoke Cromai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let cromai = invoquer(&mut t, "Invoke Cromai");
    t.g.destroy(cromai);
    t.g.refresh_priority();
    resoudre(&mut t);
    assert_eq!(pa(&t, P0), 2);
}

#[test]
fn cromai_une_seule_fois_par_tour() {
    let mut t = dromai(&[("Invoke Cromai", R), RAMPART, RAMPART, RAMPART], RIEN);
    let cromai = invoquer(&mut t, "Invoke Cromai");
    attaquer_dragon(&mut t, P0, cromai);
    resoudre(&mut t);
    let avant = pa(&t, P0);
    t.g.destroy(cromai);
    t.g.refresh_priority();
    resoudre(&mut t);
    assert_eq!(pa(&t, P0), avant);
}

#[test]
fn kyloria_qui_touche_un_heros_prend_le_controle_d_un_de_ses_items() {
    let mut t = dromai_contre(
        &[("Invoke Kyloria", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Timesnap Potion", B)],
    );
    let potion = mettre_en_jeu(&mut t, P1, "Timesnap Potion", B);
    let ky = invoquer(&mut t, "Invoke Kyloria");
    attaquer_dragon(&mut t, P0, ky);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 16);
    assert_eq!(t.g.obj(potion).controller, P0);
}

#[test]
fn kyloria_sans_item_adverse_pioche_une_carte() {
    let mut t = dromai(&[("Invoke Kyloria", R), RAMPART, RAMPART, RAMPART], RIEN);
    let ky = invoquer(&mut t, "Invoke Kyloria");
    let main = t.g.player(P0).hand.len();
    attaquer_dragon(&mut t, P0, ky);
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).hand.len(), main + 1);
}

#[test]
fn miragai_la_premiere_attaque_de_dragon_perd_phantasm() {
    let mut t = dromai_contre(
        &[("Invoke Miragai", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[GROWL, GROWL, RAMPART, RAMPART],
        RIEN,
    );
    let mi = invoquer(&mut t, "Invoke Miragai");
    // Une Ashwing avec une Ash dessous (phantasm).
    let ash = creer(&mut t, P0, "Ash", 1)[0];
    let wc = t.g.db.find_name("Aether Ashwing").unwrap();
    let wing = t.g.transform_into_token(P0, &[ash], wc).unwrap();
    t.g.refresh_priority();
    assert!(t.g.keywords(mi).has(Keywords::PHANTASM));
    // Miragai attaque en premier : pas de phantasm.
    let a = attaquer_dragon(&mut t, P0, mi);
    assert!(!t.g.keywords(a).has(Keywords::PHANTASM));
    let growl = t.en_main(P1, "Graveling Growl", R);
    defendre_avec(&mut t, P1, &[growl]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(mi).zone, Zone::Permanent);
    // La deuxième attaque de dragon garde phantasm.
    let a2 = attaquer_dragon(&mut t, P0, wing);
    assert!(t.g.keywords(a2).has(Keywords::PHANTASM));
    let growl = t.en_main(P1, "Graveling Growl", R);
    defendre_avec(&mut t, P1, &[growl]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(wing).zone, Zone::Gone);
}

#[test]
fn nekria_qui_inflige_des_degats_prend_un_compteur_et_cree_une_ash() {
    let mut t = dromai(&[("Invoke Nekria", R), RAMPART, RAMPART, RAMPART], RIEN);
    let nek = invoquer(&mut t, "Invoke Nekria");
    attaquer_dragon(&mut t, P0, nek);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 16);
    assert_eq!(t.g.obj(nek).life_counters, -1);
    assert_eq!(t.g.life_total(nek), Some(6));
    assert_eq!(jetons(&t, P0, "Ash"), 1);
}

#[test]
fn nekria_qui_recoit_des_degats_prend_un_compteur_et_cree_une_ash() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Aethersling", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Invoke Nekria", R)],
        &[],
        &[STORM],
    );
    let nek = poser_dragon(&mut t, P1, "Invoke Nekria");
    let c = t.en_main(P0, "Aethersling", R);
    t.choisir(Choice::Play(c));
    let r = t.g.obj_ref(nek);
    if t.kind() == DecisionKind::Target {
        t.choisir(Choice::Target(Target::Object(r)));
    }
    payer(&mut t);
    jusqua_question(&mut t);
    // Aethersling : ne pas engager le héros.
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert!(t.g.turn_events().iter().any(|e| matches!(
        e,
        Event::Damage { target_obj: Some(o), amount: 4, .. } if *o == nek
    )));
    assert_eq!(t.g.obj(nek).life_counters, -1);
    assert_eq!(jetons(&t, P1, "Ash"), 1);
    // Après la remise à zéro de fin de tour (CR 4.4.3a), le compteur reste :
    // 7{h} - 1.
    t.finir_le_tour();
    assert_eq!(t.g.life_total(nek), Some(6));
}

/// Met directement en jeu le dragon d'une invocation du deck de `p`
/// (sans Ash dessous).
fn poser_dragon(t: &mut Partie, p: PlayerId, invoke: &str) -> ObjectId {
    let c = retirer(t, p, |t, c| t.g.def(c).name == invoke);
    let back = t.g.def(c).flip.unwrap().back;
    t.g.obj_mut(c).card = back;
    t.g.obj_mut(c).zone = Zone::Permanent;
    t.g.obj_mut(c).face_down = false;
    if !t.g.trigger_objects.contains(&c) {
        t.g.trigger_objects.push(c);
    }
    t.g.permanents.push(c);
    t.g.refresh_priority();
    c
}

#[test]
fn ouvia_en_entrant_transforme_une_ash_en_aether_ashwing() {
    let mut t = dromai(&[("Invoke Ouvia", R), RAMPART, RAMPART, RAMPART], RIEN);
    let ash = creer(&mut t, P0, "Ash", 2)[1];
    jouer(&mut t, P0, "Invoke Ouvia", R);
    // Une Ash ciblée par l'invocation, l'autre pour Ouvia.
    if t.kind() == DecisionKind::Target {
        let a0 = t.g.obj_ref(jetons_obj(&t, P0, "Ash")[0]);
        t.choisir(Choice::Target(Target::Object(a0)));
        payer(&mut t);
    }
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(ash));
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, "Aether Ashwing"), 1);
    assert_eq!(jetons(&t, P0, "Ash"), 0);
}

#[test]
fn ouvia_au_debut_de_votre_tour_transforme_une_ash() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        &[("Invoke Ouvia", R)],
        &[],
        &[STORM],
    );
    poser_dragon(&mut t, P1, "Invoke Ouvia");
    let ash = creer(&mut t, P1, "Ash", 1)[0];
    for _ in 0..200 {
        if t.g.turn == 2 && t.kind() == DecisionKind::Effect {
            break;
        }
        let o = t.options()[0];
        match t.kind() {
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            _ => t.choisir(o),
        }
    }
    assert_eq!(t.joueur(), P1);
    repondre(&mut t, Choice::Object(ash));
    assert_eq!(jetons(&t, P1, "Aether Ashwing"), 1);
}

#[test]
fn themai_les_adversaires_ne_peuvent_rien_jouer_pendant_votre_tour() {
    let mut t = dromai_contre(
        &[("Invoke Themai", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
    );
    let e = t.en_main(P1, "Essai Étincelle", R);
    invoquer(&mut t, "Invoke Themai");
    // P0 passe : P1 n'a que « passer » (le moteur passe pour lui).
    t.choisir(Choice::Pass);
    assert!(t
        .g
        .pending_decision()
        .is_none_or(|d| d.player != P1 || !d.options.contains(&Choice::Play(e))));
    assert!(t.g.silenced(P1));
    // Pendant le tour de P1, il peut jouer.
    t.finir_le_tour();
    assert!(!t.g.silenced(P1));
}

#[test]
fn yendurai_entre_avec_un_compteur_qui_previent_3_degats_une_fois() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Aethersling", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Invoke Yendurai", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[STORM],
    );
    // Yendurai invoqué par P1 : passer au tour 2.
    t.finir_le_tour();
    creer(&mut t, P1, "Ash", 1);
    let y = jouer(&mut t, P1, "Invoke Yendurai", R);
    resoudre(&mut t);
    assert_eq!(t.g.obj(y).counters, 1);
    t.finir_le_tour();
    // Tour 3 : Aethersling (4 arcanique) sur Yendurai : 3 prévenus par le compteur.
    let c = t.en_main(P0, "Aethersling", R);
    passer_jusqua_que(&mut t, |t| t.joueur() == P0 && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    let r = t.g.obj_ref(y);
    t.choisir(Choice::Target(Target::Object(r)));
    payer(&mut t);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    assert_eq!(t.g.obj(y).counters, 0);
    // 4 dégâts, 3 prévenus : 1 seul infligé.
    assert!(t.g.turn_events().iter().any(|e| matches!(
        e,
        Event::Damage { target_obj: Some(o), amount: 1, .. } if *o == y
    )));
}

// ---------------------------------------------------------------------------
// Cartes génériques du deck
// ---------------------------------------------------------------------------

#[test]
fn deep_blue_carte_sous_le_deck_detruire_gagner_3_ressources_go_again() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[("Invoke Nekria", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[STORM, "Deep Blue"],
        &[],
    );
    let db_ = t.equipement(P0, "Deep Blue");
    t.choisir(Choice::Activate(db_, 0));
    jusqua_question(&mut t);
    let r = t.en_main(P0, "Essai Rempart", B);
    repondre(&mut t, Choice::Object(r));
    resoudre(&mut t);
    assert_eq!(t.g.obj(db_).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).resource_points, 3);
    assert_eq!(*t.g.player(P0).deck.first().unwrap(), r);
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn deep_blue_main_vide_pas_d_activation() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Deep Blue"],
        &[],
    );
    t.g.player_mut(P0).hand.clear();
    t.g.refresh_priority();
    let db_ = t.equipement(P0, "Deep Blue");
    assert!(!t
        .g
        .legal_priority_options(P0)
        .contains(&Choice::Activate(db_, 0)));
}

#[test]
fn seekers_leggings_previent_1_degat_puis_opt_1() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Blaze Headlong", R)],
        &[],
        &["Seeker's Leggings"],
    );
    let l = t.equipement(P1, "Seeker's Leggings");
    attaquer(&mut t, P0, "Essai Frappe", R);
    activer(&mut t, P1, l, 0);
    jusqua_question(&mut t);
    // Opt 1 : la carte du dessus (Blaze Headlong) va sous le deck.
    let top = *t.g.player(P1).deck.last().unwrap();
    assert_eq!(t.g.def(top).name, "Blaze Headlong");
    repondre(&mut t, Choice::Pass);
    assert_eq!(*t.g.player(P1).deck.first().unwrap(), top);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(l).zone, Zone::Graveyard);
}

#[test]
fn time_skippers_3_ressources_detruire_gagner_2_points_d_action() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Time Skippers"],
        &[],
    );
    let ts = t.equipement(P0, "Time Skippers");
    activer(&mut t, P0, ts, 0);
    resoudre(&mut t);
    assert_eq!(t.g.obj(ts).zone, Zone::Graveyard);
    // 1 point d'action dépensé, 2 gagnés.
    assert_eq!(pa(&t, P0), 2);
}

#[test]
fn jack_be_quick_bannir_un_nimblism_donne_1_puissance_et_go_again() {
    let mut t = dromai(
        &[("Jack Be Quick", R), RAMPART, RAMPART, RAMPART],
        &[("Nimblism", R)],
    );
    let n = *t.g.player(P0).deck.last().unwrap();
    t.g.put_in_graveyard(n);
    t.g.refresh_priority();
    let c = jouer(&mut t, P0, "Jack Be Quick", R);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(n));
    assert_eq!(t.g.obj(n).zone, Zone::Banished);
    assert_eq!(t.g.power(c), 4);
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn jack_be_quick_sans_bannir_pas_de_bonus() {
    let mut t = dromai(
        &[("Jack Be Quick", R), RAMPART, RAMPART, RAMPART],
        &[("Nimblism", R)],
    );
    let n = *t.g.player(P0).deck.last().unwrap();
    t.g.put_in_graveyard(n);
    t.g.refresh_priority();
    let c = jouer(&mut t, P0, "Jack Be Quick", R);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Pass);
    assert_eq!(t.g.power(c), 3);
    assert!(!t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn jack_be_quick_qui_touche_vole_un_allie_jusqu_a_la_fin_de_la_phase_d_action() {
    // Essai Étincelle : P0 garde une option après le combat.
    let mut t = dromai(&[("Jack Be Quick", R), ETINCELLE, RAMPART, RAMPART], RIEN);
    let wing1 = creer(&mut t, P1, "Aether Ashwing", 1)[0];
    t.g.obj_mut(wing1).tapped = true;
    let c = t.en_main(P0, "Jack Be Quick", R);
    t.choisir(Choice::Play(c));
    // Cible de l'attaque : le héros (l'Ashwing peut aussi être attaquée).
    if t.kind() == DecisionKind::AttackTarget {
        t.choisir(Choice::Target(Target::Player(P1)));
    }
    payer(&mut t);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(wing1).controller, P0);
    assert!(!t.g.obj(wing1).tapped);
    // P0 contrôle un dragon et Storm of Sandikai : il peut attaquer avec.
    assert!(t.g.attack_granted(wing1));
    // Fin de la phase d'action : l'allié revient à P1.
    passer_jusqua_que(&mut t, |t| {
        t.kind() == DecisionKind::Arsenal || t.g.turn > 1
    });
    assert_eq!(t.g.obj(wing1).controller, P1);
}

#[test]
fn nimblism_la_prochaine_attaque_de_cout_1_ou_moins_a_3_de_puissance_en_plus() {
    let mut t = dromai(
        &[("Nimblism", R), ("Sweeping Blow", R), RAMPART, RAMPART],
        RIEN,
    );
    jouer(&mut t, P0, "Nimblism", R);
    resoudre(&mut t);
    assert_eq!(pa(&t, P0), 1);
    let c = jouer(&mut t, P0, "Sweeping Blow", R);
    assert_eq!(t.g.power(c), 6);
}

#[test]
fn nimblism_ne_s_applique_pas_a_une_attaque_de_cout_2() {
    let mut t = dromai(
        &[("Nimblism", R), ("Essai Écrasement", R), RAMPART, RAMPART],
        RIEN,
    );
    jouer(&mut t, P0, "Nimblism", R);
    resoudre(&mut t);
    // Essai Écrasement coûte 2 : pas de bonus…
    let c = attaquer(&mut t, P0, "Essai Écrasement", R);
    assert_eq!(t.g.power(c), 6);
    // … qui reste pour la prochaine attaque de coût 1 ou moins.
    assert_eq!(t.g.next_attack_effects.len(), 1);
}

#[test]
fn oasis_respite_previent_4_degats_d_une_source_choisie() {
    let mut t = partie_vraie(
        [HEROS, DROMAI],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Oasis Respite", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let a = attaquer(&mut t, P0, "Essai Frappe", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction) && t.joueur() == P1
    });
    let o = t.en_main(P1, "Oasis Respite", R);
    t.choisir(Choice::Play(o));
    if t.kind() == DecisionKind::Target {
        t.choisir(Choice::Target(Target::Player(P1)));
    }
    payer(&mut t);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(a));
    resoudre(&mut t);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn oasis_respite_le_heros_qui_a_moins_de_pv_peut_gagner_1_pv() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[("Oasis Respite", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    t.g.player_mut(P0).life = 15;
    t.g.refresh_priority();
    let o = t.en_main(P0, "Oasis Respite", R);
    t.choisir(Choice::Play(o));
    t.choisir(Choice::Target(Target::Player(P0)));
    payer(&mut t);
    jusqua_question(&mut t);
    let src = t.options()[0];
    repondre(&mut t, src);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    assert_eq!(t.vie(P0), 16);
}

#[test]
fn oasis_respite_pas_de_pv_si_le_heros_n_a_pas_moins_de_pv() {
    let mut t = partie_vraie(
        [DROMAI, HEROS],
        &[("Oasis Respite", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let o = t.en_main(P0, "Oasis Respite", R);
    t.choisir(Choice::Play(o));
    t.choisir(Choice::Target(Target::Player(P0)));
    payer(&mut t);
    jusqua_question(&mut t);
    let src = t.options()[0];
    repondre(&mut t, src);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn outed_injouable_si_vous_etes_marque() {
    let mut t = dromai(&[("Outed", R), RAMPART, RAMPART, RAMPART], RIEN);
    t.g.mark(P0);
    t.g.refresh_priority();
    let c = t.en_main(P0, "Outed", R);
    assert!(!t.peut(Choice::Play(c)));
}

#[test]
fn outed_plus_1_si_le_heros_qui_defend_est_marque_et_go_again() {
    let mut t = dromai(&[("Outed", R), RAMPART, RAMPART, RAMPART], RIEN);
    t.g.mark(P1);
    t.g.refresh_priority();
    let c = attaquer(&mut t, P0, "Outed", R);
    assert_eq!(t.g.power(c), 4);
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn outed_sans_marque_3_de_puissance() {
    let mut t = dromai(&[("Outed", R), RAMPART, RAMPART, RAMPART], RIEN);
    let c = attaquer(&mut t, P0, "Outed", R);
    assert_eq!(t.g.power(c), 3);
}

#[test]
fn snatch_quand_elle_touche_pioche_une_carte() {
    let mut t = dromai(&[("Snatch", R), RAMPART, RAMPART, RAMPART], RIEN);
    attaquer(&mut t, P0, "Snatch", R);
    let main = t.g.player(P0).hand.len();
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).hand.len(), main + 1);
}
