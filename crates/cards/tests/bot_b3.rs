//! Tests du bot B3 (étape 7) : la recherche de B2 avec une évaluation
//! apprise par auto-jeu (`fab_ai::b3`).

mod commun;
use commun::*;
use fab_ai::b2::ReglagesB2;
use fab_ai::b3::{caracteristiques, ecrire_modele, Apprentissage, Carac, Modele, GENERIQUES};
use fab_ai::determinisation::determiniser;
use fab_ai::{creer_agent, play_out, Agent, HeuristicAgent, IsmctsAgent};
use fab_cards::{decks, real};
use fab_engine::{GameSetup, GameState, Loadout, PlayerId, Rng};
use std::sync::Arc;

fn vrais_decks(db: &fab_engine::CardDb) -> Vec<Loadout> {
    decks::noms()
        .iter()
        .map(|n| real::default_loadout_pool(db, &decks::charger_pool(db, n).unwrap()))
        .collect()
}

fn partie(db: &Arc<fab_engine::CardDb>, a: usize, b: usize, graine: u64) -> GameState {
    let d = vrais_decks(db);
    let setup = GameSetup {
        players: [d[a].clone(), d[b].clone()],
        chooser: P0,
        seed: graine,
        shuffle: true,
        max_turns: 200,
    };
    GameState::new(db.clone(), setup).unwrap()
}

/// Joue `decisions` décisions au hasard.
fn avancer(g: &mut GameState, graine: u64, decisions: usize) {
    let mut r = Rng::new(graine);
    for _ in 0..decisions {
        let Some(d) = g.pending_decision() else { break };
        let i = r.below(d.options.len() as u32) as usize;
        g.apply(i).unwrap();
    }
}

fn b3(iterations: u32, graine: u64) -> IsmctsAgent {
    IsmctsAgent::avec(
        graine,
        ReglagesB2 {
            iterations,
            ..ReglagesB2::b3()
        },
    )
}

#[test]
fn le_fichier_des_poids_de_b3_se_lit_et_connait_tous_les_heros() {
    let m = Modele::par_defaut();
    let db = real::build();
    for n in GENERIQUES {
        assert!(m.poids.iter().any(|(k, _)| k == n), "{n} absent du fichier");
    }
    for d in decks::noms() {
        let pool = decks::charger_pool(&db, &d).unwrap();
        let nom = format!("héros {}", db.get(pool.hero).name);
        assert!(m.poids.iter().any(|(k, _)| *k == nom), "{nom} absent");
    }
}

#[test]
fn les_poids_ecrits_puis_relus_donnent_la_meme_evaluation() {
    let db = Arc::new(real::build());
    let mut g = partie(&db, 1, 7, 3);
    avancer(&mut g, 3, 40);
    let mut poids: Vec<(Carac, f64)> = Vec::new();
    for p in [PlayerId::P0, PlayerId::P1] {
        caracteristiques(&g, p, &mut |c, _| {
            if !poids.iter().any(|(k, _)| *k == c) {
                poids.push((c, 0.01 * (poids.len() as f64 + 1.0)));
            }
        });
    }
    let texte = ecrire_modele(&db, &poids, "# essai\n");
    let e = Modele::lire(&texte).unwrap().indexer(&db);
    for &(c, v) in &poids {
        assert!((e.poids(c) as f64 - v).abs() < 1e-4, "{}", c.nom(&db));
    }
}

#[test]
fn l_evaluation_est_la_meme_vue_de_chaque_joueur() {
    // σ(note₀ − note₁) : les chances des deux joueurs font 1.
    let db = Arc::new(real::build());
    let e = Modele::par_defaut().indexer(&db);
    let mut g = partie(&db, 0, 4, 1);
    avancer(&mut g, 1, 60);
    let p0 = e.estimer(&g);
    let x = e.note(&g, PlayerId::P1) - e.note(&g, PlayerId::P0);
    let p1 = 1.0 / (1.0 + (-x).exp());
    assert!((p0 + p1 - 1.0).abs() < 1e-5);
    assert!(p0 > 0.0 && p0 < 1.0);
}

