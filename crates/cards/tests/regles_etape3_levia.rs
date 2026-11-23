//! Tests de règles ajoutées pour le deck Levia : blood debt (CR 8.3.11,
//! 8.3.11a), points d'action hors de la phase d'action (CR 1.13.2b),
//! permissions de jouer depuis la zone de bannissement (CR 5.1.3d), ordre
//! des déclenchements identiques (CR 6.6.6), qui a banni une carte.

mod commun;
use commun::*;
use fab_engine::card::Color;
use fab_engine::{Choice, DecisionKind, Event, ObjectId, PlayerId, Zone};

const RAMPART: (&str, Option<Color>) = ("Essai Rempart", B);
const ETINCELLE: (&str, Option<Color>) = ("Essai Étincelle", R);
const HEROS: &str = "Essai Héros";
const BLASMOPHET: &str = "Blasmophet, the Insatiable Hunger";
const GATE: &str = "Gate to i'Arathael";

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

fn du_deck(t: &Partie, p: PlayerId, nom: &str, couleur: Option<Color>) -> ObjectId {
    *t.g.player(p)
        .deck
        .iter()
        .find(|&&c| t.g.def(c).name == nom && t.g.def(c).color == couleur)
        .unwrap_or_else(|| panic!("{nom} {couleur:?} absente du deck de {p:?}"))
}

/// L'adversaire (P1) bannit face visible ou cachée une carte du deck de P0.
fn bannir(t: &mut Partie, nom: &str, couleur: Option<Color>, face_cachee: bool) -> ObjectId {
    let c = du_deck(t, P0, nom, couleur);
    t.g.banish(P1, c, face_cachee);
    t.g.refresh_priority();
    c
}

fn creer(t: &mut Partie, p: PlayerId, nom: &str) -> ObjectId {
    let c = t.g.db.find_name(nom).expect("token codé");
    t.g.create_tokens(p, c, 1);
    t.g.refresh_priority();
    *t.g.last_created.first().unwrap()
}

fn heros(main0: &[(&str, Option<Color>)], suite0: &[(&str, Option<Color>)]) -> Partie {
    partie_vraie(
        [HEROS, HEROS],
        main0,
        suite0,
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        RIEN,
        &[],
        &[],
    )
}

