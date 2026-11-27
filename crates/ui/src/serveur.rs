//! Un tout petit serveur HTTP local (sans bibliothèque extérieure) pour
//! jouer dans le navigateur.
//!
//! Il n'écoute que sur `127.0.0.1` (votre ordinateur) et refuse les
//! requêtes dont l'en-tête `Host` n'est pas local. Les routes :
//!
//! | Requête | Réponse |
//! |---|---|
//! | `GET /` | la page Arsenal (accueil et partie, navigation par `#`) |
//! | `GET /commun/arsenal.css`… | les autres fichiers de la page ([`crate::fichiers`]) |
//! | `GET /api/outils` | outils locaux présents (`proxies` : Proxies de Rathe, [`crate::fichiers::chemin_proxies`]) |
//! | `GET /outils/dos-de-carte` | le vrai dos de carte, s'il est dans `data/outils/` ([`crate::fichiers::dos_de_carte`]) |
//! | `GET /outils/proxies-de-rathe.html` | l'outil d'impression complet, s'il est dans `data/outils/` |
//! | `GET /api/etat` | l'état vu par l'humain (JSON) |
//! | `GET /api/decks` | decks et niveaux de bot proposés |
//! | `GET /api/carte?id=12` | fiche et texte officiel de la carte `[12]` |
//! | `POST /api/choix?i=3` | choisir l'option 3 (à partir de 0) |
//! | `POST /api/choix-cartes?cartes=12,15&passer=1` | plusieurs choix d'un coup (défense préparée…), annulés ensemble |
//! | `POST /api/abandon` | oublier le match en cours (et sa sauvegarde automatique) |
//! | `POST /api/annuler` | annuler le dernier choix |
//! | `POST /api/nouvelle?deck=hala&bot=dorinthea&niveau=b1&graine=42&manches=2` | nouveau match (`manches` : manches à gagner) |
//! | `POST /api/premier?qui=vous` | qui commence la manche (`vous` ou `bot`, CR 4.1.3) |
//! | `POST /api/arene?cartes=0,2,5` | cartes d'arène choisies (positions dans le card-pool, CR 4.1.4) |
//! | `POST /api/deck?n=2,2,1,…` | exemplaires de chaque carte de deck (CR 4.1.6) |
//! | `POST /api/manche-suivante` | préparer la manche suivante |
//! | `GET /api/carte-pool?id=7` | texte d'une carte du card-pool (écran de préparation) |
//! | `GET /api/sauvegarde` | la sauvegarde du match en cours (JSON, [`crate::sauvegarde`]) |
//! | `POST /api/charger` (corps : une sauvegarde) | reprend le match d'une sauvegarde |
//! | `/api/collection`, `/api/decks-projet`, `/api/decks-maison`, `/api/deck-maison…` | decks ([`crate::api_decks`]) |
//! | `/api/tournoi…`, `/api/tache…` | entraîner ([`crate::api_tournoi`]) |
//! | `/api/apprendre/…` | scènes du didacticiel ([`crate::api_apprendre`]) |
//! | `/api/conseil`, `/api/evaluation`, `/api/auto`, `/api/bug(s)` | extras de la partie ([`crate::extras`]) |
//!
//! Après chaque requête `POST` réussie, le match est écrit dans le fichier
//! de sauvegarde ([`Etat::sauvegarde`]), s'il y en a un.

use crate::decks_jouables;
use crate::extras;
use crate::fichiers;
use crate::match_web::{Etape, MatchWeb};
use crate::sauvegarde;
use crate::textes::Textes;
use crate::web::Reglages;
use fab_ai::BOTS;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Une réponse : code HTTP, type de contenu, corps.
#[derive(Debug)]
pub struct Reponse {
    pub code: u16,
    pub type_contenu: &'static str,
    pub corps: Vec<u8>,
}

impl Reponse {
    /// Le corps lu comme du texte (JSON, HTML…).
    pub fn texte(&self) -> &str {
        std::str::from_utf8(&self.corps).unwrap_or("")
    }

    fn json(v: Value) -> Reponse {
        Reponse {
            code: 200,
            type_contenu: "application/json; charset=utf-8",
            corps: v.to_string().into_bytes(),
        }
    }

