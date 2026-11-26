//! Sauvegarde et reprise d'un match (lot 3 du plan de rapatriement,
//! `docs/rapatriement-arsenal.md`).
//!
//! Une sauvegarde ne contient **que des choix** : les réglages du match
//! (decks, niveau, graine, manches), puis, dans l'ordre, les choix de
//! préparation de l'humain et tous les choix appliqués au moteur pendant
//! chaque manche (ceux de l'humain **et** ceux du bot). Le moteur étant
//! déterministe (même graine, mêmes choix = même partie), on retrouve
//! exactement la même position en rejouant ces choix, sans demander au bot
//! (dont la réflexion peut dépendre du temps ou du hasard).
//!
//! Format (JSON) :
//!
//! ```json
//! { "format": "arsenal-fabbot", "version": 1,
//!   "reglages": { "deck": "hala", "deck_bot": "dorinthea", "niveau": "b1", "graine": "42", "manches": 2 },
//!   "etapes": [ {"premier": "vous"}, {"arene": [0, 2]}, {"deck": [2, 2, 1]}, {"partie": [3, 0, 1]},
//!               {"manche_suivante": true}, … ] }
//! ```
//!
//! La partie en cours est aussi écrite à chaque coup dans un fichier
//! (`sauvegardes/en-cours.json`, à côté de `Jouer.bat`) : en relançant le
//! jeu, on la retrouve là où on l'avait laissée.

use crate::match_web::MatchWeb;
use crate::web::Reglages;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Nom du format, pour reconnaître un fichier.
pub const FORMAT: &str = "arsenal-fabbot";
pub const VERSION: u64 = 1;

/// Le fichier de la partie en cours (dossier courant, c'est-à-dire la
/// racine du projet quand on passe par `Jouer.bat`).
pub fn fichier_par_defaut() -> PathBuf {
    PathBuf::from("sauvegardes").join("en-cours.json")
}

/// La sauvegarde d'un match.
pub fn exporter(m: &MatchWeb) -> Value {
    let r = &m.reglages;
    json!({
        "format": FORMAT,
        "version": VERSION,
        "reglages": {
            "deck": r.deck,
            "deck_bot": r.deck_bot,
            "niveau": r.niveau,
            // En texte : un u64 ne tient pas toujours dans un nombre JavaScript.
            "graine": r.graine.to_string(),
            "manches": r.manches,
        },
        "etapes": m.etapes(),
    })
}

fn nombres(v: &Value, quoi: &str) -> Result<Vec<usize>, String> {
    v.as_array()
        .ok_or_else(|| format!("{quoi} : liste de nombres attendue"))?
        .iter()
        .map(|x| {
            x.as_u64()
                .map(|n| n as usize)
                .ok_or_else(|| format!("{quoi} : nombre attendu"))
        })
        .collect()
}

/// Recrée le match d'une sauvegarde en rejouant tous ses choix.
pub fn importer(v: &Value) -> Result<MatchWeb, String> {
    if v["format"] != FORMAT {
        return Err("ce fichier n'est pas une sauvegarde d'Arsenal (fabbot)".into());
    }
    if v["version"].as_u64() != Some(VERSION) {
        return Err(format!(
            "version de sauvegarde non prise en charge : {}",
            v["version"]
        ));
    }
    let r = &v["reglages"];
    let texte = |k: &str| {
        r[k].as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("réglage « {k} » manquant"))
    };
    let graine = match &r["graine"] {
        Value::String(s) => s.parse().map_err(|_| "graine invalide".to_string())?,
        x => x.as_u64().ok_or("graine invalide")?,
    };
    let reglages = Reglages {
        deck: texte("deck")?,
        deck_bot: texte("deck_bot")?,
        niveau: texte("niveau")?,
        graine,
        manches: r["manches"]
            .as_u64()
            .ok_or("réglage « manches » manquant")? as u8,
    };
    let mut m = MatchWeb::nouveau(reglages)?;
    let etapes = v["etapes"].as_array().ok_or("liste « etapes » manquante")?;
    for (n, e) in etapes.iter().enumerate() {
        let pb = |x: String| format!("étape {} : {x}", n + 1);
        if let Some(q) = e.get("premier") {
            m.premier_joueur(q.as_str().unwrap_or("")).map_err(pb)?;
        } else if let Some(a) = e.get("arene") {
            m.arene(&nombres(a, "arene").map_err(pb)?).map_err(pb)?;
        } else if let Some(d) = e.get("deck") {
            // Les coups de la manche suivent juste après.
            let coups = etapes
                .get(n + 1)
                .and_then(|x| x.get("partie"))
                .map(|c| nombres(c, "partie"))
                .transpose()
                .map_err(pb)?
                .unwrap_or_default();
            m.preparer_rejeu(coups.into_iter().map(|c| c as u32).collect());
            m.deck(&nombres(d, "deck").map_err(pb)?).map_err(pb)?;
            m.apres_coup();
        } else if e.get("partie").is_some() {
            // Déjà rejouée avec le « deck » qui précède.
        } else if e.get("manche_suivante").is_some() {
            m.manche_suivante().map_err(pb)?;
        } else {
            return Err(pb("étape inconnue".into()));
        }
    }
    Ok(m)
}

/// Écrit la sauvegarde dans un fichier (en passant par un fichier
/// temporaire, pour ne jamais laisser un fichier à moitié écrit).
pub fn ecrire(chemin: &Path, m: &MatchWeb) -> std::io::Result<()> {
    if let Some(d) = chemin.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = chemin.with_extension("json.tmp");
    std::fs::write(&tmp, exporter(m).to_string())?;
    std::fs::rename(&tmp, chemin)
}

/// Lit une sauvegarde depuis un fichier.
pub fn lire(chemin: &Path) -> Result<MatchWeb, String> {
    let t = std::fs::read_to_string(chemin).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&t).map_err(|e| format!("fichier illisible : {e}"))?;
    importer(&v)
}
