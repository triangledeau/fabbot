//! # Phase de side des bots : choix appris par simulation
//!
//! Avant chaque partie, un joueur choisit ses cartes d'arène et les 40
//! cartes de son deck dans son card-pool de 55 (CR 4.1.4 à 4.1.6).
//!
//! **Méthode** (mesurée, voir `docs/bots.md`) : pour chaque paire (mon deck,
//! deck adverse), on joue des milliers de parties où mon équipement et mes 40
//! cartes sont **tirés au hasard** dans le card-pool, l'adversaire gardant le
//! choix « liste ». Une régression linéaire estime alors ce que chaque
//! exemplaire d'une carte et chaque équipement ajoute (ou retire) aux chances
//! de gagner. Le choix appris garde, dans chaque zone, l'équipement le mieux
//! noté et les 40 exemplaires les mieux notés.
//!
//! Le résultat est rangé dans un fichier texte lisible (`data/side/b1.txt`)
//! que le propriétaire peut critiquer : pour chaque paire, l'arène choisie et
//! les cartes laissées hors du deck, avec l'écart estimé.
//!
//! Pour limiter le bruit, l'estimation d'une paire est tirée vers
//! l'estimation « générale » du deck (toutes paires confondues, régression
//! « ridge » vers un a priori).

use fab_engine::card::{CardDb, Color};
use fab_engine::state::equipment_zone;
use fab_engine::{CardId, CardPool, Rng, Zone};
use fab_match::{InfoSide, Side, SideListe};
use std::sync::Arc;

/// Les zones d'arène, dans l'ordre d'affichage.
const ZONES: [Zone; 5] = [
    Zone::Weapon,
    Zone::Head,
    Zone::Chest,
    Zone::Arms,
    Zone::Legs,
];

fn nom_zone(z: Zone) -> &'static str {
    match z {
        Zone::Weapon => "armes",
        Zone::Head => "tête",
        Zone::Chest => "torse",
        Zone::Arms => "bras",
        Zone::Legs => "jambes",
        _ => "?",
    }
}

/// Les choix possibles dans un card-pool, et leur codage en variables pour
/// la régression.
///
/// Variables : `[1, je commence, nombre d'exemplaires de chaque carte de deck
/// distincte…, une variable 0/1 par option de chaque zone d'arène…]`.
#[derive(Clone, Debug)]
pub struct Options {
    /// Les cartes de deck distinctes, dans l'ordre de la liste.
    pub cartes: Vec<CardId>,
    /// Pour chaque zone d'arène présente dans le card-pool : la zone et ses
    /// options (une option = les cartes équipées dans cette zone ; pour les
    /// armes, une combinaison qui remplit les mains au mieux).
    pub zones: Vec<(Zone, Vec<Vec<CardId>>)>,
}

/// Nombre de variables fixes en tête (constante, « je commence »).
const FIXES: usize = 2;

impl Options {
    pub fn new(db: &CardDb, pool: &CardPool) -> Self {
        let mut cartes = Vec::new();
        for &c in &pool.deck {
            if !cartes.contains(&c) {
                cartes.push(c);
            }
        }
        let mut zones = Vec::new();
        for z in ZONES {
            let elems: Vec<CardId> = pool
                .arena
                .iter()
                .copied()
                .filter(|&c| equipment_zone(db.get(c)) == z)
                .collect();
            if elems.is_empty() {
                continue;
            }
            let mut opts: Vec<Vec<CardId>> = Vec::new();
            if z == Zone::Weapon {
                // Toutes les combinaisons légales qu'on ne peut plus compléter.
                let n = elems.len();
                let ok = |m: u32| {
                    let v: Vec<CardId> = (0..n)
                        .filter(|i| m >> i & 1 == 1)
                        .map(|i| elems[i])
                        .collect();
                    fab_engine::pool::arena_compatible(db, &v)
                        .is_ok()
                        .then_some(v)
                };
                for m in 1u32..(1 << n) {
                    let Some(v) = ok(m) else { continue };
                    let maximal = (0..n).all(|i| m >> i & 1 == 1 || ok(m | 1 << i).is_none());
                    let mut cle = v.clone();
                    cle.sort();
                    if maximal
                        && !opts.iter().any(|o| {
                            let mut o = o.clone();
                            o.sort();
                            o == cle
                        })
                    {
                        opts.push(v);
                    }
                }
            } else {
                for c in elems {
                    if !opts.contains(&vec![c]) {
                        opts.push(vec![c]);
                    }
                }
            }
            zones.push((z, opts));
        }
        Options { cartes, zones }
    }

