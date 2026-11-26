//! B3 : la recherche de B2 avec une **évaluation apprise**.
//!
//! B2 arrête ses parties simulées après quelques tours et les **estime** avec
//! une formule fixe (vie + 1,5 par carte en main…). B3 garde la même
//! recherche, mais remplace cette formule par une évaluation dont les poids
//! ont été **appris** sur des parties jouées par les bots eux-mêmes
//! (« auto-jeu », `fab-sim --apprendre-b3`).
//!
//! ## L'évaluation
//!
//! Pour une position (au début d'un tour), chaque joueur reçoit une note :
//! la somme de ses **caractéristiques** multipliées par leur poids
//! ([`caracteristiques`]) :
//! - des nombres génériques : vie, vie sous 10 et sous 5, cartes en main,
//!   arsenal, deck, équipements et leur défense, cartes à blood debt dans
//!   la zone de bannissement, cimetière, bannissement, « c'est son tour » ;
//! - son **héros** (force du deck), et la vie et la main **propres au
//!   héros** (une carte en main ne vaut pas la même chose pour tous) ;
//! - chacun de ses **permanents**, carte par carte (un Runechant, un
//!   Frostbite, une Ash, un dragon…).
//!
//! Les chances de gagner du joueur 0 sont σ(note₀ − note₁), avec
//! σ(x) = 1 / (1 + e^−x). Le modèle est donc symétrique : échanger les
//! joueurs donne 1 − la valeur.
//!
//! ## L'apprentissage
//!
//! [`Apprentissage`] reçoit des positions (début de chaque tour) de parties
//! complètes, avec le résultat final de la partie, et trouve les poids par
//! **régression logistique** (méthode de Newton, petite pénalité « ridge »).
//! Les poids sont écrits dans un fichier texte lisible
//! (`data/b3/evaluation.txt`), par **nom** de carte : un nouveau deck n'a
//! simplement pas encore de poids propres (ils valent 0) tant qu'on n'a pas
//! refait l'apprentissage.
//!
//! Information : l'évaluation ne sert que dans les **mondes imaginés** de la
//! recherche (voir [`crate::determinisation`]) ; elle ne regarde jamais la
//! vraie partie.

use crate::side::{nom_carte, resoudre_systeme};
use fab_engine::{CardDb, CardId, GameState, Keywords, PlayerId, Types};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Les caractéristiques génériques, dans l'ordre des poids.
pub const GENERIQUES: [&str; 12] = [
    "vie",
    "vie sous 10",
    "vie sous 5",
    "cartes en main",
    "arsenal occupé",
    "cartes dans le deck",
    "équipements",
    "défense des équipements",
    "blood debt dans le bannissement",
    "cartes au cimetière",
    "cartes bannies",
    "c'est son tour",
];

/// Une caractéristique d'un joueur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Carac {
    /// Une des [`GENERIQUES`].
    Generique(u8),
    /// Son héros (vaut 1).
    Heros(CardId),
    /// Sa vie, pour ce héros.
    VieHeros(CardId),
    /// Ses cartes en main et en arsenal, pour ce héros.
    MainHeros(CardId),
    /// Un permanent qu'il contrôle (vaut 1 par exemplaire).
    Permanent(CardId),
}

impl Carac {
    /// Code compact (pour l'apprentissage).
    pub fn code(self) -> u32 {
        match self {
            Carac::Generique(k) => k as u32,
            Carac::Heros(c) => (1 << 16) | c.0 as u32,
            Carac::VieHeros(c) => (2 << 16) | c.0 as u32,
            Carac::MainHeros(c) => (3 << 16) | c.0 as u32,
            Carac::Permanent(c) => (4 << 16) | c.0 as u32,
        }
    }

    pub fn depuis_code(code: u32) -> Carac {
        let c = CardId((code & 0xFFFF) as u16);
        match code >> 16 {
            0 => Carac::Generique(code as u8),
            1 => Carac::Heros(c),
            2 => Carac::VieHeros(c),
            3 => Carac::MainHeros(c),
            _ => Carac::Permanent(c),
        }
    }

