//! B2 : recherche ISMCTS (Information Set Monte Carlo Tree Search).
//!
//! Pour chaque décision qui a plusieurs options, B2 répète un grand nombre
//! de fois (les **itérations**) :
//! 1. **imaginer un monde possible** : une copie de la partie où la main
//!    adverse et l'ordre des decks sont tirés au hasard parmi ce qu'il ne
//!    voit pas ([`crate::determinisation`]) ;
//! 2. **descendre dans l'arbre** des décisions déjà explorées (les siennes et
//!    celles de l'adversaire), en choisissant à chaque fois l'option la plus
//!    prometteuse (formule UCB : bon score moyen, ou peu essayée) parmi les
//!    options légales **dans ce monde** ;
//! 3. **ajouter** une option jamais essayée à l'arbre ;
//! 4. **finir la partie** très vite avec un bot simple (B1 par défaut) pour
//!    les deux joueurs, et noter le résultat (victoire 1, nulle ½, défaite 0) ;
//! 5. **remonter** ce résultat dans chaque décision du chemin.
//!
//! À la fin, il joue l'option de la racine la plus souvent choisie.
//!
//! **Réglages par défaut** (mesurés, voir `docs/bots.md`) : 300 itérations,
//! arbre limité à la décision présente (`profondeur = 1` : les décisions
//! suivantes sont jouées par B1), parties simulées arrêtées après 4 tours
//! puis estimées (vies + cartes en main). Avec peu d'itérations, un arbre
//! plus profond fait jouer l'adversaire imaginé moins bien que B1, et B1
//! joue mal la fin de partie de certains decks : les deux font perdre des
//! points.
//!
//! C'est la variante « un seul observateur » (SO-ISMCTS, Cowling, Powley et
//! Whitehouse, 2012) : un seul arbre, vu depuis l'information de B2. Comme
//! les options changent d'un monde à l'autre (la main adverse change), on
//! compte pour chaque nœud combien de fois il était **disponible** et on
//! utilise ce nombre dans la formule UCB à la place des visites du parent.
//!
//! Une option est reconnue d'un monde à l'autre par une **clé** : pour une
//! carte en main, dans le deck, l'arsenal, le pitch ou le cimetière, la clé
//! est la carte (nom + couleur) et sa zone, pas l'objet ; deux exemplaires
//! identiques en main donnent donc la même option. Pour un objet de l'arène
//! ou de la pile, la clé est l'objet lui-même.
//!
//! Information : B2 ne lit l'information cachée que pour la **redistribuer**
//! au hasard (voir les approximations dans [`crate::determinisation`]) ; ses
//! simulations se déroulent dans les mondes imaginés, jamais dans la partie
//! réelle.

use crate::determinisation::determiniser;
use crate::{Agent, HeuristicAgent, RandomAgent};
use fab_engine::{
    Choice, Decision, GameState, ObjectId, Outcome, PlayerId, PreventRef, Rng, Target, Zone,
};
use std::time::{Duration, Instant};

/// Le bot qui finit les parties simulées (pour les deux joueurs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Politique {
    /// B0 : au hasard (rapide, mais parties peu réalistes).
    Aleatoire,
    /// B1 : heuristique.
    Heuristique,
}

/// Réglages de B2.
#[derive(Clone, Copy, Debug)]
pub struct ReglagesB2 {
    /// Nombre d'itérations (mondes imaginés et parties simulées) par décision.
    pub iterations: u32,
    /// Durée maximale de réflexion par décision (`None` : seulement le nombre
    /// d'itérations). Avec une durée, les résultats ne sont plus
    /// reproductibles à l'identique (ils dépendent de la vitesse de la machine).
    pub duree_max: Option<Duration>,
    /// Constante d'exploration de la formule UCB (plus grande : essaie plus
    /// souvent les options peu explorées).
    pub exploration: f32,
    /// Bot qui finit les parties simulées.
    pub politique: Politique,
    /// Arrêter les parties simulées après ce nombre de tours et estimer le
    /// résultat (vies et cartes) ; `None` : jouer jusqu'à la fin.
    pub tours_simules: Option<u32>,
    /// Nombre maximal de décisions (à plusieurs options) dans l'arbre, la
    /// racine comprise ; au-delà, la politique joue. 1 = seule la décision
    /// présente est explorée (« Monte Carlo à plat »).
    pub profondeur: u32,
    /// Estimation d'une partie simulée arrêtée : valeur d'une carte en main
    /// ou en arsenal, en points de vie.
    pub valeur_carte: f32,
}