    pub fn nb_variables(&self) -> usize {
        FIXES + self.cartes.len() + self.zones.iter().map(|(_, o)| o.len()).sum::<usize>()
    }

    /// Les variables d'un choix (`commence` : ce joueur joue en premier).
    pub fn variables(&self, arene: &[CardId], deck: &[CardId], commence: bool) -> Vec<f64> {
        let mut x = vec![0.0; self.nb_variables()];
        x[0] = 1.0;
        x[1] = if commence { 1.0 } else { 0.0 };
        for &c in deck {
            if let Some(i) = self.cartes.iter().position(|&k| k == c) {
                x[FIXES + i] += 1.0;
            }
        }
        let mut k = FIXES + self.cartes.len();
        for (z, opts) in &self.zones {
            let mut dans_zone: Vec<CardId> = arene
                .iter()
                .copied()
                .filter(|&c| self.zone_de(c) == Some(*z))
                .collect();
            dans_zone.sort();
            for o in opts {
                let mut o = o.clone();
                o.sort();
                if o == dans_zone {
                    x[k] = 1.0;
                }
                k += 1;
            }
        }
        x
    }

    fn zone_de(&self, c: CardId) -> Option<Zone> {
        self.zones
            .iter()
            .find(|(_, opts)| opts.iter().any(|o| o.contains(&c)))
            .map(|(z, _)| *z)
    }

    /// Un choix tiré au hasard : une option par zone, et `deck_size`
    /// exemplaires du card-pool de deck.
    pub fn tirer(
        &self,
        pool: &CardPool,
        rng: &mut Rng,
        deck_size: usize,
    ) -> (Vec<CardId>, Vec<CardId>) {
        let mut arene = Vec::new();
        for (_, opts) in &self.zones {
            let i = rng.below(opts.len() as u32) as usize;
            arene.extend_from_slice(&opts[i]);
        }
        let mut deck = pool.deck.clone();
        rng.shuffle(&mut deck);
        deck.truncate(deck_size);
        (arene, deck)
    }

    /// Le meilleur choix selon les coefficients `beta` (un par variable) :
    /// la meilleure option de chaque zone, et les `deck_size` exemplaires
    /// les mieux notés (à égalité, l'ordre de la liste). Renvoie aussi les
    /// exemplaires laissés hors du deck.
    pub fn meilleur(
        &self,
        pool: &CardPool,
        beta: &[f64],
        deck_size: usize,
    ) -> (Vec<CardId>, Vec<CardId>, Vec<CardId>) {
        let mut arene = Vec::new();
        let mut k = FIXES + self.cartes.len();
        for (_, opts) in &self.zones {
            let mut best = 0;
            for i in 1..opts.len() {
                if beta[k + i] > beta[k + best] + 1e-12 {
                    best = i;
                }
            }
            arene.extend_from_slice(&opts[best]);
            k += opts.len();
        }
        let note = |c: CardId| beta[FIXES + self.cartes.iter().position(|&k| k == c).unwrap()];
        let mut ordre: Vec<usize> = (0..pool.deck.len()).collect();
        ordre.sort_by(|&a, &b| {
            note(pool.deck[b])
                .total_cmp(&note(pool.deck[a]))
                .then(a.cmp(&b))
        });
        let mut garde = vec![false; pool.deck.len()];
        for &i in ordre.iter().take(deck_size) {
            garde[i] = true;
        }
        let deck = (0..pool.deck.len())
            .filter(|&i| garde[i])
            .map(|i| pool.deck[i])
            .collect();
        let hors = (0..pool.deck.len())
            .filter(|&i| !garde[i])
            .map(|i| pool.deck[i])
            .collect();
        (arene, deck, hors)
    }
}

