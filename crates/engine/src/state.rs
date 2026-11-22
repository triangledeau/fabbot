//! L'état complet d'une partie.

use crate::card::{CardDb, CardDef, Delta, Keywords, Subtypes, Types};
use crate::decision::Decision;
use crate::events::Event;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::rng::Rng;
use crate::script::{Ctx, ScriptRef};
use crate::zones::Zone;
use std::sync::Arc;

/// Un objet de la partie : une carte, un token, ou une attaque-proxy d'arme.
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
    /// Somme des compteurs +1/-1 de défense et de puissance (CR 1.15.2a).
    pub defense_counters: i8,
    pub power_counters: i8,
    /// Autres compteurs (ex. compteurs « verse »).
    pub counters: u8,
    /// Pour une attaque-proxy : l'arme qu'elle représente (CR 1.4.3).
    pub proxy_of: Option<ObjectId>,
    /// Activations de chaque capacité ce tour (limite « Once per Turn »).
    pub activations: [u8; 4],
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
    /// Le héros est marqué (CR 8.5.50).
    pub marked: bool,
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
    /// Une capacité activée (index dans `CardDef::activated`). Si c'est une
    /// capacité « Attack », `object` est l'attaque-proxy (CR 1.4.3).
    Activated(u8),
    /// Une capacité déclenchée (index dans `CardDef::triggers`).
    Trigger(u8),
}

