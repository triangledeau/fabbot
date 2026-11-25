//! Affichage en texte de la partie, **du point de vue d'un joueur**.
//!
//! Tout ce qui est affiché passe par [`visible`] : la main et l'arsenal
//! adverses (face cachée), l'ordre des decks et les cartes bannies face
//! cachée ne sont jamais montrés.

use fab_engine::card::{Color, Subtypes, Types};
use fab_engine::state::{CombatStep, LayerKind, RuleTrigger, Task};
use fab_engine::{
    Choice, Decision, DecisionKind, EndReason, Event, GameState, ObjectId, Op, Outcome, PlayerId,
    PreventRef, Target, Zone,
};
use std::fmt::Write;

/// Le joueur `viewer` a-t-il le droit de voir cette carte ?
///
/// Une carte face visible est publique. Une carte face cachée n'est visible
/// que par son propriétaire, sauf dans le deck (CR 3.0.3a : main, arsenal,
/// cartes bannies face cachée…).
pub fn visible(g: &GameState, viewer: PlayerId, id: ObjectId) -> bool {
    let o = g.obj(id);
    !o.face_down || (o.owner == viewer && o.zone != Zone::Deck)
}

fn couleur(c: Option<Color>) -> &'static str {
    match c {
        Some(Color::Red) => " (rouge)",
        Some(Color::Yellow) => " (jaune)",
        Some(Color::Blue) => " (bleu)",
        None => "",
    }
}

/// « Edict of Steel (rouge) [12] » : nom, couleur et numéro de l'objet
/// (le numéro sert à la commande `t`). Une attaque d'arme est nommée
/// d'après son arme.
pub fn nom(g: &GameState, id: ObjectId) -> String {
    let o = g.obj(id);
    let id = o.proxy_of.unwrap_or(id);
    let d = g.def(id);
    format!("{}{} [{}]", d.name, couleur(d.color), id.0)
}

/// Le nom si la carte est visible pour `viewer`, sinon « une carte face cachée ».
pub fn nom_pour(g: &GameState, viewer: PlayerId, id: ObjectId) -> String {
    if visible(g, viewer, id) {
        nom(g, id)
    } else {
        "une carte face cachée".into()
    }
}

/// Coût, puissance, défense, pitch : « coût 1, 4 puissance, 3 défense, pitch 1 ».
pub fn valeurs(g: &GameState, id: ObjectId) -> String {
    let d = g.def(id);
    let mut v = Vec::new();
    if let Some(c) = d.cost {
        v.push(if d.x_cost {
            format!("coût {c}+X")
        } else {
            format!("coût {c}")
        });
    }
    if d.power.is_some() {
        v.push(format!("{} puissance", g.power(id)));
    }
    if d.defense.is_some() {
        v.push(format!("{} défense", g.defense(id)));
    }
    if let Some(p) = d.pitch {
        v.push(format!("pitch {p}"));
    }
    v.join(", ")
}

fn joueur(viewer: PlayerId, p: PlayerId) -> &'static str {
    if p == viewer {
        "Vous"
    } else {
        "Le bot"
    }
}

pub fn heros(g: &GameState, p: PlayerId) -> String {
    g.def(g.player(p).hero).name.clone()
}

fn cible(g: &GameState, viewer: PlayerId, t: Target) -> String {
    match t {
        Target::Player(p) => format!("{} ({})", heros(g, p), joueur(viewer, p).to_lowercase()),
        Target::Object(r) => {
            let mut s = nom_pour(g, viewer, r.id);
            if !g.is_current(r) {
                s.push_str(" (n'existe plus)");
            }
            s
        }
    }
}

