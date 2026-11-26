//! Une partie jouée depuis la page web : l'état de la partie, le bot,
//! l'historique d'annulation et le journal des événements.
//!
//! Tout ce qui part vers le navigateur est construit ici, **du point de vue
//! de l'humain** : une carte qu'il n'a pas le droit de voir (main ou arsenal
//! du bot, deck, carte bannie face cachée de l'adversaire) n'est envoyée ni
//! par son nom ni par son numéro (le numéro d'un objet pourrait trahir la
//! carte, puisqu'il suit l'ordre de la liste du deck).

use crate::images;
use crate::textes::Textes;
use crate::vue;
use fab_ai::{creer_agent, Agent, BOTS};
use fab_engine::card::{CardDef, Color, Types};
use fab_engine::{
    Choice, DecisionKind, Event, GameState, History, ObjectId, PlayerId, Target, Zone,
};
use serde_json::{json, Value};

/// Une ligne du journal.
#[derive(Clone, Debug)]
pub struct Ligne {
    pub tour: u32,
    /// Début d'un tour (affiché comme un titre).
    pub debut: bool,
    pub texte: String,
}

/// Chiffres de la manche pour l'humain, montrés à la fin de la partie
/// (fenêtre « Victoire ! » de l'Arsenal JS). Comptés depuis les événements
/// du moteur : rien n'est recalculé ici.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Dégâts infligés au héros adverse.
    pub infliges: u32,
    /// Dégâts subis par votre héros.
    pub subis: u32,
    /// Cartes de votre main qui ont défendu.
    pub blocs_main: u32,
    /// Autres cartes qui ont défendu (équipement, arsenal…).
    pub blocs_autres: u32,
}

/// Ce qu'on mémorise avant chaque choix de l'humain, pour l'annuler.
#[derive(Clone)]
struct Instantane {
    g: GameState,
    journal: usize,
    tour: u32,
    coups: usize,
    stats: Stats,
}

/// Les réglages d'une partie.
#[derive(Clone, Debug)]
pub struct Reglages {
    pub deck: String,
    pub deck_bot: String,
    pub niveau: String,
    pub graine: u64,
    /// Nombre de manches à gagner pour remporter le match (1 à 3).
    pub manches: u8,
}

pub struct Session {
    pub g: GameState,
    pub humain: PlayerId,
    pub reglages: Reglages,
    bot: Box<dyn Agent + Send>,
    nom_bot: String,
    historique: History<Instantane>,
    journal: Vec<Ligne>,
    tour: u32,
    /// Le « film » du dernier choix de l'humain : le plateau après chacun
    /// des pas du moteur qui ont suivi (son choix, puis les réponses du
    /// bot), avec les événements à animer. La page le rejoue (lot 2 du plan
    /// de rapatriement) ; tout y est vu par l'humain.
    film: Vec<Value>,
    /// Numéro du film : change à chaque choix, pour que la page ne rejoue
    /// pas deux fois le même.
    film_no: u64,
    /// Attaques qui ont touché depuis leur déclaration (pour « Bloqué »).
    touchees: Vec<ObjectId>,
    /// Tous les choix appliqués au moteur depuis le début de la manche
    /// (humain et bot) : rejoués tels quels, ils redonnent exactement la
    /// même partie (sauvegarde, lot 3 ; voir `crate::sauvegarde`).
    coups: Vec<u32>,
    stats: Stats,
}

impl Session {
    /// Nouvelle partie : l'humain est le joueur 0. Le moteur avance
    /// jusqu'au premier choix de l'humain (le bot répond aux siens).
    pub fn nouvelle(r: Reglages) -> Result<Session, String> {
        let g = crate::nouvelle_partie(&r.deck, &r.deck_bot, r.graine)?;
        Session::avec_partie(g, r)
    }

