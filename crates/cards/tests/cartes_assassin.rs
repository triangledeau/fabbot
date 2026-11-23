//! Tests des vraies cartes du deck Arakni (Assassin, Chaos, Agents of Chaos,
//! tokens Graphene Chelicera et Bloodrot Pox, cartes génériques du deck) :
//! un test par phrase du texte de chaque carte, plus des cas limites.

mod commun;
use commun::*;
use fab_engine::card::{Color, Traits};
use fab_engine::{Choice, DecisionKind, Event, Keywords, ObjectId, PlayerId, Target, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const ARAKNI: &str = "Arakni, Web of Deceit";
const HEROS: &str = "Essai Héros";
const HUNTSMAN: &str = "Mark of the Huntsman";
const CHELICERA: &str = "Graphene Chelicera";
/// Instant toujours jouable (P0 reçoit des ressources au début de chaque
/// test) : le joueur garde une option, le moteur ne passe pas tout seul.
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);
/// Attaque générique avec go again (coût 0, 3{p}).
const ASSAUT: (&str, Option<Color>) = ("Essai Assaut", R);

// ---------------------------------------------------------------------------
// Aides
// ---------------------------------------------------------------------------

/// Passe (priorité, défense, arsenal) jusqu'à ce que `cond` soit vraie ;
/// l'ordre des déclenchements est pris tel quel.
fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..400 {
        if t.g.is_over() || cond(t) {
            return;
        }
        match t.kind() {
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            DecisionKind::TriggerOrder | DecisionKind::TriggerFirst | DecisionKind::PitchOrder => {
                let c = t.options()[0];
                t.choisir(c)
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

fn jouer(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    jouer_obj(t, p, c);
    c
}

/// Joue une carte d'attaque puis passe jusqu'à ce qu'elle soit l'attaque active.
fn attaquer(t: &mut Partie, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = jouer(t, P0, nom, couleur);
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(c));
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

/// Attaque avec cette arme ; renvoie l'attaque-proxy (active).
fn attaquer_arme(t: &mut Partie, arme: ObjectId) -> ObjectId {
    activer(t, P0, arme, 0);
    let proxy = proxy_de(t, arme);
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(proxy));
    proxy
}

/// Passe jusqu'à l'étape de réaction (P0 a la priorité, défense déclarée).
fn jusqua_reactions(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.g.combat.step == Some(fab_engine::state::CombatStep::Reaction)
            && t.kind() == DecisionKind::Priority
            && t.joueur() == P0
    });
}

fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.g.stack.is_empty() && t.g.pending_triggers.is_empty() && t.g.tasks.is_empty()
    });
}

fn fin_du_combat(t: &mut Partie) {
    passer_jusqua_que(t, |t| !t.g.combat.open);
}

/// Répond aux questions des capacités (`DecisionKind::Effect`) par `c`.
fn repondre(t: &mut Partie, c: Choice) {
    assert_eq!(t.kind(), DecisionKind::Effect, "{:?}", t.noms(&t.options()));
    t.choisir(c);
}

fn jusqua_question(t: &mut Partie) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Effect);
}

/// Le joueur choisit cette cible.
fn cibler(t: &mut Partie, c: Target) {
    assert_eq!(t.kind(), DecisionKind::Target, "{:?}", t.noms(&t.options()));
    t.choisir(Choice::Target(c));
}

/// Le joueur choisit cet objet comme cible.
fn cibler_obj(t: &mut Partie, o: ObjectId) {
    let c = Target::Object(t.g.obj_ref(o));
    cibler(t, c);
}

/// Arakni (P0) contre Essai Héros (P1, 4 Essai Rempart en main).
fn arakni(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    arena0: &[&str],
) -> Partie {
    arakni_contre(main0, suite0, &[RAMPART; 4], RIEN, arena0)
}

fn arakni_contre(
    main0: &[(&str, Option<Color>)],
    suite0: &[(&str, Option<Color>)],
    main1: &[(&str, Option<Color>)],
    suite1: &[(&str, Option<Color>)],
    arena0: &[&str],
) -> Partie {
    let mut t = partie_vraie([ARAKNI, HEROS], main0, suite0, main1, suite1, arena0, &[]);
    // Ressources pour l'Essai Étincelle (et les coûts) pendant le 1er tour.
    t.g.player_mut(P0).resource_points = 9;
    t.g.refresh_priority();
    t
}

/// Le héros de P0 devient cet Agent of Chaos.
fn devenir(t: &mut Partie, nom: &str) {
    let c = t.g.db.find_name(nom).expect("agent codé");
    t.g.become_hero(P0, c);
    t.g.refresh_priority();
}

fn heros(t: &Partie, p: PlayerId) -> String {
    t.g.def(t.g.player(p).hero).name.clone()
}

fn pa(t: &Partie, p: PlayerId) -> u8 {
    t.g.player(p).action_points
}

/// Permanents de ce nom contrôlés par le joueur.
fn jetons(t: &Partie, p: PlayerId, nom: &str) -> Vec<ObjectId> {
    t.g.permanents
        .iter()
        .copied()
        .filter(|&o| t.g.obj(o).controller == p && t.g.def(o).name == nom)
        .collect()
}

