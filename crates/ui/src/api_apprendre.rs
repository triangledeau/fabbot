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
//! | `GET /api/apprendre/carte?id=12` | fiche et texte officiel d'une carte |

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
}
