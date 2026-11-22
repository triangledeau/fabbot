//! Jouer une carte (CR 5.1), activer une arme (CR 5.2) et payer les coûts (CR 1.14).

use crate::card::Types;
use crate::decision::{Choice, Decision, DecisionKind};
use crate::ids::{ObjectId, PlayerId};
use crate::state::{CombatStep, GameState, Layer, LayerKind, Object, Payment, Stage};
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
        for &w in &ps.weapons {
            if self.can_activate_weapon(p, w) {
                opts.push(Choice::Activate(w));
            }
        }
        opts.push(Choice::Pass);
        opts
    }

    /// Ressources que le joueur peut réunir sans utiliser `except` (CR 1.14.2d, 1.14.3).
    fn available_resources(&self, p: PlayerId, except: Option<ObjectId>) -> u32 {
        let ps = self.player(p);
        let pitchable: u32 = ps
            .hand
            .iter()
            .filter(|&&c| Some(c) != except)
            .map(|&c| self.def(c).pitch.unwrap_or(0) as u32)
            .sum();
        ps.resource_points as u32 + pitchable
    }

    /// Une action (carte ou capacité de type action) peut-elle être jouée maintenant ?
    /// CR 8.1.1a-c et 7.0.1a.
    fn action_timing_ok(&self, p: PlayerId, is_attack: bool) -> bool {
        if p != self.turn_player || self.player(p).action_points < 1 || !self.stack.is_empty() {
            return false;
        }
        if !self.combat.open {
            return true;
        }
        // CR 7.0.1a : pendant le combat, seules les attaques, à l'étape de résolution.
        // TODO(règles) : CR 8.1.1b semble autoriser toute action à l'étape de
        // résolution, CR 7.0.1a seulement les attaques. On applique 7.0.1a (le
        // plus restrictif). À confirmer.
        is_attack && self.combat.step == Some(CombatStep::Resolution)
    }

    pub(crate) fn can_play(&self, p: PlayerId, c: ObjectId) -> bool {
        let o = self.obj(c);
        let def = self.def(c);
        // CR 5.1.1a : seulement depuis la main ou l'arsenal, par son propriétaire.
        if o.owner != p || !matches!(o.zone, Zone::Hand | Zone::Arsenal) {
            return false;
        }
        let timing_ok = if def.types.has(Types::INSTANT) {
            true // CR 8.1.6a
        } else if def.types.has(Types::ACTION) {
            self.action_timing_ok(p, def.is_attack_card())
        } else if def.types.has(Types::ATTACK_REACTION) {
            // CR 8.1.2a
            self.combat.step == Some(CombatStep::Reaction)
                && self.combat.active_link().is_some_and(|l| l.attacker == p)
        } else if def.types.has(Types::DEFENSE_REACTION) {
            // CR 8.1.3a et 7.4.2c
            self.combat.step == Some(CombatStep::Reaction)
                && self.combat.active_link().is_some_and(|l| l.target == p)
                && self.can_become_defending(c, o.zone)
        } else {
            false // ressources, blocs (CR 8.1.7a, 8.1.12a) et autres
        };
        timing_ok && def.cost.unwrap_or(0) as u32 <= self.available_resources(p, Some(c))
    }

    pub(crate) fn can_activate_weapon(&self, p: PlayerId, w: ObjectId) -> bool {
        let o = self.obj(w);
        let Some(wa) = self.def(w).weapon_attack else {
            return false;
        };
        o.controller == p
            && matches!(o.zone, Zone::Weapon | Zone::CombatChain)
            && !(wa.once_per_turn && o.activations_this_turn >= 1)
            && self.action_timing_ok(p, true)
            && wa.cost as u32 <= self.available_resources(p, None)
    }

    /// CR 5.1.2 à 5.1.10.
    pub(crate) fn play_card(&mut self, p: PlayerId, c: ObjectId) {
        let origin = self.obj(c).zone;
        let def = self.def(c);
        let is_attack = def.is_attack_card();
        let cost = def.cost.unwrap_or(0);
        // CR 5.1.6b : une action non jouée comme instant coûte 1 point d'action.
        let ap = if def.types.has(Types::ACTION) { 1 } else { 0 };
        // Annonce (CR 5.1.2) : la carte va sur la pile.
        self.place(c, Zone::Stack, p);
        let object = self.obj_ref(c);
        let card = self.obj(c).card;
        // Cibles (CR 5.1.4b) : en partie à 2 sans allié, l'attaque vise le héros adverse.
        let target = if is_attack { Some(p.opponent()) } else { None };
        self.stack.push(Layer {
            kind: LayerKind::Card,
            object,
            card,
            controller: p,
            target,
            origin,
        });
        if is_attack {
            self.apply_next_attack_effects(p, object);
        }
        self.start_payment(p, cost, ap);
    }

    /// Activation de la capacité d'attaque d'une arme (CR 5.2, 1.4.3).
    pub(crate) fn activate_weapon(&mut self, p: PlayerId, w: ObjectId) {
        let wa = self.def(w).weapon_attack.expect("arme");
        let card = self.obj(w).card;
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
            proxy_of: Some(w),
            activations_this_turn: 0,
        });
        let proxy = ObjectId(self.objects.len() as u32 - 1);
        self.obj_mut(w).activations_this_turn += 1;
        let object = self.obj_ref(proxy);
        self.stack.push(Layer {
            kind: LayerKind::WeaponAttack,
            object,
            card,
            controller: p,
            target: Some(p.opponent()),
            origin: Zone::Weapon,
        });
        self.apply_next_attack_effects(p, object);
        // CR 8.1.1c : capacité de type action, 1 point d'action.
        self.start_payment(p, wa.cost, 1);
    }

    fn start_payment(&mut self, p: PlayerId, resources: u8, action_points: u8) {
        // CR 1.11.5 : personne n'a la priorité pendant qu'on joue une carte.
        self.priority = None;
        self.payment = Some(Payment {
            player: p,
            resources,
            action_points,
        });
        self.stage = Stage::Paying;
    }

    /// Demande la prochaine carte à pitcher, ou termine le paiement (CR 1.14.2d).
    pub(crate) fn ask_payment(&mut self) {
        let pay = self.payment.expect("paiement");
        let p = pay.player;
        if self.player(p).resource_points >= pay.resources {
            let ps = self.player_mut(p);
            ps.resource_points -= pay.resources;
            ps.action_points -= pay.action_points; // CR 1.14.2a : ressources puis points d'action
            self.payment = None;
            self.finish_play(p);
            return;
        }
        // CR 1.14.3a-b : seules les cartes avec une valeur de pitch.
        let options: Vec<Choice> = self
            .player(p)
            .hand
            .iter()
            .filter(|&&c| self.def(c).pitch.unwrap_or(0) > 0)
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
        let value = self.def(c).pitch.unwrap_or(0);
        self.move_to(c, Zone::Pitch, true);
        self.player_mut(p).resource_points += value;
    }

    /// CR 5.1.10 : la carte est jouée, le joueur récupère la priorité.
    fn finish_play(&mut self, p: PlayerId) {
        let top = *self.stack.last().expect("couche jouée");
        let is_attack = top.kind == LayerKind::WeaponAttack
            || (top.kind == LayerKind::Card && self.def(top.object.id).is_attack_card());
        if is_attack {
            if !self.combat.open {
                // CR 7.0.2a / 8.2.3b : la chaîne de combat s'ouvre.
                self.combat.open = true;
                self.combat.step = Some(CombatStep::Layer);
            } else if self.combat.step == Some(CombatStep::Resolution) {
                // CR 7.6.3a
                self.combat.step = Some(CombatStep::Layer);
            }
        }
        self.passes = 0;
        self.give_priority(p);
    }
}
