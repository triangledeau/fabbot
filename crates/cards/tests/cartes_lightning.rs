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

fn harness_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

#[test]
fn harness_lightning_lightning_flow_sans_autre_carte_lightning_jouee_ne_fait_rien() {
    let mut t = harness_partie(&[("Harness Lightning", R)]);
    jouer(&mut t, P0, "Harness Lightning", R);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 20, "aucune carte Lightning jouée ce tour");
}

#[test]
fn harness_lightning_lightning_flow_rouge_avec_carte_lightning_jouee_inflige_3_arcanique() {
    let mut t = harness_partie(&[("Cloud Cover", B), ("Harness Lightning", R)]);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    jouer(&mut t, P0, "Harness Lightning", R);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 17, "3 dégâts arcaniques au héros visé");
    assert!(
        t.g.turn_events().iter().any(|e| matches!(
            e,
            fab_engine::Event::Damage { target, amount: 3, arcane: true, .. } if *target == P1
        )),
        "les dégâts sont arcaniques"
    );
}

#[test]
fn harness_lightning_lightning_flow_jaune_avec_carte_lightning_jouee_inflige_2_arcanique() {
    let mut t = harness_partie(&[("Cloud Cover", Y), ("Harness Lightning", Y)]);
    jouer(&mut t, P0, "Cloud Cover", Y);
    resoudre(&mut t);
    jouer(&mut t, P0, "Harness Lightning", Y);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18, "2 dégâts arcaniques au héros visé");
}

#[test]
fn harness_lightning_cible_un_heros_et_seulement_un_heros() {
    let mut t = harness_partie(&[("Cloud Cover", B), ("Harness Lightning", R)]);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    jouer(&mut t, P0, "Harness Lightning", R);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Target);
    assert!(
        t.peut(Choice::Target(Target::Player(P0))),
        "héros de P0 proposé"
    );
    assert!(
        t.peut(Choice::Target(Target::Player(P1))),
        "héros de P1 proposé"
    );
}

// ---------------------------------------------------------------------------
// Static Shock : « When this hits a hero, if you've played a Lightning card
// this turn, deal 1 arcane damage to them. » (rouge : 4 ; jaune : 3)
// ---------------------------------------------------------------------------

/// Résout l'attaque jusqu'à ce que la chaîne de combat soit vide (les
/// décisions de défense, `Pass` compris, sont passées).
fn resoudre_attaque(t: &mut Partie) {
    passer_jusqua_que(t, |t| {
        t.kind() != DecisionKind::Defend
            && t.kind() != DecisionKind::Pitch
            && t.g.stack.is_empty()
            && t.g.pending_triggers.is_empty()
            && t.g.tasks.is_empty()
    });
}

fn static_shock_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

#[test]
fn static_shock_lightning_flow_touche_un_heros_avec_carte_lightning_jouee_inflige_1_arcanique() {
    let mut t = static_shock_partie(&[("Cloud Cover", B), ("Static Shock", R)]);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    jouer(&mut t, P0, "Static Shock", R);
    resoudre_attaque(&mut t);
    assert_eq!(t.vie(P1), 15, "4 de puissance + 1 dégât arcanique");
    assert!(arcanique_1_a(&t, P1), "1 dégât arcanique à P1");
}

#[test]
fn static_shock_jaune_avec_carte_lightning_jouee_inflige_3_puis_1_arcanique() {
    let mut t = static_shock_partie(&[("Cloud Cover", Y), ("Static Shock", Y)]);
    jouer(&mut t, P0, "Cloud Cover", Y);
    resoudre(&mut t);
    jouer(&mut t, P0, "Static Shock", Y);
    resoudre_attaque(&mut t);
    assert_eq!(t.vie(P1), 16, "3 de puissance + 1 dégât arcanique");
}

#[test]
fn static_shock_sans_autre_carte_lightning_jouee_ne_fait_que_sa_puissance() {
    let mut t = static_shock_partie(&[("Static Shock", R)]);
    jouer(&mut t, P0, "Static Shock", R);
    resoudre_attaque(&mut t);
    assert_eq!(t.vie(P1), 16, "4 de puissance, pas de dégât arcanique");
    assert!(!arcanique_1_a(&t, P1), "aucun dégât arcanique");
}

#[test]
fn static_shock_lui_meme_n_est_pas_une_carte_lightning() {
    let mut t = static_shock_partie(&[("Static Shock", Y)]);
    jouer(&mut t, P0, "Static Shock", Y);
    resoudre_attaque(&mut t);
    assert!(
        !arcanique_1_a(&t, P1),
        "la carte jouée ne compte pas pour son propre Lightning Flow"
    );
}

// Path of Same Ends

/// Partie où P0 a `main0` en main et P1 n'a rien.
fn path_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

/// Joue Path of Same Ends et attend que ses déclenchements soient résolus.
fn path_attaque(t: &mut Partie, c: Option<Color>) -> ObjectId {
    let a = jouer(t, P0, "Path of Same Ends", c);
    passer_jusqua_que(t, |t| {
        t.g.pending_triggers.is_empty() && t.g.stack.is_empty() && t.g.active_attack() == Some(a)
    });
    a
}