    /// Session sur une partie déjà créée (manche d'un match, après la
    /// phase de side). L'humain est le joueur 0.
    pub fn avec_partie(g: GameState, r: Reglages) -> Result<Session, String> {
        let bot = creer_agent(&r.niveau, r.graine ^ 0x5A5A).ok_or_else(|| {
            format!(
                "niveau de bot inconnu « {} » (connus : {}, b2-N)",
                r.niveau,
                BOTS.join(", ")
            )
        })?;
        Session::avec_agent(g, r, bot)
    }

    /// Session avec un adversaire donné (le didacticiel y met un bot qui
    /// suit un script, `crate::api_apprendre`).
    pub fn avec_agent(
        mut g: GameState,
        r: Reglages,
        bot: Box<dyn Agent + Send>,
    ) -> Result<Session, String> {
        if g.record.is_none() {
            g.record = Some(Vec::new());
        }
        let nom_bot = bot.name().to_string();
        let mut s = Session {
            g,
            humain: PlayerId::P0,
            reglages: r,
            bot,
            nom_bot,
            historique: History::new(),
            journal: Vec::new(),
            tour: 0,
            film: Vec::new(),
            film_no: 0,
            touchees: Vec::new(),
            coups: Vec::new(),
            stats: Stats::default(),
        };
        s.avancer()?;
        Ok(s)
    }

    /// Session dont les premiers choix sont rejoués (sauvegarde) : chaque
    /// choix de `coups` est appliqué tel quel au moteur, sans demander au
    /// bot ; les choix de l'humain peuvent ensuite être annulés comme
    /// d'habitude. Le bot reprend la main après le dernier coup.
    pub fn rejouer(mut g: GameState, r: Reglages, coups: &[u32]) -> Result<Session, String> {
        if g.record.is_none() {
            g.record = Some(Vec::new());
        }
        let bot = creer_agent(&r.niveau, r.graine ^ 0x5A5A)
            .ok_or_else(|| format!("niveau de bot inconnu « {} »", r.niveau))?;
        let nom_bot = bot.name().to_string();
        let mut s = Session {
            g,
            humain: PlayerId::P0,
            reglages: r,
            bot,
            nom_bot,
            historique: History::new(),
            journal: Vec::new(),
            tour: 0,
            film: Vec::new(),
            film_no: 0,
            touchees: Vec::new(),
            coups: Vec::new(),
            stats: Stats::default(),
        };
        for (n, &i) in coups.iter().enumerate() {
            s.lire_evenements();
            let Some(d) = s.g.pending_decision() else {
                return Err(format!(
                    "sauvegarde incompatible : la partie est finie au coup {} sur {}",
                    n + 1,
                    coups.len()
                ));
            };
            if i as usize >= d.options.len() {
                return Err(format!(
                    "sauvegarde incompatible au coup {} (le moteur ou les decks ont changé ?)",
                    n + 1
                ));
            }
            if d.player == s.humain {
                s.historique.record(&Instantane {
                    g: s.g.clone(),
                    journal: s.journal.len(),
                    tour: s.tour,
                    coups: s.coups.len(),
                    stats: s.stats,
                });
            }
            s.g.apply(i as usize)
                .map_err(|e| format!("sauvegarde incompatible au coup {} : {e:?}", n + 1))?;
            s.coups.push(i);
        }
        s.avancer()?;
        s.film.clear();
        Ok(s)
    }

    /// Les choix appliqués au moteur depuis le début de la manche.
    pub fn coups(&self) -> &[u32] {
        &self.coups
    }

    pub fn journal(&self) -> &[Ligne] {
        &self.journal
    }

    /// Nombre de choix qu'on peut annuler.
    pub fn annulables(&self) -> usize {
        self.historique.len()
    }

    /// Recopie les nouveaux événements dans le journal ; renvoie ceux qui
    /// s'animent dans la page.
    fn lire_evenements(&mut self) -> Vec<Value> {
        let evenements = self
            .g
            .record
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        let mut anim = Vec::new();
        for e in &evenements {
            self.compter(e);
            if let Some(a) = self.animation(e) {
                anim.push(a);
            }
            let Some(texte) = vue::evenement(&self.g, self.humain, e) else {
                continue;
            };
            let debut = matches!(e, Event::StartOfTurn(_));
            if debut {
                self.tour += 1;
            }
            self.journal.push(Ligne {
                tour: self.tour,
                debut,
                texte,
            });
        }
        anim
    }