    /// Nom écrit dans le fichier des poids.
    pub fn nom(self, db: &CardDb) -> String {
        match self {
            Carac::Generique(k) => GENERIQUES[k as usize].to_string(),
            Carac::Heros(c) => format!("héros {}", nom_carte(db, c)),
            Carac::VieHeros(c) => format!("vie × héros {}", nom_carte(db, c)),
            Carac::MainHeros(c) => format!("main × héros {}", nom_carte(db, c)),
            Carac::Permanent(c) => format!("permanent {}", nom_carte(db, c)),
        }
    }
}

/// Le héros « d'origine » d'un joueur (Arakni peut devenir un Agent of
/// Chaos, CR 8.5.25 : on garde le héros de départ).
pub fn heros_d_origine(g: &GameState, p: PlayerId) -> CardId {
    g.became
        .iter()
        .find(|(q, _)| *q == p)
        .map(|&(_, c)| c)
        .unwrap_or_else(|| g.obj(g.player(p).hero).card)
}

/// Les caractéristiques du joueur `p` : `f(caractéristique, valeur)`.
pub fn caracteristiques(g: &GameState, p: PlayerId, f: &mut impl FnMut(Carac, f32)) {
    let ps = g.player(p);
    let vie = ps.life as f32;
    let main = (ps.hand.len() + ps.arsenal.is_some() as usize) as f32;
    let mut gen = |k: u8, v: f32| {
        if v != 0.0 {
            f(Carac::Generique(k), v)
        }
    };
    gen(0, vie);
    gen(1, (10.0 - vie).max(0.0));
    gen(2, (5.0 - vie).max(0.0));
    gen(3, ps.hand.len() as f32);
    gen(4, ps.arsenal.is_some() as u8 as f32);
    gen(5, ps.deck.len() as f32);
    let mut nb = 0.0;
    let mut def = 0.0;
    for e in ps.equipment() {
        nb += 1.0;
        def += g.defense(e).max(0) as f32;
    }
    gen(6, nb);
    gen(7, def);
    // CR 8.3.11a : seules les cartes face visible font perdre de la vie.
    let dette = ps
        .banished
        .iter()
        .filter(|&&c| !g.obj(c).face_down && g.def(c).keywords.has(Keywords::BLOOD_DEBT))
        .count();
    gen(8, dette as f32);
    gen(9, ps.graveyard.len() as f32);
    gen(10, ps.banished.len() as f32);
    gen(11, (g.turn_player == p) as u8 as f32);
    let h = heros_d_origine(g, p);
    f(Carac::Heros(h), 1.0);
    f(Carac::VieHeros(h), vie);
    if main != 0.0 {
        f(Carac::MainHeros(h), main);
    }
    for &o in &g.permanents {
        let ob = g.obj(o);
        if ob.controller != p {
            continue;
        }
        let d = g.def(o);
        // Le héros, les armes et les équipements ont leurs propres
        // caractéristiques ; les attaques-proxy ne sont pas des permanents.
        if ob.proxy_of.is_some()
            || d.types.any(Types::HERO | Types::WEAPON | Types::EQUIPMENT)
            || d.types.has(Types::DEMI_HERO)
        {
            continue;
        }
        f(Carac::Permanent(ob.card), 1.0);
    }
}

// ---------------------------------------------------------------------------
// Le modèle (poids par nom) et son index (poids par carte, pour une base)
// ---------------------------------------------------------------------------

/// Les poids appris, rangés par nom (indépendants de la base de cartes).
#[derive(Clone, Debug, Default)]
pub struct Modele {
    /// Nom → poids, dans l'ordre du fichier.
    pub poids: Vec<(String, f32)>,
}

/// Fichier des poids par défaut de B3.
pub const MODELE_B3: &str = include_str!("../../../data/b3/evaluation.txt");

