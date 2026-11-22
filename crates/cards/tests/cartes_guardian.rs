//! Tests des vraies cartes du deck Bravo, Flattering Showman (Guardian et
//! génériques) : un test par phrase du texte de chaque carte, plus des cas
//! limites.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Delta, Event, Keywords, ObjectId, PlayerId, Until, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const BRAVO: &str = "Bravo, Flattering Showman";
const HEROS: &str = "Essai Héros";

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

/// Le joueur joue cette carte de sa main dès qu'il le peut, et paie.
fn jouer(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    payer(t);
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

/// Le joueur attaque avec son arme ; renvoie l'attaque-proxy (active).
fn attaquer_arme(t: &mut Partie, p: PlayerId, arme: &str) -> ObjectId {
    let w = t.equipement(p, arme);
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

fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.g.stack.is_empty() && t.g.pending_triggers.is_empty()
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
}

/// Nombre de permanents de ce nom contrôlés par le joueur.
fn jetons(t: &Partie, p: PlayerId, nom: &str) -> usize {
    t.g.permanents
        .iter()
        .filter(|&&o| t.g.obj(o).controller == p && t.g.def(o).name == nom)
        .count()
}

/// Avance jusqu'au tour `n`, une fois ses déclenchements de début résolus
/// (le moteur passe seul quand passer est la seule option : on s'arrête à la
/// première décision du tour `n`).
fn debut_du_tour(t: &mut Partie, n: u32) {
    while t.g.turn < n {
        t.finir_le_tour();
    }
    passer_jusqua_que(t, |t| {
        t.g.turn != n || (t.g.stack.is_empty() && t.g.pending_triggers.is_empty())
    });
    assert_eq!(t.g.turn, n);
}

/// Un clash a-t-il eu lieu ce tour (cartes révélées) ?
fn clash_revele(t: &Partie) -> bool {
    t.g.turn_events()
        .iter()
        .any(|e| matches!(e, Event::Revealed { .. }))
}

#[allow(clippy::too_many_arguments)]
fn jeu(
    heros: [&str; 2],
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    partie_vraie(heros, main0, suite0, main1, suite1, arena0, arena1)
}

/// Essai Héros contre Essai Héros.
fn duel(
    main0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    arena0: &[&str],
    arena1: &[&str],
) -> Partie {
    jeu([HEROS, HEROS], main0, RIEN, main1, RIEN, arena0, arena1)
}

// ---------------------------------------------------------------------------
// Bravo, Flattering Showman
// ---------------------------------------------------------------------------

#[test]
fn bravo_retourne_la_carte_de_l_arsenal_et_une_carte_avec_crush_gagne_2_et_dominate() {
    let mut t = jeu(
        [BRAVO, HEROS],
        &[("Boulder Drop", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let bd = t.en_main(P0, "Boulder Drop", R);
    t.g.put_in_arsenal(bd);
    t.g.refresh_priority();
    assert!(t.g.obj(bd).face_down);
    let hero = t.g.player(P0).hero;
    t.choisir(Choice::Activate(hero, 0));
    payer(&mut t);
    resoudre(&mut t);
    assert!(!t.g.obj(bd).face_down, "retournée face visible");
    assert_eq!(t.g.power(bd), 9, "7 + 2");
    assert!(t.g.keywords(bd).has(Keywords::DOMINATE));
    assert!(t.g.obj(hero).tapped, "coût {{t}}");
    assert_eq!(t.g.player(P0).resource_points, 1, "3 pitchées - 2");
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    // Jouée depuis l'arsenal ce tour, elle garde le bonus.
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(bd)));
    t.choisir(Choice::Play(bd));
    payer(&mut t);
    jusqua_active(&mut t, bd);
    let a = bd;
    assert_eq!(t.g.power(a), 9);
    assert!(t.g.keywords(a).has(Keywords::DOMINATE));
}

#[test]
fn bravo_une_carte_sans_crush_est_seulement_retournee() {
    let mut t = jeu(
        [BRAVO, HEROS],
        &[("Macho Grande", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let m = t.en_main(P0, "Macho Grande", B);
    t.g.put_in_arsenal(m);
    t.g.refresh_priority();
    let hero = t.g.player(P0).hero;
    t.choisir(Choice::Activate(hero, 0));
    payer(&mut t);
    resoudre(&mut t);
    assert!(!t.g.obj(m).face_down);
    assert_eq!(t.g.power(m), 8, "pas de bonus");
}

#[test]
fn bravo_le_bonus_dure_jusqu_a_la_fin_du_tour() {
    let mut t = jeu(
        [BRAVO, HEROS],
        &[("Boulder Drop", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let bd = t.en_main(P0, "Boulder Drop", R);
    t.g.put_in_arsenal(bd);
    t.g.refresh_priority();
    let hero = t.g.player(P0).hero;
    t.choisir(Choice::Activate(hero, 0));
    payer(&mut t);
    resoudre(&mut t);
    t.finir_le_tour();
    assert_eq!(t.g.power(bd), 7);
    assert!(!t.g.keywords(bd).has(Keywords::DOMINATE));
}

#[test]
fn bravo_capacite_indisponible_sans_carte_face_cachee_en_arsenal() {
    let mut t = jeu(
        [BRAVO, HEROS],
        &[("Boulder Drop", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let hero = t.g.player(P0).hero;
    assert!(!t.peut(Choice::Activate(hero, 0)), "arsenal vide");
    let bd = t.en_main(P0, "Boulder Drop", R);
    t.g.put_in_arsenal(bd);
    t.g.refresh_priority();
    assert!(t.peut(Choice::Activate(hero, 0)), "carte face cachée");
    t.g.turn_face_up(bd);
    t.g.refresh_priority();
    assert!(
        !t.peut(Choice::Activate(hero, 0)),
        "carte déjà face visible"
    );
}

// ---------------------------------------------------------------------------
// Miller's Grindstone
// ---------------------------------------------------------------------------

#[test]
fn millers_grindstone_attaque_pour_3_ressources_une_fois_par_tour() {
    let mut t = jeu(
        [HEROS, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Miller's Grindstone"],
        &[],
    );
    let w = t.equipement(P0, "Miller's Grindstone");
    let proxy = attaquer_arme(&mut t, P0, "Miller's Grindstone");
    assert_eq!(t.g.power(proxy), 4);
    assert_eq!(t.g.player(P0).resource_points, 0, "3 pitchées - 3");
    assert!(!t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    assert!(!t.peut(Choice::Activate(w, 0)));
}

#[test]
fn millers_grindstone_clash_gagne_detruit_le_dessus_du_deck_adverse() {
    let mut t = jeu(
        [HEROS, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Boulder Drop", R)],
        RIEN,
        RIEN,
        &["Miller's Grindstone"],
        &[],
    );
    let haut1 = t.g.top_of_deck(P1).unwrap();
    let deck1 = t.g.player(P1).deck.len();
    attaquer_arme(&mut t, P0, "Miller's Grindstone");
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 16);
    assert!(clash_revele(&t));
    assert_eq!(
        t.g.obj(haut1).zone,
        Zone::Graveyard,
        "carte du dessus détruite"
    );
    assert_eq!(t.g.player(P1).deck.len(), deck1 - 1);
    let w = t.equipement(P0, "Miller's Grindstone");
    assert_eq!(t.g.power(w), 4, "pas de compteur");
}

#[test]
fn millers_grindstone_clash_perdu_compteur_moins_un() {
    let mut t = jeu(
        [HEROS, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[RAMPART; 4],
        &[("Essai Frappe", R)],
        &["Miller's Grindstone"],
        &[],
    );
    let deck1 = t.g.player(P1).deck.len();
    attaquer_arme(&mut t, P0, "Miller's Grindstone");
    fin_du_combat(&mut t);
    let w = t.equipement(P0, "Miller's Grindstone");
    assert_eq!(t.g.obj(w).power_counters, -1);
    assert_eq!(t.g.power(w), 3);
    assert_eq!(t.g.player(P1).deck.len(), deck1, "rien de détruit");
}

#[test]
fn millers_grindstone_egalite_au_clash_rien_ne_se_passe() {
    // Les deux decks révèlent un Essai Rempart (sans puissance).
    let mut t = jeu(
        [HEROS, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Miller's Grindstone"],
        &[],
    );
    let deck1 = t.g.player(P1).deck.len();
    attaquer_arme(&mut t, P0, "Miller's Grindstone");
    fin_du_combat(&mut t);
    let w = t.equipement(P0, "Miller's Grindstone");
    assert_eq!(t.g.power(w), 4);
    assert_eq!(t.g.player(P1).deck.len(), deck1);
}

// ---------------------------------------------------------------------------
// Équipements
// ---------------------------------------------------------------------------

#[test]
fn richter_scale_detruire_cree_2_seismic_surge_et_coute_une_action() {
    let mut t = duel(&[RAMPART; 4], RIEN, &["Richter Scale"], &[]);
    let rs = t.equipement(P0, "Richter Scale");
    t.choisir(Choice::Activate(rs, 0));
    resoudre(&mut t);
    assert_eq!(t.g.obj(rs).zone, Zone::Graveyard);
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 2);
    assert_eq!(t.g.player(P0).action_points, 0, "pas de go again");
}

#[test]
fn richter_scale_battleworn() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &["Richter Scale"],
    );
    let rs = t.equipement(P1, "Richter Scale");
    jouer(&mut t, P0, "Essai Frappe", R);
    defendre_avec(&mut t, P1, &[rs]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.defense(rs), 0);
}

#[test]
fn steelbraid_buckler_temper() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &["Steelbraid Buckler"],
    );
    let sb = t.equipement(P1, "Steelbraid Buckler");
    assert_eq!(t.g.defense(sb), 2);
    jouer(&mut t, P0, "Essai Frappe", R);
    defendre_avec(&mut t, P1, &[sb]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.defense(sb), 1);
    assert_eq!(t.g.obj(sb).zone, Zone::Weapon, "off-hand : zone d'arme");
}

#[test]
fn gauntlet_of_boulderhold_prochaine_attaque_guardian_plus_2_avec_go_again() {
    let mut t = duel(
        &[("Essai Assaut", R), ("Boulder Drop", R), RAMPART, RAMPART],
        RIEN,
        &["Gauntlet of Boulderhold"],
        &[],
    );
    let gb = t.equipement(P0, "Gauntlet of Boulderhold");
    t.choisir(Choice::Activate(gb, 0));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(t.g.obj(gb).zone, Zone::Graveyard, "détruit (coût)");
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    // Une attaque non Guardian ne consomme pas l'effet.
    let a = attaquer(&mut t, P0, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 3);
    fin_du_combat(&mut t);
    let b = attaquer(&mut t, P0, "Boulder Drop", R);
    assert_eq!(t.g.power(b), 9, "7 + 2");
}

#[test]
fn civic_steps_en_defendant_cree_un_quicken_pour_l_autre_heros() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &["Civic Steps"],
    );
    let cs = t.equipement(P1, "Civic Steps");
    jouer(&mut t, P0, "Essai Frappe", R);
    defendre_avec(&mut t, P1, &[cs]);
    resoudre(&mut t);
    assert_eq!(
        jetons(&t, P0, "Quicken"),
        1,
        "sous le contrôle de l'attaquant"
    );
    assert_eq!(jetons(&t, P1, "Quicken"), 0);
    fin_du_combat(&mut t);
    assert_eq!(t.g.defense(cs), 1, "temper");
}

#[test]
fn unflinching_foothold_l_attaque_perd_et_ne_peut_plus_gagner_dominate() {
    let mut t = duel(
        &[("Essai Écrasement", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &["Unflinching Foothold"],
    );
    let uf = t.equipement(P1, "Unflinching Foothold");
    let a = jouer(&mut t, P0, "Essai Écrasement", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.active_attack() == Some(a) && t.joueur() == P1 && t.kind() == DecisionKind::Priority
    });
    assert!(t.g.keywords(a).has(Keywords::DOMINATE));
    t.choisir(Choice::Activate(uf, 0));
    resoudre(&mut t);
    assert_eq!(t.g.obj(uf).zone, Zone::Graveyard);
    assert!(!t.g.keywords(a).has(Keywords::DOMINATE), "perd dominate");
    let r = t.g.obj_ref(a);
    let d = Delta {
        grants: Keywords::DOMINATE,
        ..Default::default()
    };
    t.g.modify(r, d, Until::EndOfTurn);
    assert!(
        !t.g.keywords(a).has(Keywords::DOMINATE),
        "ne peut plus le gagner"
    );
}

// ---------------------------------------------------------------------------
// Tokens
// ---------------------------------------------------------------------------

#[test]
fn seismic_surge_debut_de_phase_d_action_detruit_et_reduit_la_prochaine_attaque_guardian() {
    let mut t = duel(
        &[("Boulder Drop", R), ("Essai Frappe", R), RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P0, "Seismic Surge", 2);
    // Celui de l'adversaire ne se déclenche que dans sa phase d'action.
    creer(&mut t, P1, "Seismic Surge", 1);
    debut_du_tour(&mut t, 2);
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 2, "pas au tour de P1");
    debut_du_tour(&mut t, 3);
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 0, "détruits");
    let bd = t.en_main(P0, "Boulder Drop", R);
    let fr = t.en_main(P0, "Essai Frappe", R);
    assert_eq!(t.g.play_cost(P0, bd), 1, "3 - 2 (les deux réductions)");
    assert_eq!(t.g.play_cost(P0, fr), 1, "pas une attaque Guardian");
    // Jouer une attaque non Guardian ne consomme pas la réduction.
    attaquer(&mut t, P0, "Essai Frappe", R);
    fin_du_combat(&mut t);
    assert_eq!(t.g.play_cost(P0, bd), 1);
}

#[test]
fn seismic_surge_la_reduction_sert_une_fois_et_dure_ce_tour() {
    let mut t = duel(
        &[("Boulder Drop", R), ("Boulder Drop", R), RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P0, "Seismic Surge", 1);
    debut_du_tour(&mut t, 3);
    let bd = t.en_main(P0, "Boulder Drop", R);
    assert_eq!(t.g.play_cost(P0, bd), 2);
    t.choisir(Choice::Play(bd));
    payer(&mut t);
    let autre = t.en_main(P0, "Boulder Drop", R);
    assert_eq!(t.g.play_cost(P0, autre), 3, "réduction utilisée");
    t.finir_le_tour();
    assert!(t.g.cost_reductions.is_empty());
}

#[test]
fn might_debut_du_tour_detruit_et_prochaine_attaque_plus_1() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P0, "Might", 1);
    debut_du_tour(&mut t, 3);
    assert_eq!(jetons(&t, P0, "Might"), 0);
    let a = attaquer(&mut t, P0, "Essai Frappe", R);
    assert_eq!(t.g.power(a), 5);
}

#[test]
fn vigor_debut_du_tour_detruit_et_gagne_une_ressource() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P0, "Vigor", 1);
    debut_du_tour(&mut t, 3);
    assert_eq!(jetons(&t, P0, "Vigor"), 0);
    assert_eq!(t.g.player(P0).resource_points, 1);
}

#[test]
fn quicken_une_carte_d_attaque_jouee_gagne_go_again() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P0, "Quicken", 1);
    let a = attaquer(&mut t, P0, "Essai Frappe", R);
    assert_eq!(jetons(&t, P0, "Quicken"), 0, "détruit");
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1);
}

#[test]
fn quicken_une_attaque_d_arme_gagne_go_again() {
    let mut t = duel(&[RAMPART; 4], RIEN, &["Miller's Grindstone"], &[]);
    creer(&mut t, P0, "Quicken", 1);
    let proxy = attaquer_arme(&mut t, P0, "Miller's Grindstone");
    assert_eq!(jetons(&t, P0, "Quicken"), 0);
    assert!(t.g.keywords(proxy).has(Keywords::GO_AGAIN));
}

#[test]
fn quicken_les_attaques_de_l_adversaire_ne_le_declenchent_pas() {
    let mut t = duel(
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    creer(&mut t, P1, "Quicken", 1);
    let a = attaquer(&mut t, P0, "Essai Frappe", R);
    assert_eq!(jetons(&t, P1, "Quicken"), 1);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

// ---------------------------------------------------------------------------
// Cartes avec crush
// ---------------------------------------------------------------------------

/// P0 (Essai Héros) attaque avec `carte` ; P1 a 4 cartes en main.
fn crush_contre(carte: (&str, Option<Color>), main1: &[(&str, Option<Color>)]) -> Partie {
    duel(&[carte, RAMPART, RAMPART, RAMPART], main1, &[], &[])
}

#[test]
fn les_cartes_avec_crush_ont_le_drapeau_crush() {
    let t = crush_contre(("Boulder Drop", R), RIEN);
    for (n, c) in [
        ("Boulder Drop", R),
        ("Boulder Drop", B),
        ("Buckling Blow", B),
        ("Chokeslam", R),
        ("Chokeslam", B),
        ("Crush Confidence", R),
        ("Debilitate", R),
        ("Debilitate", B),
        ("Disable", B),
        ("Fault Line", R),
        ("Flatten the Field", B),
    ] {
        let id = t.g.db.find(n, c).unwrap();
        assert!(t.g.db.get(id).keywords.has(Keywords::CRUSH), "{n}");
    }
    for n in [
        "Thunk",
        "Macho Grande",
        "Thunder Quake",
        "Clash of Mountains",
    ] {
        let id = t.g.db.find_name(n).unwrap();
        assert!(!t.g.db.get(id).keywords.has(Keywords::CRUSH), "{n}");
    }
}

#[test]
fn boulder_drop_crush_le_heros_remet_une_carte_de_sa_main_sur_son_deck() {
    let mut t = crush_contre(
        ("Boulder Drop", R),
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
    );
    let fr = t.en_main(P1, "Essai Frappe", R);
    attaquer(&mut t, P0, "Boulder Drop", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P1, "le héros touché choisit");
    t.choisir(Choice::Object(fr));
    assert_eq!(t.g.top_of_deck(P1), Some(fr));
    assert_eq!(t.g.player(P1).hand.len(), 3);
    assert_eq!(t.vie(P1), 13);
}

#[test]
fn boulder_drop_moins_de_4_degats_pas_de_crush() {
    let mut t = crush_contre(("Boulder Drop", R), &[RAMPART; 4]);
    attaquer(&mut t, P0, "Boulder Drop", R);
    let r = t.en_main(P1, "Essai Rempart", B);
    defendre_avec(&mut t, P1, &[r]);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17, "7 - 4 = 3 dégâts");
    assert_eq!(t.g.player(P1).hand.len(), 3, "rien remis sur le deck");
}

#[test]
fn boulder_drop_bleu_5_de_puissance() {
    let mut t = crush_contre(("Boulder Drop", B), RIEN);
    let a = attaquer(&mut t, P0, "Boulder Drop", B);
    assert_eq!(t.g.power(a), 5);
}

#[test]
fn crush_ne_compte_que_les_degats_infliges_apres_prevention() {
    // CR 8.4.2a : la condition porte sur l'événement de dégâts.
    let mut t = crush_contre(
        ("Boulder Drop", R),
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
    );
    t.g.prevent_next(P1, 4, false, None);
    attaquer(&mut t, P0, "Boulder Drop", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17, "7 - 4 prévenus");
    assert_eq!(t.g.player(P1).hand.len(), 4, "3 dégâts : pas de crush");
}

#[test]
fn buckling_blow_crush_compteur_moins_un_sur_un_equipement_du_heros() {
    let mut t = duel(
        &[("Buckling Blow", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &["Blade Beckoner Helm"],
    );
    let helm = t.equipement(P1, "Blade Beckoner Helm");
    attaquer(&mut t, P0, "Buckling Blow", B);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 14);
    assert_eq!(t.g.obj(helm).defense_counters, -1);
    assert_eq!(t.g.defense(helm), 0);
}

#[test]
fn chokeslam_les_attaques_du_heros_ne_gagnent_pas_de_puissance_a_sa_prochaine_phase_d_action() {
    let mut t = crush_contre(
        ("Chokeslam", R),
        &[("Essai Frappe", R), ("Essai Frappe", R), RAMPART, RAMPART],
    );
    attaquer(&mut t, P0, "Chokeslam", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 12);
    creer(&mut t, P1, "Might", 1);
    debut_du_tour(&mut t, 2);
    let a = attaquer(&mut t, P1, "Essai Frappe", R);
    assert_eq!(t.g.power(a), 4, "le +1 de Might ne s'applique pas");
    // Un malus s'applique toujours.
    let r = t.g.obj_ref(a);
    t.g.modify(
        r,
        fab_engine::Delta {
            power: -1,
            ..Default::default()
        },
        Until::EndOfTurn,
    );
    assert_eq!(t.g.power(a), 3);
    t.finir_le_tour();
    assert!(t.g.no_power_gain.is_empty(), "effet terminé");
}

#[test]
fn chokeslam_bleu_6_de_puissance() {
    let mut t = crush_contre(("Chokeslam", B), RIEN);
    let a = attaquer(&mut t, P0, "Chokeslam", B);
    assert_eq!(t.g.power(a), 6);
}

#[test]
fn crush_confidence_le_heros_perd_ses_capacites_jusqu_a_la_fin_de_son_prochain_tour() {
    let mut t = crush_contre(("Crush Confidence", R), RIEN);
    attaquer(&mut t, P0, "Crush Confidence", R);
    fin_du_combat(&mut t);
    let h = t.g.player(P1).hero;
    assert!(t.g.lost_abilities(h));
    t.finir_le_tour();
    assert!(t.g.lost_abilities(h), "pendant le tour de P1");
    t.finir_le_tour();
    assert!(!t.g.lost_abilities(h), "fin du prochain tour de P1");
}

#[test]
fn debilitate_la_premiere_attaque_du_prochain_tour_du_heros_a_moins_2() {
    let mut t = crush_contre(
        ("Debilitate", R),
        &[("Essai Assaut", R), ("Essai Frappe", R), RAMPART, RAMPART],
    );
    attaquer(&mut t, P0, "Debilitate", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 12);
    debut_du_tour(&mut t, 2);
    let a = attaquer(&mut t, P1, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 1, "3 - 2");
    fin_du_combat(&mut t);
    let b = attaquer(&mut t, P1, "Essai Frappe", R);
    assert_eq!(t.g.power(b), 4, "seulement la première attaque");
}

#[test]
fn debilitate_bleu_6_de_puissance() {
    let mut t = crush_contre(("Debilitate", B), RIEN);
    let a = attaquer(&mut t, P0, "Debilitate", B);
    assert_eq!(t.g.power(a), 6);
}

#[test]
fn disable_crush_la_carte_d_arsenal_du_heros_va_sous_son_deck() {
    let mut t = crush_contre(
        ("Disable", B),
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
    );
    let fr = t.en_main(P1, "Essai Frappe", R);
    t.g.put_in_arsenal(fr);
    attaquer(&mut t, P0, "Disable", B);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 13);
    assert_eq!(t.g.player(P1).arsenal, None);
    assert_eq!(t.g.player(P1).deck.first(), Some(&fr), "sous le deck");
}

#[test]
fn fault_line_plus_1_avec_une_carte_en_arsenal() {
    let mut t = crush_contre(("Fault Line", R), RIEN);
    let fl = t.en_main(P0, "Fault Line", R);
    assert_eq!(t.g.power(fl), 7);
    let r = t.en_main(P0, "Essai Rempart", B);
    t.g.put_in_arsenal(r);
    assert_eq!(t.g.power(fl), 8);
}

#[test]
fn fault_line_crush_toutes_les_cartes_d_arsenal_vont_sous_le_deck() {
    let mut t = crush_contre(
        ("Fault Line", R),
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
    );
    let r0 = t.en_main(P0, "Essai Rempart", B);
    t.g.put_in_arsenal(r0);
    let fr = t.en_main(P1, "Essai Frappe", R);
    t.g.put_in_arsenal(fr);
    attaquer(&mut t, P0, "Fault Line", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 12, "7 + 1");
    assert_eq!(t.g.player(P0).arsenal, None);
    assert_eq!(t.g.player(P1).arsenal, None);
    assert_eq!(t.g.player(P0).deck.first(), Some(&r0));
    assert_eq!(t.g.player(P1).deck.first(), Some(&fr));
}

#[test]
fn flatten_the_field_crush_detruit_un_seismic_surge_du_heros() {
    let mut t = crush_contre(("Flatten the Field", B), RIEN);
    creer(&mut t, P1, "Seismic Surge", 2);
    creer(&mut t, P0, "Seismic Surge", 1);
    attaquer(&mut t, P0, "Flatten the Field", B);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 14);
    assert_eq!(jetons(&t, P1, "Seismic Surge"), 1);
    assert_eq!(
        jetons(&t, P0, "Seismic Surge"),
        1,
        "pas ceux de l'attaquant"
    );
}

// ---------------------------------------------------------------------------
// Autres attaques
// ---------------------------------------------------------------------------

#[test]
fn macho_grande_dominate() {
    let mut t = duel(
        &[("Macho Grande", B), RAMPART, RAMPART, RAMPART],
        &[RAMPART; 4],
        &[],
        &[],
    );
    let a = attaquer(&mut t, P0, "Macho Grande", B);
    assert!(t.g.keywords(a).has(Keywords::DOMINATE));
    assert_eq!(t.g.power(a), 8);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let r = t.en_main(P1, "Essai Rempart", B);
    t.choisir(Choice::Defend(r));
    let autre = t.en_main(P1, "Essai Rempart", B);
    assert!(!t.peut(Choice::Defend(autre)), "une seule carte de la main");
}

#[test]
fn thunder_quake_heave_paie_3_va_en_arsenal_face_visible_et_cree_3_seismic_surge() {
    let mut t = duel(
        &[("Thunder Quake", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let tq = t.en_main(P0, "Thunder Quake", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P0);
    t.choisir(Choice::Yes);
    assert_eq!(t.kind(), DecisionKind::Pitch);
    assert!(
        !t.peut(Choice::Pitch(tq)),
        "Thunder Quake ne paie pas son propre heave"
    );
    payer(&mut t);
    assert_eq!(t.g.player(P0).arsenal, Some(tq));
    assert!(!t.g.obj(tq).face_down, "face visible");
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 3);
}

#[test]
fn thunder_quake_heave_refuse_la_carte_reste_en_main() {
    let mut t = duel(
        &[("Thunder Quake", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let tq = t.en_main(P0, "Thunder Quake", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    assert_eq!(t.g.obj(tq).zone, Zone::Hand);
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 0);
}

#[test]
fn thunder_quake_pas_de_heave_si_l_arsenal_est_occupe() {
    let mut t = duel(
        &[("Thunder Quake", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let r = t.en_main(P0, "Essai Rempart", B);
    t.g.put_in_arsenal(r);
    // Aucune question de heave : le tour passe directement.
    passer_jusqua_que(&mut t, |t| t.g.turn == 2);
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 0);
}

#[test]
fn thunder_quake_pas_de_heave_sans_3_ressources_hors_de_la_carte() {
    let mut t = duel(
        &[
            ("Thunder Quake", B),
            ("Essai Frappe", R),
            ("Essai Frappe", R),
            ("Essai Assaut", R),
        ],
        RIEN,
        &[],
        &[],
    );
    // Essai Assaut ne coûte rien ; il reste deux Essai Frappe (pitch 1) :
    // 2 ressources sans compter Thunder Quake, heave impossible.
    attaquer(&mut t, P0, "Essai Assaut", R);
    passer_jusqua_que(&mut t, |t| t.g.turn == 2);
    assert_eq!(jetons(&t, P0, "Seismic Surge"), 0);
}

#[test]
fn thunk_gagner_un_clash_en_le_revelant_cree_un_might() {
    let mut t = jeu(
        [HEROS, HEROS],
        &[RAMPART, RAMPART, RAMPART, RAMPART],
        &[("Thunk", B)],
        RIEN,
        RIEN,
        &["Miller's Grindstone"],
        &[],
    );
    assert_eq!(t.g.def(t.g.top_of_deck(P0).unwrap()).name, "Thunk");
    attaquer_arme(&mut t, P0, "Miller's Grindstone");
    fin_du_combat(&mut t);
    assert_eq!(jetons(&t, P0, "Might"), 1);
    assert_eq!(jetons(&t, P1, "Might"), 0);
}

#[test]
fn thunk_6_de_puissance_cout_5() {
    let mut t = duel(&[("Thunk", B), RAMPART, RAMPART, RAMPART], RIEN, &[], &[]);
    let a = attaquer(&mut t, P0, "Thunk", B);
    assert_eq!(t.g.power(a), 6);
    assert_eq!(t.g.player(P0).resource_points, 1, "6 pitchées - 5");
}

// ---------------------------------------------------------------------------
// Cartes de clash en défense
// ---------------------------------------------------------------------------

/// P0 attaque avec `attaque` ; P1 défend avec `defense` (de sa main).
fn clash_en_defense(
    attaque: (&str, Option<Color>),
    defense: (&str, Option<Color>),
    suite0: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
) -> Partie {
    let mut t = jeu(
        [HEROS, HEROS],
        &[attaque, RAMPART, RAMPART, RAMPART],
        suite0,
        &[defense, RAMPART, RAMPART, RAMPART],
        suite1,
        &[],
        &[],
    );
    attaquer(&mut t, P0, attaque.0, attaque.1);
    let d = t.en_main(P1, defense.0, defense.1);
    defendre_avec(&mut t, P1, &[d]);
    resoudre(&mut t);
    t
}

#[test]
fn clash_of_might_en_defendant_clash_le_gagnant_cree_un_might() {
    // Le défenseur révèle Essai Frappe (4), l'attaquant un Rempart.
    let t = clash_en_defense(
        ("Essai Frappe", R),
        ("Clash of Might", B),
        RIEN,
        &[("Essai Frappe", R)],
    );
    assert!(clash_revele(&t));
    assert_eq!(jetons(&t, P1, "Might"), 1);
    assert_eq!(jetons(&t, P0, "Might"), 0);
}

#[test]
fn clash_of_might_l_attaquant_peut_gagner_le_token() {
    let t = clash_en_defense(
        ("Essai Frappe", R),
        ("Clash of Might", B),
        &[("Boulder Drop", R)],
        RIEN,
    );
    assert_eq!(jetons(&t, P0, "Might"), 1);
    assert_eq!(jetons(&t, P1, "Might"), 0);
}

#[test]
fn clash_of_mountains_contre_une_attaque_guardian_cree_un_seismic_surge() {
    let t = clash_en_defense(
        ("Boulder Drop", R),
        ("Clash of Mountains", B),
        RIEN,
        &[("Essai Frappe", R)],
    );
    assert!(clash_revele(&t));
    assert_eq!(jetons(&t, P1, "Seismic Surge"), 1);
}

#[test]
fn clash_of_mountains_pas_de_clash_contre_une_attaque_non_guardian() {
    let t = clash_en_defense(
        ("Essai Frappe", R),
        ("Clash of Mountains", B),
        RIEN,
        &[("Essai Frappe", R)],
    );
    assert!(!clash_revele(&t));
    assert_eq!(jetons(&t, P1, "Seismic Surge"), 0);
}

#[test]
fn clash_of_vigor_le_gagnant_cree_un_vigor() {
    let t = clash_en_defense(
        ("Essai Frappe", R),
        ("Clash of Vigor", B),
        RIEN,
        &[("Essai Frappe", R)],
    );
    assert_eq!(jetons(&t, P1, "Vigor"), 1);
}

#[test]
fn test_of_vigor_le_gagnant_cree_un_vigor() {
    let t = clash_en_defense(
        ("Essai Frappe", R),
        ("Test of Vigor", R),
        &[("Boulder Drop", R)],
        RIEN,
    );
    assert_eq!(jetons(&t, P0, "Vigor"), 1, "l'attaquant gagne");
    assert_eq!(jetons(&t, P1, "Vigor"), 0);
}

#[test]
fn crash_and_bash_reveler_une_carte_avec_crush_cree_un_seismic_surge() {
    let mut t = jeu(
        [HEROS, HEROS],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Crash and Bash", R), ("Boulder Drop", R), RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    attaquer(&mut t, P0, "Essai Frappe", R);
    let cb = t.en_main(P1, "Crash and Bash", R);
    let bd = t.en_main(P1, "Boulder Drop", R);
    defendre_avec(&mut t, P1, &[cb]);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P1);
    assert!(t.peut(Choice::Pass), "« you may »");
    t.choisir(Choice::Object(bd));
    assert_eq!(jetons(&t, P1, "Seismic Surge"), 1);
    assert_eq!(t.g.obj(bd).zone, Zone::Hand, "révélée, pas jouée");
    assert_eq!(t.g.defense(cb), 4);
}

#[test]
fn crash_and_bash_sans_carte_avec_crush_rien_ne_se_passe() {
    let t = clash_en_defense(("Essai Frappe", R), ("Crash and Bash", R), RIEN, RIEN);
    assert_eq!(jetons(&t, P1, "Seismic Surge"), 0);
}

#[test]
fn sit_plus_3_defense_contre_une_attaque_brute() {
    // Clash of Might est une attaque Brute / Guardian.
    let t = clash_en_defense(("Clash of Might", B), ("Sit!", R), RIEN, RIEN);
    assert_eq!(t.vie(P1), 20, "4 contre 2 + 3");
}

#[test]
fn sit_pas_de_bonus_contre_une_attaque_non_brute() {
    let t = clash_en_defense(("Essai Frappe", R), ("Sit!", R), RIEN, RIEN);
    assert_eq!(t.vie(P1), 18, "4 contre 2");
}

// ---------------------------------------------------------------------------
// Staunch Response, The Suspense is Killing Me, Pummel
// ---------------------------------------------------------------------------

fn staunch(payer_en_plus: bool) -> Partie {
    let mut t = duel(
        &[("Chokeslam", R), RAMPART, RAMPART, RAMPART],
        &[("Staunch Response", R), RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    attaquer(&mut t, P0, "Chokeslam", R);
    let s = t.en_main(P1, "Staunch Response", R);
    passer_jusqua_que(&mut t, |t| t.joueur() == P1 && t.peut(Choice::Play(s)));
    t.choisir(Choice::Play(s));
    assert_eq!(t.kind(), DecisionKind::OptionalCost);
    t.choisir(if payer_en_plus {
        Choice::Yes
    } else {
        Choice::No
    });
    payer(&mut t);
    fin_du_combat(&mut t);
    t
}

#[test]
fn staunch_response_7_de_defense_sans_le_cout_additionnel() {
    let t = staunch(false);
    assert_eq!(t.vie(P1), 19, "8 contre 7");
    assert_eq!(t.g.player(P1).pitch.len(), 1, "2 payés avec un Rempart");
}

#[test]
fn staunch_response_plus_3_defense_si_le_cout_additionnel_est_paye() {
    let t = staunch(true);
    assert_eq!(t.vie(P1), 20, "8 contre 10");
    assert_eq!(t.g.player(P1).pitch.len(), 2, "6 payés avec deux Rempart");
}

#[test]
fn suspense_entre_avec_2_compteurs_et_la_premiere_attaque_du_tour_gagne_1() {
    let mut t = duel(
        &[
            ("The Suspense is Killing Me", B),
            ("Essai Assaut", R),
            ("Essai Frappe", R),
            RAMPART,
        ],
        RIEN,
        &[],
        &[],
    );
    let s = jouer(&mut t, P0, "The Suspense is Killing Me", B);
    resoudre(&mut t);
    assert_eq!(t.g.obj(s).zone, Zone::Permanent);
    assert_eq!(t.g.obj(s).counters, 2);
    let a = attaquer(&mut t, P0, "Essai Assaut", R);
    assert_eq!(t.g.power(a), 4, "première attaque : 3 + 1");
    fin_du_combat(&mut t);
    let b = attaquer(&mut t, P0, "Essai Frappe", R);
    assert_eq!(t.g.power(b), 4, "deuxième attaque : pas de bonus");
}

#[test]
fn suspense_un_compteur_retire_a_chaque_debut_de_tour_puis_detruit() {
    let mut t = duel(
        &[("The Suspense is Killing Me", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let s = jouer(&mut t, P0, "The Suspense is Killing Me", B);
    resoudre(&mut t);
    debut_du_tour(&mut t, 2);
    assert_eq!(t.g.obj(s).counters, 2, "pas au tour de l'adversaire");
    debut_du_tour(&mut t, 3);
    assert_eq!(t.g.obj(s).counters, 1);
    assert_eq!(t.g.obj(s).zone, Zone::Permanent);
    debut_du_tour(&mut t, 5);
    assert_eq!(
        t.g.obj(s).zone,
        Zone::Graveyard,
        "plus de compteur : détruite"
    );
}

#[test]
fn suspense_peut_defendre_depuis_la_main_sans_effet() {
    let t = clash_en_defense(
        ("Essai Frappe", R),
        ("The Suspense is Killing Me", B),
        RIEN,
        RIEN,
    );
    assert_eq!(t.vie(P1), 18, "4 contre 2");
    let s = t.g.player(P1).graveyard[0];
    assert_eq!(
        t.g.def(s).name,
        "The Suspense is Killing Me",
        "pas un permanent"
    );
}

#[test]
fn pummel_attaque_de_marteau_plus_4() {
    let mut t = duel(
        &[("Pummel", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Miller's Grindstone"],
        &[],
    );
    let proxy = attaquer_arme(&mut t, P0, "Miller's Grindstone");
    let p = t.en_main(P0, "Pummel", R);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(p)));
    t.choisir(Choice::Play(p));
    assert_eq!(
        t.options(),
        vec![Choice::Mode(0)],
        "seul mode avec une cible"
    );
    t.choisir(Choice::Mode(0));
    payer(&mut t);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 12, "4 + 4");
    let _ = proxy;
}

#[test]
fn pummel_carte_de_cout_2_ou_plus_plus_4_et_defausse_si_elle_touche() {
    let mut t = duel(
        &[("Boulder Drop", R), ("Pummel", R), RAMPART, RAMPART],
        &[("Essai Frappe", R), RAMPART, RAMPART, RAMPART],
        &[],
        &[],
    );
    let a = attaquer(&mut t, P0, "Boulder Drop", R);
    let p = t.en_main(P0, "Pummel", R);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(p)));
    t.choisir(Choice::Play(p));
    t.choisir(Choice::Mode(1));
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty());
    assert_eq!(t.g.power(a), 11);
    // Touche : crush de Boulder Drop puis défausse (ordre choisi par P0).
    let fr = t.en_main(P1, "Essai Frappe", R);
    for _ in 0..20 {
        if !t.g.combat.open {
            break;
        }
        match t.kind() {
            DecisionKind::TriggerOrder => {
                let o = t.options()[0];
                t.choisir(o)
            }
            DecisionKind::Effect => {
                if t.g.obj(fr).zone == Zone::Hand {
                    t.choisir(Choice::Object(fr));
                } else {
                    let o = t.options()[0];
                    t.choisir(o);
                }
            }
            _ => t.choisir(Choice::Pass),
        }
    }
    assert_eq!(t.vie(P1), 9);
    assert!(t.g.player(P1).graveyard.len() == 1, "une carte défaussée");
}

#[test]
fn pummel_injouable_sur_une_carte_de_cout_1() {
    let mut t = duel(
        &[("Essai Frappe", R), ("Pummel", R), RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    attaquer(&mut t, P0, "Essai Frappe", R);
    let p = t.en_main(P0, "Pummel", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction) && t.joueur() == P0
    });
    assert!(!t.peut(Choice::Play(p)));
}