/// Paie les pitchs avec la carte de remplissage « Essai Rempart ».
fn payer_avec_rempart(t: &mut Partie) {
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

#[test]
fn path_of_same_ends_puissance_et_defense_selon_la_couleur() {
    for (c, puissance) in [(B, 1), (R, 3), (Y, 2)] {
        let mut t = path_partie(&[("Path of Same Ends", c)]);
        let a = jouer(&mut t, P0, "Path of Same Ends", c);
        assert_eq!(t.g.power(a), puissance, "puissance selon la couleur");
        assert_eq!(t.g.def(a).defense, Some(2), "défense 2");
    }
}

#[test]
fn path_of_same_ends_quand_il_attaque_un_heros_inflige_1_degat_arcanique() {
    let mut t = path_partie(&[("Path of Same Ends", R)]);
    path_attaque(&mut t, R);
    assert!(arcanique_1_a(&t, P1), "1 dégât arcanique au héros attaqué");
    assert_eq!(
        t.vie(P1),
        19,
        "1 arcanique ; l'attaque n'est pas encore résolue"
    );
}

#[test]
fn path_of_same_ends_si_des_degats_sont_infliges_il_gagne_go_again() {
    let mut t = path_partie(&[("Path of Same Ends", B)]);
    let a = path_attaque(&mut t, B);
    assert!(
        t.g.keywords(a).has(Keywords::GO_AGAIN),
        "go again après dégâts"
    );
}

#[test]
fn path_of_same_ends_sans_degats_infliges_il_n_a_pas_go_again() {
    let mut t = path_partie(&[("Path of Same Ends", B)]);
    t.g.prevent_next(P1, 5, false, None);
    let a = path_attaque(&mut t, B);
    assert!(
        !t.g.keywords(a).has(Keywords::GO_AGAIN),
        "tout prévenu, aucun dégât : pas de go again"
    );
}

// ---------------------------------------------------------------------------
// Twinkle Toes (équipement sans couleur, Lightning, Legs)
// ---------------------------------------------------------------------------

/// P0 a Twinkle Toes équipé et `main0` en main ; P1 n'a rien.
fn twinkle_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        [HEROS, HEROS],
        main0,
        RIEN,
        RIEN,
        RIEN,
        &["Twinkle Toes"],
        &[],
    )
}

/// Active Twinkle Toes (aucun coût à payer).
fn activer_twinkle(t: &mut Partie, w: ObjectId) {
    passer_jusqua_que(t, |t| t.joueur() == P0 && t.peut(Choice::Activate(w, 0)));
    t.choisir(Choice::Activate(w, 0));
}

#[test]
fn twinkle_toes_instant_detruit_quand_activee() {
    let mut t = twinkle_partie(&[("Essai Appel", B)]);
    let w = t.equipement(P0, "Twinkle Toes");
    jouer(&mut t, P0, "Essai Appel", B);
    resoudre(&mut t);
    activer_twinkle(&mut t, w);
    resoudre(&mut t);
    assert_eq!(
        t.g.obj(w).zone,
        Zone::Graveyard,
        "détruite par son activation"
    );
}

#[test]
fn twinkle_toes_previent_les_2_prochains_degats_ce_tour() {
    let mut t = twinkle_partie(&[("Essai Appel", B)]);
    let w = t.equipement(P0, "Twinkle Toes");
    jouer(&mut t, P0, "Essai Appel", B);
    resoudre(&mut t);
    activer_twinkle(&mut t, w);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 3, false);
    assert_eq!(t.vie(P0), 19, "3 - 2 prévenus");
}

#[test]
fn twinkle_toes_ne_previent_que_2_degats() {
    let mut t = twinkle_partie(&[("Essai Appel", B)]);
    let w = t.equipement(P0, "Twinkle Toes");
    jouer(&mut t, P0, "Essai Appel", B);
    resoudre(&mut t);
    activer_twinkle(&mut t, w);
    resoudre(&mut t);
    t.g.deal_damage(None, P1, P0, 5, false);
    assert_eq!(t.vie(P0), 17, "5 - 2 prévenus : le reste passe");
}

#[test]
fn twinkle_toes_prevention_ne_dure_que_ce_tour() {
    let mut t = twinkle_partie(&[("Essai Appel", B)]);
    let w = t.equipement(P0, "Twinkle Toes");
    jouer(&mut t, P0, "Essai Appel", B);
    resoudre(&mut t);
    activer_twinkle(&mut t, w);
    resoudre(&mut t);
    t.finir_le_tour();
    t.g.deal_damage(None, P1, P0, 3, false);
    assert_eq!(t.vie(P0), 17, "la prévention a expiré à la fin du tour");
}

#[test]
fn twinkle_toes_sans_instant_joue_ce_tour_ne_peut_pas_s_activer() {
    let t = twinkle_partie(&[]);
    let w = t.equipement(P0, "Twinkle Toes");
    assert!(
        !t.peut(Choice::Activate(w, 0)),
        "aucun instant joué ce tour"
    );
}

#[test]
fn twinkle_toes_instant_joue_ce_tour_permet_l_activation() {
    let mut t = twinkle_partie(&[("Essai Appel", B)]);
    let w = t.equipement(P0, "Twinkle Toes");
    assert!(!t.peut(Choice::Activate(w, 0)), "avant de jouer l'instant");
    jouer(&mut t, P0, "Essai Appel", B);
    resoudre(&mut t);
    assert!(t.peut(Choice::Activate(w, 0)), "un instant joué ce tour");
}

#[test]
fn twinkle_toes_instant_du_tour_precedent_ne_compte_pas() {
    let mut t = twinkle_partie(&[("Essai Appel", B)]);
    let w = t.equipement(P0, "Twinkle Toes");
    jouer(&mut t, P0, "Essai Appel", B);
    resoudre(&mut t);
    t.finir_le_tour();
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P0, "retour au tour de P0");
    assert!(
        !t.peut(Choice::Activate(w, 0)),
        "l'instant date du tour précédent"
    );
}

