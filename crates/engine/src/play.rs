//! Jouer une carte (CR 5.1), activer une capacité (CR 5.2) et payer les coûts (CR 1.14).

use crate::card::{Delta, Keywords, Speed, Subtypes, Types};
use crate::decision::{Choice, Decision, DecisionKind};
use crate::events::Event;
use crate::ids::{ObjectId, PlayerId};
use crate::script::{Ctx, ScriptRef, Target, TargetSpec};
use crate::state::{CombatStep, GameState, Layer, LayerKind, Object, Task, Until};
use crate::zones::Zone;

impl GameState {
    /// Toutes les options d'un joueur qui a la priorité. `Pass` est toujours la dernière.
    pub fn legal_priority_options(&self, p: PlayerId) -> Vec<Choice> {
        let mut opts = Vec::new();
        let ps = self.player(p);
        for &c in ps.hand.iter().chain(ps.arsenal.iter()) {
            if self.can_play(p, c) {
                opts.push(Choice::Play(c));
            }
        }
        for src in self.ability_sources(p) {
            for i in 0..self.def(src).activated.len() {
                if self.can_activate(p, src, i as u8) {
                    opts.push(Choice::Activate(src, i as u8));
                }
            }
        }
        opts.push(Choice::Pass);
        opts
    }

    /// Objets dont le joueur peut activer les capacités : héros, armes,
    /// équipements, permanents contrôlés, et cartes en main (« Discard this »).
    fn ability_sources(&self, p: PlayerId) -> Vec<ObjectId> {
        let ps = self.player(p);
        let mut v: Vec<ObjectId> = std::iter::once(ps.hero)
            .chain(ps.weapons.iter().copied())
            .chain(ps.equipment())
            .chain(
                self.permanents
                    .iter()
                    .copied()
                    .filter(|&o| self.obj(o).controller == p),
            )
            .chain(ps.hand.iter().copied())
            .filter(|&o| !self.def(o).activated.is_empty())
            .collect();
        v.dedup();
        v
    }

    /// Ressources que le joueur peut réunir sans utiliser `except` (CR 1.14.2d, 1.14.3).
    pub fn available_resources(&self, p: PlayerId, except: Option<ObjectId>) -> u32 {
        let ps = self.player(p);
        let pitchable: u32 = ps
            .hand
            .iter()
            .filter(|&&c| Some(c) != except)
            .map(|&c| self.def(c).pitch_value() as u32)
            .sum();
        ps.resource_points as u32 + pitchable
    }

    /// Le moment est-il bon pour une carte / capacité de cette vitesse ?
    fn timing_ok(&self, p: PlayerId, speed: Speed, is_attack: bool) -> bool {
        match speed {
            // CR 8.1.6a
            Speed::Instant => true,
            // CR 8.1.1a-c et 7.0.1a
            Speed::Action => {
                if p != self.turn_player
                    || self.player(p).action_points < 1
                    || !self.stack.is_empty()
                {
                    return false;
                }
                if !self.combat.open {
                    return true;
                }
                // TODO(règles) : CR 8.1.1b semble autoriser toute action à l'étape de
                // résolution, CR 7.0.1a seulement les attaques. On applique 7.0.1a.
                is_attack && self.combat.step == Some(CombatStep::Resolution)
            }
            // CR 8.1.2a
            Speed::AttackReaction => {
                self.combat.step == Some(CombatStep::Reaction)
                    && self.active_link().is_some_and(|l| l.attacker == p)
            }
            // CR 8.1.3a
            Speed::DefenseReaction => {
                self.combat.step == Some(CombatStep::Reaction)
                    && self.active_link().is_some_and(|l| l.target == p)
            }
        }
    }

