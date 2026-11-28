//! Le serveur local de l'appli : la boucle qui lit les requêtes HTTP, les
//! routes propres au téléphone (`/android/…`), puis tout le reste confié à
//! `fab_ui::serveur::repondre_avec_corps`, exactement comme sur PC.

use crate::donnees;
use fab_ui::sauvegarde;
use fab_ui::serveur::{repondre_avec_corps, Etat, Reponse};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// L'adaptation de la page au téléphone (styles).
pub const MOBILE_CSS: &[u8] = include_bytes!("../web/mobile.css");
/// L'adaptation de la page au téléphone (script).
pub const MOBILE_JS: &[u8] = include_bytes!("../web/mobile.js");

/// Les deux balises ajoutées juste avant `</head>` de la page.
pub const BALISES: &str = "<link rel=\"stylesheet\" href=\"/android/mobile.css\">\n\
<script src=\"/android/mobile.js\" defer></script>\n";

const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

/// Calcule la réponse à une requête (sans réseau : testable directement).
pub fn repondre(methode: &str, requete: &str, corps: &[u8], etat: &mut Etat) -> Reponse {
    let chemin = requete.split('?').next().unwrap_or("");
    match (methode, chemin) {
        ("GET", "/android/mobile.css") => fichier(CSS, MOBILE_CSS),
        ("GET", "/android/mobile.js") => fichier(JS, MOBILE_JS),
        ("GET", "/outils/dos-de-carte") if donnees::DOS_DE_CARTE.is_some() => {
            let (octets, type_contenu) = donnees::DOS_DE_CARTE.expect("vérifié juste avant");
            Reponse {
                code: 200,
                type_contenu,
                corps: octets.to_vec(),
            }
        }
        _ => {
            let mut rep = repondre_avec_corps(methode, requete, corps, etat);
            if rep.code == 200 && methode == "GET" {
                match chemin {
                    "/" | "/index.html" => rep.corps = injecter(rep.texte()).into_bytes(),
                    "/api/outils" if donnees::DOS_DE_CARTE.is_some() => {
                        if let Ok(mut v) = serde_json::from_slice::<serde_json::Value>(&rep.corps) {
                            v["dos"] = true.into();
                            rep.corps = v.to_string().into_bytes();
                        }
                    }
                    _ => {}
                }
            }
            rep
        }
    }
}

fn fichier(type_contenu: &'static str, octets: &[u8]) -> Reponse {
    Reponse {
        code: 200,
        type_contenu,
        corps: octets.to_vec(),
    }
}

/// Ajoute les balises de `mobile.css` et `mobile.js` juste avant `</head>`
/// (à la fin, si la page n'en a pas).
pub fn injecter(page: &str) -> String {
    match page.find("</head>") {
        Some(i) => format!("{}{BALISES}{}", &page[..i], &page[i..]),
        None => format!("{page}{BALISES}"),
    }
}

/// L'en-tête `Host` désigne-t-il bien le téléphone lui-même ?
pub fn hote_local(hote: &str) -> bool {
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

fn erreur_403() -> Reponse {
    Reponse {
        code: 403,
        type_contenu: "application/json; charset=utf-8",
        corps: "{\"erreur\":\"accès réservé à cet appareil\"}"
            .as_bytes()
            .to_vec(),
    }
}

/// Lit une requête HTTP sur la connexion et y répond.
///
/// Copie de `fab_ui::serveur::servir` du 2026-11-28, **à garder alignée**
/// (lecture de la première ligne et des en-têtes `Content-Length` et
/// `Host`, refus d'un hôte non local, corps limité à 4 Mo, verrou sur
/// l'état, sauvegarde après chaque `POST` réussi). Seule différence : les
/// routes de [`repondre`] passent avant celles de `fab-ui`.
pub fn servir(flux: TcpStream, etat: &Mutex<Etat>) -> io::Result<()> {
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
    let mut corps = vec![0u8; longueur.min(4 << 20)];
    lecteur.read_exact(&mut corps)?;

    let rep = if !hote_local(&hote) {
        erreur_403()
    } else {
        let mut e = etat.lock().unwrap_or_else(|p| p.into_inner());
        let rep = repondre(&methode, &requete, &corps, &mut e);
        if methode == "POST" && rep.code == 200 {
            if let (Some(f), Some(m)) = (&e.sauvegarde, &e.partie) {
                if let Err(err) = sauvegarde::ecrire(f, m) {
                    crate::journal::ecrire(&format!(
                        "Sauvegarde impossible ({}) : {err}",
                        f.display()
                    ));
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

/// Répond aux requêtes (ne rend jamais la main), un fil par connexion.
pub fn servir_toujours(ecoute: TcpListener, etat: Arc<Mutex<Etat>>) {
    for flux in ecoute.incoming() {
        let Ok(flux) = flux else { continue };
        let etat = Arc::clone(&etat);
        let _ = std::thread::Builder::new()
            .name("connexion".into())
            .spawn(move || {
                let _ = servir(flux, &etat);
            });
    }
}
