//! Arsenal, lot 4a : decks en données.
//!
//! - **Collection Silver Age** : toutes les cartes légales du format
//!   (`data/cards/collection.tsv`, produite depuis le fichier officiel LSS
//!   par `tools/collection_silver_age.py`, intégrée au programme), avec pour
//!   chacune le drapeau `codee` (« jouable par le bot » : la carte est codée
//!   dans `fab-cards`). Le texte des cartes vient du fichier officiel local
//!   s'il est là ([`Textes`]).
//! - **Decks maison** (`data/decks/maison/<nom>.txt`, [`fab_cards::decks`]) :
//!   importés depuis une liste FaBrary ou au format du projet, vérifiés
//!   (format Silver Age, légalité, cartes codées), lus à l'exécution. Un deck
//!   maison dont tout est bon est jouable dans la page.
//!
//! | Route | Rôle |
//! |---|---|
//! | `GET /api/collection` | la collection Silver Age (cartes, drapeau `codee`) |
//! | `GET /api/decks-maison` | les decks maison et leur vérification |
//! | `GET /api/decks-projet` | les decks du bot et leur vérification (écran Jouer) |
//! | `GET /api/deck-maison?nom=x` | un deck maison : texte, cartes, vérification |
//! | `POST /api/deck-maison/verifier` (corps : la liste) | vérifie sans enregistrer |
//! | `POST /api/deck-maison/importer?nom=x` (corps : la liste) | vérifie et enregistre (remplace) |
//! | `GET /api/deck?nom=x` | un deck du projet ou maison (pour le dupliquer ou l'éditer) |
//! | `POST /api/deck-maison/archiver?nom=x` | range le deck dans `archives/` |
//! | `POST /api/deck-maison/restaurer?nom=x` | le ressort des archives |
//! | `POST /api/deck-maison/supprimer?nom=x` | supprime un deck **archivé** |
//!
//! Aucune règle ici : la vérification du format est celle de
//! [`Decklist::check`], les cartes jouables sont celles du moteur.

use crate::images;
use crate::textes::Textes;
use fab_cards::decklist::{self, Decklist};
use fab_cards::decks::{self, DeckDuProjet};
use fab_cards::import::{self, Classement};
use fab_cards::real;
use fab_engine::card::CardDb;
use fab_engine::state::equipment_zone;
use fab_engine::{Color, FormatConfig, Subtypes, Types, Zone};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Une carte de la collection Silver Age.
#[derive(Clone, Debug)]
pub struct CarteCollection {
    pub nom: String,
    pub couleur: Option<Color>,
    /// Types et sous-types (« Warrior », « Equipment », « Head »…).
    pub types: Vec<String>,
    pub type_texte: String,
    pub pitch: Option<u8>,
    pub cout: Option<u8>,
    pub puissance: Option<u8>,
    pub defense: Option<u8>,
    pub image: Option<String>,
    pub nom_fr: Option<String>,
    /// Carte codée dans `fab-cards` : le bot sait la jouer.
    pub codee: bool,
    /// Classes et talents lus dans les types (pour le filtre « pour ce
    /// héros » de l'éditeur ; ce n'est pas une vérification de règle).
    pub classes: Vec<String>,
    pub talents: Vec<String>,
    pub mots_cles: Vec<String>,
    /// « [HERO] Specialization » (CR 8.3.7) : surnoms des héros permis.
    pub specialisation: Vec<String>,
    /// « Essence of X » d'un héros : talents X permis.
    pub essences: Vec<String>,
    /// Texte de la carte (vide : il vient du fichier officiel local).
    pub texte: String,
    pub heros: bool,
    /// PV et intellect d'un héros (en-tête de l'éditeur).
    pub pv: Option<u8>,
    pub intellect: Option<u8>,
}

/// Classes de Flesh and Blood (données des cartes, pour les filtres).
const CLASSES: [&str; 18] = [
    "Adjudicator",
    "Assassin",
    "Bard",
    "Brute",
    "Generic",
    "Guardian",
    "Illusionist",
    "Mechanologist",
    "Merchant",
    "Necromancer",
    "Ninja",
    "Pirate",
    "Ranger",
    "Runeblade",
    "Shapeshifter",
    "Thief",
    "Warrior",
    "Wizard",
];
/// Talents (données des cartes, pour les filtres).
const TALENTS: [&str; 12] = [
    "Chaos",
    "Draconic",
    "Earth",
    "Elemental",
    "Ice",
    "Light",
    "Lightning",
    "Mystic",
    "Revered",
    "Reviled",
    "Royal",
    "Shadow",
];