    fn card_speed(&self, c: ObjectId) -> Option<Speed> {
        let t = self.def(c).types;
        if t.has(Types::INSTANT) {
            Some(Speed::Instant)
        } else if t.has(Types::ACTION) {
            Some(Speed::Action)
        } else if t.has(Types::ATTACK_REACTION) {
            Some(Speed::AttackReaction)
        } else if t.has(Types::DEFENSE_REACTION) {
            Some(Speed::DefenseReaction)
        } else {
            None // ressources, blocs (CR 8.1.7a, 8.1.12a), équipements…
        }
    }

    /// Coût en ressources d'une carte (CR 5.1.6), sans coût optionnel ni X.
    pub fn play_cost(&self, p: PlayerId, c: ObjectId) -> u32 {
        let def = self.def(c);
        let mut cost = def.cost.unwrap_or(0) as i16;
        if let Some(a) = def.additional_cost {
            if !a.optional {
                cost += a.resources as i16;
            }
        }
        if let Some(f) = def.cost_mod {
            cost += f(self, p, c);
        }
        cost.max(0) as u32
    }

    /// Toutes les cibles possibles pour une spécification (CR 1.8.5a : objets
    /// publics dans l'arène ou sur la pile, et les héros).
    pub fn target_candidates(&self, spec: &TargetSpec, ctx: &Ctx) -> Vec<Target> {
        let mut v = Vec::new();
        for p in PlayerId::BOTH {
            let t = Target::Player(p);
            if (spec.filter)(self, ctx, t) {
                v.push(t);
            }
        }
        for (i, o) in self.objects.iter().enumerate() {
            let public_here = (o.zone.in_arena() && o.zone != Zone::Hero) || o.zone == Zone::Stack;
            if !public_here || o.face_down {
                continue;
            }
            let t = Target::Object(o.obj_ref(ObjectId(i as u32)));
            if (spec.filter)(self, ctx, t) {
                v.push(t);
            }
        }
        v
    }

    fn targets_available(&self, specs: &[TargetSpec], ctx: &Ctx) -> bool {
        specs
            .iter()
            .all(|s| s.optional || !self.target_candidates(s, ctx).is_empty())
    }

    pub(crate) fn can_play(&self, p: PlayerId, c: ObjectId) -> bool {
        let o = self.obj(c);
        let def = self.def(c);
        // CR 5.1.1a : seulement depuis la main ou l'arsenal, par son propriétaire.
        if o.owner != p || !matches!(o.zone, Zone::Hand | Zone::Arsenal) {
            return false;
        }
        let Some(speed) = self.card_speed(c) else {
            return false;
        };
        if !self.timing_ok(p, speed, def.is_attack_card()) {
            return false;
        }
        // CR 7.4.2c : une réaction de défense doit pouvoir défendre.
        if speed == Speed::DefenseReaction && !self.can_become_defending(c, o.zone) {
            return false;
        }
        if def.play_condition.is_some_and(|f| !f(self, p, c)) {
            return false;
        }
        let ctx = Ctx {
            controller: p,
            card: o.card,
            source: Some(self.obj_ref(c)),
            ..Default::default()
        };
        self.targets_available(&def.targets, &ctx)
            && self.play_cost(p, c) <= self.available_resources(p, Some(c))
    }

    pub(crate) fn can_activate(&self, p: PlayerId, src: ObjectId, i: u8) -> bool {
        let o = self.obj(src);
        let a = &self.def(src).activated[i as usize];
        let zone_ok = if a.from_hand {
            o.zone == Zone::Hand && o.owner == p
        } else {
            o.controller == p && o.zone.in_arena()
        };
        if !zone_ok || (a.tap && o.tapped) || (a.once_per_turn && o.activations[i as usize] >= 1) {
            return false;
        }
        if !self.timing_ok(p, a.speed, a.attack) {
            return false;
        }
        if a.condition.is_some_and(|f| !f(self, p, src)) {
            return false;
        }
        let ctx = Ctx {
            controller: p,
            card: o.card,
            source: Some(self.obj_ref(src)),
            ..Default::default()
        };
        let except = if a.from_hand { Some(src) } else { None };
        self.targets_available(&a.targets, &ctx)
            && a.cost as u32 <= self.available_resources(p, except)
    }

