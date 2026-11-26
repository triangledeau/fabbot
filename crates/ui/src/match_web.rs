//! Un match joué depuis la page web : plusieurs manches, et avant chacune
//! la « phase de side » (CR 4.1.3 à 4.1.6) :
//!
//! 1. le joueur désigné choisit qui commence (CR 4.1.3 ; première manche :
//!    tirage au sort, ensuite le perdant de la manche précédente) ;
//! 2. chacun choisit ses cartes d'arène **sans voir** celles de l'autre
//!    (CR 4.1.4) ;
//! 3. les arènes sont présentées (CR 4.1.5), puis chacun choisit les 40
//!    cartes de son deck (CR 4.1.6, TRP 7.4) ;
//! 4. la manche se joue ([`Session`]).
//!
//! Le bot choisit avec la table apprise (`fab_ai::side::SideAppris`). Son
//! arène n'est envoyée au navigateur qu'après le choix de l'humain, son deck
//! jamais.

use crate::textes::Textes;
use crate::web::{genre_de, nom_couleur, Reglages, Session};
use fab_ai::side::SideAppris;
use fab_cards::real;
use fab_engine::card::CardDb;
use fab_engine::state::equipment_zone;
use fab_engine::{CardId, CardPool, FormatConfig, PlayerId, Zone};
use fab_match::{lancer_manche, InfoSide, Manche, MatchScore, Side, SideListe};
use serde_json::{json, Value};
use std::sync::Arc;

/// L'humain est toujours le joueur 0, le bot le joueur 1.
const HUMAIN: PlayerId = PlayerId::P0;
const BOT: PlayerId = PlayerId::P1;

/// Où en est la préparation de la manche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etape {
    /// L'humain doit choisir qui commence (CR 4.1.3).
    PremierJoueur,
    /// L'humain choisit ses cartes d'arène (CR 4.1.4).
    Arene,
    /// L'humain choisit son deck (CR 4.1.6).
    Deck,
    /// La manche se joue (ou vient de finir).
    Partie,
}

pub struct MatchWeb {
    pub reglages: Reglages,
    db: Arc<CardDb>,
    pools: [CardPool; 2],
    score: MatchScore,
    side_bot: SideAppris,
    pub etape: Etape,
    /// Joueur qui choisit le premier joueur de cette manche.
    chooser: PlayerId,
    premier: Option<PlayerId>,
    arenes: [Option<Vec<CardId>>; 2],
    /// Dernier choix de l'humain (proposé par défaut à la manche suivante).
    proposition: (Vec<CardId>, Vec<CardId>),
    /// La manche en cours ou qui vient de finir.
    pub session: Option<Session>,
    /// Le résultat de la manche en cours a-t-il été compté ?
    compte: bool,
    /// Les choix de l'humain depuis le début du match, dans l'ordre
    /// (sauvegarde, lot 3) : `{"premier": "vous"}`, `{"arene": [..]}`,
    /// `{"deck": [..]}`, `{"partie": [coups…]}` (manche finie),
    /// `{"manche_suivante": true}`. Voir `crate::sauvegarde`.
    etapes: Vec<Value>,
    /// Coups à rejouer dans la prochaine manche créée (chargement).
    rejeu: Option<Vec<u32>>,
}

/// Ce que sait `moi` au moment de choisir.
fn info<'a>(
    db: &'a CardDb,
    pools: &'a [CardPool; 2],
    moi: PlayerId,
    premier: PlayerId,
) -> InfoSide<'a> {
    InfoSide {
        db,
        moi,
        pool: &pools[moi.index()],
        heros_adverse: pools[moi.opponent().index()].hero,
        premier,
        deck_size: taille_deck(),
    }
}

fn taille_deck() -> usize {
    FormatConfig::silver_age().deck_size as usize
}

fn nom_zone(z: Zone) -> &'static str {
    match z {
        Zone::Weapon => "Armes",
        Zone::Head => "Tête",
        Zone::Chest => "Torse",
        Zone::Arms => "Bras",
        Zone::Legs => "Jambes",
        _ => "Autre",
    }
}

