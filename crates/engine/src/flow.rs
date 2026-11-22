//! Déroulement de la partie : phases (CR 4), priorité (CR 1.11), résolution
//! des couches (CR 5.3) et combat (CR 7).
//!
//! `advance()` fait progresser la partie jusqu'à ce qu'un joueur doive choisir
//! (une [`Decision`] est alors en attente) ou que la partie soit finie.

use crate::card::{Keywords, Types};
use crate::decision::{Choice, Decision, DecisionKind};
use crate::ids::{ObjectId, PlayerId};
use crate::state::{ChainLink, CombatStep, EndReason, GameState, LayerKind, Outcome, Stage, Until};
use crate::zones::Zone;

impl GameState {
    /// Fait avancer la partie jusqu'à la prochaine décision ou la fin.
    pub(crate) fn advance(&mut self) {
        while self.pending.is_none() && self.outcome.is_none() {
            match self.stage {
                Stage::ChooseFirstPlayer => unreachable!("décision toujours en attente"),
                Stage::StartPhase => self.start_phase(),
                Stage::Priority => self.priority_step(),
                Stage::Paying => self.ask_payment(),
                Stage::Defending => self.ask_defenders(),
                Stage::EndArsenal => self.end_arsenal(),
                Stage::EndPitchOrder(p) => self.end_pitch_order(p),
                Stage::EndRest => self.end_rest(),
                Stage::GameOver => return,
            }
        }
    }

