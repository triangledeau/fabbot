//! Solveur exact d'un tour (épreuve A de `docs/epreuves.md`).
//!
//! Pendant son propre tour, un joueur ne découvre presque rien de nouveau
//! (la pioche n'arrive qu'en fin de tour). Si l'on **fixe la défense** de
//! l'adversaire (une règle de jeu déterministe pour toutes ses décisions),
//! le tour devient un problème à un seul joueur, entièrement déterminé : on
//! peut **énumérer toutes les façons de le jouer** (cartes, ordre, pitch,
//! arme, cibles) et trouver la meilleure.
//!
//! Objectif d'un puzzle : **tuer** l'adversaire ce tour si c'est possible,
//! sinon lui faire **perdre le plus de points de vie**. La recherche
//! s'arrête à la fin du tour : à la décision d'arsenal ou d'ordre du pitch
//! (CR 4.4.3b-c, après les déclenchements de début de phase de fin), au
//! changement de tour ou à la fin de la partie.
//!
//! Défenses proposées ([`Defense`]) :
//! - **passive** : l'adversaire ne défend avec aucune carte et ne joue rien
//!   (« Pass » à chaque déclaration de défense et à chaque priorité) ; ses
//!   autres choix (payer une arcane barrier, ordre des déclenchements…) sont
//!   faits par B1 ;
//! - **B1** : toutes les décisions de l'adversaire sont prises par B1
//!   (qui ne lit que l'information de son joueur).
//!
//! Le monde est celui de la vraie partie (vraie main adverse, vrai ordre des
//! decks, même générateur aléatoire) : le solveur sait donc ce que le joueur
//! ne sait pas. C'est voulu : un puzzle a une solution exacte. Le bot testé,
//! lui, joue avec son information ordinaire (il imagine la main adverse).
//!
//! Réductions (sans perte, sauf mention) :
//! - deux options de même **clé** (deux exemplaires identiques en main) ne
//!   sont essayées qu'une fois (même clé que la recherche de B2) ;
//! - pour un même paiement, l'**ordre** dans lequel on pitche les cartes
//!   n'est essayé qu'une fois par ensemble de cartes pitchées. Seul effet
//!   perdu : l'ordre des cartes dans la zone de pitch (sans importance avant
//!   la fin du tour) et l'ordre de capacités « quand cette carte est
//!   pitchée » éventuelles ;
//! - au-delà d'un nombre de noeuds (`budget`), la recherche s'arrête : le
//!   puzzle est marqué **incomplet** (meilleure ligne trouvée jusque-là).

use crate::{Agent, HeuristicAgent};
use fab_engine::{
    Choice, Decision, DecisionKind, GameState, ObjectId, Outcome, PlayerId, PreventRef, Target,
    Zone,
};
use std::collections::HashSet;

/// La défense fixée de l'adversaire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Defense {
    /// Ne défend pas, ne joue rien (autres choix : B1).
    Passive,
    /// Toutes ses décisions par B1.
    B1,
}

impl Defense {
    pub fn depuis_nom(nom: &str) -> Option<Defense> {
        match nom {
            "passive" => Some(Defense::Passive),
            "b1" => Some(Defense::B1),
            _ => None,
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Defense::Passive => "passive",
            Defense::B1 => "b1",
        }
    }

    /// Le choix de l'adversaire pour la décision `d`.
    pub fn choisir(self, b1: &mut HeuristicAgent, g: &GameState, d: &Decision) -> usize {
        if self == Defense::Passive
            && matches!(d.kind, DecisionKind::Defend | DecisionKind::Priority)
        {
            if let Some(i) = d.index_of(Choice::Pass) {
                return i;
            }
        }
        b1.choose(g, d)
    }
}

/// Résultat d'un tour joué (par le solveur ou par un bot).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResultatTour {
    /// L'adversaire est mort pendant le tour.
    pub letal: bool,
    /// Points de vie perdus par l'adversaire pendant le tour (net).
    pub degats: i32,
    /// Les choix du joueur actif (index dans les options), dans l'ordre :
    /// permet de rejouer la ligne avec [`rejouer`].
    pub ligne: Vec<usize>,
}

impl ResultatTour {
    /// Note à maximiser : le létal d'abord, puis les dégâts.
    fn note(&self) -> (bool, i32) {
        (self.letal, self.degats)
    }
}