/// La collection et d'où elle vient.
pub struct Collection {
    /// `lss` : la table versionnée `data/cards/collection.tsv`.
    pub source: &'static str,
    pub cartes: Vec<CarteCollection>,
    /// Nom → positions dans `cartes`.
    par_nom: HashMap<String, Vec<usize>>,
}

fn db() -> &'static CardDb {
    static DB: OnceLock<CardDb> = OnceLock::new();
    DB.get_or_init(real::build)
}

fn couleur_table(s: &str) -> Option<Color> {
    match s {
        "Red" => Some(Color::Red),
        "Yellow" => Some(Color::Yellow),
        "Blue" => Some(Color::Blue),
        _ => None,
    }
}

fn nom_couleur(c: Option<Color>) -> &'static str {
    match c {
        Some(Color::Red) => "rouge",
        Some(Color::Yellow) => "jaune",
        Some(Color::Blue) => "bleu",
        None => "",
    }
}

/// La table des cartes légales en Silver Age, produite par
/// `tools/collection_silver_age.py` depuis le fichier officiel LSS
/// (légalité : TRP 7.4, `config/silver_age.toml`), intégrée au programme.
/// Sans le texte des cartes : il est lu dans le fichier officiel local
/// ([`Textes`]) quand il est là.
const TABLE: &str = include_str!("../../../data/cards/collection.tsv");

fn liste_pv(s: &str) -> Vec<String> {
    s.split(';')
        .filter(|m| !m.is_empty())
        .map(String::from)
        .collect()
}

/// Lit la table : une carte par ligne.
fn depuis_table(texte: &str, db: &CardDb) -> Vec<CarteCollection> {
    let mut out = Vec::new();
    for l in texte.lines() {
        if l.starts_with('#') || l.trim().is_empty() {
            continue;
        }
        let c: Vec<&str> = l.split('\t').collect();
        if c.len() < 14 {
            continue;
        }
        let nombre = |s: &str| s.trim().parse::<u8>().ok();
        let nom = c[0].to_string();
        let couleur = couleur_table(c[1]);
        let type_texte = c[8].to_string();
        // Mots de la ligne de type (« Brute / Warrior Action - Attack »).
        let types: Vec<String> = type_texte
            .split(|ch: char| ch.is_whitespace() || ch == '/')
            .filter(|m| !m.is_empty() && *m != "-")
            .map(String::from)
            .fold(Vec::new(), |mut v, m| {
                if !v.contains(&m) {
                    v.push(m);
                }
                v
            });
        let a = |t: &str| types.iter().any(|x| x == t);
        // « Attack Reaction », « Defense Reaction », « Off-Hand » : deux mots.
        let mut types_complets = types.clone();
        for (t, mots) in [
            ("Attack Reaction", "Attack Reaction"),
            ("Defense Reaction", "Defense Reaction"),
        ] {
            if type_texte.contains(mots) {
                types_complets.push(t.to_string());
            }
        }
        out.push(CarteCollection {
            codee: db.find(&nom, couleur).is_some(),
            couleur,
            pitch: nombre(c[2]),
            cout: nombre(c[3]),
            puissance: nombre(c[4]),
            defense: nombre(c[5]),
            pv: nombre(c[6]),
            intellect: nombre(c[7]),
            classes: types
                .iter()
                .filter(|t| CLASSES.contains(&t.as_str()))
                .cloned()
                .collect(),
            talents: types
                .iter()
                .filter(|t| TALENTS.contains(&t.as_str()))
                .cloned()
                .collect(),
            heros: a("Hero"),
            mots_cles: liste_pv(c[9]),
            specialisation: liste_pv(c[10]),
            essences: liste_pv(c[11]),
            image: (!c[12].is_empty()).then(|| c[12].to_string()),
            nom_fr: (!c[13].is_empty()).then(|| c[13].to_string()),
            texte: String::new(),
            types: types_complets,
            type_texte,
            nom,
        });
    }
    out
}