    /// Chiffres de fin de partie (voir `Stats`).
    fn compter(&mut self, e: &Event) {
        let h = self.humain;
        match *e {
            Event::Damage {
                by,
                target,
                target_obj: None,
                amount,
                ..
            } => {
                if target == h {
                    self.stats.subis += amount as u32;
                } else if by == h {
                    self.stats.infliges += amount as u32;
                }
            }
            Event::Defend {
                defender,
                from_hand,
                ..
            } if defender == h => {
                if from_hand {
                    self.stats.blocs_main += 1;
                } else {
                    self.stats.blocs_autres += 1;
                }
            }
            _ => {}
        }
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// `vous` ou `bot`.
    fn qui(&self, p: PlayerId) -> &'static str {
        if p == self.humain {
            "vous"
        } else {
            "bot"
        }
    }

    /// Un événement du moteur, tel que la page l'anime (seulement ce que
    /// l'humain peut voir : une carte cachée n'a pas de numéro).
    fn animation(&mut self, e: &Event) -> Option<Value> {
        let g = &self.g;
        let vu = |id: ObjectId| {
            let id = g.obj(id).proxy_of.unwrap_or(id);
            vue::visible(g, self.humain, id).then_some(id.0)
        };
        Some(match *e {
            Event::StartOfTurn(p) => json!({ "t": "tour", "par": self.qui(p) }),
            Event::Played { player, card, .. } => {
                json!({ "t": "joue", "par": self.qui(player), "carte": vu(card) })
            }
            Event::Attack {
                attack,
                attacker,
                target,
                target_obj,
                ..
            } => {
                self.touchees.retain(|&a| a != attack);
                json!({
                    "t": "attaque",
                    "par": self.qui(attacker),
                    "cible": self.qui(target),
                    "allie": target_obj.is_some(),
                    "carte": vu(attack),
                })
            }
            Event::Defend { card, defender, .. } => {
                json!({ "t": "defense", "par": self.qui(defender), "carte": vu(card) })
            }
            Event::Hit {
                attack,
                attacker,
                target,
                target_obj,
                damage,
                effect,
                ..
            } => {
                self.touchees.push(attack);
                json!({
                    "t": "touche",
                    "par": self.qui(attacker),
                    "cible": self.qui(target),
                    "allie": target_obj.is_some(),
                    "degats": damage,
                    "effet": effect,
                })
            }
            Event::ChainLinkResolved { attack, attacker } if !self.touchees.contains(&attack) => {
                json!({ "t": "bloque", "par": self.qui(attacker), "cible": self.qui(attacker.opponent()) })
            }
            Event::Damage {
                target,
                target_obj,
                amount,
                arcane,
                ..
            } => json!({
                "t": "degats",
                "cible": self.qui(target),
                "allie": target_obj.is_some(),
                "n": amount,
                "arcane": arcane,
            }),
            Event::Pitched { player, card } => {
                json!({ "t": "pitch", "par": self.qui(player), "carte": vu(card) })
            }
            Event::CombatChainClosed => json!({ "t": "chaine_fermee" }),
            _ => return None,
        })
    }

    /// Le plateau seul (sans question ni journal), vu par l'humain.
    fn plateau(&self) -> Value {
        let g = &self.g;
        let h = self.humain;
        let pile: Vec<String> = g.stack.iter().rev().map(|l| vue::couche(g, h, l)).collect();
        json!({
            "tour": g.turn,
            "votre_tour": g.turn_player == h,
            "vous": joueur(g, h, h),
            "bot": joueur(g, h, h.opponent()),
            "combat": combat(g, h),
            "pile": pile,
            "couches": couches(g, h),
        })
    }