// ---------------------------------------------------------------------------
// Star Fall
// ---------------------------------------------------------------------------

const RAMPART_STAR_FALL: (&str, Option<Color>) = ("Essai Rempart", B);

fn star_fall_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &["Star Fall"], &[])
}

/// Joue une carte de la main en pitchant (premières options) pour payer son coût.
fn jouer_en_pitchant(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    while t.g.pending_decision().is_some() && t.kind() == DecisionKind::Pitch {
        let o = t.options()[0];
        t.choisir(o);
    }
    c
}

/// Active l'arme (payée par les ressources ou en pitchant) et renvoie
/// l'attaque-proxy, une fois active.
fn attaquer_star_fall(t: &mut Partie, p: PlayerId) -> ObjectId {
    let w = t.equipement(p, "Star Fall");
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Activate(w, 0)));
    t.choisir(Choice::Activate(w, 0));
    for _ in 0..300 {
        if let Some(a) = t.g.active_attack() {
            return a;
        }
        match t.kind() {
            DecisionKind::Pitch => {
                let o = t.options()[0];
                t.choisir(o);
            }
            DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal => {
                t.choisir(Choice::Pass)
            }
            k => panic!("décision {k:?} inattendue ; options {:?}", t.options()),
        }
    }
    panic!("attaque de Star Fall jamais active");
}

#[test]
fn star_fall_attaque_pour_1_ressource_une_fois_par_tour() {
    let mut t = star_fall_partie(&[]);
    let w = t.equipement(P0, "Star Fall");
    let proxy = attaquer_star_fall(&mut t, P0);
    assert_eq!(t.g.power(proxy), 1, "sans carte Lightning jouée ce tour");
    assert!(!t.peut(Choice::Activate(w, 0)), "une fois par tour");
}

#[test]
fn star_fall_une_carte_lightning_jouee_ce_tour_donne_plus_1_puissance() {
    // Sizzle (rouge, Lightning) : son propre +3 s'ajoute à l'attaque ; on
    // vérifie donc d'abord la puissance de l'arme seule (1 + 1).
    let mut t = star_fall_partie(&[("Sizzle", R)]);
    let w = t.equipement(P0, "Star Fall");
    jouer(&mut t, P0, "Sizzle", R);
    resoudre(&mut t);
    assert_eq!(t.g.power(w), 2, "1 de base + 1 : une carte Lightning jouée");
}

#[test]
fn star_fall_une_carte_lightning_jouee_ce_tour_donne_go_again() {
    let mut t = star_fall_partie(&[("Sizzle", R)]);
    jouer(&mut t, P0, "Sizzle", R);
    resoudre(&mut t);
    let proxy = attaquer_star_fall(&mut t, P0);
    assert!(t.g.keywords(proxy).has(Keywords::GO_AGAIN));
}

#[test]
fn star_fall_carte_non_lightning_jouee_ne_compte_pas() {
    let mut t = star_fall_partie(&[
        ("Harvest Season", R),
        RAMPART_STAR_FALL,
        RAMPART_STAR_FALL,
        RAMPART_STAR_FALL,
    ]);
    let w = t.equipement(P0, "Star Fall");
    jouer_en_pitchant(&mut t, P0, "Harvest Season", R);
    resoudre(&mut t);
    assert_eq!(
        t.g.power(w),
        1,
        "Harvest Season n'est pas une carte Lightning"
    );
    let proxy = attaquer_star_fall(&mut t, P0);
    assert!(!t.g.keywords(proxy).has(Keywords::GO_AGAIN));
}

#[test]
fn star_fall_carte_lightning_du_tour_precedent_ne_compte_pas() {
    let mut t = star_fall_partie(&[("Sizzle", R)]);
    let w = t.equipement(P0, "Star Fall");
    jouer(&mut t, P0, "Sizzle", R);
    resoudre(&mut t);
    t.finir_le_tour();
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P0, "retour au tour de P0");
    assert_eq!(t.g.power(w), 1, "Sizzle date du tour précédent");
}

// ---------------------------------------------------------------------------
// Helm of Might and Magic (Runeblade Equipment - Head, déf. 1, sans couleur)
// ---------------------------------------------------------------------------

const HELM: &str = "Helm of Might and Magic";
const RAMPART_HELM: (&str, Option<Color>) = ("Essai Rempart", B);

/// Paie le coût en cours en pitchant des Essai Rempart.
fn payer_rempart(t: &mut Partie) {
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

/// Joue la carte de `p` en payant avec des Essai Rempart.
fn jouer_payant(t: &mut Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    let c = t.en_main(p, nom, couleur);
    passer_jusqua_que(t, |t| t.joueur() == p && t.peut(Choice::Play(c)));
    t.choisir(Choice::Play(c));
    payer_rempart(t);
    c
}

/// Partie où P0 (héros) a le Helm équipé et P1 a `main1` en main.
fn partie_helm_contre(main1: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], RIEN, RIEN, main1, RIEN, &[HELM], &[])
}

/// P1 attaque P0 avec Essai Frappe ; renvoie l'attaque active.
fn helm_attaque_par_p1(t: &mut Partie) -> ObjectId {
    t.finir_le_tour();
    assert_eq!(t.g.turn_player, P1);
    let a = jouer_payant(t, P1, "Essai Frappe", R);
    passer_jusqua_que(t, |t| t.g.active_attack() == Some(a));
    a
}

