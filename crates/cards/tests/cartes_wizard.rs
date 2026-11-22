//! Tests des vraies cartes du deck Iyslander (Wizard, Elemental, Ice et
//! génériques) : un test par phrase du texte de chaque carte, plus des cas
//! limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Event, ObjectId, PlayerId, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const IYS: &str = "Iyslander";
const HEROS: &str = "Essai Héros";
/// Instant toujours jouable (avec un Rempart à pitcher) : le joueur garde
/// une option, le moteur ne passe pas tout seul (fin de tour automatique).
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

/// Le joueur joue cette carte (main ou arsenal) dès qu'il le peut, choisit
/// la cible si on la lui demande, et paie.
fn jouer_obj(t: &mut Partie, p: PlayerId, c: ObjectId, cible: Option<Target>) {
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    if let Some(cible) = cible {
        if t.kind() == DecisionKind::Target {
            t.choisir(Choice::Target(cible));
        }
    }
    payer(t);
}

fn jouer(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    jouer_obj(t, p, c, None);
    c
}

/// Joue une carte qui vise le héros de `cible`.
fn jouer_sur(
    t: &mut Partie,
    p: PlayerId,
    nom: &str,
    couleur: Option<Color>,
    cible: PlayerId,
) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    jouer_obj(t, p, c, Some(Target::Player(cible)));
    c
}

/// Les deux joueurs passent jusqu'à ce que la pile soit vide.
fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.g.stack.is_empty() && t.g.tasks.is_empty() && t.kind() != DecisionKind::Effect
    });
}

/// Iyslander (P0) contre Essai Héros (P1).
fn iys(
    main0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_vraie([IYS, HEROS], main0, RIEN, main1, RIEN, arena0, arena1)
}

fn nb_frostbites(t: &Partie, p: PlayerId) -> usize {
    let ps = t.g.player(p);
    t.g.permanents
        .iter()
        .copied()
        .filter(|&o| t.g.obj(o).controller == p)
        .chain(ps.equipment())
        .filter(|&o| t.g.def(o).name == "Frostbite")
        .count()
}

fn creer_token(t: &mut Partie, p: PlayerId, nom: &str) {
    let id = t.g.db.find_name(nom).unwrap();
    t.g.create_tokens(p, id, 1);
    t.g.refresh_priority();
}

/// Met une carte de la main de `p` dans son arsenal (préparation d'un test).
fn en_arsenal(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    t.g.put_in_arsenal(c);
    t.g.refresh_priority();
    c
}

/// Fin du tour de P0 (tour 1) : on arrive au tour de P1.
fn tour_de_p1(t: &mut Partie) {
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P1);
}

// ---------------------------------------------------------------------------
// Iyslander
// ---------------------------------------------------------------------------

#[test]
fn iyslander_joue_une_action_bleue_non_attaque_de_l_arsenal_pendant_le_tour_adverse() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let f = en_arsenal(&mut t, P0, "Frosting", B);
    tour_de_p1(&mut t);
    jouer_obj(&mut t, P0, f, Some(Target::Player(P1)));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn iyslander_pas_d_action_bleue_depuis_la_main_pendant_le_tour_adverse() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let f = t.en_main(P0, "Frosting", B);
    tour_de_p1(&mut t);
    assert!(!t.g.legal_priority_options(P0).contains(&Choice::Play(f)));
}

#[test]
fn iyslander_pas_d_action_rouge_depuis_l_arsenal_pendant_le_tour_adverse() {
    let mut t = iys(
        &[("Ice Bolt", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let b = en_arsenal(&mut t, P0, "Ice Bolt", R);
    tour_de_p1(&mut t);
    assert!(!t.g.legal_priority_options(P0).contains(&Choice::Play(b)));
}

#[test]
fn iyslander_pendant_son_tour_l_action_de_l_arsenal_coute_un_point_d_action() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let f = en_arsenal(&mut t, P0, "Frosting", B);
    jouer_obj(&mut t, P0, f, Some(Target::Player(P1)));
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn iyslander_carte_ice_jouee_pendant_le_tour_adverse_cree_un_frostbite_chez_lui() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    let f = en_arsenal(&mut t, P0, "Frosting", B);
    tour_de_p1(&mut t);
    jouer_obj(&mut t, P0, f, Some(Target::Player(P1)));
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 1);
    assert_eq!(nb_frostbites(&t, P0), 0);
}

#[test]
fn iyslander_carte_ice_jouee_pendant_son_tour_pas_de_frostbite() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 0);
}