    /// Lit les événements et ajoute une image au film si quelque chose a
    /// changé.
    fn tourner(&mut self) {
        let anim = self.lire_evenements();
        let plateau = self.plateau();
        let pareil = self.film.last().is_some_and(|d| d["plateau"] == plateau);
        if anim.is_empty() && pareil {
            return;
        }
        self.film
            .push(json!({ "plateau": plateau, "evenements": anim }));
    }

    /// Le moteur avance ; le bot répond à ses décisions, jusqu'à une
    /// décision de l'humain ou la fin de la partie.
    fn avancer(&mut self) -> Result<(), String> {
        loop {
            self.tourner();
            let Some(d) = self.g.pending_decision().cloned() else {
                return Ok(());
            };
            if d.player == self.humain {
                return Ok(());
            }
            let i = self.bot.choose(&self.g, &d);
            self.g
                .apply(i)
                .map_err(|e| format!("choix du bot refusé : {e:?}"))?;
            self.coups.push(i as u32);
        }
    }

    /// L'humain choisit l'option `i` (à partir de 0).
    pub fn choisir(&mut self, i: usize) -> Result<(), String> {
        let Some(d) = self.g.pending_decision() else {
            return Err("la partie est terminée".into());
        };
        if d.player != self.humain {
            return Err("ce n'est pas à vous de choisir".into());
        }
        if i >= d.options.len() {
            return Err(format!(
                "option {} inexistante (il y en a {})",
                i + 1,
                d.options.len()
            ));
        }
        self.historique.record(&Instantane {
            g: self.g.clone(),
            journal: self.journal.len(),
            tour: self.tour,
            coups: self.coups.len(),
            stats: self.stats,
        });
        self.film.clear();
        self.film_no += 1;
        self.g
            .apply(i)
            .map_err(|e| format!("choix refusé par le moteur : {e:?}"))?;
        self.coups.push(i as u32);
        self.avancer()
    }

    /// Plusieurs choix de l'humain d'un coup, annulés ensemble : pour
    /// chaque carte de `cartes`, l'option de la décision en cours qui porte
    /// sur cette carte (défendre, pitcher, remettre sous le deck…), puis
    /// « passer » si `passer`. La page s'en sert quand on prépare une
    /// défense carte par carte avant de la valider (comme dans l'Arsenal
    /// JS). On s'arrête dès que la décision n'est plus à l'humain.
    pub fn choisir_cartes(&mut self, cartes: &[u32], passer: bool) -> Result<(), String> {
        let genre = match self.g.pending_decision() {
            Some(d) if d.player == self.humain => d.kind,
            Some(_) => return Err("ce n'est pas à vous de choisir".into()),
            None => return Err("la partie est terminée".into()),
        };
        self.historique.record(&Instantane {
            g: self.g.clone(),
            journal: self.journal.len(),
            tour: self.tour,
            coups: self.coups.len(),
            stats: self.stats,
        });
        self.film.clear();
        self.film_no += 1;
        let h = self.humain;
        let mut etapes: Vec<Option<u32>> = cartes.iter().map(|&c| Some(c)).collect();
        if passer {
            etapes.push(None);
        }
        for quoi in etapes {
            // Même genre de décision qu'au départ : sinon (la dernière carte
            // est partie toute seule, la défense est finie…), on s'arrête.
            let Some(d) = self
                .g
                .pending_decision()
                .filter(|d| d.player == h && d.kind == genre)
            else {
                break;
            };
            let i = d.options.iter().position(|&c| match quoi {
                Some(id) => carte_de_l_option(&self.g, h, c).map(|o| o.0) == Some(id),
                None => matches!(c, Choice::Pass),
            });
            let Some(i) = i else {
                // Annule ce qui a déjà été appliqué : tout ou rien.
                self.annuler();
                return Err(match quoi {
                    Some(id) => format!("la carte {id} ne fait pas partie des choix"),
                    None => "« passer » ne fait pas partie des choix".into(),
                });
            };
            self.g
                .apply(i)
                .map_err(|e| format!("choix refusé par le moteur : {e:?}"))?;
            self.coups.push(i as u32);
            self.tourner();
        }
        self.avancer()
    }