    /// Applique la réponse d'un joueur à une décision.
    pub(crate) fn answer(&mut self, player: PlayerId, kind: DecisionKind, choice: Choice) {
        match (kind, choice) {
            (DecisionKind::FirstPlayer, Choice::FirstPlayer(first)) => self.start_game(first),
            (DecisionKind::Priority, Choice::Pass) => self.pass(player),
            (DecisionKind::Priority, Choice::Play(id)) => self.play_card(player, id),
            (DecisionKind::Priority, Choice::Activate(id)) => self.activate_weapon(player, id),
            (DecisionKind::Pitch, Choice::Pitch(id)) => self.pitch(player, id),
            (DecisionKind::Defend, Choice::Defend(id)) => self.declared_defenders.push(id),
            (DecisionKind::Defend, Choice::Pass) => self.finish_defending(),
            (DecisionKind::Arsenal, Choice::Arsenal(id)) => {
                // CR 4.4.3b : face cachée dans une zone d'arsenal vide.
                self.move_to(id, Zone::Arsenal, true);
                self.stage = Stage::EndPitchOrder(self.turn_player);
            }
            (DecisionKind::Arsenal, Choice::Pass) => {
                self.stage = Stage::EndPitchOrder(self.turn_player)
            }
            (DecisionKind::PitchOrder, Choice::PitchToBottom(id)) => {
                self.move_to(id, Zone::Deck, false)
            }
            (k, c) => unreachable!("réponse {c:?} impossible pour une décision {k:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Actions de l'état de jeu (CR 1.10.2)
    // ------------------------------------------------------------------

    /// Retourne `true` si la partie est terminée.
    pub(crate) fn game_state_actions(&mut self) -> bool {
        // CR 1.10.2a / 4.5.3a / 4.5.4a
        let dead: Vec<bool> = PlayerId::BOTH
            .iter()
            .map(|&p| self.player(p).life <= 0)
            .collect();
        let outcome = match (dead[0], dead[1]) {
            (true, true) => Some(Outcome::Draw),
            (true, false) => Some(Outcome::Winner(PlayerId::P1)),
            (false, true) => Some(Outcome::Winner(PlayerId::P0)),
            (false, false) => None,
        };
        if let Some(o) = outcome {
            self.end_game(o, EndReason::LifeZero);
            return true;
        }
        // CR 1.10.2d : les couches déclenchées sont ajoutées à la pile.
        // TODO(règles) CR 6.6.6b : avec plusieurs déclenchements, le joueur
        // actif choisit qui empile en premier et chacun choisit son ordre.
        // Pour l'instant : ordre de création.
        if !self.pending_triggers.is_empty() {
            let t = std::mem::take(&mut self.pending_triggers);
            self.stack.extend(t);
        }
        false
    }

    pub(crate) fn end_game(&mut self, outcome: Outcome, reason: EndReason) {
        self.outcome = Some(outcome);
        self.end_reason = Some(reason);
        self.stage = Stage::GameOver;
        self.priority = None;
        self.pending = None;
    }

    // ------------------------------------------------------------------
    // Phases du tour (CR 4.2 à 4.4)
    // ------------------------------------------------------------------

    fn start_phase(&mut self) {
        // CR 4.2 : pas de priorité ; déclenchements « start of turn » (étape 2).
        // CR 4.3.2 : le joueur actif a 1 point d'action.
        let tp = self.turn_player;
        self.player_mut(tp).action_points = 1;
        // CR 4.3.3 : le joueur actif reçoit la priorité.
        self.give_priority(tp);
    }

    pub(crate) fn give_priority(&mut self, p: PlayerId) {
        self.stage = Stage::Priority;
        self.priority = Some(p);
    }

    fn priority_step(&mut self) {
        if self.game_state_actions() {
            return;
        }
        let p = self.priority.expect("priorité");
        let options = self.legal_priority_options(p);
        if options.len() == 1 {
            // Seule option : passer. Le moteur passe automatiquement, ce qui
            // ne change rien au jeu (aucune autre action n'était légale).
            self.pass(p);
        } else {
            self.pending = Some(Decision {
                player: p,
                kind: DecisionKind::Priority,
                options,
            });
        }
    }

    /// CR 1.11.4 : le joueur passe la priorité.
    pub(crate) fn pass(&mut self, p: PlayerId) {
        self.passes += 1;
        if (self.passes as usize) < PlayerId::BOTH.len() {
            self.priority = Some(p.opponent());
            return;
        }
        self.passes = 0;
        self.all_passed();
    }

    /// Tous les joueurs ont passé successivement (CR 1.11.4a).
    fn all_passed(&mut self) {
        let tp = self.turn_player;
        if let Some(top) = self.stack.last().copied() {
            if matches!(top.kind, LayerKind::WeaponAttack)
                || (top.kind == LayerKind::Card && self.def(top.object.id).is_attack_card())
            {
                // CR 5.3.1a / 7.1.2a : l'attaque va dans la file d'attente.
                self.stack.pop();
                self.combat
                    .queue
                    .push((top.object, top.target.expect("cible d'attaque")));
                if self.stack.is_empty() {
                    // CR 7.1.3 : fin de l'étape des couches.
                    self.begin_attack_step();
                } else {
                    self.give_priority(tp);
                }
            } else {
                self.resolve_top();
                self.give_priority(tp);
            }
            return;
        }
        if !self.combat.open {
            // CR 4.3.4 : fin de la phase d'action.
            self.priority = None;
            self.begin_end_phase();
            return;
        }
        match self.combat.step.expect("étape de combat") {
            CombatStep::Layer => {
                if self.combat.queue.is_empty() {
                    self.close_combat_chain();
                } else {
                    self.begin_attack_step();
                }
            }
            // CR 7.2.5
            CombatStep::Attack => self.begin_defend_step(),
            // CR 7.3.4
            CombatStep::Defend => {
                self.combat.step = Some(CombatStep::Reaction);
                self.give_priority(tp); // CR 7.4.2
            }
            // CR 7.4.3
            CombatStep::Reaction => self.damage_step(),
            // CR 7.5.4
            CombatStep::Damage => self.resolution_step(),
            // CR 7.6.3b / 7.6.4
            CombatStep::Resolution => {
                if self.combat.queue.is_empty() {
                    self.close_combat_chain();
                } else {
                    self.begin_attack_step();
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Résolution des couches (CR 5.3)
    // ------------------------------------------------------------------

    fn resolve_top(&mut self) {
        let layer = self.stack.pop().expect("couche");
        match layer.kind {
            LayerKind::TriggerOnAttack => {
                let ops = self.db.get(layer.card).on_attack.clone();
                self.run_ops(&ops, layer.controller, layer.object);
            }
            LayerKind::TriggerOnHit => {
                let ops = self.db.get(layer.card).on_hit.clone();
                self.run_ops(&ops, layer.controller, layer.object);
            }
            LayerKind::WeaponAttack => unreachable!("les attaques vont dans la file"),
            LayerKind::Card => {
                let id = layer.object.id;
                let def = self.def(id);
                let is_defense_reaction = def.types.has(Types::DEFENSE_REACTION);
                // CR 5.3.2b : une réaction de défense qui ne peut pas défendre échoue.
                if is_defense_reaction && !self.can_become_defending(id, layer.origin) {
                    self.clear(id);
                    return;
                }
                // CR 5.3.4 : effets de la couche.
                let ops = def.on_resolve.clone();
                self.run_ops(&ops, layer.controller, layer.object);
                // CR 5.3.5 : go again.
                if self.is_current(layer.object) && self.keywords(id).has(Keywords::GO_AGAIN) {
                    self.gain_action_point(layer.controller);
                }
                // CR 5.3.6 / 5.3.7 : quitter la pile.
                if !self.is_current(layer.object) {
                    return;
                }
                if is_defense_reaction {
                    // CR 5.3.6b / 7.4.2d : devient une carte qui défend.
                    self.add_defender(id, layer.origin == Zone::Hand);
                } else {
                    self.clear(id);
                }
            }
        }
    }

    /// CR 1.13.2b : seul le joueur actif, pendant sa phase d'action, gagne des points d'action.
    pub(crate) fn gain_action_point(&mut self, p: PlayerId) {
        if p == self.turn_player {
            self.player_mut(p).action_points += 1;
        }
    }

    // ------------------------------------------------------------------
    // Combat (CR 7)
    // ------------------------------------------------------------------

    /// CR 7.2 : étape d'attaque.
    fn begin_attack_step(&mut self) {
        let (attack, target) = self.combat.queue.remove(0);
        let tp = self.turn_player;
        if !self.is_current(attack) {
            // CR 7.2.2c : l'attaque ne peut pas devenir un maillon.
            if self.combat.queue.is_empty() {
                self.close_combat_chain();
            } else {
                self.begin_attack_step();
            }
            return;
        }
        let id = attack.id;
        let controller = self.obj(id).controller;
        let source = self.obj(id).proxy_of;
        // CR 7.2.2a/b : l'attaque (et sa source) vont sur la chaîne.
        self.place(id, Zone::CombatChain, controller);
        if let Some(w) = source {
            // CR 1.4.3c / 7.2.2b : l'ancienne proxy de cette arme cesse d'exister.
            let old: Vec<ObjectId> = self
                .combat
                .links
                .iter()
                .filter(|l| l.source == Some(w))
                .map(|l| l.attack.id)
                .collect();
            for o in old {
                if self.obj(o).zone != Zone::Gone {
                    self.clear(o);
                }
            }
            self.obj_mut(w).zone = Zone::CombatChain;
        }
        self.combat.links.push(ChainLink {
            attack: self.obj_ref(id),
            source,
            attacker: controller,
            target,
            defenders: Vec::new(),
            defenders_from_hand: 0,
            hit: false,
        });
        self.combat.step = Some(CombatStep::Attack);
        // CR 7.2.3 : événement « attack », déclenchements.
        let card = self.obj(id).card;
        if !self.db.get(card).on_attack.is_empty() && source.is_none() {
            self.queue_trigger(
                LayerKind::TriggerOnAttack,
                self.obj_ref(id),
                controller,
                Some(target),
            );
        }
        // CR 7.2.4
        self.give_priority(tp);
    }

    /// CR 7.3 : étape de défense.
    fn begin_defend_step(&mut self) {
        self.combat.step = Some(CombatStep::Defend);
        self.declared_defenders.clear();
        self.priority = None;
        self.stage = Stage::Defending;
    }

    /// Les cartes que le héros attaqué peut encore déclarer (CR 7.3.2a-b).
    pub fn defend_options(&self) -> Vec<Choice> {
        let link = self.combat.links.last().expect("maillon");
        let p = link.target;
        let attack = link.attack.id;
        let dominate = self.keywords(attack).has(Keywords::DOMINATE);
        let hand_declared = self
            .declared_defenders
            .iter()
            .filter(|&&d| self.obj(d).zone == Zone::Hand)
            .count();
        let mut opts = Vec::new();
        let ps = self.player(p);
        for &c in &ps.hand {
            if self.declared_defenders.contains(&c) {
                continue;
            }
            let def = self.def(c);
            // CR 7.3.2a : pas de réaction de défense ; CR 7.3.2b(A) : propriété défense.
            if def.types.has(Types::DEFENSE_REACTION) || def.defense.is_none() {
                continue;
            }
            // CR 8.3.4 : dominate, une seule carte de la main.
            if dominate && hand_declared as u8 + link.defenders_from_hand >= 1 {
                continue;
            }
            opts.push(Choice::Defend(c));
        }
        // Équipements publics (CR 7.3.2a), y compris l'off-hand en zone d'arme.
        for e in ps.equipment().chain(ps.weapons.iter().copied()) {
            let o = self.obj(e);
            if self.declared_defenders.contains(&e) || o.zone == Zone::CombatChain {
                continue;
            }
            let def = self.def(e);
            if def.types.has(Types::EQUIPMENT) && def.defense.is_some() {
                opts.push(Choice::Defend(e));
            }
        }
        opts.push(Choice::Pass);
        opts
    }

    fn ask_defenders(&mut self) {
        let opts = self.defend_options();
        let target = self.combat.links.last().expect("maillon").target;
        if opts.len() == 1 {
            self.finish_defending();
        } else {
            self.pending = Some(Decision {
                player: target,
                kind: DecisionKind::Defend,
                options: opts,
            });
        }
    }

    /// CR 7.3.2d : les cartes déclarées deviennent des cartes qui défendent.
    fn finish_defending(&mut self) {
        let cards = std::mem::take(&mut self.declared_defenders);
        for c in cards {
            let from_hand = self.obj(c).zone == Zone::Hand;
            self.add_defender(c, from_hand);
        }
        // CR 7.3.3
        let tp = self.turn_player;
        self.give_priority(tp);
    }

    /// Une carte peut-elle devenir une carte qui défend maintenant (CR 7.0.5b, 8.3.4) ?
    pub(crate) fn can_become_defending(&self, id: ObjectId, from: Zone) -> bool {
        let Some(link) = self.combat.active_link() else {
            return false;
        };
        if link.defenders.contains(&id) {
            return false;
        }
        let dominate = self.keywords(link.attack.id).has(Keywords::DOMINATE);
        !(dominate && from == Zone::Hand && link.defenders_from_hand >= 1)
    }

    pub(crate) fn add_defender(&mut self, id: ObjectId, from_hand: bool) {
        let controller = self.obj(id).owner;
        self.place(id, Zone::CombatChain, controller);
        let link = self.combat.links.last_mut().expect("maillon");
        link.defenders.push(id);
        if from_hand {
            link.defenders_from_hand += 1;
        }
    }

    /// CR 7.5 : étape des dégâts.
    fn damage_step(&mut self) {
        self.combat.step = Some(CombatStep::Damage);
        let link = self.combat.links.last().expect("maillon").clone();
        if self.is_current(link.attack) {
            let power = self.power(link.attack.id);
            // CR 7.5.2 : somme des défenses des cartes publiques qui défendent.
            let defense: i16 = link
                .defenders
                .iter()
                .filter(|&&d| !self.obj(d).face_down)
                .map(|&d| self.defense(d))
                .sum();
            if power > defense {
                let dmg = (power - defense) as i32;
                // Événement « hit » (CR 7.5.5) : dégâts physiques au héros.
                self.deal_damage(link.target, dmg);
                self.combat.links.last_mut().unwrap().hit = true;
                let card = self.obj(link.attack.id).card;
                if !self.db.get(card).on_hit.is_empty() && link.source.is_none() {
                    self.queue_trigger(
                        LayerKind::TriggerOnHit,
                        link.attack,
                        link.attacker,
                        Some(link.target),
                    );
                }
            }
        }
        // CR 7.5.3
        let tp = self.turn_player;
        self.give_priority(tp);
    }

    /// CR 7.6 : étape de résolution.
    fn resolution_step(&mut self) {
        self.combat.step = Some(CombatStep::Resolution);
        let link = self.combat.links.last().expect("maillon").clone();
        // CR 7.6.2 / 8.3.5b : go again.
        // TODO(règles) CR 7.6.2a : informations de dernier état si l'attaque a quitté la chaîne.
        if self.is_current(link.attack) && self.keywords(link.attack.id).has(Keywords::GO_AGAIN) {
            self.gain_action_point(link.attacker);
        }
        let tp = self.turn_player;
        self.give_priority(tp);
    }

    /// CR 7.7 : fermeture de la chaîne de combat.
    pub(crate) fn close_combat_chain(&mut self) {
        // CR 7.7.3 : déclenchements « combat chain closes » : blade break,
        // battleworn, temper (CR 8.3.2, 8.3.3, 8.3.10).
        // Simplification : appliqués directement (aucun joueur n'a la priorité
        // pendant la fermeture, CR 7.7.4) ; l'ordre entre plusieurs
        // déclenchements sera géré avec les autres déclenchements à l'étape 2.
        let defenders: Vec<ObjectId> = self
            .combat
            .links
            .iter()
            .flat_map(|l| l.defenders.iter().copied())
            .collect();
        // Attaques, couches d'attaque et réactions quittent la pile (CR 7.7.3).
        let stack = std::mem::take(&mut self.stack);
        for l in stack {
            if self.is_current(l.object) && self.obj(l.object.id).zone == Zone::Stack {
                self.clear(l.object.id);
            }
        }
        for (a, _) in std::mem::take(&mut self.combat.queue) {
            if self.is_current(a) && self.obj(a.id).zone == Zone::Stack {
                self.clear(a.id);
            }
        }
        // CR 7.7.5 : les permanents retournent dans leurs zones.
        let on_chain: Vec<ObjectId> = (0..self.objects.len() as u32)
            .map(ObjectId)
            .filter(|&id| self.obj(id).zone == Zone::CombatChain)
            .collect();
        for id in on_chain {
            let def = self.def(id);
            let is_permanent = self.obj(id).proxy_of.is_none()
                && (def.types.has(Types::EQUIPMENT) || def.types.has(Types::WEAPON));
            if is_permanent {
                self.return_to_equipment_zone(id);
            } else {
                // CR 7.7.6 : le reste est « clear ».
                self.clear(id);
            }
        }
        for id in defenders {
            if !self.def(id).types.has(Types::EQUIPMENT) || self.obj(id).zone == Zone::Gone {
                continue;
            }
            let kw = self.keywords(id);
            if kw.has(Keywords::BLADE_BREAK) {
                self.destroy(id);
            } else if kw.has(Keywords::TEMPER) {
                self.obj_mut(id).defense_counters -= 1;
                if self.defense(id) == 0 {
                    self.destroy(id);
                }
            } else if kw.has(Keywords::BATTLEWORN) {
                self.obj_mut(id).defense_counters -= 1;
            }
        }
        // CR 7.7.7 : la chaîne se ferme, fin des effets « this combat chain ».
        self.combat.links.clear();
        self.combat.open = false;
        self.combat.step = None;
        self.modifiers
            .retain(|m| m.until != Until::CombatChainCloses);
        let tp = self.turn_player;
        self.give_priority(tp);
    }

    /// CR 8.5.4 : détruire = mettre au cimetière du propriétaire.
    pub(crate) fn destroy(&mut self, id: ObjectId) {
        self.clear(id);
    }

    // ------------------------------------------------------------------
    // Phase de fin (CR 4.4)
    // ------------------------------------------------------------------

    fn begin_end_phase(&mut self) {
        // CR 4.4.2 : déclenchements « beginning of the end phase » (étape 2).
        self.stage = Stage::EndArsenal;
    }

    fn end_arsenal(&mut self) {
        // CR 4.4.3b
        let tp = self.turn_player;
        let ps = self.player(tp);
        if ps.arsenal.is_some() || ps.hand.is_empty() {
            self.stage = Stage::EndPitchOrder(tp);
            return;
        }
        let mut options: Vec<Choice> = ps.hand.iter().map(|&c| Choice::Arsenal(c)).collect();
        options.push(Choice::Pass);
        self.pending = Some(Decision {
            player: tp,
            kind: DecisionKind::Arsenal,
            options,
        });
    }

    fn end_pitch_order(&mut self, p: PlayerId) {
        // CR 4.4.3c : chaque joueur met son pitch sous son deck, dans l'ordre de son choix.
        // Ordre des joueurs : en commençant par le joueur actif (CR 1.9.2c).
        let pitch = &self.player(p).pitch;
        let distinct = {
            let mut cards: Vec<_> = pitch.iter().map(|&c| self.obj(c).card).collect();
            cards.sort();
            cards.dedup();
            cards.len()
        };
        if pitch.is_empty() || distinct <= 1 {
            // Ordre sans importance (aucune carte ou cartes identiques).
            for c in pitch.clone() {
                self.move_to(c, Zone::Deck, false);
            }
            if p == self.turn_player {
                self.stage = Stage::EndPitchOrder(p.opponent());
            } else {
                self.stage = Stage::EndRest;
            }
            return;
        }
        let options = pitch.iter().map(|&c| Choice::PitchToBottom(c)).collect();
        self.pending = Some(Decision {
            player: p,
            kind: DecisionKind::PitchOrder,
            options,
        });
    }

    fn end_rest(&mut self) {
        let tp = self.turn_player;
        // CR 4.4.3d : le joueur actif dégage ses permanents.
        for id in 0..self.objects.len() {
            let o = &mut self.objects[id];
            if o.controller == tp && o.zone.in_arena() {
                o.tapped = false;
            }
        }
        // CR 4.4.3e : tous les joueurs perdent leurs points d'action et de ressource.
        for p in PlayerId::BOTH {
            let ps = self.player_mut(p);
            ps.action_points = 0;
            ps.resource_points = 0;
        }
        // CR 4.4.3f : pioche jusqu'à l'intellect ; au premier tour, tous les joueurs.
        self.draw_up_to_intellect(tp);
        if self.turn == 1 {
            self.draw_up_to_intellect(tp.opponent());
        }
        // CR 4.4.4 : fin du tour.
        self.modifiers.retain(|m| m.until != Until::EndOfTurn);
        self.next_attack_effects.clear();
        for o in &mut self.objects {
            o.activations_this_turn = 0;
        }
        if self.turn >= self.max_turns {
            self.end_game(Outcome::Draw, EndReason::TurnLimit);
            return;
        }
        self.turn += 1;
        self.turn_player = tp.opponent();
        self.stage = Stage::StartPhase;
    }
}
