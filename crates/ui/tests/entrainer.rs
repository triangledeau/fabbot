//! Arsenal, lot 8 : « Entraîner » (tournoi dans la page, banc SPRT,
//! tâches d'apprentissage).

use fab_ui::api_tournoi::{llr, utiliser_dossier_tournois};
use fab_ui::serveur::{repondre, Etat};
use fab_ui::web::Reglages;
use serde_json::Value;
use std::time::{Duration, Instant};

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

/// Un seul test lance des tournois : l'état du tournoi est commun à tout
/// le programme (comme dans `fab-web`), et deux tests en parallèle se
/// gêneraient.
#[test]
fn un_tournoi_lance_depuis_la_page_se_joue_en_entier_et_va_dans_l_historique() {
    let dossier = std::env::temp_dir().join(format!("fabbot-tournois-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    utiliser_dossier_tournois(dossier.clone());
    let mut e = etat_serveur();
    // Un deck inconnu est refusé avant de lancer quoi que ce soit.
    let r = repondre("POST", "/api/tournoi/lancer?decks=hala,personne", &mut e);
    assert_eq!(r.code, 400, "{}", r.texte());

    let r = repondre(
        "POST",
        "/api/tournoi/lancer?decks=hala,ira&bot_a=b0&bot_b=b0&side_a=liste&side_b=liste&parties=2&threads=2&graine=5",
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    assert_eq!(json(&r)["total"], 8);
    // Un deuxième tournoi pendant le premier est refusé.
    let r2 = repondre("POST", "/api/tournoi/lancer?decks=hala", &mut e);
    let v = attendre_la_fin(&mut e);
    assert!(
        r2.code == 409 || r2.code == 200,
        "{} {}",
        r2.code,
        r2.texte()
    );
    if r2.code == 200 {
        // Le premier était déjà fini : c'est le second qu'on vient d'attendre.
        assert_eq!(v["total"], 2);
    } else {
        assert_eq!(v["fait"], 8, "{v}");
        let m = v["matrice"].as_array().unwrap();
        assert_eq!(m.len(), 2);
        let mut somme = 0;
        for ligne in m {
            for c in ligne.as_array().unwrap() {
                let n =
                    c["v"].as_u64().unwrap() + c["d"].as_u64().unwrap() + c["n"].as_u64().unwrap();
                assert_eq!(n + c["e"].as_u64().unwrap(), 2, "{c}");
                somme += n;
            }
        }
        assert_eq!(v["total_a"]["e"], 0, "{v}");
        assert_eq!(somme, 8);
        assert_eq!(v["classement"].as_array().unwrap().len(), 2);
        assert_eq!(v["reglage"]["heros"][0], "Hala");
    }
    // Historique : rangé, listé, relu, puis retiré.
    let nom = v["fichier"].as_str().expect("historique écrit").to_string();
    let h = json(&repondre("GET", "/api/tournois", &mut e));
    assert!(h["tournois"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["nom"] == nom.as_str()));
    let relu = json(&repondre(
        "GET",
        &format!("/api/tournoi/historique?nom={nom}"),
        &mut e,
    ));
    assert_eq!(relu["fait"], v["fait"]);
    assert_eq!(
        repondre("GET", "/api/tournoi/historique?nom=../x", &mut e).code,
        400
    );
    assert_eq!(
        repondre("POST", &format!("/api/tournoi/oublier?nom={nom}"), &mut e).code,
        200
    );
    assert!(!dossier.join(format!("{nom}.json")).exists());

    // Banc SPRT : B0 contre B0 n'a pas 300 points d'avance, le test le dit
    // et le tournoi s'arrête avant le plafond.
    let r = repondre(
        "POST",
        "/api/tournoi/lancer?decks=hala,ira&bot_a=b0&bot_b=b0&side_a=liste&side_b=liste&parties=500&threads=2&sprt=300",
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    let v = attendre_la_fin(&mut e);
    assert_eq!(v["sprt"]["conclusion"], "h0", "{}", v["sprt"]);
    assert!(v["fait"].as_u64().unwrap() < 2000);
    assert!(v["arret"].as_str().unwrap().starts_with("SPRT"));
    let _ = std::fs::remove_dir_all(&dossier);
}

fn attendre_la_fin(e: &mut Etat) -> Value {
    let debut = Instant::now();
    loop {
        let v = json(&repondre("GET", "/api/tournoi", e));
        if v["en_cours"] == false {
            return v;
        }
        assert!(
            debut.elapsed() < Duration::from_secs(300),
            "tournoi trop long"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn des_reglages_de_tournoi_faux_sont_refuses() {
    let mut e = etat_serveur();
    for q in [
        "decks=",
        "decks=hala&bot_a=b9",
        "decks=hala&side_a=nimporte",
        "decks=hala&parties=0",
        "decks=hala&parties=abc",
        "decks=hala&sprt=-5",
    ] {
        let r = repondre("POST", &format!("/api/tournoi/lancer?{q}"), &mut e);
        assert_eq!(r.code, 400, "{q} : {}", r.texte());
    }
}

#[test]
fn le_sprt_penche_du_cote_des_resultats() {
    // 70 % de points en 400 parties : nettement au-dessus de +0 Elo.
    assert!(llr(280.0, 400.0, 0.0, 0.0, 50.0) > 2.95);
    // 50 % : plus proche de +0 que de +50.
    assert!(llr(200.0, 400.0, 0.0, 0.0, 50.0) < -2.95);
    // Aucune partie : rien à conclure.
    assert_eq!(llr(0.0, 0.0, 0.0, 0.0, 50.0), 0.0);
}

#[test]
fn une_tache_inconnue_est_refusee() {
    let mut e = etat_serveur();
    let r = repondre("POST", "/api/tache/lancer?quoi=autre", &mut e);
    assert_eq!(r.code, 400, "{}", r.texte());
    let v = json(&repondre("GET", "/api/tache", &mut e));
    assert!(v["fab_sim"].is_boolean());
}
