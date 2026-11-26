//! Tests du bot B2 (ISMCTS, étape 4) et de la déterminisation (les mondes
//! imaginés à partir de l'information d'un joueur).

mod commun;
use commun::*;
use fab_ai::b2::ReglagesB2;
use fab_ai::determinisation::{determiniser, objets_caches};
use fab_ai::{creer_agent, play_out, Agent, HeuristicAgent, IsmctsAgent, RandomAgent};
use fab_cards::decklist::Decklist;
use fab_cards::real;
use fab_engine::{CardId, Choice, DecisionKind, GameSetup, GameState, Loadout, ObjectId, Rng};
use std::sync::Arc;

/// Les 9 vrais decks, dans l'ordre des fichiers.
fn vrais_decks(db: &fab_engine::CardDb) -> Vec<Loadout> {
    let fichiers: Vec<_> = fab_cards::decks::tous()
        .into_iter()
        .map(|d| d.fichier)
        .collect();
    assert!(fichiers.len() >= 9);
    fichiers
        .iter()
        .map(|f| {
            let list = Decklist::parse(&std::fs::read_to_string(f).unwrap()).unwrap();
            real::default_loadout(db, &list).unwrap()
        })
        .collect()
}

/// Une partie entre deux vrais decks (numéros 0 à 8), après `decisions`
/// décisions jouées au hasard.
fn partie_en_cours(a: usize, b: usize, graine: u64, decisions: usize) -> GameState {
    let db = Arc::new(real::build());
    let decks = vrais_decks(&db);
    let setup = GameSetup {
        players: [decks[a].clone(), decks[b].clone()],
        chooser: P0,
        seed: graine,
        shuffle: true,
        max_turns: 200,
    };
    let mut g = GameState::new(db, setup).unwrap();
    let mut r = Rng::new(graine);
    for _ in 0..decisions {
        let Some(d) = g.pending_decision() else { break };
        let i = r.below(d.options.len() as u32) as usize;
        g.apply(i).unwrap();
    }
    g
}

fn b2(iterations: u32, graine: u64) -> IsmctsAgent {
    IsmctsAgent::avec(
        graine,
        ReglagesB2 {
            iterations,
            ..Default::default()
        },
    )
}

// ----------------------------------------------------------------------
// Déterminisation
// ----------------------------------------------------------------------

#[test]
fn la_determinisation_garde_tout_ce_que_le_joueur_voit() {
    for graine in 0..12 {
        let g = partie_en_cours(1, 2, graine, 30 + 5 * graine as usize);
        // Du point de vue du joueur qui décide (comme B2).
        {
            let moi = g.pending_decision().unwrap().player;
            let adv = moi.opponent();
            let d = determiniser(&g, moi, &mut Rng::new(graine + 100));
            // Ma main, les tailles des zones, les vies, la décision en attente.
            assert_eq!(d.player(moi).hand, g.player(moi).hand);
            assert_eq!(d.player(adv).hand.len(), g.player(adv).hand.len());
            assert_eq!(d.player(adv).deck.len(), g.player(adv).deck.len());
            assert_eq!(d.player(moi).deck.len(), g.player(moi).deck.len());
            assert_eq!(
                d.player(adv).arsenal.is_some(),
                g.player(adv).arsenal.is_some()
            );
            assert_eq!(d.player(adv).life, g.player(adv).life);
            assert_eq!(d.pending, g.pending);
            // Les objets que je vois n'ont pas changé.
            let caches = objets_caches(&g, moi);
            for (i, o) in g.objects.iter().enumerate() {
                if !caches.contains(&ObjectId(i as u32)) {
                    assert_eq!(d.objects[i].zone, o.zone, "objet visible {i} déplacé");
                    assert_eq!(d.objects[i].card, o.card, "objet visible {i} changé");
                }
            }
            // Les zones cachées gardent leurs objets (seule leur carte change).
            let mut a = caches.clone();
            let mut b = objets_caches(&d, moi);
            a.sort();
            b.sort();
            assert_eq!(a, b);
            assert_eq!(d.player(adv).hand, g.player(adv).hand);
            assert!(
                d.check_invariants().is_empty(),
                "{:?}",
                d.check_invariants()
            );
        }
    }
}

