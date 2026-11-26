//! Tests des vraies cartes du deck Aurora (Lightning, Elemental, Runeblade) :
//! un test par phrase du texte de chaque carte, plus des cas limites.

#![allow(unused_imports, dead_code)]

mod commun;
use commun::*;
use fab_engine::card::{Color, Supertypes};
use fab_engine::{Choice, DecisionKind, Keywords, ObjectId, PlayerId, Target, Zone};

const HEROS: &str = "Essai Héros";

fn passer_jusqua_que(t: &mut Partie, cond: impl Fn(&Partie) -> bool) {
    for _ in 0..300 {
        if t.g.is_over() || cond(t) {
            return;
        }
        match t.kind() {
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            k => panic!("décision {k:?} inattendue ; options {:?}", t.options()),
        }
    }
    panic!("trop de décisions");
}

/// Le joueur joue la carte de sa main dès qu'il le peut.
fn jouer(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    c
}

fn resoudre(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.g.stack.is_empty() && t.g.pending_triggers.is_empty() && t.g.tasks.is_empty()
    });
}

/// Partie où P0 a `main0` en main et P1 n'a rien.
fn partie_cloud(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

#[test]
fn cloud_cover_bleu_previent_1_degat_de_la_prochaine_fois() {
    let mut t = partie_cloud(&[("Cloud Cover", B)]);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 3, false);
    assert_eq!(t.vie(P0), 18, "3 - 1 prévenu");
}

#[test]
fn cloud_cover_rouge_previent_3_degats_de_la_prochaine_fois() {
    let mut t = partie_cloud(&[("Cloud Cover", R)]);
    jouer(&mut t, P0, "Cloud Cover", R);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 5, false);
    assert_eq!(t.vie(P0), 18, "5 - 3 prévenus");
}

#[test]
fn cloud_cover_jaune_previent_2_degats_de_la_prochaine_fois() {
    let mut t = partie_cloud(&[("Cloud Cover", Y)]);
    jouer(&mut t, P0, "Cloud Cover", Y);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 4, false);
    assert_eq!(t.vie(P0), 18, "4 - 2 prévenus");
}

#[test]
fn cloud_cover_ne_previent_que_la_prochaine_fois_seulement() {
    let mut t = partie_cloud(&[("Cloud Cover", R)]);
    jouer(&mut t, P0, "Cloud Cover", R);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 3, false);
    t.g.deal_damage(None, P1, P0, 3, false);
    assert_eq!(
        t.vie(P0),
        17,
        "1er coup : 3 prévenus ; 2e coup : rien de prévenu"
    );
}

#[test]
fn cloud_cover_previent_au_plus_les_degats_recus() {
    let mut t = partie_cloud(&[("Cloud Cover", R)]);
    jouer(&mut t, P0, "Cloud Cover", R);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 2, false);
    assert_eq!(t.vie(P0), 20, "2 dégâts, tous prévenus");
}

fn sizzle_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

/// Sizzle : « Your next Lightning or Elemental attack this turn gets +3{p} »
/// (rouge) et +2 (jaune).
#[test]
fn sizzle_rouge_prochaine_attaque_elemental_plus_3() {
    let mut t = sizzle_partie(&[("Sizzle", R), ("Colors of Aria", R)]);
    jouer(&mut t, P0, "Sizzle", R);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Colors of Aria", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7 + 3, "Colors of Aria (Elemental) : 7 + 3");
}

#[test]
fn sizzle_jaune_prochaine_attaque_elemental_plus_2() {
    let mut t = sizzle_partie(&[("Sizzle", Y), ("Colors of Aria", R)]);
    jouer(&mut t, P0, "Sizzle", Y);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Colors of Aria", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7 + 2, "jaune : +2");
}