    /// Annule le dernier choix de l'humain (et les réponses du bot qui ont
    /// suivi). `false` : rien à annuler.
    pub fn annuler(&mut self) -> bool {
        match self.historique.undo() {
            Some(avant) => {
                self.g = avant.g;
                self.journal.truncate(avant.journal);
                self.tour = avant.tour;
                self.coups.truncate(avant.coups);
                self.stats = avant.stats;
                self.film.clear();
                self.film_no += 1;
                self.touchees.clear();
                true
            }
            None => false,
        }
    }

    /// Fiche et texte officiel d'une carte, si l'humain a le droit de la voir.
    pub fn carte(&self, id: u32, textes: Option<&Textes>) -> Value {
        let g = &self.g;
        if id as usize >= g.objects.len() || !vue::visible(g, self.humain, ObjectId(id)) {
            return json!({ "erreur": "carte cachée ou inconnue" });
        }
        let id = ObjectId(id);
        let d = g.def(id);
        let (typ, texte) = match textes.and_then(|t| t.texte(&d.name, d.color)) {
            Some((t, x)) => (t, x),
            None if textes.is_none() => (
                String::new(),
                "(texte officiel indisponible : lancez « bash tools/fetch_cards.sh »)".into(),
            ),
            None => (String::new(), "(texte officiel introuvable)".into()),
        };
        json!({
            "carte": carte(g, self.humain, id),
            "fiche": vue::fiche(g, id),
            "type": typ,
            "texte": texte,
        })
    }

    /// Tout ce que la page affiche, vu par l'humain.
    pub fn etat(&self) -> Value {
        let g = &self.g;
        let h = self.humain;
        let decision = match g.pending_decision() {
            Some(d) if d.player == h => {
                let options: Vec<Value> = d
                    .options
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        json!({
                            "i": i,
                            "texte": vue::option(g, d, c),
                            "carte": carte_de_l_option(g, h, c).map(|o| o.0),
                            "passer": matches!(c, Choice::Pass),
                            "action": action_de(c),
                        })
                    })
                    .collect();
                // Pitch : ce qu'il reste à payer (la page l'affiche sur les
                // symboles de ressources, comme l'Arsenal JS).
                let a_payer = match (d.kind, g.tasks.last()) {
                    (DecisionKind::Pitch, Some(fab_engine::state::Task::Pay { resources, .. })) => {
                        Some(*resources)
                    }
                    _ => None,
                };
                json!({
                    "titre": vue::titre_decision(g, d),
                    "genre": genre(d.kind),
                    "options": options,
                    "a_payer": a_payer,
                    // La carte qui pose la question (montrée à côté).
                    "source": vue::source_carte(g).map(|c| crate::match_web::carte_pool(&g.db, c)),
                })
            }
            _ => Value::Null,
        };
        let pile: Vec<String> = g.stack.iter().rev().map(|l| vue::couche(g, h, l)).collect();
        let journal: Vec<Value> = self
            .journal
            .iter()
            .map(|l| json!({ "tour": l.tour, "debut": l.debut, "texte": l.texte }))
            .collect();
        json!({
            "reglages": {
                "deck": self.reglages.deck,
                "deck_bot": self.reglages.deck_bot,
                "niveau": self.reglages.niveau,
                "nom_bot": self.nom_bot,
                "graine": self.reglages.graine.to_string(),
            },
            "tour": g.turn,
            "votre_tour": g.turn_player == h,
            "vous": joueur(g, h, h),
            "bot": joueur(g, h, h.opponent()),
            "combat": combat(g, h),
            "pile": pile,
            "decision": decision,
            "couches": couches(g, h),
            "priorite": g.priority.map(|p| self.qui(p)),
            "phase_action": g.action_phase,
            "stats": {
                "infliges": self.stats.infliges,
                "subis": self.stats.subis,
                "blocs_main": self.stats.blocs_main,
                "blocs_autres": self.stats.blocs_autres,
            },
            "annulables": self.historique.len(),
            "fini": g.is_over(),
            "resultat": if g.is_over() { Some(vue::resultat(g, h)) } else { None },
            "journal": journal,
            "film": self.film,
            "film_no": self.film_no,
        })
    }
}