impl Modele {
    /// Lit un fichier de poids : une ligne `nom = poids` par
    /// caractéristique, `#` pour les commentaires.
    pub fn lire(texte: &str) -> Result<Self, String> {
        let mut poids = Vec::new();
        for (num, ligne) in texte.lines().enumerate() {
            let ligne = ligne.trim();
            if ligne.is_empty() || ligne.starts_with('#') {
                continue;
            }
            let (nom, v) = ligne
                .rsplit_once(" = ")
                .ok_or_else(|| format!("ligne {} : « nom = poids » attendu", num + 1))?;
            let v: f32 = v
                .trim()
                .parse()
                .map_err(|_| format!("ligne {} : poids illisible « {v} »", num + 1))?;
            poids.push((nom.trim().to_string(), v));
        }
        Ok(Modele { poids })
    }

    /// Les poids rangés par carte de `db`. Un nom inconnu de la base est
    /// ignoré (il vient d'un deck qui n'est pas dans cette base).
    pub fn indexer(&self, db: &CardDb) -> Evaluation {
        let n = db.len();
        let mut e = Evaluation {
            generique: [0.0; GENERIQUES.len()],
            heros: vec![[0.0; 3]; n],
            permanent: vec![0.0; n],
        };
        let par_nom: HashMap<&str, f32> =
            self.poids.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        for (k, nom) in GENERIQUES.iter().enumerate() {
            e.generique[k] = par_nom.get(nom).copied().unwrap_or(0.0);
        }
        for i in 0..n {
            let c = CardId(i as u16);
            let nom = nom_carte(db, c);
            let lire = |prefixe: &str| {
                par_nom
                    .get(format!("{prefixe}{nom}").as_str())
                    .copied()
                    .unwrap_or(0.0)
            };
            e.heros[i] = [lire("héros "), lire("vie × héros "), lire("main × héros ")];
            e.permanent[i] = lire("permanent ");
        }
        e
    }

    /// Le modèle par défaut de B3 (`data/b3/evaluation.txt`, lu une fois).
    pub fn par_defaut() -> &'static Modele {
        static M: OnceLock<Modele> = OnceLock::new();
        M.get_or_init(|| Modele::lire(MODELE_B3).expect("data/b3/evaluation.txt valide"))
    }

    /// Un modèle lu dans un fichier (une seule fois par chemin, gardé pour
    /// toute la durée du programme).
    pub fn depuis_fichier(chemin: &str) -> Result<&'static Modele, String> {
        static CACHE: OnceLock<Mutex<HashMap<String, &'static Modele>>> = OnceLock::new();
        let mut cache = CACHE.get_or_init(Default::default).lock().unwrap();
        if let Some(m) = cache.get(chemin) {
            return Ok(m);
        }
        let texte =
            std::fs::read_to_string(chemin).map_err(|e| format!("lecture de {chemin} : {e}"))?;
        let m: &'static Modele = Box::leak(Box::new(Modele::lire(&texte)?));
        cache.insert(chemin.to_string(), m);
        Ok(m)
    }
}

/// Les poids d'un [`Modele`] rangés par carte d'une base (rapide à lire).
#[derive(Clone, Debug)]
pub struct Evaluation {
    generique: [f32; GENERIQUES.len()],
    /// Par carte : [héros, vie × héros, main × héros].
    heros: Vec<[f32; 3]>,
    permanent: Vec<f32>,
}

impl Evaluation {
    /// Poids d'une caractéristique.
    pub fn poids(&self, c: Carac) -> f32 {
        let i = |c: CardId| c.0 as usize;
        match c {
            Carac::Generique(k) => self.generique[k as usize],
            Carac::Heros(h) => self.heros.get(i(h)).map_or(0.0, |w| w[0]),
            Carac::VieHeros(h) => self.heros.get(i(h)).map_or(0.0, |w| w[1]),
            Carac::MainHeros(h) => self.heros.get(i(h)).map_or(0.0, |w| w[2]),
            Carac::Permanent(p) => self.permanent.get(i(p)).copied().unwrap_or(0.0),
        }
    }

    /// Note d'un joueur (somme des caractéristiques × poids).
    pub fn note(&self, g: &GameState, p: PlayerId) -> f32 {
        let mut s = 0.0;
        caracteristiques(g, p, &mut |c, v| s += self.poids(c) * v);
        s
    }

