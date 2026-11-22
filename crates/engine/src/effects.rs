//! Primitives d'effet utilisées par les capacités des cartes (CR 8.5).
//!
//! Ces fonctions sont publiques : le code des cartes (`fab-cards`) les appelle
//! depuis ses instructions `Op::Do`.

use crate::card::{Delta, Types};
use crate::decision::{Choice, Decision, DecisionKind};
use crate::events::Event;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::replace::{Pending, ReplaceKind};
use crate::script::Target;
use crate::state::{
    GameState, Modifier, NextAttackEffect, Object, Prevention, Task, Until, WeaponEffect,
    WeaponEffectKind,
};
use crate::zones::Zone;

/// « If you prevent damage this way, … » : joueur protégé, quantité prévenue.
pub type OnPreventFn = fn(&mut GameState, PlayerId, i32);

/// Des dégâts en cours d'application (CR 8.5.3, 6.5).
#[derive(Clone, Debug)]
pub struct DamageEvent {
    pub source: Option<ObjectId>,
    pub by: PlayerId,
    pub target: Target,
    pub amount: i32,
    pub arcane: bool,
    /// Dégâts de combat du maillon actif : événement « hit » (CR 7.5.2a).
    pub hit: bool,
    /// Les remplacements standards ont déjà été appliqués.
    pub replaced: bool,
    /// Le joueur a refusé les préventions optionnelles restantes.
    pub declined: bool,
    /// Préventions déjà appliquées à cet événement (CR 6.4.5).
    pub used: Vec<PreventRef>,
    /// « If you prevent damage this way, … » à exécuter.
    pub on_prevent: Vec<(OnPreventFn, PlayerId, i32)>,
}

/// Une prévention qu'un joueur peut appliquer (CR 6.5.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreventRef {
    /// Un effet de `GameState::preventions` (par identifiant).
    Effect(u16),
    /// Arcane barrier de cet objet (CR 8.3.8).
    ArcaneBarrier(ObjectId),
    /// Ward de cet objet (CR 8.3.20).
    Ward(ObjectId),
    /// Spellvoid de cet objet (CR 8.3.15).
    Spellvoid(ObjectId),
}

