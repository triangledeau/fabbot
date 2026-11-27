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

/// L'état sans le « film » du dernier choix (le film montre le plateau tel
/// qu'il était à chaque pas : une carte alors visible a pu être cachée
/// depuis, par exemple une carte pitchée remise sous le deck). Le film est
/// construit pas à pas avec la même règle (`vue::visible`) ; voir
/// `le_film_rejoue_les_pas_du_bot_sans_rien_de_cache`.
fn sans_film(mut v: Value) -> Value {
    if let Some(o) = v.as_object_mut() {
        o.remove("film");
        o.remove("film_no");
    }
    v
}

fn verifier_rien_de_cache(s: &Session, contexte: &str) {
    let v = sans_film(s.etat());
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
        let mut v = sans_film(s.etat());
        let mut a = sans_film(attendu);
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
    // La page ne montre pas les numéros internes du moteur (« [34] »).
    assert!(!v["fiche"]
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
        sauvegarde: None,
    }
}

fn json(r: &fab_ui::serveur::Reponse) -> Value {
    serde_json::from_str(r.texte()).unwrap()
}

#[test]
fn routes_du_serveur() {
    let mut e = etat_serveur();
    let r = repondre("GET", "/", &mut e);
    assert_eq!(r.code, 200);
    assert!(r.texte().contains("<title>Arsenal"));

    let r = repondre("GET", "/api/etat", &mut e);
    assert_eq!(json(&r)["aucune_partie"], true);
    assert_eq!(repondre("POST", "/api/choix?i=0", &mut e).code, 400);

    let r = repondre("GET", "/api/decks", &mut e);
    assert_eq!(
        json(&r)["decks"].as_array().unwrap().len(),
        fab_cards::decks::noms().len()
    );

    let r = repondre(
        "POST",
        "/api/nouvelle?deck=florian&bot=iyslander&niveau=b1&graine=5",
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
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
    assert!(v["film_no"].is_number());

    assert_eq!(repondre("POST", "/api/annuler", &mut e).code, 400);
    assert_eq!(repondre("POST", "/api/choix?i=9999", &mut e).code, 400);
    assert_eq!(repondre("POST", "/api/choix", &mut e).code, 400);
    let r = repondre("POST", "/api/choix?i=0", &mut e);
    assert_eq!(r.code, 200);
    assert_eq!(json(&r)["annulables"], 1);
    let r = repondre("POST", "/api/annuler", &mut e);
    assert_eq!(r.code, 200);
    assert_eq!(sans_film(json(&r)), sans_film(premier.clone()));

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
        assert_eq!(r.code, 200, "{}", r.texte());
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
    assert_eq!(r.code, 200, "{}", r.texte());
    let v = json(&r);
    assert_eq!(v["preparation"]["etape"], "deck");
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

fn nouveau_match(e: &mut Etat, requete: &str) -> Value {
    let r = repondre("POST", requete, e);
    assert_eq!(r.code, 200, "{}", r.texte());
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
    assert_eq!(r.code, 200, "{}", r.texte());
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

// ---------- Arsenal, lot 0 : page en plusieurs fichiers ----------

/// Les fichiers du dossier `web/`, chemins relatifs avec des `/`.
fn fichiers_du_dossier(dossier: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
    for e in std::fs::read_dir(dossier).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            fichiers_du_dossier(&p, base, out);
        } else {
            let rel = p
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
}

#[test]
fn tous_les_fichiers_de_web_sont_servis_avec_le_bon_type() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
    let mut tous = Vec::new();
    fichiers_du_dossier(&base, &base, &mut tous);
    let mut e = etat_serveur();
    for f in &tous {
        let r = repondre("GET", &format!("/{f}"), &mut e);
        assert_eq!(
            r.code, 200,
            "{f} n'est pas servi : l'ajouter dans crates/ui/src/fichiers.rs"
        );
        assert_eq!(
            Some(r.type_contenu),
            fab_ui::fichiers::type_pour(f),
            "type de {f}"
        );
        assert_eq!(
            r.corps,
            std::fs::read(base.join(f)).unwrap(),
            "contenu de {f}"
        );
    }
    assert_eq!(
        tous.len(),
        fab_ui::fichiers::FICHIERS.len(),
        "fichier déclaré mais absent de web/"
    );
}

#[test]
fn la_page_ne_reference_que_des_fichiers_servis() {
    let mut e = etat_serveur();
    let page = repondre("GET", "/", &mut e);
    assert_eq!(page.type_contenu, "text/html; charset=utf-8");
    let html = page.texte().to_string();
    let mut vus = 0;
    for attr in ["src=\"", "href=\""] {
        for morceau in html.split(attr).skip(1) {
            let lien = morceau.split('"').next().unwrap();
            if lien.starts_with("http") || lien.starts_with('#') {
                continue;
            }
            vus += 1;
            assert_eq!(
                repondre("GET", &format!("/{lien}"), &mut e).code,
                200,
                "lien cassé : {lien}"
            );
        }
    }
    assert!(
        vus >= 6,
        "la page charge ses feuilles de style et ses scripts"
    );
    // Les volets de la navigation par # existent.
    for id in ["v-accueil", "v-partie"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "volet {id} absent");
    }
}

#[test]
fn un_fichier_inconnu_ou_un_chemin_qui_remonte_donne_404() {
    let mut e = etat_serveur();
    for c in [
        "/inconnu.js",
        "/../Cargo.toml",
        "/commun/../../Cargo.toml",
        "/api/inconnue",
    ] {
        assert_eq!(repondre("GET", c, &mut e).code, 404, "{c}");
    }
    assert_eq!(repondre("POST", "/commun/arsenal.css", &mut e).code, 404);
    // Une adresse avec paramètres sert quand même le fichier.
    assert_eq!(repondre("GET", "/partie/partie.js?v=2", &mut e).code, 200);
}

// ---------- Arsenal, lot 1 : images des cartes ----------

#[test]
fn chaque_carte_des_neuf_decks_a_une_image() {
    // Si ce test échoue après l'ajout d'un deck ou d'une carte : relancer
    // `python3 tools/images_cartes.py` (voir docs/nouveau-deck.md).
    let db = fab_cards::real::build();
    let mut sans = Vec::new();
    for nom in fab_ui::decks_jouables() {
        let pool = fab_ui::charger_pool(&db, &nom).unwrap();
        let mut ids = vec![pool.hero];
        ids.extend(pool.arena.iter().chain(&pool.deck).copied());
        for c in ids {
            let d = db.get(c);
            if fab_ui::images::image(d).is_none() {
                sans.push(format!("{} ({nom})", d.name));
            }
        }
    }
    assert!(sans.is_empty(), "cartes sans image : {sans:?}");
}

#[test]
fn l_etat_donne_l_image_des_cartes_visibles_mais_rien_des_cartes_cachees() {
    let s = Session::nouvelle(reglages("hala", "dorinthea", 3)).unwrap();
    let v = s.etat();
    assert!(v["vous"]["heros_image"].is_string(), "image du héros");
    assert!(
        v["bot"]["heros_image"].is_string(),
        "image du héros adverse"
    );
    let texte = v.to_string();
    assert!(
        texte.contains("\"image\":\""),
        "au moins une carte visible a son image"
    );
    // Une carte cachée ne porte que `cache` : ni nom, ni image.
    assert!(!texte.contains("\"cache\":true,\"image\""));
    assert!(!texte.contains("\"cache\":true,\"nom"));
}

// ---------- Arsenal, lot 2 : film des pas du moteur ----------

#[test]
fn le_film_rejoue_les_pas_du_bot_sans_rien_de_cache() {
    let connus = [
        "tour",
        "joue",
        "attaque",
        "defense",
        "touche",
        "bloque",
        "degats",
        "pitch",
        "chaine_fermee",
    ];
    let mut s = Session::nouvelle(reglages("dorinthea", "ira", 11)).unwrap();
    let mut vus = std::collections::HashSet::new();
    for n in 0..400 {
        if s.g.is_over() {
            break;
        }
        let k = s.g.pending_decision().unwrap().options.len();
        let no = s.etat()["film_no"].as_u64().unwrap();
        s.choisir(n % k).unwrap();
        let v = s.etat();
        assert_eq!(v["film_no"].as_u64().unwrap(), no + 1, "un film par choix");
        let film = v["film"].as_array().unwrap();
        assert!(
            !film.is_empty(),
            "le choix de l'humain fait au moins un pas"
        );
        for image in film {
            let p = &image["plateau"];
            assert!(p["bot"]["main"].is_null(), "main du bot dans le film");
            assert!(!p.to_string().contains("\"cache\":true,\"id\""));
            for e in image["evenements"].as_array().unwrap() {
                let t = e["t"].as_str().unwrap();
                assert!(connus.contains(&t), "événement inconnu {t}");
                vus.insert(t.to_string());
            }
        }
        // Le dernier plateau du film est le plateau affiché.
        let dernier = &film.last().unwrap()["plateau"];
        assert_eq!(dernier["vous"], v["vous"]);
        assert_eq!(dernier["bot"], v["bot"]);
    }
    for t in ["tour", "joue", "attaque", "pitch"] {
        assert!(
            vus.contains(t),
            "aucun événement « {t} » en toute une partie"
        );
    }
}

#[test]
fn les_options_disent_ce_qu_elles_font_pour_le_glisser_deposer() {
    let s = Session::nouvelle(reglages("hala", "dorinthea", 5)).unwrap();
    let v = s.etat();
    for o in v["decision"]["options"].as_array().unwrap() {
        let a = o["action"].as_str().unwrap();
        assert!(["jouer", "activer", "pitch", "defendre", "arsenal", "autre"].contains(&a));
        if o["passer"] == true {
            assert_eq!(a, "autre");
        }
    }
}

// ---------- Arsenal, lot 3 : sauvegarde et reprise ----------

/// L'état affiché, sans ce qui dépend de la façon d'y arriver (film).
fn affiche(v: &Value) -> Value {
    sans_film(v.clone())
}

/// Sauvegarde le match du serveur puis le recharge dans un autre serveur.
fn sauver_et_recharger(e: &mut Etat) -> (Value, Etat) {
    let r = repondre("GET", "/api/sauvegarde", e);
    assert_eq!(r.code, 200, "{}", r.texte());
    let sauvegarde = json(&r);
    assert_eq!(sauvegarde["format"], "arsenal-fabbot");
    let mut autre = etat_serveur();
    let r = fab_ui::serveur::repondre_avec_corps(
        "POST",
        "/api/charger",
        sauvegarde.to_string().as_bytes(),
        &mut autre,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    (json(&r), autre)
}

#[test]
fn une_partie_rechargee_reprend_exactement_au_meme_point() {
    let mut e = etat_serveur();
    let v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=ira&bot=bravo&niveau=b0&graine=31&manches=2",
    );
    // Pendant la préparation déjà.
    let (r, _) = sauver_et_recharger(&mut e);
    assert_eq!(affiche(&r), affiche(&v));
    let mut v = preparer_avec_la_proposition(&mut e, v);
    for n in 0..80 {
        if v["fini"] == true {
            break;
        }
        let k = v["decision"]["options"].as_array().unwrap().len();
        v = json(&repondre(
            "POST",
            &format!("/api/choix?i={}", (n * 7) % k),
            &mut e,
        ));
    }
    let (r, mut autre) = sauver_et_recharger(&mut e);
    assert_eq!(affiche(&r), affiche(&v), "même position après rechargement");
    assert_eq!(r["annulables"], v["annulables"], "on peut toujours annuler");
    // Et la suite se joue pareil : annuler, puis le même choix.
    let a = json(&repondre("POST", "/api/annuler", &mut e));
    let b = json(&repondre("POST", "/api/annuler", &mut autre));
    assert_eq!(affiche(&a), affiche(&b));
}

#[test]
fn un_match_recharge_apres_une_manche_finie_garde_le_score() {
    let mut e = etat_serveur();
    let v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=hala&bot=dorinthea&niveau=b0&graine=8&manches=2",
    );
    let mut v = preparer_avec_la_proposition(&mut e, v);
    while v["fini"] != true {
        v = json(&repondre("POST", "/api/choix?i=0", &mut e));
    }
    let (r, _) = sauver_et_recharger(&mut e);
    assert_eq!(affiche(&r), affiche(&v), "manche finie");
    let v = json(&repondre("POST", "/api/manche-suivante", &mut e));
    preparer_avec_la_proposition(&mut e, v);
    let v = json(&repondre("POST", "/api/choix?i=0", &mut e));
    let (r, _) = sauver_et_recharger(&mut e);
    assert_eq!(affiche(&r), affiche(&v), "deuxième manche");
    assert_eq!(r["match"]["jouees"], 1);
}