#[test]
fn iyslander_carte_non_ice_jouee_pendant_le_tour_adverse_pas_de_frostbite() {
    let mut t = iys(
        &[("Arcane Twining", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let a = en_arsenal(&mut t, P0, "Arcane Twining", B);
    tour_de_p1(&mut t);
    jouer_obj(&mut t, P0, a, Some(Target::Player(P1)));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(nb_frostbites(&t, P1), 0);
}

#[test]
fn iyslander_instant_ice_depuis_la_main_pendant_le_tour_adverse_cree_un_frostbite() {
    // Toutes les zones d'équipement sont occupées : Frost Spike ne crée rien,
    // seul Iyslander crée un Frostbite.
    let mut t = iys(
        &[("Frost Spike", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[
            "Blade Beckoner Helm",
            "Spellfire Cloak",
            "Blade Beckoner Gauntlets",
            "Aetherstorm Wellingtons",
        ],
        &[
            "Essai Casque",
            "Essai Plastron",
            "Essai Gantelets",
            "Essai Bottes",
        ],
    );
    tour_de_p1(&mut t);
    jouer(&mut t, P0, "Frost Spike", B);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 1);
}

#[test]
fn iyslander_a_18_pv() {
    let t = iys(&[RAMPART; 4], &[RAMPART; 4], &[], &[]);
    assert_eq!(t.vie(P0), 18);
}

// ---------------------------------------------------------------------------
// Frostbite
// ---------------------------------------------------------------------------

#[test]
fn frostbite_les_cartes_coutent_un_de_plus() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let f = t.en_main(P0, "Frosting", B);
    assert_eq!(t.g.play_cost(P0, f), 0);
    creer_token(&mut t, P0, "Frostbite");
    assert_eq!(t.g.play_cost(P0, f), 1);
}

#[test]
fn frostbite_les_capacites_coutent_un_de_plus() {
    let mut t = iys(
        &[RAMPART; 4],
        &[RAMPART; 4],
        &["Crucible of Aetherweave"],
        &[],
    );
    let c = t.equipement(P0, "Crucible of Aetherweave");
    assert_eq!(t.g.activation_cost(c, 0), 1);
    creer_token(&mut t, P0, "Frostbite");
    assert_eq!(t.g.activation_cost(c, 0), 2);
}

#[test]
fn frostbite_ne_touche_pas_les_couts_de_l_autre_joueur() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let f = t.en_main(P0, "Frosting", B);
    creer_token(&mut t, P1, "Frostbite");
    assert_eq!(t.g.play_cost(P0, f), 0);
}

#[test]
fn frostbite_detruit_quand_son_controleur_joue_une_carte() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    creer_token(&mut t, P0, "Frostbite");
    creer_token(&mut t, P0, "Frostbite");
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    // 0 + 2 Frostbite : coût 2, payé avec un Rempart (3).
    assert_eq!(t.g.player(P0).resource_points, 1);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P0), 0, "chaque Frostbite est détruit");
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn frostbite_detruit_quand_son_controleur_active_une_capacite() {
    let mut t = iys(
        &[RAMPART; 4],
        &[RAMPART; 4],
        &["Crucible of Aetherweave"],
        &[],
    );
    creer_token(&mut t, P0, "Frostbite");
    let c = t.equipement(P0, "Crucible of Aetherweave");
    t.choisir(Choice::Activate(c, 0));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P0), 0);
}

#[test]
fn frostbite_reste_quand_l_adversaire_joue_une_carte() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    creer_token(&mut t, P1, "Frostbite");
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 1);
}