fn prevention(g: &GameState, viewer: PlayerId, r: PreventRef) -> String {
    match r {
        PreventRef::Effect(i) => match g.preventions.iter().find(|p| p.id == i) {
            Some(p) => {
                let src = p
                    .source
                    .map(|s| nom_pour(g, viewer, s.id))
                    .unwrap_or_else(|| "un effet".into());
                match p.remaining {
                    Some(n) => format!("Prévention de {src} ({n} restants)"),
                    None => format!("Prévention de {src}"),
                }
            }
            None => "Prévention".into(),
        },
        PreventRef::ArcaneBarrier(id) => format!("Arcane barrier de {}", nom(g, id)),
        PreventRef::Ward(id) => format!("Ward de {}", nom(g, id)),
        PreventRef::Spellvoid(id) => format!("Spellvoid de {}", nom(g, id)),
        PreventRef::Static(id) => format!("Prévention de {}", nom(g, id)),
        PreventRef::Counter(id) => format!("Compteur de {}", nom(g, id)),
    }
}

/// La carte à l'origine de la décision en cours (carte qu'on joue, capacité
/// qui pose une question, déclenchement dont on choisit la cible).
pub fn source_decision(g: &GameState) -> Option<String> {
    let card = match g.tasks.last()? {
        Task::Script { ctx, .. } => ctx.card,
        Task::PlaySide(c) => g.obj(*c).card,
        Task::TriggerTargets { layer, .. } => layer.card,
        Task::PlayOptionalCost
        | Task::PlayX
        | Task::PlayMode
        | Task::PlayTarget(_)
        | Task::PlayPay
        | Task::PlayEffectCost
        | Task::PlayAttackTarget => g.stack.last()?.card,
        _ => return None,
    };
    Some(g.db.get(card).name.clone())
}

/// Libellé de la cible demandée (« Target attack… »), si on la connaît.
fn libelle_cible(g: &GameState) -> Option<&'static str> {
    match g.tasks.last()? {
        Task::PlayTarget(i) => {
            let l = g.stack.last()?;
            let d = g.db.get(l.card);
            let specs = match l.kind {
                LayerKind::Activated(a) => &d.activated.get(a as usize)?.targets,
                _ => &d.targets,
            };
            specs.get(*i as usize).map(|s| s.label)
        }
        Task::TriggerTargets { layer, next } => {
            let d = g.db.get(layer.card);
            let list = match layer.kind {
                LayerKind::Trigger(t) => d.triggers.get(t as usize)?,
                LayerKind::Granted(t) => d.granted.get(t as usize)?,
                _ => return None,
            };
            list.targets.get(*next as usize).map(|s| s.label)
        }
        _ => None,
    }
}

/// Le libellé de la question posée par une capacité : on relit
/// l'instruction `Ask` en cours (une fonction pure, sans effet sur la partie).
fn question_effet(g: &GameState) -> Option<&'static str> {
    let Some(Task::Script {
        script, pc, ctx, ..
    }) = g.tasks.last()
    else {
        return None;
    };
    match g.script_ops(*script).get(*pc as usize)? {
        Op::Ask(f) => f(g, ctx).map(|a| a.label),
        _ => None,
    }
}

