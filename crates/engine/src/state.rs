//! L'état complet d'une partie.

use crate::card::{CardDb, CardDef, Delta, Keywords, Subtypes, Types};
use crate::decision::Decision;
use crate::effects::DamageEvent;
use crate::events::Event;
use crate::ids::{CardId, ObjRef, ObjectId, PlayerId};
use crate::replace::{Pending, ReplSource, Replacement};
use crate::rng::Rng;
use crate::script::{Ctx, ScriptRef, Target};
use crate::zones::Zone;
use std::sync::Arc;

/// Un objet de la partie : une carte, un token, ou une attaque-proxy d'arme.
#[derive(Clone, Debug)]
pub struct Object {
    pub card: CardId,
    pub owner: PlayerId,
    pub controller: PlayerId,
    pub zone: Zone,
    /// Génération (CR 3.0.9) : change quand l'objet devient un nouvel objet.
    pub gen: u32,
    /// Face cachée (privé), ex. arsenal.
    pub face_down: bool,
    pub tapped: bool,
    /// Somme des compteurs +1/-1 de défense et de puissance (CR 1.15.2a).
    pub defense_counters: i8,
    pub power_counters: i8,
    /// Autres compteurs (ex. compteurs « verse »). Pour une attaque-proxy :
    /// numéro de cette attaque parmi les attaques de son arme ce tour (1 = la
    /// première), pour « its first attack this turn » (Zenith Blade).
    pub counters: u8,
    /// Pour une attaque-proxy : l'arme qu'elle représente (CR 1.4.3).
    pub proxy_of: Option<ObjectId>,
    /// Activations de chaque capacité ce tour (limite « Once per Turn »).
    pub activations: [u8; 4],
    /// Vie gagnée moins vie perdue, pour un allié (CR 2.5.3a). Remise à 0
    /// au début de la procédure de fin de tour (CR 4.4.3a, 8.2.8b).
    pub life_delta: i16,
    /// Compteurs +1{h} / -1{h} d'un allié (ex. Nekria) : ils modifient sa vie
    /// de base, y compris après la remise à zéro de fin de tour (CR 4.4.3a).
    pub life_counters: i8,
    /// « Fused » (CR 8.3.17a) : la carte a payé « Lightning Fusion » en la
    /// jouant. Vrai tant que l'objet est sur la pile ou dans l'arène ; remis à
    /// faux quand il devient un nouvel objet (CR 3.0.9).
    pub fused: bool,
}

impl Object {
    pub fn obj_ref(&self, id: ObjectId) -> ObjRef {
        ObjRef { id, gen: self.gen }
    }
}

/// Les zones et ressources d'un joueur.
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub hero: ObjectId,
    /// Total de vie du héros (CR 2.5.3).
    pub life: i32,
    pub action_points: u8,
    pub resource_points: u8,
    /// Le dessus du deck est le **dernier** élément.
    pub deck: Vec<ObjectId>,
    pub hand: Vec<ObjectId>,
    pub arsenal: Option<ObjectId>,
    pub pitch: Vec<ObjectId>,
    pub graveyard: Vec<ObjectId>,
    pub banished: Vec<ObjectId>,
    /// Armes et off-hand équipés (une arme 2H n'apparaît qu'une fois).
    pub weapons: Vec<ObjectId>,
    pub head: Option<ObjectId>,
    pub chest: Option<ObjectId>,
    pub arms: Option<ObjectId>,
    pub legs: Option<ObjectId>,
    /// Le héros est marqué (CR 8.5.50).
    pub marked: bool,
}

impl PlayerState {
    /// Les équipements (hors armes) équipés.
    pub fn equipment(&self) -> impl Iterator<Item = ObjectId> + '_ {
        [self.head, self.chest, self.arms, self.legs]
            .into_iter()
            .flatten()
    }
}

/// Type de couche sur la pile (CR 1.6.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    /// Une carte jouée.
    Card,
    /// Une capacité activée (index dans `CardDef::activated`). Si c'est une
    /// capacité « Attack », `object` est l'attaque-proxy (CR 1.4.3).
    Activated(u8),
    /// Une capacité déclenchée (index dans `CardDef::triggers`).
    Trigger(u8),
    /// Une capacité déclenchée **accordée** par une carte à un autre objet
    /// (index dans `CardDef::granted` de la carte qui l'accorde), ex. Legacy
    /// of Ikaru : « Target attack gets "When this hits, …" ».
    Granted(u8),
    /// Une capacité déclenchée définie par une règle ou un mot-clé
    /// (spectra, blade break…), gérée directement par le moteur.
    Rule(RuleTrigger),
}

/// Capacités déclenchées des mots-clés (CR 8.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleTrigger {
    /// CR 8.3.14 : « When this becomes the target of an attack, destroy this. »
    Spectra,
    /// CR 8.3.3 : « When the combat chain closes, if this defended, destroy it. »
    BladeBreak,
    /// CR 8.3.2 : « … if this defended, put a -1{d} counter on it. »
    Battleworn,
    /// CR 8.3.10 : « … put a -1{d} counter on it, then if it has 0{d}, destroy it. »
    Temper,
    /// CR 8.3.34 : « … put -1{d} counters on it equal to its {d}. »
    Guardwell,
    /// CR 8.5.58 : sharpen, partie déclenchée retardée : « at the beginning of
    /// the end phase, remove all +1{p} counters from it ».
    Unsharpen,
    /// CR 8.3.42 : « At the start of your turn, remove a suspense counter from this. »
    SuspenseTick,
    /// CR 8.3.42 : « When this has no suspense counters, destroy it. »
    SuspenseEmpty,
    /// CR 8.3.11 : « While this is in your banished zone, at the beginning
    /// of your end phase, lose 1{h}. »
    BloodDebt,
    /// CR 8.3.13 : « Whenever this is defended by a non-Illusionist attack
    /// action card with 6 or more {p}, destroy this. »
    Phantasm,
}