    fn erreur(code: u16, message: impl Into<String>) -> Reponse {
        Reponse {
            code,
            type_contenu: "application/json; charset=utf-8",
            corps: json!({ "erreur": message.into() }).to_string().into_bytes(),
        }
    }
}

/// Ce que le serveur garde entre deux requêtes.
pub struct Etat {
    pub partie: Option<MatchWeb>,
    pub textes: Option<Textes>,
    /// Réglages de la partie lancée au démarrage.
    pub depart: Reglages,
    /// Fichier où le match est écrit après chaque coup (`None` : pas de
    /// sauvegarde automatique, par exemple dans les tests).
    pub sauvegarde: Option<std::path::PathBuf>,
}

/// Décode `%C3%A9` et `+` dans un paramètre d'URL.
fn decoder(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                let hex = std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Les paramètres `?a=1&b=2` d'un chemin.
fn parametres(requete: &str) -> (&str, Vec<(String, String)>) {
    let (chemin, q) = requete.split_once('?').unwrap_or((requete, ""));
    let ps = q
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decoder(k), decoder(v))
        })
        .collect();
    (chemin, ps)
}

/// Graine tirée de l'heure (quand on n'en donne pas).
pub fn graine_horloge() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
}

/// Traite une requête sans corps (sans réseau : testable directement).
pub fn repondre(methode: &str, requete: &str, etat: &mut Etat) -> Reponse {
    repondre_avec_corps(methode, requete, &[], etat)
}