/// La question posée, en une ligne.
pub fn titre_decision(g: &GameState, d: &Decision) -> String {
    let src = source_decision(g);
    let de = |s: &str| match &src {
        Some(n) => format!("{s} ({n})"),
        None => s.to_string(),
    };
    match d.kind {
        DecisionKind::FirstPlayer => "Qui commence la partie ?".into(),
        DecisionKind::Priority => "Vous avez la priorité.".into(),
        DecisionKind::OptionalCost => de("Payer le coût additionnel optionnel ?"),
        DecisionKind::ChooseX => de("Valeur de X ?"),
        DecisionKind::Mode => de("Choisissez un mode"),
        DecisionKind::Target => match libelle_cible(g) {
            Some(l) if !l.is_empty() => de(&format!("Cible : « {l} »")),
            _ => de("Choisissez une cible"),
        },
        DecisionKind::Pitch => {
            let p = g.player(d.player);
            let reste = match g.tasks.last() {
                Some(Task::Pay { resources, .. }) => {
                    format!(" — il faut {resources}, vous avez {}", p.resource_points)
                }
                _ => String::new(),
            };
            de(&format!(
                "Payez le coût : choisissez une carte à pitcher{reste}"
            ))
        }
        DecisionKind::Defend => {
            let deja: Vec<String> = g.declared_defenders.iter().map(|&c| nom(g, c)).collect();
            if deja.is_empty() {
                "Déclarez vos cartes qui défendent (une par une), puis terminez.".into()
            } else {
                format!(
                    "Déclarez vos cartes qui défendent. Déjà déclarées : {}",
                    deja.join(", ")
                )
            }
        }
        DecisionKind::Effect => match question_effet(g) {
            Some(l) if !l.is_empty() => de(l),
            _ => de("Choix demandé par une capacité"),
        },
        DecisionKind::TriggerFirst => {
            "Plusieurs déclenchements : quel joueur empile les siens en premier ?".into()
        }
        DecisionKind::TriggerOrder => {
            "Choisissez le prochain de vos déclenchements à mettre sur la pile".into()
        }
        DecisionKind::Arsenal => "Fin de tour : mettre une carte en arsenal ?".into(),
        DecisionKind::PitchOrder => {
            "Fin de tour : choisissez la prochaine carte pitchée à mettre sous le deck".into()
        }
        DecisionKind::AttackTarget => de("Cible de l'attaque ?"),
        DecisionKind::Prevention => {
            "Des dégâts vous sont infligés : appliquer une prévention ?".into()
        }
        DecisionKind::Replacement => de("Appliquer l'effet de remplacement ?"),
    }
}

/// Le texte d'une option, du point de vue du joueur qui décide.
pub fn option(g: &GameState, d: &Decision, c: Choice) -> String {
    let v = d.player;
    match c {
        Choice::Pass => match d.kind {
            DecisionKind::Priority => "Passer".into(),
            DecisionKind::Defend => "Terminer la déclaration des défenseurs".into(),
            DecisionKind::Arsenal => "Ne rien mettre en arsenal".into(),
            DecisionKind::Prevention => "Ne plus appliquer de prévention".into(),
            DecisionKind::Target => "Aucune cible".into(),
            _ => "Aucun / passer".into(),
        },
        Choice::Yes => "Oui".into(),
        Choice::No => "Non".into(),
        Choice::Play(id) => {
            let depuis = match g.obj(id).zone {
                Zone::Arsenal => " depuis l'arsenal",
                Zone::Banished => " depuis la zone de bannissement",
                _ => "",
            };
            format!("Jouer {}{depuis} — {}", nom(g, id), valeurs(g, id))
        }
        Choice::Activate(id, i) => {
            let a = &g.def(id).activated[i as usize];
            let label = if a.label.is_empty() {
                if a.attack {
                    "attaquer"
                } else {
                    "capacité"
                }
            } else {
                a.label
            };
            let mut s = format!("Activer {} : {label}", nom(g, id));
            if a.cost > 0 {
                let _ = write!(s, " (coût {})", a.cost);
            }
            if a.attack {
                let _ = write!(s, " — {} puissance", g.power(id));
            }
            s
        }
        Choice::Pitch(id) => format!(
            "Pitcher {} (+{} ressource{})",
            nom(g, id),
            g.def(id).pitch_value(),
            if g.def(id).pitch_value() > 1 { "s" } else { "" }
        ),
        Choice::Defend(id) => {
            let d = g.def(id);
            let ou = if d.types.has(Types::EQUIPMENT) {
                "équipement"
            } else if g.obj(id).zone == Zone::Arsenal {
                "arsenal, ambush"
            } else {
                "main"
            };
            format!(
                "Défendre avec {} ({ou}) — {} défense",
                nom(g, id),
                g.defense(id)
            )
        }
        Choice::Arsenal(id) => format!("Mettre {} en arsenal", nom(g, id)),
        Choice::PitchToBottom(id) => format!("Mettre {} sous le deck", nom(g, id)),
        Choice::Player(p) => format!("{} ({})", joueur(v, p), heros(g, p)),
        Choice::Target(t) => cible(g, v, t),
        Choice::Object(id) => nom_pour(g, v, id),
        Choice::Number(n) => n.to_string(),
        Choice::Mode(m) => {
            // Côté d'une carte double (CR 5.1.2c) ou mode de la carte jouée.
            let card = match g.tasks.last() {
                Some(Task::PlaySide(c)) => Some(g.obj(*c).card),
                _ => g.stack.last().map(|l| l.card),
            };
            let label = card.and_then(|c| g.db.get(c).modes.get(m as usize).copied());
            match label {
                Some(l) => format!("Mode {} : {l}", m + 1),
                None => format!("Mode {}", m + 1),
            }
        }
        Choice::Trigger(i) => match g.pending_triggers.get(i as usize) {
            Some(l) => format!("Déclenchement de {}", nom_pour(g, v, l.object.id)),
            None => format!("Déclenchement n° {i}"),
        },
        Choice::Prevent(r) => prevention(g, v, r),
        Choice::Zone(p, z) => {
            let zone = match z {
                Zone::Head => "tête",
                Zone::Chest => "torse",
                Zone::Arms => "bras",
                Zone::Legs => "jambes",
                _ => "autre",
            };
            format!("Zone {zone} de {} ({})", joueur(v, p), heros(g, p))
        }
    }
}