    /// CR 5.1.2 : annonce, puis les étapes suivantes sous forme de tâches.
    pub(crate) fn play_card(&mut self, p: PlayerId, c: ObjectId) {
        let origin = self.obj(c).zone;
        let is_attack = self.def(c).is_attack_card();
        self.place(c, Zone::Stack, p);
        let object = self.obj_ref(c);
        let card = self.obj(c).card;
        // Cible d'attaque (CR 5.1.4b) : en partie à 2 sans allié, le héros adverse.
        // TODO(étape 2) : attaquer un allié ou un objet avec spectra.
        let target = if is_attack { Some(p.opponent()) } else { None };
        let ctx = Box::new(Ctx {
            controller: p,
            card,
            source: Some(object),
            ..Default::default()
        });
        self.stack.push(Layer {
            kind: LayerKind::Card,
            object,
            card,
            controller: p,
            target,
            origin,
            ctx,
        });
        if is_attack {
            self.apply_next_attack_effects(p, object, true);
        }
        let n_targets = self.def(c).targets.len();
        self.push_play_tasks(n_targets);
    }

    /// CR 5.2.2 : activer une capacité.
    pub(crate) fn activate(&mut self, p: PlayerId, src: ObjectId, i: u8) {
        let card = self.obj(src).card;
        let origin = self.obj(src).zone;
        let a = self.db.get(card).activated[i as usize].clone();
        self.obj_mut(src).activations[i as usize] += 1;
        let object = if a.attack {
            // CR 1.4.3 : l'attaque-proxy est un nouvel objet sur la pile.
            self.objects.push(Object {
                card,
                owner: p,
                controller: p,
                zone: Zone::Stack,
                gen: 0,
                face_down: false,
                tapped: false,
                defense_counters: 0,
                power_counters: 0,
                counters: 0,
                proxy_of: Some(src),
                activations: [0; 4],
            });
            let proxy = ObjectId(self.objects.len() as u32 - 1);
            let r = self.obj_ref(proxy);
            if a.go_again {
                self.modify(
                    r,
                    Delta {
                        grants: Keywords::GO_AGAIN,
                        ..Default::default()
                    },
                    Until::EndOfTurn,
                );
            }
            self.apply_next_attack_effects(p, r, false);
            r
        } else {
            self.obj_ref(src)
        };
        let target = if a.attack { Some(p.opponent()) } else { None };
        let ctx = Box::new(Ctx {
            controller: p,
            card,
            source: Some(self.obj_ref(src)),
            ..Default::default()
        });
        self.stack.push(Layer {
            kind: LayerKind::Activated(i),
            object,
            card,
            controller: p,
            target,
            origin,
            ctx,
        });
        self.push_play_tasks(a.targets.len());
    }

    fn push_play_tasks(&mut self, n_targets: usize) {
        // CR 1.11.5 : personne n'a la priorité pendant qu'on joue.
        self.priority = None;
        self.tasks.push(Task::PlayFinish);
        self.tasks.push(Task::PlayEffectCost);
        self.tasks.push(Task::PlayPay);
        for t in (0..n_targets).rev() {
            self.tasks.push(Task::PlayTarget(t as u8));
        }
        self.tasks.push(Task::PlayMode);
        self.tasks.push(Task::PlayX);
        self.tasks.push(Task::PlayOptionalCost);
    }

    fn top_layer(&self) -> &Layer {
        self.stack.last().expect("couche en cours")
    }

    /// Spécifications de cibles de la couche du dessus.
    fn top_target_specs(&self) -> Vec<TargetSpec> {
        let l = self.top_layer();
        match l.kind {
            LayerKind::Card => self.db.get(l.card).targets.clone(),
            LayerKind::Activated(i) => self.db.get(l.card).activated[i as usize].targets.clone(),
            LayerKind::Trigger(i) => self.db.get(l.card).triggers[i as usize].targets.clone(),
        }
    }

