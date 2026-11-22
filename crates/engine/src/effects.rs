//! Effets des cartes : interprétation des instructions ([`Op`]), dégâts,
//! modificateurs et déclenchements.

use crate::card::{Keywords, Op};
use crate::ids::{ObjRef, PlayerId};
use crate::state::{GameState, Layer, LayerKind, Modifier, NextAttackEffect, Until};
use crate::zones::Zone;

impl GameState {
    /// Génère les effets d'une liste d'instructions, dans l'ordre (CR 5.3.4, 6.1.2).
    pub(crate) fn run_ops(&mut self, ops: &[Op], controller: PlayerId, source: ObjRef) {
        let _ = source;
        for op in ops {
            match *op {
                Op::Draw(n) => {
                    for _ in 0..n {
                        self.draw(controller);
                    }
                }
                Op::GainLife(n) => self.player_mut(controller).life += n as i32,
                Op::DamageOpposingHero(n) => self.deal_damage(controller.opponent(), n as i32),
                Op::ActiveAttackGetsPower(n) => {
                    if let Some(link) = self.combat.links.last() {
                        let target = link.attack;
                        if self.is_current(target) {
                            self.modifiers.push(Modifier {
                                target,
                                power: n as i16,
                                defense: 0,
                                grants: Keywords::NONE,
                                until: Until::EndOfTurn,
                            });
                        }
                    }
                }
                Op::NextAttackGetsPower(n) => self.next_attack_effects.push(NextAttackEffect {
                    player: controller,
                    power: n as i16,
                }),
            }
        }
    }

    /// Inflige des dégâts à un héros (CR 8.5.3). Le total de vie n'est jamais négatif (CR 2.5.3e).
    pub(crate) fn deal_damage(&mut self, target: PlayerId, amount: i32) {
        if amount <= 0 {
            return; // CR 1.9.1b : 0 dégât = rien ne se passe
        }
        let ps = self.player_mut(target);
        ps.life = (ps.life - amount).max(0);
    }

    /// Applique les effets « votre prochaine attaque ce tour » (CR 1.8.10, 5.1.2a).
    pub(crate) fn apply_next_attack_effects(&mut self, p: PlayerId, attack: ObjRef) {
        let mut i = 0;
        while i < self.next_attack_effects.len() {
            if self.next_attack_effects[i].player == p {
                let e = self.next_attack_effects.remove(i);
                self.modifiers.push(Modifier {
                    target: attack,
                    power: e.power,
                    defense: 0,
                    grants: Keywords::NONE,
                    until: Until::EndOfTurn,
                });
            } else {
                i += 1;
            }
        }
    }

    /// Crée une couche déclenchée, ajoutée à la pile avant la prochaine priorité (CR 6.6.6).
    pub(crate) fn queue_trigger(
        &mut self,
        kind: LayerKind,
        source: ObjRef,
        controller: PlayerId,
        target: Option<PlayerId>,
    ) {
        let card = self.obj(source.id).card;
        self.pending_triggers.push(Layer {
            kind,
            object: source,
            card,
            controller,
            target,
            origin: Zone::CombatChain,
        });
    }
}