/// Une couche sur la pile.
#[derive(Clone, Debug)]
pub struct Layer {
    pub kind: LayerKind,
    /// La carte (ou la proxy) sur la pile ; pour un déclenchement, sa source.
    pub object: ObjRef,
    /// Définition utilisée pour les effets (reste valable si la source disparaît, CR 1.7.1a).
    pub card: CardId,
    pub controller: PlayerId,
    /// Cible de l'attaque (CR 1.4.5) : un héros ou un objet attaquable.
    pub target: Option<Target>,
    /// Zone d'où la carte a été jouée (main, arsenal...).
    pub origin: Zone,
    /// Cibles, modes, X... déclarés.
    pub ctx: Box<Ctx>,
}

/// Les étapes du combat (CR 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatStep {
    Layer,
    Attack,
    Defend,
    Reaction,
    Damage,
    Resolution,
}

/// Un maillon de la chaîne de combat (CR 7.0.3).
#[derive(Clone, Debug)]
pub struct ChainLink {
    /// L'attaque active (carte ou proxy).
    pub attack: ObjRef,
    /// L'arme représentée, pour une attaque-proxy.
    pub source: Option<ObjectId>,
    /// Définition de l'attaque (informations de dernier état, CR 7.0.3c).
    pub card: CardId,
    pub attacker: PlayerId,
    /// Le joueur qui contrôle la cible de l'attaque (« defending hero », CR 7.2.3b).
    pub target: PlayerId,
    /// La cible, si ce n'est pas un héros (allié, objet avec spectra).
    pub target_obj: Option<ObjRef>,
    pub defenders: Vec<ObjectId>,
    /// Nombre de cartes venues de la main qui défendent (pour dominate).
    pub defenders_from_hand: u8,
    pub hit: bool,
    /// Index du journal du tour au début de ce maillon (« this chain link »).
    pub log_start: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Combat {
    pub open: bool,
    pub step: Option<CombatStep>,
    pub links: Vec<ChainLink>,
    /// File d'attente des attaques (CR 3.15.7) : (attaque, cible).
    pub queue: Vec<(ObjRef, Target)>,
    /// Index du journal du tour à l'ouverture (« this combat chain »).
    pub log_start: usize,
}

impl Combat {
    /// Le maillon actif (CR 7.0.3b) : de l'étape d'attaque à l'étape des dégâts.
    pub fn active_link(&self) -> Option<&ChainLink> {
        match self.step {
            Some(
                CombatStep::Attack | CombatStep::Defend | CombatStep::Reaction | CombatStep::Damage,
            ) => self.links.last(),
            _ => None,
        }
    }
}

/// Durée d'un effet continu (CR 6.2.2a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Until {
    EndOfTurn,
    CombatChainCloses,
    /// Jusqu'à la fin du tour n° N (ex. « until the end of their next turn »,
    /// voir `GameState::end_of_next_turn`).
    EndOfTurnNumber(u32),
}

impl Until {
    /// L'effet se termine-t-il à la fin du tour `turn` (CR 4.4.4) ?
    pub fn ends_at_end_of(self, turn: u32) -> bool {
        match self {
            Until::EndOfTurn => true,
            Until::CombatChainCloses => false,
            Until::EndOfTurnNumber(n) => n <= turn,
        }
    }
}

/// « X gets +N » : modification d'un objet par un effet continu.
#[derive(Clone, Copy, Debug)]
pub struct Modifier {
    pub target: ObjRef,
    pub delta: Delta,
    pub until: Until,
    /// L'effet ne compte que si la cible est « fused » (Weave Lightning :
    /// « if it's fused, it gets go again »).
    pub if_fused: bool,
}

/// « Votre prochaine attaque [qui …] ce tour gets … » (CR 1.8.10), en attente.
#[derive(Clone, Copy, Debug)]
pub struct NextAttackEffect {
    pub player: PlayerId,
    /// Filtre sur l'attaque (ex. « attaque d'arme »), `None` = toute attaque.
    pub filter: Option<fn(&GameState, ObjectId) -> bool>,
    pub delta: Delta,
    /// Seulement pour une carte d'attaque jouée (« next attack action card you play »).
    pub cards_only: bool,
    /// Seulement pendant ce tour (« their first attack during their next turn ») ;
    /// `None` = ce tour-ci.
    pub turn: Option<u32>,
    /// Seulement une attaque de cette arme (« its next attack this turn »,
    /// ex. Cut n' Carve).
    pub weapon: Option<ObjRef>,
    /// L'effet se termine quand la chaîne de combat se ferme (« … you play
    /// this combat chain », ex. Growl), CR 7.7.7.
    pub combat_chain: bool,
    /// « The next time you attack [un héros marqué] this turn » (Public
    /// Bounty) : appliqué à l'événement d'attaque (CR 7.2.3), quand la cible
    /// est connue, et non à l'annonce de l'attaque. Le filtre peut alors lire
    /// le maillon actif.
    pub at_attack: bool,
    /// Le bonus devient un [`Modifier`] `if_fused` (la fusion est décidée
    /// après, pendant le paiement des coûts d'effet ; CR 6.4.7b, Weave Earth).
    pub if_fused: bool,
}

/// « Your next [carte filtrée] this turn costs {r} less to play » (ex. le
/// token Seismic Surge), CR 5.1.6. Plusieurs réductions s'appliquent à la
/// même carte, la prochaine jouée qui vérifie le filtre ; elles sont alors
/// utilisées.
#[derive(Clone, Copy, Debug)]
pub struct CostReduction {
    pub player: PlayerId,
    pub filter: fn(&GameState, ObjectId) -> bool,
    pub amount: u8,
    /// Le tour pendant lequel la réduction existe (« this turn »).
    pub turn: u32,
}

/// « You may play your next [carte filtrée] this turn as though it were an
/// instant » (Stir the Aetherwinds), CR 8.1.1d. Utilisée par la prochaine
/// carte jouée qui vérifie le filtre.
#[derive(Clone, Copy, Debug)]
pub struct InstantPermission {
    pub player: PlayerId,
    pub filter: fn(&GameState, ObjectId) -> bool,
    /// Le tour pendant lequel la permission existe (« this turn »).
    pub turn: u32,
}

/// « You may play it this turn / during your next turn » : permission de
/// jouer un objet depuis la zone de bannissement (ex. Crouching Tiger).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayPermission {
    pub card: ObjRef,
    pub player: PlayerId,
    /// Le tour pendant lequel la carte peut être jouée.
    pub turn: u32,
    /// La carte peut être jouée même face cachée (Arakni, Trap-Door : la
    /// carte est bannie face cachée).
    pub face_down_ok: bool,
}

