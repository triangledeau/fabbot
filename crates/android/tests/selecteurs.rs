//! Les classes et identifiants de la page PC visés par `web/mobile.css` et
//! `web/mobile.js` existent-ils encore dans `crates/ui/web/` ?
//!
//! Si un autre agent renomme une classe de la page PC, l'adaptation au
//! téléphone casserait sans bruit : ce test échoue sur **sa** pull request
//! et lui dit quoi regarder (`docs/android.md`, section 4.5).

use std::path::{Path, PathBuf};

/// Classes créées par `mobile.js` lui-même : absentes de la page PC.
fn propre_a_android(nom: &str) -> bool {
    nom.starts_with("android")
}

fn est_nom(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Les `.classe` et `#identifiant` d'un sélecteur CSS.
fn noms_du_selecteur(selecteur: &str) -> Vec<String> {
    let mut noms = Vec::new();
    // Les valeurs entre crochets ([href="a.b"]) ne sont pas des sélecteurs.
    let mut dans_crochets = false;
    let car: Vec<char> = selecteur.chars().collect();
    let mut i = 0;
    while i < car.len() {
        let c = car[i];
        match c {
            '[' => dans_crochets = true,
            ']' => dans_crochets = false,
            '.' | '#' if !dans_crochets => {
                let debut = i + 1;
                let mut fin = debut;
                while fin < car.len() && est_nom(car[fin]) {
                    fin += 1;
                }
                let nom: String = car[debut..fin].iter().collect();
                if nom.chars().next().is_some_and(|p| !p.is_ascii_digit()) {
                    noms.push(format!("{c}{nom}"));
                }
                i = fin;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    noms
}

/// Les sélecteurs d'une feuille de style (hors commentaires, hors
/// déclarations, en entrant dans `@media` et `@supports`).
fn selecteurs_css(css: &str) -> Vec<String> {
    let mut sans_commentaires = String::new();
    let mut reste = css;
    while let Some(i) = reste.find("/*") {
        sans_commentaires.push_str(&reste[..i]);
        reste = reste[i..].find("*/").map_or("", |j| &reste[i + j + 2..]);
    }
    sans_commentaires.push_str(reste);

    let mut noms = Vec::new();
    let mut prelude = String::new();
    let mut it = sans_commentaires.chars();
    while let Some(c) = it.next() {
        match c {
            '{' => {
                let p = prelude.trim();
                let conteneur = p.starts_with("@media") || p.starts_with("@supports");
                if !conteneur {
                    if !p.starts_with('@') {
                        noms.extend(noms_du_selecteur(p));
                    }
                    let mut profondeur = 1;
                    for c in it.by_ref() {
                        match c {
                            '{' => profondeur += 1,
                            '}' => {
                                profondeur -= 1;
                                if profondeur == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                prelude.clear();
            }
            '}' | ';' => prelude.clear(),
            _ => prelude.push(c),
        }
    }
    noms
}

/// Les sélecteurs écrits en toutes lettres dans un script :
/// `querySelector("…")`, `querySelectorAll`, `closest`, `matches`, et
/// `getElementById("x")` (→ `#x`).
fn selecteurs_js(js: &str) -> Vec<String> {
    let mut noms = Vec::new();
    for fonction in [
        "querySelector(",
        "querySelectorAll(",
        "closest(",
        "matches(",
        "getElementById(",
    ] {
        let mut reste = js;
        while let Some(i) = reste.find(fonction) {
            reste = &reste[i + fonction.len()..];
            let mut car = reste.chars();
            let Some(guillemet) = car.next().filter(|g| matches!(g, '"' | '\'' | '`')) else {
                continue;
            };
            let texte: String = car.take_while(|c| *c != guillemet).collect();
            if fonction == "getElementById(" {
                noms.push(format!("#{texte}"));
            } else {
                noms.extend(noms_du_selecteur(&texte));
            }
        }
    }
    noms
}

/// Le nom apparaît-il comme un mot entier dans le texte ?
fn contient_le_mot(texte: &str, mot: &str) -> bool {
    texte.match_indices(mot).any(|(i, _)| {
        let avant = texte[..i].chars().next_back();
        let apres = texte[i + mot.len()..].chars().next();
        !avant.is_some_and(est_nom) && !apres.is_some_and(est_nom)
    })
}

fn fichiers_de(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dossier).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            fichiers_de(&p, sortie);
        } else if p
            .extension()
            .is_some_and(|x| x == "html" || x == "js" || x == "css")
        {
            sortie.push(p);
        }
    }
}

/// Les noms (sans `.` ni `#`) cherchés dans la page PC et introuvables.
fn manquants(noms: &[String], page: &str) -> Vec<String> {
    let mut m: Vec<String> = noms
        .iter()
        .filter(|n| !propre_a_android(&n[1..]) && !contient_le_mot(page, &n[1..]))
        .cloned()
        .collect();
    m.sort();
    m.dedup();
    m
}

#[test]
fn chaque_selecteur_de_mobile_css_et_mobile_js_existe_dans_la_page_pc() {
    let ici = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut fichiers = Vec::new();
    fichiers_de(&ici.join("../ui/web"), &mut fichiers);
    assert!(fichiers.len() > 5, "page PC introuvable");
    let page: String = fichiers
        .iter()
        .map(|f| std::fs::read_to_string(f).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    let css = std::fs::read_to_string(ici.join("web/mobile.css")).unwrap();
    let js = std::fs::read_to_string(ici.join("web/mobile.js")).unwrap();
    let mut noms = selecteurs_css(&css);
    noms.extend(selecteurs_js(&js));
    let m = manquants(&noms, &page);
    assert!(
        m.is_empty(),
        "Ces classes ou identifiants, utilisés par l'appli Android \
         (crates/android/web/mobile.css ou mobile.js), n'existent plus dans \
         crates/ui/web/ : {m:?}. Si vous les avez renommés, mettez aussi à \
         jour ces deux fichiers (docs/android.md, section 4.5)."
    );
}

#[test]
fn les_selecteurs_css_sont_lus_sans_les_declarations_ni_les_commentaires() {
    let css =
        "/* .commentaire */ html.android .main, #plateau > .carte:not(.vide) { color: #fff; }\n\
               @media (max-width: 900px) { .zone[data-x=\"a.b\"] { width: 1.5em } }\n\
               @font-face { font-family: x; }";
    let noms = selecteurs_css(css);
    assert_eq!(
        noms,
        vec![".android", ".main", "#plateau", ".carte", ".vide", ".zone"]
    );
}

#[test]
fn les_selecteurs_js_sont_lus_dans_les_appels_de_recherche() {
    let js = "document.querySelector('.main .carte'); el.closest(\"#plateau\");\n\
              document.getElementById(`journal`); x.classList.add('android-grand');";
    let mut noms = selecteurs_js(js);
    noms.sort();
    assert_eq!(noms, vec!["#journal", "#plateau", ".carte", ".main"]);
}

#[test]
fn un_selecteur_renomme_est_signale() {
    let page = "<div class=\"main plateau-x\" id=\"journal\">";
    let noms: Vec<String> = [".main", ".plateau", "#journal", ".android-grand"]
        .map(String::from)
        .to_vec();
    assert_eq!(manquants(&noms, page), vec![".plateau".to_string()]);
}