#[test]
fn la_determinisation_change_la_main_adverse_et_le_hasard() {
    let g = partie_en_cours(0, 3, 3, 10);
    let moi = g.pending_decision().unwrap().player;
    let adv = moi.opponent();
    let mut r = Rng::new(1);
    let mondes: Vec<GameState> = (0..20).map(|_| determiniser(&g, moi, &mut r)).collect();
    assert!(mondes
        .iter()
        .any(|d| cartes(d, &d.player(adv).hand) != cartes(&g, &g.player(adv).hand)));
    assert!(mondes
        .iter()
        .any(|d| d.player(moi).deck != g.player(moi).deck));
    assert!(mondes.iter().all(|d| d.rng != g.rng));
}

/// Les cartes de ces objets, triées.
fn cartes(g: &GameState, ids: &[ObjectId]) -> Vec<CardId> {
    let mut v: Vec<CardId> = ids.iter().map(|&i| g.obj(i).card).collect();
    v.sort();
    v
}

/// Retire de `v` un exemplaire de chaque carte de `moins` ; faux si l'une
/// manque.
fn retirer(v: &mut Vec<CardId>, moins: &[CardId]) -> bool {
    for c in moins {
        match v.iter().position(|x| x == c) {
            Some(k) => {
                v.remove(k);
            }
            None => return false,
        }
    }
    true
}

#[test]
fn les_cartes_imaginees_viennent_du_card_pool_moins_les_cartes_vues() {
    for graine in 0..12 {
        let (a, b) = (graine as usize % 9, (graine as usize + 5) % 9);
        let g = partie_en_cours(a, b, graine, 40 + 10 * graine as usize);
        let Some(moi) = g.pending_decision().map(|d| d.player) else {
            continue;
        };
        let adv = moi.opponent();
        let caches_adv: Vec<ObjectId> = objets_caches(&g, moi)
            .into_iter()
            .filter(|&o| g.obj(o).owner == adv)
            .collect();
        // Cartes de deck de l'adversaire que je vois (hors cartes cachées).
        let (debut, fin) = g.deck_objects[adv.index()];
        let vues: Vec<CardId> = (debut..fin)
            .map(ObjectId)
            .filter(|o| !caches_adv.contains(o))
            .map(|o| {
                let def = g.def(o);
                def.flip
                    .map(|f| f.front)
                    .or(def.split.map(|s| s.whole))
                    .unwrap_or(g.obj(o).card)
            })
            .collect();
        let mut possibles = g.card_pools[adv.index()].clone();
        assert!(retirer(&mut possibles, &vues), "graine {graine}");
        let mut r = Rng::new(graine);
        for _ in 0..10 {
            let d = determiniser(&g, moi, &mut r);
            let mut reste = possibles.clone();
            assert!(
                retirer(&mut reste, &cartes(&d, &caches_adv)),
                "graine {graine} : carte imaginée hors du card-pool"
            );
        }
    }
}

#[test]
fn le_bot_imagine_des_cartes_du_pool_que_l_adversaire_n_a_pas_mises_dans_son_deck() {
    // Ira (adversaire) : 48 cartes de deck dans son card-pool, 40 dans le deck.
    let g = partie_en_cours(2, 0, 1, 1);
    let moi = g.pending_decision().unwrap().player;
    let adv = moi.opponent();
    assert!(g.card_pools[adv.index()].len() > 40);
    let caches: Vec<ObjectId> = objets_caches(&g, moi)
        .into_iter()
        .filter(|&o| g.obj(o).owner == adv)
        .collect();
    let vraies = cartes(&g, &caches);
    let mut r = Rng::new(3);
    let hors_du_deck = (0..20).any(|_| {
        let d = determiniser(&g, moi, &mut r);
        let mut v = vraies.clone();
        !retirer(&mut v, &cartes(&d, &caches))
    });
    assert!(hors_du_deck);
}