#[test]
fn cr_8_3_11_blood_debt_perte_de_1_au_debut_de_la_phase_de_fin_du_proprietaire() {
    let mut t = heros(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
    );
    bannir(&mut t, "Feeding Frenzy", B, false);
    t.finir_le_tour(); // tour 1 : phase de fin de P0
    assert_eq!(t.vie(P0), 19);
    t.finir_le_tour(); // tour 2 : phase de fin de P1, rien pour P0
    assert_eq!(t.vie(P0), 19);
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn cr_8_3_11a_une_carte_bannie_face_cachee_ne_declenche_pas_blood_debt() {
    let mut t = heros(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B)],
    );
    bannir(&mut t, "Feeding Frenzy", B, true);
    t.finir_le_tour();
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn cr_8_3_11_blood_debt_ne_fonctionne_que_dans_la_zone_de_bannissement() {
    let mut t = heros(
        &[ETINCELLE, ("Feeding Frenzy", B), RAMPART, RAMPART],
        &[("Feeding Frenzy", Y)],
    );
    let c = du_deck(&t, P0, "Feeding Frenzy", Y);
    t.g.put_in_graveyard(c);
    t.g.refresh_priority();
    t.finir_le_tour();
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn cr_8_3_11_plusieurs_blood_debt_et_une_autre_capacite_un_seul_choix_d_ordre() {
    // Trois cartes avec blood debt et Blasmophet : le joueur choisit l'ordre
    // entre « blood debt » (les trois ensemble) et Blasmophet.
    let mut t = partie_vraie(
        ["Levia", HEROS],
        &[ETINCELLE, ("Graveling Growl", R), RAMPART, RAMPART],
        &[
            ("Feeding Frenzy", B),
            ("Feeding Frenzy", B),
            ("Feeding Frenzy", Y),
        ],
        RIEN,
        RIEN,
        &[],
        &[],
    );
    for (n, c) in [
        ("Feeding Frenzy", B),
        ("Feeding Frenzy", B),
        ("Feeding Frenzy", Y),
    ] {
        bannir(&mut t, n, c, false);
    }
    creer(&mut t, P0, BLASMOPHET);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::TriggerOrder);
    let opts = t.options();
    assert_eq!(opts.len(), 2);
    // Empiler d'abord blood debt : Blasmophet se résout en premier ; Levia
    // bannit Graveling Growl (7{p}) et ne perd rien de blood debt (CR 8.3.11 :
    // la perte est décidée à la résolution).
    let bd = opts
        .iter()
        .copied()
        .find(|o| matches!(o, Choice::Trigger(i) if matches!(t.g.pending_triggers[*i as usize].kind, fab_engine::state::LayerKind::Rule(_))))
        .unwrap();
    t.choisir(bd);
    assert_eq!(t.kind(), DecisionKind::Effect);
    let g = t.en_main(P0, "Graveling Growl", R);
    t.choisir(Choice::Object(g));
    passer_jusqua_que(&mut t, |t| t.g.turn == 2);
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn cr_1_13_2b_pas_de_point_d_action_hors_de_la_phase_d_action() {
    // Fin du tour de Levia : Blasmophet bannit une carte de 7{p}, Hooves se
    // déclenche ; le détruire ne donne pas de point d'action.
    let mut t = partie_vraie(
        ["Levia", HEROS],
        &[ETINCELLE, ("Graveling Growl", R), RAMPART, RAMPART],
        RIEN,
        RIEN,
        RIEN,
        &["Hooves of the Shadowbeast"],
        &[],
    );
    creer(&mut t, P0, BLASMOPHET);
    passer_jusqua_que(&mut t, |t| t.kind() == DecisionKind::Effect);
    let g = t.en_main(P0, "Graveling Growl", R);
    t.choisir(Choice::Object(g));
    assert_eq!(t.kind(), DecisionKind::Effect); // Hooves
    let avant = t.g.player(P0).action_points; // point non utilisé du tour
    t.choisir(Choice::Yes);
    assert_eq!(t.g.player(P0).action_points, avant);
    let hv =
        t.g.objects
            .iter()
            .position(|o| t.g.db.get(o.card).name == "Hooves of the Shadowbeast")
            .unwrap();
    assert_eq!(t.g.obj(ObjectId(hv as u32)).zone, Zone::Graveyard);
}

#[test]
fn cr_5_1_3d_une_permission_de_jouer_ce_tour_ne_consomme_pas_celle_de_blasmophet() {
    let mut t = partie_vraie(
        ["Levia", HEROS],
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feeding Frenzy", B), ("Feeding Frenzy", Y)],
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let f1 = bannir(&mut t, "Feeding Frenzy", B, false);
    let f2 = bannir(&mut t, "Feeding Frenzy", Y, false);
    creer(&mut t, P0, BLASMOPHET);
    let gate = creer(&mut t, P0, GATE);
    // Gate : permission pour f1.
    t.choisir(Choice::Activate(gate, 0));
    if t.kind() == DecisionKind::Target {
        let tf1 = fab_engine::Target::Object(t.g.obj_ref(f1));
        t.choisir(Choice::Target(tf1));
    }
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| {
        t.g.stack.is_empty() && t.kind() == DecisionKind::Priority
    });
    t.choisir(Choice::Play(f1));
    payer(&mut t);
    passer_jusqua_que(&mut t, |t| !t.g.combat.open);
    t.g.player_mut(P0).action_points = 1;
    t.g.player_mut(P0).resource_points = 2;
    t.g.refresh_priority();
    // La permission de Blasmophet n'a pas été utilisée.
    assert!(t.peut(Choice::Play(f2)));
}

#[test]
fn le_bannissement_note_le_joueur_qui_bannit_et_la_puissance_imprimee_hors_de_l_arene() {
    let mut t = heros(
        &[ETINCELLE, RAMPART, RAMPART, RAMPART],
        &[("Feasting Shadowbeast", Y)],
    );
    let c = bannir(&mut t, "Feasting Shadowbeast", Y, false);
    let e =
        t.g.turn_events()
            .iter()
            .find_map(|e| match *e {
                Event::Banished {
                    card,
                    by,
                    power,
                    from,
                    ..
                } if card == c => Some((by, power, from)),
                _ => None,
            })
            .unwrap();
    assert_eq!(e, (P1, 5, Zone::Deck));
    assert!(!t.g.you_banished_power6_this_turn(P0));
}