#[test]
fn une_sauvegarde_abimee_est_refusee_sans_toucher_au_match() {
    let mut e = etat_serveur();
    let v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=hala&bot=dorinthea&niveau=b0&graine=4&manches=1",
    );
    let v = preparer_avec_la_proposition(&mut e, v);
    let mut s = json(&repondre("GET", "/api/sauvegarde", &mut e));
    let charger = |e: &mut Etat, corps: &str| {
        fab_ui::serveur::repondre_avec_corps("POST", "/api/charger", corps.as_bytes(), e)
    };
    assert_eq!(charger(&mut e, "pas du json").code, 400);
    assert_eq!(charger(&mut e, "{\"format\":\"autre\"}").code, 400);
    // Un coup impossible (option 999) : refusé avec un message clair.
    let etapes = s["etapes"].as_array_mut().unwrap();
    let partie = etapes
        .iter_mut()
        .find(|x| x.get("partie").is_some())
        .unwrap();
    partie["partie"] = serde_json::json!([999]);
    let r = charger(&mut e, &s.to_string());
    assert_eq!(r.code, 400);
    assert!(r.texte().contains("incompatible"), "{}", r.texte());
    // Le match en cours n'a pas bougé.
    let apres = json(&repondre("GET", "/api/etat", &mut e));
    assert_eq!(affiche(&apres), affiche(&v));
}

