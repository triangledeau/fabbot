//! Jouer une carte (CR 5.1), activer une capacité (CR 5.2) et payer les coûts (CR 1.14).

use crate::card::{Delta, Keywords, Speed, Split, Subtypes, Types};
use crate::decision::{Choice, Decision, DecisionKind};
use crate::events::Event;
use crate::ids::{CardId, ObjectId, PlayerId};
use crate::script::{Ctx, ScriptRef, Target, TargetSpec};
use crate::state::{
    CombatStep, GameState, Layer, LayerKind, Object, RuleTrigger, Task, Until, WeaponEffectKind,
};
use crate::zones::Zone;

/// Case de `Object::activations` qui note l'usage, ce tour, de la permission
/// « Once per turn, you may play … from your banished zone » d'un permanent
/// (remise à zéro à la fin de chaque tour, comme les activations). Les
/// permanents qui ont cette permission n'ont pas 4 capacités activées.
const BANISHED_PLAY_SLOT: usize = 3;

impl GameState {
    /// Toutes les options d'un joueur qui a la priorité. `Pass` est toujours la dernière.
    pub fn legal_priority_options(&self, p: PlayerId) -> Vec<Choice> {
        let mut opts = Vec::new();
        // Themai : « Opponents can't play cards or activate abilities during
        // your turn. »
        if self.silenced(p) {
            opts.push(Choice::Pass);
            return opts;
        }
        let ps = self.player(p);
        for &c in ps.hand.iter().chain(ps.arsenal.iter()) {
            if self.can_play(p, c) {
                opts.push(Choice::Play(c));
            }
        }
        // Objets que le joueur a la permission de jouer depuis sa zone de
        // bannissement (« you may play it this turn », Blasmophet).
        if !self.play_permissions.is_empty() || self.banished_play_sources(p).next().is_some() {
            for &c in &ps.banished {
                if self.can_play(p, c) && !opts.contains(&Choice::Play(c)) {
                    opts.push(Choice::Play(c));
                }
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
                // CR 7.0.1a, 7.6.3a : pendant le combat, seulement des attaques
                // (et couches d'attaque), à l'étape de résolution. CR 8.1.1b est
                // la règle générale, 7.0.1a précise lesquelles (confirmé par le
                // propriétaire) : une action non-attaque attend la fermeture.
                is_attack && self.combat.step == Some(CombatStep::Resolution)
            }
            // CR 8.1.2a
            Speed::AttackReaction => {
                self.combat.step == Some(CombatStep::Reaction)
                    && self.active_link().is_some_and(|l| l.attacker == p)
            }
            // CR 8.1.3a : seulement le joueur dont le **héros** est attaqué.
            Speed::DefenseReaction => {
                self.combat.step == Some(CombatStep::Reaction)
                    && self
                        .active_link()
                        .is_some_and(|l| l.target == p && l.target_obj.is_none())
            }
        }
    }