/// Une capacité déclenchée accordée à un objet par un effet (CR 6.2,
/// « gets "When this hits, …" »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrantedTrigger {
    /// L'objet qui reçoit la capacité.
    pub target: ObjRef,
    /// La carte qui accorde la capacité (sa définition porte le script).
    pub card: CardId,
    /// Index dans `CardDef::granted`.
    pub index: u8,
    pub until: Until,
}

/// Effet d'un tour sur les attaques d'une arme (CR 5.2.3, 5.1.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponEffectKind {
    /// « Your next attack with it this turn costs {r} less to activate »
    /// (Sharp Incline) : consommé à la prochaine activation d'attaque.
    CostLess(u8),
    /// « You may attack with the weapon twice this turn » (Flurry) : fixe la
    /// limite d'activation à 2 (CR 5.2.3e ; plusieurs fois = toujours 2).
    AttackTwice,
    /// « You may attack an additional time with that weapon this turn »
    /// (Dorinthea) : dépasse la limite d'une activation (CR 5.2.3, 5.2.3c).
    AdditionalAttack,
}

/// Un [`WeaponEffectKind`] en cours, jusqu'à la fin du tour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponEffect {
    pub weapon: ObjRef,
    pub kind: WeaponEffectKind,
}

/// Un effet de prévention créé par une capacité (CR 6.4.10), qui protège un héros.
#[derive(Clone, Copy, Debug)]
pub struct Prevention {
    /// Identifiant (pour le choix de l'ordre, CR 6.5.5).
    pub id: u16,
    /// Le héros protégé ; son joueur contrôle l'effet.
    pub player: PlayerId,
    /// Quantité restante ; `None` = « prevent it » / « prevent all » (CR 6.4.10b).
    pub remaining: Option<i32>,
    pub arcane_only: bool,
    /// Seulement les dégâts de cette source.
    pub source: Option<ObjRef>,
    /// Seulement les dégâts des sources qui vérifient ce filtre (ex. « a red source »).
    pub source_filter: Option<fn(&GameState, ObjectId) -> bool>,
    /// Prévention « fixe » (« The next time … prevent N », CR 6.4.10i) : elle ne
    /// sert que pour un seul événement de dégâts. Sinon, bouclier (« prevent
    /// the next N damage », CR 6.4.10j) qui dure jusqu'à épuisement.
    pub fixed: bool,
    /// « If you prevent damage this way, … » (ex. Toe the Line) : appelé avec
    /// le joueur protégé et la quantité prévenue.
    pub on_prevent: Option<fn(&mut GameState, PlayerId, i32)>,
    pub until: Until,
}

/// Une tâche en attente (pile de tâches) : le moteur exécute toujours la tâche
/// du dessus avant de continuer le déroulement normal de la partie.
#[derive(Clone, Debug)]
pub enum Task {
    /// Exécuter une capacité à partir de l'instruction `pc`.
    /// `write_back` : à la fin, recopier le contexte dans la couche du dessus
    /// de la pile (coûts d'effet dont le résultat sert à la résolution).
    Script {
        script: ScriptRef,
        pc: u16,
        ctx: Box<Ctx>,
        write_back: bool,
    },
    /// Choix du côté d'une carte double (CR 5.1.2c), avant de la jouer :
    /// la réponse (`Choice::Mode`) est lue par `answer`.
    PlaySide(ObjectId),
    /// Étapes pour jouer / activer la couche du dessus de la pile (CR 5.1).
    PlayOptionalCost,
    PlayX,
    PlayMode,
    PlayTarget(u8),
    PlayPay,
    PlayEffectCost,
    PlayFinish,
    /// Payer des ressources puis des points d'action (CR 1.14.2).
    Pay {
        player: PlayerId,
        resources: u8,
        action_points: u8,
    },
    /// Suite de la résolution d'une couche après ses effets (CR 5.3.5-5.3.7).
    FinishResolve(Box<Layer>),
    /// Déclarer les cibles d'une couche déclenchée avant de l'empiler (CR 6.6.6a).
    TriggerTargets {
        layer: Box<Layer>,
        next: u8,
    },
    /// Déclarer la cible d'une attaque (CR 1.4.5, 5.1.4b).
    PlayAttackTarget,
    /// Des dégâts en cours (remplacements, préventions au choix), CR 6.5.
    Damage(Box<DamageEvent>),
    /// Un événement en attente d'un choix de remplacement optionnel.
    Replace {
        pending: Pending,
        used: Vec<ReplSource>,
        asking: Option<ReplSource>,
    },
}

/// Où en est le déroulement de la partie (hors tâches et priorité).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Choix du premier joueur (CR 4.1.3) : décision en attente.
    ChooseFirstPlayer,
    /// Début de tour (CR 4.2).
    StartPhase,
    /// Résoudre la pile sans priorité, puis passer à l'étape suivante (CR 4.2.2, 4.4.2).
    ResolveThen(AfterResolve),
    /// Un joueur a ou va recevoir la priorité (CR 1.11).
    Priority,
    /// Déclaration des cartes qui défendent (CR 7.3.2).
    Defending,
    /// Phase de fin, mise en arsenal (CR 4.4.3b).
    EndArsenal,
    /// Phase de fin, pitch sous le deck, joueur donné (CR 4.4.3c).
    EndPitchOrder(PlayerId),
    /// Suite et fin de la phase de fin (CR 4.4.3d-f, 4.4.4).
    EndRest,
    /// Étape de fermeture de la chaîne de combat (CR 7.7.4-7.7.7) : la pile
    /// se résout sans priorité, puis la chaîne se ferme.
    Closing,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterResolve {
    ActionPhase,
    EndArsenal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Winner(PlayerId),
    Draw,
}

