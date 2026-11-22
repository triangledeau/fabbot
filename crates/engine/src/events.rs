//! Événements de la partie (CR 1.9) et déclenchements (CR 6.6).
//!
//! Chaque événement est :
//! 1. ajouté au journal du tour (`turn_log`), que les cartes consultent pour
//!    leurs conditions (« si vous avez joué une autre carte rouge ce tour ») ;
//! 2. comparé aux capacités déclenchées des objets concernés.

use crate::card::Types;
use crate::ids::{ObjRef, ObjectId, PlayerId};
use crate::script::{Ctx, ScriptRef};
use crate::state::{GameState, Layer, LayerKind};
use crate::zones::Zone;

/// Un événement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Début du tour du joueur (CR 4.2.2).
    StartOfTurn(PlayerId),
    /// Début de la phase d'action (CR 4.3.1).
    BeginActionPhase(PlayerId),
    /// Début de la phase de fin (CR 4.4.2).
    BeginEndPhase(PlayerId),
    /// Une carte a été jouée (CR 5.1.10).
    Played {
        player: PlayerId,
        card: ObjectId,
        from: Zone,
    },
    /// Une capacité a été activée.
    Activated {
        player: PlayerId,
        source: ObjectId,
        ability: u8,
    },
    /// Une attaque devient attaquante (CR 7.2.3). `card` : la carte ou l'arme.
    Attack {
        attack: ObjectId,
        attacker: PlayerId,
        /// Le joueur qui contrôle la cible (« defending hero »).
        target: PlayerId,
        /// La cible si ce n'est pas un héros (allié, spectra).
        target_obj: Option<ObjectId>,
        link: u8,
    },
    /// Une carte devient une carte qui défend (CR 7.0.5a).
    Defend {
        card: ObjectId,
        defender: PlayerId,
        attack: ObjectId,
        with_hand_card: bool,
        from_hand: bool,
    },
    /// L'attaque touche (CR 7.5.5).
    Hit {
        attack: ObjectId,
        attacker: PlayerId,
        target: PlayerId,
        /// L'allié touché ; `None` = le héros (« when this hits a hero »).
        target_obj: Option<ObjectId>,
        damage: u16,
    },
    /// Un maillon se résout (CR 7.6.2).
    ChainLinkResolved {
        attack: ObjectId,
        attacker: PlayerId,
    },
    /// La chaîne de combat se ferme (CR 7.7.3).
    CombatChainClosed,
    /// Une carte est pitchée (CR 1.14.3c).
    Pitched {
        player: PlayerId,
        card: ObjectId,
    },
    /// Des dégâts sont infligés (CR 8.5.3).
    Damage {
        source: Option<ObjectId>,
        by: PlayerId,
        /// Le héros touché, ou le contrôleur de l'allié touché.
        target: PlayerId,
        /// L'allié touché ; `None` = le héros.
        target_obj: Option<ObjectId>,
        amount: u16,
        arcane: bool,
    },
    /// Un objet est détruit (CR 8.5.4). `card` : l'objet tel qu'il est maintenant.
    Destroyed {
        card: ObjectId,
        owner: PlayerId,
    },
    /// Un objet arrive au cimetière, depuis n'importe où.
    ToGraveyard {
        card: ObjectId,
        owner: PlayerId,
        from: Zone,
    },
    /// Un objet est banni (CR 8.5.1) ou mis dans la zone de bannissement.
    Banished {
        card: ObjectId,
        owner: PlayerId,
        from: Zone,
        power: u16,
    },
    /// Un objet entre dans l'arène / la quitte.
    EnterArena {
        card: ObjectId,
    },
    LeaveArena {
        card: ObjectId,
        owner: PlayerId,
    },
    /// Un joueur pioche.
    Drew {
        player: PlayerId,
        card: ObjectId,
    },
    /// Un objet est aiguisé (sharpen, CR 8.5.58) par ce joueur.
    Sharpened {
        player: PlayerId,
        card: ObjectId,
    },
    /// Un joueur gagne des PV.
    GainedLife {
        player: PlayerId,
        amount: u16,
    },
}

