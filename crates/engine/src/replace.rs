//! Effets de remplacement (CR 6.4, 6.5) : « If … would …, instead … ».
//!
//! Un remplacement modifie un événement **avant** qu'il se produise. Le moteur
//! décrit l'événement à venir par un [`Pending`], le passe aux remplacements
//! actifs, puis réalise l'événement modifié.
//!
//! Deux origines :
//! - [`ReplacementDef`] : capacité statique d'une carte (`CardDef::replacements`),
//!   ex. Embraforged Gauntlet « If this would be put into your graveyard from
//!   anywhere, instead banish it », ou Florian (tokens aura +1) ;
//! - [`Replacement`] : effet créé par la résolution d'une capacité, avec une
//!   durée (ex. Topsy Turvy, Crucible of Aetherweave).
//!
//! Les remplacements « internes » à une capacité (CR 6.4.7, ex. Arcane
//! Polarity « … instead gain 4 ») n'ont pas besoin de ce module : la carte
//! calcule directement le bon résultat.
//!
//! Événements gérés : mise au cimetière, mise sur le dessus d'un deck,
//! dégâts (avant les préventions, CR 6.5.4-6.5.5), création de tokens, sharpen.
//! Les préventions (CR 6.4.10) sont dans `effects.rs` (dégâts).

use crate::decision::{Choice, Decision, DecisionKind};
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::script::Target;
use crate::state::{GameState, Task, Until};
use crate::zones::Zone;

/// Le type d'événement qu'un remplacement modifie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplaceKind {
    /// Un objet va être mis dans un cimetière (détruit, défaussé, « clear »…).
    ToGraveyard,
    /// Un objet va être mis sur le dessus d'un deck.
    ToDeckTop,
    /// Des dégâts vont être infligés (montant modifiable).
    Damage,
    /// Des tokens vont être créés.
    CreateToken,
    /// Un objet va être aiguisé (sharpen) `amount` fois (ex. Swordmaster's
    /// Path : « instead sharpen it an additional time »).
    Sharpen,
}

/// L'événement sur le point de se produire, que les remplacements modifient.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    pub kind: ReplaceKind,
    /// L'objet déplacé, ou la source des dégâts.
    pub object: Option<ObjectId>,
    /// Propriétaire de l'objet déplacé, joueur qui inflige les dégâts, ou
    /// joueur sous le contrôle duquel les tokens sont créés.
    pub player: PlayerId,
    /// Destination d'un déplacement. `Zone::Gone` = retiré de la partie.
    pub zone: Zone,
    pub on_top: bool,
    /// Nombre de dégâts ou de tokens.
    pub amount: i32,
    pub arcane: bool,
    /// Cible des dégâts.
    pub target: Option<Target>,
    /// Le token créé.
    pub card: Option<CardId>,
}

impl Pending {
    pub fn new(kind: ReplaceKind, player: PlayerId) -> Pending {
        Pending {
            kind,
            object: None,
            player,
            zone: Zone::Gone,
            on_top: true,
            amount: 0,
            arcane: false,
            target: None,
            card: None,
        }
    }
}

/// Qui applique le remplacement : `me` est l'objet qui porte la capacité
/// (remplacement statique) ou l'objet mémorisé par l'effet (`Replacement::data`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Who {
    pub me: Option<ObjectId>,
    pub controller: PlayerId,
}

pub type AppliesFn = fn(&GameState, Who, &Pending) -> bool;
pub type ApplyFn = fn(&mut GameState, Who, &mut Pending);

/// Un remplacement statique imprimé sur une carte.
#[derive(Clone, Copy, Debug)]
pub struct ReplacementDef {
    pub kind: ReplaceKind,
    /// Fonctionne aussi hors de l'arène (CR 1.7.4j : « If this would be put
    /// into your graveyard from anywhere »). Sinon : seulement dans l'arène.
    pub anywhere: bool,
    /// « instead you may … » : le contrôleur choisit de l'appliquer ou non.
    pub optional: bool,
    pub applies: AppliesFn,
    pub apply: ApplyFn,
}

/// Un remplacement créé par un effet, avec une durée (CR 6.4.9).
#[derive(Clone, Copy, Debug)]
pub struct Replacement {
    pub id: u16,
    pub kind: ReplaceKind,
    pub controller: PlayerId,
    /// Objet mémorisé par l'effet (ex. « the next card you play »).
    pub data: Option<ObjRef>,
    pub optional: bool,
    pub applies: AppliesFn,
    pub apply: ApplyFn,
    pub until: Until,
    /// « The next time … » : nombre d'utilisations restantes (CR 6.4.2b).
    pub uses: Option<u8>,
}