/// Régression linéaire par moindres carrés, accumulée partie après partie
/// (on ne garde que XᵀX et Xᵀy).
#[derive(Clone, Debug)]
pub struct Regression {
    p: usize,
    xtx: Vec<f64>,
    xty: Vec<f64>,
    pub n: f64,
    pub somme_y: f64,
}

impl Regression {
    pub fn new(p: usize) -> Self {
        Regression {
            p,
            xtx: vec![0.0; p * p],
            xty: vec![0.0; p],
            n: 0.0,
            somme_y: 0.0,
        }
    }

    pub fn ajouter(&mut self, x: &[f64], y: f64) {
        debug_assert_eq!(x.len(), self.p);
        for i in 0..self.p {
            if x[i] == 0.0 {
                continue;
            }
            for j in 0..self.p {
                self.xtx[i * self.p + j] += x[i] * x[j];
            }
            self.xty[i] += x[i] * y;
        }
        self.n += 1.0;
        self.somme_y += y;
    }

    pub fn fusionner(&mut self, o: &Regression) {
        assert_eq!(self.p, o.p);
        for (a, b) in self.xtx.iter_mut().zip(&o.xtx) {
            *a += b;
        }
        for (a, b) in self.xty.iter_mut().zip(&o.xty) {
            *a += b;
        }
        self.n += o.n;
        self.somme_y += o.somme_y;
    }

    /// Coefficients β minimisant ‖y − Xβ‖² + λ‖β − a priori‖², sans
    /// pénaliser les `FIXES` premières variables (constante, « je commence »).
    pub fn resoudre(&self, lambda: f64, a_priori: &[f64]) -> Vec<f64> {
        let p = self.p;
        let mut a = self.xtx.clone();
        let mut b = self.xty.clone();
        for i in FIXES..p {
            a[i * p + i] += lambda;
            b[i] += lambda * a_priori[i];
        }
        // Un soupçon de pénalité partout pour rester inversible.
        for i in 0..p {
            a[i * p + i] += 1e-6;
        }
        resoudre_systeme(&mut a, &mut b, p);
        b
    }
}

/// Résout A·x = b (Gauss avec pivot partiel) ; le résultat remplace `b`.
pub(crate) fn resoudre_systeme(a: &mut [f64], b: &mut [f64], p: usize) {
    for col in 0..p {
        let piv = (col..p)
            .max_by(|&i, &j| a[i * p + col].abs().total_cmp(&a[j * p + col].abs()))
            .unwrap();
        if piv != col {
            for k in 0..p {
                a.swap(col * p + k, piv * p + k);
            }
            b.swap(col, piv);
        }
        let d = a[col * p + col];
        if d.abs() < 1e-12 {
            continue;
        }
        for i in 0..p {
            if i == col {
                continue;
            }
            let f = a[i * p + col] / d;
            if f == 0.0 {
                continue;
            }
            for k in col..p {
                a[i * p + k] -= f * a[col * p + k];
            }
            b[i] -= f * b[col];
        }
    }
    for i in 0..p {
        let d = a[i * p + i];
        b[i] = if d.abs() < 1e-12 { 0.0 } else { b[i] / d };
    }
}

// ---------------------------------------------------------------------------
// Table apprise (fichier texte)
// ---------------------------------------------------------------------------

/// Le choix appris pour un héros, contre un héros précis (`adverse`) ou
/// contre n'importe qui (`None`, estimation générale).
#[derive(Clone, Debug, PartialEq)]
pub struct Entree {
    pub heros: CardId,
    pub adverse: Option<CardId>,
    pub arene: Vec<CardId>,
    pub hors_deck: Vec<CardId>,
}

/// Toutes les entrées d'un fichier de choix appris.
#[derive(Clone, Debug, Default)]
pub struct TableSide {
    pub entrees: Vec<Entree>,
}