impl Default for ReglagesB2 {
    fn default() -> Self {
        ReglagesB2 {
            iterations: 300,
            duree_max: None,
            exploration: 0.4,
            politique: Politique::Heuristique,
            tours_simules: Some(4),
            profondeur: 1,
            valeur_carte: 1.5,
        }
    }
}

impl ReglagesB2 {
    /// Lit des réglages écrits après « b2- » : un nombre d'itérations
    /// (`1000`) ou une durée (`500ms`), puis des options : `t4` (parties
    /// simulées arrêtées après 4 tours), `a` (parties simulées par B0),
    /// `c0.5` (constante d'exploration), `p2` (profondeur de l'arbre ; `p0` :
    /// sans limite), `v3` (valeur d'une carte pour l'estimation), `t0` :
    /// parties simulées jusqu'à la fin.
    /// Ex. : `300t6`, `500ms`, `200c1.2`, `400p1`.
    pub fn depuis_nom(texte: &str) -> Option<Self> {
        let mut r = ReglagesB2::default();
        let nombre = |s: &str| -> (String, usize) {
            let n: String = s
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let l = n.len();
            (n, l)
        };
        let (n, l) = nombre(texte);
        let mut reste = &texte[l..];
        if let Some(apres) = reste.strip_prefix("ms") {
            r.duree_max = Some(Duration::from_millis(n.parse().ok()?));
            r.iterations = u32::MAX;
            reste = apres;
        } else {
            r.iterations = n.parse().ok().filter(|&n| n > 0)?;
        }
        while let Some(c) = reste.chars().next() {
            reste = &reste[1..];
            match c {
                'a' => r.politique = Politique::Aleatoire,
                't' | 'c' | 'p' | 'v' => {
                    let (n, l) = nombre(reste);
                    reste = &reste[l..];
                    match c {
                        't' => r.tours_simules = Some(n.parse().ok()?).filter(|&t| t > 0),
                        'p' => {
                            r.profondeur =
                                Some(n.parse().ok()?).filter(|&p| p > 0).unwrap_or(u32::MAX)
                        }
                        'v' => r.valeur_carte = n.parse().ok()?,
                        _ => r.exploration = n.parse().ok()?,
                    }
                }
                _ => return None,
            }
        }
        Some(r)
    }
}

/// Un nœud de l'arbre : une option choisie dans un certain contexte.
#[derive(Clone, Debug)]
struct Noeud {
    /// Clé de l'option qui mène à ce nœud (voir [`cle`]).
    cle: u64,
    /// Joueur qui a choisi cette option (0 ou 1).
    joueur: u8,
    visites: u32,
    /// Nombre de fois où cette option était légale quand on est passé par
    /// le nœud parent.
    disponible: u32,
    /// Somme des résultats, du point de vue de `joueur`.
    gains: f32,
    enfants: Vec<u32>,
}

impl Noeud {
    fn new(cle: u64, joueur: u8) -> Self {
        Noeud {
            cle,
            joueur,
            visites: 0,
            disponible: 0,
            gains: 0.0,
            enfants: Vec::new(),
        }
    }
}

/// B2 : recherche ISMCTS.
pub struct IsmctsAgent {
    pub reglages: ReglagesB2,
    rng: Rng,
    b1: HeuristicAgent,
    arbre: Vec<Noeud>,
    /// Statistiques : itérations faites depuis la création du bot.
    pub iterations_faites: u64,
    /// Statistiques : décisions où une recherche a eu lieu.
    pub recherches: u64,
}

