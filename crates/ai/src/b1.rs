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
//! - il met en arsenal sa meilleure carte d'attaque ;
//! - il attaque un allié adverse (un dragon) quand son attaque peut le
//!   détruire et que l'allié frappe fort, sinon le héros adverse.
//!
//! Limites connues (B2 fera mieux par la recherche) : il ne lit pas le texte
//! des cartes, donc il ne sait pas si une cible est amie ou ennemie (il vise
//! l'attaque en cours, puis le héros adverse, puis les objets adverses), il
//! répond « oui » aux questions des cartes et prend le premier mode.

use crate::vue::Vue;
use crate::Agent;
use fab_engine::state::CombatStep;
use fab_engine::{
    Choice, Decision, DecisionKind, GameState, Keywords, ObjectId, Subtypes, Target, Types,
};

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
    /// Nombre d'attaques d'un allié adverse qu'on compte éviter en le
    /// détruisant (0 : toujours viser le héros).
    pub tours_d_allie: f32,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            cout_defense_main: 2.5,
            cout_par_puissance: 0.25,
            cout_par_carte_deja_posee: 0.75,
            vie_basse: 10,
            tours_d_allie: 2.0,
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
            DecisionKind::AttackTarget => cible_d_attaque(&v, d, &self.reglages),
            DecisionKind::Target => cible(&v, d),
            DecisionKind::ChooseX => meilleur(d, |c| match c {
                Choice::Number(n) => n as f32,
                _ => -1.0,
            }),
            // Une zone où créer un token (Frost Spike : un Frostbite) : chez
            // l'adversaire, le token le gêne.
            DecisionKind::Effect if d.options.iter().any(|c| matches!(c, Choice::Zone(..))) => d
                .options
                .iter()
                .position(|c| matches!(c, Choice::Zone(p, _) if *p == v.adversaire()))
                .or(Some(0)),
            DecisionKind::OptionalCost
            | DecisionKind::Replacement
            | DecisionKind::Effect
            | DecisionKind::Prevention => d
                .index_of(Choice::Yes)
                .or_else(|| d.options.iter().position(|&c| c != Choice::Pass)),
            DecisionKind::PitchOrder => ordre_du_pitch(&v, d),
            DecisionKind::Mode | DecisionKind::TriggerFirst | DecisionKind::TriggerOrder => None,
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
    if let Some(n) = degats_arcane(c) {
        // Un sort compte comme une attaque que rien ne bloque.
        return 3.0 * n as f32;
    }
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
                if let Some(s) = score_sort(v, d, id, mon_action) {
                    return s;
                }
                let carte = v.carte(id);
                let t = carte.types;
                let ga = v.mots_cles(id).has(Keywords::GO_AGAIN);
                if t.has(Types::ACTION) && mon_action {
                    let (bonus, ga_combo) = combo_prevu(v, id);
                    let ga = ga || ga_combo;
                    let base = if carte.is_attack_card() {
                        (v.puissance(id) + bonus).max(0) as f32
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
                if let Some(s) = score_renfort(v, d, src, mon_action) {
                    return s;
                }
                let a = &v.carte(src).activated[i as usize];
                if a.attack {
                    if !mon_action {
                        return -1.0;
                    }
                    let ga = a.go_again
                        || v.mots_cles(src).has(Keywords::GO_AGAIN)
                        || go_again_d_arme_prevu(v, src);
                    v.puissance(src).max(0) as f32 + if ga { 10.0 } else { 0.0 }
                        - 0.1 * v.cout_activation(src, i) as f32
                } else if let Some(s) = equipement_d_ira(v, d, src, mon_action) {
                    s
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

/// Le go again que l'attaque d'une arme recevra, quand il n'est pas écrit
/// sur l'arme mais donné à son attaque-proxy (B1 lit les mots-clés de
/// l'arme). Seul cas dans nos decks : Zenith Blade, « If this has been
/// sharpened this turn, its first attack this turn gets go again ».
fn go_again_d_arme_prevu(v: &Vue, arme: ObjectId) -> bool {
    v.carte(arme).name == "Zenith Blade" && v.aiguise_ce_tour(arme) && !v.a_attaque_ce_tour(arme)
}

/// Ce qu'un combo ajoutera à une carte d'attaque de ma main si je la joue
/// maintenant : (puissance en plus, go again). Le moteur ne l'applique que sur
/// la pile ou la chaîne, et B1 ne lit pas le texte : liste des combos de nos
/// decks (Ira), d'après « the last attack this combat chain ».
fn combo_prevu(v: &Vue, id: ObjectId) -> (i16, bool) {
    // Scar for a Scar : « When this is played, if you have less {h} than an
    // opposing hero, it gets go again. »
    if v.carte(id).name == "Scar for a Scar" {
        return (0, v.vie(v.moi()) < v.vie(v.adversaire()));
    }
    let Some(derniere) = v.derniere_attaque() else {
        return (0, false);
    };
    match v.carte(id).name.as_str() {
        // « If Edge of Autumn was the last attack … this gets go again. »
        "Seek Vengeance" if derniere.name == "Edge of Autumn" => (0, true),
        // « If Crouching Tiger was the last attack … +1{p} and go again. »
        "Pouncing Qi" if derniere.name == "Crouching Tiger" => (1, true),
        // « If Crouching Tiger was the last attack … +4{p}. »
        "Qi Unleashed" if derniere.name == "Crouching Tiger" => (4, false),
        // « if a red attack action card was the last attack … go again »
        "Aspect of Tiger: Body"
            if derniere.color == Some(fab_engine::Color::Red)
                && derniere.is_action()
                && derniere.is_attack_card() =>
        {
            (0, true)
        }
        _ => (0, false),
    }
}

/// Les équipements d'Ira (« Destroy this ») : B1 ne lit pas leur texte et
/// n'active pas d'équipement qui se détruit. Règles d'usage, d'après les
/// textes officiels :
/// - Pouncing Paws, « Create a Crouching Tiger in your banished zone. You may
///   play it this turn » : quand un combo du Tigre (Pouncing Qi, Qi Unleashed)
///   est en main ;
/// - Tearing Shuko, « The next Crouching Tiger you play this turn gets +2{p} » :
///   juste avant de jouer un Crouching Tiger ;
/// - Blood Scent, « Gain {r} » (seulement après une attaque de Crouching
///   Tiger, vérifié par le moteur) : quand une carte de ma main coûte plus que
///   mes ressources.
fn equipement_d_ira(v: &Vue, d: &Decision, src: ObjectId, mon_action: bool) -> Option<f32> {
    if !mon_action {
        return None;
    }
    let main = v.ma_main();
    let tigre_jouable = d
        .options
        .iter()
        .any(|o| matches!(o, Choice::Play(c) if v.carte(*c).name == "Crouching Tiger"));
    Some(match v.carte(src).name.as_str() {
        "Pouncing Paws" => {
            let combo = main
                .iter()
                .any(|&c| matches!(v.carte(c).name.as_str(), "Pouncing Qi" | "Qi Unleashed"));
            if combo && !tigre_jouable {
                10.5
            } else {
                -1.0
            }
        }
        "Tearing Shuko" if tigre_jouable => 10.6,
        "Blood Scent" => {
            let r = v.points_de_ressource() as u32;
            if main
                .iter()
                .any(|&c| v.carte(c).is_action() && v.cout(c) > r)
            {
                0.9
            } else {
                -1.0
            }
        }
        "Tearing Shuko" => -1.0,
        _ => return None,
    })
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
        Choice::Arsenal(id) if glace_du_tour_adverse(v, id) => 100.0,
        Choice::Arsenal(id) => valeur_gardee(v, id),
        _ => -1.0,
    })
}

// ---------------------------------------------------------------------------
// Cible d'une attaque : le héros adverse, ou un allié qu'elle peut détruire.
// ---------------------------------------------------------------------------

/// Vise un allié adverse (un dragon…) quand l'attaque peut le détruire et
/// que ses futures attaques valent plus que les dégâts faits au héros ;
/// sinon le héros. Jamais un allié si l'attaque peut tuer le héros.
fn cible_d_attaque(v: &Vue, d: &Decision, r: &Reglages) -> Option<usize> {
    let heros = d.index_of(Choice::Target(Target::Player(v.adversaire())));
    let Some(attaque) = v.ma_carte_sur_la_pile() else {
        return heros;
    };
    let p = v.puissance(attaque).max(0) as f32;
    if p as i32 >= v.vie(v.adversaire()) {
        return heros;
    }
    meilleur(d, |c| match c {
        Choice::Target(Target::Player(q)) if q == v.adversaire() => p,
        Choice::Target(Target::Object(o)) if v.carte(o.id).subtypes.has(Subtypes::ALLY) => {
            match v.vie_objet(o.id) {
                Some(vie) if p as i32 >= vie => r.tours_d_allie * v.puissance(o.id).max(0) as f32,
                _ => -1.0,
            }
        }
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

// ---------------------------------------------------------------------------
// Sorts arcaniques (Iyslander) : passer l'arcane barrier adverse.
// ---------------------------------------------------------------------------

/// Dégâts arcaniques imprimés des cartes « Deal N arcane damage » de nos
/// decks (textes officiels ; B1 ne lit pas les scripts).
fn degats_arcane(c: &fab_engine::CardDef) -> Option<i32> {
    use fab_engine::Color;
    let col = |r: i32, y: i32, b: i32| match c.color {
        Some(Color::Red) => r,
        Some(Color::Yellow) => y,
        _ => b,
    };
    Some(match c.name.as_str() {
        // Aethersling et Turn to Mindfire n'existent qu'en rouge.
        "Aethersling" => 4,
        "Turn to Mindfire" => 5,
        "Arcane Twining" | "Frosting" => col(3, 2, 1),
        "Photon Splicing" => col(4, 3, 2),
        "Ice Bolt" | "Voltic Bolt" | "Timekeeper's Whim" => col(5, 4, 3),
        _ => return None,
    })
}

/// Ressources que je peux réunir sans pitcher les cartes `sauf`.
fn ressources_sans(v: &Vue, sauf: &[ObjectId]) -> u32 {
    v.points_de_ressource() as u32
        + v.ma_main()
            .iter()
            .filter(|c| !sauf.contains(c))
            .map(|&c| v.carte(c).pitch_value() as u32)
            .sum::<u32>()
}

/// Dégâts d'un de mes sorts qui passeraient l'arcane barrier adverse, avec
/// `plus` de bonus en plus de ceux déjà en attente.
fn passe_la_barriere(v: &Vue, id: ObjectId, plus: i32) -> Option<i32> {
    let n = degats_arcane(v.carte(id))?;
    Some(n + v.bonus_arcane_en_attente() + plus - v.barriere_arcane(v.adversaire()) as i32)
}

/// Les sorts que je peux jouer maintenant (options de la décision).
fn sorts_jouables<'a>(v: &'a Vue, d: &'a Decision) -> impl Iterator<Item = ObjectId> + 'a {
    d.options.iter().filter_map(move |&c| match c {
        Choice::Play(id) if degats_arcane(v.carte(id)).is_some() => Some(id),
        _ => None,
    })
}

/// Score d'un sort arcanique : les dégâts qui passeront la barrière. Un sort
/// entièrement arrêté attend (défense, pitch, ou renfort plus tard).
fn score_sort(v: &Vue, d: &Decision, id: ObjectId, mon_action: bool) -> Option<f32> {
    if !v.mon_tour() && v.zone(id) == fab_engine::Zone::Arsenal && glace_du_tour_adverse(v, id) {
        // Joué comme un instant au tour adverse : une gelure chez lui.
        return Some(1.5);
    }
    if !mon_action {
        return None;
    }
    if v.carte(id).name == "Stir the Aetherwinds" {
        // « If it has an arcane damage effect, instead it deals that much
        // arcane damage plus 3 » (3, 2, 1 selon la couleur).
        let n = match v.carte(id).color {
            Some(fab_engine::Color::Red) => 3,
            Some(fab_engine::Color::Yellow) => 2,
            _ => 1,
        };
        let ok = sorts_jouables(v, d).filter(|&s| s != id).any(|s| {
            let avant = passe_la_barriere(v, s, 0).unwrap_or(0);
            avant <= 0 && avant + n > 0 && ressources_sans(v, &[s, id]) >= v.cout(s) + v.cout(id)
        });
        return Some(if ok { 29.0 } else { -0.5 });
    }
    let passe = passe_la_barriere(v, id, 0)?;
    Some(if passe > 0 {
        1.0 + passe as f32 - 0.1 * v.cout(id) as f32
    } else {
        -0.5
    })
}

/// Renforts avant un sort : Crucible of Aetherweave ({r} : +1), amp 1 en
/// défaussant Arcane Twining / Photon Splicing, Stir the Aetherwinds. Joués
/// juste avant un sort quand ils lui font passer la barrière (ou, pour le
/// Crucible, ajoutent un dégât), et s'il reste de quoi payer le sort.
fn score_renfort(v: &Vue, d: &Decision, src: ObjectId, mon_action: bool) -> Option<f32> {
    let nom = v.carte(src).name.as_str();
    let est_amp = matches!(nom, "Arcane Twining" | "Photon Splicing")
        && v.zone(src) == fab_engine::Zone::Hand;
    if nom != "Crucible of Aetherweave" && !est_amp {
        return None;
    }
    if !mon_action {
        return Some(-1.0);
    }
    let ok = sorts_jouables(v, d).filter(|&s| s != src).any(|s| {
        let Some(avant) = passe_la_barriere(v, s, 0) else {
            return false;
        };
        let reste = ressources_sans(v, &[s, src]);
        if nom == "Crucible of Aetherweave" {
            avant + 1 > 0 && reste > v.cout(s)
        } else {
            // Amp : une carte en moins, seulement si elle fait passer le sort.
            avant <= 0 && avant + 1 > 0 && reste >= v.cout(s)
        }
    });
    Some(if ok { 30.0 } else { -1.0 })
}

/// Iyslander : « If it's not your turn, you may play blue non-attack action
/// cards from your arsenal as though they were instants » et « Whenever you
/// play an Ice card during an opponent's turn, create a Frostbite token under
/// their control » : une carte Ice bleue non-attaque à garder en arsenal.
fn glace_du_tour_adverse(v: &Vue, id: ObjectId) -> bool {
    let c = v.carte(id);
    v.carte(v.mon_heros()).name == "Iyslander"
        && c.color == Some(fab_engine::Color::Blue)
        && c.supertypes.has(fab_engine::Supertypes::ICE)
        && c.is_action()
        && !c.is_attack_card()
}

/// Pitch stacking : l'ordre des cartes pitchées sous le deck. La première
/// choisie sera piochée la première ; on alterne une carte qui paie (bleue)
/// et une carte à jouer, pour que chaque future main ait un sort et de quoi
/// le payer.
fn ordre_du_pitch(v: &Vue, d: &Decision) -> Option<usize> {
    let restantes = d.options.len();
    meilleur(d, |c| match c {
        Choice::PitchToBottom(id) => {
            let p = v.carte(id).pitch_value() as f32;
            // Nombre impair de cartes restantes : une carte à jouer ; pair : une qui paie.
            if restantes % 2 == 1 {
                valeur_gardee(v, id) - p
            } else {
                p
            }
        }
        _ => -1.0,
    })
}
