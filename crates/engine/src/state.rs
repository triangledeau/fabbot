//! L'état complet d'une partie.

use crate::card::{CardDb, CardDef, Keywords};
use crate::decision::Decision;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::rng::Rng;
use crate::zones::Zone;
use std::sync::Arc;

/// Un objet de la partie : une carte (ou une attaque-proxy d'arme).
#[derive(Clone, Debug)]
pub struct Object {
    pub card: CardId,
    pub owner: PlayerId,
    pub controller: PlayerId,
    pub zone: Zone,
    /// Génération (CR 3.0.9) : change quand l'objet devient un nouvel objet.
    pub gen: u32,
    /// Face cachée (privé), ex. arsenal.
    pub face_down: bool,
    pub tapped: bool,
    /// Somme des compteurs +1/-1 de défense (CR 1.15.2a).
    pub defense_counters: i8,
    pub power_counters: i8,
    /// Pour une attaque-proxy : l'arme qu'elle représente (CR 1.4.3).
    pub proxy_of: Option<ObjectId>,
    /// Nombre d'activations de capacité ce tour (limite « Once per Turn »).
    pub activations_this_turn: u8,
}

impl Object {
    pub fn obj_ref(&self, id: ObjectId) -> ObjRef {
        ObjRef { id, gen: self.gen }
    }
}

/// Les zones et ressources d'un joueur.
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub hero: ObjectId,
    /// Total de vie du héros (CR 2.5.3).
    pub life: i32,
    pub action_points: u8,
    pub resource_points: u8,
    /// Le dessus du deck est le **dernier** élément.
    pub deck: Vec<ObjectId>,
    pub hand: Vec<ObjectId>,
    pub arsenal: Option<ObjectId>,
    pub pitch: Vec<ObjectId>,
    pub graveyard: Vec<ObjectId>,
    pub banished: Vec<ObjectId>,
    /// Armes et off-hand équipés (une arme 2H n'apparaît qu'une fois).
    pub weapons: Vec<ObjectId>,
    pub head: Option<ObjectId>,
    pub chest: Option<ObjectId>,
    pub arms: Option<ObjectId>,
    pub legs: Option<ObjectId>,
}

impl PlayerState {
    /// Les équipements (hors armes) équipés.
    pub fn equipment(&self) -> impl Iterator<Item = ObjectId> + '_ {
        [self.head, self.chest, self.arms, self.legs]
            .into_iter()
            .flatten()
    }
}

/// Type de couche sur la pile (CR 1.6.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    /// Une carte jouée.
    Card,
    /// Attaque-proxy créée par la capacité d'attaque d'une arme (CR 1.4.3).
    WeaponAttack,
    /// Couche déclenchée « when this attacks ».
    TriggerOnAttack,
    /// Couche déclenchée « when this hits ».
    TriggerOnHit,
}

/// Une couche sur la pile.
#[derive(Clone, Copy, Debug)]
pub struct Layer {
    pub kind: LayerKind,
    /// La carte (ou la proxy) sur la pile ; pour un déclenchement, sa source.
    pub object: ObjRef,
    /// Définition utilisée pour les effets (reste valable si la source disparaît, CR 1.7.1a).
    pub card: CardId,
    pub controller: PlayerId,
    /// Cible de l'attaque (CR 1.4.5) ou du déclenchement.
    pub target: Option<PlayerId>,
    /// Zone d'où la carte a été jouée (main, arsenal...).
    pub origin: Zone,
}

/// Les étapes du combat (CR 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatStep {
    Layer,
    Attack,
    Defend,
    Reaction,
    Damage,
    Resolution,
}

/// Un maillon de la chaîne de combat (CR 7.0.3).
#[derive(Clone, Debug)]
pub struct ChainLink {
    /// L'attaque active (carte ou proxy).
    pub attack: ObjRef,
    /// L'arme représentée, pour une attaque-proxy.
    pub source: Option<ObjectId>,
    pub attacker: PlayerId,
    pub target: PlayerId,
    pub defenders: Vec<ObjectId>,
    /// Nombre de cartes venues de la main qui défendent (pour dominate).
    pub defenders_from_hand: u8,
    pub hit: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Combat {
    pub open: bool,
    pub step: Option<CombatStep>,
    pub links: Vec<ChainLink>,
    /// File d'attente des attaques (CR 3.15.7) : (attaque, cible).
    pub queue: Vec<(ObjRef, PlayerId)>,
}

impl Combat {
    pub fn active_link(&self) -> Option<&ChainLink> {
        match self.step {
            Some(
                CombatStep::Attack | CombatStep::Defend | CombatStep::Reaction | CombatStep::Damage,
            ) => self.links.last(),
            _ => None,
        }
    }
}

/// Durée d'un effet continu (CR 6.2.2a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Until {
    EndOfTurn,
    CombatChainCloses,
}