    /// Exécute une étape de la procédure de jeu (CR 5.1.3 à 5.1.10).
    pub(crate) fn run_play_task(&mut self, task: Task) {
        let l = self.top_layer();
        let (p, card, kind) = (l.controller, l.card, l.kind);
        let def = self.db.get(card);
        match task {
            // CR 5.1.3b
            Task::PlayOptionalCost => {
                if let (LayerKind::Card, Some(a)) = (kind, def.additional_cost) {
                    let base = self.play_cost(p, l.object.id);
                    if a.optional && base + a.resources as u32 <= self.available_resources(p, None)
                    {
                        self.pending = Some(Decision {
                            player: p,
                            kind: DecisionKind::OptionalCost,
                            options: vec![Choice::Yes, Choice::No],
                        });
                    }
                }
            }
            // CR 5.1.3a
            Task::PlayX => {
                if kind == LayerKind::Card && def.x_cost {
                    let spare = self
                        .available_resources(p, None)
                        .saturating_sub(self.layer_cost(p));
                    let options = (0..=spare.min(20) as u8).map(Choice::Number).collect();
                    self.pending = Some(Decision {
                        player: p,
                        kind: DecisionKind::ChooseX,
                        options,
                    });
                }
            }
            // CR 5.1.4a
            Task::PlayMode => {
                if kind == LayerKind::Card && !def.modes.is_empty() {
                    let options = (0..def.modes.len() as u8).map(Choice::Mode).collect();
                    self.pending = Some(Decision {
                        player: p,
                        kind: DecisionKind::Mode,
                        options,
                    });
                }
            }
            Task::PlayTarget(i) => {
                let spec = self.top_target_specs()[i as usize];
                let ctx = (*self.top_layer().ctx).clone();
                let mut options: Vec<Choice> = self
                    .target_candidates(&spec, &ctx)
                    .into_iter()
                    .map(Choice::Target)
                    .collect();
                if spec.optional {
                    options.push(Choice::Pass);
                }
                match options.len() {
                    0 => self.stack.last_mut().unwrap().ctx.targets.push(None),
                    1 if !spec.optional => {
                        let Choice::Target(t) = options[0] else {
                            unreachable!()
                        };
                        self.stack.last_mut().unwrap().ctx.targets.push(Some(t));
                    }
                    _ => {
                        self.pending = Some(Decision {
                            player: p,
                            kind: DecisionKind::Target,
                            options,
                        })
                    }
                }
            }
            // CR 5.1.6-5.1.7
            Task::PlayPay => {
                let resources = self.layer_cost(p) as u8;
                let action_points = match kind {
                    // CR 5.1.6b : une action non jouée comme instant coûte 1 point d'action.
                    LayerKind::Card => u8::from(def.types.has(Types::ACTION)),
                    LayerKind::Activated(i) => {
                        u8::from(def.activated[i as usize].speed == Speed::Action)
                    }
                    LayerKind::Trigger(_) => 0,
                };
                self.tasks.push(Task::Pay {
                    player: p,
                    resources,
                    action_points,
                });
            }
            // CR 5.1.8-5.1.9
            Task::PlayEffectCost => {
                let script = match kind {
                    LayerKind::Card => ScriptRef::EffectCost(card),
                    LayerKind::Activated(i) => {
                        if def.activated[i as usize].tap {
                            let src = self.top_layer().ctx.source_id().unwrap();
                            self.obj_mut(src).tapped = true;
                        }
                        ScriptRef::ActivatedCost(card, i)
                    }
                    LayerKind::Trigger(_) => return,
                };
                if !self.script_ops(script).is_empty() {
                    let ctx = self.top_layer().ctx.clone();
                    self.tasks.push(Task::Script {
                        script,
                        pc: 0,
                        ctx,
                        write_back: true,
                    });
                }
            }
            // CR 5.1.10
            Task::PlayFinish => self.finish_play(),
            _ => unreachable!(),
        }
    }