/// Ce que fait une option, pour le glisser-déposer de la page : on lâche
/// la carte sur la chaîne (`jouer`, `defendre`, `activer`), sur le pitch
/// (`pitch`) ou sur l'arsenal (`arsenal`).
fn action_de(c: Choice) -> &'static str {
    match c {
        Choice::Play(_) => "jouer",
        Choice::Activate(..) => "activer",
        Choice::Pitch(_) => "pitch",
        Choice::Defend(_) => "defendre",
        Choice::Arsenal(_) => "arsenal",
        _ => "autre",
    }
}

fn genre(k: DecisionKind) -> &'static str {
    match k {
        DecisionKind::Priority => "priorite",
        DecisionKind::Defend => "defense",
        DecisionKind::Pitch => "pitch",
        DecisionKind::Arsenal => "arsenal",
        DecisionKind::PitchOrder => "ordre_pitch",
        DecisionKind::Target | DecisionKind::AttackTarget => "cible",
        DecisionKind::Mode => "mode",
        DecisionKind::Effect => "effet",
        _ => "autre",
    }
}

/// La pile, du dessus (résolue en premier) vers le dessous, pour la page :
/// la carte (si l'humain la voit), le genre de la couche, son contrôleur
/// et la ligne de texte de `vue::couche`.
fn couches(g: &GameState, viewer: PlayerId) -> Vec<Value> {
    use fab_engine::state::LayerKind;
    g.stack
        .iter()
        .rev()
        .map(|l| {
            let genre = match l.kind {
                LayerKind::Card => "carte",
                LayerKind::Activated(a) => {
                    if g.db
                        .get(l.card)
                        .activated
                        .get(a as usize)
                        .is_some_and(|x| x.attack)
                    {
                        "attaque"
                    } else {
                        "capacité"
                    }
                }
                _ => "déclencheur",
            };
            let id = l.object.id;
            let carte = (id.0 as usize) < g.objects.len() && vue::visible(g, viewer, id);
            json!({
                "carte": if carte { self::carte(g, viewer, id) } else { Value::Null },
                "genre": genre,
                "par_vous": l.controller == viewer,
                "texte": vue::couche(g, viewer, l),
            })
        })
        .collect()
}

/// Effets en cours qui touchent un joueur, en mots courts (pastilles
/// « Effets en cours » de l'Arsenal JS). Seulement l'affichage d'effets que
/// le moteur garde déjà : aucune règle ici.
fn effets(g: &GameState, p: PlayerId) -> Vec<String> {
    use fab_engine::Keywords;
    let ps = g.player(p);
    let mut v = Vec::new();
    if ps.marked {
        v.push("Marqué".to_string());
    }
    if g.lost_abilities(ps.hero) {
        v.push("Capacités du héros perdues".into());
    }
    for e in g.next_attack_effects.iter().filter(|e| e.player == p) {
        let mut t = Vec::new();
        if e.delta.power != 0 {
            t.push(format!("{:+}{{p}}", e.delta.power));
        }
        if e.delta.defense != 0 {
            t.push(format!("{:+}{{d}}", e.delta.defense));
        }
        if e.delta.grants.has(Keywords::GO_AGAIN) {
            t.push("go again".into());
        }
        if e.delta.grants.has(Keywords::DOMINATE) {
            t.push("dominate".into());
        }
        let quoi = if e.weapon.is_some() {
            "Prochaine attaque de l'arme"
        } else if e.cards_only {
            "Prochaine carte d'attaque"
        } else {
            "Prochaine attaque"
        };
        if t.is_empty() {
            v.push(quoi.into());
        } else {
            v.push(format!("{quoi} : {}", t.join(", ")));
        }
    }
    for r in g
        .cost_reductions
        .iter()
        .filter(|r| r.player == p && r.turn == g.turn)
    {
        v.push(format!("Prochaine carte : coût −{}", r.amount));
    }
    if g.instant_permissions
        .iter()
        .any(|r| r.player == p && r.turn == g.turn)
    {
        v.push("Prochaine carte jouable comme un instant".into());
    }
    if g.no_power_gain.iter().any(|&(q, _)| q == p) {
        v.push("Attaques : pas de gain de {p}".into());
    }
    for x in g.preventions.iter().filter(|x| x.player == p) {
        match x.remaining {
            Some(n) => v.push(format!("Prévention {n}")),
            None => v.push("Prévention des prochains dégâts".into()),
        }
    }
    for x in g.play_permissions.iter().filter(|x| x.player == p) {
        if g.is_current(x.card) && vue::visible(g, p, x.card.id) {
            let quand = if x.turn == g.turn {
                "ce tour"
            } else {
                "plus tard"
            };
            v.push(format!(
                "Jouable depuis le banni ({quand}) : {}",
                vue::nom(g, x.card.id)
            ));
        }
    }
    v
}