/// Modification d'un objet par un effet continu : « gets +N », « gets go again ».
#[derive(Clone, Copy, Debug)]
pub struct Modifier {
    pub target: ObjRef,
    pub power: i16,
    pub defense: i16,
    pub grants: Keywords,
    pub until: Until,
}

/// Effet « votre prochaine attaque ce tour … » en attente (CR 1.8.10).
#[derive(Clone, Copy, Debug)]
pub struct NextAttackEffect {
    pub player: PlayerId,
    pub power: i16,
}

/// Où en est le déroulement de la partie (hors fenêtre de priorité).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Choix du premier joueur (CR 4.1.3) : décision en attente.
    ChooseFirstPlayer,
    /// Début de tour (CR 4.2).
    StartPhase,
    /// Un joueur a ou va recevoir la priorité (CR 1.11).
    Priority,
    /// Paiement d'un coût en cours (CR 5.1.7).
    Paying,
    /// Déclaration des cartes qui défendent (CR 7.3.2).
    Defending,
    /// Phase de fin, mise en arsenal (CR 4.4.3b).
    EndArsenal,
    /// Phase de fin, pitch sous le deck, joueur donné (CR 4.4.3c).
    EndPitchOrder(PlayerId),
    /// Suite et fin de la phase de fin (CR 4.4.3d-f, 4.4.4).
    EndRest,
    GameOver,
}

/// Paiement d'un coût en cours.
#[derive(Clone, Copy, Debug)]
pub struct Payment {
    pub player: PlayerId,
    /// Ressources restant à payer.
    pub resources: u8,
    /// Points d'action à payer après les ressources (CR 1.14.2a).
    pub action_points: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Winner(PlayerId),
    Draw,
}

/// Pourquoi la partie s'est terminée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    /// Un héros est tombé à 0 PV (CR 4.5.3a) ou les deux (CR 4.5.4a).
    LifeZero,
    /// Limite de tours du simulateur atteinte : **pas une règle du jeu**,
    /// une sécurité pour les simulations (équivalent d'une fin de ronde).
    TurnLimit,
}

/// Erreur renvoyée quand on donne une réponse impossible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    NoPendingDecision,
    InvalidChoice { index: usize, available: usize },
}

/// L'état complet d'une partie. Il doit rester **peu coûteux à copier**,
/// car l'annulation et la recherche du bot en font beaucoup. La base de cartes
/// est immuable et partagée (`Arc`), elle n'est pas recopiée.
#[derive(Clone, Debug)]
pub struct GameState {
    pub db: Arc<CardDb>,
    pub rng: Rng,
    pub objects: Vec<Object>,
    pub players: [PlayerState; 2],
    pub turn: u32,
    pub turn_player: PlayerId,
    pub first_player: PlayerId,
    /// Joueur qui choisit le premier joueur (CR 4.1.3).
    pub chooser: PlayerId,
    pub stage: Stage,
    /// Joueur ayant la priorité (CR 1.11.2).
    pub priority: Option<PlayerId>,
    /// Nombre de « passe » consécutifs (CR 1.11.4a).
    pub passes: u8,
    /// La pile : le dessus est le dernier élément (CR 3.15).
    pub stack: Vec<Layer>,
    pub combat: Combat,
    pub modifiers: Vec<Modifier>,
    pub next_attack_effects: Vec<NextAttackEffect>,
    /// Couches déclenchées créées, pas encore ajoutées à la pile (CR 6.6.6).
    pub pending_triggers: Vec<Layer>,
    pub payment: Option<Payment>,
    /// Cartes déclarées pendant la déclaration de défense en cours.
    pub declared_defenders: Vec<ObjectId>,
    pub pending: Option<Decision>,
    pub outcome: Option<Outcome>,
    pub end_reason: Option<EndReason>,
    pub max_turns: u32,
}

impl GameState {
    #[inline]
    pub fn obj(&self, id: ObjectId) -> &Object {
        &self.objects[id.0 as usize]
    }

    #[inline]
    pub fn obj_mut(&mut self, id: ObjectId) -> &mut Object {
        &mut self.objects[id.0 as usize]
    }

    #[inline]
    pub fn def(&self, id: ObjectId) -> &CardDef {
        self.db.get(self.obj(id).card)
    }

