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