#[test]
fn les_permanents_comptent_par_carte_et_pas_le_heros_ni_l_equipement() {
    let db = Arc::new(real::build());
    let mut g = partie(&db, 0, 1, 2);
    avancer(&mut g, 2, 30);
    let mut perms = 0;
    caracteristiques(&g, PlayerId::P0, &mut |c, v| {
        if let Carac::Permanent(card) = c {
            let d = db.get(card);
            assert!(!d.types.any(
                fab_engine::Types::HERO | fab_engine::Types::WEAPON | fab_engine::Types::EQUIPMENT
            ));
            assert_eq!(v, 1.0);
            perms += 1;
        }
    });
    let attendus = g
        .permanents
        .iter()
        .filter(|&&o| {
            let ob = g.obj(o);
            let d = g.def(o);
            ob.controller == PlayerId::P0
                && ob.proxy_of.is_none()
                && !d.types.any(
                    fab_engine::Types::HERO
                        | fab_engine::Types::WEAPON
                        | fab_engine::Types::EQUIPMENT
                        | fab_engine::Types::DEMI_HERO,
                )
        })
        .count();
    assert_eq!(perms, attendus);
}

#[test]
fn l_apprentissage_trouve_qu_avoir_plus_de_vie_aide_a_gagner() {
    // Positions de parties B1 contre B1 (Hala contre Dorinthea) : le poids
    // appris pour la vie doit être positif, et la prédiction meilleure que
    // « toujours 50 % » (perte 0,693).
    let db = Arc::new(real::build());
    let mut app = Apprentissage::new();
    for graine in 0..60 {
        let mut g = partie(&db, 1, 2, graine);
        let mut positions = Vec::new();
        let mut tour = 0;
        let (mut a, mut b) = (HeuristicAgent::new(), HeuristicAgent::new());
        while let Some(d) = g.pending_decision().cloned() {
            if g.turn != tour {
                tour = g.turn;
                positions.push(g.clone());
            }
            let i = if d.player == PlayerId::P0 {
                a.choose(&g, &d)
            } else {
                b.choose(&g, &d)
            };
            g.apply(i).unwrap();
        }
        let y = match g.outcome {
            Some(fab_engine::Outcome::Winner(PlayerId::P0)) => 1.0,
            Some(fab_engine::Outcome::Winner(_)) => 0.0,
            _ => 0.5,
        };
        for p in &positions {
            app.ajouter(p, y);
        }
    }
    let (poids, perte) = app.resoudre(10.0);
    let vie = poids
        .iter()
        .find(|(c, _)| *c == Carac::Generique(0))
        .map(|p| p.1)
        .unwrap();
    assert!(vie > 0.0, "poids de la vie : {vie}");
    assert!(perte < 0.6, "perte : {perte}");
    assert!((app.perte(&poids) - perte).abs() < 1e-9);
}

#[test]
fn b3_fait_le_meme_choix_quelle_que_soit_la_vraie_main_adverse() {
    let db = Arc::new(real::build());
    let mut compares = 0;
    for graine in 0..8u64 {
        let mut g = partie(&db, graine as usize % 9, 6, graine);
        avancer(&mut g, graine, 15 + 7 * graine as usize);
        let Some(d) = g.pending_decision().cloned() else {
            continue;
        };
        if d.options.len() < 2 {
            continue;
        }
        let autre = determiniser(&g, d.player, &mut Rng::new(graine + 11));
        let i = b3(30, 5).choose(&g, &d);
        let j = b3(30, 5).choose(&autre, &d);
        assert_eq!(i, j, "graine {graine} : choix différents");
        compares += 1;
    }
    assert!(compares >= 4, "trop peu de décisions comparées");
}

#[test]
fn b3_se_cree_par_son_nom() {
    let b = creer_agent("b3", 1).unwrap();
    assert!(b.name().starts_with("B3"));
    assert!(creer_agent("b3-100t2", 1).unwrap().name().starts_with("B3"));
    assert!(creer_agent("b2", 1).unwrap().name().starts_with("B2"));
    assert!(creer_agent("b3:fichier/qui/n/existe/pas.txt", 1).is_none());
    assert!(creer_agent("b3-", 1).is_none());
}

