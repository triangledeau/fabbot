//! Tests des vraies cartes du deck Florian (Runeblade, Earth, Elemental et
//! génériques) : un test par phrase du texte de chaque carte, plus des cas
//! limites.

mod commun;
use commun::*;
use fab_engine::card::{Color, Supertypes};
use fab_engine::{Choice, DecisionKind, Keywords, ObjectId, PlayerId, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const FLORIAN: &str = "Florian";
const HEROS: &str = "Essai Héros";
const RUNECHANT: &str = "Runechant";
const SEEDS: &str = "Arcane Seeds // Life";
/// Instant toujours jouable (avec un Rempart à pitcher) : le joueur garde
/// une option, le moteur ne passe pas tout seul.
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);

/// Nom de la dernière carte jouée, telle qu'elle a été jouée (côté d'une
/// carte double, CR 9.2.3).
fn nom_joue(t: &Partie) -> String {
    t.g.turn_events()
        .iter()
        .rev()
        .find_map(|e| match *e {
            fab_engine::Event::Played { def, .. } => Some(t.g.db.get(def).name.clone()),
            _ => None,
        })
        .expect("une carte jouée")
}

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

/// Joue Arcane Seeds // Life avec le côté `side` (0 : Arcane Seeds, 1 :
/// Life, 2 : les deux, meld).
fn jouer_cote(t: &mut Partie, p: PlayerId, side: u8) -> ObjectId {
    let c = t.en_main(p, SEEDS, R);
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    if t.kind() == DecisionKind::Mode {
        t.choisir(Choice::Mode(side));
    }
    payer(t);
    c
}

fn activer(t: &mut Partie, p: PlayerId, src: ObjectId) {
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Activate(src, 0)));
    t.choisir(Choice::Activate(src, 0));
    payer(t);
}

fn jusqua_active(t: &mut Partie, a: ObjectId) {
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(a));
}

/// Le joueur attaque avec son arme ; renvoie l'attaque-proxy (active).
fn attaquer_arme(t: &mut Partie, p: PlayerId, arme: &str) -> ObjectId {
    let w = t.equipement(p, arme);
    activer(t, p, w);
    let i =
        t.g.objects
            .iter()
            .rposition(|o| o.proxy_of == Some(w))
            .expect("attaque-proxy");
    let proxy = ObjectId(i as u32);
    jusqua_active(t, proxy);
    proxy
}

/// Les deux joueurs passent jusqu'à ce que la pile soit vide.
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

/// Nombre de permanents de ce nom contrôlés par le joueur.
fn jetons(t: &Partie, p: PlayerId, nom: &str) -> usize {
    t.g.permanents
        .iter()
        .filter(|&&o| t.g.obj(o).controller == p && t.g.def(o).name == nom)
        .count()
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

/// Bannit (face visible) une carte du deck (préparation d'un test).
fn bannir(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = du_deck(t, p, nom, couleur);
    t.g.banish(p, c, false);
    t.g.refresh_priority();
    c
}

/// 4 Autumn's Touch dans le deck de Florian, à bannir pour sa capacité.
const QUATRE_EARTH: &[(&str, Option<Color>)] = &[
    ("Autumn's Touch", R),
    ("Autumn's Touch", R),
    ("Autumn's Touch", Y),
    ("Autumn's Touch", B),
];

fn bannir_quatre_earth(t: &mut Partie) {
    bannir(t, P0, "Autumn's Touch", R);
    bannir(t, P0, "Autumn's Touch", R);
    bannir(t, P0, "Autumn's Touch", Y);
    bannir(t, P0, "Autumn's Touch", B);
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

/// Florian (P0) contre Essai Héros (P1).
fn florian(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    arena0: &[&str],
) -> Partie {
    partie_vraie([FLORIAN, HEROS], main0, suite0, RIEN, RIEN, arena0, &[])
}

/// Florian (P0) contre Essai Héros (P1), qui a ces cartes en main.
fn florian_contre(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    arena0: &[&str],
) -> Partie {
    partie_vraie([FLORIAN, HEROS], main0, suite0, main1, RIEN, arena0, &[])
}

/// P1 attaque Florian avec Essai Frappe (rouge, 4{p}) au tour 2.
fn p1_attaque(t: &mut Partie) -> ObjectId {
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P1);
    let a = jouer(t, P1, "Essai Frappe", R);
    jusqua_active(t, a);
    a
}

// ---------------------------------------------------------------------------
// Florian
// ---------------------------------------------------------------------------

#[test]
fn florian_sans_4_cartes_earth_bannies_cree_le_nombre_normal_de_tokens() {
    let mut t = florian(
        &[("Read the Runes", R), RAMPART, RAMPART, RAMPART],
        QUATRE_EARTH,
        &[],
    );
    bannir(&mut t, P0, "Autumn's Touch", R);
    bannir(&mut t, P0, "Autumn's Touch", R);
    bannir(&mut t, P0, "Autumn's Touch", Y);
    jouer(&mut t, P0, "Read the Runes", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 3);
}

#[test]
fn florian_avec_4_cartes_earth_bannies_cree_un_token_aura_de_plus() {
    let mut t = florian(
        &[("Read the Runes", R), RAMPART, RAMPART, RAMPART],
        QUATRE_EARTH,
        &[],
    );
    bannir_quatre_earth(&mut t);
    jouer(&mut t, P0, "Read the Runes", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 4, "3 + 1");
}

#[test]
fn florian_chaque_creation_de_tokens_est_remplacee_separement() {
    // Arcane Seeds : « Create a Runechant token. Create a Runechant token. »
    let mut t = florian(&[(SEEDS, R), RAMPART, RAMPART, RAMPART], QUATRE_EARTH, &[]);
    bannir_quatre_earth(&mut t);
    jouer_cote(&mut t, P0, 0);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 4, "(1 + 1) deux fois");
}