/// La décision et ses options numérotées à partir de 1.
pub fn decision(g: &GameState, d: &Decision) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "» {}", titre_decision(g, d));
    for (i, &c) in d.options.iter().enumerate() {
        let _ = writeln!(s, "  {:>2}) {}", i + 1, option(g, d, c));
    }
    s
}

fn liste(g: &GameState, viewer: PlayerId, ids: &[ObjectId]) -> String {
    if ids.is_empty() {
        return "—".into();
    }
    ids.iter()
        .map(|&id| nom_pour(g, viewer, id))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Précisions sur une arme ou un équipement (en plus de sa puissance ou de
/// sa défense) : compteurs, engagé.
pub fn details_equipement(g: &GameState, id: ObjectId) -> Vec<String> {
    let o = g.obj(id);
    let mut extra = Vec::new();
    if o.power_counters != 0 {
        extra.push(format!("{:+} compteur(s) puissance", o.power_counters));
    }
    if o.defense_counters != 0 {
        extra.push(format!("{:+} compteur(s) défense", o.defense_counters));
    }
    if o.tapped {
        extra.push("engagé".into());
    }
    extra
}

fn equipement(g: &GameState, id: ObjectId) -> String {
    let mut s = nom(g, id);
    let d = g.def(id);
    let mut extra = Vec::new();
    if d.types.has(Types::WEAPON) {
        if d.power.is_some() {
            extra.push(format!("{} puissance", g.power(id)));
        }
    } else if d.defense.is_some() {
        extra.push(format!("{} défense", g.defense(id)));
    }
    extra.extend(details_equipement(g, id));
    if !extra.is_empty() {
        let _ = write!(s, " ({})", extra.join(", "));
    }
    s
}

/// Précisions sur un permanent : suspense, puissance et vie d'un allié,
/// compteurs, cartes dessous, phantasm, ward, engagé, volé.
pub fn details_permanent(g: &GameState, id: ObjectId) -> Vec<String> {
    let d = g.def(id);
    let o = g.obj(id);
    let mut extra: Vec<String> = Vec::new();
    if d.keywords.has(fab_engine::Keywords::SUSPENSE) {
        extra.push(format!("{} compteur(s) suspense", o.counters));
    }
    // Allié : puissance et vie actuelles (CR 2.5.3), compteurs.
    if let Some(vie) = g.life_total(id) {
        extra.push(format!("{}{{p}}, {vie} PV", g.power(id)));
        if d.counter_prevention > 0 {
            extra.push(format!("{} compteur(s) endurance", o.counters));
        }
        if o.life_counters != 0 {
            extra.push(format!("{:+} compteur(s) vie", o.life_counters));
        }
    }
    // Cartes dessous (CR 3.0.14) et mots-clés qu'elles donnent.
    let dessous: Vec<String> = g.cards_under(id).map(|c| g.def(c).name.clone()).collect();
    if !dessous.is_empty() {
        extra.push(format!("dessous : {}", dessous.join(", ")));
    }
    if g.keywords(id).has(fab_engine::Keywords::PHANTASM) {
        extra.push("phantasm".into());
    }
    let (_, ward, _) = g.prevention_values(id);
    if ward > 0 {
        extra.push(format!("ward {ward}"));
    }
    if o.tapped {
        extra.push("engagé".into());
    }
    if o.controller != o.owner {
        extra.push("volé".into());
    }
    extra
}

/// Le bloc d'un joueur : héros, PV, zones.
fn bloc_joueur(g: &GameState, viewer: PlayerId, p: PlayerId, complet: bool) -> String {
    let ps = g.player(p);
    let mut s = String::new();
    let marque = if ps.marked { ", marqué" } else { "" };
    let _ = writeln!(
        s,
        "{} — {} : {} PV{marque} | deck {} | points d'action {} | ressources {}",
        joueur(viewer, p).to_uppercase(),
        heros(g, p),
        ps.life,
        ps.deck.len(),
        ps.action_points,
        ps.resource_points
    );
    let mut equip: Vec<String> = ps.weapons.iter().map(|&w| equipement(g, w)).collect();
    equip.extend(ps.equipment().map(|e| equipement(g, e)));
    if !equip.is_empty() {
        let _ = writeln!(s, "  Armes et équipement : {}", equip.join(" ; "));
    }
    if p == viewer {
        let _ = writeln!(s, "  Main : {}", liste(g, viewer, &ps.hand));
    } else {
        let _ = writeln!(s, "  Main : {} carte(s)", ps.hand.len());
    }
    if let Some(a) = ps.arsenal {
        let _ = writeln!(s, "  Arsenal : {}", nom_pour(g, viewer, a));
    }
    if !ps.pitch.is_empty() {
        let _ = writeln!(s, "  Pitch : {}", liste(g, viewer, &ps.pitch));
    }
    let perms: Vec<ObjectId> = g
        .permanents
        .iter()
        .copied()
        .filter(|&id| g.obj(id).controller == p)
        .collect();
    if !perms.is_empty() {
        let noms: Vec<String> = perms
            .iter()
            .map(|&id| {
                let mut n = nom_pour(g, viewer, id);
                let extra = details_permanent(g, id);
                if !extra.is_empty() {
                    let _ = write!(n, " ({})", extra.join(", "));
                }
                n
            })
            .collect();
        let _ = writeln!(s, "  Permanents : {}", noms.join(", "));
    }
    if complet {
        let _ = writeln!(s, "  Cimetière : {}", liste(g, viewer, &ps.graveyard));
        let _ = writeln!(s, "  Bannies : {}", liste(g, viewer, &ps.banished));
    } else if !ps.graveyard.is_empty() || !ps.banished.is_empty() {
        let _ = writeln!(
            s,
            "  Cimetière : {} carte(s), bannies : {} (commande « e » pour le détail)",
            ps.graveyard.len(),
            ps.banished.len()
        );
    }
    s
}

pub fn etape(step: CombatStep) -> &'static str {
    match step {
        CombatStep::Layer => "couche",
        CombatStep::Attack => "attaque",
        CombatStep::Defend => "défense",
        CombatStep::Reaction => "réactions",
        CombatStep::Damage => "dégâts",
        CombatStep::Resolution => "résolution",
    }
}

fn regle(r: RuleTrigger) -> &'static str {
    match r {
        RuleTrigger::Spectra => "spectra",
        RuleTrigger::BladeBreak => "blade break",
        RuleTrigger::Battleworn => "battleworn",
        RuleTrigger::Temper => "temper",
        RuleTrigger::Guardwell => "guardwell",
        RuleTrigger::Unsharpen => "fin du sharpen",
        RuleTrigger::SuspenseTick => "suspense : retirer un compteur",
        RuleTrigger::SuspenseEmpty => "suspense : plus de compteur, détruire",
        RuleTrigger::BloodDebt => "blood debt : perdre 1 PV",
        RuleTrigger::Phantasm => "phantasm : détruire l'attaque",
    }
}

