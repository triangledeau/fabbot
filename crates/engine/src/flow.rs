//! Déroulement de la partie : tâches, phases (CR 4), priorité (CR 1.11),
//! déclenchements (CR 6.6), résolution des couches (CR 5.3) et combat (CR 7).
//!
//! `advance()` fait progresser la partie jusqu'à ce qu'un joueur doive choisir
//! (une [`Decision`] est alors en attente) ou que la partie soit finie. Les
//! tâches en attente passent toujours avant le déroulement normal.

use crate::card::{Keywords, Speed, Subtypes, Supertypes, Types};
use crate::decision::{Choice, Decision, DecisionKind};
use crate::events::Event;
use crate::ids::{ObjectId, PlayerId};
use crate::script::{Ctx, Op, ScriptRef, Target};
use crate::state::{
    AfterResolve, ChainLink, CombatStep, EndReason, GameState, Layer, LayerKind, Outcome,
    RuleTrigger, Stage, Task, Until,
};
use crate::zones::Zone;

impl GameState {
    /// Fait avancer la partie jusqu'à la prochaine décision ou la fin.
    pub(crate) fn advance(&mut self) {
        while self.pending.is_none() && self.outcome.is_none() {
            if let Some(task) = self.tasks.pop() {
                self.run_task(task);
                continue;
            }
            match self.stage {
                Stage::ChooseFirstPlayer => unreachable!("décision toujours en attente"),
                Stage::StartPhase => self.start_phase(),
                Stage::ResolveThen(next) => self.resolve_without_priority(next),
                Stage::Priority => self.priority_step(),
                Stage::Defending => self.ask_defenders(),
                Stage::EndArsenal => self.end_arsenal(),
                Stage::EndPitchOrder(p) => self.end_pitch_order(p),
                Stage::EndRest => self.end_rest(),
                Stage::Closing => self.close_step(),
                Stage::GameOver => return,
            }
        }
    }

    fn run_task(&mut self, task: Task) {
        match task {
            Task::Script { .. } => self.run_script(task),
            Task::Pay {
                player,
                resources,
                action_points,
            } => self.run_pay(player, resources, action_points),
            Task::FinishResolve(layer) => self.finish_resolve(*layer),
            Task::TriggerTargets { layer, next } => self.trigger_targets(layer, next),
            Task::Damage(ev) => self.run_damage(ev),
            Task::Replace { pending, used, .. } => self.run_replace(pending, used),
            other => self.run_play_task(other),
        }
    }