#[test]
fn florian_ne_concerne_pas_les_tokens_crees_par_l_adversaire() {
    let mut t = florian(&[RAMPART; 4], QUATRE_EARTH, &[]);
    bannir_quatre_earth(&mut t);
    let rc = t.g.db.find_name(RUNECHANT).unwrap();
    // L'adversaire crée un token aura sous le contrôle de Florian (CR 8.5.2c).
    t.g.create_tokens_by(P1, P0, rc, 1);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
    t.g.create_tokens(P0, rc, 1);
    assert_eq!(jetons(&t, P0, RUNECHANT), 3, "1 + (1 + 1)");
}

#[test]
fn florian_ne_concerne_pas_les_tokens_qui_ne_sont_pas_des_auras() {
    let mut t = florian(&[RAMPART; 4], QUATRE_EARTH, &[]);
    bannir_quatre_earth(&mut t);
    creer(&mut t, P0, "Essai Cendre", 1);
    assert_eq!(jetons(&t, P0, "Essai Cendre"), 1);
}

// ---------------------------------------------------------------------------
// Runechant
// ---------------------------------------------------------------------------

#[test]
fn runechant_une_carte_d_attaque_jouee_le_detruit_et_inflige_1_degat_arcanique() {
    let mut t = florian(
        &[("Autumn's Touch", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    creer(&mut t, P0, RUNECHANT, 2);
    let a = jouer(&mut t, P0, "Autumn's Touch", R);
    jusqua_active(&mut t, a);
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
    assert_eq!(t.vie(P1), 18, "2 dégâts arcaniques avant le combat");
    assert!(t.g.turn_events().iter().any(|e| matches!(
        e,
        fab_engine::Event::Damage {
            arcane: true,
            target,
            ..
        } if *target == P1
    )));
}

#[test]
fn runechant_une_attaque_d_arme_le_declenche() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Nebula Blade"]);
    creer(&mut t, P0, RUNECHANT, 1);
    attaquer_arme(&mut t, P0, "Nebula Blade");
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn runechant_une_action_non_attaque_ne_le_declenche_pas() {
    let mut t = florian(
        &[("Oath of the Arknight", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    creer(&mut t, P0, RUNECHANT, 1);
    jouer(&mut t, P0, "Oath of the Arknight", B);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 2, "1 + celui d'Oath");
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn runechant_les_attaques_de_l_adversaire_ne_le_declenchent_pas() {
    let mut t = florian_contre(&[RAMPART; 4], RIEN, &[("Essai Frappe", R), RAMPART], &[]);
    creer(&mut t, P0, RUNECHANT, 1);
    p1_attaque(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
}

// ---------------------------------------------------------------------------
// Nebula Blade
// ---------------------------------------------------------------------------

#[test]
fn nebula_blade_attaque_pour_2_ressources_une_fois_par_tour_sans_go_again() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Nebula Blade"]);
    let w = t.equipement(P0, "Nebula Blade");
    let proxy = attaquer_arme(&mut t, P0, "Nebula Blade");
    assert_eq!(t.g.power(proxy), 1);
    assert_eq!(t.g.player(P0).resource_points, 1, "3 - 2");
    assert!(!t.g.keywords(proxy).has(Keywords::GO_AGAIN));
    assert!(!t.peut(Choice::Activate(w, 0)));
}

#[test]
fn nebula_blade_touche_cree_un_runechant() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Nebula Blade"]);
    attaquer_arme(&mut t, P0, "Nebula Blade");
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
}

#[test]
fn nebula_blade_plus_3_si_une_action_non_attaque_a_ete_jouee_ce_tour() {
    let mut t = florian(
        &[("Harvest Season", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Nebula Blade"],
    );
    let w = t.equipement(P0, "Nebula Blade");
    assert_eq!(t.g.power(w), 1);
    jouer(&mut t, P0, "Harvest Season", R);
    resoudre(&mut t);
    assert_eq!(t.g.power(w), 4);
    let proxy = attaquer_arme(&mut t, P0, "Nebula Blade");
    assert_eq!(t.g.power(proxy), 4);
}

#[test]
fn nebula_blade_life_seul_n_est_pas_une_action_mais_arcane_seeds_oui() {
    // CR 9.2.3 : jouée côté Life, la carte n'est qu'un instant.
    let mut t = florian(
        &[(SEEDS, R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Nebula Blade"],
    );
    let w = t.equipement(P0, "Nebula Blade");
    jouer_cote(&mut t, P0, 1);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 21);
    assert_eq!(t.g.power(w), 1);

    let mut t = florian(
        &[(SEEDS, R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Nebula Blade"],
    );
    let w = t.equipement(P0, "Nebula Blade");
    jouer_cote(&mut t, P0, 0);
    resoudre(&mut t);
    assert_eq!(t.g.power(w), 4);
}

// ---------------------------------------------------------------------------
// Scepter of Pain
// ---------------------------------------------------------------------------

#[test]
fn scepter_of_pain_1_degat_arcanique_au_heros_adverse_et_un_runechant_par_degat() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Scepter of Pain"]);
    let s = t.equipement(P0, "Scepter of Pain");
    activer(&mut t, P0, s);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
    assert_eq!(t.g.player(P0).action_points, 0, "action sans go again");
    assert!(!t.peut(Choice::Activate(s, 0)), "une fois par tour");
}

#[test]
fn scepter_of_pain_degat_prevenu_pas_de_runechant() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Scepter of Pain"]);
    t.g.prevent_next(P1, 1, false, None);
    let s = t.equipement(P0, "Scepter of Pain");
    activer(&mut t, P0, s);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20);
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
}

#[test]
fn scepter_of_pain_ne_vise_que_l_adversaire() {
    let mut t = florian(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Scepter of Pain"],
    );
    let s = t.equipement(P0, "Scepter of Pain");
    activer(&mut t, P0, s);
    // Une seule cible légale : déclarée d'office.
    let layer = t.g.stack.last().expect("capacité sur la pile");
    assert_eq!(layer.ctx.target(0), Some(Target::Player(P1)));
}

// ---------------------------------------------------------------------------
// Annals of Sutcliffe
// ---------------------------------------------------------------------------

#[test]
fn annals_of_sutcliffe_pioche_une_carte_pour_3_ressources() {
    let mut t = florian(
        &[RAMPART; 4],
        &[("Autumn's Touch", R)],
        &["Annals of Sutcliffe"],
    );
    let a = t.equipement(P0, "Annals of Sutcliffe");
    activer(&mut t, P0, a);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).hand.len(), 4, "4 - 1 pitchée + 1 piochée");
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
    assert!(!t.peut(Choice::Activate(a, 0)), "une fois par tour");
}

#[test]
fn annals_of_sutcliffe_attaque_et_non_attaque_pitchees_un_runechant() {
    let mut t = florian(
        &[
            ("Read the Runes", R),
            ("Autumn's Touch", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &["Annals of Sutcliffe"],
    );
    let a = t.equipement(P0, "Annals of Sutcliffe");
    let rr = t.en_main(P0, "Read the Runes", R);
    let at = t.en_main(P0, "Autumn's Touch", R);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Activate(a, 0)));
    t.choisir(Choice::Activate(a, 0));
    t.choisir(Choice::Pitch(rr));
    t.choisir(Choice::Pitch(at));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
}

#[test]
fn annals_of_sutcliffe_seulement_une_attaque_pitchee_pas_de_runechant() {
    let mut t = florian(
        &[("Autumn's Touch", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Annals of Sutcliffe"],
    );
    let a = t.equipement(P0, "Annals of Sutcliffe");
    let at = t.en_main(P0, "Autumn's Touch", B);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Activate(a, 0)));
    t.choisir(Choice::Activate(a, 0));
    t.choisir(Choice::Pitch(at));
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
}

// ---------------------------------------------------------------------------
// Équipements
// ---------------------------------------------------------------------------

#[test]
fn arcane_lantern_arcane_barrier_1_sans_defense() {
    let t = florian(&[RAMPART; 4], RIEN, &["Arcane Lantern"]);
    let l = t.equipement(P0, "Arcane Lantern");
    assert_eq!(t.g.prevention_values(l), (1, 0, 0));
    assert_eq!(t.g.def(l).defense, None);
}

#[test]
fn bloodied_oval_defense_egale_au_nombre_d_adversaires_avec_plus_de_vie() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Bloodied Oval"]);
    let o = t.equipement(P0, "Bloodied Oval");
    assert_eq!(t.g.defense(o), 0, "même vie");
    t.g.player_mut(P1).life = 25;
    assert_eq!(t.g.defense(o), 1);
    t.g.player_mut(P0).life = 30;
    assert_eq!(t.g.defense(o), 0, "l'adversaire a moins de vie");
}

/// Blade break : l'équipement défend, puis il est détruit à la fermeture
/// de la chaîne de combat.
fn verifier_blade_break(nom: &str) {
    let mut t = florian_contre(&[RAMPART; 4], RIEN, &[("Essai Frappe", R), RAMPART], &[nom]);
    let e = t.equipement(P0, nom);
    assert!(t.g.keywords(e).has(Keywords::BLADE_BREAK));
    p1_attaque(&mut t);
    defendre_avec(&mut t, P0, &[e]);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(e).zone, Zone::Graveyard, "{nom} détruit");
}

#[test]
fn bloodied_oval_blade_break() {
    verifier_blade_break("Bloodied Oval");
}

#[test]
fn crown_of_dichotomy_met_deux_cartes_runeblade_du_cimetiere_sur_le_deck_dans_l_ordre_choisi() {
    let mut t = florian(
        &[RAMPART; 4],
        &[("Amplify the Arknight", R), ("Read the Runes", R)],
        &["Crown of Dichotomy"],
    );
    let amp = au_cimetiere(&mut t, P0, "Amplify the Arknight", R);
    let rr = au_cimetiere(&mut t, P0, "Read the Runes", R);
    let c = t.equipement(P0, "Crown of Dichotomy");
    activer(&mut t, P0, c);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Object(rr));
    resoudre(&mut t);
    let deck = &t.g.player(P0).deck;
    assert_eq!(deck[deck.len() - 1], rr, "Read the Runes sur le dessus");
    assert_eq!(deck[deck.len() - 2], amp);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard, "Crown détruite");
    assert_eq!(t.g.player(P0).action_points, 0, "action sans go again");
}

#[test]
fn crown_of_dichotomy_il_faut_les_deux_cibles() {
    let mut t = florian(
        &[("Essai Étincelle", R), RAMPART, RAMPART, RAMPART],
        &[("Amplify the Arknight", R), ("Autumn's Touch", R)],
        &["Crown of Dichotomy"],
    );
    au_cimetiere(&mut t, P0, "Amplify the Arknight", R);
    au_cimetiere(&mut t, P0, "Autumn's Touch", R);
    let c = t.equipement(P0, "Crown of Dichotomy");
    assert!(
        !t.peut(Choice::Activate(c, 0)),
        "pas de non-attaque Runeblade"
    );
}

#[test]
fn crown_of_dichotomy_arcane_barrier_1() {
    let t = florian(&[RAMPART; 4], RIEN, &["Crown of Dichotomy"]);
    let c = t.equipement(P0, "Crown of Dichotomy");
    assert_eq!(t.g.prevention_values(c), (1, 0, 0));
}

#[test]
fn helm_of_lignum_vitae_plus_1_defense_avec_4_cartes_earth_bannies() {
    let mut t = florian(&[RAMPART; 4], QUATRE_EARTH, &["Helm of Lignum Vitae"]);
    let h = t.equipement(P0, "Helm of Lignum Vitae");
    assert_eq!(t.g.defense(h), 1);
    bannir(&mut t, P0, "Autumn's Touch", R);
    bannir(&mut t, P0, "Autumn's Touch", R);
    bannir(&mut t, P0, "Autumn's Touch", Y);
    assert_eq!(t.g.defense(h), 1, "3 cartes Earth");
    bannir(&mut t, P0, "Autumn's Touch", B);
    assert_eq!(t.g.defense(h), 2);
}

#[test]
fn helm_of_lignum_vitae_blade_break() {
    verifier_blade_break("Helm of Lignum Vitae");
}

#[test]
fn plume_of_evergrowth_renvoie_une_carte_earth_du_cimetiere_en_main() {
    let mut t = florian(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Autumn's Touch", R)],
        &["Plume of Evergrowth"],
    );
    let at = au_cimetiere(&mut t, P0, "Autumn's Touch", R);
    let p = t.equipement(P0, "Plume of Evergrowth");
    activer(&mut t, P0, p);
    resoudre(&mut t);
    assert_eq!(t.g.obj(at).zone, Zone::Hand);
    assert_eq!(t.g.obj(p).zone, Zone::Graveyard, "Plume détruite");
    assert_eq!(t.g.player(P0).action_points, 1, "instant");
}

#[test]
fn plume_of_evergrowth_ne_vise_pas_une_carte_non_earth() {
    let mut t = florian(
        &[("Essai Étincelle", R), RAMPART, RAMPART, RAMPART],
        &[("Amplify the Arknight", R)],
        &["Plume of Evergrowth"],
    );
    au_cimetiere(&mut t, P0, "Amplify the Arknight", R);
    let p = t.equipement(P0, "Plume of Evergrowth");
    assert!(!t.peut(Choice::Activate(p, 0)));
}

#[test]
fn blossom_of_spring_detruire_gagne_une_ressource_avec_go_again() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Blossom of Spring"]);
    let b = t.equipement(P0, "Blossom of Spring");
    activer(&mut t, P0, b);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).resource_points, 1);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    assert_eq!(t.g.obj(b).zone, Zone::Graveyard);
}