/// Une carte du card-pool (pas encore un objet de partie).
fn carte_pool(db: &CardDb, c: CardId) -> Value {
    let d = db.get(c);
    let (image, nom_fr) = crate::images::champs(d);
    json!({
        "carte_id": c.0,
        "nom": d.name,
        "nom_fr": nom_fr,
        "image": image,
        "couleur": nom_couleur(d.color),
        "genre": genre_de(d),
        "cout": d.cost,
        "x": d.x_cost,
        "puissance": d.power,
        "defense": d.defense,
        "pitch": d.pitch,
    })
}

impl MatchWeb {
    /// Nouveau match : prépare la première manche.
    pub fn nouveau(r: Reglages) -> Result<MatchWeb, String> {
        if !(1..=3).contains(&r.manches) {
            return Err("nombre de manches à gagner : 1, 2 ou 3".into());
        }
        if !fab_ai::bot_connu(&r.niveau) {
            return Err(format!(
                "niveau de bot inconnu « {} » (connus : {}, b2-N)",
                r.niveau,
                fab_ai::BOTS.join(", ")
            ));
        }
        let db = Arc::new(real::build());
        let pools = [
            crate::charger_pool(&db, &r.deck)?,
            crate::charger_pool(&db, &r.deck_bot)?,
        ];
        let proposition = pools[0].choix_par_ordre(&db, taille_deck());
        let side_bot = SideAppris::par_defaut(&db);
        let mut m = MatchWeb {
            score: MatchScore::new(r.manches),
            reglages: r,
            db,
            pools,
            side_bot,
            etape: Etape::Arene,
            chooser: HUMAIN,
            premier: None,
            arenes: [None, None],
            proposition,
            session: None,
            compte: false,
            etapes: Vec::new(),
            rejeu: None,
        };
        // CR 4.1.3 : première manche, joueur tiré au sort (par la graine).
        let tirage = PlayerId((m.reglages.graine % 2) as u8);
        m.preparer(tirage);
        Ok(m)
    }

    /// Début de la préparation d'une manche.
    fn preparer(&mut self, chooser: PlayerId) {
        self.chooser = chooser;
        self.premier = None;
        self.arenes = [None, None];
        self.compte = false;
        if chooser == HUMAIN {
            self.etape = Etape::PremierJoueur;
        } else {
            let i = info(&self.db, &self.pools, BOT, BOT);
            let p = self.side_bot.premier_joueur(&i);
            self.choisir_premier(p);
        }
    }

    fn choisir_premier(&mut self, p: PlayerId) {
        self.premier = Some(p);
        // CR 4.1.4 : le bot choisit son arène maintenant, sans voir celle
        // de l'humain ; elle n'est montrée qu'après le choix de l'humain.
        let a = self.side_bot.arene(&info(&self.db, &self.pools, BOT, p));
        self.arenes[BOT.index()] = Some(a);
        self.etape = Etape::Arene;
    }

    /// CR 4.1.3 : l'humain désigne le premier joueur (`vous` ou `bot`).
    pub fn premier_joueur(&mut self, qui: &str) -> Result<(), String> {
        if self.etape != Etape::PremierJoueur {
            return Err("ce n'est pas le moment de choisir qui commence".into());
        }
        let p = match qui {
            "vous" => HUMAIN,
            "bot" => BOT,
            _ => return Err("« vous » ou « bot » attendu".into()),
        };
        self.choisir_premier(p);
        self.etapes.push(json!({ "premier": qui }));
        Ok(())
    }

    /// CR 4.1.4 : l'humain choisit ses cartes d'arène, par leur position
    /// dans son card-pool d'arène.
    pub fn arene(&mut self, indices: &[usize]) -> Result<(), String> {
        if self.etape != Etape::Arene {
            return Err("ce n'est pas le moment de choisir l'équipement".into());
        }
        let pool = &self.pools[HUMAIN.index()];
        let mut vus = Vec::new();
        let mut cartes = Vec::new();
        for &i in indices {
            if i >= pool.arena.len() || vus.contains(&i) {
                return Err(format!("carte d'arène {i} inexistante"));
            }
            vus.push(i);
            cartes.push(pool.arena[i]);
        }
        pool.verifier_arene(&self.db, &cartes)
            .map_err(|e| self.message(e))?;
        self.arenes[HUMAIN.index()] = Some(cartes);
        self.etape = Etape::Deck;
        self.etapes.push(json!({ "arene": indices }));
        Ok(())
    }