/// Nom d'une carte tel qu'écrit dans le fichier : « Nom (red) » pour une
/// carte de deck colorée.
pub fn nom_carte(db: &CardDb, c: CardId) -> String {
    let d = db.get(c);
    match d.color {
        Some(Color::Red) => format!("{} (red)", d.name),
        Some(Color::Yellow) => format!("{} (yellow)", d.name),
        Some(Color::Blue) => format!("{} (blue)", d.name),
        None => d.name.clone(),
    }
}

fn lire_carte(db: &CardDb, texte: &str) -> Result<CardId, String> {
    // Retirer une annotation « [ … ] ».
    let t = texte.split('[').next().unwrap_or("").trim();
    let (nom, couleur) = if let Some(n) = t.strip_suffix(" (red)") {
        (n, Some(Color::Red))
    } else if let Some(n) = t.strip_suffix(" (yellow)") {
        (n, Some(Color::Yellow))
    } else if let Some(n) = t.strip_suffix(" (blue)") {
        (n, Some(Color::Blue))
    } else {
        (t, None)
    };
    db.find(nom, couleur)
        .ok_or_else(|| format!("carte inconnue : « {t} »"))
}

fn lire_liste(db: &CardDb, texte: &str) -> Result<Vec<CardId>, String> {
    texte
        .split(" | ")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| lire_carte(db, s))
        .collect()
}

impl TableSide {
    /// Lit un fichier de choix appris. Format :
    ///
    /// ```text
    /// # commentaire
    /// = Ira, Crimson Haze | Hala          (ou « | * » : contre n'importe qui)
    /// arène : Edge of Autumn | Mask of the Swarming Claw | …
    /// hors du deck : Humble (red) [-0,8 %] | …
    /// ```
    pub fn lire(db: &CardDb, texte: &str) -> Result<Self, String> {
        let mut entrees: Vec<Entree> = Vec::new();
        for (num, ligne) in texte.lines().enumerate() {
            let ligne = ligne.trim();
            let err = |e: String| format!("ligne {} : {e}", num + 1);
            if ligne.is_empty() || ligne.starts_with('#') {
                continue;
            }
            if let Some(r) = ligne.strip_prefix('=') {
                let (h, a) = r
                    .split_once(" | ")
                    .ok_or_else(|| err("« = héros | adverse » attendu".into()))?;
                let heros = lire_carte(db, h).map_err(err)?;
                let adverse = match a.trim() {
                    "*" => None,
                    a => Some(lire_carte(db, a).map_err(err)?),
                };
                entrees.push(Entree {
                    heros,
                    adverse,
                    arene: Vec::new(),
                    hors_deck: Vec::new(),
                });
                continue;
            }
            let e = entrees
                .last_mut()
                .ok_or_else(|| err("ligne avant le premier « = »".into()))?;
            if let Some(r) = ligne.strip_prefix("arène :") {
                e.arene = lire_liste(db, r).map_err(err)?;
            } else if let Some(r) = ligne.strip_prefix("hors du deck :") {
                e.hors_deck = lire_liste(db, r).map_err(err)?;
            } else {
                return Err(err(format!("ligne non comprise : « {ligne} »")));
            }
        }
        Ok(TableSide { entrees })
    }

    /// L'entrée pour cette paire de héros, sinon l'entrée générale du héros.
    pub fn trouver(&self, heros: CardId, adverse: CardId) -> Option<&Entree> {
        self.entrees
            .iter()
            .find(|e| e.heros == heros && e.adverse == Some(adverse))
            .or_else(|| {
                self.entrees
                    .iter()
                    .find(|e| e.heros == heros && e.adverse.is_none())
            })
    }
}