    fn card_speed(&self, card: CardId) -> Option<Speed> {
        let t = self.db.get(card).types;
        // CR 8.3.38c : une carte fusionnée a les types des deux côtés (Action
        // et Instant) ; c'est une carte d'action, soumise à CR 8.1.1a-c.
        // TODO(règles) : à confirmer par le propriétaire (Arcane Seeds // Life).
        if t.has(Types::ACTION) {
            Some(Speed::Action)
        } else if t.has(Types::INSTANT) {
            Some(Speed::Instant)
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
        self.play_cost_as(p, c, self.obj(c).card)
    }

    /// Coût de la carte `c` jouée avec la définition `card` (côté d'une carte
    /// double, CR 9.2.3 ; fusionnée : deux fois le coût de base, CR 8.3.38b).
    fn play_cost_as(&self, p: PlayerId, c: ObjectId, card: CardId) -> u32 {
        let def = self.db.get(card);
        let mut cost = def.cost.unwrap_or(0) as i16;
        if let Some(a) = def.additional_cost {
            if !a.optional {
                cost += a.resources as i16;
            }
        }
        if let Some(f) = def.cost_mod {
            cost += f(self, p, c);
        }
        // CR 5.1.6a : les augmentations, puis les réductions.
        cost += self.cost_tax(p) as i16;
        cost -= self.cost_reduction(p, c) as i16;
        cost.max(0) as u32
    }

    /// « Cards and abilities cost you an additional {r} to play or activate »
    /// (Frostbite) : somme des surcoûts des permanents que `p` contrôle.
    pub fn cost_tax(&self, p: PlayerId) -> u32 {
        let ps = self.player(p);
        self.permanents
            .iter()
            .copied()
            .filter(|&o| self.obj(o).controller == p)
            .chain(ps.equipment())
            .map(|o| {
                let t = self.def(o).cost_tax;
                if t > 0 && !self.lost_abilities(o) {
                    t as u32
                } else {
                    0
                }
            })
            .sum()
    }

    /// La carte `c` peut-elle être jouée par `p` « as though it were an
    /// instant » (CR 8.1.1d) ? Capacités statiques (Iyslander) ou permission
    /// « your next … » (Stir the Aetherwinds).
    pub fn may_play_as_instant(&self, p: PlayerId, c: ObjectId) -> bool {
        if !self.def(c).types.has(Types::ACTION) {
            return false;
        }
        if self
            .instant_permissions
            .iter()
            .any(|x| x.player == p && x.turn == self.turn && (x.filter)(self, c))
        {
            return true;
        }
        let ps = self.player(p);
        std::iter::once(ps.hero)
            .chain(ps.weapons.iter().copied())
            .chain(ps.equipment())
            .chain(
                self.permanents
                    .iter()
                    .copied()
                    .filter(|&o| self.obj(o).controller == p),
            )
            .any(|src| {
                self.def(src)
                    .play_as_instant
                    .is_some_and(|f| !self.lost_abilities(src) && f(self, src, c))
            })
    }

    /// Somme des réductions « your next … costs {r} less » qui s'appliquent à
    /// la carte `c` jouée par `p` ce tour (CR 5.1.6).
    pub fn cost_reduction(&self, p: PlayerId, c: ObjectId) -> u8 {
        self.cost_reductions
            .iter()
            .filter(|x| x.player == p && x.turn == self.turn && (x.filter)(self, c))
            .map(|x| x.amount)
            .sum()
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
            // CR 1.8.5a : « target … card from your graveyard » précise une
            // zone publique hors de l'arène.
            let public_here = (o.zone.in_arena() && o.zone != Zone::Hero)
                || o.zone == Zone::Stack
                || (spec.in_graveyard && o.zone == Zone::Graveyard)
                || (spec.in_banished && o.zone == Zone::Banished);
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

    /// Carte modale (CR 1.7.5) : au moins un mode a ses cibles (le filtre
    /// d'une cible peut dépendre du mode, `Ctx::mode`). Sans mode : les cibles.
    fn some_mode_has_targets(&self, specs: &[TargetSpec], n_modes: usize, mut ctx: Ctx) -> bool {
        if n_modes == 0 || specs.is_empty() {
            return self.targets_available(specs, &ctx);
        }
        (0..n_modes as u8).any(|m| {
            ctx.mode = m;
            self.targets_available(specs, &ctx)
        })
    }

    pub(crate) fn can_play(&self, p: PlayerId, c: ObjectId) -> bool {
        match self.def(c).split {
            // CR 5.1.2c : une carte double est jouée avec un de ses côtés,
            // ou les deux (meld).
            Some(sp) => !self.playable_sides(p, c, sp).is_empty(),
            None => self.can_play_as(p, c, self.obj(c).card),
        }
    }

    /// Les façons de jouer une carte double (CR 5.1.2c) : 0 = côté gauche,
    /// 1 = côté droit, 2 = fusionnée (meld, CR 8.3.38).
    fn playable_sides(&self, p: PlayerId, c: ObjectId, sp: Split) -> Vec<u8> {
        [Some(sp.left), Some(sp.right), sp.melded]
            .into_iter()
            .enumerate()
            .filter_map(|(i, card)| {
                card.filter(|&card| self.can_play_as(p, c, card))
                    .map(|_| i as u8)
            })
            .collect()
    }

    /// La carte `c` peut-elle être jouée avec la définition `card` ?
    fn can_play_as(&self, p: PlayerId, c: ObjectId, card: CardId) -> bool {
        let o = self.obj(c);
        let def = self.db.get(card);
        // CR 5.1.1a : seulement depuis la main ou l'arsenal, par son
        // propriétaire, sauf permission (« you may play it [from your banished
        // zone] this turn », CR 1.0.2 : une permission).
        let zone_ok = matches!(o.zone, Zone::Hand | Zone::Arsenal)
            || (o.zone == Zone::Banished
                && (self.play_permitted(p, c) || self.banished_play_source(p, c).is_some()));
        if o.owner != p || !zone_ok {
            return false;
        }
        let Some(speed) = self.card_speed(card) else {
            return false;
        };
        if !self.timing_ok(p, speed, def.is_attack_card())
            && !(speed == Speed::Action && self.may_play_as_instant(p, c))
        {
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
            card,
            source: Some(self.obj_ref(c)),
            ..Default::default()
        };
        self.some_mode_has_targets(&def.targets, def.modes.len(), ctx)
            && self.play_cost_as(p, c, card) <= self.available_resources(p, Some(c))
    }

    /// Les permanents de `p` qui ont « Once per turn, you may play … from
    /// your banished zone » et ne l'ont pas encore utilisé ce tour.
    fn banished_play_sources(&self, p: PlayerId) -> impl Iterator<Item = ObjectId> + '_ {
        self.permanents.iter().copied().filter(move |&s| {
            let o = self.obj(s);
            o.controller == p
                && o.activations[BANISHED_PLAY_SLOT] == 0
                && self.def(s).play_from_banished.is_some()
                && !self.lost_abilities(s)
        })
    }

    /// Le permanent dont la permission (« Once per turn, you may play … from
    /// your banished zone ») permet à `p` de jouer `c` depuis sa zone de
    /// bannissement (Blasmophet, the Insatiable Hunger).
    pub fn banished_play_source(&self, p: PlayerId, c: ObjectId) -> Option<ObjectId> {
        let o = self.obj(c);
        if o.zone != Zone::Banished || o.face_down || o.owner != p {
            return None;
        }
        self.banished_play_sources(p)
            .find(|&s| (self.def(s).play_from_banished.unwrap())(self, s, c))
    }

    pub(crate) fn can_activate(&self, p: PlayerId, src: ObjectId, i: u8) -> bool {
        let o = self.obj(src);
        let a = &self.def(src).activated[i as usize];
        let zone_ok = if a.from_hand {
            o.zone == Zone::Hand && o.owner == p
        } else {
            o.controller == p && o.zone.in_arena()
        };
        if !zone_ok
            || (a.tap && o.tapped)
            || (a.once_per_turn && o.activations[i as usize] >= self.activation_limit(src, i))
        {
            return false;
        }
        // « Loses all abilities » : plus de capacités activées.
        if self.lost_abilities(src) {
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
            && self.activation_cost(src, i) <= self.available_resources(p, except)
    }

    /// Limite d'activation d'une capacité « Once per Turn » ce tour (CR 5.2.3) :
    /// 1, ou 2 avec « attack twice » (CR 5.2.3e), plus les activations
    /// additionnelles (CR 5.2.3, 5.2.3c). Seules les capacités d'attaque sont
    /// concernées par ces effets.
    pub fn activation_limit(&self, src: ObjectId, i: u8) -> u8 {
        if !self.def(src).activated[i as usize].attack || self.weapon_effects.is_empty() {
            return 1;
        }
        let r = self.obj_ref(src);
        let mut twice = false;
        let mut extra = 0;
        for e in &self.weapon_effects {
            if e.weapon == r {
                match e.kind {
                    WeaponEffectKind::AttackTwice => twice = true,
                    WeaponEffectKind::AdditionalAttack => extra += 1,
                    WeaponEffectKind::CostLess(_) => {}
                }
            }
        }
        1 + u8::from(twice) + extra
    }

    /// Coût en ressources d'une capacité activée (CR 5.1.6), réductions
    /// « your next attack with it costs {r} less » comprises.
    pub fn activation_cost(&self, src: ObjectId, i: u8) -> u32 {
        let a = &self.def(src).activated[i as usize];
        let o = self.obj(src);
        let who = if a.from_hand { o.owner } else { o.controller };
        // CR 5.1.6a : augmentations (Frostbite), puis réductions.
        let mut cost = a.cost as i32 + self.cost_tax(who) as i32;
        if a.attack && !self.weapon_effects.is_empty() {
            let r = self.obj_ref(src);
            for e in &self.weapon_effects {
                if let (true, WeaponEffectKind::CostLess(n)) = (e.weapon == r, e.kind) {
                    cost -= n as i32;
                }
            }
        }
        cost.max(0) as u32
    }

    /// CR 5.1.2 : annonce, puis les étapes suivantes sous forme de tâches.
    pub(crate) fn play_card(&mut self, p: PlayerId, c: ObjectId) {
        if let Some(sp) = self.def(c).split {
            // CR 5.1.2c : le joueur déclare le côté (ou les deux, meld).
            let sides = self.playable_sides(p, c, sp);
            if let [side] = sides[..] {
                self.play_card_side(p, c, side);
            } else {
                self.priority = None;
                self.tasks.push(Task::PlaySide(c));
                self.pending = Some(Decision {
                    player: p,
                    kind: DecisionKind::Mode,
                    options: sides.into_iter().map(Choice::Mode).collect(),
                });
            }
            return;
        }
        self.play_card_now(p, c);
    }

    /// Joue une carte double avec le côté `side` (0 = gauche, 1 = droit,
    /// 2 = fusionnée) : sur la pile, elle n'a que les propriétés de ce côté
    /// (CR 9.2.3), ou des deux (CR 8.3.38c).
    pub(crate) fn play_card_side(&mut self, p: PlayerId, c: ObjectId, side: u8) {
        let sp = self.def(c).split.expect("carte double");
        let card = match side {
            0 => sp.left,
            1 => sp.right,
            _ => sp.melded.expect("meld"),
        };
        self.obj_mut(c).card = card;
        self.play_card_now(p, c);
        if side == 2 {
            // CR 8.3.38a : la carte est fusionnée.
            self.stack.last_mut().unwrap().ctx.melded = true;
        }
    }

    fn play_card_now(&mut self, p: PlayerId, c: ObjectId) {
        let origin = self.obj(c).zone;
        // CR 5.1.3d : depuis la zone de bannissement, le joueur utilise une
        // permission « you may play it this turn » s'il en a une ; sinon la
        // permission « once per turn » d'un permanent (Blasmophet), qui est
        // alors utilisée pour ce tour. (Le joueur n'a jamais intérêt à
        // dépenser la permission « once per turn » à la place de l'autre.)
        if origin == Zone::Banished && !self.play_permitted(p, c) {
            if let Some(s) = self.banished_play_source(p, c) {
                self.obj_mut(s).activations[BANISHED_PLAY_SLOT] = 1;
            }
        }
        // CR 5.1.3d, 8.1.1d : une action jouée « as though it were an
        // instant » ne coûte pas de point d'action. Le joueur n'a jamais
        // intérêt à ne pas utiliser la permission : elle est déclarée d'office
        // (calculé avant que la carte quitte sa zone).
        let as_instant = self.may_play_as_instant(p, c);
        let is_attack = self.def(c).is_attack_card();
        self.place(c, Zone::Stack, p);
        let object = self.obj_ref(c);
        let card = self.obj(c).card;
        let ctx = Box::new(Ctx {
            controller: p,
            card,
            source: Some(object),
            as_instant,
            log_start: self.turn_log.len() as u32,
            ..Default::default()
        });
        self.stack.push(Layer {
            kind: LayerKind::Card,
            object,
            card,
            controller: p,
            target: None,
            origin,
            ctx,
        });
        if is_attack {
            self.apply_next_attack_effects(p, object, true);
        }
        let n_targets = self.def(c).targets.len();
        self.push_play_tasks(n_targets, is_attack);
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
                life_delta: 0,
                life_counters: 0,
            });
            let proxy = ObjectId(self.objects.len() as u32 - 1);
            // Numéro de cette attaque parmi celles de l'arme ce tour.
            self.obj_mut(proxy).counters = self.obj(src).activations[i as usize];
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
        let ctx = Box::new(Ctx {
            controller: p,
            card,
            source: Some(self.obj_ref(src)),
            log_start: self.turn_log.len() as u32,
            ..Default::default()
        });
        self.stack.push(Layer {
            kind: LayerKind::Activated(i),
            object,
            card,
            controller: p,
            target: None,
            origin,
            ctx,
        });
        self.push_play_tasks(a.targets.len(), a.attack);
    }

