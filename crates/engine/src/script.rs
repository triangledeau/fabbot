//! Les « programmes » des capacités de cartes.
//!
//! Une capacité est une suite d'instructions [`Op`] :
//! - `Do(f)` : exécute du code Rust qui modifie la partie (piocher, infliger
//!   des dégâts…). Si ce code a besoin d'un choix d'un joueur, il ajoute une
//!   tâche ; le programme reprend après elle.
//! - `Ask(f)` : pose une question à un joueur ; la réponse est rangée dans
//!   [`Ctx::answer`] pour les instructions suivantes.
//! - `If(cond, n)` : saute les `n` instructions suivantes si la condition est fausse.
//! - `Back(cond, n)` : revient `n` instructions en arrière tant que la condition
//!   est vraie (boucles « jusqu'à 4 cartes »).
//!
//! Grâce à ce découpage, le moteur peut **suspendre** une capacité au milieu
//! de sa résolution pour attendre une réponse, puis la reprendre.

use crate::decision::Choice;
use crate::events::Event;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::state::GameState;

/// Une cible déclarée (CR 1.8.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Un héros (désigné par son joueur).
    Player(PlayerId),
    /// Un objet public dans l'arène ou sur la pile.
    Object(ObjRef),
}

/// Description d'une cible à déclarer.
#[derive(Clone, Copy, Debug)]
pub struct TargetSpec {
    pub label: &'static str,
    /// Cette cible est-elle légale pour ce contexte ?
    pub filter: fn(&GameState, &Ctx, Target) -> bool,
    /// « up to 1 target » : le joueur peut ne rien choisir.
    pub optional: bool,
}

pub type DoFn = fn(&mut GameState, &mut Ctx);
pub type AskFn = fn(&GameState, &Ctx) -> Option<Ask>;
pub type CondFn = fn(&GameState, &Ctx) -> bool;

/// Une instruction.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Do(DoFn),
    /// Renvoie `None` pour ne pas poser la question (instruction ignorée).
    Ask(AskFn),
    If(CondFn, u8),
    Back(CondFn, u8),
}

/// Une question posée par une capacité.
#[derive(Clone, Debug)]
pub struct Ask {
    pub player: PlayerId,
    pub options: Vec<Choice>,
}

/// Le contexte d'une capacité en cours : qui la contrôle, ses cibles, ses
/// choix, et des cases libres pour les calculs intermédiaires.
#[derive(Clone, Debug, Default)]
pub struct Ctx {
    pub controller: PlayerId,
    /// Définition de la carte source (reste valable si la source disparaît, CR 1.7.1a).
    pub card: CardId,
    /// L'objet source, s'il existe.
    pub source: Option<ObjRef>,
    /// Cibles déclarées, dans l'ordre des `TargetSpec`. `None` = cible
    /// optionnelle non choisie, ou cible devenue illégale (CR 1.8.9).
    pub targets: Vec<Option<Target>>,
    /// Mode choisi (CR 1.7.5).
    pub mode: u8,
    /// Valeur de X (CR 5.1.3a).
    pub x: u8,
    /// Le coût additionnel optionnel a été payé.
    pub optional_paid: bool,
    /// Dernière réponse reçue.
    pub answer: Option<Choice>,
    /// Objets choisis / concernés (usage libre par la capacité).
    pub picked: Vec<ObjectId>,
    /// Nombres et indicateurs libres.
    pub num: i32,
    pub flag: bool,
    /// Événement déclencheur (pour une capacité déclenchée).
    pub event: Option<Event>,
}

impl Ctx {
    /// La cible `i` si elle existe encore et est toujours légale.
    pub fn target(&self, i: usize) -> Option<Target> {
        self.targets.get(i).copied().flatten()
    }

    pub fn source_id(&self) -> Option<ObjectId> {
        self.source.map(|r| r.id)
    }
}

/// D'où viennent les instructions d'une tâche de script (elles restent dans
/// la base de cartes, la tâche ne garde qu'une référence).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptRef {
    Resolve(CardId),
    EffectCost(CardId),
    Activated(CardId, u8),
    ActivatedCost(CardId, u8),
    Trigger(CardId, u8),
}

impl GameState {
    pub(crate) fn script_ops(&self, r: ScriptRef) -> &[Op] {
        match r {
            ScriptRef::Resolve(c) => &self.db.get(c).resolve,
            ScriptRef::EffectCost(c) => &self.db.get(c).effect_cost,
            ScriptRef::Activated(c, i) => &self.db.get(c).activated[i as usize].script,
            ScriptRef::ActivatedCost(c, i) => &self.db.get(c).activated[i as usize].effect_cost,
            ScriptRef::Trigger(c, i) => &self.db.get(c).triggers[i as usize].script,
        }
    }
}