#[test]
fn sizzle_go_again_rend_un_point_d_action() {
    let mut t = sizzle_partie(&[("Sizzle", R)]);
    jouer(&mut t, P0, "Sizzle", R);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn sizzle_pas_pour_une_attaque_sans_lightning_ni_elemental() {
    let mut t = sizzle_partie(&[("Sizzle", R), ("Autumn's Touch", R)]);
    jouer(&mut t, P0, "Sizzle", R);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Autumn's Touch", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7, "Autumn's Touch (Runeblade) : pas de bonus");
}

/// Avance jusqu'à ce que `a` soit l'attaque active ; paie les coûts en
/// « pitch » la première option proposée.
fn jusqua_active(t: &mut Partie, a: ObjectId) {
    for _ in 0..300 {
        if t.g.active_attack() == Some(a) {
            return;
        }
        match t.kind() {
            DecisionKind::Pitch => {
                let o = t.options()[0];
                t.choisir(o)
            }
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            k => panic!("décision {k:?} inattendue ; options {:?}", t.options()),
        }
    }
    panic!("trop de décisions");
}

fn discharge_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

/// Electrostatic Discharge : « The next attack action card you play this turn
/// with cost 1 or less gets +1{p} » (bleu), +3 (rouge), +2 (jaune).
#[test]
fn electrostatic_discharge_rouge_prochaine_attaque_de_cout_1_ou_moins_plus_3() {
    let mut t = discharge_partie(&[("Electrostatic Discharge", R), ("Sweeping Blow", R)]);
    jouer(&mut t, P0, "Electrostatic Discharge", R);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Sweeping Blow", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 3 + 3, "Sweeping Blow (coût 1) : 3 + 3");
}

#[test]
fn electrostatic_discharge_jaune_prochaine_attaque_de_cout_1_ou_moins_plus_2() {
    let mut t = discharge_partie(&[("Electrostatic Discharge", Y), ("Sweeping Blow", R)]);
    jouer(&mut t, P0, "Electrostatic Discharge", Y);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Sweeping Blow", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 3 + 2, "jaune : +2");
}

#[test]
fn electrostatic_discharge_bleu_prochaine_attaque_de_cout_1_ou_moins_plus_1() {
    let mut t = discharge_partie(&[("Electrostatic Discharge", B), ("Sweeping Blow", R)]);
    jouer(&mut t, P0, "Electrostatic Discharge", B);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Sweeping Blow", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 3 + 1, "bleu : +1");
}

#[test]
fn electrostatic_discharge_ne_s_applique_pas_a_une_attaque_de_cout_3() {
    let mut t = discharge_partie(&[("Electrostatic Discharge", R), ("Colors of Aria", R)]);
    jouer(&mut t, P0, "Electrostatic Discharge", R);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Colors of Aria", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7, "Colors of Aria coûte 3 : pas de bonus");
}

#[test]
fn electrostatic_discharge_ne_vaut_que_pour_la_prochaine_attaque() {
    let mut t = discharge_partie(&[("Electrostatic Discharge", R), ("Sweeping Blow", R)]);
    jouer(&mut t, P0, "Electrostatic Discharge", R);
    resoudre(&mut t);
    assert_eq!(t.g.next_attack_effects.len(), 1, "un effet en attente");
    let a = jouer(&mut t, P0, "Sweeping Blow", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 6);
    assert!(
        t.g.next_attack_effects.is_empty(),
        "consommé par la première attaque"
    );
}

#[test]
fn electrostatic_discharge_instant_ne_consomme_pas_de_point_d_action() {
    let mut t = discharge_partie(&[("Electrostatic Discharge", R)]);
    let avant = t.g.player(P0).action_points;
    jouer(&mut t, P0, "Electrostatic Discharge", R);
    resoudre(&mut t);
    assert_eq!(
        t.g.player(P0).action_points,
        avant,
        "instant : aucun point d'action dépensé"
    );
}

// ---------------------------------------------------------------------------
// Stinging Sprite : « When this attacks or defends, deal 1 arcane damage to
// target hero. » (puissance : bleu 1, rouge 3, jaune 2)
// ---------------------------------------------------------------------------

fn sprite_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

/// Passe jusqu'à ce que le joueur doive choisir une cible, puis choisit `p`.
fn cibler_heros(t: &mut Partie, p: PlayerId) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Target);
    t.choisir(Choice::Target(Target::Player(p)));
}

/// Vrai si un dégât arcanique de 1 a été infligé au héros `p`.
fn arcanique_1_a(t: &Partie, p: PlayerId) -> bool {
    t.g.turn_events().iter().any(|e| {
        matches!(
            e,
            fab_engine::Event::Damage {
                target,
                amount: 1,
                arcane: true,
                ..
            } if *target == p
        )
    })
}

#[test]
fn stinging_sprite_quand_il_attaque_inflige_1_degat_arcanique_a_la_cible() {
    let mut t = sprite_partie(&[("Stinging Sprite", B)]);
    jouer(&mut t, P0, "Stinging Sprite", B);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert!(arcanique_1_a(&t, P1), "1 dégât arcanique à P1");
}

