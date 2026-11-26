//! Import d'une liste de deck écrite librement, en particulier l'**export
//! texte de FaBrary** (lot 4a du rapatriement d'Arsenal) :
//!
//! ```text
//! Name: Hala aggro
//! Hero: Hala
//! Format: Silver Age
//!
//! Arena cards
//! 1x Zenith Blade
//! 1x Helm of Unity
//!
//! Deck cards
//! 2x Edict of Steel (red)
//! ```
//!
//! Une liste déjà au format de `data/decks/` (avec `Arena:` et `Deck:`) est
//! lue telle quelle. Sinon, chaque ligne « N Nom » ou « Nx Nom » est une
//! carte ; une carte avec sa couleur va dans le deck, une carte sans couleur
//! est classée (héros ou emplacement d'arène) par la fonction `classer`,
//! qui consulte la base de cartes. Les lignes d'en-tête connues (Name,
//! Format, « Made with »…) sont ignorées.

use crate::decklist::{ArenaEntry, Color, DeckEntry, Decklist};

/// Ce qu'est une carte sans couleur, d'après la base de cartes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Classement {
    /// Un héros, avec sa classe (« Warrior »…), si on la connaît.
    Heros(String),
    /// Une carte d'arène, avec son emplacement (`Weapon`, `Head`, `Chest`,
    /// `Arms`, `Legs`, `Off-Hand`, `Quiver`).
    Arene(String),
    /// Une carte de deck (sans couleur dans la liste).
    Deck,
    /// Carte inconnue de la base.
    Inconnue,
}

fn couleur(s: &str) -> Option<Color> {
    match s.to_ascii_lowercase().as_str() {
        "red" | "rouge" | "r" => Some(Color::Red),
        "yellow" | "jaune" | "y" => Some(Color::Yellow),
        "blue" | "bleu" | "b" => Some(Color::Blue),
        _ => None,
    }
}

/// « 2x Nom », « 2 x Nom », « 2 Nom » → (2, « Nom »).
fn quantite(ligne: &str) -> Option<(u8, &str)> {
    let fin = ligne.find(|c: char| !c.is_ascii_digit())?;
    if fin == 0 {
        return None;
    }
    let n: u8 = ligne[..fin].parse().ok()?;
    let apres = &ligne[fin..];
    let reste = match apres.strip_prefix(['x', 'X']) {
        Some(r) => r,
        None => {
            let t = apres.trim_start();
            t.strip_prefix("x ")
                .or_else(|| t.strip_prefix("X "))
                .unwrap_or(t)
        }
    }
    .trim();
    (!reste.is_empty()).then_some((n, reste))
}

/// « Nom (red) » → (« Nom », rouge).
fn avec_couleur(nom: &str) -> (String, Option<Color>) {
    if let Some((n, c)) = nom.strip_suffix(')').and_then(|r| r.rsplit_once(" (")) {
        if let Some(c) = couleur(c.trim()) {
            return (n.trim().to_string(), Some(c));
        }
    }
    (nom.trim().to_string(), None)
}

const ENTETES_IGNORES: [&str; 12] = [
    "name",
    "format",
    "arena",
    "deck",
    "equipment",
    "weapon",
    "weapons",
    "cards",
    "sideboard",
    "inventory",
    "made with",
    "see the full",
];