#[test]
fn unyielding_grip_plus_3_defense_sans_carte_en_main() {
    let mut t = florian(&[RAMPART; 4], RIEN, &["Unyielding Grip"]);
    let u = t.equipement(P0, "Unyielding Grip");
    assert_eq!(t.g.defense(u), 0);
    for c in t.g.player(P0).hand.clone() {
        t.g.put_in_graveyard(c);
    }
    assert_eq!(t.g.defense(u), 3);
}

#[test]
fn unyielding_grip_blade_break() {
    verifier_blade_break("Unyielding Grip");
}

#[test]
fn well_grounded_seulement_avec_4_cartes_earth_bannies() {
    let mut t = florian(
        &[("Essai Étincelle", R), RAMPART, RAMPART, RAMPART],
        QUATRE_EARTH,
        &["Well Grounded"],
    );
    let w = t.equipement(P0, "Well Grounded");
    assert!(!t.peut(Choice::Activate(w, 0)));
    bannir_quatre_earth(&mut t);
    assert!(t.peut(Choice::Activate(w, 0)));
}

#[test]
fn well_grounded_previent_les_2_prochains_degats_ce_tour() {
    let mut t = florian(&[RAMPART; 4], QUATRE_EARTH, &["Well Grounded"]);
    bannir_quatre_earth(&mut t);
    let w = t.equipement(P0, "Well Grounded");
    activer(&mut t, P0, w);
    resoudre(&mut t);
    assert_eq!(t.g.obj(w).zone, Zone::Graveyard);
    t.g.deal_damage(None, P1, P0, 3, false);
    assert_eq!(t.vie(P0), 19, "3 - 2 prévenus");
}