    /// Chances de gagner du joueur 0, entre 0 et 1 (avec la force des
    /// héros : c'est la prédiction apprise).
    pub fn estimer(&self, g: &GameState) -> f32 {
        let x = self.note(g, PlayerId::P0) - self.note(g, PlayerId::P1);
        1.0 / (1.0 + (-x).exp())
    }

    /// L'estimation utilisée par la recherche : la même, **sans** la force
    /// des héros. Ce terme est le même pour tous les coups d'une décision ;
    /// gardé, il écrase les estimations près de 0 ou de 1 dans un matchup
    /// déséquilibré (Levia : −3,3), et la recherche ne voit plus de
    /// différence entre les coups (mesuré : Levia 0 % contre B2).
    pub fn estimer_pour_la_recherche(&self, g: &GameState) -> f32 {
        let h0 = self.poids(Carac::Heros(heros_d_origine(g, PlayerId::P0)));
        let h1 = self.poids(Carac::Heros(heros_d_origine(g, PlayerId::P1)));
        let x = self.note(g, PlayerId::P0) - h0 - self.note(g, PlayerId::P1) + h1;
        1.0 / (1.0 + (-x).exp())
    }
}

// ---------------------------------------------------------------------------
// Apprentissage (régression logistique)
// ---------------------------------------------------------------------------

/// Les positions observées : pour chacune, les caractéristiques du joueur 0
/// moins celles du joueur 1, et le résultat final pour le joueur 0.
#[derive(Clone, Debug, Default)]
pub struct Apprentissage {
    /// (code de la caractéristique, valeur), toutes positions bout à bout.
    entrees: Vec<(u32, f32)>,
    /// Pour chaque position : (début dans `entrees`, résultat 0 / ½ / 1).
    positions: Vec<(u32, f32)>,
    tampon: Vec<(u32, f32)>,
}

impl Apprentissage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn nb_positions(&self) -> usize {
        self.positions.len()
    }

    /// Ajoute une position de la partie et son résultat final pour le joueur 0.
    pub fn ajouter(&mut self, g: &GameState, resultat: f32) {
        let t = &mut self.tampon;
        t.clear();
        caracteristiques(g, PlayerId::P0, &mut |c, v| t.push((c.code(), v)));
        caracteristiques(g, PlayerId::P1, &mut |c, v| t.push((c.code(), -v)));
        t.sort_unstable_by_key(|e| e.0);
        let debut = self.entrees.len() as u32;
        let mut k = 0;
        while k < t.len() {
            let (code, mut v) = t[k];
            k += 1;
            while k < t.len() && t[k].0 == code {
                v += t[k].1;
                k += 1;
            }
            if v != 0.0 {
                self.entrees.push((code, v));
            }
        }
        self.positions.push((debut, resultat));
    }

    pub fn fusionner(&mut self, o: &Apprentissage) {
        let decalage = self.entrees.len() as u32;
        self.entrees.extend_from_slice(&o.entrees);
        self.positions
            .extend(o.positions.iter().map(|&(d, y)| (d + decalage, y)));
    }

    fn position(&self, i: usize) -> &[(u32, f32)] {
        let debut = self.positions[i].0 as usize;
        let fin = self
            .positions
            .get(i + 1)
            .map_or(self.entrees.len(), |p| p.0 as usize);
        &self.entrees[debut..fin]
    }

    /// Perte logarithmique moyenne de ces positions avec ces poids.
    pub fn perte(&self, poids: &[(Carac, f64)]) -> f64 {
        let w: HashMap<u32, f64> = poids.iter().map(|&(c, v)| (c.code(), v)).collect();
        let mut perte = 0.0;
        for i in 0..self.positions.len() {
            let z: f64 = self
                .position(i)
                .iter()
                .map(|&(c, v)| w.get(&c).copied().unwrap_or(0.0) * v as f64)
                .sum();
            let p = (1.0 / (1.0 + (-z).exp())).clamp(1e-9, 1.0 - 1e-9);
            let y = self.positions[i].1 as f64;
            perte -= y * p.ln() + (1.0 - y) * (1.0 - p).ln();
        }
        perte / self.positions.len().max(1) as f64
    }

    /// Régression logistique (Newton) avec la pénalité `lambda` × ‖poids‖².
    /// Renvoie les poids par caractéristique, et la perte logarithmique
    /// moyenne (plus petite = meilleures prédictions).
    pub fn resoudre(&self, lambda: f64) -> (Vec<(Carac, f64)>, f64) {
        // Index dense des caractéristiques rencontrées.
        let mut codes: Vec<u32> = self.entrees.iter().map(|e| e.0).collect();
        codes.sort_unstable();
        codes.dedup();
        let index: HashMap<u32, usize> = codes.iter().enumerate().map(|(i, &c)| (c, i)).collect();
        let d = codes.len();
        let mut w = vec![0.0f64; d];
        let mut perte = 0.0;
        let mut x: Vec<(usize, f64)> = Vec::with_capacity(64);
        for _ in 0..30 {
            let mut h = vec![0.0f64; d * d];
            let mut grad = vec![0.0f64; d];
            perte = 0.0;
            for i in 0..self.positions.len() {
                x.clear();
                x.extend(self.position(i).iter().map(|&(c, v)| (index[&c], v as f64)));
                let y = self.positions[i].1 as f64;
                let z: f64 = x.iter().map(|&(j, v)| w[j] * v).sum();
                let p = 1.0 / (1.0 + (-z).exp());
                let p = p.clamp(1e-9, 1.0 - 1e-9);
                perte -= y * p.ln() + (1.0 - y) * (1.0 - p).ln();
                let r = y - p;
                let s = p * (1.0 - p);
                for &(j, vj) in &x {
                    grad[j] += r * vj;
                    for &(k, vk) in &x {
                        h[j * d + k] += s * vj * vk;
                    }
                }
            }
            for j in 0..d {
                grad[j] -= lambda * w[j];
                h[j * d + j] += lambda + 1e-9;
            }
            resoudre_systeme(&mut h, &mut grad, d);
            let mut pas = 0.0f64;
            for j in 0..d {
                w[j] += grad[j];
                pas = pas.max(grad[j].abs());
            }
            if pas < 1e-6 {
                break;
            }
        }
        let n = self.positions.len().max(1) as f64;
        let poids = codes
            .iter()
            .zip(&w)
            .map(|(&c, &v)| (Carac::depuis_code(c), v))
            .collect();
        (poids, perte / n)
    }
}