fn helm_defend(t: &mut Partie, cartes: &[ObjectId]) {
    passer_jusqua_que(t, |t| t.kind() == DecisionKind::Defend && t.joueur() == P0);
    for &x in cartes {
        t.choisir(Choice::Defend(x));
    }
    if t.kind() == DecisionKind::Defend {
        t.choisir(Choice::Pass);
    }
}

#[test]
fn helm_of_might_and_magic_defense_1_et_sans_couleur() {
    let t = partie_helm_contre(&[]);
    let h = t.equipement(P0, HELM);
    assert_eq!(t.g.defense(h), 1);
    assert_eq!(t.g.def(h).color, None);
}

#[test]
fn helm_of_might_and_magic_blade_break_detruit_le_heaume_qui_a_defendu() {
    let mut t = partie_helm_contre(&[("Essai Frappe", R), RAMPART_HELM]);
    let h = t.equipement(P0, HELM);
    assert!(t.g.keywords(h).has(Keywords::BLADE_BREAK));
    helm_attaque_par_p1(&mut t);
    helm_defend(&mut t, &[h]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.obj(h).zone, Zone::Graveyard, "heaume détruit");
}

#[test]
fn helm_of_might_and_magic_blade_break_ignore_le_heaume_qui_n_a_pas_defendu() {
    let mut t = partie_helm_contre(&[("Essai Frappe", R), RAMPART_HELM]);
    let h = t.equipement(P0, HELM);
    helm_attaque_par_p1(&mut t);
    helm_defend(&mut t, &[]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_ne!(
        t.g.obj(h).zone,
        Zone::Graveyard,
        "heaume non défenseur gardé"
    );
}

#[test]
fn helm_of_might_and_magic_spellvoid_1_previent_1_degat_arcane_en_se_detruisant() {
    let mut t = partie_helm_contre(&[]);
    let h = t.equipement(P0, HELM);
    assert_eq!(t.g.prevention_values(h).2, 1);
    t.g.deal_damage(None, P1, P0, 3, true);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
    assert_eq!(t.joueur(), P0);
    let choix = Choice::Prevent(fab_engine::PreventRef::Spellvoid(h));
    assert!(t.peut(choix));
    t.choisir(choix);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.g.tasks.is_empty());
    assert_eq!(t.vie(P0), 18, "3 - 1 prévenu");
    assert_eq!(
        t.g.obj(h).zone,
        Zone::Graveyard,
        "heaume détruit pour prévenir"
    );
}

#[test]
fn helm_of_might_and_magic_spellvoid_ne_s_applique_pas_aux_degats_non_arcane() {
    let mut t = partie_helm_contre(&[]);
    let h = t.equipement(P0, HELM);
    t.g.deal_damage(None, P1, P0, 3, false);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.g.tasks.is_empty());
    assert_eq!(t.vie(P0), 17, "aucune prévention");
    assert_ne!(t.g.obj(h).zone, Zone::Graveyard, "heaume intact");
}

#[test]
fn helm_of_might_and_magic_spellvoid_ne_previent_qu_une_seule_fois() {
    let mut t = partie_helm_contre(&[]);
    let h = t.equipement(P0, HELM);
    t.g.deal_damage(None, P1, P0, 3, true);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Prevention);
    t.choisir(Choice::Prevent(fab_engine::PreventRef::Spellvoid(h)));
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.g.tasks.is_empty());
    assert_eq!(t.vie(P0), 18);
    t.g.deal_damage(None, P1, P0, 3, true);
    passer_jusqua_que(&mut t, |t| t.g.stack.is_empty() && t.g.tasks.is_empty());
    assert_eq!(t.vie(P0), 15, "le heaume détruit ne prévient plus");
}

// Sigil of Voltaris (bleu) — Lightning Runeblade Instant - Aura, coût 0.
// « When this enters or leaves the arena, deal 1 arcane damage to target hero.
//   At the beginning of your action phase, destroy this. »

#[test]
fn sigil_of_voltaris_en_entrant_inflige_1_degat_arcanique_au_heros_cible() {
    let mut t = partie_cloud(&[("Sigil of Voltaris", B)]);
    let s = jouer(&mut t, P0, "Sigil of Voltaris", B);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert!(arcanique_1_a(&t, P1), "1 dégât arcanique à P1");
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.obj(s).zone, Zone::Permanent, "reste dans l'arène");
}

#[test]
fn sigil_of_voltaris_au_debut_de_votre_phase_d_action_est_detruit() {
    let mut t = partie_cloud(&[("Sigil of Voltaris", B)]);
    let s = jouer(&mut t, P0, "Sigil of Voltaris", B);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert_eq!(t.g.obj(s).zone, Zone::Permanent);
    while t.g.turn < 3 {
        t.finir_le_tour();
    }
    passer_jusqua_que(&mut t, |t| t.g.obj(s).zone == Zone::Graveyard);
    assert_eq!(t.g.obj(s).zone, Zone::Graveyard, "détruite");
}

#[test]
fn sigil_of_voltaris_en_quittant_l_arene_inflige_1_degat_arcanique_au_heros_cible() {
    let mut t = partie_cloud(&[("Sigil of Voltaris", B)]);
    let s = jouer(&mut t, P0, "Sigil of Voltaris", B);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 19);
    while t.g.turn < 3 {
        t.finir_le_tour();
    }
    passer_jusqua_que(&mut t, |t| t.g.obj(s).zone == Zone::Graveyard);
    cibler_heros(&mut t, P1);
    resoudre(&mut t);
    assert_eq!(t.vie(P1), 18, "second dégât en quittant l'arène");
}