    /// CR 4.1.6 : l'humain choisit son deck : le nombre d'exemplaires de
    /// chaque carte distincte de son card-pool (dans l'ordre de
    /// [`Self::cartes_distinctes`]).
    pub fn deck(&mut self, quantites: &[usize]) -> Result<(), String> {
        if self.etape != Etape::Deck {
            return Err("ce n'est pas le moment de choisir le deck".into());
        }
        let distinctes = self.cartes_distinctes();
        if quantites.len() != distinctes.len() {
            return Err(format!(
                "{} quantités attendues, {} reçues",
                distinctes.len(),
                quantites.len()
            ));
        }
        let mut deck = Vec::new();
        for (&(c, _), &n) in distinctes.iter().zip(quantites) {
            deck.extend(std::iter::repeat_n(c, n));
        }
        let taille = taille_deck();
        let pool = &self.pools[HUMAIN.index()];
        pool.verifier_deck(&self.db, &deck, taille)
            .map_err(|e| self.message(e))?;
        let arene_h = self.arenes[HUMAIN.index()].clone().unwrap();
        let arene_b = self.arenes[BOT.index()].clone().unwrap();
        // Le bot choisit son deck en connaissant l'arène de l'humain.
        let premier = self.premier.unwrap();
        let deck_bot = self.side_bot.deck(
            &info(&self.db, &self.pools, BOT, premier),
            &arene_b,
            &arene_h,
        );
        let loadouts = [
            pool.loadout(&self.db, arene_h.clone(), deck.clone(), taille)
                .map_err(|e| self.message(e))?,
            self.pools[BOT.index()]
                .loadout(&self.db, arene_b, deck_bot, taille)
                .or_else(|_| {
                    // Par sécurité : le choix « liste ».
                    let i = info(&self.db, &self.pools, BOT, premier);
                    let a = SideListe.arene(&i);
                    let d = SideListe.deck(&i, &a, &arene_h);
                    self.pools[BOT.index()].loadout(&self.db, a, d, taille)
                })
                .map_err(|e| format!("choix du bot : {e:?}"))?,
        ];
        let manche = Manche {
            loadouts,
            chooser: self.chooser,
            premier,
        };
        let graine = self
            .reglages
            .graine
            .wrapping_add(1_000_003u64.wrapping_mul(self.score.games_played() as u64));
        let g = lancer_manche(self.db.clone(), &manche, graine, 200)
            .map_err(|e| format!("mise en place : {e:?}"))?;
        let mut r = self.reglages.clone();
        r.graine = graine;
        self.session = Some(match self.rejeu.take() {
            Some(coups) => Session::rejouer(g, r, &coups)?,
            None => Session::avec_partie(g, r)?,
        });
        self.etapes.push(json!({ "deck": quantites }));
        self.proposition = (arene_h, deck);
        self.etape = Etape::Partie;
        self.compter();
        Ok(())
    }

    fn message(&self, e: fab_engine::ChoixInvalide) -> String {
        use fab_engine::ChoixInvalide as C;
        let nom = |c: CardId| self.db.get(c).name.clone();
        match e {
            C::HorsDuPool(c) => format!("{} : pas (ou plus) dans votre card-pool", nom(c)),
            C::PasUneCarteDArene(c) => format!("{} n'est pas une carte d'arène", nom(c)),
            C::PasUneCarteDeDeck(c) => {
                format!("{} : une carte d'arène ne va pas dans le deck (CR 4.1.6a)", nom(c))
            }
            C::ZoneOccupee(c) => format!(
                "{} : plus de place dans sa zone (une carte par zone, deux mains pour les armes, un seul off-hand)",
                nom(c)
            ),
            C::TailleDuDeck { attendu, recu } => {
                format!("le deck doit compter exactement {attendu} cartes ({recu} choisies)")
            }
        }
    }

