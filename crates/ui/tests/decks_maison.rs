//! Arsenal, lot 4a : collection Silver Age et decks maison (import d'une
//! liste FaBrary, vérification, partie avec un deck maison).

use fab_cards::decklist::Decklist;
use fab_cards::decks;
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

/// Un deck du projet réécrit comme l'export texte de FaBrary.
fn export_fabrary(nom: &str) -> String {
    let l = decks::liste(nom).unwrap();
    let mut t = format!(
        "Name: {nom} maison\nHero: {}\nFormat: Silver Age\n\nArena cards\n",
        l.hero
    );
    for e in &l.arena {
        t.push_str(&format!("{}x {}\n", e.count, e.name));
    }
    t.push_str("\nDeck cards\n");
    for e in &l.deck {
        t.push_str(&format!("{}x {} ({})\n", e.count, e.name, e.color));
    }
    t.push_str("\nMade with ❤️ at https://fabrary.net\n");
    t
}

/// Supprime le deck maison à la fin du test, même en cas d'échec.
struct Nettoyer(&'static str);
impl Drop for Nettoyer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(decks::dossier_maison().join(format!("{}.txt", self.0)));
    }
}

fn importer(e: &mut Etat, nom: &str, texte: &str) -> fab_ui::serveur::Reponse {
    repondre_avec_corps(
        "POST",
        &format!("/api/deck-maison/importer?nom={nom}"),
        texte.as_bytes(),
        e,
    )
}

