//! Mécanismes du deck Arakni (Assassin, Chaos) : stealth, dagues qui
//! « touchent » par un effet, équiper un token arme, retrieve, become /
//! return to the brood, effets sur toutes les attaques d'un tour.

use crate::card::{CardDef, Delta, Keywords, Subtypes, Traits, Types};
use crate::effects::DamageEvent;
use crate::events::Event;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::script::Ctx;
use crate::script::Target;
use crate::state::{AttackBuff, GameState, Layer, LayerKind, NextAttackEffect, PlayPermission};
use crate::zones::Zone;

/// Nombre de mains qu'occupe une arme ou un off-hand (CR 8.2.2b).
fn hands_of(def: &CardDef) -> u8 {
    if def.subtypes.has(Subtypes::TWO_HAND) {
        2
    } else {
        1
    }
}

impl GameState {
    /// L'objet a-t-il stealth (CR 8.3.24) ? Lecture de la valeur **imprimée**
    /// (pour une attaque-proxy : celle de son arme), sauf « loses all
    /// abilities ». Aucun effet de nos decks n'accorde ou ne retire stealth ;
    /// cette lecture peut donc servir pendant le calcul des valeurs modifiées
    /// (capacité statique d'Arakni, Night's Embrace) sans boucle.
    pub fn has_stealth(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        let src = o.proxy_of.unwrap_or(id);
        !self.lost_abilities(src) && self.def(src).keywords.has(Keywords::STEALTH)
    }

    /// L'objet est-il une attaque (CR 1.4.1) : une couche d'attaque sur la
    /// pile, ou l'attaque d'un maillon de la chaîne de combat ?
    pub fn is_attack(&self, id: ObjectId) -> bool {
        match self.obj(id).zone {
            Zone::Stack => self
                .stack
                .iter()
                .any(|l| l.object.id == id && self.layer_is_attack(l)),
            Zone::CombatChain => self.combat.links.iter().any(|l| l.attack.id == id),
            _ => false,
        }
    }

    /// Le héros attaqué par `id`, si `id` est l'attaque active (carte ou
    /// proxy) ou la source de l'attaque-proxy active (l'arme), et que
    /// l'attaque vise un héros (« this is attacking a marked hero »).
    pub fn attacked_hero(&self, id: ObjectId) -> Option<PlayerId> {
        let link = self.combat.active_link()?;
        let attacking = link.attack.id == id || link.source == Some(id);
        (attacking && link.target_obj.is_none()).then_some(link.target)
    }

    /// « Target dagger you control deals 1 damage to them. If damage is dealt
    /// this way, the dagger has hit » (Bite, Hurl, Danger Digits) : dégâts
    /// physiques de la source à un héros ; s'ils sont infligés, la source
    /// touche (événement `Hit` avec `effect`, CR 9.3.3 pour la marque). Le
    /// résultat est dans `last_damage` (instruction suivante).
    pub fn deal_damage_and_hit(
        &mut self,
        source: ObjectId,
        by: PlayerId,
        target: PlayerId,
        amount: i32,
    ) {
        self.run_damage(Box::new(DamageEvent {
            source: Some(source),
            by,
            target: Target::Player(target),
            amount,
            arcane: false,
            hit: false,
            effect_hit: true,
            replaced: false,
            declined: false,
            used: Vec::new(),
            on_prevent: Vec::new(),
        }));
    }