/// Un remplacement candidat pour un événement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplSource {
    /// `CardDef::replacements[i]` de cet objet.
    Static(ObjectId, u8),
    /// `GameState::replacements` avec cet identifiant.
    Effect(u16),
}

impl GameState {
    /// Crée un remplacement (durée et nombre d'utilisations au choix).
    pub fn add_replacement(&mut self, mut r: Replacement) {
        self.next_effect_id = self.next_effect_id.wrapping_add(1);
        r.id = self.next_effect_id;
        self.replacements.push(r);
    }

    /// Un remplacement statique est-il fonctionnel (CR 1.7.4) ?
    fn static_functional(&self, id: ObjectId, def: &ReplacementDef) -> bool {
        let o = self.obj(id);
        if o.zone == Zone::Gone || self.lost_abilities(id) {
            return false;
        }
        def.anywhere || (o.zone.in_arena() && !o.face_down)
    }

    /// Les remplacements actifs pour cet événement, pas encore utilisés
    /// (CR 6.4.5 : un remplacement ne s'applique qu'une fois par événement).
    fn replacement_candidates(&self, p: &Pending, used: &[ReplSource]) -> Vec<(ReplSource, bool)> {
        let mut v = Vec::new();
        for &id in &self.replacement_objects {
            let defs = &self.def(id).replacements;
            for (i, d) in defs.iter().enumerate() {
                let src = ReplSource::Static(id, i as u8);
                if d.kind != p.kind || used.contains(&src) || !self.static_functional(id, d) {
                    continue;
                }
                let o = self.obj(id);
                let who = Who {
                    me: Some(id),
                    controller: if o.zone.in_arena() {
                        o.controller
                    } else {
                        o.owner
                    },
                };
                if (d.applies)(self, who, p) {
                    v.push((src, d.optional));
                }
            }
        }
        for r in &self.replacements {
            let src = ReplSource::Effect(r.id);
            if r.kind != p.kind || used.contains(&src) || r.uses == Some(0) {
                continue;
            }
            if (r.applies)(self, self.effect_who(r), p) {
                v.push((src, r.optional));
            }
        }
        v
    }

    fn effect_who(&self, r: &Replacement) -> Who {
        Who {
            me: r.data.filter(|&d| self.is_current(d)).map(|d| d.id),
            controller: r.controller,
        }
    }

    /// Applique un remplacement à l'événement.
    pub(crate) fn apply_replacement(&mut self, src: ReplSource, p: &mut Pending) {
        match src {
            ReplSource::Static(id, i) => {
                let d = self.def(id).replacements[i as usize];
                let o = self.obj(id);
                let who = Who {
                    me: Some(id),
                    controller: if o.zone.in_arena() {
                        o.controller
                    } else {
                        o.owner
                    },
                };
                (d.apply)(self, who, p);
            }
            ReplSource::Effect(rid) => {
                let Some(k) = self.replacements.iter().position(|r| r.id == rid) else {
                    return;
                };
                let r = self.replacements[k];
                if let Some(n) = &mut self.replacements[k].uses {
                    *n = n.saturating_sub(1);
                }
                let who = self.effect_who(&r);
                (r.apply)(self, who, p);
            }
        }
        self.replacements.retain(|r| r.uses != Some(0));
    }

    /// Qui contrôle un remplacement candidat (il choisit pour un optionnel).
    fn replacement_controller(&self, src: ReplSource) -> PlayerId {
        match src {
            ReplSource::Static(id, _) => {
                let o = self.obj(id);
                if o.zone.in_arena() {
                    o.controller
                } else {
                    o.owner
                }
            }
            ReplSource::Effect(rid) => self
                .replacements
                .iter()
                .find(|r| r.id == rid)
                .map_or(self.turn_player, |r| r.controller),
        }
    }

    /// Applique tous les remplacements **obligatoires** actifs, l'un après
    /// l'autre (CR 6.4.4 : l'événement modifié peut à son tour être remplacé).
    /// Renvoie le premier remplacement optionnel encore actif, s'il y en a un :
    /// l'appelant doit alors demander au joueur.
    ///
    /// TODO(règles) CR 6.5.1-6.5.4 : quand plusieurs remplacements obligatoires
    /// s'appliquent au même événement, le joueur choisit l'ordre. On applique
    /// pour l'instant les capacités des cartes puis les effets, dans l'ordre
    /// où ils ont été créés (aucun cas dans les 9 decks où l'ordre change le résultat).
    pub(crate) fn replace_mandatory(
        &mut self,
        p: &mut Pending,
        used: &mut Vec<ReplSource>,
    ) -> Option<ReplSource> {
        loop {
            let cands = self.replacement_candidates(p, used);
            if let Some(&(src, _)) = cands.iter().find(|(_, opt)| !opt) {
                used.push(src);
                self.apply_replacement(src, p);
                continue;
            }
            return cands.first().map(|&(src, _)| src);
        }
    }

