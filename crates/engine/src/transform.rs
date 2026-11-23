//! Cartes sous un permanent, transform, cartes retournables, alliés
//! (étape 3, deck Dromai).
//!
//! - **Sub-cards** (CR 3.0.14) : une carte mise sous un permanent n'est plus
//!   dans l'arène (zone [`Zone::Under`]) ; elle devient un nouvel objet. Si
//!   le permanent du dessus cesse d'exister, ses sub-cards sont « clear »
//!   (CR 3.0.14e). `GameState::sub_cards` note les paires (dessous, dessus).
//! - **Material** (CR 8.4.5) : « While this is under a permanent, that
//!   permanent gets … » (`CardDef::material`, ex. Ash : phantasm).
//! - **Transform** (CR 8.5.36) : mettre des objets sous un permanent ; si le
//!   permanent est un token, il est d'abord créé (CR 8.5.36c) ; si c'est une
//!   carte qui n'est pas encore un permanent, elle le devient d'abord
//!   (CR 8.5.36b, ex. une invocation qui se résout).
//! - **Cartes retournables** (CR 9.1.3) : une invocation entre dans l'arène
//!   face arrière active (CR 8.2.13a) ; hors de l'arène, la face avant est
//!   active (CR 9.1.3b).
//! - **Contrôle** : steal (CR 8.5.54), gain control (CR 8.5.35).

use crate::card::{Keywords, Subtypes, Types};
use crate::events::Event;
use crate::ids::{CardId, ObjectId, PlayerId};
use crate::state::GameState;
use crate::zones::Zone;

impl GameState {
    /// Change la définition active d'un objet (face d'une carte retournable)
    /// en tenant à jour les listes d'objets à capacités déclenchées et à
    /// remplacements. Public pour les bots, qui imaginent les cartes cachées
    /// (`fab_ai::determinisation`).
    pub fn set_card(&mut self, id: ObjectId, card: CardId) {
        self.obj_mut(id).card = card;
        let def = self.db.get(card);
        if !def.triggers.is_empty() && !self.trigger_objects.contains(&id) {
            self.trigger_objects.push(id);
        }
        if !def.replacements.is_empty() && !self.replacement_objects.contains(&id) {
            self.replacement_objects.push(id);
        }
    }

    /// CR 9.1.3b : hors de l'arène et de la pile, une carte retournable a sa
    /// face avant active.
    pub(crate) fn restore_front_face(&mut self, id: ObjectId) {
        if let Some(f) = self.def(id).flip {
            if self.obj(id).card != f.front {
                self.set_card(id, f.front);
            }
        }
    }

