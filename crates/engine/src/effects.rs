//! Primitives d'effet utilisées par les capacités des cartes (CR 8.5).
//!
//! Ces fonctions sont publiques : le code des cartes (`fab-cards`) les appelle
//! depuis ses instructions `Op::Do`.

use crate::card::{Delta, Types};
use crate::events::Event;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::state::{GameState, Modifier, NextAttackEffect, Object, Prevention, Until};
use crate::zones::Zone;

impl GameState {
    /// Pioche une carte (CR 8.5.6). Échoue si le deck est vide (CR 8.5.6b).
    pub fn draw(&mut self, p: PlayerId) -> bool {
        match self.top_of_deck(p) {
            Some(id) => {
                self.move_to(id, Zone::Hand, true);
                self.emit(Event::Drew {
                    player: p,
                    card: id,
                });
                true
            }
            None => false,
        }
    }

    /// Gagner des PV (CR 8.5.7a).
    pub fn gain_life(&mut self, p: PlayerId, n: u16) {
        if n == 0 {
            return;
        }
        self.player_mut(p).life += n as i32;
        self.emit(Event::GainedLife {
            player: p,
            amount: n,
        });
    }

    /// Perdre des PV sans être blessé (CR 8.5.12, 8.5.3d).
    pub fn lose_life(&mut self, p: PlayerId, n: u16) {
        let ps = self.player_mut(p);
        ps.life = (ps.life - n as i32).max(0);
    }

    /// Gagner des points de ressource (CR 8.5.7).
    pub fn gain_resources(&mut self, p: PlayerId, n: u8) {
        self.player_mut(p).resource_points += n;
    }

    /// Gagner des points d'action (CR 1.13.2b : seulement le joueur actif).
    pub fn gain_action_points(&mut self, p: PlayerId, n: u8) {
        if p == self.turn_player {
            self.player_mut(p).action_points += n;
        }
    }

    /// Inflige des dégâts à un héros (CR 8.5.3) et renvoie les dégâts réellement
    /// infligés après prévention (CR 6.4). Le total de vie n'est jamais négatif (CR 2.5.3e).
    ///
    /// TODO(étape 2) : préventions au choix du joueur (arcane barrier, ward,
    /// spellvoid) ; dégâts aux alliés.
    pub fn deal_damage(
        &mut self,
        source: Option<ObjectId>,
        by: PlayerId,
        target: PlayerId,
        amount: i32,
        arcane: bool,
    ) -> i32 {
        if amount <= 0 {
            return 0; // CR 1.9.1b : 0 dégât = rien ne se passe
        }
        let mut amount = amount;
        let src_ref = source.map(|s| self.obj_ref(s));
        for prev in self.preventions.iter_mut() {
            if amount == 0 {
                break;
            }
            if prev.player != target || prev.remaining <= 0 || (prev.arcane_only && !arcane) {
                continue;
            }
            if prev.source.is_some() && prev.source != src_ref {
                continue;
            }
            let n = prev.remaining.min(amount);
            prev.remaining -= n;
            amount -= n;
        }
        self.preventions.retain(|p| p.remaining > 0);
        if amount <= 0 {
            return 0;
        }
        let ps = self.player_mut(target);
        ps.life = (ps.life - amount).max(0);
        self.emit(Event::Damage {
            source,
            by,
            target,
            amount: amount as u16,
            arcane,
        });
        amount
    }

    /// Détruire (CR 8.5.4) : au cimetière du propriétaire.
    pub fn destroy(&mut self, id: ObjectId) {
        if self.obj(id).zone == Zone::Gone {
            return;
        }
        let owner = self.obj(id).owner;
        self.clear(id);
        self.emit(Event::Destroyed { card: id, owner });
    }

    /// Bannir (CR 8.5.1). `face_down` : banni face cachée.
    pub fn banish(&mut self, id: ObjectId, face_down: bool) {
        let owner = self.obj(id).owner;
        // CR 3.0.7a : la puissance est celle de l'objet au moment où il quitte sa zone.
        let power = if self.def(id).power.is_some() && !face_down {
            self.power(id) as u16
        } else {
            0
        };
        let from = self.move_to(id, Zone::Banished, true);
        if self.obj(id).zone == Zone::Banished {
            self.obj_mut(id).face_down = face_down;
            self.emit(Event::Banished {
                card: id,
                owner,
                from,
                power,
            });
        }
    }