#[test]
fn sigil_of_voltaris_la_cible_peut_etre_son_propre_heros() {
    let mut t = partie_cloud(&[("Sigil of Voltaris", B)]);
    jouer(&mut t, P0, "Sigil of Voltaris", B);
    let avant = t.vie(P0);
    cibler_heros(&mut t, P0);
    resoudre(&mut t);
    assert_eq!(t.vie(P0), avant - 1, "cas limite : héros quelconque");
}

// ---------------------------------------------------------------------------
// Flash of Brilliance (Lightning Equipment - Head, déf. 1, sans couleur)
// ---------------------------------------------------------------------------

const FLASH: &str = "Flash of Brilliance";
const SIGIL: &str = "Sigil of Voltaris";
const CARTE_LIGHTNING: (&str, Option<Color>) = ("Cloud Cover", R);

/// P0 a `main0` en main et `FLASH` en arène ; P1 attaque avec Essai Frappe.
fn flash_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        [HEROS, HEROS],
        main0,
        RIEN,
        &[("Essai Frappe", R), RAMPART_HELM],
        RIEN,
        &[FLASH],
        &[],
    )
}

/// Pose l'aura Sigil of Voltaris de P0 sur l'arène (il joue la carte).
fn poser_sigil(t: &mut Partie) -> ObjectId {
    let s = jouer(t, P0, SIGIL, B);
    cibler_heros(t, P1);
    resoudre(t);
    s
}

/// Vrai quand la question « défausser / renvoyer en main » est posée à P0.
fn question_flash(t: &Partie) -> bool {
    let o = t.options();
    o.contains(&Choice::Pass) && o.iter().any(|c| matches!(c, Choice::Object(_)))
}

/// P1 attaque, P0 défend avec `flash` ; s'arrête à la question du déclencheur.
fn flash_defend_jusqua_question(t: &mut Partie, flash: ObjectId) {
    helm_attaque_par_p1(t);
    helm_defend(t, &[flash]);
    passer_jusqua_que(t, |t| question_flash(t) || !t.g.combat.open);
}

/// Termine la défense ; le « leaves the arena » du Sigil demande sa cible.
fn finir_flash(t: &mut Partie) {
    passer_jusqua_que(t, |t| !t.g.combat.open || t.kind() == DecisionKind::Target);
    if t.kind() == DecisionKind::Target {
        t.choisir(Choice::Target(Target::Player(P1)));
    }
    passer_jusqua_que(t, |t| !t.g.combat.open);
}

#[test]
fn flash_of_brilliance_defense_1_sans_couleur_et_lightning() {
    let t = flash_partie(&[]);
    let f = t.equipement(P0, FLASH);
    assert_eq!(t.g.defense(f), 1);
    assert_eq!(t.g.def(f).color, None);
    assert!(t.g.def(f).supertypes.has(Supertypes::LIGHTNING));
}

#[test]
fn flash_of_brilliance_blade_break() {
    let t = flash_partie(&[]);
    let f = t.equipement(P0, FLASH);
    assert!(t.g.keywords(f).has(Keywords::BLADE_BREAK));
}

#[test]
fn flash_of_brilliance_quand_il_defend_propose_de_defausser_une_carte_lightning() {
    let mut t = flash_partie(&[CARTE_LIGHTNING]);
    let f = t.equipement(P0, FLASH);
    let cloud = t.en_main(P0, "Cloud Cover", R);
    flash_defend_jusqua_question(&mut t, f);
    assert!(question_flash(&t), "question de défausse attendue");
    assert!(t.options().contains(&Choice::Object(cloud)));
}

