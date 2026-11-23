//! B1 : bot heuristique simple, sans recherche.
//!
//! Il répond à chaque décision par une règle fixe, en ne lisant la partie
//! qu'à travers une [`Vue`] (l'information de son joueur). Son style :
//! - **à son tour**, il attaque autant qu'il peut : d'abord les cartes avec
//!   go again (pour continuer le tour), puis la plus grosse attaque ; il joue
//!   ses réactions d'attaque ;
//! - **au tour adverse**, il défend quand une carte empêche assez de dégâts
//!   (en tenant compte de sa vie) et joue ses réactions de défense utiles ;
//! - il pitche les cartes qui servent le moins, au plus juste ;
//! - il met en arsenal sa meilleure carte d'attaque.
//!
//! Limites connues (B2 fera mieux par la recherche) : il ne lit pas le texte
//! des cartes, donc il ne sait pas si une cible est amie ou ennemie (il vise
//! l'attaque en cours, puis le héros adverse, puis les objets adverses), il
//! répond « oui » aux questions des cartes et prend le premier mode.

use crate::vue::Vue;
use crate::Agent;
use fab_engine::state::CombatStep;
use fab_engine::{Choice, Decision, DecisionKind, GameState, Keywords, Subtypes, Target, Types};

/// Réglages de B1 (pour pouvoir comparer des variantes au simulateur).
#[derive(Clone, Copy, Debug)]
pub struct Reglages {
    /// Coût de base d'une carte de la main utilisée pour défendre (ce
    /// qu'elle aurait rapporté en attaquant ou en pitchant).
    pub cout_defense_main: f32,
    /// Supplément par point de puissance de la carte qui défendrait.
    pub cout_par_puissance: f32,
    /// Supplément pour chaque carte de la main déjà déclarée sur ce maillon.
    pub cout_par_carte_deja_posee: f32,
    /// Vie en dessous de laquelle le bot défend plus volontiers.
    pub vie_basse: i32,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            cout_defense_main: 2.5,
            cout_par_puissance: 0.25,
            cout_par_carte_deja_posee: 0.75,
            vie_basse: 10,
        }
    }
}

/// B1 : heuristique.
#[derive(Default)]
pub struct HeuristicAgent {
    pub reglages: Reglages,
}

impl HeuristicAgent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Agent for HeuristicAgent {
    fn name(&self) -> &str {
        "B1-heuristique"
    }

    fn choose(&mut self, state: &GameState, d: &Decision) -> usize {
        let v = Vue::new(state, d.player);
        let i = match d.kind {
            DecisionKind::Priority => priorite(&v, d),
            DecisionKind::Pitch => pitch(&v, d),
            DecisionKind::Defend => defendre(&v, d, &self.reglages),
            DecisionKind::Arsenal => arsenal(&v, d),
            DecisionKind::FirstPlayer => d.index_of(Choice::Player(d.player)),
            DecisionKind::AttackTarget => {
                d.index_of(Choice::Target(Target::Player(d.player.opponent())))
            }
            DecisionKind::Target => cible(&v, d),
            DecisionKind::ChooseX => meilleur(d, |c| match c {
                Choice::Number(n) => n as f32,
                _ => -1.0,
            }),
            DecisionKind::OptionalCost
            | DecisionKind::Replacement
            | DecisionKind::Effect
            | DecisionKind::Prevention => d
                .index_of(Choice::Yes)
                .or_else(|| d.options.iter().position(|&c| c != Choice::Pass)),
            DecisionKind::Mode
            | DecisionKind::TriggerFirst
            | DecisionKind::TriggerOrder
            | DecisionKind::PitchOrder => None,
        };
        i.unwrap_or(0)
    }
}

/// L'index de l'option de plus haut score (la première en cas d'égalité).
fn meilleur(d: &Decision, score: impl Fn(Choice) -> f32) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, &c) in d.options.iter().enumerate() {
        let s = score(c);
        if best.is_none_or(|(_, b)| s > b) {
            best = Some((i, s));
        }
    }
    best.map(|(i, _)| i)
}