    /// Défausser (CR 8.5.5).
    pub fn discard(&mut self, id: ObjectId) {
        debug_assert_eq!(self.obj(id).zone, Zone::Hand);
        self.put_in_graveyard(id);
    }

    /// Défausse une carte au hasard ; renvoie la carte défaussée.
    pub fn discard_random(&mut self, p: PlayerId) -> Option<ObjectId> {
        let n = self.player(p).hand.len();
        if n == 0 {
            return None; // CR 8.5.5b
        }
        let i = self.rng.below(n as u32) as usize;
        let id = self.player(p).hand[i];
        self.discard(id);
        Some(id)
    }

    /// Met une carte sous le deck de son propriétaire.
    pub fn put_on_bottom(&mut self, id: ObjectId) {
        self.move_to(id, Zone::Deck, false);
    }

    /// Met une carte sur le deck de son propriétaire.
    pub fn put_on_top(&mut self, id: ObjectId) {
        self.move_to(id, Zone::Deck, true);
    }

    /// Mélange le deck (CR 8.5.20).
    pub fn shuffle_deck(&mut self, p: PlayerId) {
        let mut deck = std::mem::take(&mut self.player_mut(p).deck);
        self.rng.shuffle(&mut deck);
        self.player_mut(p).deck = deck;
    }

    /// Crée un token (CR 8.5.2) dans la zone permanente, sous le contrôle de `p`.
    pub fn create_token(&mut self, p: PlayerId, card: CardId) -> ObjectId {
        debug_assert!(self.db.get(card).types.has(Types::TOKEN));
        self.objects.push(Object {
            card,
            owner: p,
            controller: p,
            zone: Zone::Gone,
            gen: 0,
            face_down: false,
            tapped: false,
            defense_counters: 0,
            power_counters: 0,
            counters: 0,
            proxy_of: None,
            activations: [0; 4],
        });
        let id = ObjectId(self.objects.len() as u32 - 1);
        if !self.db.get(card).triggers.is_empty() {
            self.trigger_objects.push(id);
        }
        self.obj_mut(id).zone = Zone::Permanent;
        self.permanents.push(id);
        self.emit(Event::EnterArena { card: id });
        id
    }

    /// Un objet « gets » +N / go again… (CR 8.5.8, 8.5.9).
    pub fn modify(&mut self, target: ObjRef, delta: Delta, until: Until) {
        if self.is_current(target) {
            self.modifiers.push(Modifier {
                target,
                delta,
                until,
            });
        }
    }

    /// « Votre prochaine attaque [filtre] ce tour gets … » (CR 1.8.10).
    pub fn next_attack(
        &mut self,
        p: PlayerId,
        filter: Option<fn(&GameState, ObjectId) -> bool>,
        delta: Delta,
        cards_only: bool,
    ) {
        self.next_attack_effects.push(NextAttackEffect {
            player: p,
            filter,
            delta,
            cards_only,
        });
    }

    /// « Prevent the next N damage that would be dealt to … » (CR 6.4).
    pub fn prevent_next(&mut self, p: PlayerId, n: i32, arcane_only: bool, source: Option<ObjRef>) {
        self.preventions.push(Prevention {
            player: p,
            remaining: n,
            arcane_only,
            source,
            until: Until::EndOfTurn,
        });
    }

    /// Marquer un héros (CR 8.5.50).
    pub fn mark(&mut self, p: PlayerId) {
        self.player_mut(p).marked = true;
    }

    /// Applique les effets « prochaine attaque » à une attaque annoncée (CR 5.1.2a).
    pub(crate) fn apply_next_attack_effects(&mut self, p: PlayerId, attack: ObjRef, is_card: bool) {
        let mut i = 0;
        while i < self.next_attack_effects.len() {
            let e = self.next_attack_effects[i];
            let ok = e.player == p
                && (is_card || !e.cards_only)
                && e.filter.is_none_or(|f| f(self, attack.id));
            if ok {
                self.next_attack_effects.remove(i);
                self.modifiers.push(Modifier {
                    target: attack,
                    delta: e.delta,
                    until: Until::EndOfTurn,
                });
            } else {
                i += 1;
            }
        }
    }
}