    #[inline]
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p.index()]
    }

    #[inline]
    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p.index()]
    }

    pub fn obj_ref(&self, id: ObjectId) -> ObjRef {
        self.obj(id).obj_ref(id)
    }

    /// L'objet référencé existe-t-il encore sous cette forme (CR 3.0.9) ?
    pub fn is_current(&self, r: ObjRef) -> bool {
        let o = self.obj(r.id);
        o.gen == r.gen && o.zone != Zone::Gone
    }

    pub fn pending_decision(&self) -> Option<&Decision> {
        self.pending.as_ref()
    }

    pub fn is_over(&self) -> bool {
        self.outcome.is_some()
    }

    /// Répond à la décision en attente en choisissant l'option `index`,
    /// puis fait avancer la partie jusqu'à la prochaine décision.
    pub fn apply(&mut self, index: usize) -> Result<(), ApplyError> {
        let decision = self.pending.take().ok_or(ApplyError::NoPendingDecision)?;
        let Some(&choice) = decision.options.get(index) else {
            let available = decision.options.len();
            self.pending = Some(decision);
            return Err(ApplyError::InvalidChoice { index, available });
        };
        self.answer(decision.player, decision.kind, choice);
        self.advance();
        Ok(())
    }

    // ------------------------------------------------------------------
    // Déplacements d'objets (CR 3.0.7)
    // ------------------------------------------------------------------

    /// Retire l'objet de son conteneur actuel (sans changer `zone`).
    pub(crate) fn detach(&mut self, id: ObjectId) {
        let o = self.obj(id);
        let owner = o.owner;
        let zone = o.zone;
        let p = self.player_mut(owner);
        let remove = |v: &mut Vec<ObjectId>| {
            if let Some(i) = v.iter().position(|x| *x == id) {
                v.remove(i);
            }
        };
        match zone {
            Zone::Deck => remove(&mut p.deck),
            Zone::Hand => remove(&mut p.hand),
            Zone::Pitch => remove(&mut p.pitch),
            Zone::Graveyard => remove(&mut p.graveyard),
            Zone::Banished => remove(&mut p.banished),
            Zone::Arsenal => {
                if p.arsenal == Some(id) {
                    p.arsenal = None;
                }
            }
            // Les équipements et armes restent « équipés » quand ils sont sur
            // la chaîne de combat (CR 7.7.5) ; on ne libère leur emplacement
            // que lorsqu'ils quittent l'arène (voir `move_to`).
            Zone::Weapon | Zone::Head | Zone::Chest | Zone::Arms | Zone::Legs => {}
            Zone::Hero | Zone::CombatChain | Zone::Stack | Zone::Permanent | Zone::Gone => {}
        }
    }

    /// Libère l'emplacement d'équipement de l'objet s'il en occupait un.
    fn unequip(&mut self, id: ObjectId) {
        let owner = self.obj(id).owner;
        let p = self.player_mut(owner);
        p.weapons.retain(|x| *x != id);
        for slot in [&mut p.head, &mut p.chest, &mut p.arms, &mut p.legs] {
            if *slot == Some(id) {
                *slot = None;
            }
        }
    }

    /// Déplace un objet vers une zone (hors pile, chaîne de combat et
    /// équipement, qui ont leurs propres fonctions). Applique CR 3.0.9.
    pub(crate) fn move_to(&mut self, id: ObjectId, zone: Zone, on_top: bool) {
        self.detach(id);
        if !zone.in_arena() {
            self.unequip(id);
        }
        let owner = self.obj(id).owner;
        {
            let p = self.player_mut(owner);
            match zone {
                Zone::Deck => {
                    if on_top {
                        p.deck.push(id)
                    } else {
                        p.deck.insert(0, id)
                    }
                }
                Zone::Hand => p.hand.push(id),
                Zone::Pitch => p.pitch.push(id),
                Zone::Graveyard => p.graveyard.push(id),
                Zone::Banished => p.banished.push(id),
                Zone::Arsenal => p.arsenal = Some(id),
                _ => {}
            }
        }
        let o = self.obj_mut(id);
        // CR 3.0.9 : entrer dans une zone hors arène (sauf pile) = nouvel objet.
        if !zone.in_arena() && zone != Zone::Stack {
            o.gen += 1;
            o.defense_counters = 0;
            o.power_counters = 0;
            o.tapped = false;
            o.activations_this_turn = 0;
        }
        o.zone = zone;
        o.face_down = matches!(zone, Zone::Deck | Zone::Hand | Zone::Arsenal);
        o.controller = owner;
    }

    /// Met un objet sur la pile ou la chaîne de combat (il garde son identité).
    pub(crate) fn place(&mut self, id: ObjectId, zone: Zone, controller: PlayerId) {
        debug_assert!(matches!(zone, Zone::Stack | Zone::CombatChain));
        self.detach(id);
        let o = self.obj_mut(id);
        o.zone = zone;
        o.face_down = false;
        o.controller = controller;
    }

    /// Retourne un équipement / une arme dans sa zone d'équipement (CR 7.7.5).
    pub(crate) fn return_to_equipment_zone(&mut self, id: ObjectId) {
        let def = self.def(id);
        let zone = equipment_zone(def);
        let o = self.obj_mut(id);
        o.zone = zone;
        o.controller = o.owner;
    }

    /// « Clear » (CR 3.0.12) : au cimetière du propriétaire ; une proxy cesse d'exister.
    pub(crate) fn clear(&mut self, id: ObjectId) {
        if self.obj(id).proxy_of.is_some() || self.def(id).types.has(crate::card::Types::TOKEN) {
            self.detach(id);
            self.unequip(id);
            let o = self.obj_mut(id);
            o.zone = Zone::Gone;
            o.gen += 1;
        } else {
            self.move_to(id, Zone::Graveyard, true);
        }
    }

    // ------------------------------------------------------------------
    // Valeurs calculées (CR 2)
    // ------------------------------------------------------------------

    fn modifier_sum(&self, r: ObjRef) -> (i16, i16, Keywords) {
        let mut p = 0;
        let mut d = 0;
        let mut k = Keywords::NONE;
        for m in &self.modifiers {
            if m.target == r {
                p += m.power;
                d += m.defense;
                k = k | m.grants;
            }
        }
        (p, d, k)
    }

    /// Puissance actuelle (CR 2.9.3, jamais négative CR 2.0.3c).
    /// Une proxy hérite de la puissance modifiée de son arme (CR 1.4.3a, 1.4.3d).
    pub fn power(&self, id: ObjectId) -> i16 {
        let o = self.obj(id);
        let base = match o.proxy_of {
            Some(w) => self.power(w),
            None => self.db.get(o.card).power.unwrap_or(0) as i16 + o.power_counters as i16,
        };
        let (p, _, _) = self.modifier_sum(o.obj_ref(id));
        (base + p).max(0)
    }

    /// Défense actuelle (CR 2.3.3, jamais négative CR 2.0.3c).
    pub fn defense(&self, id: ObjectId) -> i16 {
        let o = self.obj(id);
        let base = self.db.get(o.card).defense.unwrap_or(0) as i16 + o.defense_counters as i16;
        let (_, d, _) = self.modifier_sum(o.obj_ref(id));
        (base + d).max(0)
    }

    /// Mots-clés actuels (de base + accordés par des effets).
    pub fn keywords(&self, id: ObjectId) -> Keywords {
        let o = self.obj(id);
        let base = match o.proxy_of {
            // Une proxy n'hérite pas des capacités de résolution de sa source
            // (CR 1.4.3a) ; son « go again » vient de la capacité d'attaque.
            Some(w) => {
                let wa = self.def(w).weapon_attack;
                let mut k = Keywords::NONE;
                if wa.is_some_and(|a| a.go_again) {
                    k = k | Keywords::GO_AGAIN;
                }
                // Les mots-clés statiques de l'arme (ex. dominate) sont hérités.
                k | Keywords(self.def(w).keywords.0 & !Keywords::GO_AGAIN.0)
            }
            None => self.db.get(o.card).keywords,
        };
        let (_, _, k) = self.modifier_sum(o.obj_ref(id));
        base | k
    }
}

/// Zone d'équipement correspondant au sous-type d'une carte (CR 3.2-3.16).
pub(crate) fn equipment_zone(def: &CardDef) -> Zone {
    use crate::card::{Subtypes, Types};
    if def.types.has(Types::WEAPON) || def.subtypes.any(Subtypes::OFF_HAND | Subtypes::QUIVER) {
        Zone::Weapon
    } else if def.subtypes.has(Subtypes::HEAD) {
        Zone::Head
    } else if def.subtypes.has(Subtypes::CHEST) {
        Zone::Chest
    } else if def.subtypes.has(Subtypes::ARMS) {
        Zone::Arms
    } else if def.subtypes.has(Subtypes::LEGS) {
        Zone::Legs
    } else {
        Zone::Permanent
    }
}
