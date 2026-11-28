//! Démarrage de l'appli, copie des decks, reprise d'une partie, et le
//! serveur vu de la WebView (par de vraies connexions TCP locales).
//!
//! Ces tests ne changent ni le dossier courant ni les variables
//! d'environnement (ils tournent en parallèle avec les autres) : ils
//! appellent les étapes de `demarrer` une par une, avec des chemins complets.

use fab_android::{chemin_sauvegarde, donnees, preparer_etat, serveur};
use fab_ui::sauvegarde;
use fab_ui::serveur::Etat;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Un dossier vide, propre à chaque test.
fn dossier_vide(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fab-android-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Lance le serveur de l'appli sur un port libre ; rend son adresse.
fn lancer(etat: Etat) -> String {
    let ecoute = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let adresse = ecoute.local_addr().unwrap().to_string();
    let etat = Arc::new(Mutex::new(etat));
    std::thread::spawn(move || serveur::servir_toujours(ecoute, etat));
    adresse
}

/// Envoie une requête brute ; rend (code, corps).
fn requete(adresse: &str, methode: &str, chemin: &str, hote: &str) -> (u16, String) {
    let mut flux = TcpStream::connect(adresse).unwrap();
    write!(
        flux,
        "{methode} {chemin} HTTP/1.1\r\nHost: {hote}\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    let mut reponse = String::new();
    flux.read_to_string(&mut reponse).unwrap();
    let code = reponse
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let corps = reponse
        .split_once("\r\n\r\n")
        .map(|(_, c)| c.to_string())
        .unwrap_or_default();
    (code, corps)
}

#[test]
fn demarrage_dans_un_dossier_vide_cree_un_match() {
    let d = dossier_vide("vide");
    let etat = preparer_etat(&d, None);
    let m = etat.partie.as_ref().expect("un match doit être créé");
    assert_eq!(m.reglages.deck, "hala");
    assert_eq!(m.reglages.deck_bot, "dorinthea");
    assert_eq!(m.reglages.niveau, "b3");
    assert!(etat.textes.is_none());
    assert_eq!(etat.sauvegarde, Some(chemin_sauvegarde(&d)));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn les_decks_sont_copies_sans_ecraser_les_decks_maison() {
    let d = dossier_vide("decks").join("decks");
    std::fs::create_dir_all(d.join("maison")).unwrap();
    std::fs::write(d.join("maison/mon-deck.txt"), "mon deck à moi").unwrap();
    // Un deck retiré du projet et un deck du projet abîmé.
    std::fs::write(d.join("99_ancien.txt"), "ancien").unwrap();
    std::fs::write(d.join("02_hala.txt"), "abîmé").unwrap();

    donnees::installer_decks(&d).unwrap();

    for (nom, contenu) in donnees::DECKS {
        assert_eq!(std::fs::read(d.join(nom)).unwrap(), *contenu, "{nom}");
    }
    assert!(!d.join("99_ancien.txt").exists());
    assert_eq!(
        std::fs::read_to_string(d.join("maison/mon-deck.txt")).unwrap(),
        "mon deck à moi"
    );
    // Un second démarrage ne change rien.
    donnees::installer_decks(&d).unwrap();
    assert!(d.join("maison/mon-deck.txt").exists());
    let _ = std::fs::remove_dir_all(d.parent().unwrap());
}

#[test]
fn une_partie_sauvegardee_est_reprise_au_demarrage() {
    let d = dossier_vide("reprise");
    let mut reglages = fab_android::reglages_de_depart();
    reglages.deck = "ira".into();
    reglages.niveau = "b1".into();
    reglages.graine = 4242;
    let m = fab_ui::match_web::MatchWeb::nouveau(reglages).unwrap();
    let f = chemin_sauvegarde(&d);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    sauvegarde::ecrire(&f, &m).unwrap();

    let etat = preparer_etat(&d, None);
    let repris = etat.partie.expect("la partie doit être reprise");
    assert_eq!(repris.reglages.graine, 4242);
    assert_eq!(repris.reglages.deck, "ira");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn les_textes_des_cartes_sont_lus_dans_le_dossier_de_l_appli() {
    let d = dossier_vide("textes");
    let f = fab_android::chemin_textes(&d);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    // Un fichier vide : lu, même s'il ne contient aucune carte.
    std::fs::write(&f, "").unwrap();
    assert!(preparer_etat(&d, Some(&f)).textes.is_some());
    let absent = d.join("absent.csv");
    assert!(preparer_etat(&d, Some(&absent)).textes.is_none());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn la_page_recoit_les_deux_balises_du_telephone() {
    let d = dossier_vide("page");
    let adresse = lancer(preparer_etat(&d, None));
    for chemin in ["/", "/index.html"] {
        let (code, page) = requete(&adresse, "GET", chemin, "127.0.0.1");
        assert_eq!(code, 200);
        let balises = page.find(serveur::BALISES).expect("balises absentes");
        assert!(balises < page.find("</head>").unwrap());
        assert_eq!(page.matches("/android/mobile.js").count(), 1);
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn injection_sans_balise_head_ajoute_a_la_fin() {
    assert_eq!(
        serveur::injecter("<p>x</p>"),
        format!("<p>x</p>{}", serveur::BALISES)
    );
}

#[test]
fn mobile_css_et_mobile_js_sont_servis() {
    let d = dossier_vide("mobile");
    let adresse = lancer(preparer_etat(&d, None));
    let (code, css) = requete(&adresse, "GET", "/android/mobile.css", "localhost:38080");
    assert_eq!(code, 200);
    assert!(css.contains("html.android"));
    let (code, js) = requete(&adresse, "GET", "/android/mobile.js", "localhost:38080");
    assert_eq!(code, 200);
    assert!(js.contains("classList.add(\"android\")"));
    // Le reste de la page vient de fab-ui, comme sur PC.
    let (code, _) = requete(&adresse, "GET", "/commun/arsenal.css", "127.0.0.1");
    assert_eq!(code, 200);
    let (code, etat) = requete(&adresse, "GET", "/api/etat", "127.0.0.1");
    assert_eq!(code, 200);
    assert!(etat.starts_with('{'));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn un_hote_non_local_est_refuse() {
    let d = dossier_vide("hote");
    let adresse = lancer(preparer_etat(&d, None));
    let (code, corps) = requete(&adresse, "GET", "/", "exemple.com");
    assert_eq!(code, 403);
    assert!(corps.contains("accès réservé"));
    let (code, _) = requete(&adresse, "GET", "/android/mobile.css", "");
    assert_eq!(code, 403);
    assert!(!serveur::hote_local("127.0.0.1.exemple.com"));
    assert!(serveur::hote_local("[::1]:38080"));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn un_post_reussi_ecrit_la_sauvegarde() {
    let d = dossier_vide("post");
    let f = chemin_sauvegarde(&d);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    let adresse = lancer(preparer_etat(&d, None));
    let (code, _) = requete(
        &adresse,
        "POST",
        "/api/nouvelle?deck=bravo&bot=hala&niveau=b1&graine=77&manches=1",
        "127.0.0.1",
    );
    assert_eq!(code, 200);
    let relu = sauvegarde::lire(&f).expect("sauvegarde écrite après le POST");
    assert_eq!(relu.reglages.graine, 77);
    assert_eq!(relu.reglages.deck, "bravo");
    let _ = std::fs::remove_dir_all(&d);
}
