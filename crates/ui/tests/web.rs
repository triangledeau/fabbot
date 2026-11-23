//! Tests de l'interface web : parties complètes par l'API, annulation,
//! information cachée jamais envoyée au navigateur.

use fab_engine::{ObjectId, PlayerId};
use fab_ui::serveur::{repondre, Etat};
use fab_ui::vue;
use fab_ui::web::{Reglages, Session};
use serde_json::Value;

fn reglages(deck: &str, bot: &str, graine: u64) -> Reglages {
    Reglages {
        deck: deck.into(),
        deck_bot: bot.into(),
        niveau: "b0".into(),
        graine,
    }
}

/// Tous les numéros de cartes présents dans le JSON (clés `id`, `carte`,
/// `heros_id`).
fn numeros(v: &Value, out: &mut Vec<u32>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if matches!(k.as_str(), "id" | "carte" | "heros_id") {
                    if let Some(n) = x.as_u64() {
                        out.push(n as u32);
                    }
                }
                numeros(x, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| numeros(x, out)),
        _ => {}
    }
}

fn verifier_rien_de_cache(s: &Session, contexte: &str) {
    let v = s.etat();
    let mut ns = Vec::new();
    numeros(&v, &mut ns);
    for n in ns {
        assert!(
            vue::visible(&s.g, PlayerId::P0, ObjectId(n)),
            "{contexte} : la carte cachée [{n}] est envoyée au navigateur"
        );
    }
    assert!(
        v["bot"]["main"].is_null(),
        "{contexte} : main du bot envoyée"
    );
    // Les cartes cachées ne portent ni nom ni numéro.
    let texte = v.to_string();
    assert!(!texte.contains("\"cache\":true,\"id\""));
}

#[test]
fn une_partie_complete_se_joue_par_l_api_en_choisissant_toujours_la_premiere_option() {
    for (moi, bot) in [
        ("hala", "dorinthea"),
        ("dromai", "arakni"),
        ("levia", "ira"),
    ] {
        for graine in 1..=2 {
            let mut s = Session::nouvelle(reglages(moi, bot, graine)).unwrap();
            let mut n = 0;
            while !s.g.is_over() {
                if n % 25 == 0 {
                    verifier_rien_de_cache(&s, &format!("{moi}/{bot}/{graine}, choix {n}"));
                }
                s.choisir(0).unwrap();
                n += 1;
                assert!(n < 50_000, "partie trop longue");
            }
            let v = s.etat();
            assert_eq!(v["fini"], true);
            assert!(v["decision"].is_null());
            assert!(v["resultat"].as_str().is_some());
            assert!(!s.journal().is_empty());
            assert!(s.choisir(0).is_err(), "plus de choix après la fin");
        }
    }
}

#[test]
fn rien_de_cache_n_est_envoye_quand_on_joue_au_hasard() {
    // Choix variés (pas toujours la première option) pour passer par la
    // défense, l'arsenal, le pitch…
    for graine in 1..=4 {
        let mut s = Session::nouvelle(reglages("arakni", "levia", graine)).unwrap();
        let mut n = 0u64;
        while !s.g.is_over() && n < 400 {
            verifier_rien_de_cache(&s, &format!("graine {graine}, choix {n}"));
            let k = s.g.pending_decision().unwrap().options.len() as u64;
            s.choisir(((n * 7 + graine) % k) as usize).unwrap();
            n += 1;
        }
    }
}

#[test]
fn annuler_revient_exactement_a_l_etat_affiche_avant_le_choix() {
    let mut s = Session::nouvelle(reglages("hala", "dorinthea", 7)).unwrap();
    let mut avant = Vec::new();
    for n in 0..60 {
        if s.g.is_over() {
            break;
        }
        avant.push(s.etat());
        let k = s.g.pending_decision().unwrap().options.len();
        s.choisir(n % k).unwrap();
    }
    assert_eq!(s.annulables(), avant.len());
    while let Some(attendu) = avant.pop() {
        assert!(s.annuler());
        let mut v = s.etat();
        let mut a = attendu;
        // Seul le nombre de choix annulables change… et il doit être égal.
        assert_eq!(v["annulables"], a["annulables"]);
        v["annulables"] = Value::Null;
        a["annulables"] = Value::Null;
        assert_eq!(v, a, "état différent après annulation");
    }
    assert!(!s.annuler(), "rien à annuler au début");
    assert_eq!(s.etat()["annulables"], 0);
}