/// Le texte du fichier des poids (génériques d'abord, puis héros, puis
/// permanents, par ordre alphabétique).
pub fn ecrire_modele(db: &CardDb, poids: &[(Carac, f64)], entete: &str) -> String {
    let mut lignes: Vec<(u8, String, f64)> = poids
        .iter()
        .map(|&(c, v)| {
            let rang = match c {
                Carac::Generique(_) => 0,
                Carac::Heros(_) | Carac::VieHeros(_) | Carac::MainHeros(_) => 1,
                Carac::Permanent(_) => 2,
            };
            let cle = match c {
                Carac::Generique(k) => format!("{k:02}"),
                Carac::Heros(h) | Carac::VieHeros(h) | Carac::MainHeros(h) => {
                    let sous = match c {
                        Carac::Heros(_) => 0,
                        Carac::VieHeros(_) => 1,
                        _ => 2,
                    };
                    format!("{} {sous}", nom_carte(db, h))
                }
                Carac::Permanent(p) => nom_carte(db, p),
            };
            (rang, format!("{cle}\u{0}{}", c.nom(db)), v)
        })
        .collect();
    lignes.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut t = String::from(entete);
    let titres = [
        "\n# --- Caractéristiques génériques ---\n",
        "\n# --- Héros : force du deck, vie et cartes (main + arsenal) propres au héros ---\n",
        "\n# --- Permanents contrôlés (par exemplaire) ---\n",
    ];
    let mut rang = u8::MAX;
    for (r, cle, v) in lignes {
        if r != rang {
            rang = r;
            t.push_str(titres[r as usize]);
        }
        let nom = cle.split('\u{0}').nth(1).unwrap_or("");
        t.push_str(&format!("{nom} = {v:+.4}\n"));
    }
    t
}
