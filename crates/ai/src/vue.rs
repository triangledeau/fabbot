//! Ce qu'un joueur a le droit de voir (CR 3.0.3 : zones publiques et privées).
//!
//! Les bots heuristiques ne lisent la partie **qu'à travers** une [`Vue`] :
//! l'état complet (`GameState`) est un champ privé de ce module, le code des
//! bots ne peut donc pas y toucher directement. Une `Vue` ne montre :
//! - que les cartes visibles pour le joueur (même règle que l'interface :
//!   face visible, ou face cachée mais à lui et hors de son deck) ;
//! - des nombres publics : vies, tailles de main et de deck, points
//!   d'action et de ressource, état du combat et de la pile.
//!
//! Elle ne donne **jamais** la main adverse, l'arsenal adverse face cachée, ni
//! l'ordre d'un deck.

use fab_engine::state::{CombatStep, Task};
use fab_engine::{CardDef, GameState, Keywords, ObjectId, PlayerId, Zone};

/// L'état de la partie vu par un joueur.
pub struct Vue<'a> {
    g: &'a GameState,
    moi: PlayerId,
}

/// Le maillon actif de la chaîne de combat, en informations publiques.
#[derive(Clone, Copy, Debug)]
pub struct Maillon {
    pub attaque: ObjectId,
    pub attaquant: PlayerId,
    /// Le joueur dont le héros (ou l'allié) est attaqué.
    pub defenseur: PlayerId,
    /// L'attaque vise un héros (et pas un allié ou un objet).
    pub vise_heros: bool,
    pub etape: CombatStep,
}

impl<'a> Vue<'a> {
    pub fn new(g: &'a GameState, moi: PlayerId) -> Self {
        Vue { g, moi }
    }

    pub fn moi(&self) -> PlayerId {
        self.moi
    }

    pub fn adversaire(&self) -> PlayerId {
        self.moi.opponent()
    }

    /// La carte est-elle visible pour moi ? (même règle que `fab-ui`)
    pub fn visible(&self, id: ObjectId) -> bool {
        let o = self.g.obj(id);
        !o.face_down || (o.owner == self.moi && o.zone != Zone::Deck)
    }

    fn voir(&self, id: ObjectId) -> ObjectId {
        assert!(
            self.visible(id),
            "un bot a lu une carte cachée (objet {id:?}) : interdit"
        );
        id
    }

    /// Le texte d'une carte visible (panique sinon : erreur de programmation).
    pub fn carte(&self, id: ObjectId) -> &'a CardDef {
        self.g.def(self.voir(id))
    }

    /// Puissance actuelle d'une carte visible.
    pub fn puissance(&self, id: ObjectId) -> i16 {
        self.g.power(self.voir(id))
    }

    /// Défense actuelle d'une carte visible.
    pub fn defense(&self, id: ObjectId) -> i16 {
        self.g.defense(self.voir(id))
    }

    pub fn mots_cles(&self, id: ObjectId) -> Keywords {
        self.g.keywords(self.voir(id))
    }

    pub fn zone(&self, id: ObjectId) -> Zone {
        self.g.obj(self.voir(id)).zone
    }

    pub fn controleur(&self, id: ObjectId) -> PlayerId {
        self.g.obj(self.voir(id)).controller
    }

    /// Ma main.
    pub fn ma_main(&self) -> &'a [ObjectId] {
        &self.g.player(self.moi).hand
    }

    pub fn taille_main(&self, p: PlayerId) -> usize {
        self.g.player(p).hand.len()
    }

    pub fn vie(&self, p: PlayerId) -> i32 {
        self.g.player(p).life
    }

    pub fn points_d_action(&self) -> u8 {
        self.g.player(self.moi).action_points
    }

    pub fn points_de_ressource(&self) -> u8 {
        self.g.player(self.moi).resource_points
    }

    /// Coût en ressources d'une de mes cartes (sans coût optionnel ni X).
    pub fn cout(&self, id: ObjectId) -> u32 {
        self.g.play_cost(self.moi, self.voir(id))
    }

    /// Coût d'activation de la capacité `i` d'un objet visible.
    pub fn cout_activation(&self, id: ObjectId, i: u8) -> u32 {
        self.g.activation_cost(self.voir(id), i)
    }

    pub fn mon_tour(&self) -> bool {
        self.g.turn_player == self.moi
    }

    pub fn phase_d_action(&self) -> bool {
        self.g.action_phase
    }

    pub fn pile_vide(&self) -> bool {
        self.g.stack.is_empty()
    }

    pub fn combat_ouvert(&self) -> bool {
        self.g.combat.open
    }

    pub fn etape(&self) -> Option<CombatStep> {
        self.g.combat.step
    }

    /// La carte (ou l'allié) en haut de la pile, si c'est moi qui la joue :
    /// l'attaque dont je choisis la cible (CR 1.4.5).
    pub fn ma_carte_sur_la_pile(&self) -> Option<ObjectId> {
        let l = self.g.stack.last()?;
        (l.controller == self.moi).then_some(l.object.id)
    }

    /// Vie restante d'un objet vivant visible (allié, CR 2.5.3a).
    pub fn vie_objet(&self, id: ObjectId) -> Option<i32> {
        self.g.life_total(self.voir(id))
    }

    /// Le maillon actif (de l'étape d'attaque à l'étape des dégâts).
    pub fn maillon(&self) -> Option<Maillon> {
        let l = self.g.active_link()?;
        Some(Maillon {
            attaque: l.attack.id,
            attaquant: l.attacker,
            defenseur: l.target,
            vise_heros: l.target_obj.is_none(),
            etape: self.g.combat.step?,
        })
    }

    /// Les cartes qui défendent sur le maillon actif, plus celles que je suis
    /// en train de déclarer (étape de défense).
    pub fn defenseurs(&self) -> Vec<ObjectId> {
        let mut v: Vec<ObjectId> = self
            .g
            .active_link()
            .map(|l| l.defenders.clone())
            .unwrap_or_default();
        if self.g.active_link().is_some_and(|l| l.target == self.moi) {
            v.extend(self.g.declared_defenders.iter().copied());
        }
        v
    }

    /// Ressources qu'il me reste à payer, si je suis en train de payer.
    pub fn reste_a_payer(&self) -> Option<u32> {
        match self.g.tasks.last()? {
            Task::Pay {
                player, resources, ..
            } if *player == self.moi => {
                Some((*resources as u32).saturating_sub(self.points_de_ressource() as u32))
            }
            _ => None,
        }
    }
}
