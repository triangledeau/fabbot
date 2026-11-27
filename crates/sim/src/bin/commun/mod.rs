//! Code commun aux programmes des épreuves (`epreuve-a`, `epreuve-b`) :
//! lire les options, jouer des parties B3 contre B3 en gardant des
//! positions.
#![allow(dead_code)]

use fab_ai::side::{creer_side, TableSide};
use fab_ai::{creer_agent, Agent};
use fab_cards::{decks, real};
use fab_engine::{CardDb, CardPool, Decision, DecisionKind, FormatConfig, GameState, PlayerId};
use fab_match::{lancer_manche, preparer_manche};
use std::sync::Arc;

/// Les options de la ligne de commande.
pub struct Options {
    args: Vec<String>,
}

impl Options {
    pub fn lire() -> Self {
        Options {
            args: std::env::args().collect(),
        }
    }

    pub fn nombre(&self, nom: &str, defaut: u64) -> u64 {
        self.texte(nom)
            .map(|v| {
                v.parse().unwrap_or_else(|_| {
                    eprintln!("{nom} : nombre attendu, pas « {v} »");
                    std::process::exit(2)
                })
            })
            .unwrap_or(defaut)
    }

    pub fn texte(&self, nom: &str) -> Option<&str> {
        self.args
            .iter()
            .position(|a| a == nom)
            .and_then(|i| self.args.get(i + 1))
            .map(|s| s.as_str())
    }

    pub fn drapeau(&self, nom: &str) -> bool {
        self.args.iter().any(|a| a == nom)
    }

    pub fn threads(&self) -> u64 {
        self.nombre(
            "--threads",
            std::thread::available_parallelism()
                .map(|n| n.get() as u64)
                .unwrap_or(1),
        )
        .max(1)
    }
}

/// Un deck : nom court, héros, card-pool.
pub struct Deck {
    pub nom: String,
    pub heros: String,
    pub pool: CardPool,
}

/// Les decks demandés (`--decks ira,hala` ; par défaut ceux de `data/decks/`).
pub fn lire_decks(db: &CardDb, liste: Option<&str>) -> Vec<Deck> {
    let noms: Vec<String> = match liste {
        Some(l) => l.split(',').map(|s| s.trim().to_string()).collect(),
        None => decks::noms(),
    };
    noms.into_iter()
        .map(|nom| {
            let list = decks::liste(&nom).unwrap_or_else(|e| panic!("{e}"));
            let pool = real::card_pool(db, &list).unwrap_or_else(|e| panic!("deck {nom} : {e}"));
            Deck {
                heros: list.hero.clone(),
                nom,
                pool,
            }
        })
        .collect()
}

pub fn table_side(db: &CardDb) -> Arc<TableSide> {
    Arc::new(TableSide::lire(db, fab_ai::side::TABLE_B1).expect("data/side/b1.txt valide"))
}

/// Un agent qui garde une copie de la partie à certaines décisions de son
/// joueur.
struct Capteur<F: FnMut(&GameState, &Decision) -> bool> {
    inner: Box<dyn Agent + Send>,
    garder: F,
    positions: Vec<GameState>,
}

impl<F: FnMut(&GameState, &Decision) -> bool> Agent for Capteur<F> {
    fn name(&self) -> &str {
        "capteur"
    }
    fn choose(&mut self, g: &GameState, d: &Decision) -> usize {
        if (self.garder)(g, d) {
            self.positions.push(g.clone());
        }
        self.inner.choose(g, d)
    }
}

/// Une décision « début de la phase d'action » : priorité du joueur actif,
/// pile vide, pas de chaîne de combat ouverte, première fois ce tour.
pub fn debut_de_tour() -> impl FnMut(&GameState, &Decision) -> bool {
    let mut dernier_tour = u32::MAX;
    move |g, d| {
        let oui = d.kind == DecisionKind::Priority
            && d.player == g.turn_player
            && g.action_phase
            && g.stack.is_empty()
            && !g.combat.open
            && g.turn != dernier_tour;
        if oui {
            dernier_tour = g.turn;
        }
        oui
    }
}

/// Joue une partie `bot` contre `bot` (phase de side apprise) entre deux
/// decks, et renvoie les positions gardées par `garder` pour chaque joueur.
pub fn partie_avec_positions<F>(
    db: &Arc<CardDb>,
    table: &Arc<TableSide>,
    pools: [&CardPool; 2],
    bot: &str,
    graine: u64,
    garder: [F; 2],
) -> Result<Vec<GameState>, String>
where
    F: FnMut(&GameState, &Decision) -> bool,
{
    let chooser = PlayerId((graine % 2) as u8);
    let mut s0 = creer_side("appris", db, Some(table.clone()), graine ^ 0x51DE).expect("side");
    let mut s1 = creer_side("appris", db, Some(table.clone()), graine ^ 0xE51D).expect("side");
    let taille = FormatConfig::silver_age().deck_size as usize;
    let m = preparer_manche(db, pools, [&mut *s0, &mut *s1], chooser, taille)
        .map_err(|(p, e)| format!("choix du joueur {} : {e:?}", p.0 + 1))?;
    let mut g = lancer_manche(db.clone(), &m, graine, 200).map_err(|e| format!("{e:?}"))?;
    let [g0, g1] = garder;
    let mut a = Capteur {
        inner: creer_agent(bot, graine ^ 0xA5A5).expect("bot"),
        garder: g0,
        positions: Vec::new(),
    };
    let mut b = Capteur {
        inner: creer_agent(bot, graine ^ 0x5A5A).expect("bot"),
        garder: g1,
        positions: Vec::new(),
    };
    fab_ai::play_out(&mut g, [&mut a, &mut b], false)?;
    let mut v = a.positions;
    v.append(&mut b.positions);
    v.sort_by_key(|g| g.turn);
    Ok(v)
}

/// Nom du héros d'un joueur.
pub fn heros(g: &GameState, p: PlayerId) -> String {
    g.def(g.player(p).hero).name.clone()
}