/// Traite une requête ; `corps` sert à `POST /api/charger` et aux routes
/// des decks maison.
pub fn repondre_avec_corps(methode: &str, requete: &str, corps: &[u8], etat: &mut Etat) -> Reponse {
    let (chemin, ps) = parametres(requete);
    let param = |k: &str| ps.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let lot = crate::api_decks::repondre(methode, chemin, &param, corps, etat.textes.as_ref())
        .or_else(|| crate::api_tournoi::repondre(methode, chemin, &param))
        .or_else(|| crate::api_apprendre::repondre(methode, chemin, &param, etat.textes.as_ref()));
    if let Some((code, v)) = lot {
        return if code == 200 {
            Reponse::json(v)
        } else {
            Reponse {
                code,
                ..Reponse::json(v)
            }
        };
    }
    match (methode, chemin) {
        ("GET", "/api/decks") => Reponse::json(json!({
            "decks": decks_jouables(),
            "decks_maison": crate::api_decks::maison_jouables(),
            "niveaux": BOTS,
            "noms": crate::noms_affiches(),
            "textes": etat.textes.is_some(),
        })),
        ("GET", "/api/etat") => match &etat.partie {
            Some(m) => Reponse::json(m.etat()),
            None => Reponse::json(json!({ "aucune_partie": true })),
        },
        ("GET", "/api/carte") => {
            let Some(id) = param("id").and_then(|v| v.parse::<u32>().ok()) else {
                return Reponse::erreur(400, "paramètre id manquant");
            };
            match etat.partie.as_ref().and_then(|m| m.session.as_ref()) {
                Some(s) => Reponse::json(s.carte(id, etat.textes.as_ref())),
                None => Reponse::erreur(400, "aucune partie en cours"),
            }
        }
        ("GET", "/api/carte-pool") => {
            let Some(id) = param("id").and_then(|v| v.parse::<u32>().ok()) else {
                return Reponse::erreur(400, "paramètre id manquant");
            };
            match &etat.partie {
                Some(m) => Reponse::json(m.carte_pool(id, etat.textes.as_ref())),
                None => Reponse::erreur(400, "aucun match en cours"),
            }
        }
        ("POST", "/api/choix") => {
            let Some(i) = param("i").and_then(|v| v.parse::<usize>().ok()) else {
                return Reponse::erreur(400, "paramètre i manquant");
            };
            let Some(m) = etat.partie.as_mut() else {
                return Reponse::erreur(400, "aucune partie en cours");
            };
            let Some(s) = m.session.as_mut().filter(|_| m.etape == Etape::Partie) else {
                return Reponse::erreur(400, "la manche n'a pas commencé");
            };
            let r = s.choisir(i);
            m.apres_coup();
            match r {
                Ok(()) => Reponse::json(m.etat()),
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("POST", "/api/choix-cartes") => {
            let cartes: Result<Vec<u32>, _> = param("cartes")
                .unwrap_or_default()
                .split(',')
                .filter(|x| !x.is_empty())
                .map(|x| x.parse::<u32>())
                .collect();
            let Ok(cartes) = cartes else {
                return Reponse::erreur(400, "paramètre cartes invalide");
            };
            let passer = param("passer").is_some_and(|v| v == "1");
            let Some(m) = etat.partie.as_mut() else {
                return Reponse::erreur(400, "aucune partie en cours");
            };
            let Some(s) = m.session.as_mut().filter(|_| m.etape == Etape::Partie) else {
                return Reponse::erreur(400, "la manche n'a pas commencé");
            };
            let r = s.choisir_cartes(&cartes, passer);
            m.apres_coup();
            match r {
                Ok(()) => Reponse::json(m.etat()),
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("POST", "/api/annuler") => {
            let Some(m) = etat.partie.as_mut() else {
                return Reponse::erreur(400, "aucune partie en cours");
            };
            let Some(s) = m.session.as_mut().filter(|_| m.etape == Etape::Partie) else {
                return Reponse::erreur(400, "la manche n'a pas commencé");
            };
            if s.annuler() {
                m.apres_coup();
                Reponse::json(m.etat())
            } else {
                Reponse::erreur(400, "rien à annuler")
            }
        }
        ("POST", "/api/premier")
        | ("POST", "/api/arene")
        | ("POST", "/api/deck")
        | ("POST", "/api/manche-suivante") => {
            let Some(m) = etat.partie.as_mut() else {
                return Reponse::erreur(400, "aucun match en cours");
            };
            let nombres = |k: &str| -> Result<Vec<usize>, String> {
                let v = param(k).unwrap_or_default();
                v.split(',')
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        x.parse::<usize>()
                            .map_err(|_| format!("nombre invalide : {x}"))
                    })
                    .collect()
            };
            let r = match chemin {
                "/api/premier" => m.premier_joueur(&param("qui").unwrap_or_default()),
                "/api/arene" => nombres("cartes").and_then(|v| m.arene(&v)),
                "/api/deck" => nombres("n").and_then(|v| m.deck(&v)),
                _ => m.manche_suivante(),
            };
            match r {
                Ok(()) => Reponse::json(m.etat()),
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("GET", "/api/conseil") | ("GET", "/api/evaluation") => {
            let Some(m) = etat.partie.as_ref() else {
                return Reponse::erreur(400, "aucun match en cours");
            };
            let s = match extras::session(m) {
                Ok(s) => s,
                Err(e) => return Reponse::erreur(400, e),
            };
            if chemin == "/api/evaluation" {
                return Reponse::json(json!({ "vous": extras::evaluation(s) }));
            }
            let niveau = param("niveau").unwrap_or_else(|| "b3".into());
            match extras::conseil(s, &niveau) {
                Ok((i, texte)) => {
                    Reponse::json(json!({ "i": i, "texte": texte, "niveau": niveau }))
                }
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("POST", "/api/auto") => {
            let Some(m) = etat.partie.as_mut() else {
                return Reponse::erreur(400, "aucun match en cours");
            };
            let niveau = param("niveau").unwrap_or_else(|| "b3".into());
            let i = match extras::session(m).and_then(|s| extras::conseil(s, &niveau)) {
                Ok((i, _)) => i,
                Err(e) => return Reponse::erreur(400, e),
            };
            let r = m.session.as_mut().map(|s| s.choisir(i));
            m.apres_coup();
            match r {
                Some(Ok(())) => Reponse::json(m.etat()),
                Some(Err(e)) => Reponse::erreur(400, e),
                None => Reponse::erreur(400, "la manche n'a pas commencé"),
            }
        }
        ("POST", "/api/bug") => {
            match extras::signaler(&String::from_utf8_lossy(corps), etat.partie.as_ref()) {
                Ok(nom) => Reponse::json(json!({ "nom": nom, "dossier": extras::dossier_bugs() })),
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("GET", "/api/bugs") => Reponse::json(json!({
            "bugs": extras::bugs(),
            "dossier": extras::dossier_bugs(),
        })),
        ("GET", "/api/sauvegarde") => match &etat.partie {
            Some(m) => Reponse::json(sauvegarde::exporter(m)),
            None => Reponse::erreur(400, "aucun match en cours"),
        },
        ("POST", "/api/charger") => {
            let v: Value = match serde_json::from_slice(corps) {
                Ok(v) => v,
                Err(e) => return Reponse::erreur(400, format!("fichier illisible : {e}")),
            };
            match sauvegarde::importer(&v) {
                Ok(m) => {
                    let v = m.etat();
                    etat.partie = Some(m);
                    Reponse::json(v)
                }
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("POST", "/api/abandon") => {
            // « Abandonner et revenir au choix des decks » (Arsenal JS) : le
            // match est oublié, sa sauvegarde automatique aussi.
            etat.partie = None;
            if let Some(f) = &etat.sauvegarde {
                if f.exists() {
                    if let Err(e) = std::fs::remove_file(f) {
                        return Reponse::erreur(500, format!("sauvegarde non effacée : {e}"));
                    }
                }
            }
            Reponse::json(json!({ "aucune_partie": true }))
        }
        ("POST", "/api/nouvelle") => {
            let graine = match param("graine").filter(|g| !g.trim().is_empty()) {
                None => graine_horloge(),
                Some(g) => match g.trim().parse::<u64>() {
                    Ok(g) => g,
                    Err(_) => return Reponse::erreur(400, "graine invalide (un nombre entier)"),
                },
            };
            let d = &etat.depart;
            let manches = match param("manches").filter(|g| !g.trim().is_empty()) {
                None => d.manches,
                Some(n) => match n.trim().parse::<u8>() {
                    Ok(n) => n,
                    Err(_) => return Reponse::erreur(400, "manches : 1, 2 ou 3"),
                },
            };
            let r = Reglages {
                deck: param("deck").unwrap_or_else(|| d.deck.clone()),
                deck_bot: param("bot").unwrap_or_else(|| d.deck_bot.clone()),
                niveau: param("niveau").unwrap_or_else(|| d.niveau.clone()),
                graine,
                manches,
            };
            match MatchWeb::nouveau(r) {
                Ok(m) => {
                    let v = m.etat();
                    etat.partie = Some(m);
                    Reponse::json(v)
                }
                Err(e) => Reponse::erreur(400, e),
            }
        }
        ("GET", "/api/outils") => Reponse::json(json!({
            "proxies": fichiers::chemin_proxies().is_file(),
            "dos": fichiers::dos_de_carte().is_some(),
        })),
        ("GET", "/outils/dos-de-carte") => match fichiers::dos_de_carte() {
            Some((corps, type_contenu)) => Reponse {
                code: 200,
                type_contenu,
                corps,
            },
            None => Reponse::erreur(404, "image absente : data/outils/dos-de-carte.webp"),
        },
        ("GET", "/outils/proxies-de-rathe.html") => match fichiers::outil_proxies() {
            Some(corps) => Reponse {
                code: 200,
                type_contenu: "text/html; charset=utf-8",
                corps,
            },
            None => Reponse::erreur(404, "outil absent : data/outils/proxies-de-rathe.html"),
        },
        ("GET", c) if !c.starts_with("/api/") && fichiers::trouver(c).is_some() => {
            let f = fichiers::trouver(c).expect("vérifié juste avant");
            Reponse {
                code: 200,
                type_contenu: f.type_contenu,
                corps: f.octets.to_vec(),
            }
        }
        _ => Reponse::erreur(404, format!("introuvable : {methode} {chemin}")),
    }
}

/// L'en-tête `Host` désigne-t-il bien cet ordinateur ? (protection contre
/// une page web qui tenterait de parler au serveur sous un autre nom.)
fn hote_local(hote: &str) -> bool {
    let h = hote.rsplit_once(':').map(|(h, _)| h).unwrap_or(hote);
    matches!(h, "127.0.0.1" | "localhost" | "[::1]")
}

fn texte_code(code: u16) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Error",
    }
}

/// Lit une requête HTTP sur la connexion et y répond.
fn servir(flux: TcpStream, etat: &Mutex<Etat>) -> io::Result<()> {
    flux.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut lecteur = BufReader::new(flux.try_clone()?);
    let mut premiere = String::new();
    if lecteur.read_line(&mut premiere)? == 0 {
        return Ok(());
    }
    let mut mots = premiere.split_whitespace();
    let methode = mots.next().unwrap_or("").to_string();
    let requete = mots.next().unwrap_or("").to_string();
    let mut longueur = 0usize;
    let mut hote = String::new();
    loop {
        let mut l = String::new();
        if lecteur.read_line(&mut l)? == 0 {
            break;
        }
        let l = l.trim_end();
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            if k == "content-length" {
                longueur = v.trim().parse().unwrap_or(0);
            } else if k == "host" {
                hote = v.trim().to_string();
            }
        }
    }
    // Le corps ne sert qu'au chargement d'une sauvegarde (4 Mo au plus).
    let mut corps = vec![0u8; longueur.min(4 << 20)];
    lecteur.read_exact(&mut corps)?;

    let rep = if !hote_local(&hote) {
        Reponse::erreur(403, "accès réservé à cet ordinateur")
    } else {
        let mut e = etat.lock().unwrap_or_else(|p| p.into_inner());
        let rep = repondre_avec_corps(&methode, &requete, &corps, &mut e);
        if methode == "POST" && rep.code == 200 {
            if let (Some(f), Some(m)) = (&e.sauvegarde, &e.partie) {
                if let Err(err) = sauvegarde::ecrire(f, m) {
                    eprintln!("Sauvegarde impossible ({}) : {err}", f.display());
                }
            }
        }
        rep
    };
    let mut flux = flux;
    write!(
        flux,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        rep.code,
        texte_code(rep.code),
        rep.type_contenu,
        rep.corps.len()
    )?;
    flux.write_all(&rep.corps)?;
    flux.flush()
}

/// Ouvre l'adresse dans le navigateur par défaut (sans rien faire si ça
/// échoue : l'adresse reste affichée dans le terminal).
pub fn ouvrir_navigateur(url: &str) {
    use std::process::Command;
    let r = if cfg!(target_os = "windows") {
        // `start` est une commande interne de cmd ; "" = titre de fenêtre vide.
        Command::new("cmd").args(["/C", "start", "", url]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };
    let _ = r;
}

/// Réserve le premier port libre à partir de `port` (20 essais).
pub fn ecouter(port: u16) -> io::Result<TcpListener> {
    let mut derniere = None;
    for p in port..port.saturating_add(20) {
        match TcpListener::bind(("127.0.0.1", p)) {
            Ok(e) => return Ok(e),
            Err(e) => derniere = Some(e),
        }
    }
    Err(derniere.unwrap_or_else(|| io::Error::other("aucun port libre")))
}

/// Répond aux requêtes (ne rend jamais la main).
pub fn servir_toujours(ecoute: TcpListener, etat: Etat) {
    let etat = Arc::new(Mutex::new(etat));
    for flux in ecoute.incoming() {
        let Ok(flux) = flux else { continue };
        let etat = Arc::clone(&etat);
        // Un fil par connexion : une connexion ouverte d'avance par le
        // navigateur ne bloque pas les autres.
        std::thread::spawn(move || {
            let _ = servir(flux, &etat);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodage_des_parametres_d_url() {
        let (c, ps) = parametres("/api/nouvelle?deck=hala&graine=42&x=a%20b+c");
        assert_eq!(c, "/api/nouvelle");
        assert_eq!(ps[0], ("deck".into(), "hala".into()));
        assert_eq!(ps[1], ("graine".into(), "42".into()));
        assert_eq!(ps[2], ("x".into(), "a b c".into()));
        assert_eq!(decoder("%C3%A9t%C3%A9"), "été");
        assert_eq!(decoder("100%"), "100%");
    }

    #[test]
    fn un_port_deja_pris_est_saute() {
        let premier = ecouter(18_500).unwrap();
        let p1 = premier.local_addr().unwrap().port();
        let second = ecouter(p1).unwrap();
        assert_ne!(second.local_addr().unwrap().port(), p1);
    }

    #[test]
    fn seuls_les_hotes_locaux_sont_acceptes() {
        assert!(hote_local("127.0.0.1:8080"));
        assert!(hote_local("localhost:8080"));
        assert!(hote_local("localhost"));
        assert!(!hote_local("exemple.com:8080"));
        assert!(!hote_local(""));
    }
}
