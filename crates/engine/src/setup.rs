//! Mise en place d'une partie (CR 4.1, « start-of-game procedure »).
//!
//! Étape 1 : les cartes d'arène et le deck sont fournis déjà choisis
//! ([`Loadout`]). Le choix de l'équipement et du deck dans le card-pool de 55
//! cartes (CR 4.1.4 à 4.1.7) sera une décision du joueur à l'étape 6.

use crate::card::{CardDb, Subtypes, Types};
use crate::decision::{Choice, Decision, DecisionKind};
use crate::ids::{CardId, ObjectId, PlayerId};
use crate::rng::Rng;
use crate::state::{equipment_zone, Combat, GameState, Object, PlayerState, Stage};
use crate::zones::Zone;
use std::sync::Arc;

/// Ce avec quoi un joueur commence la partie.
#[derive(Clone, Debug)]
pub struct Loadout {
    pub hero: CardId,
    /// Armes et équipements (CR 4.1.4) : au plus un par zone.
    pub arena: Vec<CardId>,
    /// Le deck de départ. Si la partie ne mélange pas, le **premier** élément
    /// est le dessus du deck (pratique pour les tests).
    pub deck: Vec<CardId>,
}

#[derive(Clone, Debug)]
pub struct GameSetup {
    pub players: [Loadout; 2],
    /// Joueur qui choisit le premier joueur (CR 4.1.3).
    pub chooser: PlayerId,
    pub seed: u64,
    /// Mélanger les decks (CR 4.1.8). `false` uniquement pour les tests.
    pub shuffle: bool,
    /// Sécurité des simulations : partie nulle au-delà de ce nombre de tours.
    pub max_turns: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupError {
    NotAHero(PlayerId),
    NotAnArenaCard(PlayerId, CardId),
    NotADeckCard(PlayerId, CardId),
    /// Deux objets pour la même zone, ou plus de deux mains d'armes (CR 8.5.41c, 8.2).
    ZoneConflict(PlayerId, CardId),
}

impl GameState {
    /// Crée une partie et avance jusqu'à la première décision.
    pub fn new(db: Arc<CardDb>, setup: GameSetup) -> Result<GameState, SetupError> {
        let mut objects = Vec::new();
        let mut players = Vec::new();
        for p in PlayerId::BOTH {
            let lo = &setup.players[p.index()];
            let mut new_obj = |card: CardId, zone: Zone| {
                objects.push(Object {
                    card,
                    owner: p,
                    controller: p,
                    zone,
                    gen: 0,
                    face_down: matches!(zone, Zone::Deck),
                    tapped: false,
                    defense_counters: 0,
                    power_counters: 0,
                    counters: 0,
                    proxy_of: None,
                    activations: [0; 4],
                });
                ObjectId(objects.len() as u32 - 1)
            };
            // CR 4.1.2 : le héros dans sa zone de héros.
            let hero_def = db.get(lo.hero);
            if !hero_def.types.has(Types::HERO) {
                return Err(SetupError::NotAHero(p));
            }
            let hero = new_obj(lo.hero, Zone::Hero);
            let mut ps = PlayerState {
                hero,
                life: hero_def.life.unwrap_or(0) as i32,
                action_points: 0,
                resource_points: 0,
                deck: Vec::new(),
                hand: Vec::new(),
                arsenal: None,
                pitch: Vec::new(),
                graveyard: Vec::new(),
                banished: Vec::new(),
                weapons: Vec::new(),
                head: None,
                chest: None,
                arms: None,
                legs: None,
                marked: false,
            };
            // CR 4.1.4-4.1.5 : cartes d'arène équipées.
            let mut hands_used = 0;
            let mut off_hands = 0;
            for &c in &lo.arena {
                let def = db.get(c);
                if def.types.is_deck_card() || def.types.has(Types::HERO) {
                    return Err(SetupError::NotAnArenaCard(p, c));
                }
                let zone = equipment_zone(def);
                let id = new_obj(c, zone);
                let conflict = match zone {
                    Zone::Weapon => {
                        let hands = if def.subtypes.has(Subtypes::TWO_HAND) {
                            2
                        } else {
                            1
                        };
                        hands_used += hands;
                        if def.subtypes.has(Subtypes::OFF_HAND) {
                            off_hands += 1;
                        }
                        ps.weapons.push(id);
                        hands_used > 2 || off_hands > 1 // CR 8.2.2b, 8.2.10b
                    }
                    Zone::Head => ps.head.replace(id).is_some(),
                    Zone::Chest => ps.chest.replace(id).is_some(),
                    Zone::Arms => ps.arms.replace(id).is_some(),
                    Zone::Legs => ps.legs.replace(id).is_some(),
                    _ => true,
                };
                if conflict {
                    return Err(SetupError::ZoneConflict(p, c));
                }
            }
            // CR 4.1.6 : le deck.
            for &c in lo.deck.iter().rev() {
                if !db.get(c).types.is_deck_card() {
                    return Err(SetupError::NotADeckCard(p, c));
                }
                let id = new_obj(c, Zone::Deck);
                ps.deck.push(id);
            }
            players.push(ps);
        }
        let [p0, p1]: [PlayerState; 2] = players.try_into().unwrap();
        let mut g = GameState {
            db,
            rng: Rng::new(setup.seed),
            objects,
            players: [p0, p1],
            permanents: Vec::new(),
            turn: 0,
            turn_player: PlayerId::P0,
            first_player: PlayerId::P0,
            chooser: setup.chooser,
            stage: Stage::ChooseFirstPlayer,
            priority: None,
            passes: 0,
            stack: Vec::new(),
            combat: Combat::default(),
            modifiers: Vec::new(),
            next_attack_effects: Vec::new(),
            preventions: Vec::new(),
            pending_triggers: Vec::new(),
            trigger_first: None,
            tasks: Vec::new(),
            declared_defenders: Vec::new(),
            turn_log: Vec::new(),
            pending: None,
            outcome: None,
            end_reason: None,
            max_turns: setup.max_turns,
            trigger_objects: Vec::new(),
        };
        g.trigger_objects = (0..g.objects.len() as u32)
            .map(ObjectId)
            .filter(|&o| !g.def(o).triggers.is_empty())
            .collect();
        // CR 4.1.8 : chaque joueur mélange son deck.
        if setup.shuffle {
            for p in PlayerId::BOTH {
                let mut deck = std::mem::take(&mut g.player_mut(p).deck);
                g.rng.shuffle(&mut deck);
                g.player_mut(p).deck = deck;
            }
        }
        // CR 4.1.3 : le joueur choisisseur désigne le premier joueur.
        g.pending = Some(Decision {
            player: setup.chooser,
            kind: DecisionKind::FirstPlayer,
            options: vec![
                Choice::Player(setup.chooser),
                Choice::Player(setup.chooser.opponent()),
            ],
        });
        Ok(g)
    }

    /// Suite de la mise en place une fois le premier joueur choisi (CR 4.1.9-4.1.10).
    pub(crate) fn start_game(&mut self, first: PlayerId) {
        self.first_player = first;
        self.turn_player = first;
        self.turn = 1;
        // CR 4.1.10 : chaque joueur pioche jusqu'à son intellect.
        for p in PlayerId::BOTH {
            self.draw_up_to_intellect(p);
        }
        self.stage = Stage::StartPhase;
    }

    pub(crate) fn intellect(&self, p: PlayerId) -> usize {
        self.def(self.player(p).hero).intellect.unwrap_or(0) as usize
    }

    /// CR 4.4.3f / 4.1.10.
    pub(crate) fn draw_up_to_intellect(&mut self, p: PlayerId) {
        let n = self.intellect(p);
        while self.player(p).hand.len() < n {
            if !self.draw(p) {
                break;
            }
        }
    }
}
