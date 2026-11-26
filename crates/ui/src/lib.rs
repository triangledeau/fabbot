//! fab-ui — jouer contre le bot.
//!
//! - étape 2 : interface texte dans le terminal ([`jeu`], [`vue`]) ;
//! - étape 5 : page web locale ([`serveur`], [`web`] ; plateau, boutons de
//!   choix, « Annuler », journal), lancée par `cargo run --release -p fab-ui --bin fab-web` ;
//! - étape 6 : matchs en plusieurs manches et choix de l'équipement et du
//!   deck avant chaque manche dans la page web ([`match_web`]) ;
//! - Arsenal, lot 0 : page en plusieurs fichiers intégrés au programme
//!   ([`fichiers`]), accueil et navigation par `#` ;
//! - Arsenal, lot 4a : collection Silver Age et decks maison ([`api_decks`]) ;
//! - Arsenal, lot 8 : tournoi et apprentissage depuis la page ([`api_tournoi`]) ;
//! - Arsenal, lot 9 : conseil du bot, évaluation, Bot contre Bot, signalement de bug ([`extras`]).

pub mod api_apprendre;
pub mod api_decks;
pub mod api_tournoi;
pub mod extras;
pub mod fichiers;
pub mod images;
pub mod jeu;
pub mod match_web;
pub mod sauvegarde;
pub mod serveur;
pub mod textes;
pub mod vue;
pub mod web;

use fab_ai::side::SideAppris;
use fab_cards::real;
use fab_engine::{CardDb, CardPool, FormatConfig, GameSetup, GameState, Loadout, PlayerId};
use fab_match::{InfoSide, Side};
use std::sync::Arc;

/// Les decks jouables (noms courts), dans l'ordre des fichiers de
/// `data/decks/` (voir `fab_cards::decks`).
pub fn decks_jouables() -> Vec<String> {
    fab_cards::decks::noms()
}

/// Charge le card-pool d'un deck jouable par son nom court.
pub fn charger_pool(db: &CardDb, nom: &str) -> Result<CardPool, String> {
    fab_cards::decks::charger_pool(db, nom)
}

/// Charge un deck jouable avec le choix « dans l'ordre de la liste »
/// (`real::default_loadout`).
pub fn charger_deck(db: &CardDb, nom: &str) -> Result<Loadout, String> {
    Ok(real::default_loadout_pool(db, &charger_pool(db, nom)?))
}

/// Le choix du bot (table apprise, `fab_ai::side`) contre ce héros adverse.
pub fn choix_du_bot(db: &CardDb, pool: &CardPool, heros_adverse: fab_engine::CardId) -> Loadout {
    let taille = FormatConfig::silver_age().deck_size as usize;
    let info = InfoSide {
        db,
        moi: PlayerId::P1,
        pool,
        heros_adverse,
        premier: PlayerId::P1,
        deck_size: taille,
    };
    let mut side = SideAppris::par_defaut(db);
    let arene = side.arene(&info);
    let deck = side.deck(&info, &arene, &[]);
    pool.loadout(db, arene, deck, taille)
        .expect("choix appris légal")
}

/// Prépare une partie : l'humain est le joueur 0, le bot le joueur 1.
/// Le joueur qui choisit qui commence (CR 4.1.3) est tiré au sort.
/// L'humain joue le choix « liste », le bot son choix appris (la table
/// apprise ne dépend pas du premier joueur, la décision « qui commence »
/// peut donc venir après). L'écran de choix complet est dans `fab-web`.
pub fn nouvelle_partie(
    deck_humain: &str,
    deck_bot: &str,
    graine: u64,
) -> Result<GameState, String> {
    let db = Arc::new(real::build());
    let humain = charger_pool(&db, deck_humain)?;
    let bot = charger_pool(&db, deck_bot)?;
    let lo = [
        real::default_loadout_pool(&db, &humain),
        choix_du_bot(&db, &bot, humain.hero),
    ];
    let setup = GameSetup {
        players: lo,
        chooser: PlayerId((graine % 2) as u8),
        seed: graine,
        shuffle: true,
        max_turns: 200,
    };
    let mut g = GameState::new(db, setup).map_err(|e| format!("mise en place : {e:?}"))?;
    g.record = Some(Vec::new());
    Ok(g)
}