#[test]
fn une_partie_determinisee_se_joue_jusqu_au_bout_sans_erreur() {
    // Toutes les paires de decks (en diagonale décalée), à divers moments.
    for a in 0..9 {
        let b = (a + 4) % 9;
        let g = partie_en_cours(a, b, a as u64, 20 + 15 * a);
        let mut r = Rng::new(a as u64);
        let Some(moi) = g.pending_decision().map(|d| d.player) else {
            continue;
        };
        for _ in 0..2 {
            let mut d = determiniser(&g, moi, &mut r);
            let mut x = RandomAgent::new(1);
            let mut y = HeuristicAgent::new();
            play_out(&mut d, [&mut x, &mut y], true)
                .unwrap_or_else(|e| panic!("decks {a} contre {b} : {e}"));
        }
    }
}

// ----------------------------------------------------------------------
// B2 n'utilise pas l'information cachée
// ----------------------------------------------------------------------

#[test]
fn b2_fait_le_meme_choix_quelle_que_soit_la_vraie_main_adverse() {
    // Deux parties qui ne diffèrent que par l'information cachée pour le
    // joueur qui décide (main adverse, decks) : avec la même graine, B2 doit
    // faire exactement le même choix.
    let mut compares = 0;
    for graine in 0..10 {
        let g = partie_en_cours(graine as usize % 9, 2, graine, 12 + 9 * graine as usize);
        let Some(d) = g.pending_decision().cloned() else {
            continue;
        };
        if d.options.len() < 2 {
            continue;
        }
        let autre = determiniser(&g, d.player, &mut Rng::new(graine + 7));
        let i = b2(40, 5).choose(&g, &d);
        let j = b2(40, 5).choose(&autre, &d);
        assert_eq!(i, j, "graine {graine} : choix différents");
        compares += 1;
    }
    assert!(compares >= 5, "trop peu de décisions comparées");
}

#[test]
fn b2_meme_graine_meme_choix() {
    let g = partie_en_cours(4, 6, 2, 25);
    let d = g.pending_decision().unwrap().clone();
    assert_eq!(b2(30, 9).choose(&g, &d), b2(30, 9).choose(&g, &d));
}

// ----------------------------------------------------------------------
// B2 cherche le bon coup
// ----------------------------------------------------------------------

#[test]
fn b2_joue_le_degat_direct_qui_gagne_plutot_qu_une_attaque_bloquee() {
    // L'adversaire est à 1 PV avec 4 Parades rouges (défense 4) en main.
    // Une Frappe rouge (4) serait entièrement bloquée ; l'Étincelle (instant,
    // 1 dégât au héros adverse) gagne tout de suite. Toutes les cartes de P0
    // sont rouges (1 ressource chacune) : il ne peut payer qu'une carte.
    let mut t = partie(
        &[
            ("Essai Étincelle", R),
            ("Essai Frappe", R),
            ("Essai Frappe", R),
            ("Essai Frappe", R),
        ],
        RIEN,
        &[
            ("Essai Parade", R),
            ("Essai Parade", R),
            ("Essai Parade", R),
            ("Essai Parade", R),
        ],
        RIEN,
        &[],
        &[],
    );
    t.g.player_mut(P1).life = 1;
    assert_eq!(t.kind(), DecisionKind::Priority);
    let etincelle = t.en_main(P0, "Essai Étincelle", R);
    let d = t.g.pending_decision().unwrap().clone();
    let i = b2(200, 1).choose(&t.g, &d);
    assert_eq!(d.options[i], Choice::Play(etincelle));
}

#[test]
fn b2_ne_cherche_pas_quand_il_n_y_a_qu_un_choix() {
    let t = partie(&[("Essai Frappe", R)], RIEN, RIEN, RIEN, &[], &[]);
    let mut d = t.g.pending_decision().unwrap().clone();
    d.options.truncate(1);
    let mut bot = b2(50, 1);
    assert_eq!(bot.choose(&t.g, &d), 0);
    assert_eq!(bot.recherches, 0);
    assert_eq!(bot.iterations_faites, 0);
}

#[test]
fn b2_respecte_son_nombre_d_iterations() {
    let g = partie_en_cours(0, 1, 4, 15);
    let d = g.pending_decision().unwrap().clone();
    let mut bot = b2(37, 1);
    bot.choose(&g, &d);
    if bot.recherches == 1 {
        assert_eq!(bot.iterations_faites, 37);
    }
}