#[test]
fn meme_graine_meme_partie() {
    let a = Session::nouvelle(reglages("ira", "bravo", 99)).unwrap();
    let b = Session::nouvelle(reglages("ira", "bravo", 99)).unwrap();
    assert_eq!(a.etat(), b.etat());
}

#[test]
fn fiche_d_une_carte_visible_mais_pas_d_une_carte_cachee() {
    let mut s = Session::nouvelle(reglages("hala", "dorinthea", 3)).unwrap();
    // Premier choix : qui commence ; ensuite les mains sont piochées.
    while s.g.player(PlayerId::P0).hand.is_empty() || s.g.player(PlayerId::P1).hand.is_empty() {
        s.choisir(0).unwrap();
    }
    let main = s.g.player(PlayerId::P0).hand.clone();
    let v = s.carte(main[0].0, None);
    assert!(v["erreur"].is_null(), "{v}");
    assert!(v["fiche"]
        .as_str()
        .unwrap()
        .contains(&format!("[{}]", main[0].0)));
    let main_bot = s.g.player(PlayerId::P1).hand.clone();
    let v = s.carte(main_bot[0].0, None);
    assert!(v["erreur"].is_string(), "carte du bot révélée : {v}");
    let v = s.carte(u32::MAX, None);
    assert!(v["erreur"].is_string());
}

fn etat_serveur() -> Etat {
    Etat {
        session: None,
        textes: None,
        depart: reglages("hala", "dorinthea", 1),
    }
}

fn json(r: &fab_ui::serveur::Reponse) -> Value {
    serde_json::from_str(&r.corps).unwrap()
}

#[test]
fn routes_du_serveur() {
    let mut e = etat_serveur();
    let r = repondre("GET", "/", &mut e);
    assert_eq!(r.code, 200);
    assert!(r.corps.contains("<title>fabbot</title>"));

    let r = repondre("GET", "/api/etat", &mut e);
    assert_eq!(json(&r)["aucune_partie"], true);
    assert_eq!(repondre("POST", "/api/choix?i=0", &mut e).code, 400);

    let r = repondre("GET", "/api/decks", &mut e);
    assert_eq!(json(&r)["decks"].as_array().unwrap().len(), 9);

    let r = repondre(
        "POST",
        "/api/nouvelle?deck=florian&bot=iyslander&niveau=b1&graine=5",
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.corps);
    let v = json(&r);
    assert_eq!(v["reglages"]["deck"], "florian");
    assert_eq!(v["reglages"]["niveau"], "b1");
    assert_eq!(v["reglages"]["graine"], "5");
    let premier = v.clone();

    assert_eq!(repondre("POST", "/api/annuler", &mut e).code, 400);
    assert_eq!(repondre("POST", "/api/choix?i=9999", &mut e).code, 400);
    assert_eq!(repondre("POST", "/api/choix", &mut e).code, 400);
    let r = repondre("POST", "/api/choix?i=0", &mut e);
    assert_eq!(r.code, 200);
    assert_eq!(json(&r)["annulables"], 1);
    let r = repondre("POST", "/api/annuler", &mut e);
    assert_eq!(r.code, 200);
    assert_eq!(json(&r), premier);

    let id = premier["vous"]["heros_id"].as_u64().unwrap();
    let r = repondre("GET", &format!("/api/carte?id={id}"), &mut e);
    assert_eq!(r.code, 200);
    assert!(json(&r)["fiche"].is_string());

    assert_eq!(
        repondre("POST", "/api/nouvelle?deck=inconnu", &mut e).code,
        400
    );
    assert_eq!(
        repondre("POST", "/api/nouvelle?niveau=b9", &mut e).code,
        400
    );
    assert_eq!(
        repondre("POST", "/api/nouvelle?graine=abc", &mut e).code,
        400
    );
    assert_eq!(repondre("GET", "/inconnu", &mut e).code, 404);
    assert_eq!(repondre("GET", "/api/choix?i=0", &mut e).code, 404);
}
