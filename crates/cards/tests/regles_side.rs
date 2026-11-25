//! Choix de l'équipement et du deck avant chaque partie (« phase de side ») :
//! CR 4.1.3 à 4.1.7 et TRP 7.4 (Silver Age), avec les vrais decks.

use fab_ai::side::{Options, SideAppris, TableSide, TABLE_B1};
use fab_cards::decklist::Decklist;
use fab_cards::real;
use fab_engine::{CardDb, CardId, CardPool, ChoixInvalide, FormatConfig, PlayerId};
use fab_match::{lancer_manche, preparer_manche, InfoSide, Side, SideListe};
use std::path::PathBuf;
use std::sync::Arc;

fn pool(db: &CardDb, nom: &str) -> CardPool {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/decks");
    let f = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .split_once('_')
                .is_some_and(|(_, r)| r.starts_with(nom))
        })
        .unwrap();
    let l = Decklist::parse(&std::fs::read_to_string(f).unwrap()).unwrap();
    real::card_pool(db, &l).unwrap()
}

const DECKS: [&str; 9] = [
    "ira",
    "hala",
    "dorinthea",
    "bravo",
    "iyslander",
    "florian",
    "levia",
    "dromai",
    "arakni",
];

fn taille() -> usize {
    FormatConfig::silver_age().deck_size as usize
}

fn id(db: &CardDb, nom: &str) -> CardId {
    db.find_name(nom).unwrap()
}

// TRP 7.4 : « a combined maximum of 55 arena-cards and deck-cards ».
#[test]
fn trp_7_4_le_card_pool_compte_55_cartes_d_arene_et_de_deck() {
    let db = real::build();
    for d in DECKS {
        let p = pool(&db, d);
        assert_eq!(p.arena.len() + p.deck.len(), 55, "{d}");
    }
}

// TRP 7.4 : « A Player must start the game with exactly 40 cards in their deck. »
#[test]
fn trp_7_4_le_deck_compte_exactement_40_cartes() {
    let db = real::build();
    let p = pool(&db, "ira");
    let (_, deck) = p.choix_par_ordre(&db, 40);
    assert_eq!(p.verifier_deck(&db, &deck, 40), Ok(()));
    assert_eq!(
        p.verifier_deck(&db, &deck[..39], 40),
        Err(ChoixInvalide::TailleDuDeck {
            attendu: 40,
            recu: 39
        })
    );
    let mut trop = deck.clone();
    trop.push(p.deck[40]);
    assert!(matches!(
        p.verifier_deck(&db, &trop, 40),
        Err(ChoixInvalide::TailleDuDeck { recu: 41, .. })
    ));
}

#[test]
fn une_carte_absente_du_card_pool_est_refusee() {
    let db = real::build();
    let ira = pool(&db, "ira");
    let hala = pool(&db, "hala");
    let mut deck = ira.choix_par_ordre(&db, 40).1;
    let etrangere = *hala.deck.iter().find(|c| !ira.deck.contains(c)).unwrap();
    deck[0] = etrangere;
    assert_eq!(
        ira.verifier_deck(&db, &deck, 40),
        Err(ChoixInvalide::HorsDuPool(etrangere))
    );
}

#[test]
fn un_troisieme_exemplaire_est_refuse() {
    let db = real::build();
    let p = pool(&db, "ira");
    let c = p.deck[0];
    let mut deck: Vec<CardId> = p
        .deck
        .iter()
        .copied()
        .filter(|&k| k != c)
        .take(37)
        .collect();
    deck.extend([c, c, c]);
    assert_eq!(
        p.verifier_deck(&db, &deck, 40),
        Err(ChoixInvalide::HorsDuPool(c))
    );
}

// CR 4.1.6a : « Arena-cards cannot be included in a player's deck. »
#[test]
fn cr_4_1_6a_une_carte_d_arene_ne_va_pas_dans_le_deck() {
    let db = real::build();
    let p = pool(&db, "ira");
    let mut deck = p.choix_par_ordre(&db, 40).1;
    deck[0] = p.arena[0];
    assert_eq!(
        p.verifier_deck(&db, &deck, 40),
        Err(ChoixInvalide::PasUneCarteDeDeck(p.arena[0]))
    );
}

// CR 4.1.4a : « up to one arena-card for each of their arms, chest, head,
// legs, and weapon zones ».
#[test]
fn cr_4_1_4a_une_seule_carte_par_zone() {
    let db = real::build();
    let p = pool(&db, "ira");
    let mask = id(&db, "Mask of the Swarming Claw");
    let helm = id(&db, "Blade Beckoner Helm");
    assert_eq!(p.verifier_arene(&db, &[mask]), Ok(()));
    assert_eq!(
        p.verifier_arene(&db, &[mask, helm]),
        Err(ChoixInvalide::ZoneOccupee(helm))
    );
    // « up to » : aucune carte d'arène est légal.
    assert_eq!(p.verifier_arene(&db, &[]), Ok(()));
}