#[test]
fn frostbite_detruit_au_debut_de_la_phase_de_fin_de_son_controleur() {
    let mut t = iys(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    creer_token(&mut t, P0, "Frostbite");
    creer_token(&mut t, P1, "Frostbite");
    t.finir_le_tour();
    assert_eq!(nb_frostbites(&t, P0), 0);
    assert_eq!(nb_frostbites(&t, P1), 1, "pas la phase de fin de P1");
    t.finir_le_tour();
    assert_eq!(nb_frostbites(&t, P1), 0);
}

// ---------------------------------------------------------------------------
// Crucible of Aetherweave
// ---------------------------------------------------------------------------

fn activer_crucible(t: &mut Partie) {
    let c = t.equipement(P0, "Crucible of Aetherweave");
    passer_jusqua_que(t, |t| t.joueur() == P0 && t.peut(Choice::Activate(c, 0)));
    t.choisir(Choice::Activate(c, 0));
    payer(t);
    resoudre(t);
}

#[test]
fn crucible_la_prochaine_carte_a_degats_arcaniques_inflige_un_de_plus() {
    let mut t = iys(
        &[("Voltic Bolt", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Crucible of Aetherweave"],
        &[],
    );
    activer_crucible(&mut t);
    jouer_sur(&mut t, P0, "Voltic Bolt", R, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 14);
}

#[test]
fn crucible_une_carte_sans_degats_arcaniques_ne_l_utilise_pas() {
    let mut t = iys(
        &[("Winter's Bite", B), ("Frosting", B), RAMPART, RAMPART],
        &[],
        &["Crucible of Aetherweave"],
        &[],
    );
    // P1 n'a pas de carte : Winter's Bite (sans dégâts, go again) ne demande rien.
    t.g.player_mut(P1).hand.clear();
    activer_crucible(&mut t);
    jouer_sur(&mut t, P0, "Winter's Bite", B, P1);
    resoudre(&mut t);
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18, "Frosting : 1 + 1");
}

#[test]
fn crucible_seulement_la_prochaine_carte() {
    let mut t = iys(
        &[("Frosting", B), ("Frosting", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Crucible of Aetherweave"],
        &[],
    );
    activer_crucible(&mut t);
    let f1 = t.en_main(P0, "Frosting", B);
    jouer_obj(&mut t, P0, f1, Some(Target::Player(P1)));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18);
    // La seconde (jouée au tour suivant de P0) n'a pas de bonus.
    t.finir_le_tour();
    t.finir_le_tour();
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn crucible_une_fois_par_tour() {
    let mut t = iys(
        &[RAMPART; 4],
        &[RAMPART; 4],
        &["Crucible of Aetherweave"],
        &[],
    );
    activer_crucible(&mut t);
    let c = t.equipement(P0, "Crucible of Aetherweave");
    assert!(!t.peut(Choice::Activate(c, 0)));
}

#[test]
fn crucible_effet_perdu_en_fin_de_tour() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Crucible of Aetherweave"],
        &[],
    );
    activer_crucible(&mut t);
    t.finir_le_tour();
    t.finir_le_tour();
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
}

// ---------------------------------------------------------------------------
// Équipements
// ---------------------------------------------------------------------------

#[test]
fn spellfire_cloak_detruire_pour_gagner_une_ressource_pendant_le_tour_adverse() {
    let mut t = iys(
        &[RAMPART; 4],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &["Spellfire Cloak"],
        &[],
    );
    let cape = t.equipement(P0, "Spellfire Cloak");
    tour_de_p1(&mut t);
    passer_jusqua_que(&mut t, |t| t.joueur() == P0);
    t.choisir(Choice::Activate(cape, 0));
    resoudre(&mut t);
    assert_eq!(t.g.obj(cape).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).resource_points, 1);
}

#[test]
fn spellfire_cloak_pas_pendant_son_propre_tour() {
    let t = iys(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &["Spellfire Cloak"],
        &[],
    );
    let cape = t.equipement(P0, "Spellfire Cloak");
    assert!(!t.peut(Choice::Activate(cape, 0)));
}

#[test]
fn spellfire_cloak_arcane_barrier_1() {
    let mut t = partie_vraie(
        [HEROS, IYS],
        &[("Voltic Bolt", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &["Spellfire Cloak"],
    );
    let cape = t.equipement(P1, "Spellfire Cloak");
    jouer_sur(&mut t, P0, "Voltic Bolt", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
    t.choisir(Choice::Prevent(fab_engine::PreventRef::ArcaneBarrier(cape)));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18 - 4);
}

#[test]
fn aetherstorm_wellingtons_arcane_barrier_2() {
    let mut t = partie_vraie(
        [HEROS, IYS],
        &[("Voltic Bolt", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &["Aetherstorm Wellingtons"],
    );
    let bottes = t.equipement(P1, "Aetherstorm Wellingtons");
    jouer_sur(&mut t, P0, "Voltic Bolt", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
    t.choisir(Choice::Prevent(fab_engine::PreventRef::ArcaneBarrier(
        bottes,
    )));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18 - 3);
    // 2 ressources payées avec un Rempart (3) : il en reste 1.
    assert_eq!(t.g.player(P1).resource_points, 1);
}

#[test]
fn blade_beckoner_plating_et_gauntlets_plus_1_contre_une_attaque_d_arme() {
    for nom in ["Blade Beckoner Plating", "Blade Beckoner Gauntlets"] {
        let mut t = partie_vraie(
            ["Hala", HEROS],
            &[ETINCELLE, RAMPART, RAMPART, RAMPART],
            RIEN,
            &[RAMPART; 4],
            RIEN,
            &["Zenith Blade"],
            &[nom],
        );
        let eq = t.equipement(P1, nom);
        let epee = t.equipement(P0, "Zenith Blade");
        t.choisir(Choice::Activate(epee, 0));
        payer(&mut t);
        passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
        t.choisir(Choice::Defend(eq));
        t.choisir(Choice::Pass);
        assert_eq!(t.g.defense(eq), 2, "{nom}");
        passer_jusqua_que(&mut t, |t| !t.g.combat.open);
        // 3 contre 2 : 1 dégât ; guardwell : défense 0 ensuite.
        assert_eq!(t.vie(P1), 19);
        assert_eq!(t.g.defense(eq), 0);
    }
}

#[test]
fn blade_beckoner_plating_pas_de_bonus_contre_une_carte_d_attaque() {
    let mut t = partie_vraie(
        [HEROS, HEROS],
        &[("Essai Frappe", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &["Blade Beckoner Plating"],
    );
    let eq = t.equipement(P1, "Blade Beckoner Plating");
    jouer(&mut t, P0, "Essai Frappe", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    t.choisir(Choice::Defend(eq));
    t.choisir(Choice::Pass);
    assert_eq!(t.g.defense(eq), 1);
}

// ---------------------------------------------------------------------------
// Sorts
// ---------------------------------------------------------------------------

#[test]
fn aethersling_inflige_4_degats_arcaniques() {
    let mut t = iys(
        &[("Aethersling", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Aethersling", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.vie(P1), 16);
    let evs = t.g.turn_events();
    assert!(evs.iter().any(|e| matches!(
        e,
        Event::Damage {
            arcane: true,
            amount: 4,
            ..
        }
    )));
}

#[test]
fn aethersling_engager_le_heros_donne_go_again() {
    let mut t = iys(
        &[("Aethersling", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Aethersling", R, P1);
    assert_eq!(t.g.player(P0).action_points, 0);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P0);
    t.choisir(Choice::Yes);
    resoudre(&mut t);
    let hero = t.g.player(P0).hero;
    assert!(t.g.obj(hero).tapped);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn aethersling_sans_engager_pas_de_go_again() {
    let mut t = iys(
        &[("Aethersling", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Aethersling", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    resoudre(&mut t);
    let hero = t.g.player(P0).hero;
    assert!(!t.g.obj(hero).tapped);
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn aethersling_sans_degats_pas_de_question() {
    let mut t = iys(
        &[("Aethersling", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    t.g.prevent_next(P1, 10, false, None);
    jouer_sur(&mut t, P0, "Aethersling", R, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20);
    assert_eq!(t.g.player(P0).action_points, 0);
}

#[test]
fn aethersling_heros_deja_engage_pas_de_question() {
    let mut t = iys(
        &[("Aethersling", R), ("Aethersling", R), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Aethersling", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Yes);
    resoudre(&mut t);
    jouer_sur(&mut t, P0, "Aethersling", R, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 12);
    assert_eq!(t.g.player(P0).action_points, 0, "pas de second go again");
}

#[test]
fn aethersling_peut_viser_un_allie_mais_voltic_bolt_seulement_un_heros() {
    let mut t = iys(
        &[("Aethersling", R), ("Voltic Bolt", R), ETINCELLE, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    creer_token(&mut t, P1, "Essai Dragon");
    let dragon = *t.g.permanents.last().unwrap();
    let a = t.en_main(P0, "Aethersling", R);
    t.choisir(Choice::Play(a));
    assert!(t.peut(Choice::Target(Target::Object(t.g.obj_ref(dragon)))));
    assert!(t.peut(Choice::Target(Target::Player(P0))));
    t.choisir(Choice::Target(Target::Object(t.g.obj_ref(dragon))));
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    resoudre(&mut t);
    assert_eq!(t.g.obj(dragon).zone, Zone::Gone, "2 PV, 4 dégâts : détruit");
    let v = t.en_main(P0, "Voltic Bolt", R);
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    t.choisir(Choice::Play(v));
    assert_eq!(t.kind(), DecisionKind::Target);
    assert!(t
        .options()
        .iter()
        .all(|c| matches!(c, Choice::Target(Target::Player(_)))));
}

#[test]
fn turn_to_mindfire_5_degats_et_engager_cree_un_ponder() {
    let mut t = iys(
        &[("Turn to Mindfire", R), ETINCELLE, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Turn to Mindfire", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.vie(P1), 15);
    t.choisir(Choice::Yes);
    resoudre(&mut t);
    assert!(t
        .g
        .permanents
        .iter()
        .any(|&o| t.g.def(o).name == "Ponder" && t.g.obj(o).controller == P0));
    assert_eq!(t.g.player(P0).action_points, 0, "pas de go again");
}

#[test]
fn turn_to_mindfire_sans_engager_pas_de_ponder() {
    let mut t = iys(
        &[("Turn to Mindfire", R), ETINCELLE, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Turn to Mindfire", R, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    resoudre(&mut t);
    assert!(t.g.permanents.is_empty());
}

#[test]
fn ponder_detruit_en_phase_de_fin_et_pioche() {
    let mut t = iys(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    creer_token(&mut t, P0, "Ponder");
    t.finir_le_tour();
    assert!(t.g.permanents.is_empty());
    // 4 cartes + 1 pioche de Ponder (puis rien : déjà 4 cartes ou plus).
    assert_eq!(t.g.player(P0).hand.len(), 5);
}

#[test]
fn arcane_twining_et_photon_splicing_bleus_infligent_1_et_2() {
    let mut t = iys(
        &[
            ("Arcane Twining", B),
            ("Photon Splicing", B),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Arcane Twining", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    jouer_sur(&mut t, P0, "Photon Splicing", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn photon_splicing_defausser_pour_amp_1_une_seule_fois() {
    let mut t = iys(
        &[
            ("Photon Splicing", B),
            ("Frosting", B),
            ("Frosting", B),
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let p = t.en_main(P0, "Photon Splicing", B);
    t.choisir(Choice::Activate(p, 0));
    assert_eq!(t.g.obj(p).zone, Zone::Graveyard, "défaussée comme coût");
    resoudre(&mut t);
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18, "1 + amp 1");
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 17, "amp utilisé");
}

#[test]
fn arcane_twining_amp_pendant_le_tour_adverse() {
    let mut t = iys(
        &[("Arcane Twining", B), ("Frosting", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let a = t.en_main(P0, "Arcane Twining", B);
    let f = en_arsenal(&mut t, P0, "Frosting", B);
    tour_de_p1(&mut t);
    passer_jusqua_que(&mut t, |t| t.joueur() == P0);
    t.choisir(Choice::Activate(a, 0));
    resoudre(&mut t);
    jouer_obj(&mut t, P0, f, Some(Target::Player(P1)));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18);
}

#[test]
fn amp_ne_concerne_pas_les_degats_non_arcaniques() {
    let mut t = iys(
        &[
            ("Arcane Twining", B),
            ("Ravenous Rabble", R),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let a = t.en_main(P0, "Arcane Twining", B);
    t.choisir(Choice::Activate(a, 0));
    resoudre(&mut t);
    jouer(&mut t, P0, "Ravenous Rabble", R);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    // Rabble 5 - 3 (Rempart révélé) = 2, sans amp.
    assert_eq!(t.vie(P1), 18);
}

#[test]
fn frosting_bleu_1_degat_a_n_importe_quelle_cible() {
    let mut t = iys(
        &[("Frosting", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Frosting", B, P0);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 17);
}

#[test]
fn ice_bolt_rouge_5_et_bleu_3() {
    let mut t = iys(
        &[("Ice Bolt", R), ("Ice Bolt", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Ice Bolt", R, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 15);
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    jouer_sur(&mut t, P0, "Ice Bolt", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 12);
}

#[test]
fn voltic_bolt_rouge_5_et_bleu_3() {
    let mut t = iys(
        &[("Voltic Bolt", R), ("Voltic Bolt", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Voltic Bolt", R, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 15);
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    jouer_sur(&mut t, P0, "Voltic Bolt", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 12);
}

#[test]
fn timekeepers_whim_3_degats_et_cimetiere_pendant_son_tour() {
    let mut t = iys(
        &[("Timekeeper's Whim", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let w = jouer_sur(&mut t, P0, "Timekeeper's Whim", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(w).zone, Zone::Graveyard);
}

#[test]
fn timekeepers_whim_jouee_pendant_le_tour_adverse_va_sous_le_deck() {
    let mut t = iys(
        &[("Timekeeper's Whim", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let w = en_arsenal(&mut t, P0, "Timekeeper's Whim", B);
    tour_de_p1(&mut t);
    jouer_obj(&mut t, P0, w, Some(Target::Player(P1)));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(w).zone, Zone::Deck);
    assert_eq!(t.g.player(P0).deck[0], w, "sous le deck");
}

// ---------------------------------------------------------------------------
// Ice Eternal
// ---------------------------------------------------------------------------

fn jouer_ice_eternal(t: &mut Partie, x: u8) -> ObjectId {
    let c = t.en_main(P0, "Ice Eternal", B);
    t.choisir(Choice::Play(c));
    assert_eq!(t.kind(), DecisionKind::ChooseX);
    t.choisir(Choice::Number(x));
    t.choisir(Choice::Target(Target::Player(P1)));
    payer(t);
    c
}

#[test]
fn ice_eternal_coute_x_plus_x_et_cree_x_frostbite_sans_degats_sans_fusion() {
    let mut t = iys(
        &[("Ice Eternal", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    // 9 ressources possibles : X va de 0 à 4.
    let c = t.en_main(P0, "Ice Eternal", B);
    t.choisir(Choice::Play(c));
    assert_eq!(t.options().len(), 5);
    t.choisir(Choice::Number(2));
    t.choisir(Choice::Target(Target::Player(P1)));
    payer(&mut t);
    // Coût 4 : deux Remparts pitchés (6), il reste 2.
    assert_eq!(t.g.player(P0).resource_points, 2);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 2);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn ice_eternal_fusionnee_inflige_autant_que_de_frostbite_controles() {
    let mut t = iys(
        &[("Ice Eternal", B), ("Frosting", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    creer_token(&mut t, P1, "Frostbite");
    let frosting = t.en_main(P0, "Frosting", B);
    jouer_ice_eternal(&mut t, 1);
    // Ice Fusion : révéler une carte Ice de la main.
    assert_eq!(t.kind(), DecisionKind::Effect);
    assert_eq!(t.options(), vec![Choice::Object(frosting), Choice::Pass]);
    t.choisir(Choice::Object(frosting));
    assert_eq!(t.g.obj(frosting).zone, Zone::Hand, "révélée, pas défaussée");
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 2);
    assert_eq!(t.vie(P1), 18);
}

#[test]
fn ice_eternal_refuser_la_fusion_pas_de_degats() {
    let mut t = iys(
        &[("Ice Eternal", B), ("Frosting", B), RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_ice_eternal(&mut t, 1);
    t.choisir(Choice::Pass);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P1), 1);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn ice_eternal_sans_carte_ice_en_main_pas_de_fusion() {
    let mut t = iys(
        &[("Ice Eternal", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_ice_eternal(&mut t, 1);
    assert_ne!(t.kind(), DecisionKind::Effect, "pas de question de fusion");
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20);
}

// ---------------------------------------------------------------------------
// Frost Spike
// ---------------------------------------------------------------------------

#[test]
fn frost_spike_cree_un_frostbite_dans_une_zone_exposee_au_choix() {
    let mut t = iys(
        &[("Frost Spike", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[
            "Blade Beckoner Helm",
            "Spellfire Cloak",
            "Blade Beckoner Gauntlets",
            "Aetherstorm Wellingtons",
        ],
        &["Essai Casque"],
    );
    jouer(&mut t, P0, "Frost Spike", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(
        t.options(),
        vec![
            Choice::Zone(P1, Zone::Chest),
            Choice::Zone(P1, Zone::Arms),
            Choice::Zone(P1, Zone::Legs),
        ]
    );
    t.choisir(Choice::Zone(P1, Zone::Arms));
    resoudre(&mut t);
    let fb = t.g.player(P1).arms.expect("Frostbite dans la zone de bras");
    assert_eq!(t.g.def(fb).name, "Frostbite");
    // CR 8.5.2c : sous le contrôle du joueur à qui appartient la zone.
    assert_eq!(t.g.obj(fb).controller, P1);
    assert_eq!(t.g.cost_tax(P1), 1);
    assert_eq!(t.g.cost_tax(P0), 0);
}

#[test]
fn frost_spike_le_frostbite_de_zone_est_detruit_et_libere_la_zone() {
    let mut t = iys(
        &[("Frost Spike", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Frost Spike", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Zone(P1, Zone::Head));
    resoudre(&mut t);
    assert!(t.g.player(P1).head.is_some());
    // Fin du tour de P0, puis phase de fin de P1 : détruit.
    t.finir_le_tour();
    t.finir_le_tour();
    assert!(
        t.g.player(P1).head.is_none(),
        "la zone est de nouveau exposée"
    );
}

#[test]
fn frost_spike_sans_zone_exposee_ne_fait_rien() {
    let mut t = iys(
        &[("Frost Spike", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[
            "Blade Beckoner Helm",
            "Spellfire Cloak",
            "Blade Beckoner Gauntlets",
            "Aetherstorm Wellingtons",
        ],
        &[
            "Essai Casque",
            "Essai Plastron",
            "Essai Gantelets",
            "Essai Bottes",
        ],
    );
    jouer(&mut t, P0, "Frost Spike", B);
    resoudre(&mut t);
    assert_eq!(nb_frostbites(&t, P0) + nb_frostbites(&t, P1), 0);
}

// ---------------------------------------------------------------------------
// Pyroglyphic Protection
// ---------------------------------------------------------------------------

#[test]
fn pyroglyphic_protection_previent_1_de_chaque_degat_arcanique() {
    let mut t = iys(
        &[("Pyroglyphic Protection", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Pyroglyphic Protection", B);
    resoudre(&mut t);
    assert_eq!(t.g.permanents.len(), 1, "une aura");
    t.g.deal_damage(None, P1, P0, 3, true);
    assert_eq!(t.vie(P0), 16);
    t.g.deal_damage(None, P1, P0, 1, true);
    assert_eq!(t.vie(P0), 16, "à chaque fois");
}

#[test]
fn pyroglyphic_protection_pas_les_degats_non_arcaniques() {
    let mut t = iys(
        &[("Pyroglyphic Protection", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Pyroglyphic Protection", B);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 3, false);
    assert_eq!(t.vie(P0), 15);
}

#[test]
fn pyroglyphic_protection_detruite_au_debut_de_sa_phase_d_action() {
    let mut t = iys(
        &[("Pyroglyphic Protection", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Pyroglyphic Protection", B);
    resoudre(&mut t);
    t.finir_le_tour();
    assert_eq!(t.g.permanents.len(), 1, "pas à la phase d'action de P1");
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P0);
    assert!(t.g.permanents.is_empty());
}

// ---------------------------------------------------------------------------
// Stir the Aetherwinds
// ---------------------------------------------------------------------------

#[test]
fn stir_la_prochaine_action_wizard_se_joue_comme_un_instant_avec_plus_1() {
    let mut t = iys(
        &[
            ("Stir the Aetherwinds", B),
            ("Voltic Bolt", R),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Stir the Aetherwinds", B);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 0);
    jouer_sur(&mut t, P0, "Voltic Bolt", R, P1);
    assert_eq!(t.g.player(P0).action_points, 0);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 14, "5 + 1");
}

#[test]
fn stir_seulement_la_prochaine_action_wizard() {
    let mut t = iys(
        &[
            ("Stir the Aetherwinds", B),
            ("Frosting", B),
            ("Frosting", B),
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Stir the Aetherwinds", B);
    resoudre(&mut t);
    jouer_sur(&mut t, P0, "Frosting", B, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18);
    let f2 = t.en_main(P0, "Frosting", B);
    assert!(!t.peut(Choice::Play(f2)), "plus de point d'action");
}

#[test]
fn stir_peut_se_jouer_pendant_le_combat() {
    // La permission permet de jouer l'action quand le joueur a la priorité,
    // par exemple pendant l'étape de réaction (CR 8.1.1d).
    let mut t = iys(
        &[
            ("Stir the Aetherwinds", B),
            ("Ravenous Rabble", R),
            ("Frosting", B),
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Stir the Aetherwinds", B);
    resoudre(&mut t);
    t.g.player_mut(P0).action_points = 1;
    t.g.refresh_priority();
    jouer(&mut t, P0, "Ravenous Rabble", R);
    let f = t.en_main(P0, "Frosting", B);
    passer_jusqua_que(&mut t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction) && t.joueur() == P0
    });
    assert!(t.peut(Choice::Play(f)));
}

#[test]
fn stir_permission_perdue_en_fin_de_tour() {
    let mut t = iys(
        &[
            ("Stir the Aetherwinds", B),
            ("Frosting", B),
            RAMPART,
            RAMPART,
        ],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer(&mut t, P0, "Stir the Aetherwinds", B);
    resoudre(&mut t);
    let f = t.en_main(P0, "Frosting", B);
    tour_de_p1(&mut t);
    assert!(!t.g.legal_priority_options(P0).contains(&Choice::Play(f)));
}

// ---------------------------------------------------------------------------
// Winter's Bite
// ---------------------------------------------------------------------------

#[test]
fn winters_bite_le_heros_cible_paie_et_ne_defausse_pas() {
    let mut t = iys(
        &[("Winter's Bite", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    jouer_sur(&mut t, P0, "Winter's Bite", B, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P1);
    t.choisir(Choice::Yes);
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(t.g.player(P1).resource_points, 2);
    assert!(t.g.player(P1).graveyard.is_empty());
    assert_eq!(t.g.player(P1).hand.len(), 3);
}

#[test]
fn winters_bite_sans_payer_il_defausse_la_carte_choisie() {
    let mut t = iys(
        &[("Winter's Bite", B), RAMPART, RAMPART, RAMPART],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    let frappe = t.en_main(P1, "Essai Frappe", R);
    jouer_sur(&mut t, P0, "Winter's Bite", B, P1);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    assert_eq!(t.joueur(), P1);
    t.choisir(Choice::Object(frappe));
    resoudre(&mut t);
    assert_eq!(t.g.obj(frappe).zone, Zone::Graveyard);
}

#[test]
fn winters_bite_go_again() {
    let mut t = iys(
        &[("Winter's Bite", B), RAMPART, RAMPART, RAMPART],
        &[],
        &[],
        &[],
    );
    // P1 n'a aucune carte : rien à défausser, pas de question.
    t.g.player_mut(P1).hand.clear();
    jouer_sur(&mut t, P0, "Winter's Bite", B, P1);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1);
}

// ---------------------------------------------------------------------------
// Génériques du deck
// ---------------------------------------------------------------------------

#[test]
fn ravenous_rabble_perd_la_valeur_de_pitch_de_la_carte_revelee() {
    for (dessus, attendu) in [(RAMPART, 2), (("Essai Frappe", R), 4)] {
        let mut t = partie_vraie(
            [HEROS, HEROS],
            &[("Ravenous Rabble", R), RAMPART, RAMPART, RAMPART],
            &[dessus],
            &[RAMPART; 4],
            RIEN,
            &[],
            &[],
        );
        let r = jouer(&mut t, P0, "Ravenous Rabble", R);
        passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
        assert_eq!(t.g.power(r), attendu);
        assert!(t
            .g
            .turn_events()
            .iter()
            .any(|e| matches!(e, Event::Revealed { player, .. } if *player == P0)));
        // La carte révélée reste sur le deck.
        assert_eq!(t.g.player(P0).deck.len(), 40 - 4);
    }
}

#[test]
fn ravenous_rabble_go_again() {
    let mut t = partie_vraie(
        [HEROS, HEROS],
        &[("Ravenous Rabble", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &[],
    );
    jouer(&mut t, P0, "Ravenous Rabble", R);
    passer_jusqua_que(&mut t, |t| {
        !t.g.combat.open
            || t.g.combat.links.len() == 1
                && t.g.combat.step == Some(fab_engine::state::CombatStep::Resolution)
    });
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn wounded_bull_plus_1_si_moins_de_vie() {
    for (vie, attendu) in [(10, 8), (20, 7)] {
        let mut t = partie_vraie(
            [HEROS, HEROS],
            &[("Wounded Bull", R), RAMPART, RAMPART, RAMPART],
            RIEN,
            &[RAMPART; 4],
            RIEN,
            &[],
            &[],
        );
        t.g.player_mut(P0).life = vie;
        let b = jouer(&mut t, P0, "Wounded Bull", R);
        passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
        assert_eq!(t.g.power(b), attendu, "vie {vie}");
    }
}

#[test]
fn unmovable_plus_1_si_jouee_depuis_l_arsenal() {
    for (arsenal, attendu) in [(true, 8), (false, 7)] {
        let mut t = partie_vraie(
            [HEROS, HEROS],
            &[("Essai Assaut", R), ETINCELLE, RAMPART, RAMPART],
            RIEN,
            &[("Unmovable", R), RAMPART, RAMPART, RAMPART],
            RIEN,
            &[],
            &[],
        );
        let u = if arsenal {
            en_arsenal(&mut t, P1, "Unmovable", R)
        } else {
            t.en_main(P1, "Unmovable", R)
        };
        assert_eq!(t.g.defense(u), 7);
        jouer(&mut t, P0, "Essai Assaut", R);
        jouer_obj(&mut t, P1, u, None);
        resoudre(&mut t);
        assert_eq!(t.g.obj(u).zone, Zone::CombatChain);
        assert_eq!(t.g.defense(u), attendu, "depuis l'arsenal : {arsenal}");
    }
}

#[test]
fn timesnap_potion_detruire_pour_gagner_2_points_d_action() {
    let mut t = partie_vraie(
        [HEROS, HEROS],
        &[("Timesnap Potion", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        RIEN,
        &[],
        &[],
    );
    let p = jouer(&mut t, P0, "Timesnap Potion", B);
    resoudre(&mut t);
    assert_eq!(t.g.obj(p).zone, Zone::Permanent, "un objet (item)");
    assert_eq!(t.g.player(P0).action_points, 0);
    assert!(
        !t.peut(Choice::Activate(p, 0)),
        "action : il faut un point d'action"
    );
    t.finir_le_tour();
    t.finir_le_tour();
    t.choisir(Choice::Activate(p, 0));
    assert_eq!(t.g.obj(p).zone, Zone::Graveyard, "détruite comme coût");
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 2);
}