    /// Les dagues que `p` contrôle dans l'arène (armes, pas les attaques-proxy).
    pub fn daggers_of(&self, p: PlayerId) -> impl Iterator<Item = ObjectId> + '_ {
        self.player(p).weapons.iter().copied().filter(move |&w| {
            let o = self.obj(w);
            o.controller == p && o.zone.in_arena() && self.def(w).subtypes.has(Subtypes::DAGGER)
        })
    }

    /// CR 8.5.41c, 8.2.2b : `p` peut-il équiper cette arme (assez de mains
    /// libres) ?
    pub fn can_equip_weapon(&self, p: PlayerId, card: CardId) -> bool {
        let used: u8 = self
            .player(p)
            .weapons
            .iter()
            .map(|&w| hands_of(self.def(w)))
            .sum();
        used + hands_of(self.db.get(card)) <= 2
    }

    /// « Equip a Graphene Chelicera token » (CR 8.5.41d) : crée le token
    /// arme et l'équipe dans une zone d'arme libre de `p`. Si aucune zone
    /// n'est libre, le token n'est pas créé (CR 8.5.41d-e). Aucun
    /// remplacement de création de tokens de nos decks ne concerne les armes.
    pub fn equip_weapon_token(&mut self, p: PlayerId, card: CardId) -> Option<ObjectId> {
        debug_assert!(self.db.get(card).types.has(Types::TOKEN));
        if !self.can_equip_weapon(p, card) {
            return None;
        }
        let id = self.new_object(p, card);
        self.obj_mut(id).zone = Zone::Weapon;
        self.player_mut(p).weapons.push(id);
        self.emit(Event::EnterArena { card: id });
        Some(id)
    }

    /// Retrieve (CR 8.5.51), partie « equip » : la carte de l'arme quitte le
    /// cimetière et est équipée par son propriétaire (nouvel objet, CR 3.0.9).
    /// Le paiement de {r} est fait par la capacité. Renvoie `false` si la
    /// carte ne peut pas être équipée (CR 8.5.51a).
    pub fn equip_from_graveyard(&mut self, id: ObjectId) -> bool {
        let o = self.obj(id);
        let p = o.owner;
        if o.zone != Zone::Graveyard || !self.can_equip_weapon(p, o.card) {
            return false;
        }
        self.detach(id);
        let o = self.obj_mut(id);
        o.gen += 1;
        o.zone = Zone::Weapon;
        o.controller = p;
        o.face_down = false;
        self.player_mut(p).weapons.push(id);
        self.emit(Event::EnterArena { card: id });
        true
    }

    /// Les cartes qui ont ce trait (CR 2.13.4 : le groupe comprend **toutes**
    /// les cartes qui ont le trait, pas seulement celles du card-pool), dans
    /// l'ordre de la base.
    pub fn cards_with_trait(&self, t: Traits) -> Vec<CardId> {
        (0..self.db.len() as u16)
            .map(CardId)
            .filter(|&c| self.db.get(c).traits.has(t))
            .collect()
    }

    /// Become (CR 8.5.25) : le héros de `p` perd ses propriétés et prend
    /// celles de `card`. La vie « * » d'un Agent of Chaos vaut la vie
    /// imprimée du héros d'origine (CR 8.5.25e, 1.12.3b) : le total de vie
    /// du joueur ne change pas. CR 8.5.25d : devenir à nouveau la même
    /// référence échoue.
    pub fn become_hero(&mut self, p: PlayerId, card: CardId) {
        let hero = self.player(p).hero;
        let current = self.obj(hero).card;
        if current == card && self.became.iter().any(|&(q, _)| q == p) {
            return;
        }
        if !self.became.iter().any(|&(q, _)| q == p) {
            self.became.push((p, current));
        }
        self.set_card(hero, card);
        self.emit(Event::Became { player: p, card });
    }

    /// « You become a random Agent of Chaos » (Arakni, Web of Deceit,
    /// CR 2.13.4) : tirage parmi toutes les cartes qui ont le trait.
    pub fn become_random(&mut self, p: PlayerId, t: Traits) {
        let v = self.cards_with_trait(t);
        if v.is_empty() {
            return;
        }
        let i = self.rng.below(v.len() as u32) as usize;
        self.become_hero(p, v[i]);
    }

    /// Return to the brood (CR 8.5.52) : les effets « become » sur le héros
    /// de `p` cessent ; il redevient sa carte d'origine.
    pub fn return_to_brood(&mut self, p: PlayerId) {
        if let Some(i) = self.became.iter().position(|&(q, _)| q == p) {
            let (_, base) = self.became.remove(i);
            let hero = self.player(p).hero;
            self.set_card(hero, base);
        }
    }

    /// La carte d'origine du héros de `p` (avant un « become »).
    pub fn base_hero_card(&self, p: PlayerId) -> CardId {
        self.became
            .iter()
            .find(|&&(q, _)| q == p)
            .map(|&(_, c)| c)
            .unwrap_or(self.obj(self.player(p).hero).card)
    }

    /// « Your attacks [filtrées] get … this turn » (Night's Embrace).
    pub fn buff_attacks_this_turn(
        &mut self,
        p: PlayerId,
        filter: fn(&GameState, ObjectId) -> bool,
        delta: Delta,
    ) {
        self.attack_buffs.push(AttackBuff {
            player: p,
            filter,
            delta,
        });
    }

    /// Somme des effets « your attacks get … this turn » sur l'objet.
    pub(crate) fn attack_buff_delta(&self, id: ObjectId) -> Option<Delta> {
        let p = self.obj(id).controller;
        if !self.attack_buffs.iter().any(|b| b.player == p) || !self.is_attack(id) {
            return None;
        }
        let mut d = Delta::default();
        for b in &self.attack_buffs {
            if b.player == p && (b.filter)(self, id) {
                d.power += b.delta.power;
                d.defense += b.delta.defense;
                d.grants = d.grants | b.delta.grants;
            }
        }
        Some(d)
    }

    /// « The next time you attack [cible filtrée] this turn, the attack gets
    /// … » (Public Bounty) : appliqué à l'événement d'attaque (CR 7.2.3). Le
    /// filtre reçoit l'attaque ; le maillon actif est alors le sien.
    pub fn next_time_you_attack(
        &mut self,
        p: PlayerId,
        filter: fn(&GameState, ObjectId) -> bool,
        delta: Delta,
    ) {
        self.next_attack_effects.push(NextAttackEffect {
            player: p,
            filter: Some(filter),
            delta,
            cards_only: false,
            turn: None,
            weapon: None,
            combat_chain: false,
            at_attack: true,
            if_fused: false,
        });
    }

    /// Applique les effets « the next time you attack … » à l'attaque qui
    /// vient de devenir l'attaque du maillon actif.
    pub(crate) fn apply_at_attack_effects(&mut self, p: PlayerId, attack: ObjRef) {
        let mut i = 0;
        while i < self.next_attack_effects.len() {
            let e = self.next_attack_effects[i];
            if e.at_attack && e.player == p && e.filter.is_none_or(|f| f(self, attack.id)) {
                self.next_attack_effects.remove(i);
                self.modify(attack, e.delta, crate::state::Until::EndOfTurn);
            } else {
                i += 1;
            }
        }
    }

    /// « You may play it until the start of your next turn » pour une carte
    /// bannie **face cachée** (Arakni, Trap-Door) : permission pour le tour
    /// `turn`.
    pub fn permit_play_face_down(&mut self, p: PlayerId, card: ObjectId, turn: u32) {
        let card = self.obj_ref(card);
        self.play_permissions.push(PlayPermission {
            card,
            player: p,
            turn,
            face_down_ok: true,
        });
    }

    /// Déclenche la capacité n° `index` de l'objet `source` (contrôlée par
    /// son contrôleur) quand la condition dépend d'un effet de la carte
    /// elle-même, que l'événement ne décrit pas (Art of Desire: Body :
    /// « Whenever this banishes a red card »). La capacité (dont la
    /// condition `when` est toujours fausse) est mise en attente comme une
    /// capacité déclenchée normale (CR 6.6.6).
    pub fn queue_trigger(&mut self, source: ObjectId, index: u8) {
        let o = self.obj(source);
        let (controller, card) = (o.controller, o.card);
        let object = self.obj_ref(source);
        self.pending_triggers.push(Layer {
            kind: LayerKind::Trigger(index),
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
}
