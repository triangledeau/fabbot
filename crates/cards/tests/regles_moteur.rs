//! Tests de la fin de l'étape 1 : préventions au choix (arcane barrier, ward,
//! spellvoid), effets de remplacement, attaques contre un allié ou un objet
//! avec spectra, perte des capacités, effets « jusqu'à leur prochain tour »,
//! déclenchements à la fermeture de la chaîne. Cartes **fictives**.

mod commun;
use commun::*;
use fab_cards::test_set::id;
use fab_engine::script::Target;
use fab_engine::state::CombatStep;
use fab_engine::{Choice, DecisionKind, Event, ObjectId, PlayerId, PreventRef, Zone};

const QUATRE_REMPARTS: &[(&str, Option<fab_engine::Color>)] = &[
    ("Essai Rempart", B),
    ("Essai Rempart", B),
    ("Essai Rempart", B),
    ("Essai Rempart", B),
];

/// P0 joue Essai Éclair (3 dégâts arcaniques) sur P1, en pitchant Essai Frappe
/// bleue, puis passe jusqu'à la décision attendue.
fn eclair_sur_p1(t: &mut Partie, attendue: DecisionKind) {
    let eclair = t.en_main(P0, "Essai Éclair", R);
    t.choisir(Choice::Play(eclair));
    t.choisir(Choice::Target(Target::Player(P1)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(attendue);
}

fn main_eclair() -> Vec<(&'static str, Option<fab_engine::Color>)> {
    vec![
        ("Essai Éclair", R),
        ("Essai Frappe", B),
        ("Essai Rempart", B),
        ("Essai Rempart", B),
    ]
}

fn token(t: &mut Partie, p: PlayerId, nom: &str) -> ObjectId {
    let c = id(&t.g.db, nom, None);
    let o = t.g.create_token(p, c).expect("token créé");
    t.g.refresh_priority();
    o
}

// ----------------------------------------------------------------------
// Arcane barrier, spellvoid, ward (CR 8.3.8, 8.3.15, 8.3.20, 6.5.5)
// ----------------------------------------------------------------------

#[test]
fn cr_8_3_8_arcane_barrier_payer_pour_prevenir() {
    let mut t = partie(
        &main_eclair(),
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Capuche"],
    );
    eclair_sur_p1(&mut t, DecisionKind::Prevention);
    let capuche = t.equipement(P1, "Essai Capuche");
    assert_eq!(t.joueur(), P1);
    assert_eq!(
        t.options(),
        vec![
            Choice::Prevent(PreventRef::ArcaneBarrier(capuche)),
            Choice::Pass
        ]
    );
    t.choisir(Choice::Prevent(PreventRef::ArcaneBarrier(capuche)));
    // « pay {r} » : P1 doit pitcher une carte.
    assert_eq!(t.kind(), DecisionKind::Pitch);
    assert_eq!(t.joueur(), P1);
    let rempart = t.en_main(P1, "Essai Rempart", B);
    t.choisir(Choice::Pitch(rempart));
    assert_eq!(t.vie(P1), 18); // 3 - 1 prévenu
    assert_eq!(t.g.player(P1).resource_points, 2);
}

#[test]
fn cr_8_3_8_arcane_barrier_refusee() {
    let mut t = partie(
        &main_eclair(),
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Capuche"],
    );
    eclair_sur_p1(&mut t, DecisionKind::Prevention);
    t.choisir(Choice::Pass);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.player(P1).hand.len(), 4);
}

#[test]
fn cr_8_3_8_arcane_barrier_seulement_contre_les_degats_arcaniques() {
    // Essai Étincelle : 1 dégât non arcanique.
    let mut t = partie(
        &[
            ("Essai Étincelle", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Capuche"],
    );
    let e = t.en_main(P0, "Essai Étincelle", R);
    t.choisir(Choice::Play(e));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 19);
}

#[test]
fn cr_8_3_8_arcane_barrier_impossible_sans_ressources() {
    let mut t = partie(
        &main_eclair(),
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Capuche"],
    );
    // P1 n'a plus de carte en main : il ne peut pas payer {r}.
    for c in t.g.player(P1).hand.clone() {
        t.g.discard(c);
    }
    eclair_sur_p1(&mut t, DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 17);
}

#[test]
fn cr_8_3_15_spellvoid_detruire_pour_prevenir() {
    let mut t = partie(
        &main_eclair(),
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Masque"],
    );
    eclair_sur_p1(&mut t, DecisionKind::Prevention);
    let masque = t.equipement(P1, "Essai Masque");
    // Deux préventions optionnelles sur le même objet, dans l'ordre choisi.
    assert_eq!(
        t.options(),
        vec![
            Choice::Prevent(PreventRef::ArcaneBarrier(masque)),
            Choice::Prevent(PreventRef::Spellvoid(masque)),
            Choice::Pass
        ]
    );
    t.choisir(Choice::Prevent(PreventRef::Spellvoid(masque)));
    // Le masque est détruit : son arcane barrier disparaît avec lui.
    assert_eq!(t.g.obj(masque).zone, Zone::Graveyard);
    assert!(t.g.player(P1).head.is_none());
    assert_eq!(t.vie(P1), 19); // 3 - 2
}

#[test]
fn cr_6_5_5_arcane_barrier_puis_spellvoid() {
    let mut t = partie(
        &main_eclair(),
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Masque"],
    );
    eclair_sur_p1(&mut t, DecisionKind::Prevention);
    let masque = t.equipement(P1, "Essai Masque");
    t.choisir(Choice::Prevent(PreventRef::ArcaneBarrier(masque)));
    t.choisir(Choice::Pitch(t.en_main(P1, "Essai Rempart", B)));
    // Reste 2 dégâts : le spellvoid est encore proposé (CR 6.4.5 : chaque
    // prévention s'applique une fois par événement).
    assert_eq!(
        t.options(),
        vec![Choice::Prevent(PreventRef::Spellvoid(masque)), Choice::Pass]
    );
    t.choisir(Choice::Prevent(PreventRef::Spellvoid(masque)));
    assert_eq!(t.vie(P1), 20);
}

#[test]
fn cr_8_3_20_ward_obligatoire_contre_des_degats_physiques() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Voile"],
    );
    let voile = t.equipement(P1, "Essai Voile");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    // Ward 1 : le voile est détruit (aucune question), 1 dégât prévenu ;
    // « when this is destroyed » : +1{h}.
    assert_eq!(t.g.obj(voile).zone, Zone::Graveyard);
    assert_eq!(t.vie(P1), 20 - 3 + 1);
}