#[test]
fn les_reglages_de_b2_se_lisent_dans_son_nom() {
    let r = ReglagesB2::depuis_nom("300t6c1.5a").unwrap();
    assert_eq!(r.iterations, 300);
    assert_eq!(r.tours_simules, Some(6));
    assert_eq!(r.exploration, 1.5);
    assert_eq!(r.politique, fab_ai::b2::Politique::Aleatoire);
    let r = ReglagesB2::depuis_nom("500ms").unwrap();
    assert_eq!(r.duree_max, Some(std::time::Duration::from_millis(500)));
    let r = ReglagesB2::depuis_nom("100t0p0").unwrap();
    assert_eq!(r.tours_simules, None);
    assert_eq!(r.profondeur, u32::MAX);
    let r = ReglagesB2::depuis_nom("300d2.5g0.5").unwrap();
    assert_eq!(r.dette_tours, 2.5);
    assert_eq!(r.dette_cimetiere, 0.5);
    assert!(ReglagesB2::depuis_nom("0").is_none());
    assert!(ReglagesB2::depuis_nom("12x").is_none());
    assert!(creer_agent("b2-50", 1).is_some());
    assert!(creer_agent("b2-", 1).is_none());
}

// ----------------------------------------------------------------------
// Parties complètes
// ----------------------------------------------------------------------

#[test]
fn b2_joue_des_parties_completes_avec_les_9_decks_sans_erreur() {
    // Peu d'itérations (le test doit rester rapide) : on vérifie que la
    // recherche marche avec chaque deck et que l'état reste cohérent.
    let db = Arc::new(real::build());
    let decks = vrais_decks(&db);
    for (i, a) in decks.iter().enumerate() {
        let j = (i + 2) % 9;
        let setup = GameSetup {
            players: [a.clone(), decks[j].clone()],
            chooser: P0,
            seed: i as u64,
            shuffle: true,
            max_turns: 200,
        };
        let mut g = GameState::new(db.clone(), setup).unwrap();
        let mut x = b2(4, i as u64);
        let mut y = HeuristicAgent::new();
        play_out(&mut g, [&mut x, &mut y], true)
            .unwrap_or_else(|e| panic!("decks {i} contre {j} : {e}"));
        assert!(g.is_over());
        assert!(x.recherches > 0);
    }
}

// ---------------------------------------------------------------------------
// Blood debt dans l'estimation (Levia)
// ---------------------------------------------------------------------------

/// Une partie Levia (joueur 0) contre Ira, avec trois cartes à blood debt de
/// Levia prises dans son deck : une bannie face visible, une bannie face
/// cachée, une au cimetière.
fn levia_avec_dette() -> GameState {
    use fab_engine::Keywords;
    let mut g = partie_en_cours(6, 0, 5, 0);
    assert_eq!(g.def(g.player(P0).hero).name, "Levia");
    let dettes: Vec<ObjectId> = g
        .player(P0)
        .deck
        .iter()
        .copied()
        .filter(|&c| g.def(c).keywords.has(Keywords::BLOOD_DEBT))
        .take(3)
        .collect();
    assert_eq!(dettes.len(), 3);
    g.banish(P0, dettes[0], false);
    g.banish(P0, dettes[1], true);
    g.put_in_graveyard(dettes[2]);
    g
}

#[test]
fn dette_a_venir_compte_les_cartes_bannies_face_visible_et_une_part_du_cimetiere() {
    use fab_ai::b2::dette_a_venir;
    let g = levia_avec_dette();
    // CR 8.3.11a : la carte bannie face cachée ne compte pas.
    assert_eq!(dette_a_venir(&g, P0, 2.0, 0.0), 2.0);
    // Cimetière : 0,5 × 2 tours × 1 carte.
    assert_eq!(dette_a_venir(&g, P0, 2.0, 0.5), 3.0);
    assert_eq!(dette_a_venir(&g, P0, 0.0, 0.5), 0.0);
    // Ira n'a pas de blood debt.
    assert_eq!(dette_a_venir(&g, P1, 2.0, 0.5), 0.0);
}

#[test]
fn dette_a_venir_ne_depasse_pas_la_vie() {
    use fab_ai::b2::dette_a_venir;
    let mut g = levia_avec_dette();
    g.players[0].life = 2;
    assert_eq!(dette_a_venir(&g, P0, 10.0, 0.0), 2.0);
}