/// Lit une liste libre (export FaBrary ou format de `data/decks/`).
/// Renvoie la liste et les remarques (lignes ignorées, cartes inconnues).
pub fn lire(
    texte: &str,
    classer: impl Fn(&str) -> Classement,
) -> Result<(Decklist, Vec<String>), String> {
    if texte.lines().any(|l| l.trim() == "Arena:") && texte.lines().any(|l| l.trim() == "Deck:") {
        return Decklist::parse(texte)
            .map(|d| (d, Vec::new()))
            .map_err(|e| e.to_string());
    }
    let mut d = Decklist {
        title: String::new(),
        hero: String::new(),
        class: String::new(),
        source: String::new(),
        arena: Vec::new(),
        deck: Vec::new(),
    };
    let mut remarques = Vec::new();
    let mut heros_ecrit: Option<String> = None;
    for (i, brut) in texte.lines().enumerate() {
        let ligne = brut.trim().trim_matches(',').trim();
        if ligne.is_empty() || ligne.starts_with("//") {
            continue;
        }
        if let Some(t) = ligne.strip_prefix('#') {
            if d.title.is_empty() {
                d.title = t.trim().to_string();
            }
            continue;
        }
        if let Some((cle, valeur)) = ligne.split_once(':') {
            let cle = cle.trim().to_ascii_lowercase();
            let valeur = valeur.trim();
            match cle.as_str() {
                "hero" => {
                    heros_ecrit = Some(valeur.to_string());
                    continue;
                }
                "name" if !valeur.is_empty() => {
                    d.title = valeur.to_string();
                    continue;
                }
                "class" => {
                    d.class = valeur.to_string();
                    continue;
                }
                "source" => {
                    d.source = valeur.to_string();
                    continue;
                }
                _ => {}
            }
        }
        let bas = ligne.to_ascii_lowercase();
        if ENTETES_IGNORES.iter().any(|e| bas.starts_with(e)) || bas.starts_with("http") {
            continue;
        }
        let Some((n, reste)) = quantite(ligne) else {
            remarques.push(format!("ligne {} ignorée : « {ligne} »", i + 1));
            continue;
        };
        if n == 0 {
            continue;
        }
        let (nom, c) = avec_couleur(reste);
        match c {
            Some(color) => d.deck.push(DeckEntry {
                count: n,
                name: nom,
                color,
            }),
            None => match classer(&nom) {
                Classement::Heros(classe) => {
                    d.hero = nom;
                    if d.class.is_empty() {
                        d.class = classe;
                    }
                }
                Classement::Arene(slot) => d.arena.push(ArenaEntry {
                    count: n,
                    name: nom,
                    slot,
                }),
                Classement::Deck => return Err(format!(
                    "ligne {} : « {nom} » est une carte de deck, sa couleur manque (ex. « (red) »)",
                    i + 1
                )),
                Classement::Inconnue => {
                    return Err(format!("ligne {} : carte inconnue « {nom} »", i + 1))
                }
            },
        }
    }
    if let Some(h) = heros_ecrit {
        // Le nom exact : « Hala, Bladesaint of the Vow » (adulte) n'est pas
        // « Hala » (jeune), on ne remplace pas l'un par l'autre.
        match classer(&h) {
            Classement::Heros(classe) => {
                if d.class.is_empty() {
                    d.class = classe;
                }
                d.hero = h;
            }
            _ => {
                return Err(format!(
                    "héros inconnu : « {h} » (Silver Age : un héros « Young », nom exact)"
                ))
            }
        }
    }
    if d.hero.is_empty() {
        return Err("héros manquant (ligne « Hero: … »)".into());
    }
    if d.deck.is_empty() {
        return Err("aucune carte de deck (avec sa couleur, ex. « 2x Nom (red) »)".into());
    }
    Ok((d, remarques))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classer(nom: &str) -> Classement {
        match nom {
            "Hala" => Classement::Heros("Warrior".into()),
            "Zenith Blade" => Classement::Arene("Weapon".into()),
            "Helm of Unity" => Classement::Arene("Head".into()),
            "Edict of Steel" => Classement::Deck,
            _ => Classement::Inconnue,
        }
    }

    const FABRARY: &str = "Name: Hala test\nHero: Hala\nFormat: Silver Age\n\n\
        Arena cards\n1x Zenith Blade\n1x Helm of Unity\n\nDeck cards\n2x Edict of Steel (red)\n\
        1x Edict of Steel (yellow)\n\nMade with ❤️ at https://fabrary.net\n";

    #[test]
    fn l_export_texte_de_fabrary_est_lu() {
        let (d, r) = lire(FABRARY, classer).unwrap();
        assert_eq!(d.title, "Hala test");
        assert_eq!(d.hero, "Hala");
        assert_eq!(d.class, "Warrior");
        assert_eq!(d.arena.len(), 2);
        assert_eq!(d.arena[0].slot, "Weapon");
        assert_eq!(d.deck_count(), 3);
        assert_eq!(d.deck[1].color, Color::Yellow);
        assert!(r.is_empty(), "{r:?}");
    }

    #[test]
    fn le_format_des_fichiers_du_projet_est_lu_tel_quel() {
        let t = "Hero: Hala\nArena:\n1 Zenith Blade [Weapon]\nDeck:\n2 Edict of Steel (red)\n";
        let (d, _) = lire(t, classer).unwrap();
        assert_eq!(d, Decklist::parse(t).unwrap());
    }

    #[test]
    fn une_carte_inconnue_ou_sans_couleur_est_signalee() {
        let e = lire("Hero: Hala\n1x Machin\n", classer).unwrap_err();
        assert!(e.contains("inconnue"), "{e}");
        let e = lire("Hero: Hala\n2x Edict of Steel\n", classer).unwrap_err();
        assert!(e.contains("couleur manque"), "{e}");
    }

    #[test]
    fn un_heros_inconnu_ou_absent_est_refuse() {
        assert!(lire("Hero: Personne\n2x Edict of Steel (red)\n", classer).is_err());
        // Le héros adulte n'est pas remplacé par le jeune du même nom.
        assert!(lire(
            "Hero: Hala, Bladesaint of the Vow\n2x Edict of Steel (red)\n",
            classer
        )
        .is_err());
        assert!(lire("2x Edict of Steel (red)\n", classer).is_err());
    }

    #[test]
    fn les_formes_de_quantite_sont_acceptees() {
        assert_eq!(quantite("2x Nom"), Some((2, "Nom")));
        assert_eq!(quantite("2 x Nom"), Some((2, "Nom")));
        assert_eq!(quantite("2 Nom"), Some((2, "Nom")));
        assert_eq!(quantite("Nom"), None);
        assert_eq!(quantite("3"), None);
    }
}