struct PreventCandidate {
    what: PreventRef,
    optional: bool,
    /// Quantité prévenue au plus ; `None` = tout.
    amount: Option<i32>,
}

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

    /// Inflige des dégâts à un héros (CR 8.5.3). Raccourci de [`Self::deal_damage_to`].
    pub fn deal_damage(
        &mut self,
        source: Option<ObjectId>,
        by: PlayerId,
        target: PlayerId,
        amount: i32,
        arcane: bool,
    ) {
        self.deal_damage_to(source, by, Target::Player(target), amount, arcane);
    }

    /// Inflige des dégâts à un héros ou à un allié (CR 8.5.3).
    ///
    /// L'événement passe par les remplacements (CR 6.5.4) puis les préventions
    /// (CR 6.5.5) ; si le joueur protégé a un choix à faire (arcane barrier,
    /// spellvoid, ordre des préventions), les dégâts attendent sa réponse.
    /// Le résultat est rangé dans `last_damage`, à lire dans l'instruction
    /// **suivante** du script.
    pub fn deal_damage_to(
        &mut self,
        source: Option<ObjectId>,
        by: PlayerId,
        target: Target,
        amount: i32,
        arcane: bool,
    ) {
        self.run_damage(Box::new(DamageEvent {
            source,
            by,
            target,
            amount,
            arcane,
            hit: false,
            replaced: false,
            declined: false,
            used: Vec::new(),
            on_prevent: Vec::new(),
        }));
    }

    /// Fait avancer un événement de dégâts jusqu'à sa fin ou jusqu'à un choix.
    pub(crate) fn run_damage(&mut self, mut ev: Box<DamageEvent>) {
        // CR 1.9.1b : 0 dégât, rien ne se passe.
        if ev.amount <= 0 {
            self.last_damage = 0;
            return;
        }
        if let Target::Object(r) = ev.target {
            // CR 8.5.3c : un objet qui n'est pas vivant ne peut pas recevoir de dégâts.
            let alive = self.is_current(r) && self.is_living_permanent(r.id);
            if !alive {
                self.last_damage = 0;
                return;
            }
        }
        // CR 6.5.4 : remplacements standards (ex. « deals that much plus 1 »).
        if !ev.replaced {
            ev.replaced = true;
            if !self.no_replacements() {
                let mut p = Pending::new(ReplaceKind::Damage, ev.by);
                p.object = ev.source;
                p.amount = ev.amount;
                p.arcane = ev.arcane;
                p.target = Some(ev.target);
                let opt = self.replace_mandatory(&mut p, &mut Vec::new());
                debug_assert!(opt.is_none(), "remplacement optionnel de dégâts");
                ev.amount = p.amount;
                ev.arcane = p.arcane;
            }
        }
        // CR 6.5.5 : préventions, appliquées une à une par leur contrôleur.
        while ev.amount > 0 {
            let cands = self.prevention_candidates(&ev);
            if cands.is_empty() {
                break;
            }
            let all_mandatory = cands.iter().all(|c| !c.optional);
            // Si toutes les préventions sont obligatoires et qu'elles seront
            // toutes entièrement utilisées, l'ordre ne change rien.
            let total: Option<i32> = cands.iter().map(|c| c.amount).sum();
            if all_mandatory && (cands.len() == 1 || total.is_some_and(|t| t <= ev.amount)) {
                self.apply_prevention(&mut ev, cands[0].what);
                continue;
            }
            let Target::Player(player) = ev.target else {
                unreachable!("préventions seulement pour les héros")
            };
            let mut options: Vec<Choice> = cands.iter().map(|c| Choice::Prevent(c.what)).collect();
            if cands.iter().all(|c| c.optional) {
                options.push(Choice::Pass);
            }
            self.tasks.push(Task::Damage(ev));
            self.pending = Some(Decision {
                player,
                kind: DecisionKind::Prevention,
                options,
            });
            return;
        }
        self.finish_damage(*ev);
    }

    /// Les préventions actives pour un événement de dégâts (CR 6.4.2).
    fn prevention_candidates(&self, ev: &DamageEvent) -> Vec<PreventCandidate> {
        let mut v = Vec::new();
        // TODO(règles) : préventions des dégâts infligés à un allié (aucune
        // carte des 9 decks n'en crée).
        let Target::Player(p) = ev.target else {
            return v;
        };
        let src_ref = ev.source.map(|s| self.obj_ref(s));
        for prev in &self.preventions {
            let what = PreventRef::Effect(prev.id);
            if prev.player != p
                || ev.used.contains(&what)
                || prev.remaining.is_some_and(|r| r <= 0)
                || (prev.arcane_only && !ev.arcane)
                || (prev.source.is_some() && prev.source != src_ref)
            {
                continue;
            }
            if let Some(f) = prev.source_filter {
                if !ev.source.is_some_and(|s| f(self, s)) {
                    continue;
                }
            }
            v.push(PreventCandidate {
                what,
                optional: false,
                amount: prev.remaining,
            });
        }
        // Arcane barrier, ward et spellvoid des objets du joueur : « you » est
        // le contrôleur de l'objet (CR 1.1.2b).
        let ps = self.player(p);
        let sources = std::iter::once(ps.hero)
            .chain(ps.weapons.iter().copied())
            .chain(ps.equipment())
            .chain(
                self.permanents
                    .iter()
                    .copied()
                    .filter(|&o| self.obj(o).controller == p),
            );
        for o in sources {
            let def = self.def(o);
            if def.arcane_barrier == 0
                && def.ward == 0
                && def.spellvoid == 0
                && self.modifiers.is_empty()
                && def.self_mod.is_none()
            {
                continue; // chemin rapide : aucune valeur possible
            }
            let (ab, ward, sv) = self.prevention_values(o);
            // CR 8.3.8 : « you may pay N to prevent N of that damage ».
            if ev.arcane && ab > 0 && !ev.declined {
                let what = PreventRef::ArcaneBarrier(o);
                if !ev.used.contains(&what) && self.available_resources(p, None) >= ab as u32 {
                    v.push(PreventCandidate {
                        what,
                        optional: true,
                        amount: Some(ab as i32),
                    });
                }
            }
            // CR 8.3.15 : « you may destroy this to prevent N ».
            if ev.arcane && sv > 0 && !ev.declined {
                let what = PreventRef::Spellvoid(o);
                if !ev.used.contains(&what) {
                    v.push(PreventCandidate {
                        what,
                        optional: true,
                        amount: Some(sv as i32),
                    });
                }
            }
            // CR 8.3.20 : « destroy this to prevent N » (obligatoire).
            if ward > 0 {
                let what = PreventRef::Ward(o);
                if !ev.used.contains(&what) {
                    v.push(PreventCandidate {
                        what,
                        optional: false,
                        amount: Some(ward as i32),
                    });
                }
            }
        }
        v
    }

    /// Applique une prévention à l'événement (CR 6.4.10a).
    pub(crate) fn apply_prevention(&mut self, ev: &mut DamageEvent, what: PreventRef) {
        ev.used.push(what);
        let Target::Player(p) = ev.target else {
            return;
        };
        match what {
            PreventRef::Effect(id) => {
                let Some(k) = self.preventions.iter().position(|x| x.id == id) else {
                    return;
                };
                let prev = self.preventions[k];
                let n = prev.remaining.unwrap_or(ev.amount).min(ev.amount);
                ev.amount -= n;
                if prev.fixed || prev.remaining.is_some_and(|r| r <= n) {
                    // CR 6.4.10i-j : prévention fixe utilisée, ou bouclier épuisé.
                    self.preventions.remove(k);
                } else if let Some(r) = &mut self.preventions[k].remaining {
                    *r -= n;
                }
                if n > 0 {
                    if let Some(f) = prev.on_prevent {
                        ev.on_prevent.push((f, p, n));
                    }
                }
            }
            PreventRef::ArcaneBarrier(o) => {
                // Le paiement est demandé par l'appelant (tâche Pay).
                let n = (self.prevention_values(o).0 as i32).min(ev.amount);
                ev.amount -= n;
            }
            PreventRef::Spellvoid(o) => {
                let n = (self.prevention_values(o).2 as i32).min(ev.amount);
                self.destroy(o);
                ev.amount -= n;
            }
            PreventRef::Ward(o) => {
                let n = (self.prevention_values(o).1 as i32).min(ev.amount);
                self.destroy(o);
                ev.amount -= n;
            }
        }
    }

    /// Réponse au choix de prévention.
    pub(crate) fn answer_prevention(&mut self, choice: Choice) {
        let Some(Task::Damage(mut ev)) = self.tasks.pop() else {
            unreachable!("une question de prévention vient d'une tâche Damage")
        };
        match choice {
            Choice::Prevent(what @ PreventRef::ArcaneBarrier(o)) => {
                let Target::Player(p) = ev.target else {
                    unreachable!()
                };
                let cost = self.prevention_values(o).0;
                self.apply_prevention(&mut ev, what);
                // CR 8.3.8 : payer N, puis la suite des dégâts.
                self.tasks.push(Task::Damage(ev));
                self.tasks.push(Task::Pay {
                    player: p,
                    resources: cost,
                    action_points: 0,
                });
            }
            Choice::Prevent(what) => {
                self.apply_prevention(&mut ev, what);
                self.run_damage(ev);
            }
            _ => {
                // Le joueur refuse les préventions optionnelles restantes.
                ev.declined = true;
                self.run_damage(ev);
            }
        }
    }

    /// CR 8.5.3 : les dégâts restants sont infligés.
    fn finish_damage(&mut self, ev: DamageEvent) {
        for &(f, p, n) in &ev.on_prevent {
            f(self, p, n);
        }
        let dealt = ev.amount.max(0);
        self.last_damage = dealt;
        if dealt == 0 {
            return;
        }
        let (player, target_obj) = match ev.target {
            Target::Player(p) => {
                // CR 2.5.3e : jamais négatif.
                let ps = self.player_mut(p);
                ps.life = (ps.life - dealt).max(0);
                (p, None)
            }
            Target::Object(r) => {
                let base = self.def(r.id).life.unwrap_or(0) as i16;
                let o = self.obj_mut(r.id);
                o.life_delta = (o.life_delta - dealt as i16).max(-base);
                let c = o.controller;
                self.check_allies = true;
                (c, Some(r.id))
            }
        };
        self.emit(Event::Damage {
            source: ev.source,
            by: ev.by,
            target: player,
            target_obj,
            amount: dealt as u16,
            arcane: ev.arcane,
        });
        if ev.hit {
            // CR 9.3.3 : un héros marqué touché par une source adverse n'est plus marqué.
            if target_obj.is_none() && player != ev.by {
                self.player_mut(player).marked = false;
            }
            // CR 7.5.5 : l'attaque touche.
            if let Some(link) = self.combat.links.last_mut() {
                link.hit = true;
            }
            self.emit(Event::Hit {
                attack: ev.source.expect("source de l'attaque"),
                attacker: ev.by,
                target: player,
                target_obj,
                damage: dealt as u16,
            });
        }
    }

    /// Détruire (CR 8.5.4) : au cimetière du propriétaire.
    pub fn destroy(&mut self, id: ObjectId) {
        if self.obj(id).zone == Zone::Gone {
            return;
        }
        let owner = self.obj(id).owner;
        self.clear(id);
        // TODO(règles) : si la mise au cimetière est remplacée (« instead
        // banish it »), l'objet est-il encore « détruit » (déclencheurs « when
        // this is destroyed ») ? On considère que oui. Question au propriétaire.
        self.emit(Event::Destroyed { card: id, owner });
    }

    /// Bannir (CR 8.5.1). `face_down` : banni face cachée.
    pub fn banish(&mut self, id: ObjectId, face_down: bool) {
        let owner = self.obj(id).owner;
        // CR 3.0.7a : la puissance est celle de l'objet au moment où il quitte sa zone.
        let power = if face_down {
            0
        } else {
            self.power_if_public(id)
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

    /// Crée un token (CR 8.5.2), en passant par les remplacements. Renvoie le
    /// token créé, ou `None` si la création a été remplacée ou attend un choix.
    pub fn create_token(&mut self, p: PlayerId, card: CardId) -> Option<ObjectId> {
        self.create_tokens(p, card, 1);
        self.last_created.first().copied()
    }

    /// « Create a Crouching Tiger in your banished zone » : crée une carte
    /// (pas un token : ex. Crouching Tiger, carte avec ephemeral) dans la zone
    /// de bannissement de `p`, et la renvoie. Aucune carte des 9 decks ne
    /// remplace cette création.
    pub fn create_card_banished(&mut self, p: PlayerId, card: CardId) -> ObjectId {
        let id = self.new_object(p, card);
        self.obj_mut(id).zone = Zone::Banished;
        self.player_mut(p).banished.push(id);
        id
    }

    /// Un nouvel objet (hors de toute zone) appartenant à `p`.
    fn new_object(&mut self, p: PlayerId, card: CardId) -> ObjectId {
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
            life_delta: 0,
        });
        let id = ObjectId(self.objects.len() as u32 - 1);
        if !self.db.get(card).triggers.is_empty() {
            self.trigger_objects.push(id);
        }
        if !self.db.get(card).replacements.is_empty() {
            self.replacement_objects.push(id);
        }
        id
    }

    /// Crée un token, sans remplacement, dans la zone permanente (sous le
    /// contrôle de `p`) ou dans la zone de bannissement de `p`.
    pub(crate) fn create_token_raw(&mut self, p: PlayerId, card: CardId) -> ObjectId {
        debug_assert!(self.db.get(card).types.has(Types::TOKEN));
        let id = self.new_object(p, card);
        self.obj_mut(id).zone = Zone::Permanent;
        self.permanents.push(id);
        self.emit(Event::EnterArena { card: id });
        id
    }

    /// « You may play it this turn » (`turn = self.turn`) ou « during your
    /// next turn » (`turn = self.next_turn_of(p)`) : `p` peut jouer cet objet
    /// depuis sa zone de bannissement pendant le tour `turn`.
    pub fn permit_play(&mut self, p: PlayerId, card: ObjectId, turn: u32) {
        let card = self.obj_ref(card);
        self.play_permissions.push(crate::state::PlayPermission {
            card,
            player: p,
            turn,
        });
    }

    /// L'objet peut-il être joué depuis la zone de bannissement par `p` ce tour ?
    pub fn play_permitted(&self, p: PlayerId, card: ObjectId) -> bool {
        let o = self.obj(card);
        o.zone == Zone::Banished
            && !o.face_down
            && self
                .play_permissions
                .iter()
                .any(|x| x.player == p && x.turn == self.turn && x.card == o.obj_ref(card))
    }

    /// « Target attack gets "When this hits, …" » : accorde à `target` la
    /// capacité déclenchée n° `index` de `CardDef::granted` de la carte `card`.
    pub fn grant_trigger(&mut self, target: ObjRef, card: CardId, index: u8, until: Until) {
        if self.is_current(target) {
            self.granted_triggers.push(crate::state::GrantedTrigger {
                target,
                card,
                index,
                until,
            });
        }
    }

    /// Un objet « gets » +N / go again… (CR 8.5.8, 8.5.9).
    pub fn modify(&mut self, target: ObjRef, delta: Delta, until: Until) {
        if self.is_current(target) {
            self.any_lose_abilities |= delta.lose_abilities;
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
            turn: None,
            weapon: None,
            combat_chain: false,
        });
    }

    /// « The next [attaque filtrée] you play this combat chain gets … »
    /// (ex. Growl : « the next Crouching Tiger you play this combat chain ») :
    /// carte d'attaque jouée seulement, l'effet se termine quand la chaîne de
    /// combat se ferme (CR 7.7.7).
    pub fn next_attack_this_chain(
        &mut self,
        p: PlayerId,
        filter: Option<fn(&GameState, ObjectId) -> bool>,
        delta: Delta,
    ) {
        self.next_attack_effects.push(NextAttackEffect {
            player: p,
            filter,
            delta,
            cards_only: true,
            turn: None,
            weapon: None,
            combat_chain: true,
        });
    }

    /// Comme [`Self::next_attack`], mais pour la première attaque du tour n°
    /// `turn` (ex. Debilitate : « their first attack during their next turn »,
    /// avec `turn = self.next_turn_of(p)`).
    pub fn next_attack_on_turn(
        &mut self,
        p: PlayerId,
        turn: u32,
        filter: Option<fn(&GameState, ObjectId) -> bool>,
        delta: Delta,
    ) {
        self.next_attack_effects.push(NextAttackEffect {
            player: p,
            filter,
            delta,
            cards_only: false,
            turn: Some(turn),
            weapon: None,
            combat_chain: false,
        });
    }

    /// « Its next attack this turn gets … » : la prochaine attaque de cette
    /// arme (attaque-proxy, CR 1.4.3) ce tour (ex. Cut n' Carve).
    pub fn next_attack_with(&mut self, p: PlayerId, weapon: ObjRef, delta: Delta) {
        self.next_attack_effects.push(NextAttackEffect {
            player: p,
            filter: None,
            delta,
            cards_only: false,
            turn: None,
            weapon: Some(weapon),
            combat_chain: false,
        });
    }

    /// Effet du tour sur les attaques d'une arme : coût réduit, « attack
    /// twice », « attack an additional time » (CR 5.2.3).
    pub fn add_weapon_effect(&mut self, weapon: ObjRef, kind: WeaponEffectKind) {
        if self.is_current(weapon) {
            self.weapon_effects.push(WeaponEffect { weapon, kind });
        }
    }

    /// Aiguiser (sharpen, CR 8.5.58) : un compteur +1{p}, retiré au début de
    /// la phase de fin. Passe par les remplacements (Swordmaster's Path).
    pub fn sharpen(&mut self, p: PlayerId, id: ObjectId) {
        if !self.obj(id).zone.in_arena() {
            return;
        }
        let mut pending = Pending::new(ReplaceKind::Sharpen, p);
        pending.object = Some(id);
        pending.amount = 1;
        self.run_replace(pending, Vec::new());
    }

    /// Un sharpen, sans remplacement.
    pub(crate) fn sharpen_raw(&mut self, p: PlayerId, id: ObjectId) {
        if !self.obj(id).zone.in_arena() {
            return;
        }
        self.obj_mut(id).power_counters += 1;
        let r = self.obj_ref(id);
        if !self.sharpened.contains(&r) {
            self.sharpened.push(r);
        }
        self.emit(Event::Sharpened {
            player: p,
            card: id,
        });
    }

    /// « … unless you pay {r} » : le joueur paie `n` ressources (il pitche
    /// si besoin). À n'utiliser que si `available_resources` suffit.
    pub fn pay(&mut self, p: PlayerId, n: u8) {
        if n > 0 {
            self.tasks.push(Task::Pay {
                player: p,
                resources: n,
                action_points: 0,
            });
        }
    }

    /// Renvoie une carte dans la main de son propriétaire (ex. Shove Off :
    /// une carte qui défend). Elle cesse de défendre (CR 7.0.5a).
    pub fn return_to_hand(&mut self, id: ObjectId) {
        if self.obj(id).zone == Zone::Gone {
            return;
        }
        for l in &mut self.combat.links {
            l.defenders.retain(|&d| d != id);
        }
        self.move_to(id, Zone::Hand, true);
    }

    /// « Prevent the next N damage that would be dealt to … » : bouclier
    /// (CR 6.4.10j), jusqu'à la fin du tour.
    pub fn prevent_next(&mut self, p: PlayerId, n: i32, arcane_only: bool, source: Option<ObjRef>) {
        self.add_prevention(Prevention {
            id: 0,
            player: p,
            remaining: Some(n),
            arcane_only,
            source,
            source_filter: None,
            fixed: false,
            on_prevent: None,
            until: Until::EndOfTurn,
        });
    }

    /// « The next time you would be dealt damage this turn, prevent N of that
    /// damage » : prévention fixe (CR 6.4.10i). `n = None` : « prevent it ».
    pub fn prevent_next_time(&mut self, p: PlayerId, n: Option<i32>) {
        self.add_prevention(Prevention {
            id: 0,
            player: p,
            remaining: n,
            arcane_only: false,
            source: None,
            source_filter: None,
            fixed: true,
            on_prevent: None,
            until: Until::EndOfTurn,
        });
    }

    /// Ajoute un effet de prévention quelconque (l'identifiant est attribué ici).
    pub fn add_prevention(&mut self, mut prev: Prevention) {
        self.next_effect_id = self.next_effect_id.wrapping_add(1);
        prev.id = self.next_effect_id;
        self.preventions.push(prev);
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
                && e.turn.is_none_or(|t| t == self.turn)
                && (is_card || !e.cards_only)
                && e.weapon.is_none_or(|w| {
                    self.obj(attack.id).proxy_of == Some(w.id) && self.is_current(w)
                })
                && e.filter.is_none_or(|f| f(self, attack.id));
            if ok {
                self.next_attack_effects.remove(i);
                self.any_lose_abilities |= e.delta.lose_abilities;
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
