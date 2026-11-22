//! Lecture et vérification des listes de decks (`data/decks/*.txt`).
//!
//! Format d'un fichier :
//! ```text
//! # Titre libre
//! Hero: Nom du héros
//! Class: Classe
//! Source: Tournoi, place
//!
//! Arena:
//! 1 Nom de l'équipement [Head]
//!
//! Deck:
//! 2 Nom de la carte (red)
//! ```

use fab_engine::FormatConfig;
use std::collections::BTreeMap;
use std::fmt;

/// Couleur d'une carte de deck. Elle donne sa valeur de pitch
/// (rouge 1, jaune 2, bleu 3), qui fait partie de son unicité (CR 2.7.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    Red,
    Yellow,
    Blue,
}

impl Color {
    pub fn pitch(self) -> u8 {
        match self {
            Color::Red => 1,
            Color::Yellow => 2,
            Color::Blue => 3,
        }
    }

    fn parse(s: &str) -> Option<Color> {
        match s {
            "red" => Some(Color::Red),
            "yellow" => Some(Color::Yellow),
            "blue" => Some(Color::Blue),
            _ => None,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Color::Red => "red",
            Color::Yellow => "yellow",
            Color::Blue => "blue",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArenaEntry {
    pub count: u8,
    pub name: String,
    /// Emplacement indiqué dans la liste : Weapon, Head, Chest, Arms, Legs, Off-Hand.
    pub slot: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckEntry {
    pub count: u8,
    pub name: String,
    pub color: Color,
}

/// Le card-pool d'un joueur : un héros + cartes d'arène + cartes de deck.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decklist {
    pub title: String,
    pub hero: String,
    pub class: String,
    pub source: String,
    pub arena: Vec<ArenaEntry>,
    pub deck: Vec<DeckEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {} : {}", self.line, self.message)
    }
}

impl Decklist {
    pub fn parse(text: &str) -> Result<Decklist, ParseError> {
        #[derive(PartialEq)]
        enum Section {
            Header,
            Arena,
            Deck,
        }
        let mut d = Decklist {
            title: String::new(),
            hero: String::new(),
            class: String::new(),
            source: String::new(),
            arena: Vec::new(),
            deck: Vec::new(),
        };
        let mut section = Section::Header;
        for (i, raw) in text.lines().enumerate() {
            let line_no = i + 1;
            let err = |m: &str| ParseError {
                line: line_no,
                message: m.to_string(),
            };
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(t) = line.strip_prefix('#') {
                d.title = t.trim().to_string();
                continue;
            }
            match line {
                "Arena:" => {
                    section = Section::Arena;
                    continue;
                }
                "Deck:" => {
                    section = Section::Deck;
                    continue;
                }
                _ => {}
            }
            if section == Section::Header {
                let (key, value) = line
                    .split_once(':')
                    .ok_or_else(|| err("« Clé: valeur » attendu"))?;
                let value = value.trim().to_string();
                match key.trim() {
                    "Hero" => d.hero = value,
                    "Class" => d.class = value,
                    "Source" => d.source = value,
                    other => return Err(err(&format!("clé inconnue « {other} »"))),
                }
                continue;
            }
            let (count, rest) = line
                .split_once(' ')
                .ok_or_else(|| err("« N Nom » attendu"))?;
            let count: u8 = count
                .parse()
                .map_err(|_| err("nombre d'exemplaires invalide"))?;
            if count == 0 {
                return Err(err("0 exemplaire"));
            }
            match section {
                Section::Arena => {
                    let (name, slot) = rest
                        .strip_suffix(']')
                        .and_then(|r| r.rsplit_once(" ["))
                        .ok_or_else(|| err("emplacement « [Head] » attendu en fin de ligne"))?;
                    d.arena.push(ArenaEntry {
                        count,
                        name: name.trim().to_string(),
                        slot: slot.to_string(),
                    });
                }
                Section::Deck => {
                    let (name, color) = rest
                        .strip_suffix(')')
                        .and_then(|r| r.rsplit_once(" ("))
                        .ok_or_else(|| err("couleur « (red) » attendue en fin de ligne"))?;
                    let color = Color::parse(color).ok_or_else(|| err("couleur inconnue"))?;
                    d.deck.push(DeckEntry {
                        count,
                        name: name.trim().to_string(),
                        color,
                    });
                }
                Section::Header => unreachable!(),
            }
        }
        if d.hero.is_empty() {
            return Err(ParseError {
                line: 0,
                message: "héros manquant".into(),
            });
        }
        Ok(d)
    }

    pub fn arena_count(&self) -> u32 {
        self.arena.iter().map(|e| e.count as u32).sum()
    }

    pub fn deck_count(&self) -> u32 {
        self.deck.iter().map(|e| e.count as u32).sum()
    }

    /// Vérifie les règles de construction du format (TRP 7.4) qu'on peut
    /// contrôler **sans** la base de cartes. Les contrôles de rareté, de
    /// sous-type « young » du héros et de type de carte viendront avec la base.
    pub fn check(&self, format: &FormatConfig) -> Vec<String> {
        let mut problems = Vec::new();
        let total = self.arena_count() + self.deck_count();
        if total > format.card_pool_max as u32 {
            problems.push(format!(
                "card-pool de {total} cartes (max {})",
                format.card_pool_max
            ));
        }
        // CR 4.1.6 + TRP 7.4 : il faut pouvoir former un deck de 40 cartes exactement.
        if self.deck_count() < format.deck_size as u32 {
            problems.push(format!(
                "seulement {} cartes de deck, il en faut {}",
                self.deck_count(),
                format.deck_size
            ));
        }
        // TRP 7.4 + CR 2.7.1 : unicité = nom + pitch. Les cartes d'arène n'ont
        // pas de pitch dans ces listes : on les compte par nom.
        let mut copies: BTreeMap<(String, u8), u32> = BTreeMap::new();
        for e in &self.deck {
            *copies.entry((e.name.clone(), e.color.pitch())).or_default() += e.count as u32;
        }
        for e in &self.arena {
            *copies.entry((e.name.clone(), 0)).or_default() += e.count as u32;
        }
        for ((name, _), n) in copies {
            let allowed = format.copies_allowed(&name) as u32;
            if n > allowed {
                problems.push(format!("{n} copies de « {name} » (max {allowed})"));
            }
        }
        if format.benched_heroes.iter().any(|h| h == &self.hero) {
            problems.push(format!("héros au banc : {}", self.hero));
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXEMPLE: &str = "# Test\nHero: Héros fictif\nClass: Ninja\nSource: test\n\n\
        Arena:\n1 Épée fictive [Weapon]\n\nDeck:\n2 Coup fictif (red)\n1 Coup fictif (blue)\n";

    fn format() -> FormatConfig {
        FormatConfig::from_toml(include_str!("../../../config/silver_age.toml")).unwrap()
    }

    #[test]
    fn lit_une_liste_simple() {
        let d = Decklist::parse(EXEMPLE).unwrap();
        assert_eq!(d.hero, "Héros fictif");
        assert_eq!(d.arena_count(), 1);
        assert_eq!(d.deck_count(), 3);
        assert_eq!(d.deck[1].color, Color::Blue);
    }

    #[test]
    fn la_meme_carte_dans_deux_couleurs_compte_comme_deux_cartes_uniques() {
        // 2 rouges + 2 bleues de la même carte : légal (CR 2.7.1).
        let text = "Hero: H\nDeck:\n2 Coup (red)\n2 Coup (blue)\n";
        let d = Decklist::parse(text).unwrap();
        assert!(!d.check(&format()).iter().any(|p| p.contains("copies")));
    }

    #[test]
    fn trois_copies_sont_refusees() {
        let text = "Hero: H\nDeck:\n2 Coup (red)\n1 Coup (red)\n";
        let d = Decklist::parse(text).unwrap();
        assert!(d.check(&format()).iter().any(|p| p.contains("3 copies")));
    }

    #[test]
    fn moins_de_40_cartes_de_deck_est_refuse() {
        let d = Decklist::parse(EXEMPLE).unwrap();
        assert!(d
            .check(&format())
            .iter()
            .any(|p| p.contains("il en faut 40")));
    }

    #[test]
    fn ligne_mal_formee_donne_son_numero() {
        let e = Decklist::parse("Hero: H\nDeck:\n2 Coup sans couleur\n").unwrap_err();
        assert_eq!(e.line, 3);
    }
}
