//! Card-pool et choix des cartes avant une partie (CR 4.1.4 à 4.1.7).
//!
//! En Silver Age (TRP 7.4), le card-pool d'un joueur comprend un héros et au
//! plus 55 cartes d'arène et de deck. **Avant chaque partie**, le joueur y
//! choisit ses cartes d'arène (CR 4.1.4), puis les cartes de son deck
//! (CR 4.1.6) : exactement 40 en Silver Age (`FormatConfig::deck_size`).
//! Les cartes non choisies forment son inventaire (CR 4.1.7).
//!
//! Ce module ne fait que **vérifier** un choix ; qui choisit et comment
//! (bot ou humain) est l'affaire de `fab-match`, `fab-ai` et `fab-ui`.

use crate::card::{CardDb, Subtypes, Types};
use crate::ids::CardId;
use crate::setup::Loadout;
use crate::state::equipment_zone;
use crate::zones::Zone;

/// Le card-pool d'un joueur : un exemplaire par élément (deux exemplaires
/// d'une carte = deux éléments identiques).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardPool {
    pub hero: CardId,
    /// Cartes d'arène (armes, équipements), dans l'ordre de la liste.
    pub arena: Vec<CardId>,
    /// Cartes qui peuvent aller dans le deck, dans l'ordre de la liste.
    pub deck: Vec<CardId>,
}

/// Pourquoi un choix d'arène ou de deck n'est pas légal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoixInvalide {
    /// La carte n'est pas dans le card-pool, ou en trop d'exemplaires.
    HorsDuPool(CardId),
    /// CR 4.1.4 : seules les cartes d'arène vont dans les zones d'arène.
    PasUneCarteDArene(CardId),
    /// CR 4.1.6a : les cartes d'arène ne peuvent pas aller dans le deck.
    PasUneCarteDeDeck(CardId),
    /// CR 4.1.4a : au plus une carte par zone ; CR 8.2.2b : deux mains au
    /// plus pour les armes ; CR 8.2.10b : un seul off-hand.
    ZoneOccupee(CardId),
    /// TRP 7.4 : « exactly 40 cards in their deck ».
    TailleDuDeck { attendu: usize, recu: usize },
}

/// Nombre de mains occupées par une carte de la zone d'armes (CR 8.2.2b).
fn mains(db: &CardDb, c: CardId) -> u8 {
    if db.get(c).subtypes.has(Subtypes::TWO_HAND) {
        2
    } else {
        1
    }
}

/// Les cartes d'arène `arena` peuvent-elles être équipées ensemble ?
/// Renvoie la première carte qui ne trouve pas de place.
///
/// CR 4.1.4a : au plus une carte par zone de tête, torse, bras et jambes ;
/// les armes et off-hands se partagent deux mains (CR 8.2.2b), avec un seul
/// off-hand (CR 8.2.10b).
pub fn arena_compatible(db: &CardDb, arena: &[CardId]) -> Result<(), ChoixInvalide> {
    let mut mains_prises = 0;
    let mut off_hands = 0;
    let mut zones: Vec<Zone> = Vec::new();
    for &c in arena {
        let def = db.get(c);
        if def.types.is_deck_card() || def.types.has(Types::HERO) {
            return Err(ChoixInvalide::PasUneCarteDArene(c));
        }
        let zone = equipment_zone(def);
        let conflit = match zone {
            Zone::Weapon => {
                mains_prises += mains(db, c);
                if def.subtypes.has(Subtypes::OFF_HAND) {
                    off_hands += 1;
                }
                mains_prises > 2 || off_hands > 1
            }
            Zone::Head | Zone::Chest | Zone::Arms | Zone::Legs => {
                let deja = zones.contains(&zone);
                zones.push(zone);
                deja
            }
            _ => return Err(ChoixInvalide::PasUneCarteDArene(c)),
        };
        if conflit {
            return Err(ChoixInvalide::ZoneOccupee(c));
        }
    }
    Ok(())
}

/// `choix` est-il inclus dans `pool`, exemplaire par exemplaire ?
fn inclus(choix: &[CardId], pool: &[CardId]) -> Result<(), ChoixInvalide> {
    let mut reste = pool.to_vec();
    for &c in choix {
        match reste.iter().position(|&x| x == c) {
            Some(i) => {
                reste.swap_remove(i);
            }
            None => return Err(ChoixInvalide::HorsDuPool(c)),
        }
    }
    Ok(())
}

impl CardPool {
    /// Vérifie un choix de cartes d'arène (CR 4.1.4).
    pub fn verifier_arene(&self, db: &CardDb, arena: &[CardId]) -> Result<(), ChoixInvalide> {
        inclus(arena, &self.arena)?;
        arena_compatible(db, arena)
    }

    /// Vérifie un choix de deck (CR 4.1.6, TRP 7.4).
    pub fn verifier_deck(
        &self,
        db: &CardDb,
        deck: &[CardId],
        deck_size: usize,
    ) -> Result<(), ChoixInvalide> {
        if let Some(&c) = deck.iter().find(|&&c| !db.get(c).types.is_deck_card()) {
            return Err(ChoixInvalide::PasUneCarteDeDeck(c));
        }
        inclus(deck, &self.deck)?;
        if deck.len() != deck_size {
            return Err(ChoixInvalide::TailleDuDeck {
                attendu: deck_size,
                recu: deck.len(),
            });
        }
        Ok(())
    }

    /// Le `Loadout` d'une partie après vérification du choix. Le card-pool
    /// de deck est gardé dans `Loadout::pool` (les bots y imaginent les
    /// cartes cachées de l'adversaire).
    pub fn loadout(
        &self,
        db: &CardDb,
        arena: Vec<CardId>,
        deck: Vec<CardId>,
        deck_size: usize,
    ) -> Result<Loadout, ChoixInvalide> {
        self.verifier_arene(db, &arena)?;
        self.verifier_deck(db, &deck, deck_size)?;
        Ok(Loadout {
            hero: self.hero,
            arena,
            deck,
            pool: self.deck.clone(),
        })
    }

    /// Choix « dans l'ordre de la liste » : chaque carte d'arène est prise
    /// si elle trouve encore une place (donc la première de chaque zone, et
    /// deux armes à une main si elles sont listées avant une arme à deux
    /// mains), et les `deck_size` premières cartes de deck.
    pub fn choix_par_ordre(&self, db: &CardDb, deck_size: usize) -> (Vec<CardId>, Vec<CardId>) {
        let mut arena: Vec<CardId> = Vec::new();
        for &c in &self.arena {
            arena.push(c);
            if arena_compatible(db, &arena).is_err() {
                arena.pop();
            }
        }
        let deck = self.deck.iter().copied().take(deck_size).collect();
        (arena, deck)
    }
}
