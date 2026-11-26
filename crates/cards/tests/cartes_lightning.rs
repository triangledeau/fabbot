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