/// Résultat du solveur.
#[derive(Clone, Debug)]
pub struct Solution {
    pub meilleur: ResultatTour,
    /// Noeuds explorés (décisions du joueur actif essayées).
    pub noeuds: u64,
    /// La recherche a tout exploré (sinon, budget atteint).
    pub complet: bool,
}

/// La position est-elle terminée pour le puzzle ?
fn fin_du_tour(g: &GameState, tour0: u32) -> bool {
    match &g.pending {
        None => true,
        Some(d) => {
            g.outcome.is_some()
                || g.turn != tour0
                || matches!(d.kind, DecisionKind::Arsenal | DecisionKind::PitchOrder)
        }
    }
}

fn mesurer(g: &GameState, actif: PlayerId, vie0: i32) -> (bool, i32) {
    let adv = actif.opponent();
    let letal = matches!(g.outcome, Some(Outcome::Winner(p)) if p == actif);
    (letal, vie0 - g.player(adv).life)
}

/// Fait avancer la partie tant que ce n'est pas au joueur actif de choisir
/// entre plusieurs options (la défense joue pour l'adversaire).
fn avancer(
    g: &mut GameState,
    actif: PlayerId,
    defense: Defense,
    b1: &mut HeuristicAgent,
    tour0: u32,
) {
    while !fin_du_tour(g, tour0) {
        let d = g.pending.as_ref().expect("décision");
        let i = if d.options.len() == 1 {
            0
        } else if d.player != actif {
            let d = d.clone();
            defense.choisir(b1, g, &d)
        } else {
            return;
        };
        let _ = g.apply(i);
    }
}

/// Joue le tour du joueur qui a la décision en attente dans `g` (au début de
/// sa phase d'action, en principe) avec `agent`, contre `defense`.
pub fn jouer_tour(g: &GameState, agent: &mut dyn Agent, defense: Defense) -> ResultatTour {
    let mut g = g.clone();
    let actif = g
        .pending
        .as_ref()
        .map(|d| d.player)
        .unwrap_or(g.turn_player);
    let vie0 = g.player(actif.opponent()).life;
    let tour0 = g.turn;
    let mut b1 = HeuristicAgent::new();
    let mut ligne = Vec::new();
    loop {
        avancer(&mut g, actif, defense, &mut b1, tour0);
        if fin_du_tour(&g, tour0) {
            break;
        }
        let d = g.pending.clone().expect("décision");
        let i = agent.choose(&g, &d);
        ligne.push(i);
        let _ = g.apply(i);
    }
    let (letal, degats) = mesurer(&g, actif, vie0);
    ResultatTour {
        letal,
        degats,
        ligne,
    }
}

/// Rejoue une ligne (index des choix du joueur actif) et renvoie, pour
/// chaque choix, la position avant le choix et la décision : sert à décrire
/// la ligne en clair.
pub fn rejouer(
    g: &GameState,
    ligne: &[usize],
    defense: Defense,
) -> Vec<(GameState, Decision, usize)> {
    let mut g = g.clone();
    let actif = g
        .pending
        .as_ref()
        .map(|d| d.player)
        .unwrap_or(g.turn_player);
    let tour0 = g.turn;
    let mut b1 = HeuristicAgent::new();
    let mut v = Vec::new();
    for &i in ligne {
        avancer(&mut g, actif, defense, &mut b1, tour0);
        if fin_du_tour(&g, tour0) {
            break;
        }
        let d = g.pending.clone().expect("décision");
        v.push((g.clone(), d, i));
        let _ = g.apply(i);
    }
    v
}

struct Recherche {
    actif: PlayerId,
    defense: Defense,
    b1: HeuristicAgent,
    vie0: i32,
    tour0: u32,
    budget: u64,
    noeuds: u64,
    meilleur: Option<ResultatTour>,
    ligne: Vec<usize>,
    /// Ensembles de cartes déjà pitchées pour un même paiement :
    /// (numéro du paiement, clés triées).
    pitchs_vus: HashSet<(u64, Vec<u64>)>,
    paiements: u64,
}

impl Recherche {
    fn termine(&self) -> bool {
        self.meilleur.as_ref().is_some_and(|m| m.letal) || self.noeuds >= self.budget
    }