/// Pourquoi la partie s'est terminée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    /// Un héros est tombé à 0 PV (CR 4.5.3a) ou les deux (CR 4.5.4a).
    LifeZero,
    /// Plus aucun joueur ne peut faire avancer la partie (CR 4.5.4d), détecté
    /// dans le cas simple (decks, mains et arsenaux vides des deux côtés,
    /// aucune capacité gratuite) ou quand, decks et mains vides, aucun total
    /// de vie ne change pendant 6 tours.
    Stalemate,
    /// Limite de tours du simulateur atteinte : **pas une règle du jeu**,
    /// une sécurité pour les simulations (équivalent d'une fin de ronde).
    TurnLimit,
}

/// Erreur renvoyée quand on donne une réponse impossible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    NoPendingDecision,
    InvalidChoice { index: usize, available: usize },
}

/// L'état complet d'une partie. Il doit rester **peu coûteux à copier**,
/// car l'annulation et la recherche du bot en font beaucoup. La base de cartes
/// est immuable et partagée (`Arc`), elle n'est pas recopiée.
#[derive(Clone, Debug)]
pub struct GameState {
    pub db: Arc<CardDb>,
    pub rng: Rng,
    pub objects: Vec<Object>,
    pub players: [PlayerState; 2],
    /// Zone permanente partagée (CR 3.13).
    pub permanents: Vec<ObjectId>,
    pub turn: u32,
    pub turn_player: PlayerId,
    pub first_player: PlayerId,
    /// Joueur qui choisit le premier joueur (CR 4.1.3).
    pub chooser: PlayerId,
    pub stage: Stage,
    /// Joueur ayant la priorité (CR 1.11.2).
    pub priority: Option<PlayerId>,
    /// Nombre de « passe » consécutifs (CR 1.11.4a).
    pub passes: u8,
    /// La pile : le dessus est le dernier élément (CR 3.15).
    pub stack: Vec<Layer>,
    pub combat: Combat,
    pub modifiers: Vec<Modifier>,
    /// Un des `modifiers` fait perdre ses capacités à un objet (chemin rapide).
    pub any_lose_abilities: bool,
    /// Un allié a perdu des PV : vérifier s'il est à 0 (CR 1.10.2b).
    pub check_allies: bool,
    pub next_attack_effects: Vec<NextAttackEffect>,
    /// Effets du tour sur les attaques des armes (coût, limite d'activation).
    pub weapon_effects: Vec<WeaponEffect>,
    /// Permissions de jouer des objets depuis la zone de bannissement.
    pub play_permissions: Vec<PlayPermission>,
    /// Capacités déclenchées accordées par des effets.
    pub granted_triggers: Vec<GrantedTrigger>,
    /// Objets aiguisés ce tour (sharpen, CR 8.5.58) : leurs compteurs +1{p}
    /// sont retirés au début de la phase de fin.
    pub sharpened: Vec<ObjRef>,
    /// Réductions de coût « your next … costs {r} less » en attente.
    pub cost_reductions: Vec<CostReduction>,
    /// Permissions « play your next … as though it were an instant » en attente.
    pub instant_permissions: Vec<InstantPermission>,
    /// « Attack action cards they control can't gain {p} during their next
    /// action phase » (Chokeslam) : (joueur, numéro du tour).
    pub no_power_gain: Vec<(PlayerId, u32)>,
    /// Le joueur actif est dans sa phase d'action (CR 4.3), entre les
    /// événements `BeginActionPhase` et `BeginEndPhase`.
    pub action_phase: bool,
    pub preventions: Vec<Prevention>,
    /// Effets de remplacement créés par des capacités (CR 6.4).
    pub replacements: Vec<Replacement>,
    /// Objets dont la carte a des remplacements statiques.
    pub replacement_objects: Vec<ObjectId>,
    /// Compteur d'identifiants des préventions et remplacements.
    pub next_effect_id: u16,
    /// Dégâts réellement infligés par le dernier événement de dégâts terminé
    /// (« if damage is dealt this way », « for each damage dealt this way »).
    /// À lire dans l'instruction **suivante** du script : les dégâts peuvent
    /// attendre un choix de prévention.
    pub last_damage: i32,
    /// Tokens créés par le dernier `create_tokens` (même remarque).
    pub last_created: Vec<ObjectId>,
    /// Couches déclenchées créées, pas encore ajoutées à la pile (CR 6.6.6).
    pub pending_triggers: Vec<Layer>,
    /// Joueur qui empile ses déclenchements en premier (CR 6.6.6b).
    pub trigger_first: Option<PlayerId>,
    pub tasks: Vec<Task>,
    /// Cartes déclarées pendant la déclaration de défense en cours.
    pub declared_defenders: Vec<ObjectId>,
    /// Journal des événements du tour.
    pub turn_log: Vec<Event>,
    /// Journal complet facultatif, jamais vidé par le moteur (`None` par
    /// défaut, donc gratuit pour les simulations). L'interface l'active pour
    /// afficher ce qui s'est passé, y compris en fin de tour, et le vide
    /// elle-même au fil de l'affichage.
    pub record: Option<Vec<Event>>,
    pub pending: Option<Decision>,
    pub outcome: Option<Outcome>,
    pub end_reason: Option<EndReason>,
    pub max_turns: u32,
    /// Objets dont la carte a des capacités déclenchées (optimisation de `emit`).
    pub trigger_objects: Vec<ObjectId>,
    /// Cartes sous un permanent (CR 3.0.14) : (sub-card, top-card).
    pub sub_cards: Vec<(ObjectId, ObjectId)>,
    /// Contrôle gagné « until the end of this action phase » (steal, Jack Be
    /// Quick) : (objet, contrôleur précédent), rendu à la fin de la phase
    /// d'action (CR 8.5.54).
    pub steals: Vec<(ObjRef, PlayerId)>,
    /// Effets « become » en cours sur un héros (CR 8.5.25) : (joueur, carte
    /// d'origine du héros). « Return to the brood » les fait cesser
    /// (CR 8.5.52).
    pub became: Vec<(PlayerId, CardId)>,
    /// « Your attacks with stealth get +1{p} this turn » (Night's Embrace) :
    /// effets du tour sur toutes les attaques d'un joueur, présentes et futures.
    pub attack_buffs: Vec<AttackBuff>,
    /// Blocage (CR 4.5.4d), cas étendu : nombre de tours de suite terminés
    /// sans carte dans les decks et les mains, et sans changement des totaux
    /// de vie (`stale_life`).
    pub stale_turns: u8,
    pub stale_life: [i32; 2],
    /// Le card-pool de chaque joueur (cartes qui peuvent aller dans son
    /// deck, `Loadout::pool`), partagé entre les copies de la partie.
    pub card_pools: Arc<[Vec<CardId>; 2]>,
    /// Les objets créés pour le deck de chaque joueur à la mise en place :
    /// identifiants `début..fin` (les cartes créées plus tard, comme Crouching
    /// Tiger, n'en font pas partie).
    pub deck_objects: [(u32, u32); 2],
}