impl Collection {
    fn nouvelle(source: &'static str, mut cartes: Vec<CarteCollection>) -> Collection {
        cartes.sort_by(|a, b| a.nom.cmp(&b.nom).then(a.pitch.cmp(&b.pitch)));
        let mut par_nom: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, c) in cartes.iter().enumerate() {
            par_nom.entry(c.nom.clone()).or_default().push(i);
        }
        Collection {
            source,
            cartes,
            par_nom,
        }
    }

    /// Les cartes de ce nom (toutes couleurs).
    pub fn par_nom(&self, nom: &str) -> impl Iterator<Item = &CarteCollection> {
        self.par_nom
            .get(nom)
            .into_iter()
            .flatten()
            .map(|&i| &self.cartes[i])
    }

    /// Classes et talents d'un texte de type, par côté d'une carte hybride
    /// (« Brute / Warrior Action » → {Brute}, {Warrior}) ; un seul côté pour
    /// une carte ordinaire. « Generic » ne compte pas. Une carte double
    /// (« Runeblade Action // Earth Instant ») compte ses deux moitiés.
    fn cotes(type_texte: &str) -> Vec<HashSet<&str>> {
        let moities: Vec<&str> = type_texte
            .split(" // ")
            .map(|m| m.split(" - ").next().unwrap_or(""))
            .collect();
        fn mots(c: &str) -> Vec<&str> {
            c.split_whitespace()
                .filter(|m| *m != "Generic" && (CLASSES.contains(m) || TALENTS.contains(m)))
                .collect()
        }
        if moities.len() > 1 {
            // Meld : le héros doit pouvoir jouer les deux moitiés.
            return vec![moities.iter().flat_map(|m| mots(m)).collect()];
        }
        moities[0]
            .split(" / ")
            .map(|c| mots(c).into_iter().collect())
            .collect()
    }

    /// La carte `nom` est d'une classe ou d'un talent que le héros n'a pas.
    /// Le héros doit avoir toutes les classes et tous les talents de la
    /// carte ; pour une carte hybride, ceux d'un seul des deux côtés
    /// suffisent (règles confirmées par le propriétaire le 2026-11-26). « Essence of X »
    /// sur le héros ajoute le talent X.
    /// `None` : on ne sait pas (types inconnus du héros ou de la carte).
    pub fn hors_classe(&self, heros: &str, nom: &str) -> Option<bool> {
        let h = self
            .par_nom(heros)
            .find(|c| c.heros && !c.type_texte.is_empty())?;
        let c = self.par_nom(nom).find(|c| !c.type_texte.is_empty())?;
        let mut du_heros: HashSet<&str> =
            Self::cotes(&h.type_texte).into_iter().flatten().collect();
        // « Essence of Ice » (Iyslander), « Essence of Earth, Ice, and
        // Lightning » (Bravo) : le héros peut avoir des cartes de ces talents.
        du_heros.extend(h.essences.iter().map(String::as_str));
        Some(
            !Self::cotes(&c.type_texte)
                .iter()
                .any(|cote| cote.is_subset(&du_heros)),
        )
    }

    /// CR 8.3.7 : « [HERO] Specialization » = la carte ne va que dans le
    /// deck d'un héros dont le surnom (CR 2.7.3, le nom avant la virgule) est
    /// HERO. `None` : carte inconnue de la table ou sans spécialisation.
    pub fn specialisation_d_un_autre(&self, heros: &str, nom: &str) -> Option<String> {
        let c = self.par_nom(nom).find(|c| !c.specialisation.is_empty())?;
        let surnom = heros.split(',').next().unwrap_or("").trim();
        (!c.specialisation.iter().any(|s| s == surnom)).then(|| c.specialisation.join(", "))
    }

    /// La carte de ce nom et de cette couleur est légale en Silver Age :
    /// présente dans la table (toutes les cartes légales du fichier officiel).
    pub fn legale(&self, nom: &str, couleur: Option<Color>) -> Option<bool> {
        Some(self.par_nom(nom).any(|c| c.couleur == couleur))
    }

    /// La collection en JSON, avec le texte officiel quand il est là.
    pub fn json(&self, textes: Option<&Textes>) -> Value {
        let cartes: Vec<Value> = self
            .cartes
            .iter()
            .map(|c| {
                let texte = textes
                    .and_then(|t| t.texte(&c.nom.replace(" // ", "||"), c.couleur))
                    .map(|(_, b)| b)
                    .unwrap_or_else(|| c.texte.clone());
                json!({
                    "nom": c.nom,
                    "nom_fr": c.nom_fr,
                    "couleur": nom_couleur(c.couleur),
                    "types": c.types,
                    "type_texte": c.type_texte,
                    "pitch": c.pitch,
                    "cout": c.cout,
                    "puissance": c.puissance,
                    "defense": c.defense,
                    "image": c.image,
                    "codee": c.codee,
                    "classes": c.classes,
                    "talents": c.talents,
                    "mots_cles": c.mots_cles,
                    "specialisation": c.specialisation,
                    "essences": c.essences,
                    "texte": texte,
                    "heros": c.heros,
                    "pv": c.pv,
                    "intellect": c.intellect,
                    "emplacement": emplacement_collection(c),
                })
            })
            .collect();
        json!({
            "source": self.source,
            "nombre": cartes.len(),
            "codees": self.cartes.iter().filter(|c| c.codee).count(),
            "textes": textes.is_some(),
            "cartes": cartes,
        })
    }
}

