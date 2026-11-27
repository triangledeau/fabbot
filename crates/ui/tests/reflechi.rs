//! Le niveau réfléchi dans la page : le bot réfléchit dans un fil à part,
//! la page suit sa réflexion (jauge), « Joue maintenant » et « Annuler »
//! l'interrompent, et rien de caché n'est montré.

use fab_engine::{ObjectId, PlayerId};
use fab_ui::vue;
use fab_ui::web::{Reglages, Session};
use std::time::{Duration, Instant};

fn session(niveau: &str, graine: u64) -> Session {
    Session::nouvelle(Reglages {
        deck: "hala".into(),
        deck_bot: "dorinthea".into(),
        niveau: niveau.into(),
        graine,
        manches: 1,
    })
    .unwrap()
}

/// Joue (option 0 pour l'humain) jusqu'à ce que le bot réfléchisse.
fn jusqu_a_une_reflexion(s: &mut Session) {
    for _ in 0..2_000 {
        if s.reflechit() {
            return;
        }
        assert!(!s.g.is_over(), "partie finie sans réflexion du bot");
        s.choisir(0).unwrap();
    }
    panic!("le bot n'a jamais réfléchi");
}

/// Attend que le coup du bot soit appliqué, comme la page qui interroge le
/// serveur (sans attendre ses réflexions suivantes).
fn attendre_le_coup(s: &mut Session) {
    let debut = Instant::now();
    while !s.poursuivre(false).unwrap() {
        assert!(
            debut.elapsed() < Duration::from_secs(5),
            "coup du bot trop long"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Attend que la réflexion ait vraiment commencé (le fil a démarré).
fn attendre_le_debut(s: &Session) {
    let debut = Instant::now();
    while s.reflexion_json()["iterations"].as_u64().unwrap_or(0) == 0 {
        assert!(debut.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn le_bot_reflechit_a_part_et_la_page_suit_sa_reflexion() {
    let mut s = session("reflechi-200ms", 5);
    let mut reflexions = 0;
    let mut n = 0;
    while !s.g.is_over() && n < 120 {
        if s.reflechit() {
            reflexions += 1;
            // Pendant la réflexion : la page a la jauge, pas de décision.
            let v = s.etat();
            assert!(v["decision"].is_null());
            assert_eq!(v["reflexion"]["en_cours"], true);
            assert!(v["reflexion"]["prevu_ms"].as_u64().unwrap() <= 200);
            assert!(
                s.choisir(0).is_err(),
                "l'humain ne joue pas pendant la réflexion"
            );
            attendre_le_debut(&s);
            let r = s.reflexion_json();
            // Les options montrées ne nomment aucune carte cachée du bot.
            let g = &s.g;
            let cachees: Vec<&str> = g.players[1]
                .hand
                .iter()
                .chain(g.players[1].arsenal.iter())
                .map(|&o| g.def(o).name.as_str())
                .filter(|nom| {
                    !(0..g.objects.len() as u32).any(|i| {
                        vue::visible(g, PlayerId::P0, ObjectId(i))
                            && g.def(ObjectId(i)).name == *nom
                    })
                })
                .collect();
            for o in r["options"].as_array().unwrap() {
                let t = o["texte"].as_str().unwrap();
                for nom in &cachees {
                    assert!(!t.contains(nom), "carte cachée nommée : {t}");
                }
                assert!((0.0..=100.0).contains(&o["chances"].as_f64().unwrap()));
            }
            assert!(s.poursuivre(true).unwrap());
            assert!(!s.reflechit());
            assert!(s.reflexion_json().is_null());
        } else {
            s.choisir(0).unwrap();
            n += 1;
        }
    }
    assert!(reflexions >= 3, "{reflexions} réflexions seulement");
}

#[test]
fn joue_maintenant_arrete_la_reflexion() {
    // Un niveau à 5 minutes par décision : sans le bouton, le test durerait.
    let mut s = session("reflechi-5m", 6);
    jusqu_a_une_reflexion(&mut s);
    attendre_le_debut(&s);
    assert!(s.jouer_maintenant());
    attendre_le_coup(&mut s);
}

#[test]
fn annuler_pendant_la_reflexion_l_interrompt() {
    let mut s = session("reflechi-5m", 7);
    jusqu_a_une_reflexion(&mut s);
    while s.annulables() == 0 {
        // Rien à annuler (le bot réfléchit avant votre premier choix) :
        // « Annuler » ne coupe pas sa réflexion.
        assert!(!s.annuler());
        assert!(s.reflechit());
        attendre_le_debut(&s);
        assert!(s.jouer_maintenant());
        attendre_le_coup(&mut s);
        jusqu_a_une_reflexion(&mut s);
    }
    let coups = s.coups().len();
    let debut = Instant::now();
    assert!(s.annuler());
    assert!(debut.elapsed() < Duration::from_secs(5));
    assert!(!s.reflechit());
    assert!(s.coups().len() < coups);
    // La partie reprend normalement : le bot réfléchit à nouveau après le
    // même choix, et « Joue maintenant » marche encore.
    jusqu_a_une_reflexion(&mut s);
    attendre_le_debut(&s);
    assert!(s.jouer_maintenant());
    attendre_le_coup(&mut s);
}

#[test]
fn les_autres_niveaux_ne_reflechissent_pas_a_part() {
    let mut s = session("b1", 8);
    for _ in 0..50 {
        if s.g.is_over() {
            break;
        }
        assert!(!s.reflechit());
        assert!(s.etat()["reflexion"].is_null());
        s.choisir(0).unwrap();
    }
}