#[test]
fn un_deck_fabrary_importe_est_enregistre_jouable_et_se_joue() {
    let nom = "test-import-hala";
    let _n = Nettoyer(nom);
    let mut e = etat_serveur();
    let r = importer(&mut e, nom, &export_fabrary("hala"));
    assert_eq!(r.code, 200, "{}", r.texte());
    let v = json(&r);
    assert_eq!(v["jouable"], true, "{v}");
    assert_eq!(v["heros"], "Hala");
    assert_eq!(
        v["arene"].as_u64().unwrap() + v["deck"].as_u64().unwrap(),
        55
    );
    // Le fichier écrit est la même liste que le deck du projet.
    let ecrit = Decklist::parse(
        &std::fs::read_to_string(decks::dossier_maison().join(format!("{nom}.txt"))).unwrap(),
    )
    .unwrap();
    let projet = decks::liste("hala").unwrap();
    assert_eq!(ecrit.arena, projet.arena);
    assert_eq!(ecrit.deck, projet.deck);
    // Proposé dans la page, et jouable contre le bot.
    let d = json(&repondre("GET", "/api/decks", &mut e));
    assert!(d["decks_maison"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x == nom));
    let r = repondre(
        "POST",
        &format!("/api/nouvelle?deck={nom}&bot=dorinthea&niveau=b0&graine=3&manches=1"),
        &mut e,
    );
    assert_eq!(r.code, 200, "{}", r.texte());
    assert_eq!(json(&r)["reglages"]["deck"], nom);
    // Relu par la route du deck.
    let v = json(&repondre(
        "GET",
        &format!("/api/deck-maison?nom={nom}"),
        &mut e,
    ));
    assert_eq!(v["jouable"], true);
    assert!(v["texte"].as_str().unwrap().contains("Hero: Hala"));
    let l = json(&repondre("GET", "/api/decks-maison", &mut e));
    assert!(l["decks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["nom"] == nom));
}

#[test]
fn un_deck_avec_une_carte_non_codee_est_garde_mais_pas_jouable() {
    let nom = "test-import-non-codee";
    let _n = Nettoyer(nom);
    let mut e = etat_serveur();
    // Une vraie carte Silver Age que le bot ne sait pas encore jouer.
    let texte = export_fabrary("hala")
        .replace("Deck cards\n", "Deck cards\n2x Command and Conquer (red)\n");
    let v = json(&importer(&mut e, nom, &texte));
    assert_eq!(v["jouable"], false, "{v}");
    let nc: Vec<&str> = v["non_codees"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert_eq!(nc, ["Command and Conquer (rouge)"]);
    let d = json(&repondre("GET", "/api/decks", &mut e));
    assert!(!d["decks_maison"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x == nom));
    assert!(decks::dossier_maison().join(format!("{nom}.txt")).exists());
}

#[test]
fn verifier_signale_le_format_sans_rien_enregistrer() {
    let mut e = etat_serveur();
    let texte =
        export_fabrary("hala").replace("2x Edict of Steel (red)", "3x Edict of Steel (red)");
    let r = repondre_avec_corps(
        "POST",
        "/api/deck-maison/verifier",
        texte.as_bytes(),
        &mut e,
    );
    let v = json(&r);
    assert_eq!(v["jouable"], false);
    assert!(
        v["problemes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p.as_str().unwrap().contains("3 copies")),
        "{v}"
    );
}

#[test]
fn un_nom_invalide_ou_deja_pris_est_refuse() {
    let mut e = etat_serveur();
    let t = export_fabrary("hala");
    for nom in ["", "Avec%20Espace", "hala", "accent%C3%A9"] {
        let r = importer(&mut e, nom, &t);
        assert_eq!(r.code, 400, "{nom} : {}", r.texte());
    }
    // Liste illisible : refusée, rien d'écrit.
    let r = importer(&mut e, "test-illisible", "Hero: Personne\n1x Rien\n");
    assert_eq!(r.code, 400);
    assert!(!decks::dossier_maison().join("test-illisible.txt").exists());
}

#[test]
fn la_collection_contient_les_cartes_des_neuf_decks_marquees_codees() {
    let mut e = etat_serveur();
    let v = json(&repondre("GET", "/api/collection", &mut e));
    let cartes = v["cartes"].as_array().unwrap();
    assert!(["fabcube", "codees"].contains(&v["source"].as_str().unwrap()));
    for d in decks::tous() {
        let l = decks::liste(&d.nom).unwrap();
        for e in &l.deck {
            let coul = match e.color {
                fab_cards::decklist::Color::Red => "rouge",
                fab_cards::decklist::Color::Yellow => "jaune",
                fab_cards::decklist::Color::Blue => "bleu",
            };
            let c = cartes
                .iter()
                .find(|c| c["nom"] == e.name.as_str() && c["couleur"] == coul)
                .unwrap_or_else(|| panic!("{} : {} ({coul}) absente", d.nom, e.name));
            assert_eq!(c["codee"], true, "{}", e.name);
        }
    }
}

// ---------- lot 4b : éditeur ----------

#[test]
fn un_deck_du_projet_se_lit_pour_etre_duplique() {
    let mut e = etat_serveur();
    let v = json(&repondre("GET", "/api/deck?nom=hala", &mut e));
    assert_eq!(v["maison"], false);
    assert_eq!(v["heros"], "Hala");
    let l = decks::liste("hala").unwrap();
    assert_eq!(v["liste"]["arene"].as_array().unwrap().len(), l.arena.len());
    assert_eq!(v["liste"]["deck"].as_array().unwrap().len(), l.deck.len());
    // Un nom approché ne suffit pas ici (on édite un deck précis).
    assert_eq!(repondre("GET", "/api/deck?nom=hal", &mut e).code, 404);
}

#[test]
fn archiver_restaurer_puis_supprimer_un_deck_maison() {
    let nom = "test-archives";
    let _n = Nettoyer(nom);
    let _a = NettoyerArchive(nom);
    let mut e = etat_serveur();
    assert_eq!(importer(&mut e, nom, &export_fabrary("hala")).code, 200);
    let post = |e: &mut Etat, quoi: &str| {
        repondre("POST", &format!("/api/deck-maison/{quoi}?nom={nom}"), e).code
    };
    // Un deck pas archivé ne se supprime pas.
    assert_eq!(post(&mut e, "supprimer"), 400);
    assert_eq!(post(&mut e, "archiver"), 200);
    let l = json(&repondre("GET", "/api/decks-maison", &mut e));
    assert!(!l["decks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["nom"] == nom));
    assert!(l["archives"].as_array().unwrap().iter().any(|d| d == nom));
    let d = json(&repondre("GET", "/api/decks", &mut e));
    assert!(!d["decks_maison"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x == nom));
    assert_eq!(post(&mut e, "restaurer"), 200);
    assert!(decks::dossier_maison().join(format!("{nom}.txt")).exists());
    assert_eq!(post(&mut e, "archiver"), 200);
    assert_eq!(post(&mut e, "supprimer"), 200);
    let l = json(&repondre("GET", "/api/decks-maison", &mut e));
    assert!(!l["archives"].as_array().unwrap().iter().any(|d| d == nom));
}

struct NettoyerArchive(&'static str);
impl Drop for NettoyerArchive {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(
            decks::dossier_maison()
                .join("archives")
                .join(format!("{}.txt", self.0)),
        );
    }
}

#[test]
fn la_collection_donne_l_emplacement_des_equipements_et_les_heros() {
    let mut e = etat_serveur();
    let v = json(&repondre("GET", "/api/collection", &mut e));
    let c = v["cartes"].as_array().unwrap();
    let trouve = |n: &str| c.iter().find(|x| x["nom"] == n).unwrap().clone();
    assert_eq!(trouve("Zenith Blade")["emplacement"], "Weapon");
    assert_eq!(trouve("Helm of Unity")["emplacement"], "Head");
    assert_eq!(trouve("Hala")["heros"], true);
    assert!(trouve("Hala")["emplacement"].is_null());
    assert!(c
        .iter()
        .filter(|x| x["couleur"] != "")
        .all(|x| x["emplacement"].is_null()));
}

#[test]
fn les_neuf_decks_du_projet_respectent_les_classes_de_leur_heros() {
    for d in decks::tous() {
        let l = decks::liste(&d.nom).unwrap();
        let v = fab_ui::api_decks::verifier(&l);
        assert!(
            !v["problemes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p.as_str().unwrap().contains("classe")),
            "{} : {v}",
            d.nom
        );
    }
}

#[test]
fn une_carte_d_une_autre_classe_est_signalee_mais_une_hybride_a_moitie_suffit() {
    let col = fab_ui::api_decks::collection();
    if col.source != "fabcube" {
        // Sans la base communautaire (CI), les types des cartes sont inconnus.
        return;
    }
    // Hala est Light Warrior (Young) : une carte Ninja n'est pas pour elle.
    assert_eq!(col.hors_classe("Hala", "Bleed Out"), Some(true));
    // Hybride Guardian / Warrior : le côté Warrior suffit.
    assert_eq!(col.hors_classe("Hala", "Agile Windup"), Some(false));
    // Iyslander (Elemental Wizard) a « Essence of Ice » : Ice Bolt est pour elle.
    assert_eq!(col.hors_classe("Iyslander", "Ice Bolt"), Some(false));
    assert_eq!(col.hors_classe("Iyslander", "Bleed Out"), Some(true));
    // Generic : pour tout le monde.
    assert_eq!(col.hors_classe("Hala", "Adrenaline Rush"), Some(false));
}

#[test]
fn l_ecran_jouer_recoit_les_neuf_decks_du_bot_avec_heros_et_source() {
    let mut e = etat_serveur();
    let v = json(&repondre("GET", "/api/decks-projet", &mut e));
    let d = v["decks"].as_array().unwrap();
    assert_eq!(d.len(), decks::noms().len());
    let ira = d.iter().find(|x| x["nom"] == "ira").unwrap();
    assert_eq!(ira["heros"], "Ira, Crimson Haze");
    assert_eq!(ira["source"], "Sunday Showdown: Sydney 2nd (Ira)");
    assert_eq!(ira["jouable"], true);
}

#[test]
fn la_collection_ne_propose_pas_les_cotes_d_une_carte_double_comme_cartes_a_part() {
    let mut e = etat_serveur();
    let v = json(&repondre("GET", "/api/collection", &mut e));
    let cartes = v["cartes"].as_array().unwrap();
    for cote in ["Arcane Seeds", "Life"] {
        assert!(
            !cartes.iter().any(|c| c["nom"] == cote),
            "« {cote} » seul ne doit pas être dans la collection"
        );
    }
}