/// La collection, lue une fois.
pub fn collection() -> &'static Collection {
    static C: OnceLock<Collection> = OnceLock::new();
    C.get_or_init(|| {
        let db = db();
        // Une carte codée absente de la table (carte plus récente que la
        // table) n'y est pas : refaire la table (tools/collection_silver_age.py).
        Collection::nouvelle("lss", depuis_table(TABLE, db))
    })
}

/// Emplacement d'arène écrit dans les listes (`[Head]`…) d'une carte codée.
fn emplacement_code(db: &CardDb, nom: &str) -> Option<String> {
    let d = db.get(db.find(nom, None)?);
    if !(d.types.has(Types::EQUIPMENT) || d.types.has(Types::WEAPON)) {
        return None;
    }
    Some(
        match equipment_zone(d) {
            Zone::Weapon if d.subtypes.has(Subtypes::OFF_HAND) => "Off-Hand",
            Zone::Weapon if d.subtypes.has(Subtypes::QUIVER) => "Quiver",
            Zone::Weapon => "Weapon",
            Zone::Head => "Head",
            Zone::Chest => "Chest",
            Zone::Arms => "Arms",
            Zone::Legs => "Legs",
            _ => return None,
        }
        .to_string(),
    )
}

/// Emplacement d'arène d'une carte de la collection (`null` : carte de deck
/// ou héros).
fn emplacement_collection(c: &CarteCollection) -> Option<String> {
    // Une carte colorée est une carte de deck (ex. les équipements « Evo »,
    // de type Action).
    if c.heros || c.couleur.is_some() {
        return None;
    }
    if c.types.is_empty() {
        return emplacement_code(db(), &c.nom);
    }
    let a = |t: &str| c.types.iter().any(|x| x == t);
    if a("Weapon") {
        return Some("Weapon".into());
    }
    if !a("Equipment") {
        return None;
    }
    ["Head", "Chest", "Arms", "Legs", "Off-Hand", "Quiver"]
        .into_iter()
        .find(|e| a(e))
        .map(String::from)
        .or(Some("Equipment".into()))
}