/// Une couche de la pile en une ligne : « Edict of Steel (rouge) [9] (le bot) → … ».
pub fn couche(g: &GameState, viewer: PlayerId, l: &fab_engine::state::Layer) -> String {
    let qui = joueur(viewer, l.controller);
    let quoi = match l.kind {
        LayerKind::Card => nom(g, l.object.id),
        LayerKind::Activated(a)
            if g.db
                .get(l.card)
                .activated
                .get(a as usize)
                .is_some_and(|x| x.attack) =>
        {
            format!("attaque avec {}", nom(g, l.object.id))
        }
        LayerKind::Activated(_) => format!("capacité de {}", g.db.get(l.card).name),
        LayerKind::Trigger(_) => format!("déclenchement de {}", g.db.get(l.card).name),
        LayerKind::Granted(_) => format!(
            "déclenchement de {} (accordé par {})",
            nom(g, l.object.id),
            g.db.get(l.card).name
        ),
        LayerKind::Rule(r) => format!("{} ({})", regle(r), g.db.get(l.card).name),
    };
    let cibles: Vec<String> = l
        .ctx
        .targets
        .iter()
        .flatten()
        .map(|&t| cible(g, viewer, t))
        .collect();
    let c = if cibles.is_empty() {
        String::new()
    } else {
        format!(" → {}", cibles.join(", "))
    };
    format!("{quoi} ({}){c}", qui.to_lowercase())
}