impl IsmctsAgent {
    pub fn new(graine: u64) -> Self {
        Self::avec(graine, ReglagesB2::default())
    }

    pub fn avec(graine: u64, reglages: ReglagesB2) -> Self {
        IsmctsAgent {
            reglages,
            rng: Rng::new(graine),
            b1: HeuristicAgent::new(),
            arbre: Vec::new(),
            iterations_faites: 0,
            recherches: 0,
        }
    }

    /// Une itération : un monde imaginé, une descente, une partie simulée.
    fn iteration(&mut self, racine: &GameState, moi: PlayerId, cles: &mut Vec<(u64, usize)>) {
        let mut g = determiniser(racine, moi, &mut self.rng);
        let tour0 = g.turn;
        let mut chemin: Vec<u32> = Vec::with_capacity(32);
        chemin.push(0);
        let mut noeud = 0u32;
        let mut ajoute = false;
        let mut a_essayer: Vec<(u64, usize)> = Vec::new();
        while let Some(d) = g.pending.as_ref() {
            if d.options.len() == 1 {
                let _ = g.apply(0);
                continue;
            }
            if ajoute || chemin.len() as u32 > self.reglages.profondeur {
                break;
            }
            options_par_cle(&g, d, cles);
            a_essayer.clear();
            let mut meilleur: Option<(f32, u32, usize)> = None;
            let c = self.reglages.exploration;
            for &(k, i) in cles.iter() {
                let enfant = self.arbre[noeud as usize]
                    .enfants
                    .iter()
                    .copied()
                    .find(|&e| self.arbre[e as usize].cle == k);
                match enfant {
                    None => a_essayer.push((k, i)),
                    Some(e) => {
                        let n = &mut self.arbre[e as usize];
                        n.disponible += 1;
                        let v = n.visites.max(1) as f32;
                        let ucb = n.gains / v + c * ((n.disponible as f32).ln() / v).sqrt();
                        if meilleur.is_none_or(|(m, _, _)| ucb > m) {
                            meilleur = Some((ucb, e, i));
                        }
                    }
                }
            }
            let joueur = d.player.0;
            let (suivant, i) = if a_essayer.is_empty() {
                let (_, e, i) = meilleur.expect("au moins une option");
                (e, i)
            } else {
                let (k, i) = a_essayer[self.rng.below(a_essayer.len() as u32) as usize];
                let e = self.arbre.len() as u32;
                let mut n = Noeud::new(k, joueur);
                n.disponible = 1;
                self.arbre.push(n);
                self.arbre[noeud as usize].enfants.push(e);
                ajoute = true;
                (e, i)
            };
            let _ = g.apply(i);
            noeud = suivant;
            chemin.push(noeud);
        }
        let r = self.simuler(&mut g, tour0);
        for &n in &chemin {
            let n = &mut self.arbre[n as usize];
            n.visites += 1;
            n.gains += if n.joueur == 0 { r } else { 1.0 - r };
        }
        self.iterations_faites += 1;
    }

    /// Finit la partie avec la politique ; renvoie le résultat pour le joueur 0.
    fn simuler(&mut self, g: &mut GameState, tour0: u32) -> f32 {
        let mut aleatoire = RandomAgent::new(self.rng.next_u64());
        let mut n = 0u32;
        while let Some(d) = g.pending.as_ref() {
            if let Some(t) = self.reglages.tours_simules {
                if g.turn >= tour0 + t {
                    return estimer(g, self.reglages.valeur_carte);
                }
            }
            let i = if d.options.len() == 1 {
                0
            } else {
                match self.reglages.politique {
                    Politique::Heuristique => self.b1.choose(g, d),
                    Politique::Aleatoire => aleatoire.choose(g, d),
                }
            };
            let _ = g.apply(i);
            n += 1;
            if n > 100_000 {
                break;
            }
        }
        match g.outcome {
            Some(Outcome::Winner(p)) if p == PlayerId::P0 => 1.0,
            Some(Outcome::Winner(_)) => 0.0,
            _ => 0.5,
        }
    }
}