    /// Compte le résultat de la manche finie (une seule fois).
    fn compter(&mut self) {
        let Some(s) = &self.session else { return };
        if self.compte || !s.g.is_over() {
            return;
        }
        if let Some(o) = s.g.outcome {
            self.score.record(o, self.chooser);
            self.compte = true;
        }
    }

    /// Après un choix ou une annulation dans la manche.
    pub fn apres_coup(&mut self) {
        if let Some(s) = &self.session {
            if !s.g.is_over() && self.compte {
                // Une annulation après la fin : la manche reprend. On
                // recrée le score sans cette manche.
                let mut score = MatchScore::new(self.reglages.manches);
                let n = self.score.games_played() - 1;
                for (o, c) in self.score.historique().into_iter().take(n) {
                    score.record(o, c);
                }
                self.score = score;
                self.compte = false;
            }
        }
        self.compter();
    }

    /// Passe à la manche suivante (le perdant choisit qui commence).
    pub fn manche_suivante(&mut self) -> Result<(), String> {
        let finie = self.session.as_ref().is_some_and(|s| s.g.is_over());
        if self.etape != Etape::Partie || !finie {
            return Err("la manche n'est pas finie".into());
        }
        if self.score.winner().is_some() {
            return Err("le match est terminé".into());
        }
        let chooser = self.score.next_chooser(self.chooser);
        if let Some(s) = &self.session {
            self.etapes.push(json!({ "partie": s.coups() }));
        }
        self.etapes.push(json!({ "manche_suivante": true }));
        self.preparer(chooser);
        Ok(())
    }

    /// Les cartes de deck distinctes du card-pool de l'humain et leur
    /// nombre d'exemplaires.
    pub fn cartes_distinctes(&self) -> Vec<(CardId, usize)> {
        let mut v: Vec<(CardId, usize)> = Vec::new();
        for &c in &self.pools[HUMAIN.index()].deck {
            match v.iter_mut().find(|(k, _)| *k == c) {
                Some(e) => e.1 += 1,
                None => v.push((c, 1)),
            }
        }
        v
    }

    /// Les cartes que l'humain a le droit de voir hors partie : son
    /// card-pool, les deux héros, et l'arène du bot une fois présentée.
    fn visible_hors_partie(&self, c: CardId) -> bool {
        let p = &self.pools[HUMAIN.index()];
        p.hero == c
            || p.arena.contains(&c)
            || p.deck.contains(&c)
            || self.pools[BOT.index()].hero == c
            || (matches!(self.etape, Etape::Deck | Etape::Partie)
                && self.arenes[BOT.index()]
                    .as_ref()
                    .is_some_and(|a| a.contains(&c)))
    }

    /// Texte officiel d'une carte du card-pool (écran de préparation).
    pub fn carte_pool(&self, c: u32, textes: Option<&Textes>) -> Value {
        if c as usize >= self.db.len() {
            return json!({ "erreur": "carte cachée ou inconnue" });
        }
        let c = CardId(c as u16);
        if !self.visible_hors_partie(c) {
            return json!({ "erreur": "carte cachée ou inconnue" });
        }
        let d = self.db.get(c);
        let (typ, texte) = match textes.and_then(|t| t.texte(&d.name, d.color)) {
            Some(x) => x,
            None if textes.is_none() => (
                String::new(),
                "(texte officiel indisponible : lancez « bash tools/fetch_cards.sh »)".into(),
            ),
            None => (String::new(), "(texte officiel introuvable)".into()),
        };
        let couleur = nom_couleur(d.color);
        let fiche = if couleur.is_empty() {
            d.name.clone()
        } else {
            format!("{} ({couleur})", d.name)
        };
        json!({ "fiche": fiche, "type": typ, "texte": texte })
    }