/// Une couche sur la pile.
#[derive(Clone, Debug)]
pub struct Layer {
    pub kind: LayerKind,
    /// La carte (ou la proxy) sur la pile ; pour un déclenchement, sa source.
    pub object: ObjRef,
    /// Définition utilisée pour les effets (reste valable si la source disparaît, CR 1.7.1a).
    pub card: CardId,
    pub controller: PlayerId,
    /// Cible de l'attaque (CR 1.4.5).
    pub target: Option<PlayerId>,
    /// Zone d'où la carte a été jouée (main, arsenal...).
    pub origin: Zone,
    /// Cibles, modes, X... déclarés.
    pub ctx: Box<Ctx>,
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
    /// Définition de l'attaque (informations de dernier état, CR 7.0.3c).
    pub card: CardId,
    pub attacker: PlayerId,
    pub target: PlayerId,
    pub defenders: Vec<ObjectId>,
    /// Nombre de cartes venues de la main qui défendent (pour dominate).
    pub defenders_from_hand: u8,
    pub hit: bool,
    /// Index du journal du tour au début de ce maillon (« this chain link »).
    pub log_start: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Combat {
    pub open: bool,
    pub step: Option<CombatStep>,
    pub links: Vec<ChainLink>,
    /// File d'attente des attaques (CR 3.15.7) : (attaque, cible).
    pub queue: Vec<(ObjRef, PlayerId)>,
    /// Index du journal du tour à l'ouverture (« this combat chain »).
    pub log_start: usize,
}

impl Combat {
    /// Le maillon actif (CR 7.0.3b) : de l'étape d'attaque à l'étape des dégâts.
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

/// « X gets +N » : modification d'un objet par un effet continu.
#[derive(Clone, Copy, Debug)]
pub struct Modifier {
    pub target: ObjRef,
    pub delta: Delta,
    pub until: Until,
}

/// « Votre prochaine attaque [qui …] ce tour gets … » (CR 1.8.10), en attente.
#[derive(Clone, Copy, Debug)]
pub struct NextAttackEffect {
    pub player: PlayerId,
    /// Filtre sur l'attaque (ex. « attaque d'arme »), `None` = toute attaque.
    pub filter: Option<fn(&GameState, ObjectId) -> bool>,
    pub delta: Delta,
    /// Seulement pour une carte d'attaque jouée (« next attack action card you play »).
    pub cards_only: bool,
}

/// « Prevent the next N damage that would be dealt to … » (CR 6.4).
#[derive(Clone, Copy, Debug)]
pub struct Prevention {
    pub player: PlayerId,
    pub remaining: i32,
    pub arcane_only: bool,
    /// Seulement les dégâts de cette source.
    pub source: Option<ObjRef>,
    pub until: Until,
}

/// Une tâche en attente (pile de tâches) : le moteur exécute toujours la tâche
/// du dessus avant de continuer le déroulement normal de la partie.
#[derive(Clone, Debug)]
pub enum Task {
    /// Exécuter une capacité à partir de l'instruction `pc`.
    /// `write_back` : à la fin, recopier le contexte dans la couche du dessus
    /// de la pile (coûts d'effet dont le résultat sert à la résolution).
    Script {
        script: ScriptRef,
        pc: u16,
        ctx: Box<Ctx>,
        write_back: bool,
    },
    /// Étapes pour jouer / activer la couche du dessus de la pile (CR 5.1).
    PlayOptionalCost,
    PlayX,
    PlayMode,
    PlayTarget(u8),
    PlayPay,
    PlayEffectCost,
    PlayFinish,
    /// Payer des ressources puis des points d'action (CR 1.14.2).
    Pay {
        player: PlayerId,
        resources: u8,
        action_points: u8,
    },
    /// Suite de la résolution d'une couche après ses effets (CR 5.3.5-5.3.7).
    FinishResolve(Box<Layer>),
    /// Déclarer les cibles d'une couche déclenchée avant de l'empiler (CR 6.6.6a).
    TriggerTargets {
        layer: Box<Layer>,
        next: u8,
    },
}

/// Où en est le déroulement de la partie (hors tâches et priorité).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Choix du premier joueur (CR 4.1.3) : décision en attente.
    ChooseFirstPlayer,
    /// Début de tour (CR 4.2).
    StartPhase,
    /// Résoudre la pile sans priorité, puis passer à l'étape suivante (CR 4.2.2, 4.4.2).
    ResolveThen(AfterResolve),
    /// Un joueur a ou va recevoir la priorité (CR 1.11).
    Priority,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterResolve {
    ActionPhase,
    EndArsenal,
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
    /// Plus aucun joueur ne peut faire avancer la partie (CR 4.5.4d), détecté
    /// dans le cas simple : decks, mains et arsenaux vides des deux côtés.
    Stalemate,
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
    /// Zone permanente partagée (CR 3.13).
    pub permanents: Vec<ObjectId>,
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
    pub preventions: Vec<Prevention>,
    /// Couches déclenchées créées, pas encore ajoutées à la pile (CR 6.6.6).
    pub pending_triggers: Vec<Layer>,
    /// Joueur qui empile ses déclenchements en premier (CR 6.6.6b).
    pub trigger_first: Option<PlayerId>,
    pub tasks: Vec<Task>,
    /// Cartes déclarées pendant la déclaration de défense en cours.
    pub declared_defenders: Vec<ObjectId>,
    /// Journal des événements du tour.
    pub turn_log: Vec<Event>,
    pub pending: Option<Decision>,
    pub outcome: Option<Outcome>,
    pub end_reason: Option<EndReason>,
    pub max_turns: u32,
    /// Objets dont la carte a des capacités déclenchées (optimisation de `emit`).
    pub trigger_objects: Vec<ObjectId>,
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

    /// Le maillon actif, s'il existe.
    pub fn active_link(&self) -> Option<&ChainLink> {
        self.combat.active_link()
    }

    /// L'attaque du maillon actif (CR 7.0.4).
    pub fn active_attack(&self) -> Option<ObjectId> {
        self.combat
            .active_link()
            .map(|l| l.attack.id)
            .filter(|&a| self.obj(a).zone == Zone::CombatChain)
    }

    // ------------------------------------------------------------------
    // Déplacements d'objets (CR 3.0.7)
    // ------------------------------------------------------------------

    /// Retire l'objet de son conteneur actuel (sans changer `zone`).
    pub(crate) fn detach(&mut self, id: ObjectId) {
        let o = self.obj(id);
        let owner = o.owner;
        let zone = o.zone;
        if zone == Zone::Permanent {
            self.permanents.retain(|x| *x != id);
            return;
        }
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
            Zone::Arsenal if p.arsenal == Some(id) => p.arsenal = None,
            // Les équipements et armes restent « équipés » quand ils sont sur
            // la chaîne de combat (CR 7.7.5) ; leur emplacement n'est libéré
            // que lorsqu'ils quittent l'arène (voir `unequip`).
            _ => {}
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

    /// Déplace un objet vers une zone d'un joueur (deck, main, cimetière…) ou
    /// la zone permanente. Applique CR 3.0.9 (nouvel objet hors de l'arène).
    /// Renvoie la zone d'origine.
    pub(crate) fn move_to(&mut self, id: ObjectId, zone: Zone, on_top: bool) -> Zone {
        let from = self.obj(id).zone;
        let was_in_arena = from.in_arena();
        // Un token qui quitte l'arène cesse d'exister (CR 8.1.8a).
        if self.def(id).types.has(Types::TOKEN) && was_in_arena && !zone.in_arena() {
            self.detach(id);
            self.unequip(id);
            let o = self.obj_mut(id);
            o.zone = Zone::Gone;
            o.gen += 1;
            self.emit(Event::LeaveArena {
                card: id,
                owner: self.obj(id).owner,
            });
            return from;
        }
        self.detach(id);
        if !zone.in_arena() {
            self.unequip(id);
        }
        let owner = self.obj(id).owner;
        {
            if zone == Zone::Permanent {
                self.permanents.push(id);
            }
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
            o.counters = 0;
            o.tapped = false;
            o.activations = [0; 4];
        }
        o.zone = zone;
        o.face_down = matches!(zone, Zone::Deck | Zone::Hand | Zone::Arsenal);
        o.controller = owner;
        if was_in_arena && !zone.in_arena() {
            self.emit(Event::LeaveArena { card: id, owner });
        }
        if !was_in_arena && zone.in_arena() {
            self.emit(Event::EnterArena { card: id });
        }
        from
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
        let zone = equipment_zone(self.def(id));
        let o = self.obj_mut(id);
        o.zone = zone;
        o.controller = o.owner;
        if zone == Zone::Permanent {
            self.permanents.push(id);
        }
    }

    /// « Clear » (CR 3.0.12) : au cimetière du propriétaire ; une proxy ou un
    /// token cesse d'exister.
    pub(crate) fn clear(&mut self, id: ObjectId) {
        if self.obj(id).proxy_of.is_some() || self.def(id).types.has(Types::TOKEN) {
            let was_in_arena = self.obj(id).zone.in_arena();
            self.detach(id);
            self.unequip(id);
            let owner = self.obj(id).owner;
            let o = self.obj_mut(id);
            o.zone = Zone::Gone;
            o.gen += 1;
            if was_in_arena && self.obj(id).proxy_of.is_none() {
                self.emit(Event::LeaveArena { card: id, owner });
            }
        } else {
            self.put_in_graveyard(id);
        }
    }

    /// Met un objet au cimetière de son propriétaire (événement « ToGraveyard »).
    pub fn put_in_graveyard(&mut self, id: ObjectId) {
        let owner = self.obj(id).owner;
        let from = self.move_to(id, Zone::Graveyard, true);
        if self.obj(id).zone == Zone::Graveyard {
            self.emit(Event::ToGraveyard {
                card: id,
                owner,
                from,
            });
        }
    }

    // ------------------------------------------------------------------
    // Valeurs calculées (CR 2, CR 6.3)
    // ------------------------------------------------------------------

    /// Somme des effets continus qui s'appliquent à un objet.
    pub fn continuous_delta(&self, id: ObjectId) -> Delta {
        let o = self.obj(id);
        let r = o.obj_ref(id);
        let mut d = Delta::default();
        let mut add = |x: Delta| {
            d.power += x.power;
            d.defense += x.defense;
            d.grants = d.grants | x.grants;
        };
        for m in &self.modifiers {
            if m.target == r {
                add(m.delta);
            }
        }
        // Effets statiques de la carte elle-même (CR 5.4).
        if o.proxy_of.is_none() {
            if let Some(f) = self.db.get(o.card).self_mod {
                add(f(self, id));
            }
        }
        // Effets statiques des héros, équipements et permanents (CR 1.7.4).
        for src in self.static_sources() {
            if let Some(f) = self.def(src).aura {
                add(f(self, src, id));
            }
        }
        d
    }

    /// Objets dont les effets statiques « aura » sont actifs.
    fn static_sources(&self) -> impl Iterator<Item = ObjectId> + '_ {
        let heroes = self.players.iter().map(|p| p.hero);
        let equip = self
            .players
            .iter()
            .flat_map(|p| p.equipment().chain(p.weapons.iter().copied()));
        heroes
            .chain(equip)
            .chain(self.permanents.iter().copied())
            .filter(|&s| self.db.get(self.obj(s).card).aura.is_some())
    }

    /// Puissance actuelle (CR 2.9.3, jamais négative CR 2.0.3c).
    /// Une proxy hérite de la puissance modifiée de son arme (CR 1.4.3a, 1.4.3d).
    pub fn power(&self, id: ObjectId) -> i16 {
        let o = self.obj(id);
        let base = match o.proxy_of {
            Some(w) => self.power(w),
            None => self.db.get(o.card).power.unwrap_or(0) as i16 + o.power_counters as i16,
        };
        (base + self.continuous_delta(id).power).max(0)
    }

    /// Défense actuelle (CR 2.3.3, jamais négative CR 2.0.3c).
    pub fn defense(&self, id: ObjectId) -> i16 {
        let o = self.obj(id);
        let base = self.db.get(o.card).defense.unwrap_or(0) as i16 + o.defense_counters as i16;
        (base + self.continuous_delta(id).defense).max(0)
    }

    /// Mots-clés actuels (de base + accordés par des effets).
    pub fn keywords(&self, id: ObjectId) -> Keywords {
        let o = self.obj(id);
        let base = match o.proxy_of {
            // Une proxy n'hérite pas des capacités de résolution de sa source
            // (CR 1.4.3a) : son « go again » vient de la capacité d'attaque,
            // fixé à sa création (modificateur). Les mots-clés statiques de
            // l'arme (ex. dominate) sont hérités.
            Some(w) => self.keywords(w).without(Keywords::GO_AGAIN),
            None => self.db.get(o.card).keywords,
        };
        base | self.continuous_delta(id).grants
    }

    /// Le dessus du deck d'un joueur.
    pub fn top_of_deck(&self, p: PlayerId) -> Option<ObjectId> {
        self.player(p).deck.last().copied()
    }
}

/// Zone d'équipement correspondant au sous-type d'une carte (CR 3.2-3.16).
pub(crate) fn equipment_zone(def: &CardDef) -> Zone {
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