#[test]
fn b3_joue_des_parties_completes_avec_chaque_deck_sans_erreur() {
    let db = Arc::new(real::build());
    let n = decks::noms().len();
    for i in 0..n {
        let j = (i + 4) % n;
        let mut g = partie(&db, i, j, 40 + i as u64);
        let mut x = b3(4, i as u64);
        let mut y = HeuristicAgent::new();
        play_out(&mut g, [&mut x, &mut y], true)
            .unwrap_or_else(|e| panic!("decks {i} contre {j} : {e}"));
        assert!(g.is_over());
        assert!(x.recherches > 0);
    }
}

#[test]
fn le_contenu_de_la_main_compte_chaque_carte_de_la_main_et_de_l_arsenal() {
    use fab_engine::{Subtypes, Types};
    let db = Arc::new(real::build());
    let mut verifies = 0;
    for graine in 0..6u64 {
        let mut g = partie(&db, graine as usize % 9, (graine as usize + 3) % 9, graine);
        avancer(&mut g, graine, 25 + 5 * graine as usize);
        for p in [PlayerId::P0, PlayerId::P1] {
            let ps = g.player(p);
            let cartes: Vec<_> = ps.hand.iter().chain(ps.arsenal.iter()).copied().collect();
            let mut en_main = 0.0;
            let mut gen = [0.0f32; 5];
            caracteristiques(&g, p, &mut |c, v| match c {
                Carac::EnMain(card) => {
                    assert!(cartes.iter().any(|&o| g.obj(o).card == card));
                    en_main += v;
                }
                Carac::Generique(k) if k >= 12 => gen[k as usize - 12] = v,
                _ => {}
            });
            assert_eq!(en_main, cartes.len() as f32, "une valeur par carte");
            let mut attendu = [0.0f32; 5];
            for &o in &cartes {
                let d = g.def(o);
                if d.subtypes.has(Subtypes::ATTACK) {
                    attendu[0] += 1.0;
                    attendu[1] += d.power.unwrap_or(0) as f32;
                }
                attendu[2] += d.defense.unwrap_or(0) as f32;
                if d.types.has(Types::DEFENSE_REACTION) {
                    attendu[3] += 1.0;
                }
                attendu[4] += d.pitch.unwrap_or(0) as f32;
            }
            assert_eq!(
                gen, attendu,
                "attaques, puissance, défense, réactions, ressources"
            );
            verifies += (!cartes.is_empty()) as u32;
        }
    }
    assert!(verifies >= 6, "trop peu de mains vérifiées");
}

#[test]
fn les_noms_des_caracteristiques_du_contenu_de_la_main() {
    let db = real::build();
    assert_eq!(GENERIQUES[12], "attaques en main");
    assert_eq!(GENERIQUES[16], "ressources en main");
    let c = fab_engine::CardId(3);
    assert!(Carac::EnMain(c).nom(&db).starts_with("en main "));
    assert_eq!(
        Carac::depuis_code(Carac::EnMain(c).code()),
        Carac::EnMain(c)
    );
    assert!(Apprentissage::est_contenu_de_la_main(Carac::EnMain(c)));
    assert!(Apprentissage::est_contenu_de_la_main(Carac::Generique(14)));
    assert!(!Apprentissage::est_contenu_de_la_main(Carac::Generique(3)));
    assert!(!Apprentissage::est_contenu_de_la_main(Carac::Permanent(c)));
}

#[test]
fn filtrer_retire_les_caracteristiques_du_contenu_de_la_main() {
    let db = Arc::new(real::build());
    let mut app = Apprentissage::new();
    for graine in 0..4u64 {
        let mut g = partie(&db, 1, 2, graine);
        avancer(&mut g, graine, 30);
        app.ajouter(&g, (graine % 2) as f32);
    }
    let sans = app.filtrer(|c| !Apprentissage::est_contenu_de_la_main(c));
    assert_eq!(sans.nb_positions(), app.nb_positions());
    let (poids, _) = app.resoudre(10.0);
    assert!(poids.iter().any(|(c, _)| matches!(c, Carac::EnMain(_))));
    let (poids, _) = sans.resoudre(10.0);
    assert!(!poids
        .iter()
        .any(|(c, _)| Apprentissage::est_contenu_de_la_main(*c)));
    assert!(poids.iter().any(|(c, _)| *c == Carac::Generique(0)));
}