#[test]
fn flash_of_brilliance_ne_pose_pas_de_question_quand_il_ne_defend_pas() {
    let mut t = flash_partie(&[CARTE_LIGHTNING]);
    helm_attaque_par_p1(&mut t);
    helm_defend(&mut t, &[]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
}

#[test]
fn flash_of_brilliance_may_passer_ne_defausse_rien_ni_ne_renvoie_d_aura() {
    let mut t = flash_partie(&[CARTE_LIGHTNING, (SIGIL, B)]);
    let f = t.equipement(P0, FLASH);
    let cloud = t.en_main(P0, "Cloud Cover", R);
    let s = poser_sigil(&mut t);
    flash_defend_jusqua_question(&mut t, f);
    t.choisir(Choice::Pass);
    finir_flash(&mut t);
    assert_eq!(t.g.obj(cloud).zone, Zone::Hand, "carte gardée en main");
    assert_eq!(t.g.obj(s).zone, Zone::Permanent, "aura restée en arène");
}

#[test]
fn flash_of_brilliance_defausse_la_carte_lightning_choisie() {
    let mut t = flash_partie(&[CARTE_LIGHTNING]);
    let f = t.equipement(P0, FLASH);
    let cloud = t.en_main(P0, "Cloud Cover", R);
    flash_defend_jusqua_question(&mut t, f);
    t.choisir(Choice::Object(cloud));
    finir_flash(&mut t);
    assert_eq!(
        t.g.obj(cloud).zone,
        Zone::Graveyard,
        "carte Lightning défaussée"
    );
}

#[test]
fn flash_of_brilliance_ne_propose_pas_une_carte_qui_n_est_pas_lightning() {
    let mut t = flash_partie(&[CARTE_LIGHTNING, RAMPART_HELM]);
    let f = t.equipement(P0, FLASH);
    let cloud = t.en_main(P0, "Cloud Cover", R);
    let rempart = t.en_main(P0, "Essai Rempart", B);
    flash_defend_jusqua_question(&mut t, f);
    let o = t.options();
    assert!(o.contains(&Choice::Object(cloud)));
    assert!(!o.contains(&Choice::Object(rempart)), "{o:?}");
}

#[test]
fn flash_of_brilliance_si_defausse_renvoie_l_aura_controlee_en_main_de_son_proprietaire() {
    let mut t = flash_partie(&[CARTE_LIGHTNING, (SIGIL, B)]);
    let f = t.equipement(P0, FLASH);
    let cloud = t.en_main(P0, "Cloud Cover", R);
    let s = poser_sigil(&mut t);
    flash_defend_jusqua_question(&mut t, f);
    t.choisir(Choice::Object(cloud));
    finir_flash(&mut t);
    assert_eq!(t.g.obj(cloud).zone, Zone::Graveyard, "carte défaussée");
    assert_eq!(t.g.obj(s).zone, Zone::Hand, "aura renvoyée en main");
    assert_eq!(t.g.obj(s).owner, P0);
}

#[test]
fn flash_of_brilliance_sans_aura_controlee_defausse_sans_rien_renvoyer() {
    let mut t = flash_partie(&[CARTE_LIGHTNING]);
    let f = t.equipement(P0, FLASH);
    let cloud = t.en_main(P0, "Cloud Cover", R);
    flash_defend_jusqua_question(&mut t, f);
    t.choisir(Choice::Object(cloud));
    finir_flash(&mut t);
    assert_eq!(t.g.obj(cloud).zone, Zone::Graveyard, "carte défaussée");
}

#[test]
fn flash_of_brilliance_sans_carte_lightning_en_main_ne_pose_pas_de_question() {
    let mut t = flash_partie(&[RAMPART_HELM]);
    let f = t.equipement(P0, FLASH);
    helm_attaque_par_p1(&mut t);
    helm_defend(&mut t, &[f]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
}

// ---------------------------------------------------------------- Beckoning Haunt

const BECKONING: &str = "Beckoning Haunt";
const RAMPART_BECKONING: (&str, Option<Color>) = ("Essai Rempart", B);

/// Partie où P0 (héros) a le Beckoning Haunt équipé et P1 a `main1` en main.
fn partie_beckoning_contre(main1: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], RIEN, RIEN, main1, RIEN, &[BECKONING], &[])
}

#[test]
fn beckoning_haunt_defense_2_sans_couleur_arms_runeblade_guardwell() {
    let t = partie_beckoning_contre(&[]);
    let b = t.equipement(P0, BECKONING);
    assert_eq!(t.g.defense(b), 2);
    assert_eq!(t.g.def(b).color, None);
    assert_eq!(t.g.obj(b).zone, Zone::Arms);
    assert!(t.g.def(b).supertypes.has(Supertypes::RUNEBLADE));
    assert!(t.g.keywords(b).has(Keywords::GUARDWELL));
}

#[test]
fn beckoning_haunt_guardwell_retire_des_compteurs_egaux_a_sa_defense_quand_il_defend() {
    let mut t = partie_beckoning_contre(&[("Essai Frappe", R), RAMPART_BECKONING]);
    let b = t.equipement(P0, BECKONING);
    helm_attaque_par_p1(&mut t);
    helm_defend(&mut t, &[b]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.defense(b), 0, "-2 compteurs : défense 0");
    assert_eq!(t.g.obj(b).zone, Zone::Arms, "guardwell ne détruit pas");
}

#[test]
fn beckoning_haunt_guardwell_ne_retire_rien_s_il_ne_defend_pas() {
    let mut t = partie_beckoning_contre(&[("Essai Frappe", R), RAMPART_BECKONING]);
    let b = t.equipement(P0, BECKONING);
    helm_attaque_par_p1(&mut t);
    helm_defend(&mut t, &[]);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    assert_eq!(t.g.defense(b), 2, "il n'a pas défendu : rien retiré");
}

// ---- Quick Succession (bleu, rouge, jaune) ----