/// La chaîne de combat et la pile.
fn combat_et_pile(g: &GameState, viewer: PlayerId) -> String {
    let mut s = String::new();
    if g.combat.open {
        let step = g.combat.step.map(etape).unwrap_or("—");
        let _ = writeln!(s, "CHAÎNE DE COMBAT (étape : {step})");
        for (i, l) in g.combat.links.iter().enumerate() {
            let a = l.attack.id;
            let actif = g.is_current(l.attack) && g.obj(a).zone == Zone::CombatChain;
            let puissance = if actif {
                format!("{} puissance", g.power(a))
            } else {
                "terminée".into()
            };
            let cible = match l.target_obj {
                Some(r) => nom_pour(g, viewer, r.id),
                None => heros(g, l.target),
            };
            let _ = writeln!(
                s,
                "  Maillon {} : {} attaque {} — {}{}",
                i + 1,
                nom(g, a),
                cible,
                puissance,
                if l.hit { ", a touché" } else { "" }
            );
            if !l.defenders.is_empty() {
                let def: Vec<String> = l
                    .defenders
                    .iter()
                    .map(|&d| format!("{} ({} défense)", nom(g, d), g.defense(d)))
                    .collect();
                let total: i16 = l.defenders.iter().map(|&d| g.defense(d).max(0)).sum();
                let _ = writeln!(s, "    défendu par {} — total {total}", def.join(", "));
            }
        }
    }
    if !g.stack.is_empty() {
        let _ = writeln!(s, "PILE (du dessus vers le dessous)");
        for l in g.stack.iter().rev() {
            let _ = writeln!(s, "  {}", couche(g, viewer, l));
        }
    }
    s
}