// ---------------------------------------------------------------------------
// Cartes du deck
// ---------------------------------------------------------------------------

#[test]
fn amplify_the_arknight_coute_1_de_moins_par_runechant() {
    let mut t = florian(
        &[("Amplify the Arknight", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let a = t.en_main(P0, "Amplify the Arknight", R);
    assert_eq!(t.g.play_cost(P0, a), 3);
    creer(&mut t, P0, RUNECHANT, 2);
    assert_eq!(t.g.play_cost(P0, a), 1);
    creer(&mut t, P0, RUNECHANT, 2);
    assert_eq!(t.g.play_cost(P0, a), 0, "jamais négatif");
}

#[test]
fn autumns_touch_attaque_earth_sans_texte() {
    let mut t = florian(
        &[("Autumn's Touch", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let a = jouer(&mut t, P0, "Autumn's Touch", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7);
    assert!(t.g.supertypes(a).has(Supertypes::EARTH));
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 13);
}

#[test]
fn chorus_of_rotwood_cree_3_runechants_sans_decompose_possible() {
    let mut t = florian(
        &[("Chorus of Rotwood", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Chorus of Rotwood", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 3);
    assert_eq!(jetons(&t, P0, "Embodiment of Earth"), 0);
    assert_eq!(t.g.player(P0).action_points, 0, "pas de go again");
}

/// Un cimetière où decompose est possible : 2 cartes Earth et une action.
const CIMETIERE: &[(&str, Option<Color>)] = &[
    ("Autumn's Touch", R),
    ("Autumn's Touch", Y),
    ("Read the Runes", B),
];

fn remplir_cimetiere(t: &mut Partie) -> Vec<ObjectId> {
    vec![
        au_cimetiere(t, P0, "Autumn's Touch", R),
        au_cimetiere(t, P0, "Autumn's Touch", Y),
        au_cimetiere(t, P0, "Read the Runes", B),
    ]
}

/// Répond oui à decompose et choisit ces 3 cartes.
fn decomposer(t: &mut Partie, cartes: &[ObjectId]) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Yes);
    for &c in cartes {
        t.choisir(Choice::Object(c));
    }
}

#[test]
fn chorus_of_rotwood_decompose_bannit_3_cartes_et_cree_embodiment_of_earth() {
    let mut t = florian(
        &[("Chorus of Rotwood", R), RAMPART, RAMPART, RAMPART],
        CIMETIERE,
        &[],
    );
    let cartes = remplir_cimetiere(&mut t);
    jouer(&mut t, P0, "Chorus of Rotwood", R);
    decomposer(&mut t, &cartes);
    resoudre(&mut t);
    for c in cartes {
        assert_eq!(t.g.obj(c).zone, Zone::Banished);
    }
    assert_eq!(jetons(&t, P0, "Embodiment of Earth"), 1);
    assert_eq!(jetons(&t, P0, RUNECHANT), 3);
}

#[test]
fn chorus_of_rotwood_decompose_refuse_rien_n_est_banni() {
    let mut t = florian(
        &[("Chorus of Rotwood", R), RAMPART, RAMPART, RAMPART],
        CIMETIERE,
        &[],
    );
    remplir_cimetiere(&mut t);
    jouer(&mut t, P0, "Chorus of Rotwood", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::No);
    resoudre(&mut t);
    assert!(t.g.player(P0).banished.is_empty());
    assert_eq!(jetons(&t, P0, "Embodiment of Earth"), 0);
    assert_eq!(jetons(&t, P0, RUNECHANT), 3);
}

#[test]
fn decompose_ne_propose_que_des_cartes_qui_permettent_de_finir() {
    // Cimetière : 2 Earth d'attaque, une action Runeblade, un bloc.
    let mut t = florian(
        &[("Chorus of Rotwood", R), RAMPART, RAMPART, RAMPART],
        &[
            ("Autumn's Touch", R),
            ("Autumn's Touch", Y),
            ("Read the Runes", B),
            RAMPART,
        ],
        &[],
    );
    let cartes = remplir_cimetiere(&mut t);
    let bloc = au_cimetiere(&mut t, P0, "Essai Rempart", B);
    jouer(&mut t, P0, "Chorus of Rotwood", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Yes);
    assert!(!t.peut(Choice::Object(bloc)), "ni Earth ni action");
    // Read the Runes (non-Earth) choisie en premier : elle est l'action,
    // les deux cartes Earth restent proposées.
    t.choisir(Choice::Object(cartes[2]));
    assert!(t.peut(Choice::Object(cartes[0])) && t.peut(Choice::Object(cartes[1])));
}

#[test]
fn decompose_impossible_sans_carte_d_action_en_plus_des_2_earth() {
    // 2 cartes Earth seulement : il faut 3 cartes distinctes.
    let mut t = florian(
        &[("Chorus of Rotwood", R), RAMPART, RAMPART, RAMPART],
        &[("Autumn's Touch", R), ("Autumn's Touch", Y), RAMPART],
        &[],
    );
    au_cimetiere(&mut t, P0, "Autumn's Touch", R);
    au_cimetiere(&mut t, P0, "Autumn's Touch", Y);
    au_cimetiere(&mut t, P0, "Essai Rempart", B);
    jouer(&mut t, P0, "Chorus of Rotwood", R);
    resoudre(&mut t);
    assert!(t.g.player(P0).banished.is_empty());
    assert_eq!(jetons(&t, P0, RUNECHANT), 3);
}

#[test]
fn colors_of_aria_face_visible_est_earth_ice_et_lightning() {
    let mut t = florian(&[RAMPART; 4], &[("Colors of Aria", R)], &[]);
    let c = du_deck(&t, P0, "Colors of Aria", R);
    assert!(
        !t.g.supertypes(c).has(Supertypes::EARTH),
        "face cachée dans le deck"
    );
    t.g.put_in_graveyard(c);
    let s = t.g.supertypes(c);
    assert!(s.has(Supertypes::EARTH | Supertypes::ICE | Supertypes::LIGHTNING));
    assert!(s.has(Supertypes::ELEMENTAL));
}

#[test]
fn colors_of_aria_bannie_compte_comme_carte_earth_pour_florian() {
    let mut t = florian(
        &[("Read the Runes", B), RAMPART, RAMPART, RAMPART],
        &[
            ("Autumn's Touch", R),
            ("Autumn's Touch", R),
            ("Autumn's Touch", Y),
            ("Colors of Aria", R),
        ],
        &[],
    );
    bannir(&mut t, P0, "Autumn's Touch", R);
    bannir(&mut t, P0, "Autumn's Touch", R);
    bannir(&mut t, P0, "Autumn's Touch", Y);
    bannir(&mut t, P0, "Colors of Aria", R);
    jouer(&mut t, P0, "Read the Runes", B);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 2, "1 + 1");
}

#[test]
fn fruits_of_the_forest_defausser_gagne_2_pv() {
    let mut t = florian(
        &[("Fruits of the Forest", Y), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let f = t.en_main(P0, "Fruits of the Forest", Y);
    activer(&mut t, P0, f);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 22);
    assert_eq!(t.g.obj(f).zone, Zone::Graveyard);
    assert_eq!(t.g.player(P0).action_points, 1, "instant");
}

#[test]
fn fruits_of_the_forest_pendant_le_tour_adverse() {
    let mut t = florian(
        &[("Fruits of the Forest", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let f = t.en_main(P0, "Fruits of the Forest", B);
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P1);
    activer(&mut t, P0, f);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 22);
}

#[test]
fn harvest_season_go_again_puis_detruite_en_debut_de_phase_d_action_et_gagne_des_pv() {
    for (couleur, gain) in [(R, 3), (Y, 2), (B, 1)] {
        let mut t = florian(
            &[("Harvest Season", couleur), RAMPART, RAMPART, RAMPART],
            RIEN,
            &[],
        );
        let h = jouer(&mut t, P0, "Harvest Season", couleur);
        resoudre(&mut t);
        assert_eq!(t.g.obj(h).zone, Zone::Permanent);
        assert_eq!(t.g.player(P0).action_points, 1, "go again");
        debut_du_tour(&mut t, 2);
        assert_eq!(t.g.obj(h).zone, Zone::Permanent, "pas au tour adverse");
        debut_du_tour(&mut t, 3);
        assert_eq!(t.g.obj(h).zone, Zone::Graveyard);
        assert_eq!(t.vie(P0), 20 + gain);
    }
}

#[test]
fn oath_of_the_arknight_prochaine_attaque_runeblade_plus_3_un_runechant_et_go_again() {
    let mut t = florian(
        &[
            ("Oath of the Arknight", R),
            ("Amplify the Arknight", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Oath of the Arknight", R);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    let a = jouer(&mut t, P0, "Amplify the Arknight", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 9, "6 + 3");
}

#[test]
fn oath_of_the_arknight_pas_pour_une_attaque_qui_n_est_pas_runeblade() {
    let mut t = florian(
        &[
            ("Oath of the Arknight", B),
            ("Autumn's Touch", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Oath of the Arknight", B);
    let a = jouer(&mut t, P0, "Autumn's Touch", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7);
}

#[test]
fn read_the_runes_cree_3_2_ou_1_runechants() {
    for (couleur, n) in [(R, 3), (Y, 2), (B, 1)] {
        let mut t = florian(
            &[("Read the Runes", couleur), RAMPART, RAMPART, RAMPART],
            RIEN,
            &[],
        );
        jouer(&mut t, P0, "Read the Runes", couleur);
        resoudre(&mut t);
        assert_eq!(jetons(&t, P0, RUNECHANT), n);
        assert_eq!(t.g.player(P0).action_points, 0, "pas de go again");
    }
}

#[test]
fn reduce_to_runechant_coute_moins_par_runechant_et_en_cree_un_en_defendant() {
    let mut t = florian_contre(
        &[("Reduce to Runechant", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[("Essai Frappe", R), RAMPART],
        &[],
    );
    let r = t.en_main(P0, "Reduce to Runechant", R);
    assert_eq!(t.g.play_cost(P0, r), 1);
    creer(&mut t, P0, RUNECHANT, 1);
    assert_eq!(t.g.play_cost(P0, r), 0);
    p1_attaque(&mut t);
    defendre_avec(&mut t, P0, &[]);
    jouer(&mut t, P0, "Reduce to Runechant", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.stack.is_empty() && t.g.pending_triggers.is_empty()
    });
    assert_eq!(jetons(&t, P0, RUNECHANT), 2);
    assert!(t.g.active_link().unwrap().defenders.contains(&r));
    assert_eq!(t.g.defense(r), 4);
}

#[test]
fn rootbound_carapace_decompose_plus_1_defense() {
    let mut t = florian_contre(
        &[("Rootbound Carapace", Y), ETINCELLE, RAMPART, RAMPART],
        CIMETIERE,
        &[("Essai Frappe", R), RAMPART],
        &[],
    );
    let cartes = remplir_cimetiere(&mut t);
    p1_attaque(&mut t);
    defendre_avec(&mut t, P0, &[]);
    let r = jouer(&mut t, P0, "Rootbound Carapace", Y);
    decomposer(&mut t, &cartes);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.g.tasks.is_empty());
    assert!(t.g.active_link().unwrap().defenders.contains(&r));
    assert_eq!(t.g.defense(r), 3, "2 + 1");
}

#[test]
fn rootbound_carapace_sans_decompose_defense_imprimee() {
    let mut t = florian_contre(
        &[("Rootbound Carapace", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Essai Frappe", R), RAMPART],
        &[],
    );
    p1_attaque(&mut t);
    defendre_avec(&mut t, P0, &[]);
    let r = jouer(&mut t, P0, "Rootbound Carapace", R);
    resoudre(&mut t);
    assert_eq!(t.g.defense(r), 3);
}

#[test]
fn runeblood_incantation_compteurs_verse_un_runechant_par_phase_d_action() {
    let mut t = florian(
        &[("Runeblood Incantation", Y), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let r = jouer(&mut t, P0, "Runeblood Incantation", Y);
    resoudre(&mut t);
    assert_eq!(t.g.obj(r).counters, 2, "entre avec 2 compteurs verse");
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
    debut_du_tour(&mut t, 3);
    assert_eq!(t.g.obj(r).counters, 1);
    assert_eq!(jetons(&t, P0, RUNECHANT), 1);
    debut_du_tour(&mut t, 5);
    assert_eq!(t.g.obj(r).counters, 0);
    assert_eq!(jetons(&t, P0, RUNECHANT), 2);
    debut_du_tour(&mut t, 7);
    assert_eq!(
        t.g.obj(r).zone,
        Zone::Graveyard,
        "plus de compteur : détruite"
    );
    assert_eq!(jetons(&t, P0, RUNECHANT), 2);
}

#[test]
fn runeblood_incantation_rouge_3_compteurs_bleue_1() {
    for (couleur, n) in [(R, 3), (B, 1)] {
        let mut t = florian(
            &[
                ("Runeblood Incantation", couleur),
                RAMPART,
                RAMPART,
                RAMPART,
            ],
            RIEN,
            &[],
        );
        let r = jouer(&mut t, P0, "Runeblood Incantation", couleur);
        resoudre(&mut t);
        assert_eq!(t.g.obj(r).counters, n);
    }
}

#[test]
fn sigil_of_silphidae_en_entrant_peut_bannir_une_autre_aura_pour_1_degat_arcanique() {
    let mut t = florian(
        &[("Sigil of Silphidae", B), RAMPART, RAMPART, RAMPART],
        &[("Harvest Season", R)],
        &[],
    );
    let h = au_cimetiere(&mut t, P0, "Harvest Season", R);
    jouer(&mut t, P0, "Sigil of Silphidae", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(P1)));
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Object(h));
    resoudre(&mut t);
    assert_eq!(t.g.obj(h).zone, Zone::Banished);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn sigil_of_silphidae_refuser_de_bannir_pas_de_degat() {
    let mut t = florian(
        &[("Sigil of Silphidae", B), RAMPART, RAMPART, RAMPART],
        &[("Harvest Season", R)],
        &[],
    );
    let h = au_cimetiere(&mut t, P0, "Harvest Season", R);
    jouer(&mut t, P0, "Sigil of Silphidae", B);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(P1)));
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Pass);
    resoudre(&mut t);
    assert_eq!(t.g.obj(h).zone, Zone::Graveyard);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn sigil_of_silphidae_detruit_en_phase_d_action_et_en_partant_peut_bannir_une_aura() {
    let mut t = florian(
        &[("Sigil of Silphidae", B), RAMPART, RAMPART, RAMPART],
        &[("Harvest Season", R)],
        &[],
    );
    let s = jouer(&mut t, P0, "Sigil of Silphidae", B);
    // La cible est déclarée même sans autre aura (CR 6.6.6a).
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(P1)));
    resoudre(&mut t); // cimetière sans autre aura : pas de question
    assert_eq!(t.g.obj(s).zone, Zone::Permanent);
    let h = au_cimetiere(&mut t, P0, "Harvest Season", R);
    while t.g.turn < 3 {
        t.finir_le_tour();
    }
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    assert_eq!(t.g.obj(s).zone, Zone::Graveyard, "détruite");
    t.choisir(Choice::Target(Target::Player(P1)));
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    // Elle-même n'est pas proposée (« another aura »).
    assert!(!t.peut(Choice::Object(s)));
    t.choisir(Choice::Object(h));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn embodiment_of_earth_plus_1_defense_aux_actions_non_attaque_qui_defendent() {
    let mut t = florian_contre(
        &[
            ("Read the Runes", R),
            ("Autumn's Touch", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[("Essai Frappe", R), RAMPART],
        &[],
    );
    let rr = t.en_main(P0, "Read the Runes", R);
    let at = t.en_main(P0, "Autumn's Touch", R);
    creer(&mut t, P0, "Embodiment of Earth", 1);
    assert_eq!(t.g.defense(rr), 2, "pas en train de défendre");
    p1_attaque(&mut t);
    defendre_avec(&mut t, P0, &[rr, at]);
    assert_eq!(t.g.defense(rr), 3, "2 + 1");
    assert_eq!(t.g.defense(at), 3, "carte d'attaque : pas de bonus");
}

#[test]
fn embodiment_of_earth_detruit_au_debut_de_la_phase_d_action() {
    let mut t = florian(&[RAMPART; 4], RIEN, &[]);
    creer(&mut t, P0, "Embodiment of Earth", 1);
    debut_du_tour(&mut t, 2);
    assert_eq!(
        jetons(&t, P0, "Embodiment of Earth"),
        1,
        "pas au tour adverse"
    );
    debut_du_tour(&mut t, 3);
    assert_eq!(jetons(&t, P0, "Embodiment of Earth"), 0);
}

// ---------------------------------------------------------------------------
// Arcane Seeds // Life
// ---------------------------------------------------------------------------

#[test]
fn arcane_seeds_cree_2_runechants_avec_go_again() {
    let mut t = florian(&[(SEEDS, R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    jouer_cote(&mut t, P0, 0);
    assert_eq!(nom_joue(&t), "Arcane Seeds");
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 2);
    assert_eq!(t.vie(P0), 20);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn life_gagne_1_pv_meme_pendant_le_tour_adverse() {
    let mut t = florian(&[(SEEDS, R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P1);
    // Seul le côté Life (instant) est jouable : pas de choix de côté.
    let c = t.en_main(P0, SEEDS, R);
    passer_jusqua_que(&mut t, |t| t.joueur() == P0 && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    assert_eq!(nom_joue(&t), "Life");
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 21);
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
}

#[test]
fn arcane_seeds_life_meld_life_puis_arcane_seeds_avec_go_again() {
    // Essai Étincelle (instant) : Florian garde une option, le moteur ne
    // passe pas tout seul.
    let mut t = florian(&[(SEEDS, R), ETINCELLE, RAMPART, RAMPART], RIEN, &[]);
    jouer_cote(&mut t, P0, 2);
    assert_eq!(t.g.player(P0).action_points, 0, "action : 1 point d'action");
    // Première résolution : Life seulement, puis la priorité revient
    // (l'adversaire n'a que des blocs : il passe tout seul).
    t.choisir(Choice::Pass);
    assert_eq!(t.vie(P0), 21);
    assert_eq!(jetons(&t, P0, RUNECHANT), 0);
    assert_eq!(t.g.stack.len(), 1, "la carte reste sur la pile");
    assert_eq!(t.kind(), DecisionKind::Priority);
    assert_eq!(t.joueur(), P0);
    resoudre(&mut t);
    assert_eq!(jetons(&t, P0, RUNECHANT), 2);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn arcane_seeds_life_a_les_proprietes_des_deux_cotes_hors_de_la_pile() {
    let mut t = florian(&[RAMPART; 4], &[(SEEDS, R)], &[]);
    let c = au_cimetiere(&mut t, P0, SEEDS, R);
    let s = t.g.supertypes(c);
    assert!(s.has(Supertypes::EARTH) && s.has(Supertypes::RUNEBLADE));
    assert_eq!(t.g.defense(c), 3);
    assert_eq!(t.g.def(c).pitch, Some(1));
}

#[test]
fn arcane_seeds_life_revient_a_la_carte_entiere_au_cimetiere() {
    let mut t = florian(&[(SEEDS, R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    let c = jouer_cote(&mut t, P0, 1);
    resoudre(&mut t);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    assert_eq!(t.g.def(c).name, SEEDS);
}