fn quick_succession_partie(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([HEROS, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

#[test]
fn quick_succession_go_again_rend_un_point_d_action() {
    let mut t = quick_succession_partie(&[("Quick Succession", B)]);
    jouer(&mut t, P0, "Quick Succession", B);
    resoudre(&mut t);
    assert_eq!(t.g.player(P0).action_points, 1, "go again");
}

#[test]
fn quick_succession_donne_go_again_a_la_prochaine_attaque_runeblade() {
    let mut t = quick_succession_partie(&[("Quick Succession", B), ("Colors of Aria", R)]);
    jouer(&mut t, P0, "Quick Succession", B);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Colors of Aria", R);
    jusqua_active(&mut t, a);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn quick_succession_donne_go_again_sans_puissance_rouge() {
    let mut t = quick_succession_partie(&[("Quick Succession", R), ("Colors of Aria", R)]);
    jouer(&mut t, P0, "Quick Succession", R);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Colors of Aria", R);
    jusqua_active(&mut t, a);
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
    assert_eq!(
        t.g.power(a),
        7,
        "rouge : pas de +1 dans le code (voir TODO)"
    );
}

#[test]
fn quick_succession_ne_donne_pas_go_again_a_une_attaque_sans_runeblade_ni_lightning() {
    let mut t = quick_succession_partie(&[("Quick Succession", B), ("Essai Frappe", R)]);
    jouer(&mut t, P0, "Quick Succession", B);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Essai Frappe", R);
    jusqua_active(&mut t, a);
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn quick_succession_bleu_plus_1_a_la_prochaine_attaque_avec_go_again() {
    let mut t = quick_succession_partie(&[("Quick Succession", B), ("Colors of Aria", R)]);
    jouer(&mut t, P0, "Quick Succession", B);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Colors of Aria", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 7 + 1, "bleu : +1 car l'attaque a go again");
}

#[test]
fn quick_succession_bleu_pas_de_plus_1_sans_go_again() {
    let mut t = quick_succession_partie(&[("Quick Succession", B), ("Essai Frappe", R)]);
    jouer(&mut t, P0, "Quick Succession", B);
    resoudre(&mut t);
    let a = jouer(&mut t, P0, "Essai Frappe", R);
    jusqua_active(&mut t, a);
    assert_eq!(t.g.power(a), 4, "pas de go again : pas de +1");
}

// ---------------------------------------------------------------------------
// Voltbound Duality (Lightning Runeblade Action - Attack, bleu / rouge / jaune)
// ---------------------------------------------------------------------------

const REMPART: (&str, Option<Color>) = ("Essai Rempart", B);

/// Partie où P0 a Voltbound Duality (couleur `c`) et trois Remparts en main
/// (de quoi payer le {r}).
fn partie_voltbound(c: Option<Color>) -> Partie {
    partie_vraie(
        [HEROS, HEROS],
        &[("Voltbound Duality", c), REMPART, REMPART, REMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    )
}

fn en_main_voltbound(t: &Partie, couleur: Option<Color>) -> ObjectId {
    t.en_main(P0, "Voltbound Duality", couleur)
}

/// Active Voltbound Duality depuis la main : cible le héros adverse, paie
/// {r} en défaussant un Rempart, puis résout la capacité.
fn activer_voltbound(t: &mut Partie, c: ObjectId) {
    passer_jusqua_que(t, |t| t.joueur() == P0 && t.peut(Choice::Activate(c, 0)));
    t.choisir(Choice::Activate(c, 0));
    for _ in 0..20 {
        if t.g.is_over() || t.g.pending_decision().is_none() {
            break;
        }
        match t.kind() {
            DecisionKind::Pitch => {
                let opts = t.options();
                let rempart = opts
                    .iter()
                    .copied()
                    .find(|o| matches!(o, Choice::Pitch(x) if t.g.def(*x).name == "Essai Rempart"))
                    .unwrap_or(opts[0]);
                t.choisir(rempart);
            }
            DecisionKind::Target => t.choisir(Choice::Target(Target::Player(P1))),
            _ => break,
        }
    }
    resoudre(t);
}

fn jetons_lightning_flow(t: &Partie, p: PlayerId) -> usize {
    t.g.permanents
        .iter()
        .copied()
        .filter(|&o| t.g.obj(o).controller == p && t.g.def(o).name == "Lightning Flow")
        .count()
}

#[test]
fn voltbound_duality_instant_se_defausse_en_payant_r() {
    let mut t = partie_voltbound(B);
    let c = en_main_voltbound(&t, B);
    activer_voltbound(&mut t, c);
    assert_ne!(t.g.obj(c).zone, Zone::Hand, "la carte est défaussée");
}

#[test]
fn voltbound_duality_inflige_1_degat_arcanique_au_heros_cible() {
    let mut t = partie_voltbound(R);
    let c = en_main_voltbound(&t, R);
    activer_voltbound(&mut t, c);
    assert_eq!(t.vie(P1), 19, "1 dégât arcane au héros adverse");
    assert_eq!(t.vie(P0), 20, "le héros de P0 n'est pas touché");
}

#[test]
fn voltbound_duality_cree_un_jeton_lightning_flow() {
    let mut t = partie_voltbound(Y);
    let c = en_main_voltbound(&t, Y);
    activer_voltbound(&mut t, c);
    assert_eq!(
        jetons_lightning_flow(&t, P0),
        1,
        "un jeton pour le contrôleur"
    );
    assert_eq!(jetons_lightning_flow(&t, P1), 0);
}

#[test]
fn voltbound_duality_puissance_selon_la_couleur() {
    for (couleur, attendue) in [(B, 2), (R, 4), (Y, 3)] {
        let t = partie_voltbound(couleur);
        let c = en_main_voltbound(&t, couleur);
        assert_eq!(t.g.def(c).power, Some(attendue));
        assert_eq!(t.g.def(c).defense, Some(3));
    }
}

#[test]
fn lightning_flow_est_un_jeton_aura_elementaire_sans_texte() {
    let t = partie_voltbound(B);
    let id = t.g.db.find_name("Lightning Flow").expect("jeton codé");
    let d = t.g.db.get(id);
    assert!(d.types.has(fab_engine::card::Types::TOKEN));
    assert!(d.subtypes.has(fab_engine::card::Subtypes::AURA));
    assert!(d.supertypes.has(Supertypes::ELEMENTAL));
    assert!(d.triggers.is_empty() && d.activated.is_empty());
}

#[test]
fn voltbound_duality_pas_activable_une_fois_defaussee() {
    let mut t = partie_voltbound(B);
    let c = en_main_voltbound(&t, B);
    activer_voltbound(&mut t, c);
    assert!(
        !t.peut(Choice::Activate(c, 0)),
        "plus en main, plus activable"
    );
    assert_eq!(t.vie(P1), 19, "un seul dégât");
}

// ---------------------------------------------------------------------------
// Aurora et Embodiment of Lightning
// ---------------------------------------------------------------------------

const AURORA: &str = "Aurora";

fn partie_aurora(main0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie([AURORA, HEROS], main0, RIEN, RIEN, RIEN, &[], &[])
}

/// Paie les coûts en pitchant des Essai Rempart.
fn payer_aurora(t: &mut Partie) {
    while t.kind() == DecisionKind::Pitch {
        let opts = t.options();
        let c = opts
            .iter()
            .copied()
            .find(|c| matches!(c, Choice::Pitch(o) if t.g.def(*o).name == "Essai Rempart"))
            .unwrap_or(opts[0]);
        t.choisir(c);
    }
}

/// P0 active Aurora (ce qui paie {r}{r} en pitchant des Rempart) et résout.
fn activer_aurora(t: &mut Partie) {
    let h = t.g.player(P0).hero;
    passer_jusqua_que(t, |t| t.joueur() == P0 && t.peut(Choice::Activate(h, 0)));
    t.choisir(Choice::Activate(h, 0));
    payer_aurora(t);
    resoudre(t);
}

/// Crée un jeton Embodiment of Lightning pour `p`.
fn creer_embodiment(t: &mut Partie, p: PlayerId) {
    let c =
        t.g.db
            .find_name("Embodiment of Lightning")
            .expect("jeton codé");
    t.g.create_tokens(p, c, 1);
}

fn jetons_embodiment(t: &Partie, p: PlayerId) -> usize {
    t.g.permanents
        .iter()
        .copied()
        .filter(|&o| t.g.obj(o).controller == p && t.g.def(o).name == "Embodiment of Lightning")
        .count()
}

#[test]
fn aurora_est_un_heros_jeune_elementaire_runeblade_de_20_pv_et_4_intellect() {
    let t = partie_aurora(&[]);
    let d = t.g.def(t.g.player(P0).hero);
    assert_eq!(d.name, AURORA);
    assert_eq!(d.life, Some(20));
    assert_eq!(d.intellect, Some(4));
    assert!(d.subtypes.has(fab_engine::card::Subtypes::YOUNG));
    assert!(d.supertypes.has(Supertypes::ELEMENTAL));
    assert!(d.supertypes.has(Supertypes::RUNEBLADE));
}

#[test]
fn aurora_cree_un_jeton_embodiment_of_lightning_apres_une_carte_lightning() {
    let mut t = partie_aurora(&[("Cloud Cover", B), REMPART, REMPART, REMPART]);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    activer_aurora(&mut t);
    assert_eq!(jetons_embodiment(&t, P0), 1, "un jeton pour Aurora");
    assert_eq!(jetons_embodiment(&t, P1), 0);
}

#[test]
fn aurora_pas_activable_sans_carte_lightning_jouee_ce_tour() {
    let mut t = partie_aurora(&[REMPART, REMPART, REMPART, REMPART]);
    let h = t.g.player(P0).hero;
    passer_jusqua_que(&mut t, |t| {
        t.joueur() == P0 && t.kind() == DecisionKind::Priority
    });
    assert!(
        !t.peut(Choice::Activate(h, 0)),
        "aucune carte Lightning jouée"
    );
    assert_eq!(jetons_embodiment(&t, P0), 0);
}

#[test]
fn aurora_une_fois_par_tour() {
    let mut t = partie_aurora(&[("Cloud Cover", B), REMPART, REMPART, REMPART, REMPART]);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    activer_aurora(&mut t);
    let h = t.g.player(P0).hero;
    assert!(!t.peut(Choice::Activate(h, 0)), "une seule fois par tour");
    assert_eq!(jetons_embodiment(&t, P0), 1);
}

#[test]
fn embodiment_of_lightning_detruit_et_donne_go_again_a_l_attaque_jouee() {
    let mut t = partie_vraie(
        [HEROS, HEROS],
        &[("Essai Frappe", R), REMPART, REMPART, REMPART],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    creer_embodiment(&mut t, P0);
    let a = jouer(&mut t, P0, "Essai Frappe", R);
    jusqua_active(&mut t, a);
    assert_eq!(jetons_embodiment(&t, P0), 0, "détruit par l'attaque jouée");
    assert!(t.g.keywords(a).has(Keywords::GO_AGAIN));
}

#[test]
fn embodiment_of_lightning_reste_si_la_carte_jouee_n_attaque_pas() {
    let mut t = partie_aurora(&[("Cloud Cover", B), REMPART, REMPART, REMPART]);
    creer_embodiment(&mut t, P0);
    jouer(&mut t, P0, "Cloud Cover", B);
    resoudre(&mut t);
    assert_eq!(
        jetons_embodiment(&t, P0),
        1,
        "carte non attaque : le jeton reste"
    );
}

#[test]
fn embodiment_of_lightning_ne_se_declenche_pas_pour_l_attaque_adverse() {
    let mut t = partie_vraie(
        [HEROS, HEROS],
        &[("Essai Frappe", R), REMPART, REMPART, REMPART],
        RIEN,
        &[REMPART, REMPART, REMPART, REMPART],
        RIEN,
        &[],
        &[],
    );
    creer_embodiment(&mut t, P1);
    let a = jouer(&mut t, P0, "Essai Frappe", R);
    jusqua_active(&mut t, a);
    assert_eq!(jetons_embodiment(&t, P1), 1, "le jeton de P1 ne bouge pas");
    assert!(!t.g.keywords(a).has(Keywords::GO_AGAIN));
}