/// L'état de la partie vu par `viewer`. `complet` : avec le détail des
/// cimetières et des cartes bannies.
pub fn etat(g: &GameState, viewer: PlayerId, complet: bool) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "──── Tour {} ({}) ────",
        g.turn,
        if g.turn_player == viewer {
            "votre tour"
        } else {
            "tour du bot"
        }
    );
    s.push_str(&bloc_joueur(g, viewer, viewer.opponent(), complet));
    s.push_str(&combat_et_pile(g, viewer));
    s.push_str(&bloc_joueur(g, viewer, viewer, complet));
    s
}

/// Un événement, du point de vue de `viewer`. `None` : rien à afficher.
pub fn evenement(g: &GameState, viewer: PlayerId, e: &Event) -> Option<String> {
    let qui = |p: PlayerId| joueur(viewer, p);
    // « Le bot joue X » / « Vous jouez X ».
    let sujet = |p: PlayerId, il: &str, vous: &str, quoi: &str| {
        if p == viewer {
            format!("Vous {vous} {quoi}")
        } else {
            format!("Le bot {il} {quoi}")
        }
    };
    Some(match *e {
        Event::StartOfTurn(p) => if p == viewer {
            "═══ Début de votre tour ═══"
        } else {
            "═══ Début du tour du bot ═══"
        }
        .into(),
        Event::Played {
            player, card, from, ..
        } => {
            let depuis = if from == Zone::Arsenal {
                " depuis l'arsenal"
            } else {
                ""
            };
            format!("{}{depuis}.", sujet(player, "joue", "jouez", &nom(g, card)))
        }
        Event::Activated {
            player,
            source,
            ability,
        } => {
            let a = g.def(source).activated.get(ability as usize)?;
            if a.attack {
                return None; // l'événement « attaque » suit
            }
            format!(
                "{} : {}.",
                sujet(player, "active", "activez", &nom(g, source)),
                a.label
            )
        }
        Event::Attack {
            attack, target_obj, ..
        } => {
            let cible = match target_obj {
                Some(o) => format!(" (cible : {})", nom(g, o)),
                None => String::new(),
            };
            format!("Attaque : {}{cible}.", nom(g, attack))
        }
        Event::Defend { card, defender, .. } => {
            format!(
                "{}.",
                sujet(defender, "défend avec", "défendez avec", &nom(g, card))
            )
        }
        Event::Hit {
            attack, target_obj, ..
        } => match target_obj {
            Some(o) => format!("{} touche {}.", nom(g, attack), nom(g, o)),
            None => format!("{} touche !", nom(g, attack)),
        },
        Event::Damage {
            source,
            target,
            target_obj,
            amount,
            ..
        } => {
            let src = source
                .map(|s| format!(" ({})", nom(g, s)))
                .unwrap_or_default();
            let a = match target_obj {
                Some(o) => nom(g, o),
                None => format!("{} ({})", heros(g, target), qui(target).to_lowercase()),
            };
            format!("{amount} dégât(s) infligé(s) à {a}{src}.")
        }
        Event::Pitched { player, card } => {
            format!("{}.", sujet(player, "pitche", "pitchez", &nom(g, card)))
        }
        Event::Destroyed { card, .. } => format!("{} est détruit.", nom(g, card)),
        Event::Banished { card, by, .. } => {
            format!(
                "{}.",
                sujet(by, "bannit", "bannissez", &nom_pour(g, viewer, card))
            )
        }
        Event::Drew { player, card } => {
            if player == viewer {
                format!("Vous piochez {}.", nom(g, card))
            } else {
                "Le bot pioche une carte.".into()
            }
        }
        Event::Sharpened { card, .. } => format!("{} est aiguisé (+1 puissance).", nom(g, card)),
        Event::GainedLife { player, amount } => {
            format!(
                "{} PV.",
                sujet(player, "gagne", "gagnez", &amount.to_string())
            )
        }
        Event::CombatChainClosed => "La chaîne de combat se ferme.".into(),
        Event::Became { player, card } => {
            let n = &g.db.get(card).name;
            if player == viewer {
                format!("Votre héros devient {n}.")
            } else {
                format!("Le héros du bot devient {n}.")
            }
        }
        Event::Revealed { player, card } => {
            format!("{}.", sujet(player, "révèle", "révélez", &nom(g, card)))
        }
        Event::ClashWon { winner, .. } => {
            format!("{}.", sujet(winner, "gagne", "gagnez", "le clash"))
        }
        Event::ToGraveyard { card, from, owner } => {
            // Seules les défausses depuis la main sont utiles à signaler.
            if from == Zone::Hand {
                format!("{}.", sujet(owner, "défausse", "défaussez", &nom(g, card)))
            } else {
                return None;
            }
        }
        Event::BeginActionPhase(_)
        | Event::BeginEndPhase(_)
        | Event::ChainLinkResolved { .. }
        | Event::EnterArena { .. }
        | Event::LeaveArena { .. } => return None,
    })
}