#[test]
fn la_sauvegarde_automatique_ecrit_un_fichier_relu_a_l_identique() {
    let dossier = std::env::temp_dir().join(format!("fabbot-sauvegarde-{}", std::process::id()));
    let f = dossier.join("en-cours.json");
    let mut e = etat_serveur();
    let v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=levia&bot=arakni&niveau=b0&graine=12&manches=2",
    );
    let mut v = preparer_avec_la_proposition(&mut e, v);
    for _ in 0..10 {
        v = json(&repondre("POST", "/api/choix?i=0", &mut e));
    }
    fab_ui::sauvegarde::ecrire(&f, e.partie.as_ref().unwrap()).unwrap();
    let m = fab_ui::sauvegarde::lire(&f).unwrap();
    assert_eq!(affiche(&m.etat()), affiche(&v));
    std::fs::remove_dir_all(&dossier).ok();
}

// ---------- Arsenal : tapis de jeu ----------

#[test]
fn chaque_arme_et_equipement_dit_son_emplacement_sur_le_tapis() {
    let s = Session::nouvelle(reglages("hala", "dorinthea", 5)).unwrap();
    let v = s.etat();
    for qui in ["vous", "bot"] {
        let eq = v[qui]["equipement"].as_array().unwrap();
        assert!(!eq.is_empty(), "{qui} : pas d'équipement");
        let mut vus = Vec::new();
        for c in eq {
            let e = c["emplacement"].as_str().unwrap();
            assert!(
                ["arme", "tete", "torse", "bras", "jambes"].contains(&e),
                "{e}"
            );
            if e != "arme" {
                assert!(!vus.contains(&e), "{qui} : deux cartes en {e}");
                vus.push(e);
            }
        }
        assert!(
            eq.iter().any(|c| c["emplacement"] == "arme"),
            "{qui} : pas d'arme"
        );
    }
    // Hors armes et équipement, pas d'emplacement.
    for c in v["vous"]["main"].as_array().unwrap() {
        assert!(c["emplacement"].is_null());
    }
}