    /// Les cartes sous le permanent `top` (CR 3.0.14).
    pub fn cards_under(&self, top: ObjectId) -> impl Iterator<Item = ObjectId> + '_ {
        self.sub_cards
            .iter()
            .filter(move |&&(_, t)| t == top)
            .map(|&(s, _)| s)
    }

    /// Mots-clés que les cartes sous `top` lui donnent (« Material », CR 8.4.5).
    #[inline]
    pub fn material_keywords(&self, top: ObjectId) -> Keywords {
        if self.sub_cards.is_empty() {
            return Keywords::NONE;
        }
        let mut k = Keywords::NONE;
        for &(s, t) in &self.sub_cards {
            if t == top {
                k = k | self.def(s).material;
            }
        }
        k
    }

    /// Met `obj` sous le permanent `top` (CR 8.5.36, 3.0.14) : il quitte sa
    /// zone et devient un nouvel objet (CR 3.0.14b). Ses propres sub-cards
    /// passent sous `top` (CR 3.0.14c).
    pub fn put_under(&mut self, obj: ObjectId, top: ObjectId) {
        if obj == top || self.obj(obj).zone == Zone::Gone || !self.obj(top).zone.in_arena() {
            return;
        }
        let was_in_arena = self.obj(obj).zone.in_arena();
        let owner = self.obj(obj).owner;
        self.detach(obj);
        self.unequip(obj);
        {
            let o = self.obj_mut(obj);
            o.zone = Zone::Under;
            o.gen += 1;
            o.face_down = false;
            o.tapped = false;
            o.defense_counters = 0;
            o.power_counters = 0;
            o.counters = 0;
            o.activations = [0; 4];
            o.life_delta = 0;
            o.life_counters = 0;
        }
        for e in &mut self.sub_cards {
            if e.1 == obj {
                e.1 = top;
            }
        }
        self.sub_cards.push((obj, top));
        if was_in_arena {
            self.emit(Event::LeaveArena { card: obj, owner });
        }
    }

    /// CR 3.0.14e : le permanent `top` a cessé d'exister (ou est devenu un
    /// nouvel objet) : ses sub-cards sont « clear ».
    pub(crate) fn clear_sub_cards(&mut self, top: ObjectId) {
        if self.sub_cards.is_empty() {
            return;
        }
        let subs: Vec<ObjectId> = self.cards_under(top).collect();
        if subs.is_empty() {
            return;
        }
        self.sub_cards.retain(|&(_, t)| t != top);
        for s in subs {
            self.clear(s);
        }
    }

    /// Transform (CR 8.5.36c) : crée le token `card` sous le contrôle de `p`,
    /// puis met `objs` sous lui. Renvoie le token. Échoue (CR 8.5.36d) si un
    /// des objets n'existe plus ou si la création est remplacée.
    pub fn transform_into_token(
        &mut self,
        p: PlayerId,
        objs: &[ObjectId],
        card: CardId,
    ) -> Option<ObjectId> {
        if objs.iter().any(|&o| !self.obj(o).zone.in_arena()) {
            return None;
        }
        self.create_tokens(p, card, 1);
        let token = self.last_created.first().copied()?;
        for &o in objs {
            self.put_under(o, token);
        }
        Some(token)
    }

    /// Transform (CR 8.5.36b) : la carte `card_obj` (ex. une invocation qui
    /// se résout) devient d'abord un permanent sous le contrôle de `p`, puis
    /// `objs` sont mis sous elle.
    pub fn transform_into_card(&mut self, p: PlayerId, objs: &[ObjectId], card_obj: ObjectId) {
        if objs.iter().any(|&o| !self.obj(o).zone.in_arena()) {
            return;
        }
        if !self.obj(card_obj).zone.in_arena() {
            self.become_permanent(card_obj, p);
        }
        for &o in objs {
            self.put_under(o, card_obj);
        }
    }

    /// Une carte de la pile devient un permanent (CR 5.3.6a) ; une
    /// invocation entre dans l'arène face arrière active (CR 8.2.13a).
    pub(crate) fn become_permanent(&mut self, id: ObjectId, controller: PlayerId) {
        if let Some(f) = self.def(id).flip {
            if self.db.get(f.front).subtypes.has(Subtypes::INVOCATION) {
                self.set_card(id, f.back);
            }
        }
        // Le contrôleur est fixé avant l'entrée (déclenchements « when this
        // enters the arena »).
        self.obj_mut(id).controller = controller;
        self.move_to(id, Zone::Permanent, true);
        self.obj_mut(id).controller = controller;
    }

    /// Steal (CR 8.5.54) « until the end of this action phase » : `p` gagne
    /// le contrôle de l'objet jusqu'à la fin de la phase d'action.
    pub fn steal_until_end_of_action_phase(&mut self, p: PlayerId, id: ObjectId) {
        if !self.obj(id).zone.in_arena() || self.obj(id).controller == p {
            return;
        }
        let prev = self.obj(id).controller;
        let r = self.obj_ref(id);
        self.steals.push((r, prev));
        self.obj_mut(id).controller = p;
    }

    /// Gain control (CR 8.5.35), sans durée (ex. Kyloria : un objet).
    pub fn gain_control(&mut self, p: PlayerId, id: ObjectId) {
        if self.obj(id).zone.in_arena() {
            self.steals.retain(|(r, _)| r.id != id);
            self.obj_mut(id).controller = p;
        }
    }

    /// Fin de la phase d'action : les objets volés reviennent à leur
    /// contrôleur précédent.
    pub(crate) fn end_steals(&mut self) {
        for (r, prev) in std::mem::take(&mut self.steals) {
            if self.is_current(r) && self.obj(r.id).zone.in_arena() {
                self.obj_mut(r.id).controller = prev;
            }
        }
    }

    /// L'allié a-t-il reçu la capacité « Once per Turn Action -- 0: Attack »
    /// d'une capacité statique d'un objet de son contrôleur (Storm of
    /// Sandikai : « Dragon allies you control get … ») ?
    pub fn attack_granted(&self, ally: ObjectId) -> bool {
        let p = self.obj(ally).controller;
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
                    .grants_attack
                    .is_some_and(|f| !self.lost_abilities(src) && f(self, src, ally))
            })
    }

    /// Themai : « Opponents can't play cards or activate abilities during
    /// your turn. » Le joueur `p` est-il concerné ?
    pub fn silenced(&self, p: PlayerId) -> bool {
        let opp = p.opponent();
        self.turn_player == opp
            && self
                .permanents
                .iter()
                .copied()
                .chain(self.attacking_allies())
                .any(|o| {
                    self.obj(o).controller == opp
                        && self.def(o).silences_opponents
                        && !self.lost_abilities(o)
                })
    }

    /// Compteurs +1{h} / -1{h} sur un allié (ex. Nekria).
    pub fn add_life_counters(&mut self, id: ObjectId, n: i8) {
        if self.obj(id).zone.in_arena() && self.def(id).life.is_some() {
            let o = self.obj_mut(id);
            o.life_counters = o.life_counters.saturating_add(n);
            self.check_allies = true;
        }
    }

    /// CR 4.4.3a : la vie de chaque allié revient à sa vie de base, modifiée
    /// par ses compteurs.
    pub(crate) fn reset_ally_life(&mut self) {
        for k in 0..self.permanents.len() {
            let id = self.permanents[k];
            if self.def(id).life.is_some() {
                self.obj_mut(id).life_delta = 0;
            }
        }
    }

    /// Les alliés attaquants (sources d'une attaque-proxy sur la chaîne).
    pub fn attacking_allies(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.combat.links.iter().filter_map(move |l| {
            l.source.filter(|&s| {
                self.obj(s).zone == Zone::CombatChain
                    && self.def(s).subtypes.has(Subtypes::ALLY)
                    && !self.def(s).types.has(Types::WEAPON)
            })
        })
    }
}
