//! Didacticiel « Apprendre » (phase 2 du plan de rapatriement,
//! `docs/didacticiel.md`) : les **scènes** des leçons.
//!
//! Une scène est une petite partie sur le vrai moteur fabbot, en règles
//! complètes, dont tout est fixé à l'avance : héros, mains, ordre des
//! decks, équipement, qui commence, et ce que fait l'adversaire (un bot qui
//! suit un script). L'élève prend ses décisions comme dans une vraie
//! partie ; c'est le moteur qui applique les règles. Le texte des leçons
//! est dans la page (`web/apprendre/lecons.js`).
//!
//! | Requête | Réponse |
//! |---|---|
//! | `GET /api/apprendre/scenes` | les scènes connues |
//! | `POST /api/apprendre/scene?id=attaquer` | (re)lance une scène, renvoie son état |
//! | `GET /api/apprendre/etat` | état de la scène en cours (comme `/api/etat`) |
//! | `POST /api/apprendre/choix?i=0` | choix de l'élève |
//! | `POST /api/apprendre/annuler` | annule le dernier choix |
//! | `GET /api/apprendre/carte?id=12` | fiche et texte officiel d'une carte de la scène |
//! | `GET /api/apprendre/fiche?nom=Brutal%20Assault&couleur=bleu` | une carte montrée dans une leçon |

use crate::textes::Textes;
use crate::vue;
use crate::web::{Reglages, Session};
use fab_ai::Agent;
use fab_engine::card::{CardDb, Color};
use fab_engine::{Choice, Decision, GameSetup, GameState, Loadout, PlayerId};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// Réponse d'une route : code HTTP et corps JSON.
pub type Resultat = (u16, Value);

const R: Option<Color> = Some(Color::Red);
const Y: Option<Color> = Some(Color::Yellow);
const B: Option<Color> = Some(Color::Blue);

/// Une carte de deck : nom anglais exact et couleur.
type C = (&'static str, Option<Color>);

/// Un joueur d'une scène.
pub struct Cote {
    pub heros: &'static str,
    /// La main de départ (les premières cartes du deck, CR 4.1.9).
    pub main: &'static [C],
    /// Le reste du deck, du dessus vers le dessous.
    pub suite: &'static [C],
    pub arene: &'static [&'static str],
}

/// Une scène du didacticiel.
pub struct Scene {
    pub id: &'static str,
    /// L'élève (joueur 0), puis l'adversaire (joueur 1).
    pub joueurs: [Cote; 2],
    /// L'élève commence (sinon l'adversaire).
    pub eleve_commence: bool,
    /// Ce que fait l'adversaire, dans l'ordre : à chaque décision, s'il
    /// existe une option dont le texte contient la prochaine étape du
    /// script, il la prend ; sinon il passe (ou prend la première option).
    pub script: &'static [&'static str],
}

/// Remplissage des decks (CR 4.1 : 40 cartes en Silver Age) : une attaque
/// sans texte, pour que les cartes piochées plus tard restent lisibles.
const REMPLISSAGE: C = ("Brutal Assault", B);