// ---------- Arsenal, lot 4c : plan conseillé avant la manche ----------

#[test]
fn le_plan_conseille_est_un_choix_valide_que_le_serveur_accepte() {
    let mut e = etat_serveur();
    let mut v = nouveau_match(
        &mut e,
        "/api/nouvelle?deck=hala&bot=dorinthea&niveau=b0&graine=6&manches=1",
    );
    if v["preparation"]["etape"] == "premier" {
        v = json(&repondre("POST", "/api/premier?qui=vous", &mut e));
    }
    let c = v["preparation"]["conseil"].clone();
    let arene: Vec<String> = c["arene"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.to_string())
        .collect();
    assert!(!arene.is_empty());
    let r = repondre(
        "POST",
        &format!("/api/arene?cartes={}", arene.join(",")),
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    let v = json(&r);
    let p = &v["preparation"];
    let deck = p["conseil"]["deck"].as_array().unwrap();
    let cartes = p["deck"].as_array().unwrap();
    assert_eq!(deck.len(), cartes.len());
    let total: u64 = deck.iter().map(|n| n.as_u64().unwrap()).sum();
    assert_eq!(total, p["taille_deck"].as_u64().unwrap());
    for (n, c) in deck.iter().zip(cartes) {
        assert!(n.as_u64().unwrap() <= c["max"].as_u64().unwrap());
    }
    let n: Vec<String> = deck.iter().map(|x| x.to_string()).collect();
    let r = repondre("POST", &format!("/api/deck?n={}", n.join(",")), &mut e);
    assert_eq!(r.code, 200, "{}", r.texte());
    // Pendant la partie, plus de conseil (ni de préparation).
    assert!(json(&r)["preparation"].is_null());
}

#[test]
fn la_carte_qui_pose_une_question_est_envoyee_avec_la_decision() {
    // Une question posée par l'effet d'une carte (« bannir… ? », cibles,
    // modes) porte la carte source, montrée dans le ruban et en grand.
    let mut vues = 0;
    for graine in 1..=6 {
        let mut s = Session::nouvelle(reglages("florian", "hala", graine)).unwrap();
        let mut n = 0u64;
        while !s.g.is_over() && n < 600 {
            let v = s.etat();
            let src = &v["decision"]["source"];
            if !src.is_null() {
                let nom = vue::source_decision(&s.g).expect("source attendue");
                assert_eq!(src["nom"], nom.as_str());
                assert!(src["carte_id"].as_u64().is_some());
                vues += 1;
            }
            let k = s.g.pending_decision().unwrap().options.len() as u64;
            s.choisir(((n * 7 + graine) % k) as usize).unwrap();
            n += 1;
        }
    }
    assert!(vues > 0, "aucune question de carte rencontrée");
}