#[test]
fn le_blood_debt_a_venir_fait_baisser_l_estimation_de_levia() {
    use fab_ai::b2::{estimer, estimer_avec_dette};
    let g = levia_avec_dette();
    let sans = estimer(&g, 1.5);
    assert_eq!(estimer_avec_dette(&g, 1.5, [0.0, 0.0]), sans);
    assert!(estimer_avec_dette(&g, 1.5, [3.0, 0.0]) < sans);
    assert!(estimer_avec_dette(&g, 1.5, [0.0, 3.0]) > sans);
    // B3 : même sens.
    let e = fab_ai::b3::Modele::par_defaut().indexer(&g.db);
    let sans = e.estimer_pour_la_recherche(&g);
    assert_eq!(e.estimer_pour_la_recherche_avec_dette(&g, [0.0, 0.0]), sans);
    assert!(e.estimer_pour_la_recherche_avec_dette(&g, [3.0, 0.0]) < sans);
}

/// Levia (P0) au tour 1, deux Feeding Frenzy (5{p} et 4{p}, blood debt)
/// bannies face visible par l'adversaire, `main` en main.
fn levia_endettee(main: &[(&str, Option<fab_engine::Color>)], dettes: usize) -> Partie {
    let mut t = partie_vraie(
        ["Levia", "Essai Héros"],
        main,
        &[("Feeding Frenzy", Y), ("Feeding Frenzy", B)],
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let frenzy: Vec<ObjectId> = t
        .g
        .player(P0)
        .deck
        .iter()
        .copied()
        .filter(|&c| t.g.def(c).name == "Feeding Frenzy")
        .take(dettes)
        .collect();
    for c in frenzy {
        t.g.banish(P1, c, false);
    }
    t.g.refresh_priority();
    t
}

/// L'option « passer » de la décision en cours de Levia (qui finirait son tour).
fn passer_de_levia(t: &Partie) -> (fab_engine::Decision, usize) {
    let d = t.g.pending_decision().expect("décision").clone();
    assert_eq!(d.player, P0);
    assert_eq!(d.kind, DecisionKind::Priority);
    let i = d.index_of(Choice::Pass).expect("passer");
    (d, i)
}

const REMPART: (&str, Option<fab_engine::Color>) = ("Essai Rempart", B);

#[test]
fn parties_simulees_levia_bannit_cleave_rouge_plutot_que_payer_son_blood_debt() {
    use fab_ai::b2::eviter_le_blood_debt;
    let mut t = levia_endettee(&[("Cleave the Heavens", R), REMPART, REMPART, REMPART], 2);
    let (d, passer) = passer_de_levia(&t);
    let i = eviter_le_blood_debt(&t.g, &d, passer).expect("une capacité à activer");
    let cleave = t.en_main(P0, "Cleave the Heavens", R);
    assert!(matches!(d.options[i], Choice::Activate(o, _) if o == cleave));
    // Cleave rouge (6{p}) bannie : Levia ne perd rien en fin de tour.
    let avant = t.vie(P0);
    t.g.apply(i).unwrap();
    t.finir_le_tour();
    assert_eq!(t.vie(P0), avant);
    // Autre choix que « passer » : rien ne change.
    assert_eq!(eviter_le_blood_debt(&t.g, &d, i), None);
}

#[test]
fn parties_simulees_cleave_jaune_ne_protege_pas_levia_donc_rien_n_est_active() {
    use fab_ai::b2::eviter_le_blood_debt;
    // 5{p} : bannir la jaune ne protège pas Levia (et ajoute une dette).
    let t = levia_endettee(&[("Cleave the Heavens", Y), REMPART, REMPART, REMPART], 2);
    let (d, passer) = passer_de_levia(&t);
    assert_eq!(eviter_le_blood_debt(&t.g, &d, passer), None);
}

#[test]
fn parties_simulees_une_seule_carte_de_dette_ne_vaut_pas_une_carte_de_la_main() {
    use fab_ai::b2::eviter_le_blood_debt;
    let t = levia_endettee(&[("Cleave the Heavens", R), REMPART, REMPART, REMPART], 1);
    let (d, passer) = passer_de_levia(&t);
    assert_eq!(eviter_le_blood_debt(&t.g, &d, passer), None);
}
