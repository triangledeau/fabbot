//! Arsenal, lot 9 : conseil du bot, barre d'évaluation, Bot contre Bot,
//! signalement de bug.

use fab_ui::serveur::{repondre, repondre_avec_corps, Etat};
use fab_ui::web::Reglages;
use serde_json::Value;

fn etat_serveur() -> Etat {
    Etat {
        partie: None,
        textes: None,
        depart: Reglages {
            deck: "hala".into(),
            deck_bot: "dorinthea".into(),
            niveau: "b0".into(),
            graine: 1,
            manches: 1,
        },
        sauvegarde: None,
    }
}

fn json(r: &fab_ui::serveur::Reponse) -> Value {
    serde_json::from_str(r.texte()).unwrap()
}

/// Un match Hala contre Dorinthea (B0), préparé avec la proposition, prêt
/// à jouer.
fn en_partie(e: &mut Etat, graine: u64) -> Value {
    let r = repondre(
        "POST",
        &format!("/api/nouvelle?deck=hala&bot=dorinthea&niveau=b0&graine={graine}&manches=1"),
        e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    let mut v = json(&r);
    if v["preparation"]["etape"] == "premier" {
        v = json(&repondre("POST", "/api/premier?qui=vous", e));
    }
    let cartes: Vec<String> = v["preparation"]["arene"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["propose"] == true)
        .map(|c| c["i"].to_string())
        .collect();
    let v = json(&repondre(
        "POST",
        &format!("/api/arene?cartes={}", cartes.join(",")),
        e,
    ));
    let n: Vec<String> = v["preparation"]["deck"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["propose"].to_string())
        .collect();
    let r = repondre("POST", &format!("/api/deck?n={}", n.join(",")), e);
    assert_eq!(r.code, 200, "{}", r.texte());
    json(&r)
}

#[test]
fn le_conseil_du_bot_est_une_option_proposee_et_ne_change_pas_si_on_redemande() {
    let mut e = etat_serveur();
    // Pas de match : pas de conseil.
    assert_eq!(repondre("GET", "/api/conseil", &mut e).code, 400);
    let v = en_partie(&mut e, 4);
    let d = &v["decision"];
    assert!(d.is_object(), "{v}");
    for niveau in ["b1", "b2"] {
        let c = json(&repondre(
            "GET",
            &format!("/api/conseil?niveau={niveau}"),
            &mut e,
        ));
        let i = c["i"].as_u64().expect("option conseillée");
        assert!(d["options"].as_array().unwrap().iter().any(|o| o["i"] == i));
        assert!(!c["texte"].as_str().unwrap().is_empty());
        let encore = json(&repondre(
            "GET",
            &format!("/api/conseil?niveau={niveau}"),
            &mut e,
        ));
        assert_eq!(encore["i"], c["i"]);
    }
    assert_eq!(repondre("GET", "/api/conseil?niveau=b9", &mut e).code, 400);
    // Demander un conseil ne joue rien.
    let apres = json(&repondre("GET", "/api/etat", &mut e));
    assert_eq!(apres["decision"], v["decision"]);
}

#[test]
fn la_barre_d_evaluation_donne_une_chance_entre_0_et_1() {
    let mut e = etat_serveur();
    en_partie(&mut e, 5);
    let v = json(&repondre("GET", "/api/evaluation", &mut e));
    let p = v["vous"].as_f64().expect("évaluation");
    assert!((0.0..=1.0).contains(&p), "{p}");
    // Même position : même chiffre.
    let w = json(&repondre("GET", "/api/evaluation", &mut e));
    assert_eq!(v, w);
}

#[test]
fn bot_contre_bot_joue_la_manche_jusqu_au_bout_et_reste_annulable() {
    let mut e = etat_serveur();
    en_partie(&mut e, 6);
    let r = repondre("POST", "/api/auto?niveau=b1", &mut e);
    assert_eq!(r.code, 200, "{}", r.texte());
    assert!(json(&r)["annulables"].as_u64().unwrap() >= 1);
    let mut fini = false;
    for _ in 0..5000 {
        let r = repondre("POST", "/api/auto?niveau=b0", &mut e);
        if r.code != 200 {
            break;
        }
        if json(&r)["fini"] == true {
            fini = true;
            break;
        }
    }
    assert!(fini, "la manche ne finit pas");
    let v = json(&repondre("GET", "/api/evaluation", &mut e));
    assert!([0.0, 0.5, 1.0].contains(&v["vous"].as_f64().unwrap()));
    assert_eq!(repondre("POST", "/api/auto", &mut e).code, 400);
}

#[test]
fn un_bug_signale_est_enregistre_avec_la_sauvegarde_du_match() {
    let dossier = std::env::temp_dir().join(format!("fabbot-bugs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    fab_ui::extras::utiliser_dossier_bugs(dossier.clone());
    let mut e = etat_serveur();
    en_partie(&mut e, 7);
    assert_eq!(
        repondre_avec_corps("POST", "/api/bug", b"   ", &mut e).code,
        400
    );
    let r = repondre_avec_corps(
        "POST",
        "/api/bug",
        "Le bot a bloqué avec une carte qui n'avait pas de défense.".as_bytes(),
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    let nom = json(&r)["nom"].as_str().unwrap().to_string();
    let b: Value = serde_json::from_str(
        &std::fs::read_to_string(dossier.join(format!("{nom}.json"))).unwrap(),
    )
    .unwrap();
    assert!(b["texte"].as_str().unwrap().starts_with("Le bot a bloqué"));
    // La sauvegarde jointe se recharge : on retrouve la partie signalée.
    let r = repondre_avec_corps(
        "POST",
        "/api/charger",
        b["sauvegarde"].to_string().as_bytes(),
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    let l = json(&repondre("GET", "/api/bugs", &mut e));
    assert!(l["bugs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["nom"] == nom));
    let _ = std::fs::remove_dir_all(&dossier);
}
