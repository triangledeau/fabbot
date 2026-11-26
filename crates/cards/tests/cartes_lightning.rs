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