#[test]
fn cr_8_3_20_ward_d_un_token_et_d_un_equipement_ensemble() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Voile"],
    );
    let ecran = token(&mut t, P1, "Essai Écran");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    // Les deux wards sont entièrement utilisés (2 ≤ 4) : pas de question.
    assert_eq!(t.g.obj(ecran).zone, Zone::Gone); // un token détruit cesse d'exister
    assert_eq!(t.vie(P1), 20 - 2 + 1);
}

#[test]
fn cr_6_5_5_ordre_des_preventions_au_choix_du_joueur() {
    // Bouclier « prevent the next 2 » (Essai Garde) + ward 1, contre 1 dégât :
    // l'ordre change le résultat, P1 choisit.
    let mut t = partie(
        &[
            ("Essai Étincelle", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Garde", Y),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &["Essai Voile"],
    );
    let voile = t.equipement(P1, "Essai Voile");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Étincelle", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    // P1 répond avec Essai Garde (instant), qui se résout en premier.
    assert_eq!(t.joueur(), P1);
    t.choisir(Choice::Play(t.en_main(P1, "Essai Garde", Y)));
    t.passer_jusqua(DecisionKind::Prevention);
    let shield = t.g.preventions[0].id;
    assert_eq!(
        t.options(),
        vec![
            Choice::Prevent(PreventRef::Effect(shield)),
            Choice::Prevent(PreventRef::Ward(voile))
        ]
    );
    t.choisir(Choice::Prevent(PreventRef::Effect(shield)));
    // Le dégât est prévenu par le bouclier : le voile reste, le bouclier garde 1.
    assert_eq!(t.vie(P1), 20);
    assert_eq!(t.g.obj(voile).zone, Zone::Head);
    assert_eq!(t.g.preventions[0].remaining, Some(1));
}

#[test]
fn cr_6_4_10i_prevention_fixe_une_seule_fois() {
    let mut t = partie(
        &[
            ("Essai Parapet", Y),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    t.choisir(Choice::Play(t.en_main(P0, "Essai Parapet", Y)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.preventions.len(), 1);
    // « The next time you would be dealt damage, prevent 2 of that damage » :
    // 1 dégât prévenu, le reste de la prévention est perdu (CR 6.4.10i).
    t.g.deal_damage(None, P1, P0, 1, false);
    assert_eq!(t.g.last_damage, 0);
    // « If you prevent damage this way » : +1{h}.
    assert_eq!(t.vie(P0), 21);
    assert!(t.g.preventions.is_empty());
    t.g.deal_damage(None, P1, P0, 1, false);
    assert_eq!(t.vie(P0), 20);
}

#[test]
fn cr_7_5_5b_toutes_les_blessures_prevenues_pas_de_touche() {
    // Comme Sand Cover : la cendre de P1 gagne ward 4 et absorbe l'attaque.
    let mut t = partie(
        &[
            ("Essai Morsure", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Couverture", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    );
    let cendre = token(&mut t, P1, "Essai Cendre");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Morsure", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.choisir(Choice::Pass);
    // P1 répond : la cible (sa cendre) est la seule possible.
    t.choisir(Choice::Play(t.en_main(P1, "Essai Couverture", R)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    // 3 dégâts, ward 4 : tout est prévenu, la cendre est détruite, pas de
    // touche (Essai Morsure « when this hits » ne donne pas de PV).
    assert_eq!(t.vie(P1), 20);
    assert_eq!(t.vie(P0), 20);
    assert_eq!(t.g.obj(cendre).zone, Zone::Gone);
    assert!(!t
        .g
        .turn_events()
        .iter()
        .any(|e| matches!(e, Event::Hit { .. })));
}

// ----------------------------------------------------------------------
// Effets de remplacement (CR 6.4, 6.5)
// ----------------------------------------------------------------------

#[test]
fn cr_6_4_9_au_cimetiere_banni_a_la_place_reaction_de_defense() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Ombre", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    );
    let ombre = t.en_main(P1, "Essai Ombre", R);
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Priority);
    while t.joueur() != P1 {
        t.choisir(Choice::Pass);
    }
    t.choisir(Choice::Play(ombre));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 19); // 4 - 3
                               // À la fermeture, la carte qui défendait aurait été mise au cimetière :
                               // elle est bannie à la place.
    assert_eq!(t.g.obj(ombre).zone, Zone::Banished);
    assert!(t.g.player(P1).graveyard.is_empty());
    assert!(t
        .g
        .turn_events()
        .iter()
        .any(|e| matches!(e, Event::Banished { card, .. } if *card == ombre)));
}

#[test]
fn cr_6_4_9_equipement_detruit_banni_a_la_place() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Gantelet Sombre"],
    );
    let gantelet = t.equipement(P1, "Essai Gantelet Sombre");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Defend(gantelet));
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    // Temper : -1{d}, il tombe à 0 et est détruit… donc banni.
    assert_eq!(t.g.obj(gantelet).zone, Zone::Banished);
    assert!(t.g.player(P1).arms.is_none());
}

#[test]
fn cr_8_3_21_ephemere_retire_de_la_partie() {
    let mut t = partie(
        &[
            ("Essai Tigre", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let tigre = t.en_main(P0, "Essai Tigre", R);
    t.choisir(Choice::Play(tigre));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 19);
    assert_eq!(t.g.obj(tigre).zone, Zone::Gone);
    assert!(t.g.player(P0).graveyard.is_empty());
}

#[test]
fn cr_6_4_9_remplacement_cree_par_un_effet_dessus_du_deck_devient_dessous() {
    let mut t = partie(
        &[
            ("Essai Renversement", B),
            ("Essai Frappe", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let frappe = t.en_main(P0, "Essai Frappe", R);
    t.g.put_on_top(frappe);
    assert_eq!(t.g.top_of_deck(P0), Some(frappe));
    t.g.draw(P0);
    t.choisir(Choice::Play(t.en_main(P0, "Essai Renversement", B)));
    t.passer_jusqua(DecisionKind::Priority);
    t.g.put_on_top(frappe);
    assert_eq!(t.g.player(P0).deck[0], frappe, "mise sous le deck");
    assert_ne!(t.g.top_of_deck(P0), Some(frappe));
    // L'effet dure jusqu'à la fin du tour.
    t.finir_le_tour();
    assert!(t.g.replacements.is_empty());
}

#[test]
fn cr_6_4_9_remplacement_statique_du_heros_un_token_de_plus() {
    let mut t = partie_heros(
        ["Essai Florian", "Essai Héros"],
        &[
            ("Essai Incantation", Y),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    t.choisir(Choice::Play(t.en_main(P0, "Essai Incantation", Y)));
    t.passer_jusqua(DecisionKind::Arsenal);
    let runes =
        t.g.permanents
            .iter()
            .filter(|&&o| t.g.def(o).name == "Essai Rune")
            .count();
    assert_eq!(runes, 2);
}

fn partie_gel() -> Partie {
    partie(
        &[
            ("Essai Gel", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Écailles"],
    )
}

fn givres(t: &Partie, p: PlayerId) -> usize {
    t.g.permanents
        .iter()
        .filter(|&&o| t.g.def(o).name == "Essai Givre" && t.g.obj(o).controller == p)
        .count()
}

#[test]
fn cr_6_4_9_remplacement_optionnel_accepte() {
    let mut t = partie_gel();
    let ecailles = t.equipement(P1, "Essai Écailles");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Gel", B)));
    t.choisir(Choice::Target(Target::Player(P1)));
    t.passer_jusqua(DecisionKind::Replacement);
    assert_eq!(t.joueur(), P1);
    t.choisir(Choice::Yes);
    assert_eq!(givres(&t, P1), 0);
    assert_eq!(t.g.obj(ecailles).zone, Zone::Graveyard);
}

#[test]
fn cr_6_4_9_remplacement_optionnel_refuse() {
    let mut t = partie_gel();
    let ecailles = t.equipement(P1, "Essai Écailles");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Gel", B)));
    t.choisir(Choice::Target(Target::Player(P1)));
    t.passer_jusqua(DecisionKind::Replacement);
    t.choisir(Choice::No);
    assert_eq!(givres(&t, P1), 1);
    assert_eq!(t.g.obj(ecailles).zone, Zone::Chest);
}

#[test]
fn cr_6_4_9_remplacement_qui_ne_concerne_pas_le_joueur() {
    // Un Givre créé sous le contrôle de P0 : les écailles de P1 ne s'appliquent pas.
    let mut t = partie_gel();
    t.choisir(Choice::Play(t.en_main(P0, "Essai Gel", B)));
    t.choisir(Choice::Target(Target::Player(P0)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(givres(&t, P0), 1);
}

#[test]
fn cr_6_5_4_remplacement_des_degats_avant_la_prevention() {
    let mut t = partie(
        &[
            ("Essai Attisement", R),
            ("Essai Éclair", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Capuche"],
    );
    t.choisir(Choice::Play(t.en_main(P0, "Essai Attisement", R)));
    t.passer_jusqua(DecisionKind::Priority);
    eclair_sur_p1(&mut t, DecisionKind::Prevention);
    let capuche = t.equipement(P1, "Essai Capuche");
    t.choisir(Choice::Prevent(PreventRef::ArcaneBarrier(capuche)));
    t.choisir(Choice::Pitch(t.en_main(P1, "Essai Rempart", B)));
    // 3 + 3 = 6 dégâts arcaniques, 1 prévenu.
    assert_eq!(t.vie(P1), 15);
    // « The next time » : l'effet est consommé.
    assert!(t.g.replacements.is_empty());
}

// ----------------------------------------------------------------------
// Attaquer autre chose qu'un héros (CR 1.4.5, 7.3.2a, 8.3.14)
// ----------------------------------------------------------------------

#[test]
fn cr_1_4_5_attaquer_un_allie() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Parade", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    );
    let dragon = token(&mut t, P1, "Essai Dragon");
    let r = t.g.obj_ref(dragon);
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    // CR 1.4.5 : P0 choisit la cible de l'attaque.
    assert_eq!(t.kind(), DecisionKind::AttackTarget);
    assert_eq!(
        t.options(),
        vec![
            Choice::Target(Target::Player(P1)),
            Choice::Target(Target::Object(r))
        ]
    );
    t.choisir(Choice::Target(Target::Object(r)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    // CR 7.3.2a : personne ne défend un allié ; CR 8.1.3a : pas de réaction
    // de défense non plus.
    let parade = t.en_main(P1, "Essai Parade", R);
    while t.kind() != DecisionKind::Arsenal {
        assert_ne!(t.kind(), DecisionKind::Defend);
        assert!(!t.peut(Choice::Play(parade)));
        t.choisir(Choice::Pass);
    }
    assert!(!t
        .g
        .turn_events()
        .iter()
        .any(|e| matches!(e, Event::Defend { .. })));
    // 4 dégâts à l'allié (2{h}) : il meurt (CR 1.10.2b), le héros n'a rien.
    assert_eq!(t.vie(P1), 20);
    assert_eq!(t.g.obj(dragon).zone, Zone::Gone);
    assert!(t.g.turn_events().iter().any(|e| matches!(
        e,
        Event::Hit { target_obj: Some(o), damage: 4, .. } if *o == dragon
    )));
}

#[test]
fn cr_2_5_3_allie_blesse_mais_vivant() {
    let mut t = partie(
        &[
            ("Essai Tigre", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let dragon = token(&mut t, P1, "Essai Dragon");
    let r = t.g.obj_ref(dragon);
    t.choisir(Choice::Play(t.en_main(P0, "Essai Tigre", R)));
    t.choisir(Choice::Target(Target::Object(r)));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.life_total(dragon), Some(1));
    assert_eq!(t.g.obj(dragon).zone, Zone::Permanent);
}

#[test]
fn cr_8_3_14_spectra_detruit_quand_il_devient_la_cible() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Mirage", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    );
    t.finir_le_tour();
    // Tour 2 : P1 joue son aura avec spectra.
    let mirage = t.en_main(P1, "Essai Mirage", B);
    t.choisir(Choice::Play(mirage));
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.obj(mirage).zone, Zone::Permanent);
    t.finir_le_tour();
    // Tour 3 : P0 attaque le mirage.
    let r = t.g.obj_ref(mirage);
    let frappe = t.en_main(P0, "Essai Frappe", R);
    t.choisir(Choice::Play(frappe));
    assert!(t.peut(Choice::Target(Target::Object(r))));
    t.choisir(Choice::Target(Target::Object(r)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Arsenal);
    // Le déclenchement de spectra détruit le mirage avant que l'attaque
    // devienne un maillon : l'attaque n'a plus de cible légale, elle est
    // retirée (CR 7.2.2c) et la chaîne se ferme.
    assert_eq!(t.g.obj(mirage).zone, Zone::Graveyard);
    assert_eq!(t.g.obj(frappe).zone, Zone::Graveyard);
    assert_eq!(t.vie(P1), 20);
    assert!(!t
        .g
        .turn_events()
        .iter()
        .any(|e| matches!(e, Event::Attack { .. })));
    assert!(!t.g.combat.open);
}

#[test]
fn cr_7_7_5_un_allie_qui_attaque_revient_dans_la_zone_permanente() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &[],
    );
    let dragon = token(&mut t, P0, "Essai Dragon");
    assert!(t.peut(Choice::Activate(dragon, 0)));
    t.choisir(Choice::Activate(dragon, 0));
    t.passer_jusqua(DecisionKind::Defend);
    assert_eq!(t.g.obj(dragon).zone, Zone::CombatChain);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 17);
    assert_eq!(t.g.obj(dragon).zone, Zone::Permanent);
    assert!(t.g.permanents.contains(&dragon));
    assert!(t.g.check_invariants().is_empty());
}

// ----------------------------------------------------------------------
// « Loses all abilities », « until the end of their next turn »
// ----------------------------------------------------------------------

/// P0 attaque avec `carte` (coût 0) et touche P1 (héros Essai Ninja). P1 a
/// deux Essai Assaut rouges (3{p}, go again, coût 0).
fn touche_le_ninja(carte: &str) -> Partie {
    let mut t = partie_heros(
        ["Essai Héros", "Essai Ninja"],
        &[
            (carte, R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Assaut", R),
            ("Essai Assaut", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    );
    t.choisir(Choice::Play(t.en_main(P0, carte, R)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 17);
    t
}

/// Au tour de P1 : joue un Essai Assaut et renvoie sa puissance pendant
/// l'attaque, puis avance jusqu'à la prochaine priorité de P1 à l'étape de
/// résolution (ou jusqu'à la fin du tour).
fn puissance_assaut_de_p1(t: &mut Partie) -> i16 {
    let a = t.en_main(P1, "Essai Assaut", R);
    t.choisir(Choice::Play(a));
    t.passer_jusqua(DecisionKind::Defend);
    let p = t.g.power(a);
    t.choisir(Choice::Pass);
    loop {
        let resolution = t.g.combat.step == Some(CombatStep::Resolution);
        match t.kind() {
            DecisionKind::Priority if t.joueur() == P1 && resolution => break,
            DecisionKind::Arsenal => break,
            _ => t.choisir(Choice::Pass),
        }
    }
    p
}

#[test]
fn humble_le_heros_perd_ses_capacites_jusqu_a_la_fin_de_son_prochain_tour() {
    let mut t = touche_le_ninja("Essai Humilité");
    let heros = t.g.player(P1).hero;
    assert!(t.g.lost_abilities(heros));
    t.finir_le_tour();
    // Tour 2 (P1) : la deuxième attaque ne reçoit pas le +1 du héros.
    assert!(t.g.lost_abilities(heros));
    assert_eq!(puissance_assaut_de_p1(&mut t), 3);
    assert_eq!(puissance_assaut_de_p1(&mut t), 3);
    t.finir_le_tour();
    // Tour 3 : l'effet est terminé.
    assert_eq!(t.g.turn, 3);
    assert!(!t.g.lost_abilities(heros));
}

#[test]
fn temoin_sans_humble_la_deuxieme_attaque_recoit_le_bonus() {
    let mut t = touche_le_ninja("Essai Assaut");
    t.finir_le_tour();
    assert_eq!(puissance_assaut_de_p1(&mut t), 3);
    assert_eq!(puissance_assaut_de_p1(&mut t), 4);
}

#[test]
fn debilitate_premiere_attaque_du_prochain_tour() {
    let mut t = touche_le_ninja("Essai Affaiblissement");
    t.finir_le_tour();
    // Tour 2 : première attaque -2, la deuxième normale (+1 du héros).
    assert_eq!(puissance_assaut_de_p1(&mut t), 1);
    assert_eq!(puissance_assaut_de_p1(&mut t), 4);
}

#[test]
fn debilitate_ne_s_applique_pas_au_tour_en_cours() {
    let mut t = touche_le_ninja("Essai Affaiblissement");
    // L'effet attend le tour 2 de P1 ; il survit à la fin du tour 1.
    assert_eq!(t.g.next_attack_effects.len(), 1);
    t.finir_le_tour();
    assert_eq!(t.g.next_attack_effects.len(), 1);
}

#[test]
fn perte_des_capacites_supprime_mots_cles_et_preventions() {
    let mut t = partie(
        QUATRE_REMPARTS,
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Masque"],
    );
    let masque = t.equipement(P1, "Essai Masque");
    assert_eq!(t.g.prevention_values(masque), (1, 0, 2));
    let r = t.g.obj_ref(masque);
    t.g.modify(
        r,
        fab_engine::Delta {
            lose_abilities: true,
            ..Default::default()
        },
        fab_engine::Until::EndOfTurn,
    );
    assert_eq!(t.g.prevention_values(masque), (0, 0, 0));
    t.g.deal_damage(None, P0, P1, 3, true);
    assert_eq!(t.vie(P1), 17);
}

// ----------------------------------------------------------------------
// Fermeture de la chaîne de combat (CR 7.7.3-7.7.5)
// ----------------------------------------------------------------------

#[test]
fn cr_7_7_4_declenchement_a_la_fermeture_avant_le_retour_des_permanents() {
    let mut t = partie(
        &[
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &["Essai Marteau"],
        &[],
    );
    let marteau = t.equipement(P0, "Essai Marteau");
    t.choisir(Choice::Activate(marteau, 0));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.vie(P1), 16);
    // « When the combat chain closes, if this attacked, banish it » : résolu
    // pendant la fermeture, avant que le marteau revienne en zone d'arme.
    assert_eq!(t.g.obj(marteau).zone, Zone::Banished);
    assert!(t.g.player(P0).weapons.is_empty());
    let i_ferme =
        t.g.turn_events()
            .iter()
            .position(|e| matches!(e, Event::CombatChainClosed))
            .unwrap();
    let i_banni =
        t.g.turn_events()
            .iter()
            .position(|e| matches!(e, Event::Banished { card, .. } if *card == marteau))
            .unwrap();
    assert!(i_ferme < i_banni);
    assert!(t.g.check_invariants().is_empty());
}

#[test]
fn cr_7_7_5_une_aura_qui_defend_n_est_pas_un_permanent() {
    // CR 1.3.3 : une carte de deck sur la chaîne de combat n'est pas un
    // permanent ; après la fermeture, elle va au cimetière.
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[
            ("Essai Aura", Y),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        &[],
        &[],
    );
    let aura = t.en_main(P1, "Essai Aura", Y);
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Defend(aura));
    t.choisir(Choice::Pass);
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.obj(aura).zone, Zone::Graveyard);
}

#[test]
fn cr_7_7_3_blade_break_resolu_pendant_la_fermeture() {
    let mut t = partie(
        &[
            ("Essai Frappe", R),
            ("Essai Frappe", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        QUATRE_REMPARTS,
        RIEN,
        &[],
        &["Essai Casque", "Essai Plastron", "Essai Gantelets"],
    );
    let casque = t.equipement(P1, "Essai Casque");
    let plastron = t.equipement(P1, "Essai Plastron");
    let gantelets = t.equipement(P1, "Essai Gantelets");
    t.choisir(Choice::Play(t.en_main(P0, "Essai Frappe", R)));
    t.choisir(Choice::Pitch(t.en_main(P0, "Essai Frappe", B)));
    t.passer_jusqua(DecisionKind::Defend);
    t.choisir(Choice::Defend(casque));
    t.choisir(Choice::Defend(plastron));
    t.choisir(Choice::Defend(gantelets));
    t.choisir(Choice::Pass);
    // Trois déclenchements de mots-clés différents : pas de question d'ordre.
    t.passer_jusqua(DecisionKind::Arsenal);
    assert_eq!(t.g.obj(casque).zone, Zone::Graveyard); // blade break
    assert_eq!(t.g.obj(gantelets).zone, Zone::Graveyard); // temper 1 → 0
    assert_eq!(t.g.obj(plastron).zone, Zone::Chest); // battleworn
    assert_eq!(t.g.defense(plastron), 1);
}

#[test]
fn des_parties_aleatoires_avec_les_nouveaux_mecanismes_restent_coherentes() {
    use fab_cards::test_set;
    use fab_engine::{GameSetup, GameState};
    let db = std::sync::Arc::new(test_set::build());
    for seed in 0..300 {
        let setup = GameSetup {
            players: test_set::mechanics_loadouts(&db),
            chooser: PlayerId((seed % 2) as u8),
            seed,
            shuffle: true,
            max_turns: 200,
        };
        let mut g = GameState::new(db.clone(), setup).unwrap();
        let mut a = fab_ai::RandomAgent::new(seed);
        let mut b = fab_ai::RandomAgent::new(seed + 1000);
        fab_ai::play_out(&mut g, [&mut a, &mut b], true)
            .unwrap_or_else(|e| panic!("graine {seed} : {e}"));
        assert!(g.is_over());
    }
}
