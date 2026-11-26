//! Arsenal, lot 9 : extras de l'écran de partie.
//!
//! - **Conseil du bot** : le coup que jouerait un bot (B1, B2, B3…) à votre
//!   place. Le bot ne lit que ce que vous voyez (règle des bots, voir
//!   `fab_ai::Agent`) : le conseil ne trahit rien de caché.
//! - **Barre d'évaluation** : vos chances de gagner selon l'évaluation
//!   apprise de B3 (`fab_ai::b3`), en moyenne sur des mondes tirés au hasard
//!   pour ce que vous ne voyez pas (main et deck du bot), pour ne rien
//!   trahir non plus.
//! - **Bot contre Bot** : `POST /api/auto` joue votre prochain choix avec un
//!   bot ; la page enchaîne tant que le bouton est actif (le match devient
//!   un match entre deux bots, qu'on regarde et qu'on peut annuler).
//! - **Signalement de bug** : un texte et la sauvegarde du match (graine et
//!   coups : on rejoue exactement la partie) dans `sauvegardes/bugs/`.
//!
//! | Route | Rôle |
//! |---|---|
//! | `GET /api/conseil?niveau=b2` | l'option conseillée (`i`) et son texte |
//! | `GET /api/evaluation` | vos chances de gagner (0 à 1) |
//! | `POST /api/auto?niveau=b1` | joue votre choix avec ce bot |
//! | `POST /api/bug` (corps : le texte) | enregistre un signalement |
//! | `GET /api/bugs` | les signalements enregistrés |

use crate::match_web::{Etape, MatchWeb};
use crate::web::Session;
use fab_ai::b3::{Evaluation, Modele};
use fab_ai::determinisation::determiniser;
use fab_ai::{bot_connu, creer_agent};
use fab_engine::{Outcome, Rng};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// La manche en cours, si elle se joue.
pub fn session(m: &MatchWeb) -> Result<&Session, String> {
    m.session
        .as_ref()
        .filter(|_| m.etape == Etape::Partie)
        .ok_or_else(|| "la manche n'a pas commencé".into())
}

/// L'option que choisirait le bot `niveau` à la place de l'humain.
pub fn conseil(s: &Session, niveau: &str) -> Result<(usize, String), String> {
    if !bot_connu(niveau) {
        return Err(format!("bot inconnu : {niveau}"));
    }
    let d =
        s.g.pending_decision()
            .filter(|d| d.player == s.humain)
            .ok_or("ce n'est pas à vous de choisir")?
            .clone();
    // Graine fixée par la position : le même conseil si on redemande.
    let graine = s.reglages.graine ^ (s.coups().len() as u64).wrapping_mul(0x9E37_79B9);
    let mut a = creer_agent(niveau, graine).ok_or("bot inconnu")?;
    let i = a.choose(&s.g, &d);
    let texte = crate::vue::option(&s.g, &d, d.options[i]);
    Ok((i, texte))
}

fn evaluation_b3(s: &Session) -> &'static Evaluation {
    static E: OnceLock<Evaluation> = OnceLock::new();
    E.get_or_init(|| Modele::par_defaut().indexer(&s.g.db))
}

/// Mondes tirés pour la barre d'évaluation.
const MONDES: u32 = 24;

/// Chances de gagner de l'humain (0 à 1), ou `None` pendant un choix du
/// bot (on ne peut pas tirer de monde à ce moment, voir `determiniser`).
pub fn evaluation(s: &Session) -> Option<f64> {
    let h = s.humain;
    if let Some(o) = s.g.outcome {
        return Some(match o {
            Outcome::Winner(p) if p == h => 1.0,
            Outcome::Winner(_) => 0.0,
            Outcome::Draw => 0.5,
        });
    }
    if s.g.pending.as_ref().is_some_and(|d| d.player != h) {
        return None;
    }
    let e = evaluation_b3(s);
    let mut rng = Rng::new(s.reglages.graine ^ 0xE7A1 ^ s.coups().len() as u64);
    let mut somme = 0.0;
    for _ in 0..MONDES {
        let g = determiniser(&s.g, h, &mut rng);
        let p0 = e.estimer(&g) as f64;
        somme += if h.index() == 0 { p0 } else { 1.0 - p0 };
    }
    Some(somme / MONDES as f64)
}

// ---------------------------------------------------------------- bugs

fn dossier_reglable() -> &'static Mutex<Option<PathBuf>> {
    static D: Mutex<Option<PathBuf>> = Mutex::new(None);
    &D
}

/// Le dossier des signalements (`sauvegardes/bugs/` du dossier courant).
pub fn dossier_bugs() -> PathBuf {
    dossier_reglable()
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| PathBuf::from("sauvegardes").join("bugs"))
}

/// Change le dossier des signalements (tests).
pub fn utiliser_dossier_bugs(d: PathBuf) {
    *dossier_reglable().lock().unwrap() = Some(d);
}

/// Enregistre un signalement ; renvoie le nom du fichier.
pub fn signaler(texte: &str, m: Option<&MatchWeb>) -> Result<String, String> {
    let texte = texte.trim();
    if texte.is_empty() {
        return Err("décrivez le problème".into());
    }
    let maintenant = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let mut v = json!({
        "format": "arsenal-bug",
        "date": maintenant / 1000,
        "texte": texte,
    });
    if let Some(m) = m {
        v["sauvegarde"] = crate::sauvegarde::exporter(m);
        if let Ok(s) = session(m) {
            v["tour"] = json!(s.g.turn);
            v["journal"] = json!(s
                .journal()
                .iter()
                .rev()
                .take(40)
                .map(|l| l.texte.clone())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>());
        }
    }
    let nom = format!("bug-{maintenant}");
    let d = dossier_bugs();
    std::fs::create_dir_all(&d)
        .and_then(|_| {
            std::fs::write(
                d.join(format!("{nom}.json")),
                serde_json::to_string_pretty(&v).unwrap_or_default(),
            )
        })
        .map_err(|e| format!("écriture dans {} : {e}", d.display()))?;
    Ok(nom)
}

/// Les signalements enregistrés (les plus récents d'abord).
pub fn bugs() -> Vec<Value> {
    let Ok(l) = std::fs::read_dir(dossier_bugs()) else {
        return Vec::new();
    };
    let mut v: Vec<Value> = l
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let p = e.path();
            if p.extension()? != "json" {
                return None;
            }
            let b: Value = serde_json::from_str(&std::fs::read_to_string(&p).ok()?).ok()?;
            Some(json!({
                "nom": p.file_stem()?.to_str()?,
                "date": b["date"],
                "texte": b["texte"],
                "tour": b["tour"],
            }))
        })
        .collect();
    v.sort_by_key(|x| std::cmp::Reverse(x["date"].as_u64().unwrap_or(0)));
    v
}