/// Ce que vaut une de mes cartes si je la garde (pour choisir quoi pitcher
/// ou mettre en arsenal) : à peu près les dégâts qu'elle peut faire.
fn valeur_gardee(v: &Vue, id: fab_engine::ObjectId) -> f32 {
    let c = v.carte(id);
    if c.is_attack_card() {
        let ga = if v.mots_cles(id).has(Keywords::GO_AGAIN) {
            1.0
        } else {
            0.0
        };
        v.puissance(id).max(0) as f32 + ga
    } else if c.types.has(Types::DEFENSE_REACTION) {
        v.defense(id).max(0) as f32
    } else if c
        .types
        .any(Types::ACTION | Types::ATTACK_REACTION | Types::INSTANT)
    {
        2.0
    } else {
        0.5
    }
}

// ---------------------------------------------------------------------------
// Priorité : jouer une carte, activer une capacité, ou passer.
// ---------------------------------------------------------------------------

fn priorite(v: &Vue, d: &Decision) -> Option<usize> {
    let mon_action = v.mon_tour()
        && v.phase_d_action()
        && v.pile_vide()
        && (!v.combat_ouvert() || v.etape() == Some(CombatStep::Resolution));
    let maillon = v.maillon();
    let reaction = v.etape() == Some(CombatStep::Reaction);
    let j_attaque = reaction && maillon.is_some_and(|m| m.attaquant == v.moi());
    let je_defends = reaction && maillon.is_some_and(|m| m.defenseur == v.moi() && m.vise_heros);

    // Dégâts qui passeront sur l'attaque en cours, si rien ne change.
    let restant = maillon
        .map(|m| {
            let def: i16 = v.defenseurs().iter().map(|&c| v.defense(c).max(0)).sum();
            (v.puissance(m.attaque) - def).max(0) as i32
        })
        .unwrap_or(0);

    let score = |c: Choice| -> f32 {
        match c {
            Choice::Pass => 0.0,
            Choice::Play(id) => {
                let carte = v.carte(id);
                let t = carte.types;
                let ga = v.mots_cles(id).has(Keywords::GO_AGAIN);
                if t.has(Types::ACTION) && mon_action {
                    let base = if carte.is_attack_card() {
                        v.puissance(id).max(0) as f32
                    } else {
                        // Action non-attaque : souvent un bonus pour la
                        // prochaine attaque, à jouer avant elle.
                        2.0
                    };
                    let bonus = if ga {
                        10.0 + if carte.is_attack_card() { 0.0 } else { 2.0 }
                    } else {
                        0.0
                    };
                    base + bonus - 0.1 * v.cout(id) as f32
                } else if t.has(Types::ATTACK_REACTION) && j_attaque {
                    3.0
                } else if t.has(Types::DEFENSE_REACTION) && je_defends {
                    let evite = (v.defense(id).max(0) as i32).min(restant);
                    if restant >= v.vie(v.moi()) {
                        10.0 + evite as f32
                    } else {
                        evite as f32 - 1.5
                    }
                } else if t.has(Types::INSTANT) && (mon_action || j_attaque) {
                    1.0
                } else {
                    -1.0
                }
            }
            Choice::Activate(src, i) => {
                let a = &v.carte(src).activated[i as usize];
                if a.attack {
                    if !mon_action {
                        return -1.0;
                    }
                    let ga = a.go_again || v.mots_cles(src).has(Keywords::GO_AGAIN);
                    v.puissance(src).max(0) as f32 + if ga { 10.0 } else { 0.0 }
                        - 0.1 * v.cout_activation(src, i) as f32
                } else if mon_action && !a.from_hand && activation_sans_perte(v, src) {
                    0.8
                } else {
                    -1.0
                }
            }
            _ => -1.0,
        }
    };
    meilleur(d, score)
}