    fn json_score(&self) -> Value {
        let gagnant = self
            .score
            .winner()
            .map(|p| if p == HUMAIN { "vous" } else { "bot" });
        json!({
            "manches": self.reglages.manches,
            "vous": self.score.wins(HUMAIN),
            "bot": self.score.wins(BOT),
            "jouees": self.score.games_played(),
            "gagnant": gagnant,
        })
    }

    fn json_preparation(&self) -> Value {
        let db = &self.db;
        let pool = &self.pools[HUMAIN.index()];
        let (prop_arene, prop_deck) = &self.proposition;
        // Proposition d'arène : les positions des cartes proposées.
        let mut pris = vec![false; pool.arena.len()];
        for c in prop_arene {
            if let Some(i) = (0..pool.arena.len()).find(|&i| !pris[i] && pool.arena[i] == *c) {
                pris[i] = true;
            }
        }
        let arene: Vec<Value> = pool
            .arena
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let mut v = carte_pool(db, c);
                v["i"] = json!(i);
                v["zone"] = json!(nom_zone(equipment_zone(db.get(c))));
                v["propose"] = json!(pris[i]);
                v
            })
            .collect();
        let deck: Vec<Value> = self
            .cartes_distinctes()
            .into_iter()
            .map(|(c, max)| {
                let mut v = carte_pool(db, c);
                v["max"] = json!(max);
                v["propose"] = json!(prop_deck.iter().filter(|&&k| k == c).count());
                v
            })
            .collect();
        let arene_bot = match self.etape {
            Etape::Deck => self.arenes[BOT.index()]
                .as_ref()
                .map(|a| a.iter().map(|&c| carte_pool(db, c)).collect::<Vec<_>>()),
            _ => None,
        };
        let qui = |p: PlayerId| if p == HUMAIN { "vous" } else { "bot" };
        json!({
            "etape": match self.etape {
                Etape::PremierJoueur => "premier",
                Etape::Arene => "arene",
                Etape::Deck => "deck",
                Etape::Partie => "partie",
            },
            "choisit_premier": qui(self.chooser),
            "premier": self.premier.map(qui),
            "heros": db.get(pool.hero).name,
            "heros_id": pool.hero.0,
            "heros_bot": db.get(self.pools[BOT.index()].hero).name,
            "heros_bot_id": self.pools[BOT.index()].hero.0,
            "taille_deck": taille_deck(),
            "arene": arene,
            "deck": deck,
            "arene_bot": arene_bot,
        })
    }

    /// Tout ce que la page affiche, vu par l'humain.
    pub fn etat(&self) -> Value {
        let mut v = match (&self.session, self.etape) {
            (Some(s), Etape::Partie) => s.etat(),
            _ => json!({
                "reglages": {
                    "deck": self.reglages.deck,
                    "deck_bot": self.reglages.deck_bot,
                    "niveau": self.reglages.niveau,
                    "graine": self.reglages.graine.to_string(),
                },
            }),
        };
        v["reglages"]["manches"] = json!(self.reglages.manches);
        v["phase"] = json!(if self.etape == Etape::Partie {
            "partie"
        } else {
            "preparation"
        });
        v["match"] = self.json_score();
        if self.etape == Etape::Partie {
            let finie = self.session.as_ref().is_some_and(|s| s.g.is_over());
            v["manche_suivante"] = json!(finie && self.score.winner().is_none());
        } else {
            v["preparation"] = self.json_preparation();
        }
        v
    }

    /// Les choix de l'humain depuis le début du match, plus les coups de
    /// la manche en cours (pour la sauvegarde).
    pub fn etapes(&self) -> Vec<Value> {
        let mut v = self.etapes.clone();
        if self.etape == Etape::Partie {
            if let Some(s) = &self.session {
                v.push(json!({ "partie": s.coups() }));
            }
        }
        v
    }

    /// Le match est-il fini (un gagnant) ?
    pub fn termine(&self) -> bool {
        self.score.winner().is_some()
    }

    /// La prochaine manche créée rejouera ces coups (chargement).
    pub(crate) fn preparer_rejeu(&mut self, coups: Vec<u32>) {
        self.rejeu = Some(coups);
    }
}