    /// `pitch` : le paiement en cours (numéro, clés pitchées jusqu'ici).
    fn explorer(&mut self, mut g: GameState, pitch: Option<(u64, Vec<u64>)>) {
        avancer(&mut g, self.actif, self.defense, &mut self.b1, self.tour0);
        let en_pitch = !fin_du_tour(&g, self.tour0)
            && g.pending
                .as_ref()
                .is_some_and(|d| d.kind == DecisionKind::Pitch);
        // Fin d'un paiement : cet ensemble de cartes pitchées a-t-il déjà été vu ?
        let pitch = match pitch {
            Some((n, mut cles)) if !en_pitch => {
                cles.sort_unstable();
                if !self.pitchs_vus.insert((n, cles)) {
                    return;
                }
                None
            }
            p => p,
        };
        if fin_du_tour(&g, self.tour0) {
            let (letal, degats) = mesurer(&g, self.actif, self.vie0);
            let r = ResultatTour {
                letal,
                degats,
                ligne: self.ligne.clone(),
            };
            if self.meilleur.as_ref().is_none_or(|m| r.note() > m.note()) {
                self.meilleur = Some(r);
            }
            return;
        }
        let d = g.pending.clone().expect("décision");
        let mut options: Vec<(u64, usize)> = Vec::new();
        for (i, &c) in d.options.iter().enumerate() {
            let k = cle(&g, &d, c);
            if !options.iter().any(|&(x, _)| x == k) {
                options.push((k, i));
            }
        }
        let pitch = match pitch {
            Some(p) => Some(p),
            None if en_pitch => {
                self.paiements += 1;
                Some((self.paiements, Vec::new()))
            }
            None => None,
        };
        for (k, i) in options {
            if self.termine() {
                return;
            }
            self.noeuds += 1;
            let mut suite = g.clone();
            let _ = suite.apply(i);
            self.ligne.push(i);
            let p = pitch.clone().map(|(n, mut cles)| {
                cles.push(k);
                (n, cles)
            });
            self.explorer(suite, p);
            self.ligne.pop();
        }
    }
}

/// Cherche la meilleure façon de jouer le tour du joueur qui a la décision
/// en attente dans `g`, contre `defense`, en explorant au plus `budget`
/// noeuds.
pub fn resoudre(g: &GameState, defense: Defense, budget: u64) -> Solution {
    let actif = g
        .pending
        .as_ref()
        .map(|d| d.player)
        .unwrap_or(g.turn_player);
    let mut r = Recherche {
        actif,
        defense,
        b1: HeuristicAgent::new(),
        vie0: g.player(actif.opponent()).life,
        tour0: g.turn,
        budget,
        noeuds: 0,
        meilleur: None,
        ligne: Vec::new(),
        pitchs_vus: HashSet::new(),
        paiements: 0,
    };
    r.explorer(g.clone(), None);
    let complet = r.noeuds < budget || r.meilleur.as_ref().is_some_and(|m| m.letal);
    Solution {
        meilleur: r.meilleur.unwrap_or_default(),
        noeuds: r.noeuds,
        complet,
    }
}

/// Clé d'un objet : la carte et sa zone pour une carte « neuve » hors de
/// l'arène (les exemplaires identiques sont interchangeables), l'objet
/// lui-même sinon (même règle que la recherche de B2).
fn cle_objet(g: &GameState, id: ObjectId) -> u64 {
    let o = g.obj(id);
    match o.zone {
        Zone::Hand | Zone::Deck | Zone::Arsenal | Zone::Pitch | Zone::Graveyard => {
            (1 << 47) | ((o.card.0 as u64) << 8) | ((o.owner.0 as u64) << 6) | o.zone as u64
        }
        _ => id.0 as u64,
    }
}