// CR 4.1.4 : seules les cartes d'arène vont dans les zones d'arène.
#[test]
fn cr_4_1_4_une_carte_de_deck_ne_s_equipe_pas() {
    let db = real::build();
    let p = pool(&db, "ira");
    assert_eq!(
        p.verifier_arene(&db, &[p.deck[0]]),
        Err(ChoixInvalide::HorsDuPool(p.deck[0]))
    );
    assert_eq!(
        fab_engine::pool::arena_compatible(&db, &[p.deck[0]]),
        Err(ChoixInvalide::PasUneCarteDArene(p.deck[0]))
    );
}

// CR 8.2.2b : deux mains au plus ; Florian ne peut pas équiper Nebula Blade
// (2H) avec Annals of Sutcliffe (2H) ni avec un off-hand.
#[test]
fn cr_8_2_2b_une_arme_a_deux_mains_occupe_les_deux_mains() {
    let db = real::build();
    let p = pool(&db, "florian");
    let nebula = id(&db, "Nebula Blade");
    let annals = id(&db, "Annals of Sutcliffe");
    let lantern = id(&db, "Arcane Lantern");
    let scepter = id(&db, "Scepter of Pain");
    assert_eq!(
        p.verifier_arene(&db, &[nebula, annals]),
        Err(ChoixInvalide::ZoneOccupee(annals))
    );
    assert_eq!(
        p.verifier_arene(&db, &[nebula, lantern]),
        Err(ChoixInvalide::ZoneOccupee(lantern))
    );
    assert_eq!(p.verifier_arene(&db, &[scepter, lantern]), Ok(()));
}

// CR 8.2.10b : un seul off-hand.
#[test]
fn cr_8_2_10b_un_seul_off_hand() {
    let db = real::build();
    let p = pool(&db, "florian");
    let lantern = id(&db, "Arcane Lantern");
    let oval = id(&db, "Bloodied Oval");
    assert_eq!(
        p.verifier_arene(&db, &[lantern, oval]),
        Err(ChoixInvalide::ZoneOccupee(oval))
    );
}

#[test]
fn deux_armes_a_une_main_identiques_si_le_card_pool_en_a_deux() {
    let db = real::build();
    let p = pool(&db, "arakni");
    let mark = id(&db, "Mark of the Huntsman");
    assert_eq!(p.verifier_arene(&db, &[mark, mark]), Ok(()));
    let ira = pool(&db, "ira");
    let edge = id(&db, "Edge of Autumn");
    assert_eq!(
        ira.verifier_arene(&db, &[edge, edge]),
        Err(ChoixInvalide::HorsDuPool(edge))
    );
}

/// Un side qui note ce qu'il a su au moment de choisir.
struct Espion {
    premier_vu: Option<PlayerId>,
    arene_adverse_vue: Vec<CardId>,
    heros_adverse_vu: Option<CardId>,
    choix_premier: Option<PlayerId>,
}

impl Espion {
    fn new(choix_premier: Option<PlayerId>) -> Self {
        Espion {
            premier_vu: None,
            arene_adverse_vue: Vec::new(),
            heros_adverse_vu: None,
            choix_premier,
        }
    }
}

impl Side for Espion {
    fn nom(&self) -> &str {
        "espion"
    }
    fn premier_joueur(&mut self, info: &InfoSide) -> PlayerId {
        self.choix_premier.unwrap_or(info.moi)
    }
    fn arene(&mut self, info: &InfoSide) -> Vec<CardId> {
        self.premier_vu = Some(info.premier);
        self.heros_adverse_vu = Some(info.heros_adverse);
        SideListe.arene(info)
    }
    fn deck(&mut self, info: &InfoSide, a: &[CardId], adv: &[CardId]) -> Vec<CardId> {
        self.arene_adverse_vue = adv.to_vec();
        SideListe.deck(info, a, adv)
    }
}

// CR 4.1.3 puis 4.1.4 : le premier joueur est choisi avant les cartes
// d'arène, par le joueur désigné, et la partie commence avec lui.
#[test]
fn cr_4_1_3_le_premier_joueur_est_connu_avant_le_choix_des_cartes() {
    let db = Arc::new(real::build());
    let (ira, hala) = (pool(&db, "ira"), pool(&db, "hala"));
    let mut a = Espion::new(None);
    let mut b = Espion::new(Some(PlayerId::P0));
    // Le joueur 2 choisit et désigne le joueur 1.
    let m = preparer_manche(&db, [&ira, &hala], [&mut a, &mut b], PlayerId::P1, taille()).unwrap();
    assert_eq!(m.premier, PlayerId::P0);
    assert_eq!(a.premier_vu, Some(PlayerId::P0));
    assert_eq!(b.premier_vu, Some(PlayerId::P0));
    // CR 4.1.2 : les héros sont visibles.
    assert_eq!(a.heros_adverse_vu, Some(hala.hero));
    let g = lancer_manche(db.clone(), &m, 7, 200).unwrap();
    assert_eq!(g.first_player, PlayerId::P0);
    assert_eq!(g.turn_player, PlayerId::P0);
}