/// Une capacité non-attaque qui ne coûte ni une carte de la main ni un
/// équipement : celles du héros, des armes et des permanents (tokens,
/// alliés…). Sans lire le texte, B1 n'active que celles-là : « Destroy this »
/// sur un équipement (Predatory Plating) ou « Banish this from your hand »
/// (Cleave the Heavens) gâchent souvent une carte.
fn activation_sans_perte(v: &Vue, src: fab_engine::ObjectId) -> bool {
    use fab_engine::Zone;
    match v.zone(src) {
        Zone::Hero | Zone::Permanent => true,
        Zone::Weapon => !v.carte(src).types.has(Types::EQUIPMENT),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Pitch : la carte qui sert le moins, en couvrant le reste à payer au plus juste.
// ---------------------------------------------------------------------------

fn pitch(v: &Vue, d: &Decision) -> Option<usize> {
    let reste = v.reste_a_payer().unwrap_or(1).max(1) as f32;
    meilleur(d, |c| match c {
        Choice::Pitch(id) => {
            let p = v.carte(id).pitch_value() as f32;
            // Couvrir le reste en une carte, sans trop de gaspillage ; sinon
            // la plus grosse valeur. Puis garder les meilleures cartes.
            let couverture = if p >= reste { 10.0 - (p - reste) } else { p };
            couverture * 3.0 - valeur_gardee(v, id)
        }
        _ => f32::MIN,
    })
}

// ---------------------------------------------------------------------------
// Défense : une carte à la fois, puis « Pass ».
// ---------------------------------------------------------------------------

fn defendre(v: &Vue, d: &Decision, r: &Reglages) -> Option<usize> {
    let Some(m) = v.maillon() else {
        return d.index_of(Choice::Pass);
    };
    let deja = v.defenseurs();
    let def_deja: i16 = deja.iter().map(|&c| v.defense(c).max(0)).sum();
    let restant = (v.puissance(m.attaque) - def_deja).max(0) as i32;
    if restant == 0 {
        return d.index_of(Choice::Pass);
    }
    let vie = v.vie(v.moi());
    let mortel = restant >= vie;
    let main_posees = deja
        .iter()
        .filter(|&&c| v.zone(c) == fab_engine::Zone::Hand)
        .count() as f32;
    let pression = if vie <= r.vie_basse { 1.5 } else { 1.0 };

    let score = |c: Choice| -> f32 {
        let Choice::Defend(id) = c else {
            return 0.0; // Pass
        };
        let def = v.defense(id).max(0) as i32;
        let evite = def.min(restant) as f32;
        if mortel {
            return 100.0 + evite;
        }
        // Bloquer entièrement l'attaque évite aussi ses effets « on hit ».
        let gain = (evite + if def >= restant { 1.0 } else { 0.0 }) * pression;
        let carte = v.carte(id);
        let cout = if carte.types.has(Types::EQUIPMENT) {
            let kw = v.mots_cles(id);
            if kw.has(Keywords::BLADE_BREAK) {
                def as f32 * 0.8
            } else if kw.any(Keywords::BATTLEWORN | Keywords::TEMPER) {
                1.0
            } else {
                0.5
            }
        } else {
            let p = if carte.subtypes.has(Subtypes::ATTACK) {
                v.puissance(id).max(0) as f32
            } else {
                0.0
            };
            r.cout_defense_main
                + r.cout_par_puissance * p
                + r.cout_par_carte_deja_posee * main_posees
        };
        gain - cout
    };
    meilleur(d, score)
}

// ---------------------------------------------------------------------------
// Arsenal : la carte la plus utile au prochain tour.
// ---------------------------------------------------------------------------

fn arsenal(v: &Vue, d: &Decision) -> Option<usize> {
    meilleur(d, |c| match c {
        Choice::Arsenal(id) => valeur_gardee(v, id),
        _ => -1.0,
    })
}

// ---------------------------------------------------------------------------
// Cibles : sans lire le texte, on suppose qu'un effet aide son contrôleur.
// ---------------------------------------------------------------------------

fn cible(v: &Vue, d: &Decision) -> Option<usize> {
    let attaque = v.maillon().map(|m| (m.attaque, m.attaquant));
    meilleur(d, |c| match c {
        Choice::Target(Target::Object(r)) => match attaque {
            Some((a, qui)) if a == r.id && qui == v.moi() => 3.0,
            _ if v.visible(r.id) && v.controleur(r.id) == v.adversaire() => 1.0,
            _ => 0.0,
        },
        Choice::Target(Target::Player(p)) if p == v.adversaire() => 2.0,
        Choice::Pass => -1.0,
        _ => 0.0,
    })
}