/// Clé d'une option : deux options de même clé mènent à des positions
/// équivalentes.
fn cle(g: &GameState, d: &Decision, c: Choice) -> u64 {
    let (tag, extra, contenu): (u64, u64, u64) = match c {
        Choice::Pass => (0, 0, 0),
        Choice::Yes => (1, 0, 0),
        Choice::No => (2, 0, 0),
        Choice::Play(o) => (3, 0, cle_objet(g, o)),
        Choice::Activate(o, i) => (4, i as u64, cle_objet(g, o)),
        Choice::Pitch(o) => (5, 0, cle_objet(g, o)),
        Choice::Defend(o) => (6, 0, cle_objet(g, o)),
        Choice::Arsenal(o) => (7, 0, cle_objet(g, o)),
        Choice::PitchToBottom(o) => (8, 0, cle_objet(g, o)),
        Choice::Player(p) => (9, 0, p.0 as u64),
        Choice::Target(Target::Player(p)) => (10, 0, p.0 as u64),
        Choice::Target(Target::Object(r)) => (10, 1, cle_objet(g, r.id)),
        Choice::Object(o) => (11, 0, cle_objet(g, o)),
        Choice::Number(n) => (12, 0, n as u64),
        Choice::Mode(m) => (13, 0, m as u64),
        Choice::Trigger(t) => (14, 0, t as u64),
        Choice::Prevent(PreventRef::Effect(e)) => (15, 0, e as u64),
        Choice::Prevent(PreventRef::ArcaneBarrier(o)) => (15, 1, cle_objet(g, o)),
        Choice::Prevent(PreventRef::Ward(o)) => (15, 2, cle_objet(g, o)),
        Choice::Prevent(PreventRef::Spellvoid(o)) => (15, 3, cle_objet(g, o)),
        Choice::Prevent(PreventRef::Static(o)) => (15, 4, cle_objet(g, o)),
        Choice::Prevent(PreventRef::Counter(o)) => (15, 5, cle_objet(g, o)),
        Choice::Zone(p, z) => (16, 0, ((p.0 as u64) << 8) | z as u64),
    };
    ((d.player.0 as u64) << 63)
        | ((d.kind as u64) << 58)
        | (tag << 53)
        | ((extra & 31) << 48)
        | (contenu & ((1 << 48) - 1))
}

/// Un choix en clair (« joue Command and Conquer (red) », « pitche … »).
pub fn decrire_choix(g: &GameState, c: Choice) -> String {
    let nom = |id: ObjectId| {
        let o = g.obj(id);
        let id = o.proxy_of.unwrap_or(id);
        crate::side::nom_carte(&g.db, g.obj(id).card)
    };
    match c {
        Choice::Pass => "passe".into(),
        Choice::Yes => "oui".into(),
        Choice::No => "non".into(),
        Choice::Play(o) => {
            let de = if g.obj(o).zone == Zone::Arsenal {
                " (arsenal)"
            } else {
                ""
            };
            format!("joue {}{de}", nom(o))
        }
        Choice::Activate(o, i) => format!("active {} [{i}]", nom(o)),
        Choice::Pitch(o) => format!("pitche {}", nom(o)),
        Choice::Defend(o) => format!("défend avec {}", nom(o)),
        Choice::Arsenal(o) => format!("arsenal {}", nom(o)),
        Choice::PitchToBottom(o) => format!("sous le deck {}", nom(o)),
        Choice::Target(Target::Player(p)) | Choice::Player(p) => format!("joueur {}", p.0 + 1),
        Choice::Target(Target::Object(r)) => format!("cible {}", nom(r.id)),
        Choice::Object(o) => format!("choisit {}", nom(o)),
        Choice::Number(n) => format!("X = {n}"),
        Choice::Mode(m) => format!("mode {}", m + 1),
        Choice::Trigger(t) => format!("déclenchement {}", t + 1),
        Choice::Prevent(_) => "prévention".into(),
        Choice::Zone(p, z) => format!("zone {z:?} du joueur {}", p.0 + 1),
    }
}

/// Une ligne en clair, choix séparés par « , » ; plusieurs « passe » de
/// suite sont regroupés (« passe ×3 »).
pub fn decrire_ligne(g: &GameState, ligne: &[usize], defense: Defense) -> String {
    let mut morceaux: Vec<(String, u32)> = Vec::new();
    for (g, d, i) in rejouer(g, ligne, defense) {
        let t = decrire_choix(&g, d.options[i]);
        match morceaux.last_mut() {
            Some((dernier, n)) if *dernier == t && t == "passe" => *n += 1,
            _ => morceaux.push((t, 1)),
        }
    }
    morceaux
        .into_iter()
        .map(|(t, n)| if n > 1 { format!("{t} ×{n}") } else { t })
        .collect::<Vec<_>>()
        .join(", ")
}
