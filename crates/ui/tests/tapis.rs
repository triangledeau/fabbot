//! Écran de partie repris de l'Arsenal JS : défense préparée carte par
//! carte puis envoyée d'un coup (`/api/choix-cartes`), pile détaillée,
//! effets en cours, chiffres de fin de partie, abandon du match.

use fab_ui::serveur::{repondre, Etat};
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

/// Avance (le bot joue pour l'humain) jusqu'à une décision de ce genre.
fn jusqu_a(e: &mut Etat, genre: &str) -> Option<Value> {
    for _ in 0..3000 {
        let v = json(&repondre("GET", "/api/etat", e));
        if v["fini"] == true {
            return None;
        }
        if v["decision"]["genre"] == genre {
            return Some(v);
        }
        let r = repondre("POST", "/api/auto?niveau=b0", e);
        assert_eq!(r.code, 200, "{}", r.texte());
    }
    None
}

#[test]
fn l_etat_donne_pile_detaillee_priorite_effets_et_chiffres() {
    let mut e = etat_serveur();
    let v = en_partie(&mut e, 3);
    assert!(v["couches"].is_array());
    assert!(v["vous"]["effets"].is_array() && v["bot"]["effets"].is_array());
    assert!(v["stats"]["infliges"].is_u64() && v["stats"]["blocs_main"].is_u64());
    assert!(v["phase_action"].is_boolean());
    assert!(v["priorite"].is_null() || v["priorite"] == "vous" || v["priorite"] == "bot");
}

#[test]
fn une_defense_preparee_part_d_un_coup_et_s_annule_d_un_coup() {
    let mut e = etat_serveur();
    en_partie(&mut e, 5);
    let mut trouve = false;
    for _ in 0..40 {
        let Some(v) = jusqu_a(&mut e, "defense") else {
            break;
        };
        let cartes: Vec<u64> = v["decision"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|o| o["passer"] == false && o["carte"].is_u64())
            .filter_map(|o| o["carte"].as_u64())
            .take(2)
            .collect();
        if cartes.len() < 2 {
            // Pas assez de bloqueurs ici : on ne bloque pas et on continue.
            let r = repondre("POST", "/api/choix-cartes?cartes=&passer=1", &mut e);
            assert_eq!(r.code, 200, "{}", r.texte());
            continue;
        }
        let avant = v["annulables"].as_u64().unwrap();
        let liste = cartes
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let r = repondre(
            "POST",
            &format!("/api/choix-cartes?cartes={liste}&passer=1"),
            &mut e,
        );
        assert_eq!(r.code, 200, "{}", r.texte());
        let apres = json(&r);
        assert_eq!(
            apres["annulables"].as_u64().unwrap(),
            avant + 1,
            "un seul pas d'annulation"
        );
        assert!(
            apres["stats"]["blocs_main"].as_u64().unwrap()
                + apres["stats"]["blocs_autres"].as_u64().unwrap()
                >= 2
        );
        // Annuler une fois revient à la même défense, rien de posé.
        let r = repondre("POST", "/api/annuler", &mut e);
        assert_eq!(r.code, 200, "{}", r.texte());
        let w = json(&r);
        assert_eq!(w["decision"]["genre"], "defense");
        assert_eq!(w["decision"]["options"], v["decision"]["options"]);
        trouve = true;
        break;
    }
    assert!(trouve, "aucune défense à deux cartes rencontrée");
}

#[test]
fn une_carte_hors_des_choix_est_refusee_sans_rien_changer() {
    let mut e = etat_serveur();
    let v = en_partie(&mut e, 4);
    let avant = v["annulables"].as_u64().unwrap();
    let r = repondre("POST", "/api/choix-cartes?cartes=99999&passer=0", &mut e);
    assert_eq!(r.code, 400);
    let w = json(&repondre("GET", "/api/etat", &mut e));
    assert_eq!(w["annulables"].as_u64().unwrap(), avant);
    assert_eq!(w["decision"], v["decision"]);
}

#[test]
fn abandonner_oublie_le_match_et_sa_sauvegarde() {
    let mut e = etat_serveur();
    let dossier = std::env::temp_dir().join(format!("fab-abandon-{}", std::process::id()));
    std::fs::create_dir_all(&dossier).unwrap();
    let f = dossier.join("match.json");
    std::fs::write(&f, "{}").unwrap();
    e.sauvegarde = Some(f.clone());
    en_partie(&mut e, 2);
    let r = repondre("POST", "/api/abandon", &mut e);
    assert_eq!(r.code, 200, "{}", r.texte());
    assert_eq!(
        json(&repondre("GET", "/api/etat", &mut e))["aucune_partie"],
        true
    );
    assert!(!f.exists());
    let _ = std::fs::remove_dir_all(&dossier);
}

#[test]
fn les_chiffres_de_fin_comptent_les_degats_subis() {
    let mut e = etat_serveur();
    en_partie(&mut e, 8);
    let mut v = Value::Null;
    for _ in 0..5000 {
        let r = repondre("POST", "/api/auto?niveau=b1", &mut e);
        if r.code != 200 {
            break;
        }
        v = json(&r);
        if v["fini"] == true {
            break;
        }
    }
    assert_eq!(v["fini"], true);
    let s = &v["stats"];
    if v["bot"]["vie"].as_i64().unwrap() <= 0 {
        assert!(s["infliges"].as_u64().unwrap() > 0, "{s}");
    }
    if v["vous"]["vie"].as_i64().unwrap() <= 0 {
        assert!(s["subis"].as_u64().unwrap() > 0, "{s}");
    }
}