#[test]
fn stinging_sprite_quand_il_defend_inflige_1_degat_arcanique_a_la_cible() {
    let mut t = partie_vraie(
        [HEROS, HEROS],
        &[("Sweeping Blow", R)],
        RIEN,
        &[("Stinging Sprite", B)],
        RIEN,
        &[],
        &[],
    );
    jouer(&mut t, P0, "Sweeping Blow", R);
    for _ in 0..300 {
        if t.kind() == DecisionKind::Defend {
            break;
        }
        match t.kind() {
            DecisionKind::Pitch => {
                let o = t.options()[0];
                t.choisir(o)
            }
            DecisionKind::Priority | DecisionKind::Arsenal => t.choisir(Choice::Pass),
            k => panic!("décision {k:?} inattendue ; options {:?}", t.options()),
        }
    }
    let sprite = t.en_main(P1, "Stinging Sprite", B);
    t.choisir(Choice::Defend(sprite));
    cibler_heros(&mut t, P0);
    resoudre(&mut t);
    assert!(
        arcanique_1_a(&t, P0),
        "1 dégât arcanique à P0 (cible choisie)"
    );
}

#[test]
fn stinging_sprite_la_cible_est_un_heros_quelconque_y_compris_le_sien() {
    let mut t = sprite_partie(&[("Stinging Sprite", R)]);
    jouer(&mut t, P0, "Stinging Sprite", R);
    cibler_heros(&mut t, P0);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), 19, "1 dégât arcanique sur son propre héros");
    assert!(arcanique_1_a(&t, P0));
}

#[test]
fn stinging_sprite_puissance_bleu_1_rouge_3_jaune_2() {
    for (c, attendu) in [(B, 1), (R, 3), (Y, 2)] {
        let mut t = sprite_partie(&[("Stinging Sprite", c)]);
        let a = jouer(&mut t, P0, "Stinging Sprite", c);
        assert_eq!(t.g.power(a), attendu, "puissance selon la couleur");
    }
}

/// Met une Lightning Surge de couleur `c` dans l'arsenal de P0 et la joue
/// depuis là.
fn jouer_surge_depuis_arsenal(t: &mut Partie, c: Option<Color>) -> ObjectId {
    let id = t.en_main(P0, "Lightning Surge", c);
    t.g.put_in_arsenal(id);
    t.g.refresh_priority();
    passer_jusqua_que(t, |t| t.joueur() == P0 && t.peut(Choice::Play(id)));
    t.choisir(Choice::Play(id));
    id
}