/// Le texte d'une entrée, avec en commentaire les options de chaque zone et
/// l'écart estimé de chaque carte laissée hors du deck (en points de
/// pourcentage de victoire par exemplaire, par rapport à la moins bonne
/// carte gardée).
pub fn ecrire_entree(
    db: &CardDb,
    opts: &Options,
    pool: &CardPool,
    adverse: Option<CardId>,
    beta: &[f64],
    deck_size: usize,
) -> String {
    let (arene, deck, hors) = opts.meilleur(pool, beta, deck_size);
    let note = |c: CardId| beta[FIXES + opts.cartes.iter().position(|&k| k == c).unwrap()];
    let pire_garde = deck.iter().map(|&c| note(c)).fold(f64::INFINITY, f64::min);
    let pct = |v: f64| format!("{:+.1} %", 100.0 * v).replace('.', ",");
    let mut s = String::new();
    s.push_str(&format!(
        "= {} | {}\n",
        nom_carte(db, pool.hero),
        adverse.map_or("*".to_string(), |a| nom_carte(db, a))
    ));
    let mut k = FIXES + opts.cartes.len();
    for (z, o) in &opts.zones {
        if o.len() > 1 {
            let best = (0..o.len())
                .map(|i| beta[k + i])
                .fold(f64::NEG_INFINITY, f64::max);
            let detail: Vec<String> = o
                .iter()
                .enumerate()
                .map(|(i, cartes)| {
                    let noms: Vec<String> = cartes.iter().map(|&c| nom_carte(db, c)).collect();
                    format!("{} [{}]", noms.join(" + "), pct(beta[k + i] - best))
                })
                .collect();
            s.push_str(&format!("# {} : {}\n", nom_zone(*z), detail.join(" ; ")));
        }
        k += o.len();
    }
    let noms: Vec<String> = arene.iter().map(|&c| nom_carte(db, c)).collect();
    s.push_str(&format!("arène : {}\n", noms.join(" | ")));
    let noms: Vec<String> = hors
        .iter()
        .map(|&c| format!("{} [{}]", nom_carte(db, c), pct(note(c) - pire_garde)))
        .collect();
    s.push_str(&format!("hors du deck : {}\n", noms.join(" | ")));
    s
}

// ---------------------------------------------------------------------------
// Les façons de choisir des bots
// ---------------------------------------------------------------------------

/// Le choix appris (`data/side/b1.txt`), compilé dans le programme.
pub const TABLE_B1: &str = include_str!("../../../data/side/b1.txt");

/// Bot de side qui suit une table apprise. Sans entrée pour son héros, ou
/// si l'entrée n'est pas légale pour ce card-pool, il choisit comme
/// [`SideListe`].
pub struct SideAppris {
    table: Arc<TableSide>,
    /// N'utiliser que l'estimation générale du héros (pas celle de la paire).
    general: bool,
}

impl SideAppris {
    pub fn new(table: Arc<TableSide>) -> Self {
        SideAppris {
            table,
            general: false,
        }
    }

    /// Variante qui ignore le héros adverse (sert à mesurer l'intérêt des
    /// entrées par paire).
    pub fn general(table: Arc<TableSide>) -> Self {
        SideAppris {
            table,
            general: true,
        }
    }

    /// Avec la table compilée [`TABLE_B1`].
    pub fn par_defaut(db: &CardDb) -> Self {
        SideAppris::new(Arc::new(
            TableSide::lire(db, TABLE_B1).expect("data/side/b1.txt valide"),
        ))
    }

    fn entree(&self, info: &InfoSide) -> Option<&Entree> {
        if self.general {
            self.table
                .entrees
                .iter()
                .find(|e| e.heros == info.pool.hero && e.adverse.is_none())
        } else {
            self.table.trouver(info.pool.hero, info.heros_adverse)
        }
    }
}

impl Side for SideAppris {
    fn nom(&self) -> &str {
        if self.general {
            "general"
        } else {
            "appris"
        }
    }

    fn arene(&mut self, info: &InfoSide) -> Vec<CardId> {
        match self.entree(info) {
            Some(e) if info.pool.verifier_arene(info.db, &e.arene).is_ok() => e.arene.clone(),
            _ => SideListe.arene(info),
        }
    }

