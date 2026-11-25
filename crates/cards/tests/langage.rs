//! Le langage de cartes : fichiers lisibles, et texte réécrit identique au
//! texte officiel (si le fichier officiel a été téléchargé).

use fab_cards::langage::officiel::Base;
use fab_cards::langage::{self, ProgramId};

#[test]
fn langage_fichiers_valides() {
    let f = langage::program(ProgramId::Warrior);
    assert_eq!(f.cards.len(), 25);
}

#[test]
fn langage_texte_reecrit_identique_au_texte_officiel() {
    let Some(base) = Base::charger() else {
        eprintln!("fichier officiel absent : test sauté (bash tools/fetch_cards.sh)");
        return;
    };
    let f = langage::program(ProgramId::Warrior);
    for card in &f.cards {
        let colors: Vec<(usize, &str)> = if card.colors.is_empty() {
            vec![(0, "")]
        } else {
            card.colors
                .iter()
                .map(|c| match c {
                    fab_engine::card::Color::Red => (0, "red"),
                    fab_engine::card::Color::Yellow => (1, "yellow"),
                    fab_engine::card::Color::Blue => (2, "blue"),
                })
                .collect()
        };
        for (ci, col) in colors {
            let o = base
                .get(&card.name, col)
                .unwrap_or_else(|| panic!("{} {col} absente", card.name));
            assert_eq!(
                langage::render(f, card, ci),
                langage::normalize_official(&o.textbox),
                "{} {col}",
                card.name
            );
            assert_eq!(card.typebox, o.typebox, "{}", card.name);
        }
    }
}
