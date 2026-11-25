//! Lecture du fichier officiel LSS (`data/cards/sources/public_card_data.csv`,
//! non versionné, voir `tools/fetch_cards.sh`).

use std::collections::HashMap;

/// Une carte (une couleur) du fichier officiel, en anglais.
#[derive(Debug, Clone, Default)]
pub struct Officielle {
    pub name: String,
    /// "red", "yellow", "blue" ou "".
    pub color: String,
    pub typebox: String,
    /// Texte à jour, errata compris (`true_textbox`).
    pub textbox: String,
    pub cost: String,
    pub power: String,
    pub defense: String,
    pub life: String,
    pub intellect: String,
    pub pitch: String,
    pub rarities: Vec<String>,
}

/// Toutes les cartes anglaises, par (nom, couleur).
pub struct Base {
    pub cards: HashMap<(String, String), Officielle>,
}

impl Base {
    pub fn depuis_csv(texte: &str) -> Base {
        let rows = lire_csv(texte);
        let mut cards: HashMap<(String, String), Officielle> = HashMap::new();
        let Some(head) = rows.first() else {
            return Base { cards };
        };
        let col = |n: &str| head.iter().position(|h| h == n).unwrap_or(usize::MAX);
        let (lang, rar) = (col("print_language"), col("rarity"));
        let f = |n: &str| col(&format!("face_1_{n}"));
        let (name, color, tb, text) = (
            f("true_name"),
            f("true_color"),
            f("true_typebox"),
            f("true_textbox"),
        );
        let (cost, power, def, life, int, pitch) = (
            f("true_cost"),
            f("true_power"),
            f("true_defense"),
            f("true_life"),
            f("true_intellect"),
            f("true_pitch"),
        );
        let get = |r: &Vec<String>, i: usize| r.get(i).cloned().unwrap_or_default();
        for r in &rows[1..] {
            if get(r, lang) != "en" {
                continue;
            }
            let key = (get(r, name), get(r, color));
            let e = cards.entry(key.clone()).or_insert_with(|| Officielle {
                name: key.0.clone(),
                color: key.1.clone(),
                typebox: get(r, tb),
                textbox: get(r, text),
                cost: get(r, cost),
                power: get(r, power),
                defense: get(r, def),
                life: get(r, life),
                intellect: get(r, int),
                pitch: get(r, pitch),
                rarities: vec![],
            });
            let rr = get(r, rar).to_lowercase();
            if !e.rarities.contains(&rr) {
                e.rarities.push(rr);
            }
        }
        Base { cards }
    }

    /// Le fichier du dépôt, s'il a été téléchargé.
    pub fn charger() -> Option<Base> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/cards/sources/public_card_data.csv");
        std::fs::read_to_string(p)
            .ok()
            .map(|t| Base::depuis_csv(&t))
    }

    pub fn get(&self, name: &str, color: &str) -> Option<&Officielle> {
        self.cards.get(&(name.to_string(), color.to_string()))
    }
}

/// Lecteur CSV minimal (guillemets, virgules et retours à la ligne dans les champs).
pub fn lire_csv(texte: &str) -> Vec<Vec<String>> {
    let mut lignes = Vec::new();
    let mut ligne = Vec::new();
    let mut champ = String::new();
    let mut guillemets = false;
    let mut it = texte.chars().peekable();
    while let Some(c) = it.next() {
        if guillemets {
            match c {
                '"' if it.peek() == Some(&'"') => {
                    champ.push('"');
                    it.next();
                }
                '"' => guillemets = false,
                _ => champ.push(c),
            }
        } else {
            match c {
                '"' => guillemets = true,
                ',' => ligne.push(std::mem::take(&mut champ)),
                '\r' => {}
                '\n' => {
                    ligne.push(std::mem::take(&mut champ));
                    lignes.push(std::mem::take(&mut ligne));
                }
                _ => champ.push(c),
            }
        }
    }
    if !champ.is_empty() || !ligne.is_empty() {
        ligne.push(champ);
        lignes.push(ligne);
    }
    lignes
}
