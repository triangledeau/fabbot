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
        manches: 2,
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
        partie: None,
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
    assert_eq!(v["phase"], "preparation");
    // Pas de partie avant la phase de side.
    assert_eq!(repondre("POST", "/api/choix?i=0", &mut e).code, 400);
    let v = preparer_avec_la_proposition(&mut e, v);
    assert_eq!(v["phase"], "partie");
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

/// Passe la préparation d'une manche en gardant la proposition par défaut
/// (en commençant si c'est à l'humain de choisir).
fn preparer_avec_la_proposition(e: &mut Etat, v: Value) -> Value {
    let mut v = v;
    if v["preparation"]["etape"] == "premier" {
        let r = repondre("POST", "/api/premier?qui=vous", e);
        assert_eq!(r.code, 200, "{}", r.corps);
        v = json(&r);
    }
    assert_eq!(v["preparation"]["etape"], "arene");
    let cartes: Vec<String> = v["preparation"]["arene"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["propose"] == true)
        .map(|c| c["i"].to_string())
        .collect();
    let r = repondre(
        "POST",
        &format!("/api/arene?cartes={}", cartes.join(",")),
        e,
    );
    assert_eq!(r.code, 200, "{}", r.corps);
    let v = json(&r);
    assert_eq!(v["preparation"]["etape"], "deck");
    let n: Vec<String> = v["preparation"]["deck"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["propose"].to_string())
        .collect();
    let r = repondre("POST", &format!("/api/deck?n={}", n.join(",")), e);
    assert_eq!(r.code, 200, "{}", r.corps);
    json(&r)
}

fn nouveau_match(e: &mut Etat, requete: &str) -> Value {
    let r = repondre("POST", requete, e);
    assert_eq!(r.code, 200, "{}", r.corps);
    json(&r)
}

// CR 4.1.4 : l'arène du bot n'est pas montrée avant le choix de l'humain ;
// CR 4.1.5 : elle l'est ensuite ; le deck du bot ne l'est jamais.
#[test]
fn l_arene_du_bot_n_est_montree_qu_apres_le_choix_de_l_humain() {
    let mut e = etat_serveur();
    let mut v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=ira&bot=arakni&niveau=b0&graine=4",
    );
    if v["preparation"]["etape"] == "premier" {
        v = json(&repondre("POST", "/api/premier?qui=bot", &mut e));
        assert_eq!(v["preparation"]["premier"], "bot");
    }
    assert_eq!(v["preparation"]["etape"], "arene");
    assert!(v["preparation"]["arene_bot"].is_null());
    // Texte d'une carte d'arène du bot : refusé tant qu'elle n'est pas présentée.
    let digits = fab_cards::real::build()
        .find_name("Danger Digits")
        .unwrap()
        .0;
    let r = repondre("GET", &format!("/api/carte-pool?id={digits}"), &mut e);
    assert!(json(&r)["erreur"].is_string());
    // Choix d'arène illégal : deux têtes.
    let arene = v["preparation"]["arene"].as_array().unwrap();
    let tetes: Vec<String> = arene
        .iter()
        .filter(|c| c["zone"] == "Tête")
        .map(|c| c["i"].to_string())
        .collect();
    assert!(tetes.len() >= 2);
    let r = repondre(
        "POST",
        &format!("/api/arene?cartes={}", tetes.join(",")),
        &mut e,
    );
    assert_eq!(r.code, 400);
    // Aucune carte d'arène : légal (« up to », CR 4.1.4a).
    let r = repondre("POST", "/api/arene?cartes=", &mut e);
    assert_eq!(r.code, 200, "{}", r.corps);
    let v = json(&r);
    let bot = v["preparation"]["arene_bot"].as_array().unwrap();
    assert!(!bot.is_empty());
    assert!(!v.to_string().contains("deck_bot\":["));
    // Deck de 39 cartes : refusé (TRP 7.4).
    let mut n: Vec<u64> = v["preparation"]["deck"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["propose"].as_u64().unwrap())
        .collect();
    let k = n.iter().position(|&x| x > 0).unwrap();
    n[k] -= 1;
    let texte: Vec<String> = n.iter().map(|x| x.to_string()).collect();
    let r = repondre("POST", &format!("/api/deck?n={}", texte.join(",")), &mut e);
    assert_eq!(r.code, 400);
    assert!(json(&r)["erreur"].as_str().unwrap().contains("40"));
    // Trop d'exemplaires d'une carte : refusé.
    n[k] += 1;
    let j = n.iter().position(|&x| x == 0).unwrap_or(k);
    n[j] = 3;
    let texte: Vec<String> = n.iter().map(|x| x.to_string()).collect();
    assert_eq!(
        repondre("POST", &format!("/api/deck?n={}", texte.join(",")), &mut e).code,
        400
    );
}

#[test]
fn un_match_en_deux_manches_gagnantes_se_joue_jusqu_au_bout() {
    let mut e = etat_serveur();
    let mut v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=hala&bot=dorinthea&niveau=b0&graine=8&manches=2",
    );
    assert_eq!(v["reglages"]["manches"], 2);
    let mut manches = 0;
    loop {
        v = preparer_avec_la_proposition(&mut e, v);
        manches += 1;
        // Choisir toujours la première option jusqu'à la fin de la manche.
        let mut n = 0;
        while v["fini"] != true {
            v = json(&repondre("POST", "/api/choix?i=0", &mut e));
            n += 1;
            assert!(n < 20_000, "manche trop longue");
        }
        assert_eq!(v["match"]["jouees"], manches);
        if v["match"]["gagnant"].is_string() {
            assert_eq!(v["manche_suivante"], false);
            assert_eq!(repondre("POST", "/api/manche-suivante", &mut e).code, 400);
            break;
        }
        assert_eq!(v["manche_suivante"], true);
        v = json(&repondre("POST", "/api/manche-suivante", &mut e));
        assert_eq!(v["phase"], "preparation");
        assert!(manches < 5);
    }
    let s = &v["match"];
    assert!(s["vous"] == 2 || s["bot"] == 2, "{s}");
    assert!(manches >= 2);
}

// Annuler le dernier choix d'une manche finie la fait reprendre, et le
// résultat n'est plus compté.
#[test]
fn annuler_apres_la_fin_d_une_manche_decompte_le_resultat() {
    let mut e = etat_serveur();
    let v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=hala&bot=dorinthea&niveau=b0&graine=2&manches=2",
    );
    let mut v = preparer_avec_la_proposition(&mut e, v);
    while v["fini"] != true {
        v = json(&repondre("POST", "/api/choix?i=0", &mut e));
    }
    assert_eq!(v["match"]["jouees"], 1);
    let v = json(&repondre("POST", "/api/annuler", &mut e));
    assert_eq!(v["fini"], false);
    assert_eq!(v["match"]["jouees"], 0);
}