fn armes(t: &Partie, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .weapons
        .iter()
        .map(|&w| t.g.def(w).name.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Arakni, Web of Deceit
// ---------------------------------------------------------------------------

#[test]
fn arakni_attaque_stealth_contre_un_heros_marque_gagne_1_puissance() {
    let mut t = arakni(
        &[("Double Trouble", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    t.g.mark(P1);
    let a = attaquer(&mut t, "Double Trouble", R);
    assert_eq!(t.g.power(a), 4);
}

#[test]
fn arakni_pas_de_bonus_contre_un_heros_non_marque() {
    let mut t = arakni(
        &[("Double Trouble", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Double Trouble", R);
    assert_eq!(t.g.power(a), 3);
}

#[test]
fn arakni_pas_de_bonus_pour_une_attaque_sans_stealth() {
    let mut t = arakni(
        &[
            ("Pursue to the Edge of Oblivion", R),
            RAMPART,
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    t.g.mark(P1);
    let a = attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    assert_eq!(t.g.power(a), 4);
}

#[test]
fn arakni_attaque_stealth_qui_touche_un_heros_marque_gagne_go_again() {
    let mut t = arakni(
        &[("Double Trouble", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    t.g.mark(P1);
    let a = attaquer(&mut t, "Double Trouble", R);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
    passer_jusqua_que(&mut t, |t| t.vie(P1) < 20);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 16);
    // La touche retire la marque (CR 9.3.3), le go again est quand même gagné.
    assert!(!t.g.player(P1).marked);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    fin_du_combat(&mut t);
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn arakni_attaque_stealth_qui_touche_un_heros_non_marque_sans_go_again() {
    let mut t = arakni(
        &[("Double Trouble", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Double Trouble", R);
    passer_jusqua_que(&mut t, |t| t.vie(P1) < 20);
    resoudre(&mut t);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn arakni_fin_de_tour_adversaire_marque_devient_un_agent_of_chaos() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[]);
    t.g.mark(P1);
    t.finir_le_tour();
    let hero = t.g.player(P0).hero;
    assert!(t.g.def(hero).traits.has(Traits::AGENT_OF_CHAOS));
    // CR 8.5.25e : le total de vie ne change pas.
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn arakni_fin_de_tour_adversaire_non_marque_reste_arakni() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[]);
    t.finir_le_tour();
    assert_eq!(heros(&t, P0), ARAKNI);
}

#[test]
fn agent_of_chaos_retourne_a_la_couvee_a_la_fin_de_votre_tour_suivant() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], &[RAMPART; 8], &[]);
    t.g.mark(P1);
    t.finir_le_tour(); // tour 1 (P0) : devient un Agent
    assert_ne!(heros(&t, P0), ARAKNI);
    t.finir_le_tour(); // tour 2 (P1) : toujours l'Agent
    assert_ne!(heros(&t, P0), ARAKNI);
    // Tour 3 (P0) : à la phase de fin, l'Agent retourne à la couvée. La
    // capacité d'Arakni ne se déclenche pas (le héros était l'Agent), même
    // si l'adversaire est marqué.
    t.g.mark(P1);
    t.finir_le_tour();
    assert_eq!(heros(&t, P0), ARAKNI);
}

// ---------------------------------------------------------------------------
// Agents of Chaos
// ---------------------------------------------------------------------------

#[test]
fn black_widow_defausse_une_carte_assassin_pour_3_puissance_et_bannissement_de_main() {
    let mut t = arakni(
        &[("Mark the Prey", R), ("Bite", B), ETINCELLE, RAMPART],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Black Widow");
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    activer(&mut t, P0, hero, 0);
    // Une seule carte Assassin en main : défaussée d'office ; cible = l'attaque.
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, a);
    }
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
    assert_eq!(t.g.power(a), 6);
    // Touche : P1 bannit une carte de sa main (au choix).
    passer_jusqua_que(&mut t, |t| {
        t.vie(P1) < 20 || t.kind() == DecisionKind::Effect
    });
    jusqua_question(&mut t);
    assert_eq!(t.joueur(), P1);
    let c = t.g.player(P1).hand[0];
    repondre(&mut t, Choice::Object(c));
    assert_eq!(t.g.obj(c).zone, Zone::Banished);
}

#[test]
fn black_widow_ne_cible_pas_une_attaque_non_assassin() {
    let mut t = arakni(
        &[
            ("Pursue to the Edge of Oblivion", R),
            ("Bite", B),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Black Widow");
    attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    assert!(!t.peut(Choice::Activate(hero, 0)));
}

#[test]
fn black_widow_sans_carte_assassin_en_main_ne_peut_pas_activer() {
    let mut t = arakni(
        &[("Mark the Prey", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Black Widow");
    attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    assert!(!t.peut(Choice::Activate(hero, 0)));
}

#[test]
fn funnel_web_l_attaque_stealth_bannit_la_carte_en_arsenal() {
    let mut t = arakni(
        &[("Mark the Prey", R), ("Bite", B), RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let ars = t.g.player(P1).hand[0];
    t.g.put_in_arsenal(ars);
    devenir(&mut t, "Arakni, Funnel Web");
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    activer(&mut t, P0, hero, 0);
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, a);
    }
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 14);
    assert_eq!(t.g.obj(ars).zone, Zone::Banished);
    assert_eq!(t.g.player(P1).arsenal, None);
}

#[test]
fn redback_l_attaque_stealth_gagne_3_puissance_et_go_again() {
    let mut t = arakni(
        &[("Mark the Prey", R), ("Bite", B), ETINCELLE, RAMPART],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Redback");
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    activer(&mut t, P0, hero, 0);
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, a);
    }
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 6);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn redback_une_fois_par_tour() {
    let mut t = arakni(
        &[("Mark the Prey", R), ("Bite", B), ("Bite", Y), ETINCELLE],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Redback");
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    activer(&mut t, P0, hero, 0);
    if t.kind() == DecisionKind::Effect {
        let c = t.en_main(P0, "Bite", B);
        repondre(&mut t, Choice::Object(c));
    }
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, a);
    }
    resoudre(&mut t);
    assert!(!t.peut(Choice::Activate(hero, 0)));
}

#[test]
fn tarantula_une_dague_qui_touche_fait_perdre_1_pv() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    devenir(&mut t, "Arakni, Tarantula");
    let dague = t.equipement(P0, HUNTSMAN);
    attaquer_arme(&mut t, dague);
    jusqua_question(&mut t); // Mark of the Huntsman : détruire et marquer ?
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    // 1 dégât de la dague + 1 PV perdu.
    assert_eq!(t.vie(P1), 18);
}

#[test]
fn tarantula_une_attaque_qui_n_est_pas_une_dague_ne_fait_pas_perdre_de_pv() {
    let mut t = arakni(
        &[("Mark the Prey", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Tarantula");
    attaquer(&mut t, "Mark the Prey", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn tarantula_defausse_une_carte_assassin_attaque_de_dague_plus_3() {
    let mut t = arakni(
        &[("Bite", B), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[HUNTSMAN],
    );
    devenir(&mut t, "Arakni, Tarantula");
    let dague = t.equipement(P0, HUNTSMAN);
    let a = attaquer_arme(&mut t, dague);
    jusqua_reactions(&mut t);
    let hero = t.g.player(P0).hero;
    activer(&mut t, P0, hero, 0);
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, a);
    }
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 4);
}

#[test]
fn orb_weaver_les_graphene_chelicerae_coutent_1_de_moins() {
    let mut t = arakni(
        &[("Orb-Weaver Spinneret", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Orb-Weaver Spinneret", R);
    resoudre(&mut t);
    let c = t.equipement(P0, CHELICERA);
    assert_eq!(t.g.activation_cost(c, 0), 1);
    devenir(&mut t, "Arakni, Orb-Weaver");
    assert_eq!(t.g.activation_cost(c, 0), 0);
}

#[test]
fn orb_weaver_instant_equipe_une_chelicera_et_la_prochaine_attaque_stealth_plus_3() {
    let mut t = arakni(
        &[("Mark the Prey", R), ("Bite", B), ETINCELLE, RAMPART],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Orb-Weaver");
    let hero = t.g.player(P0).hero;
    activer(&mut t, P0, hero, 0);
    let bite = t.en_main(P0, "Bite", B);
    repondre(&mut t, Choice::Object(bite));
    resoudre(&mut t);
    assert_eq!(armes(&t, P0), vec![CHELICERA.to_string()]);
    let a = attaquer(&mut t, "Mark the Prey", R);
    assert_eq!(t.g.power(a), 6);
}

#[test]
fn trap_door_cherche_un_piege_banni_face_cachee_jouable_jusqu_au_prochain_tour() {
    let mut t = arakni(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Lair of the Spider", R), RAMPART],
        &[],
    );
    devenir(&mut t, "Arakni, Trap-Door");
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    let lair =
        *t.g.player(P0)
            .deck
            .iter()
            .find(|&&c| t.g.def(c).name == "Lair of the Spider")
            .unwrap();
    repondre(&mut t, Choice::Object(lair));
    resoudre(&mut t);
    assert_eq!(t.g.obj(lair).zone, Zone::Banished);
    assert!(t.g.obj(lair).face_down);
    assert!(t.g.play_permitted(P0, lair));
    let turn = t.g.turn;
    assert!(t
        .g
        .play_permissions
        .iter()
        .any(|p| p.turn == turn + 1 && p.card.id == lair));
    assert!(!t
        .g
        .play_permissions
        .iter()
        .any(|p| p.turn == turn + 2 && p.card.id == lair));
}

#[test]
fn trap_door_une_carte_qui_n_est_pas_un_piege_est_bannie_sans_permission() {
    let mut t = arakni(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Bite", R), RAMPART],
        &[],
    );
    devenir(&mut t, "Arakni, Trap-Door");
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    let bite =
        *t.g.player(P0)
            .deck
            .iter()
            .find(|&&c| t.g.def(c).name == "Bite")
            .unwrap();
    repondre(&mut t, Choice::Object(bite));
    resoudre(&mut t);
    assert_eq!(t.g.obj(bite).zone, Zone::Banished);
    assert!(!t.g.play_permitted(P0, bite));
}

#[test]
fn trap_door_le_joueur_peut_refuser_de_chercher() {
    let mut t = arakni(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Bite", R), RAMPART],
        &[],
    );
    let deck = t.g.player(P0).deck.len();
    devenir(&mut t, "Arakni, Trap-Door");
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).deck.len(), deck);
    assert!(t.g.player(P0).banished.is_empty());
}

#[test]
fn trap_door_le_piege_banni_se_joue_en_defense_pendant_le_tour_adverse() {
    let mut t = arakni_contre(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[
            ("Lair of the Spider", R),
            RAMPART,
            RAMPART,
            RAMPART,
            RAMPART,
        ],
        &[ASSAUT, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    devenir(&mut t, "Arakni, Trap-Door");
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    let lair =
        *t.g.player(P0)
            .deck
            .iter()
            .find(|&&c| t.g.def(c).name == "Lair of the Spider")
            .unwrap();
    repondre(&mut t, Choice::Object(lair));
    resoudre(&mut t);
    t.finir_le_tour();
    // Tour de P1 : il attaque avec Essai Assaut (go again) ; P0 joue Lair.
    let a = jouer(&mut t, P1, "Essai Assaut", R);
    passer_jusqua_que(&mut t, |t| {
        t.g.active_attack() == Some(a) && t.kind() == DecisionKind::Defend
    });
    t.choisir(Choice::Pass);
    jouer_obj(&mut t, P0, lair);
    resoudre(&mut t);
    assert!(t.g.player(P1).marked);
}

// ---------------------------------------------------------------------------
// Équipements et armes
// ---------------------------------------------------------------------------

#[test]
fn danger_digits_une_dague_hors_du_maillon_inflige_1_degat_et_touche() {
    let mut t = arakni(
        &[("Mark the Prey", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &["Danger Digits", HUNTSMAN],
    );
    t.g.mark(P1);
    attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let dd = t.equipement(P0, "Danger Digits");
    let dague = t.equipement(P0, HUNTSMAN);
    activer(&mut t, P0, dd, 0);
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, dague);
    }
    // La dague a touché : Mark of the Huntsman demande s'il faut la détruire.
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    // La touche de la dague a retiré la marque (CR 9.3.3).
    assert!(!t.g.player(P1).marked);
    assert_eq!(t.g.obj(dague).zone, Zone::Graveyard);
    assert_eq!(t.g.obj(dd).zone, Zone::Graveyard);
}

#[test]
fn danger_digits_ne_cible_pas_la_dague_du_maillon_actif() {
    let mut t = arakni(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &["Danger Digits", HUNTSMAN],
    );
    let dague = t.equipement(P0, HUNTSMAN);
    attaquer_arme(&mut t, dague);
    jusqua_reactions(&mut t);
    let dd = t.equipement(P0, "Danger Digits");
    assert!(!t.peut(Choice::Activate(dd, 0)));
}

#[test]
fn mark_of_the_huntsman_attaque_pour_2_avec_go_again() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    assert_eq!(t.g.activation_cost(dague, 0), 2);
    let a = attaquer_arme(&mut t, dague);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    assert_eq!(t.g.power(a), 1);
}

#[test]
fn mark_of_the_huntsman_contre_un_heros_marque_gagne_1_puissance() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    t.g.mark(P1);
    let dague = t.equipement(P0, HUNTSMAN);
    let a = attaquer_arme(&mut t, dague);
    assert_eq!(t.g.power(a), 2);
}

#[test]
fn mark_of_the_huntsman_qui_touche_peut_se_detruire_pour_marquer() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    attaquer_arme(&mut t, dague);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Yes);
    assert!(t.g.player(P1).marked);
    assert_eq!(t.g.obj(dague).zone, Zone::Graveyard);
}

#[test]
fn mark_of_the_huntsman_qui_touche_peut_refuser() {
    let mut t = arakni(&[ETINCELLE, RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    attaquer_arme(&mut t, dague);
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    assert!(!t.g.player(P1).marked);
    assert!(t.g.obj(dague).zone.in_arena());
}

#[test]
fn prey_spotters_se_detruit_pour_marquer_le_heros_adverse() {
    let mut t = arakni(
        &[("Mark the Prey", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &["Prey Spotters"],
    );
    let ps = t.equipement(P0, "Prey Spotters");
    assert!(t.g.keywords(ps).has(Keywords::BATTLEWORN));
    attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    activer(&mut t, P0, ps, 0);
    resoudre(&mut t);
    assert!(t.g.player(P1).marked);
    assert_eq!(t.g.obj(ps).zone, Zone::Graveyard);
}

#[test]
fn stalkers_steps_donne_go_again_a_une_attaque_stealth() {
    let mut t = arakni(
        &[("Mark the Prey", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &["Stalker's Steps"],
    );
    let st = t.equipement(P0, "Stalker's Steps");
    assert_eq!(t.g.prevention_values(st).0, 1);
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    activer(&mut t, P0, st, 0);
    resoudre(&mut t);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    assert_eq!(t.g.obj(st).zone, Zone::Graveyard);
}

#[test]
fn stalkers_steps_ne_cible_pas_une_attaque_sans_stealth() {
    let mut t = arakni(
        &[
            ("Pursue to the Edge of Oblivion", R),
            ETINCELLE,
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &["Stalker's Steps"],
    );
    attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    jusqua_reactions(&mut t);
    let st = t.equipement(P0, "Stalker's Steps");
    assert!(!t.peut(Choice::Activate(st, 0)));
}

#[test]
fn topsy_turvy_les_cartes_mises_sur_un_deck_vont_dessous() {
    let mut t = arakni(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART; 3],
        &["Topsy Turvy"],
    );
    let tt = t.equipement(P0, "Topsy Turvy");
    assert_eq!(t.g.prevention_values(tt).0, 1);
    activer(&mut t, P0, tt, 0);
    resoudre(&mut t);
    assert_eq!(t.g.obj(tt).zone, Zone::Graveyard);
    let c = t.g.player(P0).hand[0];
    t.g.put_on_top(c);
    assert_eq!(t.g.player(P0).deck[0], c, "la carte est sous le deck");
    // Aussi pour le deck de l'adversaire.
    let c1 = t.g.player(P1).hand[0];
    t.g.put_on_top(c1);
    assert_eq!(t.g.player(P1).deck[0], c1);
}

#[test]
fn topsy_turvy_dure_jusqu_a_la_fin_du_tour() {
    let mut t = arakni(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[RAMPART; 8],
        &["Topsy Turvy"],
    );
    let tt = t.equipement(P0, "Topsy Turvy");
    activer(&mut t, P0, tt, 0);
    resoudre(&mut t);
    t.finir_le_tour();
    let c = t.g.player(P1).hand[0];
    t.g.put_on_top(c);
    assert_eq!(*t.g.player(P1).deck.last().unwrap(), c);
}

#[test]
fn graphene_chelicera_attaque_pour_1_avec_stealth() {
    let mut t = arakni(
        &[("Orb-Weaver Spinneret", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Orb-Weaver Spinneret", R);
    resoudre(&mut t);
    let c = t.equipement(P0, CHELICERA);
    assert!(t.g.keywords(c).has(Keywords::STEALTH));
    let a = attaquer_arme(&mut t, c);
    // Orb-Weaver Spinneret : +3 à la prochaine attaque avec stealth.
    assert_eq!(t.g.power(a), 4);
    assert!(t.g.has_stealth(a));
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn graphene_chelicera_qui_attaque_un_heros_marque_gagne_go_again() {
    let mut t = arakni(
        &[("Orb-Weaver Spinneret", R), ETINCELLE, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Orb-Weaver Spinneret", R);
    resoudre(&mut t);
    t.g.mark(P1);
    let c = t.equipement(P0, CHELICERA);
    let a = attaquer_arme(&mut t, c);
    resoudre(&mut t);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    // Arakni : +1 contre un héros marqué (stealth), en plus du +3.
    assert_eq!(t.g.power(a), 5);
}

#[test]
fn bloodrot_pox_detruit_puis_2_degats_si_le_joueur_ne_paie_pas() {
    let mut t = arakni(&[("Infect", R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    attaquer(&mut t, "Infect", R);
    fin_du_combat(&mut t);
    assert_eq!(jetons(&t, P1, "Bloodrot Pox").len(), 1);
    t.finir_le_tour();
    // Fin du tour de P1 : la question de paiement.
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert_eq!(t.joueur(), P1);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert!(jetons(&t, P1, "Bloodrot Pox").is_empty());
    assert_eq!(t.vie(P1), 15);
}

#[test]
fn bloodrot_pox_le_joueur_peut_payer_3_pour_eviter_les_degats() {
    let mut t = arakni(&[("Infect", R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    attaquer(&mut t, "Infect", R);
    fin_du_combat(&mut t);
    t.finir_le_tour();
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    repondre(&mut t, Choice::Yes);
    payer(&mut t);
    resoudre(&mut t);
    assert!(jetons(&t, P1, "Bloodrot Pox").is_empty());
    assert_eq!(t.vie(P1), 17);
}

// ---------------------------------------------------------------------------
// Cartes du deck
// ---------------------------------------------------------------------------

#[test]
fn art_of_desire_body_touche_bannit_le_dessus_et_carte_rouge_pioche_et_soigne() {
    let mut t = arakni_contre(
        &[("Art of Desire: Body", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[RAMPART; 4],
        &[("Essai Étincelle", R)],
        &[],
    );
    t.g.player_mut(P0).life = 15;
    let top = *t.g.player(P1).deck.last().unwrap();
    assert_eq!(t.g.def(top).name, "Essai Étincelle");
    let main = t.g.player(P0).hand.len();
    attaquer(&mut t, "Art of Desire: Body", R);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(top).zone, Zone::Banished);
    assert_eq!(t.g.player(P0).hand.len(), main - 1 + 1);
    assert_eq!(t.vie(P0), 16);
}

#[test]
fn art_of_desire_body_carte_bannie_non_rouge_rien_de_plus() {
    let mut t = arakni_contre(
        &[("Art of Desire: Body", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[RAMPART; 4],
        &[RAMPART],
        &[],
    );
    let top = *t.g.player(P1).deck.last().unwrap();
    let main = t.g.player(P0).hand.len();
    attaquer(&mut t, "Art of Desire: Body", R);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(top).zone, Zone::Banished);
    assert_eq!(t.g.player(P0).hand.len(), main - 1);
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn bite_qui_attaque_une_dague_inflige_1_degat_touche_puis_est_detruite() {
    let mut t = arakni(&[("Bite", R), RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    jouer(&mut t, P0, "Bite", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    cibler_obj(&mut t, dague);
    jusqua_question(&mut t); // Mark of the Huntsman a touché
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.obj(dague).zone, Zone::Graveyard);
}

#[test]
fn bite_le_joueur_peut_ne_pas_choisir_de_dague() {
    let mut t = arakni(&[("Bite", R), RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    jouer(&mut t, P0, "Bite", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Pass);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20);
    assert!(t.g.obj(dague).zone.in_arena());
}

#[test]
fn concoct_disorder_chaque_heros_met_le_dessus_en_arsenal_et_go_again() {
    let mut t = arakni_contre(
        &[("Concoct Disorder", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[RAMPART; 4],
        &[RAMPART],
        &[],
    );
    let a = attaquer(&mut t, "Concoct Disorder", R);
    resoudre(&mut t);
    for p in PlayerId::BOTH {
        let ars = t.g.player(p).arsenal.expect("carte en arsenal");
        assert!(t.g.obj(ars).face_down);
    }
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn concoct_disorder_une_seule_carte_mise_en_arsenal_pas_de_go_again() {
    let mut t = arakni_contre(
        &[("Concoct Disorder", R), RAMPART, RAMPART, RAMPART],
        &[RAMPART],
        &[RAMPART; 4],
        &[RAMPART],
        &[],
    );
    let ars = t.g.player(P1).hand[0];
    t.g.put_in_arsenal(ars);
    let a = attaquer(&mut t, "Concoct Disorder", R);
    resoudre(&mut t);
    assert_eq!(t.g.player(P1).arsenal, Some(ars));
    assert!(t.g.player(P0).arsenal.is_some());
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn double_trouble_deux_reactions_d_attaque_plus_2_et_bannit_2_cartes() {
    let mut t = arakni_contre(
        &[
            ("Double Trouble", R),
            ("Razor's Edge", R),
            ("Razor's Edge", B),
            ETINCELLE,
        ],
        RIEN,
        &[RAMPART; 4],
        &[RAMPART, RAMPART, RAMPART],
        &[],
    );
    let a = attaquer(&mut t, "Double Trouble", R);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Razor's Edge", R);
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 6);
    jouer(&mut t, P0, "Razor's Edge", B);
    resoudre(&mut t);
    // 3 + 3 + 1 + 2.
    assert_eq!(t.g.power(a), 9);
    let deck = t.g.player(P1).deck.len();
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 11);
    assert_eq!(t.g.player(P1).deck.len(), deck - 2);
}

#[test]
fn double_trouble_une_reaction_jouee_et_une_activee_comptent() {
    let mut t = arakni(
        &[
            ("Double Trouble", R),
            ("Razor's Edge", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &["Prey Spotters"],
    );
    let a = attaquer(&mut t, "Double Trouble", R);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Razor's Edge", R);
    resoudre(&mut t);
    let ps = t.equipement(P0, "Prey Spotters");
    activer(&mut t, P0, ps, 0);
    resoudre(&mut t);
    // 3 + 3 + 2, et Arakni +1 (P1 marqué par Prey Spotters).
    assert_eq!(t.g.power(a), 9);
}

#[test]
fn double_trouble_une_seule_reaction_pas_de_bonus() {
    let mut t = arakni(
        &[
            ("Double Trouble", R),
            ("Razor's Edge", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Double Trouble", R);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Razor's Edge", R);
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 6);
    let deck = t.g.player(P1).deck.len();
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P1).deck.len(), deck);
}

#[test]
fn hurl_coute_additionnel_paye_la_dague_inflige_1_degat_quand_il_attaque() {
    let mut t = arakni(&[("Hurl", R), RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    let c = t.en_main(P0, "Hurl", R);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    assert_eq!(t.kind(), DecisionKind::OptionalCost);
    t.choisir(Choice::Yes);
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    if t.peut(Choice::Target(Target::Object(t.g.obj_ref(dague)))) {
        cibler_obj(&mut t, dague);
    }
    cibler(&mut t, Target::Player(P1));
    jusqua_question(&mut t);
    repondre(&mut t, Choice::No);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.obj(dague).zone, Zone::Graveyard);
    assert!(t.g.keywords(c).has(Keywords::GO_AGAIN));
}

#[test]
fn hurl_sans_le_cout_additionnel_rien_quand_il_attaque() {
    let mut t = arakni(&[("Hurl", R), RAMPART, RAMPART, RAMPART], RIEN, &[HUNTSMAN]);
    let dague = t.equipement(P0, HUNTSMAN);
    let c = t.en_main(P0, "Hurl", R);
    passer_jusqua_que(&mut t, |t| t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    t.choisir(Choice::No);
    passer_jusqua_que(&mut t, |t| t.g.active_attack() == Some(c));
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20);
    assert!(t.g.obj(dague).zone.in_arena());
}

#[test]
fn infect_touche_cree_un_bloodrot_pox_chez_l_adversaire() {
    let mut t = arakni(&[("Infect", R), RAMPART, RAMPART, RAMPART], RIEN, &[]);
    attaquer(&mut t, "Infect", R);
    fin_du_combat(&mut t);
    assert_eq!(jetons(&t, P1, "Bloodrot Pox").len(), 1);
    assert!(jetons(&t, P0, "Bloodrot Pox").is_empty());
}

/// P0 (Essai Héros) attaque ; P1 (Arakni) défend avec Lair of the Spider.
fn lair_contre(attaque: (&str, Option<Color>)) -> Partie {
    let mut t = partie_vraie(
        [HEROS, ARAKNI],
        &[attaque, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[("Lair of the Spider", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    );
    let a = jouer(&mut t, P0, attaque.0, attaque.1);
    passer_jusqua_que(&mut t, |t| {
        t.g.active_attack() == Some(a) && t.kind() == DecisionKind::Defend
    });
    t.choisir(Choice::Pass);
    jouer(&mut t, P1, "Lair of the Spider", R);
    resoudre(&mut t);
    t
}

#[test]
fn lair_of_the_spider_defend_une_attaque_avec_go_again_marque_l_attaquant() {
    let t = lair_contre(ASSAUT);
    assert!(t.g.player(P0).marked);
}

#[test]
fn lair_of_the_spider_attaque_sans_go_again_pas_de_marque() {
    let t = lair_contre(("Essai Frappe", R));
    assert!(!t.g.player(P0).marked);
}

#[test]
fn mark_of_the_black_widow_touche_un_heros_marque_il_bannit_une_carte_de_sa_main() {
    let mut t = arakni(
        &[("Mark of the Black Widow", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    t.g.mark(P1);
    attaquer(&mut t, "Mark of the Black Widow", R);
    jusqua_question(&mut t);
    assert_eq!(t.joueur(), P1);
    let c = t.g.player(P1).hand[1];
    repondre(&mut t, Choice::Object(c));
    resoudre(&mut t);
    assert_eq!(t.g.obj(c).zone, Zone::Banished);
    assert_eq!(t.g.player(P1).hand.len(), 3);
}

#[test]
fn mark_of_the_black_widow_heros_non_marque_rien() {
    let mut t = arakni(
        &[("Mark of the Black Widow", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Mark of the Black Widow", R);
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P1).hand.len(), 4);
    assert!(t.g.player(P1).banished.is_empty());
}

#[test]
fn mark_of_the_funnel_web_touche_un_heros_marque_bannit_son_arsenal() {
    let mut t = arakni(
        &[("Mark of the Funnel Web", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let ars = t.g.player(P1).hand[0];
    t.g.put_in_arsenal(ars);
    t.g.mark(P1);
    attaquer(&mut t, "Mark of the Funnel Web", R);
    fin_du_combat(&mut t);
    assert_eq!(t.g.obj(ars).zone, Zone::Banished);
}

#[test]
fn mark_of_the_funnel_web_heros_non_marque_rien() {
    let mut t = arakni(
        &[("Mark of the Funnel Web", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let ars = t.g.player(P1).hand[0];
    t.g.put_in_arsenal(ars);
    attaquer(&mut t, "Mark of the Funnel Web", R);
    fin_du_combat(&mut t);
    assert_eq!(t.g.player(P1).arsenal, Some(ars));
}

#[test]
fn mark_the_prey_touche_marque_le_heros() {
    let mut t = arakni(
        &[("Mark the Prey", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Mark the Prey", R);
    assert!(t.g.has_stealth(a));
    fin_du_combat(&mut t);
    assert!(t.g.player(P1).marked);
}

#[test]
fn nights_embrace_les_attaques_stealth_gagnent_1_ce_tour() {
    let mut t = arakni(
        &[
            ("Mark the Prey", R),
            ("Night's Embrace", B),
            ("Pick Up the Point", R),
            ("Infect", R),
        ],
        RIEN,
        &[],
    );
    // Première attaque (go again) : Pick Up the Point n'a pas stealth.
    let p = attaquer(&mut t, "Pick Up the Point", R);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Night's Embrace", B);
    resoudre(&mut t);
    assert_eq!(t.g.power(p), 3);
    fin_du_combat(&mut t);
    // Les attaques avec stealth suivantes du tour ont +1.
    let a = attaquer(&mut t, "Mark the Prey", R);
    assert_eq!(t.g.power(a), 4);
}

#[test]
fn orb_weaver_spinneret_equipe_une_chelicera_plus_3_et_go_again() {
    let mut t = arakni(
        &[
            ("Orb-Weaver Spinneret", R),
            ("Mark the Prey", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Orb-Weaver Spinneret", R);
    resoudre(&mut t);
    assert_eq!(armes(&t, P0), vec![CHELICERA.to_string()]);
    assert_eq!(pa(&t, P0), 1);
    let a = attaquer(&mut t, "Mark the Prey", R);
    assert_eq!(t.g.power(a), 6);
}

#[test]
fn orb_weaver_spinneret_sans_main_libre_pas_de_chelicera() {
    let mut t = arakni(
        &[("Orb-Weaver Spinneret", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[HUNTSMAN, HUNTSMAN],
    );
    jouer(&mut t, P0, "Orb-Weaver Spinneret", B);
    resoudre(&mut t);
    assert_eq!(
        armes(&t, P0),
        vec![HUNTSMAN.to_string(), HUNTSMAN.to_string()]
    );
}

#[test]
fn pick_up_the_point_qui_attaque_peut_reprendre_une_dague_du_cimetiere() {
    let mut t = arakni(
        &[("Pick Up the Point", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[HUNTSMAN],
    );
    let dague = t.equipement(P0, HUNTSMAN);
    t.g.destroy(dague);
    t.g.refresh_priority();
    let a = attaquer(&mut t, "Pick Up the Point", R);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    jusqua_question(&mut t);
    repondre(&mut t, Choice::Object(dague));
    payer(&mut t);
    resoudre(&mut t);
    assert_eq!(armes(&t, P0), vec![HUNTSMAN.to_string()]);
    assert_eq!(t.g.obj(dague).zone, Zone::Weapon);
}

#[test]
fn pick_up_the_point_sans_dague_au_cimetiere_pas_de_question() {
    let mut t = arakni(
        &[("Pick Up the Point", R), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Pick Up the Point", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn razors_edge_une_carte_d_attaque_avec_stealth_gagne_3() {
    let mut t = arakni(
        &[
            ("Mark the Prey", R),
            ("Razor's Edge", Y),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Razor's Edge", Y);
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 5);
}

#[test]
fn razors_edge_ne_cible_pas_une_attaque_d_arme() {
    let mut t = arakni(
        &[
            ("Orb-Weaver Spinneret", B),
            ("Razor's Edge", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Orb-Weaver Spinneret", B);
    resoudre(&mut t);
    let c = t.equipement(P0, CHELICERA);
    attaquer_arme(&mut t, c);
    jusqua_reactions(&mut t);
    let r = t.en_main(P0, "Razor's Edge", R);
    assert!(!t.peut(Choice::Play(r)));
}

#[test]
fn reapers_call_se_defausse_pour_marquer_le_heros_adverse() {
    let mut t = arakni(
        &[("Reaper's Call", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let c = t.en_main(P0, "Reaper's Call", B);
    assert!(t.g.has_stealth(c));
    activer(&mut t, P0, c, 0);
    resoudre(&mut t);
    assert!(t.g.player(P1).marked);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
}

#[test]
fn shred_une_carte_qui_defend_contre_une_attaque_assassin_perd_4_defense() {
    let mut t = arakni(
        &[("Mark the Prey", R), ("Shred", R), ETINCELLE, RAMPART],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Mark the Prey", R);
    let _ = a;
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let def = t.g.player(P1).hand[0];
    t.choisir(Choice::Defend(def));
    t.choisir(Choice::Pass);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Shred", R);
    if t.kind() == DecisionKind::Target {
        cibler_obj(&mut t, def);
    }
    resoudre(&mut t);
    assert_eq!(t.g.defense(def), 0);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn shred_ne_cible_pas_une_carte_qui_defend_contre_une_attaque_non_assassin() {
    let mut t = arakni(
        &[
            ("Pursue to the Edge of Oblivion", R),
            ("Shred", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Defend);
    let def = t.g.player(P1).hand[0];
    t.choisir(Choice::Defend(def));
    t.choisir(Choice::Pass);
    jusqua_reactions(&mut t);
    let s = t.en_main(P0, "Shred", R);
    assert!(!t.peut(Choice::Play(s)));
}

#[test]
fn stains_of_the_redback_coute_1_de_moins_si_le_heros_qui_defend_est_marque() {
    let mut t = arakni(
        &[
            ("Mark the Prey", R),
            ("Stains of the Redback", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    let s = t.en_main(P0, "Stains of the Redback", R);
    assert_eq!(t.g.play_cost(P0, s), 1);
    t.g.mark(P1);
    assert_eq!(t.g.play_cost(P0, s), 0);
}

#[test]
fn stains_of_the_redback_attaque_stealth_plus_3_et_go_again() {
    let mut t = arakni(
        &[
            ("Mark the Prey", R),
            ("Stains of the Redback", R),
            ETINCELLE,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    let a = attaquer(&mut t, "Mark the Prey", R);
    jusqua_reactions(&mut t);
    jouer(&mut t, P0, "Stains of the Redback", R);
    resoudre(&mut t);
    assert_eq!(t.g.power(a), 6);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

// ---------------------------------------------------------------------------
// Cartes génériques du deck
// ---------------------------------------------------------------------------

#[test]
fn public_bounty_marque_et_la_prochaine_attaque_contre_un_heros_marque_gagne_3() {
    let mut t = arakni(
        &[
            ("Public Bounty", R),
            ("Pursue to the Edge of Oblivion", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Public Bounty", R);
    resoudre(&mut t);
    assert!(t.g.player(P1).marked);
    assert_eq!(pa(&t, P0), 1);
    let a = attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    assert_eq!(t.g.power(a), 7);
}

#[test]
fn public_bounty_pas_de_bonus_si_le_heros_n_est_plus_marque() {
    let mut t = arakni(
        &[
            ("Public Bounty", B),
            ("Pursue to the Edge of Oblivion", R),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    jouer(&mut t, P0, "Public Bounty", B);
    resoudre(&mut t);
    t.g.player_mut(P1).marked = false;
    let a = attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    assert_eq!(t.g.power(a), 4);
    // L'effet attend toujours la prochaine attaque contre un héros marqué.
    assert!(t.g.next_attack_effects.iter().any(|e| e.at_attack));
}

#[test]
fn pursue_to_the_edge_of_oblivion_touche_marque() {
    let mut t = arakni(
        &[
            ("Pursue to the Edge of Oblivion", R),
            RAMPART,
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Pursue to the Edge of Oblivion", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 16);
    assert!(t.g.player(P1).marked);
}

#[test]
fn pursue_to_the_pits_of_despair_touche_marque() {
    let mut t = arakni(
        &[
            ("Pursue to the Pits of Despair", R),
            RAMPART,
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Pursue to the Pits of Despair", R);
    fin_du_combat(&mut t);
    assert_eq!(t.vie(P1), 15);
    assert!(t.g.player(P1).marked);
}

#[test]
fn relentless_pursuit_marque_et_va_au_cimetiere_sans_attaque() {
    let mut t = arakni(
        &[("Relentless Pursuit", B), RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
    );
    let c = jouer(&mut t, P0, "Relentless Pursuit", B);
    resoudre(&mut t);
    assert!(t.g.player(P1).marked);
    assert_eq!(t.g.obj(c).zone, Zone::Graveyard);
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn relentless_pursuit_apres_une_attaque_va_sous_le_deck_avec_go_again() {
    let mut t = arakni(
        &[
            ("Pick Up the Point", R),
            ("Relentless Pursuit", B),
            RAMPART,
            RAMPART,
        ],
        RIEN,
        &[],
    );
    attaquer(&mut t, "Pick Up the Point", R);
    fin_du_combat(&mut t);
    let c = jouer(&mut t, P0, "Relentless Pursuit", B);
    resoudre(&mut t);
    assert!(t.g.player(P1).marked);
    assert_eq!(t.g.obj(c).zone, Zone::Deck);
    assert_eq!(t.g.player(P0).deck[0], c);
    // CR 5.3.5a : go again de la couche.
    assert_eq!(pa(&t, P0), 1);
}

#[test]
fn le_deck_arakni_equipe_deux_mark_of_the_huntsman() {
    let db = fab_cards::real::build();
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/decks/09_arakni.txt"
    ))
    .unwrap();
    let list = fab_cards::decklist::Decklist::parse(&text).unwrap();
    let lo = fab_cards::real::default_loadout(&db, &list).unwrap();
    let n = lo
        .arena
        .iter()
        .filter(|&&c| db.get(c).name == HUNTSMAN)
        .count();
    assert_eq!(n, 2);
    let _ = Event::CombatChainClosed;
}
