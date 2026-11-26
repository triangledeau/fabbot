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

/// Ce qu'on mémorise avant chaque choix de l'humain, pour l'annuler.
#[derive(Clone)]
struct Instantane {
    g: GameState,
    journal: usize,
    tour: u32,
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
    pub fn avec_partie(mut g: GameState, r: Reglages) -> Result<Session, String> {
        if g.record.is_none() {
            g.record = Some(Vec::new());
        }
        let bot = creer_agent(&r.niveau, r.graine ^ 0x5A5A).ok_or_else(|| {
            format!(
                "niveau de bot inconnu « {} » (connus : {}, b2-N)",
                r.niveau,
                BOTS.join(", ")
            )
        })?;
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
        };
        s.avancer()?;
        Ok(s)
    }

    pub fn journal(&self) -> &[Ligne] {
        &self.journal
    }

    /// Nombre de choix qu'on peut annuler.
    pub fn annulables(&self) -> usize {
        self.historique.len()
    }

    /// Recopie les nouveaux événements dans le journal.
    fn lire_evenements(&mut self) {
        let evenements = self
            .g
            .record
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        for e in &evenements {
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
    }

    /// Le moteur avance ; le bot répond à ses décisions, jusqu'à une
    /// décision de l'humain ou la fin de la partie.
    fn avancer(&mut self) -> Result<(), String> {
        loop {
            self.lire_evenements();
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
        });
        self.g
            .apply(i)
            .map_err(|e| format!("choix refusé par le moteur : {e:?}"))?;
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
                        })
                    })
                    .collect();
                json!({
                    "titre": vue::titre_decision(g, d),
                    "genre": genre(d.kind),
                    "options": options,
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
            "annulables": self.historique.len(),
            "fini": g.is_over(),
            "resultat": if g.is_over() { Some(vue::resultat(g, h)) } else { None },
            "journal": journal,
        })
    }
}

fn genre(k: DecisionKind) -> &'static str {
    match k {
        DecisionKind::Priority => "priorite",
        DecisionKind::Defend => "defense",
        DecisionKind::Pitch => "pitch",
        DecisionKind::Arsenal => "arsenal",
        _ => "autre",
    }
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
    })
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
            json!({
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
        "maillons": maillons,
    })
}