/// Les scènes. Le texte qui les explique est dans `web/apprendre/lecons.js`.
pub static SCENES: &[Scene] = &[
    // Chapitre 1, « Le but » : une attaque sans défense fait perdre des PV.
    Scene {
        id: "attaquer",
        joueurs: [
            Cote {
                heros: "Dorinthea",
                main: &[
                    ("Raging Onslaught", R),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
            Cote {
                heros: "Bravo, Flattering Showman",
                main: &[
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
        ],
        eleve_commence: true,
        script: &[],
    },
    // Chapitre 4, « Les points d'action » : un seul point d'action, une seule attaque.
    Scene {
        id: "un-point",
        joueurs: [
            Cote {
                heros: "Dorinthea",
                main: &[
                    ("Brutal Assault", R),
                    ("Brutal Assault", R),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
            Cote {
                heros: "Bravo, Flattering Showman",
                main: &[
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
        ],
        eleve_commence: true,
        script: &[],
    },
    // Chapitre 5, « Payer » : plusieurs façons de payer un coût de 3.
    Scene {
        id: "payer",
        joueurs: [
            Cote {
                heros: "Dorinthea",
                main: &[
                    ("Raging Onslaught", R),
                    ("Brutal Assault", R),
                    ("Brutal Assault", Y),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
            Cote {
                heros: "Bravo, Flattering Showman",
                main: &[
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
        ],
        eleve_commence: true,
        script: &[],
    },
    // Chapitre 6, « Enchaîner » : une attaque avec go again, puis une seconde.
    Scene {
        id: "go-again",
        joueurs: [
            Cote {
                heros: "Ira, Crimson Haze",
                main: &[
                    ("Bittering Thorns", R),
                    ("Brutal Assault", R),
                    ("Brutal Assault", Y),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
            Cote {
                heros: "Bravo, Flattering Showman",
                main: &[
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
        ],
        eleve_commence: true,
        script: &[],
    },
    // Chapitre 7, « Défendre » : l'adversaire attaque avec 7 de puissance.
    Scene {
        id: "defendre",
        joueurs: [
            Cote {
                heros: "Dorinthea",
                main: &[
                    ("Brutal Assault", R),
                    ("Brutal Assault", R),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
            Cote {
                heros: "Bravo, Flattering Showman",
                main: &[
                    ("Raging Onslaught", R),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                    ("Brutal Assault", B),
                ],
                suite: &[],
                arene: &[],
            },
        ],
        eleve_commence: false,
        script: &["Jouer Raging Onslaught", "Pitcher"],
    },
];

/// Le bot des scènes : il suit le script, puis passe.
struct BotScripte {
    script: Vec<&'static str>,
}

impl Agent for BotScripte {
    fn name(&self) -> &str {
        "adversaire du didacticiel"
    }

    fn choose(&mut self, state: &GameState, d: &Decision) -> usize {
        if let Some(&etape) = self.script.first() {
            if let Some(i) = d
                .options
                .iter()
                .position(|&c| vue::option(state, d, c).contains(etape))
            {
                self.script.remove(0);
                return i;
            }
        }
        d.index_of(Choice::Pass).unwrap_or(0)
    }
}

fn carte(db: &CardDb, (nom, couleur): C) -> Result<fab_engine::CardId, String> {
    db.find(nom, couleur)
        .ok_or_else(|| format!("carte inconnue dans une scène : {nom} {couleur:?}"))
}

fn equipement(db: &CardDb, cote: &Cote) -> Result<Loadout, String> {
    let mut deck = Vec::new();
    for &c in cote.main.iter().chain(cote.suite) {
        deck.push(carte(db, c)?);
    }
    while deck.len() < 40 {
        deck.push(carte(db, REMPLISSAGE)?);
    }
    let arena = cote
        .arene
        .iter()
        .map(|&n| carte(db, (n, None)))
        .collect::<Result<_, _>>()?;
    Ok(Loadout {
        hero: carte(db, (cote.heros, None))?,
        arena,
        deck,
        pool: Vec::new(),
    })
}

/// Prépare la partie d'une scène (decks dans l'ordre, sans mélange) et la
/// session : le moteur avance jusqu'au premier choix de l'élève.
pub fn lancer(scene: &Scene) -> Result<Session, String> {
    let db = Arc::new(fab_cards::real::build());
    let setup = GameSetup {
        players: [
            equipement(&db, &scene.joueurs[0])?,
            equipement(&db, &scene.joueurs[1])?,
        ],
        chooser: PlayerId::P0,
        seed: 1,
        shuffle: false,
        max_turns: 30,
    };
    let mut g = GameState::new(db, setup).map_err(|e| format!("mise en place : {e:?}"))?;
    // CR 4.1.3 : le joueur 0 choisit qui commence.
    let premier = if scene.eleve_commence {
        PlayerId::P0
    } else {
        PlayerId::P1
    };
    let i = g
        .pending_decision()
        .and_then(|d| d.index_of(Choice::Player(premier)))
        .ok_or("pas de choix du premier joueur")?;
    g.apply(i).map_err(|e| format!("{e:?}"))?;
    let r = Reglages {
        deck: scene.id.into(),
        deck_bot: scene.id.into(),
        niveau: "script".into(),
        graine: 1,
        manches: 1,
    };
    let bot = Box::new(BotScripte {
        script: scene.script.to_vec(),
    });
    Session::avec_agent(g, r, bot)
}

/// La scène en cours (une seule à la fois, à part du match de l'écran Jouer).
static EN_COURS: Mutex<Option<(&'static str, Session)>> = Mutex::new(None);

fn etat(id: &str, s: &Session) -> Value {
    let mut v = s.etat();
    v["scene"] = json!(id);
    v
}

/// Base des cartes, chargée une fois (pour les fiches montrées hors scène).
fn base() -> &'static CardDb {
    static DB: std::sync::OnceLock<CardDb> = std::sync::OnceLock::new();
    DB.get_or_init(fab_cards::real::build)
}

/// Une carte montrée dans une leçon (hors partie) : image, valeurs
/// imprimées, ligne de type et texte officiel.
fn fiche(nom: &str, couleur: Option<Color>, textes: Option<&Textes>) -> Resultat {
    let db = base();
    let Some(id) = db.find(nom, couleur) else {
        return erreur(404, format!("carte inconnue : {nom}"));
    };
    let d = db.get(id);
    let (image, nom_fr) = crate::images::champs(d);
    let (typ, texte) = textes
        .and_then(|t| t.texte(&d.name, d.color))
        .unwrap_or_default();
    (
        200,
        json!({
            "nom": d.name, "nom_fr": nom_fr, "image": image,
            "couleur": crate::web::nom_couleur(d.color),
            "cout": d.cost, "pitch": d.pitch, "puissance": d.power, "defense": d.defense,
            "vie": d.life, "intellect": d.intellect,
            "type": typ, "texte": texte,
        }),
    )
}

fn erreur(code: u16, m: impl Into<String>) -> Resultat {
    (code, json!({ "erreur": m.into() }))
}

/// Les routes `/api/apprendre/…` ; `None` pour les autres chemins.
pub fn repondre(
    methode: &str,
    chemin: &str,
    param: &dyn Fn(&str) -> Option<String>,
    textes: Option<&Textes>,
) -> Option<Resultat> {
    if !chemin.starts_with("/api/apprendre/") {
        return None;
    }
    let mut en_cours = EN_COURS.lock().unwrap_or_else(|e| e.into_inner());
    Some(match (methode, chemin) {
        ("GET", "/api/apprendre/scenes") => (
            200,
            json!({ "scenes": SCENES.iter().map(|s| s.id).collect::<Vec<_>>() }),
        ),
        ("GET", "/api/apprendre/fiche") => {
            let nom = param("nom").unwrap_or_default();
            let couleur = match param("couleur").as_deref() {
                Some("rouge") => R,
                Some("jaune") => Y,
                Some("bleu") => B,
                _ => None,
            };
            fiche(&nom, couleur, textes)
        }
        ("POST", "/api/apprendre/scene") => {
            let id = param("id").unwrap_or_default();
            let Some(scene) = SCENES.iter().find(|s| s.id == id) else {
                return Some(erreur(404, format!("scène inconnue « {id} »")));
            };
            match lancer(scene) {
                Ok(s) => {
                    let v = etat(scene.id, &s);
                    *en_cours = Some((scene.id, s));
                    (200, v)
                }
                Err(e) => erreur(500, e),
            }
        }
        (_, c) => {
            let Some((id, s)) = en_cours.as_mut() else {
                return Some(erreur(409, "aucune scène en cours"));
            };
            match (methode, c) {
                ("GET", "/api/apprendre/etat") => (200, etat(id, s)),
                ("POST", "/api/apprendre/choix") => {
                    let i = param("i").and_then(|v| v.parse::<usize>().ok());
                    match i.map(|i| s.choisir(i)) {
                        Some(Ok(())) => (200, etat(id, s)),
                        Some(Err(e)) => erreur(400, e),
                        None => erreur(400, "paramètre i manquant"),
                    }
                }
                ("POST", "/api/apprendre/annuler") => {
                    s.annuler();
                    (200, etat(id, s))
                }
                ("GET", "/api/apprendre/carte") => {
                    match param("id").and_then(|v| v.parse::<u32>().ok()) {
                        Some(n) => (200, s.carte(n, textes)),
                        None => erreur(400, "paramètre id manquant"),
                    }
                }
                _ => erreur(404, "route inconnue"),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque scène se met en place et s'arrête sur une décision de l'élève.
    #[test]
    fn toutes_les_scenes_se_lancent() {
        for sc in SCENES {
            let s = lancer(sc).unwrap_or_else(|e| panic!("{} : {e}", sc.id));
            let v = s.etat();
            assert!(!v["decision"].is_null(), "{} : pas de décision", sc.id);
        }
    }

    /// Scène « attaquer » : l'adversaire ne défend pas, Raging Onslaught
    /// rouge (puissance 7) lui fait perdre 7 PV.
    #[test]
    fn scene_attaquer_fait_perdre_7_pv() {
        let mut s = lancer(&SCENES[0]).unwrap();
        let vie = s.etat()["bot"]["vie"].as_i64().unwrap();
        // L'élève joue l'attaque, puis paie avec une carte bleue, puis passe.
        for _ in 0..12 {
            let v = s.etat();
            let opts = v["decision"]["options"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let pick = opts
                .iter()
                .find(|o| {
                    o["texte"]
                        .as_str()
                        .unwrap_or("")
                        .contains("Raging Onslaught")
                })
                .or_else(|| {
                    opts.iter()
                        .find(|o| o["texte"].as_str().unwrap_or("").contains("Brutal Assault"))
                })
                .or_else(|| opts.iter().find(|o| o["passer"] == true))
                .and_then(|o| o["i"].as_u64());
            let Some(i) = pick else { break };
            s.choisir(i as usize).unwrap();
            if s.etat()["bot"]["vie"].as_i64().unwrap() < vie {
                break;
            }
        }
        assert_eq!(s.etat()["bot"]["vie"].as_i64().unwrap(), vie - 7);
    }

    /// Toutes les cartes montrées par les leçons (`{ nom: "…", couleur: "…" }`
    /// dans `lecons.js`) existent dans la base.
    #[test]
    fn les_cartes_des_lecons_existent() {
        let js = include_str!("../web/apprendre/lecons.js");
        let mut n = 0;
        for bloc in js.split("{ nom: \"").skip(1) {
            let nom = &bloc[..bloc.find('"').unwrap()];
            let reste = &bloc[nom.len()..bloc.find('}').unwrap()];
            let couleur = reste
                .split("couleur: \"")
                .nth(1)
                .map(|c| &c[..c.find('"').unwrap()]);
            let c = match couleur {
                Some("rouge") => R,
                Some("jaune") => Y,
                Some("bleu") => B,
                _ => None,
            };
            assert!(
                base().find(nom, c).is_some(),
                "carte inconnue : {nom} {couleur:?}"
            );
            n += 1;
        }
        assert!(n > 10, "{n} cartes seulement");
    }

    /// Une notion (`{{notion}}`) n'est employée qu'après le plan qui
    /// l'introduit (`introduit: [...]`), et chaque notion est définie dans
    /// `NOTIONS`. C'est ce qui garantit qu'aucun mot du jeu n'arrive avant
    /// d'avoir été expliqué.
    #[test]
    fn ordre_des_notions() {
        let js = include_str!("../web/apprendre/lecons.js");
        let (defs, corps) = js.split_at(js.find("const LECONS").unwrap());
        let mut connues = std::collections::HashSet::new();
        let mut i = 0;
        while i < corps.len() {
            if !corps.is_char_boundary(i) {
                i += 1;
                continue;
            }
            if corps[i..].starts_with("introduit: [") {
                let fin = i + corps[i..].find(']').unwrap();
                for n in corps[i + 12..fin].split(',') {
                    let n = n.trim().trim_matches('"');
                    assert!(
                        defs.contains(&format!("\n  {n}: {{")),
                        "notion « {n} » absente de NOTIONS"
                    );
                    connues.insert(n.to_string());
                }
                i = fin;
            } else if corps[i..].starts_with("{{") {
                let fin = i + corps[i..].find("}}").unwrap();
                let n = corps[i + 2..fin].split('|').next().unwrap();
                let mut d = i.saturating_sub(60);
                while !corps.is_char_boundary(d) {
                    d += 1;
                }
                let contexte = &corps[d..fin];
                assert!(
                    connues.contains(n),
                    "« {n} » employé avant d'être introduit : …{contexte}"
                );
                i = fin;
            }
            i += 1;
        }
        assert!(connues.len() > 10);
    }

    /// Scène « un-point » : après la première attaque, l'élève n'a plus de
    /// point d'action et ne peut plus jouer la seconde (CR 8.1.1c, 4.3.2).
    #[test]
    fn scene_un_point_une_seule_attaque() {
        let sc = SCENES.iter().find(|s| s.id == "un-point").unwrap();
        let mut s = lancer(sc).unwrap();
        let joue = |s: &mut Session, mot: &str| {
            let v = s.etat();
            let o = v["decision"]["options"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["texte"].as_str().unwrap().contains(mot))
                .map(|o| o["i"].as_u64().unwrap() as usize);
            o.map(|i| s.choisir(i).unwrap()).is_some()
        };
        assert!(joue(&mut s, "Jouer Brutal Assault (rouge)"));
        assert!(joue(&mut s, "Pitcher Brutal Assault (bleu)"));
        let v = s.etat();
        assert_eq!(v["bot"]["vie"].as_i64().unwrap(), 20 - 6);
        let plus_d_action = v["decision"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["action"] == "jouer");
        assert!(!plus_d_action, "plus aucune carte ne se joue : {v}");
    }

    /// Scène « payer » : jaune + rouge (2 + 1) paient le coût de 3 de
    /// Raging Onslaught ; il ne reste rien (CR 1.14.2d).
    #[test]
    fn scene_payer_avec_deux_cartes() {
        let sc = SCENES.iter().find(|s| s.id == "payer").unwrap();
        let mut s = lancer(sc).unwrap();
        for mot in [
            "Jouer Raging Onslaught",
            "Pitcher Brutal Assault (jaune)",
            "Pitcher Brutal Assault (rouge)",
        ] {
            let v = s.etat();
            let i = v["decision"]["options"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["texte"].as_str().unwrap().contains(mot))
                .map(|o| o["i"].as_u64().unwrap() as usize)
                .unwrap_or_else(|| panic!("option « {mot} » absente : {v}"));
            s.choisir(i).unwrap();
        }
        let v = s.etat();
        assert_eq!(v["bot"]["vie"].as_i64().unwrap(), 20 - 7);
        assert_eq!(v["vous"]["pitch"].as_array().unwrap().len(), 2);
    }

    /// Applique les options dont le texte contient chacun des mots, dans l'ordre.
    fn jouer_suite(s: &mut Session, mots: &[&str]) {
        for mot in mots {
            let v = s.etat();
            let i = v["decision"]["options"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["texte"].as_str().unwrap().contains(mot))
                .map(|o| o["i"].as_u64().unwrap() as usize)
                .unwrap_or_else(|| panic!("option « {mot} » absente : {v}"));
            s.choisir(i).unwrap();
        }
    }

    /// Scène « go-again » : Bittering Thorns (4, go again) puis Brutal
    /// Assault rouge (6, +1 grâce à Bittering Thorns, +1 par la capacité
    /// d'Ira, « Your second attack each turn gets +1 ») : 12 PV en tout.
    #[test]
    fn scene_go_again_deux_attaques() {
        let sc = SCENES.iter().find(|s| s.id == "go-again").unwrap();
        let mut s = lancer(sc).unwrap();
        jouer_suite(
            &mut s,
            &[
                "Jouer Bittering Thorns",
                "Pitcher Brutal Assault (jaune)",
                "Jouer Brutal Assault (rouge)",
                "Pitcher Brutal Assault (bleu)",
            ],
        );
        assert_eq!(s.etat()["bot"]["vie"].as_i64().unwrap(), 20 - 12);
    }

    /// Scène « defendre » : deux cartes de défense 3 contre 7 de
    /// puissance, 1 dégât (CR 7.5.2).
    #[test]
    fn scene_defendre_deux_cartes() {
        let sc = SCENES.iter().find(|s| s.id == "defendre").unwrap();
        let mut s = lancer(sc).unwrap();
        assert_eq!(s.etat()["decision"]["genre"], "defense");
        jouer_suite(
            &mut s,
            &[
                "Défendre avec Brutal Assault (rouge)",
                "Défendre avec Brutal Assault (rouge)",
                "Terminer la déclaration",
            ],
        );
        // Les réactions éventuelles : l'élève passe.
        while s.etat()["vous"]["vie"].as_i64() == Some(20)
            && !s.etat()["decision"].is_null()
            && s.etat()["vous"]["cimetiere"].as_array().unwrap().is_empty()
        {
            jouer_suite(&mut s, &["Passer"]);
        }
        let v = s.etat();
        assert_eq!(v["vous"]["vie"].as_i64().unwrap(), 20 - 1, "{v}");
    }
}