    /// Aucun remplacement ne peut exister (chemin rapide).
    #[inline]
    pub(crate) fn no_replacements(&self) -> bool {
        self.replacement_objects.is_empty() && self.replacements.is_empty()
    }

    /// CR 6.4 : remplacements d'un déplacement vers le cimetière ou le dessus
    /// d'un deck. Renvoie la destination finale.
    ///
    /// Les déplacements sont instantanés dans le moteur : un remplacement
    /// optionnel n'y est pas géré (aucune carte des 9 decks n'en a).
    pub(crate) fn replace_move(&mut self, id: ObjectId, zone: Zone, on_top: bool) -> (Zone, bool) {
        let kind = match zone {
            Zone::Graveyard => ReplaceKind::ToGraveyard,
            Zone::Deck if on_top => ReplaceKind::ToDeckTop,
            _ => return (zone, on_top),
        };
        if self.no_replacements() {
            return (zone, on_top);
        }
        let mut p = Pending::new(kind, self.obj(id).owner);
        p.object = Some(id);
        p.zone = zone;
        p.on_top = on_top;
        let mut used = Vec::new();
        let opt = self.replace_mandatory(&mut p, &mut used);
        debug_assert!(opt.is_none(), "remplacement optionnel d'un déplacement");
        (p.zone, p.on_top)
    }

    /// Créer `n` tokens (CR 8.5.2) en passant par les remplacements
    /// (ex. Florian : un de plus ; Smoldering Scales : « instead you may destroy
    /// this »). Peut demander un choix : les tokens sont alors créés après la
    /// réponse. Les tokens créés sont dans `last_created`.
    pub fn create_tokens(&mut self, p: PlayerId, card: CardId, n: u8) {
        self.last_created.clear();
        let mut pending = Pending::new(ReplaceKind::CreateToken, p);
        pending.card = Some(card);
        pending.amount = n as i32;
        pending.zone = Zone::Permanent;
        self.run_replace(pending, Vec::new());
    }

    /// Fait avancer un événement remplaçable ; demande un choix si un
    /// remplacement optionnel est actif.
    pub(crate) fn run_replace(&mut self, mut p: Pending, mut used: Vec<ReplSource>) {
        if p.amount > 0 {
            if let Some(src) = self.replace_mandatory(&mut p, &mut used) {
                let player = self.replacement_controller(src);
                self.tasks.push(Task::Replace {
                    pending: p,
                    used,
                    asking: Some(src),
                });
                self.pending = Some(Decision {
                    player,
                    kind: DecisionKind::Replacement,
                    options: vec![Choice::Yes, Choice::No],
                });
                return;
            }
        }
        self.finish_replaced(p);
    }

    /// Réponse à « appliquer ce remplacement optionnel ? ».
    pub(crate) fn answer_replacement(&mut self, yes: bool) {
        let Some(Task::Replace {
            pending,
            used,
            asking,
        }) = self.tasks.pop()
        else {
            unreachable!("une question de remplacement vient d'une tâche Replace")
        };
        let (mut p, mut used) = (pending, used);
        let src = asking.expect("remplacement demandé");
        used.push(src);
        if yes {
            self.apply_replacement(src, &mut p);
        }
        self.run_replace(p, used);
    }

    /// Réalise l'événement une fois tous les remplacements appliqués.
    fn finish_replaced(&mut self, p: Pending) {
        // Les autres événements (dégâts, déplacements) sont réalisés
        // directement par le moteur.
        match p.kind {
            ReplaceKind::CreateToken => {
                if let Some(card) = p.card {
                    for _ in 0..p.amount.max(0) {
                        let id = self.create_token_raw(p.player, card);
                        self.last_created.push(id);
                    }
                }
            }
            ReplaceKind::Sharpen => {
                if let Some(id) = p.object {
                    for _ in 0..p.amount.max(0) {
                        self.sharpen_raw(p.player, id);
                    }
                }
            }
            _ => {}
        }
    }
}