// CR 4.1.5 puis 4.1.6 : le deck est choisi après la présentation des
// cartes d'arène des deux joueurs.
#[test]
fn cr_4_1_6_le_deck_est_choisi_en_connaissant_l_arene_adverse() {
    let db = real::build();
    let (ira, hala) = (pool(&db, "ira"), pool(&db, "hala"));
    let mut a = Espion::new(None);
    let mut b = Espion::new(None);
    let m = preparer_manche(&db, [&ira, &hala], [&mut a, &mut b], PlayerId::P0, taille()).unwrap();
    assert_eq!(a.arene_adverse_vue, m.loadouts[1].arena);
    assert_eq!(b.arene_adverse_vue, m.loadouts[0].arena);
}

// CR 4.1.7 : les cartes non choisies ne sont pas dans la partie, mais le
// card-pool de deck est gardé pour les bots (main adverse imaginée).
#[test]
fn cr_4_1_7_les_cartes_non_choisies_restent_hors_de_la_partie() {
    let db = Arc::new(real::build());
    let (ira, hala) = (pool(&db, "ira"), pool(&db, "hala"));
    let m = preparer_manche(
        &db,
        [&ira, &hala],
        [&mut SideListe, &mut SideListe],
        PlayerId::P0,
        taille(),
    )
    .unwrap();
    let g = lancer_manche(db.clone(), &m, 3, 200).unwrap();
    let p0 = g.player(PlayerId::P0);
    assert_eq!(p0.deck.len() + p0.hand.len(), 40);
    assert_eq!(g.card_pools[0].len(), ira.deck.len());
}

struct Tricheur;
impl Side for Tricheur {
    fn nom(&self) -> &str {
        "tricheur"
    }
    fn arene(&mut self, info: &InfoSide) -> Vec<CardId> {
        SideListe.arene(info)
    }
    fn deck(&mut self, info: &InfoSide, _: &[CardId], _: &[CardId]) -> Vec<CardId> {
        info.pool.deck.clone() // plus de 40 cartes
    }
}

#[test]
fn un_choix_illegal_est_refuse_avec_le_joueur_fautif() {
    let db = real::build();
    let (ira, hala) = (pool(&db, "ira"), pool(&db, "hala"));
    let e = preparer_manche(
        &db,
        [&ira, &hala],
        [&mut SideListe, &mut Tricheur],
        PlayerId::P0,
        taille(),
    )
    .unwrap_err();
    assert_eq!(e.0, PlayerId::P1);
    assert!(matches!(e.1, ChoixInvalide::TailleDuDeck { .. }));
}

/// La table apprise (`data/side/b1.txt`) donne un choix légal pour chacune
/// des 81 paires, et le bot « appris » la suit.
#[test]
fn la_table_apprise_est_legale_pour_les_81_paires() {
    let db = real::build();
    let table = Arc::new(TableSide::lire(&db, TABLE_B1).unwrap());
    let pools: Vec<CardPool> = DECKS.iter().map(|d| pool(&db, d)).collect();
    for a in &pools {
        for b in &pools {
            let e = table
                .trouver(a.hero, b.hero)
                .expect("entrée pour chaque paire");
            assert_eq!(e.adverse, Some(b.hero));
            assert_eq!(a.verifier_arene(&db, &e.arene), Ok(()));
            assert_eq!(e.hors_deck.len(), a.deck.len() - taille());
            let mut side = SideAppris::new(table.clone());
            let info = InfoSide {
                db: &db,
                moi: PlayerId::P0,
                pool: a,
                heros_adverse: b.hero,
                premier: PlayerId::P0,
                deck_size: taille(),
            };
            assert_eq!(side.arene(&info), e.arene);
            let deck = side.deck(&info, &e.arene, &[]);
            assert_eq!(a.verifier_deck(&db, &deck, taille()), Ok(()));
            for c in &e.hors_deck {
                let dans_pool = a.deck.iter().filter(|k| *k == c).count();
                let hors = e.hors_deck.iter().filter(|k| *k == c).count();
                let gardes = deck.iter().filter(|k| *k == c).count();
                assert_eq!(gardes, dans_pool - hors);
            }
        }
    }
}

// Les options d'arène d'un card-pool : une par carte dans chaque zone, et
// pour les armes, chaque combinaison qui remplit les mains au mieux.
#[test]
fn options_d_armes_de_florian_et_d_arakni() {
    let db = real::build();
    let f = Options::new(&db, &pool(&db, "florian"));
    let armes = &f.zones[0].1;
    let nebula = id(&db, "Nebula Blade");
    let annals = id(&db, "Annals of Sutcliffe");
    let scepter = id(&db, "Scepter of Pain");
    assert!(armes.contains(&vec![nebula]));
    assert!(armes.contains(&vec![annals]));
    assert!(armes.iter().any(|o| o.contains(&scepter) && o.len() == 2));
    assert!(armes.iter().all(|o| !o.contains(&nebula) || o.len() == 1));
    let a = Options::new(&db, &pool(&db, "arakni"));
    let mark = id(&db, "Mark of the Huntsman");
    assert_eq!(a.zones[0].1, vec![vec![mark, mark]]);
}