impl Agent for IsmctsAgent {
    fn name(&self) -> &str {
        "B2-ismcts"
    }

    fn choose(&mut self, state: &GameState, decision: &Decision) -> usize {
        if decision.options.len() <= 1 {
            return 0;
        }
        let mut cles = Vec::new();
        options_par_cle(state, decision, &mut cles);
        if cles.len() == 1 {
            return cles[0].1;
        }
        self.recherches += 1;
        self.arbre.clear();
        self.arbre.push(Noeud::new(0, 0));
        let debut = Instant::now();
        let moi = decision.player;
        let mut tampon = Vec::new();
        for k in 0..self.reglages.iterations.max(1) {
            if let Some(max) = self.reglages.duree_max {
                // Mesurer l'heure coûte peu, mais pas rien : toutes les 8 itérations.
                if k % 8 == 0 && debut.elapsed() >= max {
                    break;
                }
            }
            self.iteration(state, moi, &mut tampon);
        }
        // L'option de la racine la plus visitée (à égalité : meilleur score).
        let racine = &self.arbre[0];
        let mut meilleur: Option<(u32, f32, usize)> = None;
        for &(k, i) in &cles {
            if let Some(&e) = racine
                .enfants
                .iter()
                .find(|&&e| self.arbre[e as usize].cle == k)
            {
                let n = &self.arbre[e as usize];
                let moy = n.gains / n.visites.max(1) as f32;
                if meilleur.is_none_or(|(v, m, _)| n.visites > v || (n.visites == v && moy > m)) {
                    meilleur = Some((n.visites, moy, i));
                }
            }
        }
        match meilleur {
            Some((_, _, i)) => i,
            None => self.b1.choose(state, decision),
        }
    }
}

/// Estimation du résultat d'une partie simulée arrêtée avant la fin, pour le
/// joueur 0 : écart de vie et de cartes disponibles, ramené entre 0 et 1.
fn estimer(g: &GameState, valeur_carte: f32) -> f32 {
    let p = |i: usize| {
        let ps = &g.players[i];
        let cartes = ps.hand.len() + ps.arsenal.is_some() as usize;
        ps.life as f32 + valeur_carte * cartes as f32 + 0.25 * ps.deck.len() as f32
    };
    let ecart = p(0) - p(1);
    1.0 / (1.0 + (-ecart / 8.0).exp())
}

/// Les options de `d`, une par clé (la première option de chaque clé).
fn options_par_cle(g: &GameState, d: &Decision, sortie: &mut Vec<(u64, usize)>) {
    sortie.clear();
    for (i, &c) in d.options.iter().enumerate() {
        let k = cle(g, d, c);
        if !sortie.iter().any(|&(x, _)| x == k) {
            sortie.push((k, i));
        }
    }
}

/// Clé d'un objet : la carte et sa zone pour une carte « neuve » hors de
/// l'arène (main, deck, arsenal, pitch, cimetière : les exemplaires
/// identiques sont interchangeables), l'objet lui-même sinon.
fn cle_objet(g: &GameState, id: ObjectId) -> u64 {
    let o = g.obj(id);
    match o.zone {
        Zone::Hand | Zone::Deck | Zone::Arsenal | Zone::Pitch | Zone::Graveyard => {
            (1 << 47) | ((o.card.0 as u64) << 8) | ((o.owner.0 as u64) << 6) | o.zone as u64
        }
        _ => id.0 as u64,
    }
}

/// Clé d'une option, la même d'un monde imaginé à l'autre :
/// joueur (1 bit) | type de décision (5) | type d'option (5) | complément (5)
/// | contenu (48).
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