/// Classe une carte sans couleur d'une liste importée : la base de cartes
/// codées d'abord, puis la collection.
pub fn classer(nom: &str) -> Classement {
    let db = db();
    let col = collection();
    let classe_col = || -> String {
        col.par_nom(nom)
            .next()
            .map(|c| {
                c.types
                    .iter()
                    .filter(|t| !["Hero", "Young"].contains(&t.as_str()))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    };
    if let Some(id) = db.find(nom, None) {
        let d = db.get(id);
        if d.types.has(Types::HERO) {
            return Classement::Heros(classe_col());
        }
        if let Some(e) = emplacement_code(db, nom) {
            return Classement::Arene(e);
        }
        return Classement::Deck;
    }
    let Some(c) = col.par_nom(nom).next() else {
        return Classement::Inconnue;
    };
    let a = |t: &str| c.types.iter().any(|x| x == t);
    if a("Hero") {
        return Classement::Heros(classe_col());
    }
    if a("Weapon") {
        return Classement::Arene("Weapon".into());
    }
    if a("Equipment") {
        for e in ["Head", "Chest", "Arms", "Legs", "Off-Hand", "Quiver"] {
            if a(e) {
                return Classement::Arene(e.into());
            }
        }
        return Classement::Arene("Equipment".into());
    }
    Classement::Deck
}

fn couleur_liste(c: decklist::Color) -> Color {
    match c {
        decklist::Color::Red => Color::Red,
        decklist::Color::Yellow => Color::Yellow,
        decklist::Color::Blue => Color::Blue,
    }
}

/// Vérification d'une liste : format Silver Age, légalité, cartes codées,
/// et si le bot peut la jouer.
pub fn verifier(liste: &Decklist) -> Value {
    let db = db();
    let col = collection();
    let format = FormatConfig::silver_age();
    let mut problemes = liste.check(&format);
    let mut non_codees: Vec<String> = Vec::new();
    let mut illegales: Vec<String> = Vec::new();
    let mut voir = |nom: &str, c: Option<Color>| {
        let lib = match c {
            Some(_) => format!("{nom} ({})", nom_couleur(c)),
            None => nom.to_string(),
        };
        if db.find(nom, c).is_none() && !non_codees.contains(&lib) {
            non_codees.push(lib.clone());
        }
        if db.find(nom, c).is_none()
            && col.legale(nom, c) == Some(false)
            && !illegales.contains(&lib)
        {
            illegales.push(lib);
        }
    };
    voir(&liste.hero, None);
    for e in &liste.arena {
        voir(&e.name, None);
    }
    for e in &liste.deck {
        voir(&e.name, Some(couleur_liste(e.color)));
    }
    // Le héros doit être « Young » (Silver Age).
    if let Some(h) = db.find(&liste.hero, None).map(|id| db.get(id)) {
        if !h.types.has(Types::HERO) {
            problemes.push(format!("« {} » n'est pas un héros", liste.hero));
        } else if !h.subtypes.has(Subtypes::YOUNG) {
            problemes.push(format!("le héros « {} » n'est pas « Young »", liste.hero));
        }
    }
    for n in &illegales {
        problemes.push(format!("pas légale en Silver Age : {n}"));
    }
    let mut hors: Vec<&str> = Vec::new();
    for n in liste
        .arena
        .iter()
        .map(|e| e.name.as_str())
        .chain(liste.deck.iter().map(|e| e.name.as_str()))
    {
        if col.hors_classe(&liste.hero, n) == Some(true) && !hors.contains(&n) {
            hors.push(n);
        }
    }
    for n in hors {
        problemes.push(format!(
            "« {n} » n'est pas d'une classe ou d'un talent de {}",
            liste.hero
        ));
    }
    // CR 8.3.7 : spécialisation d'un autre héros.
    let mut vues: Vec<&str> = Vec::new();
    for n in liste.deck.iter().map(|e| e.name.as_str()) {
        if vues.contains(&n) {
            continue;
        }
        vues.push(n);
        if let Some(h) = col.specialisation_d_un_autre(&liste.hero, n) {
            problemes.push(format!("« {n} » est une spécialisation de {h}"));
        }
    }
    // Le card-pool se monte-t-il dans le moteur (emplacements, mains) ?
    if problemes.is_empty() && non_codees.is_empty() {
        match real::card_pool(db, liste) {
            Ok(pool) => {
                let taille = format.deck_size as usize;
                let (a, d) = pool.choix_par_ordre(db, taille);
                if let Err(e) = pool.loadout(db, a, d, taille) {
                    problemes.push(format!("équipement de départ impossible : {e:?}"));
                }
            }
            Err(e) => problemes.push(e),
        }
    }
    let jouable = problemes.is_empty() && non_codees.is_empty();
    let heros = db.find(&liste.hero, None).map(|id| db.get(id));
    json!({
        "titre": liste.title,
        "source": liste.source,
        "heros": liste.hero,
        "heros_image": heros.and_then(|h| images::champs(h).0),
        "classe": liste.class,
        "arene": liste.arena_count(),
        "deck": liste.deck_count(),
        "jouable": jouable,
        "problemes": problemes,
        "non_codees": non_codees,
        "legalite_verifiee": true,
    })
}

fn lire_deck(d: &DeckDuProjet) -> Result<(String, Decklist), String> {
    let texte = std::fs::read_to_string(&d.fichier)
        .map_err(|e| format!("lecture de {} : {e}", d.fichier.display()))?;
    let liste = Decklist::parse(&texte).map_err(|e| format!("{} : {e}", d.nom))?;
    Ok((texte, liste))
}

fn resume(d: &DeckDuProjet) -> Value {
    match lire_deck(d) {
        Ok((_, l)) => {
            let mut v = verifier(&l);
            v["nom"] = json!(d.nom);
            v
        }
        Err(e) => json!({ "nom": d.nom, "jouable": false, "problemes": [e], "non_codees": [] }),
    }
}

/// Les decks maison jouables (toutes leurs cartes codées, format respecté).
pub fn maison_jouables() -> Vec<String> {
    decks::maison()
        .iter()
        .filter(|d| lire_deck(d).map(|(_, l)| verifier(&l)["jouable"] == true) == Ok(true))
        .map(|d| d.nom.clone())
        .collect()
}

/// Une réponse de ce module : (code HTTP, JSON).
pub type Resultat = (u16, Value);

fn erreur(code: u16, m: impl Into<String>) -> Resultat {
    (code, json!({ "erreur": m.into() }))
}

fn lire_corps(corps: &[u8]) -> Result<(Decklist, Vec<String>), Resultat> {
    let texte = std::str::from_utf8(corps).map_err(|_| erreur(400, "liste illisible (UTF-8)"))?;
    if texte.trim().is_empty() {
        return Err(erreur(400, "liste vide"));
    }
    import::lire(texte, classer).map_err(|e| erreur(400, e))
}

/// Traite une route de ce module ; `None` si la route n'en est pas.
pub fn repondre(
    methode: &str,
    chemin: &str,
    param: &dyn Fn(&str) -> Option<String>,
    corps: &[u8],
    textes: Option<&Textes>,
) -> Option<Resultat> {
    Some(match (methode, chemin) {
        ("GET", "/api/collection") => (200, collection().json(textes)),
        ("GET", "/api/decks-maison") => {
            let v: Vec<Value> = decks::maison().iter().map(resume).collect();
            (
                200,
                json!({
                    "decks": v,
                    "archives": decks::archives_maison(),
                    "dossier": decks::dossier_maison(),
                }),
            )
        }
        // Les decks du bot (data/decks/), pour les tuiles de l'écran Jouer.
        ("GET", "/api/decks-projet") => {
            let v: Vec<Value> = decks::tous().iter().map(resume).collect();
            (200, json!({ "decks": v }))
        }
        ("GET", "/api/deck-maison") | ("GET", "/api/deck") => {
            let nom = param("nom").unwrap_or_default();
            let trouve = if chemin == "/api/deck" {
                decks::trouver(&nom).ok().filter(|d| d.nom == nom)
            } else {
                decks::maison().into_iter().find(|d| d.nom == nom)
            };
            match trouve {
                None => erreur(404, format!("deck inconnu : {nom}")),
                Some(d) => match lire_deck(&d) {
                    Err(e) => erreur(400, e),
                    Ok((texte, l)) => {
                        let mut v = verifier(&l);
                        v["nom"] = json!(d.nom);
                        v["maison"] = json!(d.maison);
                        v["texte"] = json!(texte);
                        v["liste"] = json!({
                            "arene": l.arena.iter().map(|e| json!({"n": e.count, "nom": e.name, "emplacement": e.slot})).collect::<Vec<_>>(),
                            "deck": l.deck.iter().map(|e| json!({"n": e.count, "nom": e.name, "couleur": nom_couleur(Some(couleur_liste(e.color)))})).collect::<Vec<_>>(),
                        });
                        (200, v)
                    }
                },
            }
        }
        ("POST", "/api/deck-maison/verifier") => match lire_corps(corps) {
            Err(r) => r,
            Ok((l, remarques)) => {
                let mut v = verifier(&l);
                v["remarques"] = json!(remarques);
                v["texte"] = json!(l.to_text());
                (200, v)
            }
        },
        ("POST", "/api/deck-maison/importer") => {
            let nom = param("nom").unwrap_or_default().trim().to_lowercase();
            if let Err(e) = decks::verifier_nom_maison(&nom) {
                return Some(erreur(400, e));
            }
            match lire_corps(corps) {
                Err(r) => r,
                Ok((mut l, remarques)) => {
                    if l.source.is_empty() {
                        l.source = "importé dans Arsenal".into();
                    }
                    if let Err(e) = decks::enregistrer_maison(&nom, &l) {
                        return Some(erreur(500, e));
                    }
                    let mut v = verifier(&l);
                    v["nom"] = json!(nom);
                    v["remarques"] = json!(remarques);
                    (200, v)
                }
            }
        }
        ("POST", "/api/deck-maison/archiver")
        | ("POST", "/api/deck-maison/restaurer")
        | ("POST", "/api/deck-maison/supprimer") => {
            let nom = param("nom").unwrap_or_default();
            let r = match chemin {
                "/api/deck-maison/archiver" => decks::archiver_maison(&nom),
                "/api/deck-maison/restaurer" => decks::restaurer_maison(&nom),
                _ => decks::supprimer_archive_maison(&nom),
            };
            match r {
                Ok(()) => (200, json!({ "ok": true, "nom": nom })),
                Err(e) => erreur(400, e),
            }
        }
        _ => return None,
    })
}