    fn deck(&mut self, info: &InfoSide, arene: &[CardId], adv: &[CardId]) -> Vec<CardId> {
        if let Some(e) = self.entree(info) {
            let mut deck = info.pool.deck.clone();
            let mut ok = true;
            for c in &e.hors_deck {
                match deck.iter().rposition(|k| k == c) {
                    Some(i) => {
                        deck.remove(i);
                    }
                    None => ok = false,
                }
            }
            if ok
                && info
                    .pool
                    .verifier_deck(info.db, &deck, info.deck_size)
                    .is_ok()
            {
                return deck;
            }
        }
        SideListe.deck(info, arene, adv)
    }
}

/// Bot de side qui tire tout au hasard (sert à l'apprentissage).
pub struct SideHasard {
    opts: Option<Options>,
    rng: Rng,
}

impl SideHasard {
    pub fn new(graine: u64) -> Self {
        SideHasard {
            opts: None,
            rng: Rng::new(graine),
        }
    }
}

impl Side for SideHasard {
    fn nom(&self) -> &str {
        "hasard"
    }

    fn arene(&mut self, info: &InfoSide) -> Vec<CardId> {
        let opts = self
            .opts
            .get_or_insert_with(|| Options::new(info.db, info.pool));
        opts.tirer(info.pool, &mut self.rng, info.deck_size).0
    }

    fn deck(&mut self, info: &InfoSide, _: &[CardId], _: &[CardId]) -> Vec<CardId> {
        let mut deck = info.pool.deck.clone();
        self.rng.shuffle(&mut deck);
        deck.truncate(info.deck_size);
        deck
    }
}

/// Les façons de choisir connues, par leur nom.
pub const SIDES: [&str; 4] = ["liste", "appris", "general", "hasard"];

/// Crée un bot de side par son nom (`liste`, `appris`, `general`, `hasard`).
pub fn creer_side(
    nom: &str,
    db: &CardDb,
    table: Option<Arc<TableSide>>,
    graine: u64,
) -> Option<Box<dyn Side + Send>> {
    match nom {
        "liste" => Some(Box::new(SideListe)),
        "appris" => Some(Box::new(match table {
            Some(t) => SideAppris::new(t),
            None => SideAppris::par_defaut(db),
        })),
        "general" => Some(Box::new(SideAppris::general(table.unwrap_or_else(|| {
            Arc::new(TableSide::lire(db, TABLE_B1).expect("table valide"))
        })))),
        "hasard" => Some(Box::new(SideHasard::new(graine))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Données sans bruit : y = 0,5 + 0,1·a − 0,2·b ; la régression retrouve
    // les effets de a et b.
    #[test]
    fn la_regression_retrouve_des_effets_connus() {
        let mut r = Regression::new(4);
        let mut rng = Rng::new(3);
        for _ in 0..2000 {
            let a = rng.below(3) as f64;
            let b = rng.below(2) as f64;
            let c = rng.below(2) as f64;
            r.ajouter(&[1.0, c, a, b], 0.5 + 0.1 * a - 0.2 * b);
        }
        let beta = r.resoudre(1e-3, &[0.0; 4]);
        assert!((beta[2] - 0.1).abs() < 1e-3, "{beta:?}");
        assert!((beta[3] + 0.2).abs() < 1e-3, "{beta:?}");
        assert!(beta[1].abs() < 1e-3, "{beta:?}");
    }

    // Sans données, une forte pénalité ramène les coefficients à l'a priori.
    #[test]
    fn sans_donnees_l_a_priori_l_emporte() {
        let r = Regression::new(3);
        let beta = r.resoudre(1000.0, &[0.0, 0.0, 0.07]);
        assert!((beta[2] - 0.07).abs() < 1e-6);
    }

    #[test]
    fn une_carte_inconnue_dans_la_table_est_signalee() {
        let db = CardDb::new();
        let e = TableSide::lire(&db, "= Personne | *\n").unwrap_err();
        assert!(e.contains("ligne 1"), "{e}");
        assert!(TableSide::lire(&db, "arène : X\n").is_err());
    }
}