impl GameState {
    /// Signale un événement : journal du tour + déclenchements.
    pub fn emit(&mut self, ev: Event) {
        self.turn_log.push(ev);
        if let Some(r) = &mut self.record {
            r.push(ev);
        }
        let any_lost = self.any_lose_abilities;
        // Les objets dont une capacité peut se déclencher.
        for k in 0..self.trigger_objects.len() {
            let id = self.trigger_objects[k];
            let o = &self.objects[id.0 as usize];
            if o.zone == Zone::Gone {
                continue;
            }
            let def = self.db.get(o.card);
            if def.triggers.is_empty() {
                continue;
            }
            // CR 1.7.4 : fonctionne dans l'arène (et sur la pile pour les attaques),
            // ou partout si la capacité le précise. Une proxy n'a pas les
            // capacités de son arme (seule l'arme en a).
            let in_play = o.zone.in_arena() || o.zone == Zone::Stack;
            if o.proxy_of.is_some() || o.face_down && !in_play {
                continue;
            }
            // « Loses all abilities » : plus de capacités déclenchées.
            if any_lost && self.lost_abilities(id) {
                continue;
            }
            // CR 1.7.4a : une carte qui défend (non permanente) n'a pas ses
            // capacités, sauf celles qui parlent de défendre.
            for (t, trig) in def.triggers.iter().enumerate() {
                if !(in_play || trig.anywhere) {
                    continue;
                }
                if (trig.when)(self, id, &ev) {
                    let controller = if in_play { o.controller } else { o.owner };
                    let card = o.card;
                    let source = ObjRef { id, gen: o.gen };
                    self.pending_triggers.push(Layer {
                        kind: LayerKind::Trigger(t as u8),
                        object: source,
                        card,
                        controller,
                        target: None,
                        origin: Zone::Stack,
                        ctx: Box::new(Ctx {
                            controller,
                            card,
                            source: Some(source),
                            event: Some(ev),
                            ..Default::default()
                        }),
                    });
                }
            }
        }
    }

    /// Événements de ce tour (CR : « this turn »).
    pub fn turn_events(&self) -> &[Event] {
        &self.turn_log
    }

    /// A-t-on joué ce tour une carte qui vérifie `f` (hors `except`) ?
    pub fn played_this_turn(
        &self,
        p: PlayerId,
        except: Option<ObjectId>,
        f: impl Fn(&GameState, ObjectId) -> bool,
    ) -> bool {
        self.turn_log.iter().any(|e| match *e {
            Event::Played { player, card, .. } => {
                player == p && Some(card) != except && f(self, card)
            }
            _ => false,
        })
    }

    /// Nombre d'attaques de ce joueur ce tour.
    pub fn attacks_this_turn(&self, p: PlayerId) -> usize {
        self.turn_log
            .iter()
            .filter(|e| matches!(e, Event::Attack { attacker, .. } if *attacker == p))
            .count()
    }

    /// Le joueur a-t-il reçu des dégâts arcaniques ce tour ?
    pub fn arcane_damage_taken_this_turn(&self, p: PlayerId) -> u32 {
        self.turn_log
            .iter()
            .map(|e| match *e {
                Event::Damage {
                    target,
                    target_obj: None,
                    amount,
                    arcane: true,
                    ..
                } if target == p => amount as u32,
                _ => 0,
            })
            .sum()
    }

    /// Une carte de puissance 6 ou plus a-t-elle été mise dans la zone de
    /// bannissement de ce joueur ce tour ?
    pub fn banished_power6_this_turn(&self, p: PlayerId) -> bool {
        self.turn_log.iter().any(
            |e| matches!(e, Event::Banished { owner, power, .. } if *owner == p && *power >= 6),
        )
    }

    /// Cet objet a-t-il été aiguisé ce tour (CR 8.5.58a) ?
    pub fn sharpened_this_turn(&self, id: ObjectId) -> bool {
        self.turn_log
            .iter()
            .any(|e| matches!(e, Event::Sharpened { card, .. } if *card == id))
    }

    /// Le joueur a-t-il aiguisé ce tour un objet qui vérifie `f` (CR 8.5.58a) ?
    pub fn player_sharpened_this_turn(
        &self,
        p: PlayerId,
        f: impl Fn(&GameState, ObjectId) -> bool,
    ) -> bool {
        self.turn_log.iter().any(|e| match *e {
            Event::Sharpened { player, card } => player == p && f(self, card),
            _ => false,
        })
    }

    /// Une attaque d'arme : attaque-proxy dont la source est une arme (CR 1.4.3).
    pub fn is_weapon_attack(&self, attack: ObjectId) -> bool {
        self.obj(attack)
            .proxy_of
            .is_some_and(|w| self.def(w).types.has(Types::WEAPON))
    }

    pub(crate) fn script_for(&self, layer: &Layer) -> Option<ScriptRef> {
        match layer.kind {
            LayerKind::Card => Some(ScriptRef::Resolve(layer.card)),
            LayerKind::Activated(i) => Some(ScriptRef::Activated(layer.card, i)),
            LayerKind::Trigger(i) => Some(ScriptRef::Trigger(layer.card, i)),
            LayerKind::Rule(_) => None,
        }
    }

    /// Le type de carte d'un objet.
    pub fn is_type(&self, id: ObjectId, t: Types) -> bool {
        self.def(id).types.has(t)
    }
}