    /// Applique la réponse d'un joueur à une décision.
    pub(crate) fn answer(&mut self, player: PlayerId, kind: DecisionKind, choice: Choice) {
        match (kind, choice) {
            (DecisionKind::FirstPlayer, Choice::Player(first)) => self.start_game(first),
            (DecisionKind::Priority, Choice::Pass) => self.pass(player),
            (DecisionKind::Priority, Choice::Play(id)) => self.play_card(player, id),
            (DecisionKind::Priority, Choice::Activate(id, i)) => self.activate(player, id, i),
            (DecisionKind::OptionalCost, c) => {
                self.stack.last_mut().unwrap().ctx.optional_paid = c == Choice::Yes
            }
            (DecisionKind::ChooseX, Choice::Number(x)) => self.stack.last_mut().unwrap().ctx.x = x,
            (DecisionKind::Mode, Choice::Mode(m)) => {
                if let Some(&Task::PlaySide(c)) = self.tasks.last() {
                    self.tasks.pop();
                    self.play_card_side(player, c, m);
                } else {
                    self.stack.last_mut().unwrap().ctx.mode = m
                }
            }
            (DecisionKind::Target, c) => {
                let t = match c {
                    Choice::Target(t) => Some(t),
                    _ => None,
                };
                if let Some(Task::TriggerTargets { layer, next }) = self.tasks.last_mut() {
                    layer.ctx.targets.push(t);
                    *next += 1;
                } else {
                    self.stack.last_mut().unwrap().ctx.targets.push(t);
                }
            }
            (DecisionKind::Pitch, Choice::Pitch(id)) => self.pitch(player, id),
            (DecisionKind::Effect, c) => match self.tasks.last_mut() {
                Some(Task::Script { pc, ctx, .. }) => {
                    ctx.answer = Some(c);
                    *pc += 1;
                }
                _ => unreachable!("une question d'effet vient toujours d'un script"),
            },
            (DecisionKind::TriggerFirst, Choice::Player(p)) => self.trigger_first = Some(p),
            (DecisionKind::TriggerOrder, Choice::Trigger(i)) => {
                let layer = self.pending_triggers.remove(i as usize);
                // Les déclenchements d'un même mot-clé sont interchangeables
                // (une seule option, voir `flush_triggers`) : ils sont tous
                // empilés ensemble.
                if let LayerKind::Rule(_) = layer.kind {
                    let (kind, controller) = (layer.kind, layer.controller);
                    let mut k = 0;
                    while k < self.pending_triggers.len() {
                        let l = &self.pending_triggers[k];
                        if l.kind == kind && l.controller == controller {
                            let same = self.pending_triggers.remove(k);
                            self.tasks.push(Task::TriggerTargets {
                                layer: Box::new(same),
                                next: 0,
                            });
                        } else {
                            k += 1;
                        }
                    }
                }
                self.tasks.push(Task::TriggerTargets {
                    layer: Box::new(layer),
                    next: 0,
                });
            }
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
                self.move_to(id, Zone::Deck, false);
            }
            (DecisionKind::AttackTarget, Choice::Target(t)) => {
                self.stack.last_mut().unwrap().target = Some(t)
            }
            (DecisionKind::Prevention, c) => self.answer_prevention(c),
            (DecisionKind::Replacement, c) => self.answer_replacement(c == Choice::Yes),
            (k, c) => unreachable!("réponse {c:?} impossible pour une décision {k:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Scripts des capacités
    // ------------------------------------------------------------------

    /// Exécute un script jusqu'à sa fin ou jusqu'à une question.
    fn run_script(&mut self, task: Task) {
        let Task::Script {
            script,
            mut pc,
            mut ctx,
            write_back,
        } = task
        else {
            unreachable!()
        };
        loop {
            let ops = self.script_ops(script);
            let Some(&op) = ops.get(pc as usize) else {
                if write_back {
                    if let Some(l) = self.stack.last_mut() {
                        l.ctx = ctx;
                    }
                }
                return;
            };
            match op {
                Op::Do(f) => {
                    let before = self.tasks.len();
                    f(self, &mut ctx);
                    pc += 1;
                    // L'effet a besoin d'un choix (nouvelle tâche) : on reprendra après.
                    if self.tasks.len() > before || self.pending.is_some() {
                        self.tasks.insert(
                            before,
                            Task::Script {
                                script,
                                pc,
                                ctx,
                                write_back,
                            },
                        );
                        return;
                    }
                }
                Op::Ask(f) => match f(self, &ctx) {
                    Some(ask) if !ask.options.is_empty() => {
                        self.tasks.push(Task::Script {
                            script,
                            pc,
                            ctx,
                            write_back,
                        });
                        self.pending = Some(Decision {
                            player: ask.player,
                            kind: DecisionKind::Effect,
                            options: ask.options,
                        });
                        return;
                    }
                    _ => {
                        ctx.answer = None;
                        pc += 1;
                    }
                },
                Op::If(cond, n) => pc += if cond(self, &ctx) { 1 } else { 1 + n as u16 },
                Op::Back(cond, n) => {
                    if cond(self, &ctx) {
                        pc -= n as u16;
                    } else {
                        pc += 1;
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Actions de l'état de jeu (CR 1.10.2) et déclenchements
    // ------------------------------------------------------------------

    /// Actions de l'état de jeu (CR 1.10.2a-b). Retourne `true` si la partie est terminée.
    fn check_heroes(&mut self) -> bool {
        if self.check_game_over() {
            return true;
        }
        if self.check_attack_gone() {
            return true;
        }
        // CR 1.10.2b / 2.5.3f : un allié à 0 PV est « clear ». Vérifié
        // seulement quand un allié a perdu des PV (chemin rapide).
        if !self.check_allies {
            return false;
        }
        self.check_allies = false;
        let dead: Vec<ObjectId> = self
            .permanents
            .iter()
            .copied()
            .chain(self.attacking_allies())
            .filter(|&o| self.def(o).life.is_some() && self.life_total(o) == Some(0))
            .collect();
        for o in dead {
            self.clear(o);
        }
        false
    }

    /// CR 7.7.2c : si l'attaque active cesse d'exister avant le calcul des
    /// dégâts (ex. détruite par phantasm, ou son allié détruit, CR 1.4.3c) et
    /// qu'il n'y a plus d'attaque dans la file, l'étape de fermeture commence
    /// (action de l'état de jeu). Renvoie `true` si la chaîne se ferme.
    fn check_attack_gone(&mut self) -> bool {
        if !self.combat.open
            || self.stage == Stage::Closing
            || !self.combat.queue.is_empty()
            || !matches!(
                self.combat.step,
                Some(CombatStep::Attack | CombatStep::Defend | CombatStep::Reaction)
            )
        {
            return false;
        }
        let Some(link) = self.combat.links.last() else {
            return false;
        };
        let gone = !self.is_current(link.attack)
            || self.obj(link.attack.id).zone != Zone::CombatChain
            || link
                .source
                .is_some_and(|s| self.obj(s).zone != Zone::CombatChain);
        if gone {
            self.close_combat_chain();
        }
        gone
    }

    fn check_game_over(&mut self) -> bool {
        // CR 1.10.2a / 4.5.3a / 4.5.4a
        let dead = [
            self.player(PlayerId::P0).life <= 0,
            self.player(PlayerId::P1).life <= 0,
        ];
        let outcome = match dead {
            [true, true] => Some(Outcome::Draw),
            [true, false] => Some(Outcome::Winner(PlayerId::P1)),
            [false, true] => Some(Outcome::Winner(PlayerId::P0)),
            [false, false] => None,
        };
        if let Some(o) = outcome {
            self.end_game(o, EndReason::LifeZero);
            return true;
        }
        false
    }

    /// CR 1.10.2d / 6.6.6 : ajoute les couches déclenchées à la pile.
    /// Renvoie `true` si quelque chose a été fait (décision ou tâche).
    fn flush_triggers(&mut self) -> bool {
        if self.pending_triggers.is_empty() {
            self.trigger_first = None;
            return false;
        }
        let tp = self.turn_player;
        let has = |g: &GameState, p: PlayerId| g.pending_triggers.iter().any(|l| l.controller == p);
        // CR 6.6.6b : le joueur actif choisit quel joueur empile en premier.
        if self.trigger_first.is_none() {
            if has(self, tp) && has(self, tp.opponent()) {
                self.pending = Some(Decision {
                    player: tp,
                    kind: DecisionKind::TriggerFirst,
                    options: vec![Choice::Player(tp), Choice::Player(tp.opponent())],
                });
                return true;
            }
            self.trigger_first = Some(if has(self, tp) { tp } else { tp.opponent() });
        }
        let mut p = self.trigger_first.unwrap();
        if !has(self, p) {
            p = p.opponent();
        }
        let mine: Vec<usize> = (0..self.pending_triggers.len())
            .filter(|&i| self.pending_triggers[i].controller == p)
            .collect();
        let all_same = mine.windows(2).all(|w| {
            let (a, b) = (&self.pending_triggers[w[0]], &self.pending_triggers[w[1]]);
            a.card == b.card && a.kind == b.kind && a.ctx.event == b.ctx.event
        });
        // Les déclenchements des mots-clés (blade break, temper…) n'ont pas
        // d'effet dont l'ordre importe : pas de question.
        let all_rules = mine
            .iter()
            .all(|&i| matches!(self.pending_triggers[i].kind, LayerKind::Rule(_)));
        if mine.len() >= 2 && !all_same && !all_rules {
            // Le joueur choisit l'ordre dans lequel il empile ses déclenchements.
            // Les déclenchements d'un même mot-clé (ex. blood debt de
            // plusieurs cartes) sont interchangeables : une seule option.
            let mut options: Vec<Choice> = Vec::new();
            for (n, &i) in mine.iter().enumerate() {
                let kind = self.pending_triggers[i].kind;
                let dup = matches!(kind, LayerKind::Rule(_))
                    && mine[..n]
                        .iter()
                        .any(|&j| self.pending_triggers[j].kind == kind);
                if !dup {
                    options.push(Choice::Trigger(i as u8));
                }
            }
            self.pending = Some(Decision {
                player: p,
                kind: DecisionKind::TriggerOrder,
                options,
            });
            return true;
        }
        let layer = self.pending_triggers.remove(mine[0]);
        self.tasks.push(Task::TriggerTargets {
            layer: Box::new(layer),
            next: 0,
        });
        true
    }

    /// CR 6.6.6a : déclarer les cibles d'une couche déclenchée, puis l'empiler.
    fn trigger_targets(&mut self, layer: Box<Layer>, next: u8) {
        let specs = match layer.kind {
            LayerKind::Trigger(i) => self.db.get(layer.card).triggers[i as usize].targets.clone(),
            LayerKind::Granted(i) => self.db.get(layer.card).granted[i as usize].targets.clone(),
            // Déclenchement d'un mot-clé : pas de cible.
            _ => Vec::new(),
        };
        let Some(spec) = specs.get(next as usize) else {
            self.stack.push(*layer);
            return;
        };
        let mut options: Vec<Choice> = self
            .target_candidates(spec, &layer.ctx)
            .into_iter()
            .map(Choice::Target)
            .collect();
        if options.is_empty() && !spec.optional {
            // Pas de cible légale : la couche cesse d'exister.
            return;
        }
        if spec.optional {
            options.push(Choice::Pass);
        }
        let player = layer.controller;
        self.tasks.push(Task::TriggerTargets { layer, next });
        if options.len() == 1 {
            let c = options[0];
            self.answer(player, DecisionKind::Target, c);
        } else {
            self.pending = Some(Decision {
                player,
                kind: DecisionKind::Target,
                options,
            });
        }
    }

    /// Crée la couche déclenchée d'un mot-clé (CR 8.3), contrôlée par le
    /// contrôleur de l'objet.
    pub(crate) fn push_rule_trigger(&mut self, kind: RuleTrigger, id: ObjectId) {
        let o = self.obj(id);
        let (controller, card) = (o.controller, o.card);
        let object = self.obj_ref(id);
        self.pending_triggers.push(Layer {
            kind: LayerKind::Rule(kind),
            object,
            card,
            controller,
            target: None,
            origin: Zone::Stack,
            ctx: Box::new(Ctx {
                controller,
                card,
                source: Some(object),
                ..Default::default()
            }),
        });
    }

    /// Résolution d'un déclenchement de mot-clé.
    fn resolve_rule_trigger(&mut self, kind: RuleTrigger, obj: crate::ids::ObjRef) {
        if kind == RuleTrigger::BloodDebt {
            // CR 1.7.1a : la couche existe indépendamment de sa source ; la
            // condition (carte publique dans la zone de bannissement) n'est
            // vérifiée qu'au déclenchement (CR 8.3.11a).
            let p = self.obj(obj.id).owner;
            let hero = self.player(p).hero;
            let immune = !self.lost_abilities(hero)
                && self.def(hero).blood_debt_immune.is_some_and(|f| f(self, p));
            if !immune {
                self.lose_life(p, 1);
            }
            return;
        }
        if !self.is_current(obj) {
            return; // la source a quitté sa zone : nouvel objet (CR 3.0.9)
        }
        let id = obj.id;
        match kind {
            // CR 8.3.14b
            RuleTrigger::Spectra | RuleTrigger::BladeBreak => self.destroy(id),
            RuleTrigger::Battleworn => self.obj_mut(id).defense_counters -= 1,
            RuleTrigger::Temper => {
                self.obj_mut(id).defense_counters -= 1;
                if self.defense(id) == 0 {
                    self.destroy(id);
                }
            }
            RuleTrigger::Guardwell => {
                let d = self.defense(id) as i8;
                self.obj_mut(id).defense_counters -= d;
            }
            // CR 8.5.58 : retirer tous les compteurs +1{p} (les compteurs +1
            // et -1 sont additionnés dans `power_counters`).
            RuleTrigger::Unsharpen => {
                let o = self.obj_mut(id);
                o.power_counters = o.power_counters.min(0);
            }
            RuleTrigger::SuspenseTick => {
                let o = self.obj_mut(id);
                o.counters = o.counters.saturating_sub(1);
                if o.counters == 0 {
                    // « When this has no suspense counters, destroy it » :
                    // déclenchement basé sur un état (CR 6.6.1c).
                    self.push_rule_trigger(RuleTrigger::SuspenseEmpty, id);
                }
            }
            RuleTrigger::SuspenseEmpty => {
                if self.obj(id).counters == 0 {
                    self.destroy(id);
                }
            }
            RuleTrigger::BloodDebt => unreachable!("traité plus haut"),
            // CR 8.3.13a : si la condition est toujours remplie, l'attaque
            // est détruite. Pour une attaque-proxy d'allié, c'est l'allié
            // qui est détruit (la proxy cesse alors d'exister, CR 1.4.3c).
            // TODO(règles) : à confirmer par le propriétaire.
            RuleTrigger::Phantasm => {
                let on_active = self.combat.links.last().is_some_and(|l| l.attack == obj);
                if on_active && self.phantasm_condition() {
                    match self.obj(id).proxy_of {
                        Some(src) => self.destroy(src),
                        None => self.destroy(id),
                    }
                }
            }
        }
    }

    pub(crate) fn end_game(&mut self, outcome: Outcome, reason: EndReason) {
        self.outcome = Some(outcome);
        self.end_reason = Some(reason);
        self.stage = Stage::GameOver;
        self.priority = None;
        self.pending = None;
        self.tasks.clear();
    }

    // ------------------------------------------------------------------
    // Phases du tour (CR 4.2 à 4.4)
    // ------------------------------------------------------------------

    fn start_phase(&mut self) {
        // CR 4.2.2 : début du tour, déclenchements résolus sans priorité.
        let tp = self.turn_player;
        self.emit(Event::StartOfTurn(tp));
        // CR 8.3.42 : « At the start of your turn, remove a suspense counter from this. »
        let suspense: Vec<ObjectId> = self
            .permanents
            .iter()
            .copied()
            .filter(|&o| self.obj(o).controller == tp && self.keywords(o).has(Keywords::SUSPENSE))
            .collect();
        for o in suspense {
            self.push_rule_trigger(RuleTrigger::SuspenseTick, o);
        }
        self.stage = Stage::ResolveThen(AfterResolve::ActionPhase);
    }

    /// Résout la pile « comme si tous les joueurs passaient » (CR 4.2.2, 4.4.2).
    fn resolve_without_priority(&mut self, next: AfterResolve) {
        if self.check_heroes() || self.flush_triggers() {
            return;
        }
        if let Some(layer) = self.stack.pop() {
            self.begin_resolve(layer);
            return;
        }
        match next {
            AfterResolve::ActionPhase => {
                // CR 4.3.1-4.3.3
                let tp = self.turn_player;
                self.action_phase = true;
                self.emit(Event::BeginActionPhase(tp));
                self.player_mut(tp).action_points = 1;
                self.give_priority(tp);
            }
            AfterResolve::EndArsenal => self.stage = Stage::EndArsenal,
        }
    }

    /// CR 7.7.4 : la pile se résout comme si tous les joueurs passaient, puis
    /// la chaîne se ferme (CR 7.7.5-7.7.7).
    fn close_step(&mut self) {
        if self.check_heroes() || self.flush_triggers() {
            return;
        }
        if let Some(layer) = self.stack.pop() {
            self.begin_resolve(layer);
            return;
        }
        self.finish_close();
    }

    pub(crate) fn give_priority(&mut self, p: PlayerId) {
        self.stage = Stage::Priority;
        self.priority = Some(p);
    }

    fn priority_step(&mut self) {
        // CR 1.10.2 : actions de l'état de jeu avant la priorité.
        if self.check_heroes() || self.flush_triggers() {
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
        if let Some(top) = self.stack.pop() {
            if self.layer_is_attack(&top) {
                // CR 5.3.1a / 7.1.2a : l'attaque va dans la file d'attente.
                self.combat
                    .queue
                    .push((top.object, top.target.expect("cible d'attaque")));
                // (la cible a été déclarée en jouant l'attaque, CR 1.4.5)
                if self.stack.is_empty() {
                    // CR 7.1.3 : fin de l'étape des couches.
                    self.begin_attack_step();
                } else {
                    self.give_priority(tp);
                }
            } else {
                self.begin_resolve(top);
                self.give_priority(tp);
            }
            return;
        }
        if !self.combat.open {
            // CR 4.3.4 : fin de la phase d'action.
            self.priority = None;
            self.action_phase = false;
            // Steal « until the end of this action phase » (Jack Be Quick).
            if !self.steals.is_empty() {
                self.end_steals();
            }
            self.emit(Event::BeginEndPhase(tp));
            // CR 8.5.58 : déclenchements retardés des sharpen.
            for r in std::mem::take(&mut self.sharpened) {
                if self.is_current(r) {
                    self.push_rule_trigger(RuleTrigger::Unsharpen, r.id);
                }
            }
            // CR 8.3.11, 8.3.11a : blood debt, seulement les cartes publiques
            // de la zone de bannissement du joueur actif.
            for k in 0..self.player(tp).banished.len() {
                let c = self.player(tp).banished[k];
                if !self.obj(c).face_down && self.def(c).keywords.has(Keywords::BLOOD_DEBT) {
                    self.push_rule_trigger(RuleTrigger::BloodDebt, c);
                }
            }
            self.stage = Stage::ResolveThen(AfterResolve::EndArsenal);
            return;
        }
        match self.combat.step.expect("étape de combat") {
            CombatStep::Layer | CombatStep::Resolution => {
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
        }
    }

    // ------------------------------------------------------------------
    // Résolution des couches (CR 5.3)
    // ------------------------------------------------------------------

    fn begin_resolve(&mut self, mut layer: Layer) {
        if let LayerKind::Rule(kind) = layer.kind {
            self.resolve_rule_trigger(kind, layer.object);
            return;
        }
        if layer.kind == LayerKind::Card {
            let id = layer.object.id;
            // CR 5.3.2b : une réaction de défense qui ne peut pas défendre échoue.
            if self.def(id).types.has(Types::DEFENSE_REACTION)
                && !self.can_become_defending(id, layer.origin)
            {
                self.clear(id);
                return;
            }
        }
        // CR 1.8.9 : une cible devenue illégale ne l'est plus pour les effets.
        let specs = match layer.kind {
            LayerKind::Card => self.db.get(layer.card).targets.clone(),
            LayerKind::Activated(i) => self.db.get(layer.card).activated[i as usize]
                .targets
                .clone(),
            LayerKind::Trigger(i) => self.db.get(layer.card).triggers[i as usize].targets.clone(),
            LayerKind::Granted(i) => self.db.get(layer.card).granted[i as usize].targets.clone(),
            LayerKind::Rule(_) => unreachable!(),
        };
        for (i, spec) in specs.iter().enumerate() {
            if let Some(Some(t)) = layer.ctx.targets.get(i).copied() {
                let still = match t {
                    Target::Player(_) => true,
                    Target::Object(r) => self.is_current(r),
                } && (spec.filter)(self, &layer.ctx, t);
                if !still {
                    layer.ctx.targets[i] = None;
                }
            }
        }
        let script = self.script_for(&layer).expect("script");
        let ctx = layer.ctx.clone();
        self.tasks.push(Task::FinishResolve(Box::new(layer)));
        if !self.script_ops(script).is_empty() {
            // CR 5.3.4 : effets de la couche.
            self.tasks.push(Task::Script {
                script,
                pc: 0,
                ctx,
                write_back: false,
            });
        }
    }

    /// CR 5.3.5 à 5.3.7.
    fn finish_resolve(&mut self, mut layer: Layer) {
        match layer.kind {
            LayerKind::Card => {
                let id = layer.object.id;
                let alive = self.is_current(layer.object) && self.obj(id).zone == Zone::Stack;
                // CR 5.3.4d : une carte fusionnée se résout d'abord avec les
                // effets du côté droit ; la résolution s'arrête et le joueur
                // actif reçoit la priorité. La seconde fois, le côté gauche.
                if alive && layer.ctx.melded {
                    let sp = self.def(id).split.expect("carte double");
                    if layer.card != sp.left {
                        layer.card = sp.left;
                        layer.ctx.card = sp.left;
                        self.stack.push(layer);
                        return;
                    }
                }
                // CR 5.3.5 : go again.
                // CR 5.3.5a : une carte devenue permanent pendant sa propre
                // résolution (invocation transformée, CR 8.5.36b) a quitté
                // la pile : on lit son go again imprimé (dernier état connu
                // de la couche) et les effets qui s'appliquent encore à elle.
                // TODO(règles) CR 5.3.5a : dernier état connu des autres
                // cartes qui ont quitté la pile pendant leur résolution.
                let moved_to_arena =
                    !alive && self.is_current(layer.object) && self.obj(id).zone.in_arena();
                // CR 5.3.5a : une carte que sa propre résolution a mise
                // ailleurs (Relentless Pursuit : « put this on the bottom of
                // its owner's deck ») garde le go again imprimé de sa couche.
                let left_stack = !alive
                    && !moved_to_arena
                    && !matches!(self.obj(id).zone, Zone::Gone | Zone::Stack);
                let go_again = if alive {
                    self.keywords(id).has(Keywords::GO_AGAIN)
                } else if moved_to_arena {
                    (self.db.get(layer.card).keywords | self.continuous_delta(id).grants)
                        .has(Keywords::GO_AGAIN)
                } else if left_stack {
                    self.db.get(layer.card).keywords.has(Keywords::GO_AGAIN)
                } else {
                    false
                };
                if go_again {
                    self.gain_action_points(layer.controller, 1);
                }
                if !alive {
                    return;
                }
                let def = self.def(id);
                if def.types.has(Types::DEFENSE_REACTION) {
                    // CR 5.3.6b / 7.4.2d : devient une carte qui défend.
                    self.add_defenders(&[id], layer.origin == Zone::Hand);
                } else if def.subtypes.any(Subtypes::PERMANENT) {
                    // CR 5.3.6a : un permanent entre dans l'arène (une
                    // invocation face arrière active, CR 8.2.13a).
                    self.become_permanent(id, layer.controller);
                } else {
                    // CR 5.3.7
                    self.clear(id);
                }
            }
            LayerKind::Activated(i) => {
                if self.db.get(layer.card).activated[i as usize].go_again {
                    self.gain_action_points(layer.controller, 1);
                }
            }
            LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => {}
        }
    }

    // ------------------------------------------------------------------
    // Combat (CR 7)
    // ------------------------------------------------------------------

    /// CR 7.2 : étape d'attaque.
    fn begin_attack_step(&mut self) {
        let (attack, target) = self.combat.queue.remove(0);
        let tp = self.turn_player;
        let attacker = self.obj(attack.id).controller;
        // CR 1.4.3c : une proxy dont la source a quitté l'arène cesse d'exister.
        let alive = self.is_current(attack)
            && self
                .obj(attack.id)
                .proxy_of
                .is_none_or(|w| self.obj(w).zone.in_arena());
        if !alive || !self.attack_target_legal(attacker, target) {
            // CR 7.2.2c : l'attaque ne peut pas devenir un maillon (ou n'a plus
            // de cible légale) : elle est retirée de la file.
            if self.is_current(attack) && self.obj(attack.id).zone == Zone::Stack {
                self.clear(attack.id);
            }
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
            if self.obj(w).zone == Zone::Permanent {
                // Un allié qui attaque quitte la zone permanente pour la chaîne.
                self.detach(w);
            }
            self.obj_mut(w).zone = Zone::CombatChain;
        }
        let card = self.obj(id).card;
        let target_obj = match target {
            Target::Object(r) => Some(r),
            Target::Player(_) => None,
        };
        self.combat.links.push(ChainLink {
            attack: self.obj_ref(id),
            source,
            card,
            attacker: controller,
            target: self.target_controller(target),
            target_obj,
            defenders: Vec::new(),
            defenders_from_hand: 0,
            hit: false,
            log_start: self.turn_log.len(),
        });
        self.combat.step = Some(CombatStep::Attack);
        // « The next time you attack a marked hero this turn » (Public Bounty).
        if self.next_attack_effects.iter().any(|e| e.at_attack) {
            let r = self.obj_ref(id);
            self.apply_at_attack_effects(controller, r);
        }
        // CR 7.2.3 : événement « attack ».
        let link = self.combat.links.len() as u8;
        self.emit(Event::Attack {
            attack: source.unwrap_or(id),
            attacker: controller,
            target: self.target_controller(target),
            target_obj: target_obj.map(|r| r.id),
            link,
        });
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
        // CR 7.3.2a : on ne déclare des cartes que pour un héros attaqué.
        if link.target_obj.is_some() {
            return vec![Choice::Pass];
        }
        let dominate = self.keywords(link.attack.id).has(Keywords::DOMINATE);
        let hand_declared = self
            .declared_defenders
            .iter()
            .filter(|&&d| self.obj(d).zone == Zone::Hand)
            .count() as u8;
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
            if dominate && hand_declared + link.defenders_from_hand >= 1 {
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
        // CR 8.3.28 : ambush, « while this is in your arsenal, you may defend with it ».
        if let Some(c) = ps.arsenal {
            if !self.declared_defenders.contains(&c)
                && self.def(c).defense.is_some()
                && !self.def(c).types.has(Types::DEFENSE_REACTION)
                && self.keywords(c).has(Keywords::AMBUSH)
            {
                opts.push(Choice::Defend(c));
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
        let from_hand: Vec<bool> = cards
            .iter()
            .map(|&c| self.obj(c).zone == Zone::Hand)
            .collect();
        let with_hand_card = from_hand.iter().any(|&h| h);
        let link = self.combat.links.last().expect("maillon");
        let (defender, attack) = (link.target, link.attack.id);
        for (&c, &h) in cards.iter().zip(&from_hand) {
            self.place(c, Zone::CombatChain, self.obj(c).owner);
            let link = self.combat.links.last_mut().unwrap();
            link.defenders.push(c);
            link.defenders_from_hand += u8::from(h);
        }
        // CR 7.0.5a : événements « defend », une fois toutes les cartes posées.
        for (&c, &h) in cards.iter().zip(&from_hand) {
            self.emit(Event::Defend {
                card: c,
                defender,
                attack,
                with_hand_card,
                from_hand: h,
            });
        }
        if !cards.is_empty() {
            self.check_phantasm();
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
        if link.defenders.contains(&id) || link.target_obj.is_some() {
            return false;
        }
        let dominate = self.keywords(link.attack.id).has(Keywords::DOMINATE);
        !(dominate && from == Zone::Hand && link.defenders_from_hand >= 1)
    }

    /// Ajoute des cartes qui défendent « seules » (réaction de défense…), CR 7.0.5.
    pub fn add_defenders(&mut self, cards: &[ObjectId], from_hand: bool) {
        let Some(link) = self.combat.links.last() else {
            return;
        };
        let (defender, attack) = (link.target, link.attack.id);
        for &c in cards {
            self.place(c, Zone::CombatChain, defender);
            let link = self.combat.links.last_mut().unwrap();
            link.defenders.push(c);
            link.defenders_from_hand += u8::from(from_hand);
        }
        for &c in cards {
            self.emit(Event::Defend {
                card: c,
                defender,
                attack,
                with_hand_card: from_hand,
                from_hand,
            });
        }
        if !cards.is_empty() {
            self.check_phantasm();
        }
    }

    /// CR 8.3.13a : la condition de phantasm est-elle remplie pour
    /// l'attaque active ? (Elle est défendue par une carte d'action
    /// d'attaque non-Illusionist qui a 6{p} ou plus.)
    pub fn phantasm_condition(&self) -> bool {
        let Some(link) = self.combat.links.last() else {
            return false;
        };
        let a = link.attack.id;
        self.is_current(link.attack)
            && self.keywords(a).has(Keywords::PHANTASM)
            && link.defenders.iter().any(|&d| {
                let def = self.def(d);
                self.obj(d).zone == Zone::CombatChain
                    && !self.obj(d).face_down
                    && def.is_action()
                    && def.is_attack_card()
                    && !self.supertypes(d).has(Supertypes::ILLUSIONIST)
                    && self.power(d) >= 6
            })
    }

    /// CR 8.3.13a : phantasm se déclenche quand sa condition devient vraie
    /// (une seule couche à la fois).
    fn check_phantasm(&mut self) {
        let Some(link) = self.combat.links.last() else {
            return;
        };
        let a = link.attack;
        let already = self
            .stack
            .iter()
            .chain(self.pending_triggers.iter())
            .any(|l| l.kind == LayerKind::Rule(RuleTrigger::Phantasm) && l.object == a);
        if !already && self.phantasm_condition() {
            self.push_rule_trigger(RuleTrigger::Phantasm, a.id);
        }
    }

    /// CR 7.5 : étape des dégâts.
    fn damage_step(&mut self) {
        self.combat.step = Some(CombatStep::Damage);
        let link = self.combat.links.last().expect("maillon").clone();
        // CR 7.5.2b : cible disparue ou illégale, pas de dégâts.
        let target = match link.target_obj {
            Some(r) => Target::Object(r),
            None => Target::Player(link.target),
        };
        // CR 1.4.3c : une proxy dont la source a quitté le maillon n'existe plus.
        let source_here = link
            .source
            .is_none_or(|s| self.obj(s).zone == Zone::CombatChain);
        if self.is_current(link.attack)
            && source_here
            && self.attack_target_legal(link.attacker, target)
        {
            let power = self.power(link.attack.id);
            // CR 7.5.2 : somme des défenses des cartes publiques qui défendent.
            let defense: i16 = link
                .defenders
                .iter()
                .filter(|&&d| !self.obj(d).face_down)
                .map(|&d| self.defense(d))
                .sum();
            if power > defense {
                // Événement « hit » (CR 7.5.2a, 7.5.5) : dégâts physiques à la
                // cible. CR 7.5.5b : si tout est prévenu, l'attaque ne touche
                // pas (voir `finish_damage`). Les dégâts peuvent attendre un
                // choix de prévention : la tâche passe avant la priorité.
                let source = link.source.unwrap_or(link.attack.id);
                self.run_damage(Box::new(crate::effects::DamageEvent {
                    source: Some(source),
                    by: link.attacker,
                    target,
                    amount: (power - defense) as i32,
                    arcane: false,
                    hit: true,
                    effect_hit: false,
                    replaced: false,
                    declined: false,
                    used: Vec::new(),
                    on_prevent: Vec::new(),
                }));
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
        // CR 7.6.2 / 8.3.5b : go again ; CR 7.6.2a : sinon, dernier état connu.
        let go_again = if self.is_current(link.attack) {
            self.keywords(link.attack.id).has(Keywords::GO_AGAIN)
        } else {
            self.db.get(link.card).keywords.has(Keywords::GO_AGAIN)
        };
        let source = link.source.unwrap_or(link.attack.id);
        self.emit(Event::ChainLinkResolved {
            attack: source,
            attacker: link.attacker,
        });
        if go_again {
            self.gain_action_points(link.attacker, 1);
        }
        let tp = self.turn_player;
        self.give_priority(tp);
    }

    /// CR 7.7.3 : début de l'étape de fermeture de la chaîne de combat.
    pub(crate) fn close_combat_chain(&mut self) {
        // Événement « combat chain closes » : les capacités se déclenchent
        // (ex. Hell Hammer), dont blade break, battleworn, temper, guardwell.
        self.emit(Event::CombatChainClosed);
        let defenders: Vec<ObjectId> = self
            .combat
            .links
            .iter()
            .flat_map(|l| l.defenders.iter().copied())
            .collect();
        for id in defenders {
            // « if this defended » : la carte est encore sur la chaîne. Ces
            // mots-clés n'existent que sur des équipements.
            if self.obj(id).zone != Zone::CombatChain || !self.def(id).types.has(Types::EQUIPMENT) {
                continue;
            }
            let kw = self.keywords(id);
            for (k, t) in [
                (Keywords::BLADE_BREAK, RuleTrigger::BladeBreak),
                (Keywords::TEMPER, RuleTrigger::Temper),
                (Keywords::GUARDWELL, RuleTrigger::Guardwell),
                (Keywords::BATTLEWORN, RuleTrigger::Battleworn),
            ] {
                if kw.has(k) {
                    self.push_rule_trigger(t, id);
                }
            }
        }
        // Attaques, couches d'attaque et réactions quittent la pile et la file.
        let stack = std::mem::take(&mut self.stack);
        for l in stack {
            if self.is_combat_layer(&l) {
                let o = l.object.id;
                let is_object_layer = l.kind == LayerKind::Card || self.obj(o).proxy_of.is_some();
                if is_object_layer && self.is_current(l.object) && self.obj(o).zone == Zone::Stack {
                    self.clear(o);
                }
            } else {
                self.stack.push(l);
            }
        }
        for (a, _) in std::mem::take(&mut self.combat.queue) {
            if self.is_current(a) && self.obj(a.id).zone == Zone::Stack {
                self.clear(a.id);
            }
        }
        // CR 7.7.1 / 7.7.4 : pas de priorité ; la suite dans `close_step`.
        self.priority = None;
        self.passes = 0;
        self.stage = Stage::Closing;
    }

    /// Une couche que CR 7.7.3 retire de la pile : attaque, couche d'attaque,
    /// réaction (carte ou capacité activée).
    fn is_combat_layer(&self, l: &Layer) -> bool {
        let def = self.db.get(l.card);
        match l.kind {
            LayerKind::Card => {
                def.subtypes.has(Subtypes::ATTACK)
                    || def
                        .types
                        .any(Types::ATTACK_REACTION | Types::DEFENSE_REACTION)
            }
            LayerKind::Activated(i) => {
                let a = &def.activated[i as usize];
                a.attack || matches!(a.speed, Speed::AttackReaction | Speed::DefenseReaction)
            }
            LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => false,
        }
    }

    /// CR 7.7.5 à 7.7.7 : fin de la fermeture, une fois la pile vide.
    fn finish_close(&mut self) {
        // CR 7.7.5 : les permanents retournent dans leurs zones.
        let on_chain: Vec<ObjectId> = (0..self.objects.len() as u32)
            .map(ObjectId)
            .filter(|&id| self.obj(id).zone == Zone::CombatChain)
            .collect();
        for id in on_chain {
            // CR 1.3.3 : une carte de deck sur la chaîne n'est pas un permanent
            // (ex. une aura qui défend depuis la main). Seuls les équipements,
            // armes, tokens, et les permanents venus attaquer (alliés) le sont.
            let def = self.def(id);
            let is_source = self.combat.links.iter().any(|l| l.source == Some(id));
            let is_permanent = self.obj(id).proxy_of.is_none()
                && (def
                    .types
                    .any(Types::EQUIPMENT | Types::WEAPON | Types::TOKEN)
                    || is_source);
            if is_permanent {
                self.return_to_equipment_zone(id);
            } else {
                // CR 7.7.6 : le reste est « clear ».
                self.clear(id);
            }
        }
        // CR 7.7.7 : la chaîne se ferme, fin des effets « this combat chain ».
        self.combat.links.clear();
        self.combat.open = false;
        self.combat.step = None;
        self.modifiers
            .retain(|m| m.until != Until::CombatChainCloses);
        self.any_lose_abilities = self.modifiers.iter().any(|m| m.delta.lose_abilities);
        self.preventions
            .retain(|m| m.until != Until::CombatChainCloses);
        self.replacements
            .retain(|m| m.until != Until::CombatChainCloses);
        self.next_attack_effects.retain(|e| !e.combat_chain);
        if !self.granted_triggers.is_empty() {
            let mut g = std::mem::take(&mut self.granted_triggers);
            g.retain(|x| x.until != Until::CombatChainCloses && self.is_current(x.target));
            self.granted_triggers = g;
        }
        let tp = self.turn_player;
        self.give_priority(tp);
    }

    // ------------------------------------------------------------------
    // Phase de fin (CR 4.4)
    // ------------------------------------------------------------------

    fn end_arsenal(&mut self) {
        // CR 4.4.3a : la vie des alliés revient à leur vie de base.
        self.reset_ally_life();
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
        // CR 4.4.3c : chaque joueur met son pitch sous son deck, dans l'ordre de son choix,
        // en commençant par le joueur actif (CR 1.9.2c).
        let pitch = &self.player(p).pitch;
        let mut cards: Vec<_> = pitch.iter().map(|&c| self.obj(c).card).collect();
        cards.sort();
        cards.dedup();
        if pitch.is_empty() || cards.len() <= 1 {
            // Ordre sans importance (aucune carte, ou cartes identiques).
            for c in pitch.clone() {
                self.move_to(c, Zone::Deck, false);
            }
            self.stage = if p == self.turn_player {
                Stage::EndPitchOrder(p.opponent())
            } else {
                Stage::EndRest
            };
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
        for o in &mut self.objects {
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
        let turn = self.turn;
        self.modifiers.retain(|m| !m.until.ends_at_end_of(turn));
        self.any_lose_abilities = self.modifiers.iter().any(|m| m.delta.lose_abilities);
        self.preventions.retain(|m| !m.until.ends_at_end_of(turn));
        self.replacements.retain(|m| !m.until.ends_at_end_of(turn));
        self.next_attack_effects
            .retain(|e| e.turn.is_some_and(|t| t > turn));
        self.attack_buffs.clear();
        self.weapon_effects.clear();
        self.play_permissions.retain(|x| x.turn > turn);
        self.cost_reductions.retain(|x| x.turn > turn);
        self.instant_permissions.retain(|x| x.turn > turn);
        self.no_power_gain.retain(|&(_, t)| t > turn);
        self.granted_triggers
            .retain(|x| !x.until.ends_at_end_of(turn));
        self.sharpened.clear();
        self.turn_log.clear();
        for o in &mut self.objects {
            o.activations = [0; 4];
        }
        if self.is_stalemate() || self.no_progress() {
            self.end_game(Outcome::Draw, EndReason::Stalemate);
            return;
        }
        if self.turn >= self.max_turns {
            self.end_game(Outcome::Draw, EndReason::TurnLimit);
            return;
        }
        self.turn += 1;
        self.turn_player = tp.opponent();
        self.stage = Stage::StartPhase;
    }

    /// CR 4.5.4d, cas simple : aucun joueur n'a de carte en deck, main ou
    /// arsenal, et aucune capacité activable gratuitement. Plus rien ne peut
    /// faire baisser un total de vie.
    fn is_stalemate(&self) -> bool {
        PlayerId::BOTH.iter().all(|&p| {
            let ps = self.player(p);
            let no_cards = ps.deck.is_empty() && ps.hand.is_empty() && ps.arsenal.is_none();
            let free_ability = std::iter::once(ps.hero)
                .chain(ps.weapons.iter().copied())
                .chain(ps.equipment())
                .chain(self.permanents.iter().copied())
                .any(|o| {
                    self.def(o).activated.iter().any(|a| a.cost == 0)
                        || !self.def(o).triggers.is_empty()
                });
            no_cards && !free_ability
        })
    }

    /// CR 4.5.4d-e, cas étendu (heuristique du simulateur) : plus aucune
    /// carte dans les decks et les mains, et aucun total de vie n'a changé
    /// pendant 6 tours de suite. Ce qui reste (une carte injouable en
    /// arsenal, des capacités déclenchées du héros comme pour Arakni ou
    /// Dromai) ne fait plus avancer la partie. Appelé une fois à la fin de
    /// chaque tour.
    fn no_progress(&mut self) -> bool {
        let no_cards = PlayerId::BOTH.iter().all(|&p| {
            let ps = self.player(p);
            ps.deck.is_empty() && ps.hand.is_empty()
        });
        let life = [self.players[0].life, self.players[1].life];
        if !no_cards || life != self.stale_life {
            self.stale_turns = 0;
            self.stale_life = life;
            return false;
        }
        self.stale_turns += 1;
        self.stale_turns >= 6
    }

    /// Exécute un script hors pile (ex. coûts), pour les tests et les cartes.
    pub fn start_script(&mut self, script: ScriptRef, ctx: crate::script::Ctx) {
        self.tasks.push(Task::Script {
            script,
            pc: 0,
            ctx: Box::new(ctx),
            write_back: false,
        });
    }
}