/// La carte concernée par une option, si l'humain peut la voir : cliquer
/// sur la carte du plateau propose cette option.
pub fn carte_de_l_option(g: &GameState, viewer: PlayerId, c: Choice) -> Option<ObjectId> {
    let id = match c {
        Choice::Play(id)
        | Choice::Activate(id, _)
        | Choice::Pitch(id)
        | Choice::Defend(id)
        | Choice::Arsenal(id)
        | Choice::PitchToBottom(id)
        | Choice::Object(id) => id,
        Choice::Target(Target::Object(r)) => r.id,
        _ => return None,
    };
    let id = g.obj(id).proxy_of.unwrap_or(id);
    vue::visible(g, viewer, id).then_some(id)
}

/// Le genre d'une carte, en français (« action », « arme »…).
pub fn genre_de(d: &CardDef) -> &'static str {
    let est = |t| d.types.has(t);
    if est(Types::WEAPON) {
        "arme"
    } else if est(Types::EQUIPMENT) {
        "équipement"
    } else if est(Types::ATTACK_REACTION) {
        "réaction d'attaque"
    } else if est(Types::DEFENSE_REACTION) {
        "réaction de défense"
    } else if est(Types::INSTANT) {
        "instant"
    } else if est(Types::ACTION) {
        "action"
    } else if est(Types::TOKEN) {
        "token"
    } else if est(Types::RESOURCE) {
        "ressource"
    } else {
        ""
    }
}

pub fn nom_couleur(c: Option<Color>) -> &'static str {
    match c {
        Some(Color::Red) => "rouge",
        Some(Color::Yellow) => "jaune",
        Some(Color::Blue) => "bleu",
        None => "",
    }
}

/// Une carte vue par `viewer` : `{"cache": true}` si elle lui est cachée.
pub fn carte(g: &GameState, viewer: PlayerId, id: ObjectId) -> Value {
    if !vue::visible(g, viewer, id) {
        return json!({ "cache": true });
    }
    let o = g.obj(id);
    let id = o.proxy_of.unwrap_or(id);
    let o = g.obj(id);
    let d = g.def(id);
    let details = match o.zone {
        Zone::Permanent => vue::details_permanent(g, id),
        Zone::Weapon | Zone::Head | Zone::Chest | Zone::Arms | Zone::Legs => {
            vue::details_equipement(g, id)
        }
        _ => Vec::new(),
    };
    let genre = genre_de(d);
    let (image, nom_fr) = images::champs(d);
    json!({
        "id": id.0,
        "nom": d.name,
        "nom_fr": nom_fr,
        "image": image,
        "couleur": nom_couleur(d.color),
        "genre": genre,
        "cout": d.cost,
        "x": d.x_cost,
        "puissance": d.power.map(|_| g.power(id)),
        "defense": d.defense.map(|_| g.defense(id)),
        // Valeurs imprimées : la page signale ce qui a changé sur l'image.
        "puissance_imprimee": d.power,
        "defense_imprimee": d.defense,
        "pitch": d.pitch,
        "engage": o.tapped,
        // Une carte en main est privée, pas « face cachée » au sens du plateau.
        "face_cachee": o.face_down && o.zone != Zone::Hand,
        "details": details,
        // Emplacement sur le tapis (armes et équipement), sinon null.
        "emplacement": emplacement(o.zone),
    })
}