    /// Coût total en ressources de la couche du dessus (coût optionnel et X compris).
    fn layer_cost(&self, p: PlayerId) -> u32 {
        let l = self.top_layer();
        let def = self.db.get(l.card);
        match l.kind {
            LayerKind::Card => {
                let mut c = self.play_cost(p, l.object.id);
                if def.additional_cost.is_some_and(|a| a.optional) && l.ctx.optional_paid {
                    c += def.additional_cost.unwrap().resources as u32;
                }
                c + l.ctx.x as u32
            }
            LayerKind::Activated(i) => def.activated[i as usize].cost as u32,
            LayerKind::Trigger(_) => 0,
        }
    }

    /// CR 1.14.2 : demande la prochaine carte à pitcher, ou termine le paiement.
    pub(crate) fn run_pay(&mut self, p: PlayerId, resources: u8, action_points: u8) {
        if self.player(p).resource_points >= resources {
            let ps = self.player_mut(p);
            ps.resource_points -= resources;
            ps.action_points -= action_points; // CR 1.14.2a : ressources puis points d'action
            return;
        }
        self.tasks.push(Task::Pay {
            player: p,
            resources,
            action_points,
        });
        // CR 1.14.3a-b : seulement les cartes avec une valeur de pitch.
        let options: Vec<Choice> = self
            .player(p)
            .hand
            .iter()
            .filter(|&&c| self.def(c).pitch_value() > 0)
            .map(|&c| Choice::Pitch(c))
            .collect();
        debug_assert!(
            !options.is_empty(),
            "coût vérifié avant de proposer la carte"
        );
        self.pending = Some(Decision {
            player: p,
            kind: DecisionKind::Pitch,
            options,
        });
    }

    /// CR 1.14.3 : pitcher une carte de la main.
    pub(crate) fn pitch(&mut self, p: PlayerId, c: ObjectId) {
        let value = self.def(c).pitch_value();
        self.move_to(c, Zone::Pitch, true);
        self.player_mut(p).resource_points += value;
        self.emit(Event::Pitched { player: p, card: c });
    }

    /// CR 5.1.10 : la carte est jouée, le joueur récupère la priorité.
    fn finish_play(&mut self) {
        let l = self.top_layer();
        let (p, kind, object, origin) = (l.controller, l.kind, l.object, l.origin);
        let is_attack = self.layer_is_attack(l);
        match kind {
            LayerKind::Card => self.emit(Event::Played {
                player: p,
                card: object.id,
                from: origin,
            }),
            LayerKind::Activated(i) => {
                let source = l.ctx.source_id().unwrap();
                self.emit(Event::Activated {
                    player: p,
                    source,
                    ability: i,
                })
            }
            LayerKind::Trigger(_) => {}
        }
        if is_attack {
            if !self.combat.open {
                // CR 7.0.2a / 8.2.3b : la chaîne de combat s'ouvre.
                self.combat.open = true;
                self.combat.step = Some(CombatStep::Layer);
                self.combat.log_start = self.turn_log.len();
            } else if self.combat.step == Some(CombatStep::Resolution) {
                // CR 7.6.3a
                self.combat.step = Some(CombatStep::Layer);
            }
        }
        self.passes = 0;
        self.give_priority(p);
    }

    /// La couche est-elle une attaque (carte d'attaque ou attaque-proxy) ?
    pub(crate) fn layer_is_attack(&self, l: &Layer) -> bool {
        match l.kind {
            LayerKind::Card => self.db.get(l.card).subtypes.has(Subtypes::ATTACK),
            LayerKind::Activated(i) => self.db.get(l.card).activated[i as usize].attack,
            LayerKind::Trigger(_) => false,
        }
    }
}
