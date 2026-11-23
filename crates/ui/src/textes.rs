//! Textes officiels des cartes, lus dans le fichier LSS
//! (`data/cards/sources/public_card_data.csv`, téléchargé par
//! `bash tools/fetch_cards.sh`, non versionné). Facultatif : sans ce fichier,
//! l'interface fonctionne mais n'affiche pas le texte des cartes.

use fab_engine::card::Color;
use std::collections::HashMap;

/// Texte officiel (colonne `face_1_true_textbox`, errata compris) et ligne
/// de type, par (nom, couleur).
#[derive(Default)]
pub struct Textes {
    cartes: HashMap<(String, String), (String, String)>,
}

fn nom_couleur(c: Option<Color>) -> &'static str {
    match c {
        Some(Color::Red) => "red",
        Some(Color::Yellow) => "yellow",
        Some(Color::Blue) => "blue",
        None => "",
    }
}

/// Découpe un CSV (RFC 4180 : champs entre guillemets, `""` pour un
/// guillemet, retours à la ligne permis dans un champ).
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

/// Rend lisible la mise en forme du fichier officiel : `{br}` → retour à la
/// ligne, `**gras**` et `_italique_` → texte simple.
pub fn nettoyer(t: &str) -> String {
    t.replace("{br}", "\n").replace("**", "").replace('_', "")
}

impl Textes {
    /// Lit le fichier officiel ; ne garde que les impressions anglaises.
    pub fn depuis_csv(texte: &str) -> Textes {
        let mut lignes = lire_csv(texte).into_iter();
        let Some(entete) = lignes.next() else {
            return Textes::default();
        };
        let col = |n: &str| entete.iter().position(|h| h == n);
        let (Some(langue), Some(nom), Some(couleur), Some(boite), Some(typ)) = (
            col("print_language"),
            col("face_1_true_name"),
            col("face_1_true_color"),
            col("face_1_true_textbox"),
            col("face_1_true_typebox"),
        ) else {
            return Textes::default();
        };
        // Face arrière d'une carte retournable (ex. Azvolai, dos d'Invoke
        // Azvolai) : colonnes facultatives `face_2_…`.
        let face2 = (
            col("face_2_true_name"),
            col("face_2_true_color"),
            col("face_2_true_textbox"),
            col("face_2_true_typebox"),
        );
        let mut cartes = HashMap::new();
        for l in lignes {
            if l.get(langue).map(String::as_str) != Some("en") {
                continue;
            }
            let (Some(n), Some(c), Some(b), Some(t)) =
                (l.get(nom), l.get(couleur), l.get(boite), l.get(typ))
            else {
                continue;
            };
            cartes
                .entry((n.clone(), c.clone()))
                .or_insert_with(|| (t.clone(), b.clone()));
            if let (Some(n2), Some(c2), Some(b2), Some(t2)) = face2 {
                if let (Some(n), Some(c), Some(b), Some(t)) =
                    (l.get(n2), l.get(c2), l.get(b2), l.get(t2))
                {
                    if !n.is_empty() {
                        cartes
                            .entry((n.clone(), c.clone()))
                            .or_insert_with(|| (t.clone(), b.clone()));
                    }
                }
            }
        }
        Textes { cartes }
    }

    /// Cherche le fichier officiel à son emplacement habituel.
    pub fn charger() -> Option<Textes> {
        let chemin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/cards/sources/public_card_data.csv");
        let texte = std::fs::read_to_string(chemin).ok()?;
        Some(Textes::depuis_csv(&texte))
    }

    pub fn len(&self) -> usize {
        self.cartes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cartes.is_empty()
    }

    /// Ligne de type et texte officiel nettoyé.
    pub fn texte(&self, nom: &str, couleur: Option<Color>) -> Option<(String, String)> {
        self.cartes
            .get(&(nom.to_string(), nom_couleur(couleur).to_string()))
            .map(|(t, b)| (t.clone(), nettoyer(b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_avec_guillemets_virgules_et_retours_a_la_ligne() {
        let l = lire_csv("a,b,c\n1,\"x, \"\"y\"\"\nz\",3\n");
        assert_eq!(l.len(), 2);
        assert_eq!(l[1], vec!["1", "x, \"y\"\nz", "3"]);
    }

    #[test]
    fn texte_officiel_retrouve_par_nom_et_couleur() {
        let csv = "print_language,face_1_true_name,face_1_true_color,face_1_true_typebox,face_1_true_textbox\n\
                   de,Edict of Steel,red,Warrior Action,Deutsch\n\
                   en,Edict of Steel,red,Warrior Action,\"**Sharpen** target sword.{br}**Go again**\"\n";
        let t = Textes::depuis_csv(csv);
        let (typ, texte) = t.texte("Edict of Steel", Some(Color::Red)).unwrap();
        assert_eq!(typ, "Warrior Action");
        assert_eq!(texte, "Sharpen target sword.\nGo again");
        assert!(t.texte("Edict of Steel", Some(Color::Blue)).is_none());
    }
}