/// Emplacement d'une arme ou d'un équipement sur le tapis (zones
/// d'équipement et d'armes), pour placer la carte comme sur un playmat.
fn emplacement(z: Zone) -> Option<&'static str> {
    match z {
        Zone::Weapon => Some("arme"),
        Zone::Head => Some("tete"),
        Zone::Chest => Some("torse"),
        Zone::Arms => Some("bras"),
        Zone::Legs => Some("jambes"),
        _ => None,
    }
}

fn cartes(g: &GameState, viewer: PlayerId, ids: &[ObjectId]) -> Vec<Value> {
    ids.iter().map(|&id| carte(g, viewer, id)).collect()
}

/// Le bloc d'un joueur. La main adverse n'est donnée que par son nombre.
fn joueur(g: &GameState, viewer: PlayerId, p: PlayerId) -> Value {
    let ps = g.player(p);
    let mut equipement: Vec<ObjectId> = ps.weapons.clone();
    equipement.extend(ps.equipment());
    let permanents: Vec<ObjectId> = g
        .permanents
        .iter()
        .copied()
        .filter(|&id| g.obj(id).controller == p)
        .collect();
    json!({
        "heros": vue::heros(g, p),
        "heros_id": ps.hero.0,
        "heros_image": images::champs(g.def(ps.hero)).0,
        "vie": ps.life,
        "marque": ps.marked,
        "deck": ps.deck.len(),
        "points_action": ps.action_points,
        "ressources": ps.resource_points,
        "equipement": cartes(g, viewer, &equipement),
        "main": if p == viewer { Value::from(cartes(g, viewer, &ps.hand)) } else { Value::Null },
        "main_nombre": ps.hand.len(),
        "arsenal": ps.arsenal.map(|a| carte(g, viewer, a)),
        "pitch": cartes(g, viewer, &ps.pitch),
        "permanents": cartes(g, viewer, &permanents),
        "cimetiere": cartes(g, viewer, &ps.graveyard),
        "bannies": cartes(g, viewer, &ps.banished),
        "effets": effets(g, p),
    })
}

/// La chaîne de combat.
fn combat(g: &GameState, viewer: PlayerId) -> Value {
    if !g.combat.open {
        return Value::Null;
    }
    let maillons: Vec<Value> = g
        .combat
        .links
        .iter()
        .map(|l| {
            let a = l.attack.id;
            let actif = g.is_current(l.attack) && g.obj(a).zone == Zone::CombatChain;
            let cible = match l.target_obj {
                Some(r) => vue::nom_pour(g, viewer, r.id),
                None => vue::heros(g, l.target),
            };
            let total: i16 = l.defenders.iter().map(|&d| g.defense(d).max(0)).sum();
            let mots = g.keywords(a);
            json!({
                "go_again": actif && mots.has(fab_engine::Keywords::GO_AGAIN),
                "dominate": actif && mots.has(fab_engine::Keywords::DOMINATE),
                "attaque": carte(g, viewer, a),
                "par_vous": l.attacker == viewer,
                "cible": cible,
                "puissance": if actif { Some(g.power(a)) } else { None },
                "touche": l.hit,
                "defenseurs": cartes(g, viewer, &l.defenders),
                "defense_totale": total,
            })
        })
        .collect();
    json!({
        "etape": g.combat.step.map(vue::etape).unwrap_or("—"),
        // Rang de l'étape dans la barre « Layer · Attaque · Défense ·
        // Réactions · Dégâts · Résolution » (0 à 5).
        "etape_rang": g.combat.step.map(|s| s as u8),
        "maillons": maillons,
    })
}