/// Le résultat de la partie.
pub fn resultat(g: &GameState, viewer: PlayerId) -> String {
    match (g.outcome, g.end_reason) {
        (Some(Outcome::Winner(p)), _) if p == viewer => "Victoire ! Vous avez gagné.".into(),
        (Some(Outcome::Winner(_)), _) => "Défaite : le bot a gagné.".into(),
        (Some(Outcome::Draw), Some(EndReason::TurnLimit)) => {
            "Partie nulle : limite de tours atteinte.".into()
        }
        (Some(Outcome::Draw), Some(EndReason::Stalemate)) => {
            "Partie nulle : plus personne ne peut avancer.".into()
        }
        (Some(Outcome::Draw), _) => "Partie nulle : les deux héros sont tombés à 0 PV.".into(),
        (None, _) => "Partie en cours.".into(),
    }
}

/// Fiche d'une carte : nom, ligne de type simplifiée, valeurs.
pub fn fiche(g: &GameState, id: ObjectId) -> String {
    let o = g.obj(id);
    let d = g.def(id);
    let mut s = nom(g, id);
    let mut types = Vec::new();
    for (t, n) in [
        (Types::HERO, "Héros"),
        (Types::WEAPON, "Arme"),
        (Types::EQUIPMENT, "Équipement"),
        (Types::ACTION, "Action"),
        (Types::ATTACK_REACTION, "Réaction d'attaque"),
        (Types::DEFENSE_REACTION, "Réaction de défense"),
        (Types::INSTANT, "Instant"),
        (Types::RESOURCE, "Ressource"),
        (Types::TOKEN, "Token"),
        (Types::BLOCK, "Block"),
    ] {
        if d.types.has(t) {
            types.push(n);
        }
    }
    if d.subtypes.has(Subtypes::ATTACK) {
        types.push("Attaque");
    }
    let _ = write!(s, " — {}", types.join(", "));
    let v = valeurs(g, id);
    if !v.is_empty() {
        let _ = write!(s, " — {v}");
    }
    if let Some(l) = d.life {
        let _ = write!(s, " — {l} PV");
    }
    let zone = match o.zone {
        Zone::Deck => "deck",
        Zone::Hand => "main",
        Zone::Arsenal => "arsenal",
        Zone::Pitch => "pitch",
        Zone::Graveyard => "cimetière",
        Zone::Banished => "bannie",
        Zone::Hero => "héros",
        Zone::Weapon => "arme",
        Zone::Head | Zone::Chest | Zone::Arms | Zone::Legs => "équipement",
        Zone::CombatChain => "chaîne de combat",
        Zone::Stack => "pile",
        Zone::Permanent => "permanents",
        Zone::Under => "sous un permanent",
        Zone::Gone => "hors jeu",
    };
    let _ = write!(s, " — zone : {zone}");
    s
}