/// « Your attacks [filtrées] get … this turn » (Night's Embrace) : s'applique
/// à chaque attaque du joueur (carte ou proxy, sur la pile ou la chaîne de
/// combat) jusqu'à la fin du tour.
#[derive(Clone, Copy, Debug)]
pub struct AttackBuff {
    pub player: PlayerId,
    /// Filtre sur l'attaque. Il est appelé pendant le calcul des valeurs :
    /// il ne doit pas lire les valeurs modifiées (`power`, `keywords`…).
    pub filter: fn(&GameState, ObjectId) -> bool,
    pub delta: Delta,
}

impl GameState {
    #[inline]
    pub fn obj(&self, id: ObjectId) -> &Object {
        &self.objects[id.0 as usize]
    }

    #[inline]
    pub fn obj_mut(&mut self, id: ObjectId) -> &mut Object {
        &mut self.objects[id.0 as usize]
    }

    #[inline]
    pub fn def(&self, id: ObjectId) -> &CardDef {
        self.db.get(self.obj(id).card)
    }

    #[inline]
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p.index()]
    }

    #[inline]
    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p.index()]
    }

    pub fn obj_ref(&self, id: ObjectId) -> ObjRef {
        self.obj(id).obj_ref(id)
    }

    /// L'objet référencé existe-t-il encore sous cette forme (CR 3.0.9) ?
    pub fn is_current(&self, r: ObjRef) -> bool {
        let o = self.obj(r.id);
        o.gen == r.gen && o.zone != Zone::Gone
    }

    pub fn pending_decision(&self) -> Option<&Decision> {
        self.pending.as_ref()
    }

    pub fn is_over(&self) -> bool {
        self.outcome.is_some()
    }

    /// Répond à la décision en attente en choisissant l'option `index`,
    /// puis fait avancer la partie jusqu'à la prochaine décision.
    pub fn apply(&mut self, index: usize) -> Result<(), ApplyError> {
        let decision = self.pending.take().ok_or(ApplyError::NoPendingDecision)?;
        let Some(&choice) = decision.options.get(index) else {
            let available = decision.options.len();
            self.pending = Some(decision);
            return Err(ApplyError::InvalidChoice { index, available });
        };
        self.answer(decision.player, decision.kind, choice);
        self.advance();
        Ok(())
    }

    /// Recalcule la décision de priorité en attente après une modification
    /// directe de l'état (tests, outils). Sans effet sur les autres décisions.
    pub fn refresh_priority(&mut self) {
        if self
            .pending
            .as_ref()
            .is_some_and(|d| d.kind == crate::decision::DecisionKind::Priority)
        {
            self.pending = None;
            self.advance();
        }
    }

    /// Le maillon actif, s'il existe.
    pub fn active_link(&self) -> Option<&ChainLink> {
        self.combat.active_link()
    }

    /// L'attaque du maillon actif (CR 7.0.4).
    pub fn active_attack(&self) -> Option<ObjectId> {
        self.combat
            .active_link()
            .map(|l| l.attack.id)
            .filter(|&a| self.obj(a).zone == Zone::CombatChain)
    }

    // ------------------------------------------------------------------
    // Déplacements d'objets (CR 3.0.7)
    // ------------------------------------------------------------------

    /// Retire l'objet de son conteneur actuel (sans changer `zone`).
    pub(crate) fn detach(&mut self, id: ObjectId) {
        let o = self.obj(id);
        let owner = o.owner;
        let zone = o.zone;
        if zone == Zone::Permanent {
            self.permanents.retain(|x| *x != id);
            return;
        }
        if zone == Zone::Under {
            self.sub_cards.retain(|&(s, _)| s != id);
            return;
        }
        let p = self.player_mut(owner);
        let remove = |v: &mut Vec<ObjectId>| {
            if let Some(i) = v.iter().position(|x| *x == id) {
                v.remove(i);
            }
        };
        match zone {
            Zone::Deck => remove(&mut p.deck),
            Zone::Hand => remove(&mut p.hand),
            Zone::Pitch => remove(&mut p.pitch),
            Zone::Graveyard => remove(&mut p.graveyard),
            Zone::Banished => remove(&mut p.banished),
            Zone::Arsenal if p.arsenal == Some(id) => p.arsenal = None,
            // Les équipements et armes restent « équipés » quand ils sont sur
            // la chaîne de combat (CR 7.7.5) ; leur emplacement n'est libéré
            // que lorsqu'ils quittent l'arène (voir `unequip`).
            _ => {}
        }
    }

    /// Libère l'emplacement d'équipement de l'objet s'il en occupait un.
    pub(crate) fn unequip(&mut self, id: ObjectId) {
        let owner = self.obj(id).owner;
        let p = self.player_mut(owner);
        p.weapons.retain(|x| *x != id);
        for slot in [&mut p.head, &mut p.chest, &mut p.arms, &mut p.legs] {
            if *slot == Some(id) {
                *slot = None;
            }
        }
    }

    /// Déplace un objet vers une zone d'un joueur (deck, main, cimetière…) ou
    /// la zone permanente. Applique CR 3.0.9 (nouvel objet hors de l'arène).
    /// Renvoie la zone d'origine.
    pub(crate) fn move_to(&mut self, id: ObjectId, zone: Zone, on_top: bool) -> Zone {
        // CR 6.4 : « If this would be put into a graveyard, instead … ».
        let (zone, on_top) = self.replace_move(id, zone, on_top);
        let from = self.obj(id).zone;
        if zone == Zone::Gone {
            // Retiré de la partie (CR 8.3.21b, ex. ephemeral).
            let was_in_arena = from.in_arena();
            self.detach(id);
            self.unequip(id);
            let owner = self.obj(id).owner;
            let o = self.obj_mut(id);
            o.zone = Zone::Gone;
            o.gen += 1;
            self.clear_sub_cards(id);
            if was_in_arena {
                self.emit(Event::LeaveArena { card: id, owner });
            }
            return from;
        }
        let was_in_arena = from.in_arena();
        // Un token qui quitte l'arène cesse d'exister (CR 8.1.8a).
        if self.def(id).types.has(Types::TOKEN) && was_in_arena && !zone.in_arena() {
            self.detach(id);
            self.unequip(id);
            let o = self.obj_mut(id);
            o.zone = Zone::Gone;
            o.gen += 1;
            self.clear_sub_cards(id);
            self.emit(Event::LeaveArena {
                card: id,
                owner: self.obj(id).owner,
            });
            return from;
        }
        self.detach(id);
        if !zone.in_arena() {
            self.unequip(id);
        }
        let owner = self.obj(id).owner;
        {
            if zone == Zone::Permanent {
                self.permanents.push(id);
            }
            let p = self.player_mut(owner);
            match zone {
                Zone::Deck => {
                    if on_top {
                        p.deck.push(id)
                    } else {
                        p.deck.insert(0, id)
                    }
                }
                Zone::Hand => p.hand.push(id),
                Zone::Pitch => p.pitch.push(id),
                Zone::Graveyard => p.graveyard.push(id),
                Zone::Banished => p.banished.push(id),
                Zone::Arsenal => p.arsenal = Some(id),
                _ => {}
            }
        }
        // CR 9.2.3 : une carte double n'a les propriétés du côté choisi que
        // tant qu'elle est sur la pile ; ailleurs, elle a celles des deux côtés.
        if let Some(sp) = self.def(id).split {
            if !zone.in_arena() && zone != Zone::Stack {
                self.obj_mut(id).card = sp.whole;
            }
        }
        let o = self.obj_mut(id);
        // CR 3.0.9 : entrer dans une zone hors arène (sauf pile) = nouvel objet.
        if !zone.in_arena() && zone != Zone::Stack {
            o.gen += 1;
            o.defense_counters = 0;
            o.power_counters = 0;
            o.counters = 0;
            o.tapped = false;
            o.activations = [0; 4];
            o.life_delta = 0;
            o.life_counters = 0;
            o.fused = false;
        }
        o.zone = zone;
        o.face_down = matches!(zone, Zone::Deck | Zone::Hand | Zone::Arsenal);
        o.controller = owner;
        if !zone.in_arena() && zone != Zone::Stack {
            // CR 3.0.14e : nouvel objet, ses sub-cards sont « clear ».
            self.clear_sub_cards(id);
        }
        if was_in_arena && !zone.in_arena() {
            self.emit(Event::LeaveArena { card: id, owner });
        }
        // CR 9.1.3b : une carte retournable redevient face avant hors de
        // l'arène (après l'événement, pour les capacités « when this leaves
        // the arena » de sa face arrière, CR 5.4.6a).
        if !zone.in_arena() && zone != Zone::Stack {
            self.restore_front_face(id);
        }
        if !was_in_arena && zone.in_arena() {
            // CR 8.3.42 : « This enters the arena with 2 suspense counters. »
            if zone == Zone::Permanent && self.def(id).keywords.has(Keywords::SUSPENSE) {
                self.obj_mut(id).counters = 2;
            }
            // « This enters the arena with N verse counters » (Runeblood Incantation).
            let n = self.def(id).enter_counters;
            if n > 0 {
                self.obj_mut(id).counters = n;
            }
            self.emit(Event::EnterArena { card: id });
        }
        from
    }

    /// Met un objet sur la pile ou la chaîne de combat (il garde son identité).
    pub(crate) fn place(&mut self, id: ObjectId, zone: Zone, controller: PlayerId) {
        debug_assert!(matches!(zone, Zone::Stack | Zone::CombatChain));
        self.detach(id);
        let o = self.obj_mut(id);
        o.zone = zone;
        o.face_down = false;
        o.controller = controller;
    }

    /// Retourne un équipement / une arme dans sa zone d'équipement (CR 7.7.5).
    pub(crate) fn return_to_equipment_zone(&mut self, id: ObjectId) {
        let zone = equipment_zone(self.def(id));
        let o = self.obj_mut(id);
        o.zone = zone;
        // Un allié (volé, Jack Be Quick) garde son contrôleur ; un équipement
        // revient à son propriétaire.
        if zone != Zone::Permanent {
            o.controller = o.owner;
        }
        if zone == Zone::Permanent {
            self.permanents.push(id);
        }
    }

    /// « Clear » (CR 3.0.12) : au cimetière du propriétaire ; une proxy ou un
    /// token cesse d'exister.
    pub fn clear(&mut self, id: ObjectId) {
        if self.obj(id).proxy_of.is_some() || self.def(id).types.has(Types::TOKEN) {
            let was_in_arena = self.obj(id).zone.in_arena();
            self.detach(id);
            self.unequip(id);
            let owner = self.obj(id).owner;
            let o = self.obj_mut(id);
            o.zone = Zone::Gone;
            o.gen += 1;
            self.clear_sub_cards(id);
            if was_in_arena && self.obj(id).proxy_of.is_none() {
                self.emit(Event::LeaveArena { card: id, owner });
            }
        } else {
            self.put_in_graveyard(id);
        }
    }

    /// Met un objet au cimetière de son propriétaire (événement « ToGraveyard »).
    pub fn put_in_graveyard(&mut self, id: ObjectId) {
        let owner = self.obj(id).owner;
        // Puissance utile seulement si un remplacement envoie l'objet en bannissement.
        let power = if self.no_replacements() {
            0
        } else {
            self.power_before_banish(id)
        };
        let from = self.move_to(id, Zone::Graveyard, true);
        match self.obj(id).zone {
            Zone::Graveyard => self.emit(Event::ToGraveyard {
                card: id,
                owner,
                from,
            }),
            // Remplacé par « instead banish it » (ex. Embraforged Gauntlet) :
            // l'effet de remplacement est celui de la carte, contrôlé par son
            // propriétaire.
            Zone::Banished => self.emit(Event::Banished {
                card: id,
                owner,
                by: owner,
                from,
                power,
                face_down: false,
            }),
            _ => {}
        }
    }

    /// Puissance d'un objet public qui a la propriété (0 sinon), avant un déplacement.
    pub(crate) fn power_if_public(&self, id: ObjectId) -> u16 {
        if self.def(id).power.is_some() && !self.obj(id).face_down {
            self.power(id) as u16
        } else {
            0
        }
    }

    /// Puissance d'un objet qui va être banni face visible : dernier état
    /// connu s'il est dans l'arène (CR 3.0.7a), sinon puissance imprimée
    /// (hors de l'arène, ses capacités statiques ne fonctionnent pas, CR 1.7.4 ;
    /// une carte du deck ou de la main devient publique en arrivant).
    pub(crate) fn power_before_banish(&self, id: ObjectId) -> u16 {
        if self.obj(id).zone.in_arena() {
            self.power_if_public(id)
        } else {
            self.def(id).power.unwrap_or(0) as u16
        }
    }

    // ------------------------------------------------------------------
    // Valeurs calculées (CR 2, CR 6.3)
    // ------------------------------------------------------------------

    /// Somme des effets continus qui s'appliquent à un objet.
    pub fn continuous_delta(&self, id: ObjectId) -> Delta {
        self.delta_with(id, false, false)
    }

    /// `no_gain` : les bonus de puissance sont ignorés (« can't gain {p} »),
    /// seuls les malus comptent.
    fn delta_with(&self, id: ObjectId, no_gain: bool, skip_self_mod: bool) -> Delta {
        let o = self.obj(id);
        let r = o.obj_ref(id);
        let mut d = Delta::default();
        let mut add = |x: Delta| {
            d.power += if no_gain { x.power.min(0) } else { x.power };
            d.defense += x.defense;
            d.grants = d.grants | x.grants;
            d.arcane_barrier += x.arcane_barrier;
            d.ward += x.ward;
            d.spellvoid += x.spellvoid;
            d.lose_abilities |= x.lose_abilities;
            d.loses = d.loses | x.loses;
        };
        for m in &self.modifiers {
            if m.target == r && (!m.if_fused || o.fused) {
                add(m.delta);
            }
        }
        // « Your attacks with stealth get +1{p} this turn » (Night's Embrace).
        if !self.attack_buffs.is_empty() {
            if let Some(x) = self.attack_buff_delta(id) {
                add(x);
            }
        }
        // Effets statiques de la carte elle-même (CR 5.4), sauf si elle a
        // perdu ses capacités.
        if !skip_self_mod && o.proxy_of.is_none() && !self.lost_abilities(id) {
            if let Some(f) = self.db.get(o.card).self_mod {
                add(f(self, id));
            }
        }
        // Effets statiques d'une arme sur ses attaques (« this card's attacks
        // get … ») : évalués en continu sur l'attaque-proxy (CR 1.4.3).
        if let Some(w) = o.proxy_of {
            if !self.lost_abilities(w) {
                if let Some(f) = self.db.get(self.obj(w).card).attack_mod {
                    add(f(self, w));
                }
            }
        }
        // Effets statiques des héros, équipements et permanents (CR 1.7.4).
        for src in self.static_sources() {
            if let Some(f) = self.def(src).aura {
                add(f(self, src, id));
            }
        }
        d
    }

    /// L'objet a-t-il perdu toutes ses capacités (« loses all abilities ») ?
    ///
    /// CR 6.3.6 : il ne perd que ses capacités propres (imprimées) ; celles
    /// qu'un autre effet lui donne restent (ex. go again accordé).
    /// TODO(règles) CR 6.3.4 : on ne tient pas compte de l'ordre chronologique
    /// (une capacité accordée *avant* la perte devrait peut-être disparaître).
    #[inline]
    pub fn lost_abilities(&self, id: ObjectId) -> bool {
        if !self.any_lose_abilities {
            return false;
        }
        let r = self.obj_ref(id);
        self.modifiers
            .iter()
            .any(|m| m.delta.lose_abilities && m.target == r)
    }

    /// Objets dont les effets statiques « aura » sont actifs.
    fn static_sources(&self) -> impl Iterator<Item = ObjectId> + '_ {
        let heroes = self.players.iter().map(|p| p.hero);
        let equip = self
            .players
            .iter()
            .flat_map(|p| p.equipment().chain(p.weapons.iter().copied()));
        // Un allié qui attaque est sur la chaîne de combat, toujours dans
        // l'arène : ses capacités statiques fonctionnent (ex. Miragai).
        heroes
            .chain(equip)
            .chain(self.permanents.iter().copied())
            .chain(self.attacking_allies())
            .filter(|&s| self.db.get(self.obj(s).card).aura.is_some() && !self.lost_abilities(s))
    }

    /// Puissance actuelle (CR 2.9.3, jamais négative CR 2.0.3c).
    /// Une proxy hérite de la puissance modifiée de son arme (CR 1.4.3a, 1.4.3d).
    pub fn power(&self, id: ObjectId) -> i16 {
        let o = self.obj(id);
        let base = match o.proxy_of {
            Some(w) => self.power(w),
            None => self.db.get(o.card).power.unwrap_or(0) as i16 + o.power_counters as i16,
        };
        (base + self.delta_with(id, self.cant_gain_power(id), false).power).max(0)
    }

    /// Chokeslam : « attack action cards they control can't gain {p} during
    /// their next action phase ».
    pub fn cant_gain_power(&self, id: ObjectId) -> bool {
        if self.no_power_gain.is_empty() || !self.action_phase {
            return false;
        }
        let o = self.obj(id);
        let def = self.db.get(o.card);
        o.proxy_of.is_none()
            && def.is_action()
            && def.is_attack_card()
            && self
                .no_power_gain
                .iter()
                .any(|&(p, t)| p == o.controller && t == self.turn)
    }

    /// Défense actuelle (CR 2.3.3, jamais négative CR 2.0.3c).
    pub fn defense(&self, id: ObjectId) -> i16 {
        let o = self.obj(id);
        let base = self.db.get(o.card).defense.unwrap_or(0) as i16 + o.defense_counters as i16;
        (base + self.continuous_delta(id).defense).max(0)
    }

    /// Mots-clés actuels (de base + accordés par des effets).
    pub fn keywords(&self, id: ObjectId) -> Keywords {
        self.keywords_with(id, false)
    }

    /// Mots-clés en ignorant l'effet statique `self_mod` de la carte elle-même.
    /// Sert à un `self_mod` qui dépend des mots-clés (« if this has go again,
    /// it gets +1{p} ») sans boucler : keywords → delta → self_mod → keywords.
    pub fn keywords_sans_self_mod(&self, id: ObjectId) -> Keywords {
        self.keywords_with(id, true)
    }

    fn keywords_with(&self, id: ObjectId, skip_self_mod: bool) -> Keywords {
        let o = self.obj(id);
        let base = match o.proxy_of {
            // Une proxy n'hérite pas des capacités de résolution de sa source
            // (CR 1.4.3a) : son « go again » vient de la capacité d'attaque,
            // fixé à sa création (modificateur). Les mots-clés statiques de
            // l'arme (ex. dominate) sont hérités.
            Some(w) => self.keywords(w).without(Keywords::GO_AGAIN),
            None if self.lost_abilities(id) => self.material_keywords(id),
            None => self.db.get(o.card).keywords | self.material_keywords(id),
        };
        let d = self.delta_with(id, false, skip_self_mod);
        // « Loses and can't gain » l'emporte sur les mots-clés accordés.
        (base | d.grants).without(d.loses)
    }

    /// Valeurs actuelles des préventions au choix d'un objet :
    /// (arcane barrier, ward, spellvoid), CR 8.3.8, 8.3.20, 8.3.15.
    pub fn prevention_values(&self, id: ObjectId) -> (u8, u8, u8) {
        let def = self.def(id);
        let printed = if self.lost_abilities(id) {
            (0, 0, 0)
        } else {
            (def.arcane_barrier, def.ward, def.spellvoid)
        };
        let d = self.continuous_delta(id);
        (
            printed.0 + d.arcane_barrier,
            printed.1 + d.ward,
            printed.2 + d.spellvoid,
        )
    }

    /// Total de vie d'un objet vivant : héros (CR 2.5.3) ou allié. `None` si
    /// l'objet n'a pas la propriété vie.
    pub fn life_total(&self, id: ObjectId) -> Option<i32> {
        let o = self.obj(id);
        if self.player(o.owner).hero == id {
            return Some(self.player(o.owner).life);
        }
        let base = self.def(id).life? as i32;
        Some((base + o.life_counters as i32 + o.life_delta as i32).max(0))
    }

    /// CR 2.5.1a : un permanent qui a la propriété vie (allié…), hors héros.
    /// Un allié qui attaque (sur la chaîne de combat) reste un permanent.
    pub fn is_living_permanent(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        self.def(id).life.is_some()
            && (o.zone == Zone::Permanent
                || (o.zone == Zone::CombatChain
                    && o.proxy_of.is_none()
                    && self.combat.links.iter().any(|l| l.source == Some(id))))
    }

    /// CR 1.4.5a : l'objet peut-il être attaqué ? (permanent vivant, ou spectra)
    pub fn is_attackable_object(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        if !o.zone.in_arena() || o.zone == Zone::Hero || o.face_down {
            return false;
        }
        // Spectra imprimé seulement (aucune carte des 9 decks ne le donne).
        self.is_living_permanent(id)
            || (self.def(id).keywords.has(Keywords::SPECTRA) && !self.lost_abilities(id))
    }

    /// Les cibles qu'une attaque du joueur `p` peut déclarer (CR 1.4.5) : le
    /// héros adverse et les objets attaquables qu'un adversaire contrôle.
    pub fn attack_targets(&self, p: PlayerId) -> Vec<Target> {
        let opp = p.opponent();
        let mut v = vec![Target::Player(opp)];
        for &id in &self.permanents {
            if self.obj(id).controller == opp && self.is_attackable_object(id) {
                v.push(Target::Object(self.obj_ref(id)));
            }
        }
        v
    }

    /// Une cible d'attaque est-elle encore légale (CR 7.2.2c, 7.5.2b) ?
    pub fn attack_target_legal(&self, attacker: PlayerId, t: Target) -> bool {
        match t {
            Target::Player(p) => p != attacker,
            Target::Object(r) => {
                self.is_current(r)
                    && self.obj(r.id).controller != attacker
                    && self.is_attackable_object(r.id)
            }
        }
    }

    /// Le joueur qui contrôle une cible (héros ou objet).
    pub fn target_controller(&self, t: Target) -> PlayerId {
        match t {
            Target::Player(p) => p,
            Target::Object(r) => self.obj(r.id).controller,
        }
    }

    /// Numéro du prochain tour du joueur `p` (« their next turn »), partie à 2.
    pub fn next_turn_of(&self, p: PlayerId) -> u32 {
        if p == self.turn_player {
            self.turn + 2
        } else {
            self.turn + 1
        }
    }

    /// Durée « until the end of their next turn ».
    pub fn end_of_next_turn(&self, p: PlayerId) -> Until {
        Until::EndOfTurnNumber(self.next_turn_of(p))
    }

    /// Le dessus du deck d'un joueur.
    pub fn top_of_deck(&self, p: PlayerId) -> Option<ObjectId> {
        self.player(p).deck.last().copied()
    }
}

/// Zone d'équipement correspondant au sous-type d'une carte (CR 3.2-3.16).
pub fn equipment_zone(def: &CardDef) -> Zone {
    if def.types.has(Types::WEAPON) || def.subtypes.any(Subtypes::OFF_HAND | Subtypes::QUIVER) {
        Zone::Weapon
    } else if def.subtypes.has(Subtypes::HEAD) {
        Zone::Head
    } else if def.subtypes.has(Subtypes::CHEST) {
        Zone::Chest
    } else if def.subtypes.has(Subtypes::ARMS) {
        Zone::Arms
    } else if def.subtypes.has(Subtypes::LEGS) {
        Zone::Legs
    } else {
        Zone::Permanent
    }
}