    fn push_play_tasks(&mut self, n_targets: usize, is_attack: bool) {
        // CR 1.11.5 : personne n'a la priorité pendant qu'on joue.
        self.priority = None;
        self.tasks.push(Task::PlayFinish);
        self.tasks.push(Task::PlayEffectCost);
        self.tasks.push(Task::PlayPay);
        for t in (0..n_targets).rev() {
            self.tasks.push(Task::PlayTarget(t as u8));
        }
        if is_attack {
            // CR 5.1.4b / 1.4.5 : la cible de l'attaque.
            self.tasks.push(Task::PlayAttackTarget);
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
            LayerKind::Granted(i) => self.db.get(l.card).granted[i as usize].targets.clone(),
            LayerKind::Rule(_) => Vec::new(),
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
                    // Coût « XX » (CR 2.1) : chaque unité de X coûte 2.
                    let per_x = if def.x_twice { 2 } else { 1 };
                    let options = (0..=(spare / per_x).min(20) as u8)
                        .map(Choice::Number)
                        .collect();
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
                    // CR 1.7.5, 5.1.4 : seulement les modes dont les cibles existent.
                    let mut ctx = (*self.top_layer().ctx).clone();
                    let specs = def.targets.clone();
                    let options = (0..def.modes.len() as u8)
                        .filter(|&m| {
                            ctx.mode = m;
                            self.targets_available(&specs, &ctx)
                        })
                        .map(Choice::Mode)
                        .collect();
                    self.pending = Some(Decision {
                        player: p,
                        kind: DecisionKind::Mode,
                        options,
                    });
                }
            }
            // CR 1.4.5 : le joueur déclare un objet attaquable d'un adversaire.
            Task::PlayAttackTarget => {
                let options: Vec<Choice> = self
                    .attack_targets(p)
                    .into_iter()
                    .map(Choice::Target)
                    .collect();
                if let [Choice::Target(t)] = options[..] {
                    self.stack.last_mut().unwrap().target = Some(t);
                } else {
                    self.pending = Some(Decision {
                        player: p,
                        kind: DecisionKind::AttackTarget,
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
                if let LayerKind::Activated(i) = kind {
                    if def.activated[i as usize].attack && !self.weapon_effects.is_empty() {
                        // CR 5.1.6 : la réduction « next attack » est utilisée.
                        let src = self.top_layer().ctx.source_id().unwrap();
                        let r = self.obj_ref(src);
                        self.weapon_effects.retain(|e| {
                            !(e.weapon == r && matches!(e.kind, WeaponEffectKind::CostLess(_)))
                        });
                    }
                }
                if kind == LayerKind::Card && !self.cost_reductions.is_empty() {
                    // CR 5.1.6 : les réductions « your next … » sont utilisées.
                    let c = self.top_layer().object.id;
                    let turn = self.turn;
                    let mut v = std::mem::take(&mut self.cost_reductions);
                    v.retain(|x| !(x.player == p && x.turn == turn && (x.filter)(self, c)));
                    self.cost_reductions = v;
                }
                let action_points = match kind {
                    // CR 5.1.6b : une action non jouée comme instant coûte 1 point d'action.
                    LayerKind::Card => {
                        u8::from(def.types.has(Types::ACTION) && !self.top_layer().ctx.as_instant)
                    }
                    LayerKind::Activated(i) => {
                        u8::from(def.activated[i as usize].speed == Speed::Action)
                    }
                    LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => 0,
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
                    LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => return,
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
                let per_x = if def.x_twice { 2 } else { 1 };
                c + l.ctx.x as u32 * per_x
            }
            LayerKind::Activated(i) => match l.ctx.source_id() {
                Some(src) if self.obj(src).card == l.card => self.activation_cost(src, i),
                _ => def.activated[i as usize].cost as u32,
            },
            LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => 0,
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
            LayerKind::Card => {
                // « Your next … this turn » : la carte utilise les permissions
                // d'instant qui la concernent, et les effets « the next card
                // you play » la mémorisent.
                if !self.instant_permissions.is_empty() {
                    let turn = self.turn;
                    let mut v = std::mem::take(&mut self.instant_permissions);
                    v.retain(|x| !(x.player == p && x.turn == turn && (x.filter)(self, object.id)));
                    self.instant_permissions = v;
                }
                if !self.replacements.is_empty() {
                    self.bind_replacements(p, object.id);
                }
                let def = self.obj(object.id).card;
                self.emit(Event::Played {
                    player: p,
                    card: object.id,
                    from: origin,
                    def,
                })
            }
            LayerKind::Activated(i) => {
                let source = l.ctx.source_id().unwrap();
                self.emit(Event::Activated {
                    player: p,
                    source,
                    ability: i,
                })
            }
            LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => {}
        }
        if is_attack {
            // CR 8.3.14 : « When this becomes the target of an attack, destroy this. »
            if let Some(Target::Object(r)) = self.top_layer().target {
                if self.is_current(r) && self.keywords(r.id).has(Keywords::SPECTRA) {
                    self.push_rule_trigger(RuleTrigger::Spectra, r.id);
                }
            }
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
            LayerKind::Trigger(_) | LayerKind::Granted(_) | LayerKind::Rule(_) => false,
        }
    }
}