#[test]
fn lightning_surge_jouee_depuis_l_arsenal_a_go_again() {
    let mut t = partie_cloud(&[("Lightning Surge", B)]);
    let a = jouer_surge_depuis_arsenal(&mut t, B);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn lightning_surge_jouee_depuis_la_main_n_a_pas_go_again() {
    let mut t = partie_cloud(&[("Lightning Surge", R)]);
    let a = jouer(&mut t, P0, "Lightning Surge", R);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn lightning_surge_puissance_bleu_2_rouge_4_jaune_3_defense_2() {
    for (c, puissance) in [(B, 2), (R, 4), (Y, 3)] {
        let mut t = partie_cloud(&[("Lightning Surge", c)]);
        let a = jouer(&mut t, P0, "Lightning Surge", c);
        assert_eq!(t.g.power(a), puissance, "puissance selon la couleur");
    }
}

#[test]
fn lightning_surge_jaune_jouee_depuis_l_arsenal_a_go_again() {
    let mut t = partie_cloud(&[("Lightning Surge", Y)]);
    let a = jouer_surge_depuis_arsenal(&mut t, Y);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

/// Partie où P0 joue `nom` de couleur `c` ; le texte « when this attacks »
/// est résolu dès que la carte est en attaque.
fn second_strike_attaque(t: &mut Partie, c: Option<Color>) -> ObjectId {
    let a = jouer(t, P0, "Second Strike", c);
    passer_jusqua_que(t, |t| {
        t.g.pending_triggers.is_empty() && t.g.stack.is_empty() && t.g.active_attack() == Some(a)
    });
    a
}

#[test]
fn second_strike_puissance_bleu_1_rouge_3_jaune_2_defense_2() {
    for (c, puissance) in [(B, 1), (R, 3), (Y, 2)] {
        let mut t = partie_cloud(&[("Second Strike", c)]);
        let a = jouer(&mut t, P0, "Second Strike", c);
        assert_eq!(t.g.power(a), puissance, "puissance selon la couleur");
        assert_eq!(t.g.def(a).defense, Some(2), "défense 2");
    }
}

#[test]
fn second_strike_quand_il_attaque_sans_degat_ce_tour_ne_gagne_rien() {
    let mut t = partie_cloud(&[("Second Strike", R)]);
    let a = second_strike_attaque(&mut t, R);
    assert_eq!(
        t.g.power(a),
        3,
        "pas de bonus de puissance sans dégât infligé ce tour"
    );
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn second_strike_ayant_inflige_des_degats_ce_tour_gagne_1_p() {
    let mut t = partie_cloud(&[("Second Strike", R)]);
    t.g.deal_damage(None, P0, P1, 1, false);
    let a = second_strike_attaque(&mut t, R);
    assert_eq!(t.g.power(a), 4, "bonus de puissance : 3 + 1");
}

#[test]
fn second_strike_ayant_inflige_des_degats_ce_tour_a_go_again() {
    let mut t = partie_cloud(&[("Second Strike", Y)]);
    t.g.deal_damage(None, P0, P1, 1, false);
    let a = second_strike_attaque(&mut t, Y);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn second_strike_degats_infliges_par_l_adversaire_ne_comptent_pas() {
    let mut t = partie_cloud(&[("Second Strike", B)]);
    t.g.deal_damage(None, P1, P0, 2, false);
    let a = second_strike_attaque(&mut t, B);
    assert_eq!(t.g.power(a), 1, "seuls les dégâts de P0 comptent");
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

// ---------------------------------------------------------------------------
// Rush of Power : « Quickstrike - If this has go again, it gets +1{p}.
// When this hits a hero, deal 1 arcane damage to them. » (défense 3 ;
// puissance : bleu 1, rouge 3, jaune 2)
// ---------------------------------------------------------------------------

/// Joue « Rush of Power » (avec `go_again` accordé par un effet) et l'attaque.
fn rush_attaque(t: &mut Partie, c: Option<Color>, go_again: bool) -> ObjectId {
    let carte = t.en_main(P0, "Rush of Power", c);
    if go_again {
        let r = t.g.obj(carte).obj_ref(carte);
        t.g.modify(
            r,
            fab_engine::Delta {
                grants: Keywords::GO_AGAIN,
                ..Default::default()
            },
            fab_engine::Until::EndOfTurn,
        );
    }
    let a = jouer(t, P0, "Rush of Power", c);
    passer_jusqua_que(t, |t| {
        t.g.pending_triggers.is_empty() && t.g.stack.is_empty() && t.g.active_attack() == Some(a)
    });
    a
}

fn rush_partie() -> Partie {
    partie_cloud(&[("Rush of Power", R)])
}

#[test]
fn rush_of_power_puissance_bleu_1_rouge_3_jaune_2_defense_3() {
    for (c, puissance) in [(B, 1), (R, 3), (Y, 2)] {
        let mut t = partie_cloud(&[("Rush of Power", c)]);
        let a = jouer(&mut t, P0, "Rush of Power", c);
        assert_eq!(t.g.power(a), puissance, "puissance selon la couleur");
        assert_eq!(t.g.def(a).defense, Some(3), "défense 3");
    }
}

#[test]
fn rush_of_power_quickstrike_avec_go_again_gagne_1_p() {
    let mut t = rush_partie();
    let a = rush_attaque(&mut t, R, true);
    assert_eq!(t.g.power(a), 4, "3 + 1 (quickstrike, go again)");
}

#[test]
fn rush_of_power_quickstrike_sans_go_again_ne_gagne_rien() {
    let mut t = rush_partie();
    let a = rush_attaque(&mut t, R, false);
    assert_eq!(t.g.power(a), 3, "pas de go again, pas de +1 de puissance");
}

#[test]
fn rush_of_power_quand_il_touche_un_heros_inflige_1_degat_arcanique() {
    let mut t = rush_partie();
    rush_attaque(&mut t, R, false);
    passer_jusqua_que(&mut t, |t| t.vie(P1) < 20);
    resoudre(&mut t);
    assert!(arcanique_1_a(&t, P1), "1 dégât arcanique au héros touché");
    assert_eq!(
        t.vie(P1),
        20 - 3 - 1,
        "3 de dégâts d'attaque puis 1 arcanique"
    );
}

#[test]
fn rush_of_power_degat_arcanique_suit_la_couleur_de_puissance_jaune() {
    let mut t = partie_cloud(&[("Rush of Power", Y)]);
    rush_attaque(&mut t, Y, false);
    passer_jusqua_que(&mut t, |t| t.vie(P1) < 20);
    resoudre(&mut t);
    assert!(arcanique_1_a(&t, P1));
    assert_eq!(
        t.vie(P1),
        20 - 2 - 1,
        "2 de dégâts d'attaque puis 1 arcanique"
    );
}

// ---------------------------------------------------------------------------
// Visionary of Orbits (rouge) : « When this hits, you may put an instant card
// from your graveyard on the bottom of your deck. » (coût 3, 7{p} 2{d})
// ---------------------------------------------------------------------------

/// Met au cimetière une carte de `p` (deck ou main, selon la pioche de départ).
fn au_cimetiere_depuis_deck(t: &mut Partie, p: PlayerId, nom: &str, c: Option<Color>) -> ObjectId {
    let joueur = t.g.player(p);
    let id = *joueur
        .deck
        .iter()
        .chain(joueur.hand.iter())
        .find(|&&x| t.g.def(x).name == nom && t.g.def(x).color == c)
        .unwrap_or_else(|| panic!("{nom} absente du deck et de la main de {p:?}"));
    t.g.put_in_graveyard(id);
    t.g.refresh_priority();
    id
}

/// Joue « Visionary of Orbits » et paie son coût (pitch) jusqu'à la décision suivante.
fn jouer_visionary(t: &mut Partie) -> ObjectId {
    let a = jouer(t, P0, "Visionary of Orbits", R);
    while t.kind() == DecisionKind::Pitch {
        let opts = t.options();
        t.choisir(opts[0]);
    }
    a
}

/// Partie : P0 a « Visionary of Orbits » en main et `suite0` dans son deck.
fn visionary_partie(suite0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        [HEROS, HEROS],
        &[("Visionary of Orbits", R)],
        suite0,
        RIEN,
        RIEN,
        &[],
        &[],
    )
}

#[test]
fn visionary_of_orbits_puissance_7_defense_2_cout_3() {
    let mut t = visionary_partie(&[]);
    let a = jouer(&mut t, P0, "Visionary of Orbits", R);
    assert_eq!(t.g.power(a), 7, "puissance 7");
    assert_eq!(t.g.def(a).defense, Some(2), "défense 2");
    assert_eq!(t.g.def(a).cost, Some(3), "coût 3");
}

#[test]
fn visionary_of_orbits_quand_il_touche_met_une_carte_instant_du_cimetiere_sous_le_deck() {
    let mut t = visionary_partie(&[("Cloud Cover", R)]);
    let instant = au_cimetiere_depuis_deck(&mut t, P0, "Cloud Cover", R);
    jouer_visionary(&mut t);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert!(
        t.peut(Choice::Object(instant)),
        "la carte instant du cimetière est proposée"
    );
    t.choisir(Choice::Object(instant));
    resoudre(&mut t);
    let deck = &t.g.player(P0).deck;
    assert_eq!(deck[0], instant, "la carte instant est sous le deck");
    assert_eq!(t.g.obj(instant).zone, Zone::Deck);
}

#[test]
fn visionary_of_orbits_may_passer_ne_met_rien_sous_le_deck() {
    let mut t = visionary_partie(&[("Cloud Cover", R)]);
    let instant = au_cimetiere_depuis_deck(&mut t, P0, "Cloud Cover", R);
    jouer_visionary(&mut t);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    t.choisir(Choice::Pass);
    resoudre(&mut t);
    assert_eq!(
        t.g.obj(instant).zone,
        Zone::Graveyard,
        "la carte reste au cimetière"
    );
}

#[test]
fn visionary_of_orbits_ne_propose_que_les_cartes_instant_de_son_cimetiere() {
    let mut t = visionary_partie(&[("Sizzle", R), ("Cloud Cover", R)]);
    let action = au_cimetiere_depuis_deck(&mut t, P0, "Sizzle", R);
    let instant = au_cimetiere_depuis_deck(&mut t, P0, "Cloud Cover", R);
    jouer_visionary(&mut t);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    assert!(t.peut(Choice::Object(instant)), "instant proposé");
    assert!(!t.peut(Choice::Object(action)), "action non proposée");
}

#[test]
fn visionary_of_orbits_sans_instant_au_cimetiere_ne_pose_pas_de_question() {
    let mut t = visionary_partie(&[("Sizzle", R)]);
    au_cimetiere_depuis_deck(&mut t, P0, "Sizzle", R);
    jouer_visionary(&mut t);
    passer_jusqua_que(&mut t, |t| t.vie(P1) < 20);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20 - 7, "touche : 7 de dégâts, sans question");
}
