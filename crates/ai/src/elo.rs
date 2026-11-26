//! Classement Elo des bots à partir des résultats de leurs parties.
//!
//! Le modèle d'Elo dit : si A a `d` points d'Elo de plus que B, A marque en
//! moyenne `1 / (1 + 10^(-d/400))` point par partie (victoire 1, nulle ½,
//! défaite 0). Exemples : +0 → 50 %, +100 → 64 %, +200 → 76 %, +400 → 91 %.
//!
//! Avec plusieurs bots, on cherche les Elo qui expliquent le mieux tous les
//! résultats (modèle de Bradley-Terry, algorithme MM de Hunter 2004). Le
//! premier bot (B0, l'aléatoire) sert de référence : son Elo vaut 0.

/// Elo d'écart correspondant à un score moyen `s` (entre 0 et 1, exclu).
pub fn ecart(s: f64) -> f64 {
    let s = s.clamp(1e-6, 1.0 - 1e-6);
    -400.0 * (1.0 / s - 1.0).log10()
}

/// Écart d'Elo et intervalle de confiance à 95 % pour `points` marqués en
/// `parties` parties (nulle = ½ point). Renvoie `(elo, bas, haut)`.
///
/// L'intervalle vient de l'erreur type du score moyen (loi normale) ; il est
/// valable quand il y a beaucoup de parties (des centaines au moins).
pub fn ecart_avec_intervalle(points: f64, parties: f64, nulles: f64) -> (f64, f64, f64) {
    let n = parties.max(1.0);
    let s = points / n;
    // Variance d'une partie : E[x²] - s², avec x ∈ {0, ½, 1}.
    let gagnees = points - nulles / 2.0;
    let ex2 = (gagnees + nulles / 4.0) / n;
    let var = (ex2 - s * s).max(0.0);
    let se = (var / n).sqrt();
    (ecart(s), ecart(s - 1.96 * se), ecart(s + 1.96 * se))
}

/// Résultats d'un tournoi entre bots.
#[derive(Clone, Debug)]
pub struct Tableau {
    pub noms: Vec<String>,
    /// `points[i][j]` : points marqués par `i` contre `j`.
    pub points: Vec<Vec<f64>>,
    /// `parties[i][j]` : parties jouées entre `i` et `j` (symétrique).
    pub parties: Vec<Vec<f64>>,
}

impl Tableau {
    pub fn new(noms: &[&str]) -> Self {
        let n = noms.len();
        Tableau {
            noms: noms.iter().map(|s| s.to_string()).collect(),
            points: vec![vec![0.0; n]; n],
            parties: vec![vec![0.0; n]; n],
        }
    }

    /// Ajoute une partie : `score` = points de `i` (1, ½ ou 0).
    pub fn ajouter(&mut self, i: usize, j: usize, score: f64) {
        self.points[i][j] += score;
        self.points[j][i] += 1.0 - score;
        self.parties[i][j] += 1.0;
        self.parties[j][i] += 1.0;
    }

    /// Elo de chaque bot, le premier valant 0 (Bradley-Terry, MM).
    pub fn elo(&self) -> Vec<f64> {
        let n = self.noms.len();
        // Un bot sans aucun point (ou sans aucune défaite) aurait un Elo
        // infini : on ajoute ½ partie nulle fictive contre chaque adversaire
        // rencontré, ce qui borne les valeurs sans changer les gros écarts.
        let mut pts = self.points.clone();
        let mut nb = self.parties.clone();
        for i in 0..n {
            for j in 0..n {
                if i != j && self.parties[i][j] > 0.0 {
                    pts[i][j] += 0.25;
                    nb[i][j] += 0.5;
                }
            }
        }
        let mut g = vec![1.0f64; n];
        for _ in 0..10_000 {
            let mut nouveau = g.clone();
            let mut bouge = 0.0f64;
            for i in 0..n {
                let w: f64 = pts[i].iter().sum();
                let den: f64 = (0..n)
                    .filter(|&j| j != i && nb[i][j] > 0.0)
                    .map(|j| nb[i][j] / (g[i] + g[j]))
                    .sum();
                if den > 0.0 {
                    nouveau[i] = w / den;
                }
            }
            let ref0 = nouveau[0];
            for i in 0..n {
                nouveau[i] /= ref0;
                bouge = bouge.max((nouveau[i] - g[i]).abs() / g[i]);
            }
            g = nouveau;
            if bouge < 1e-12 {
                break;
            }
        }
        g.iter().map(|x| 400.0 * x.log10()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_score_de_50_pourcent_donne_un_ecart_nul() {
        assert!(ecart(0.5).abs() < 1e-9);
    }

    #[test]
    fn un_score_de_76_pourcent_vaut_environ_200_points() {
        let e = ecart(1.0 / (1.0 + 10f64.powf(-0.5)));
        assert!((e - 200.0).abs() < 1e-6, "{e}");
    }

    #[test]
    fn l_intervalle_entoure_l_estimation_et_se_resserre_avec_les_parties() {
        let (e, lo, hi) = ecart_avec_intervalle(700.0, 1000.0, 0.0);
        assert!(lo < e && e < hi);
        let (_, lo2, hi2) = ecart_avec_intervalle(7000.0, 10000.0, 0.0);
        assert!(hi2 - lo2 < hi - lo);
    }

    #[test]
    fn deux_bots_le_classement_retrouve_l_ecart_direct() {
        let mut t = Tableau::new(&["B0", "B1"]);
        for k in 0..10_000 {
            t.ajouter(1, 0, if k % 4 == 0 { 0.0 } else { 1.0 });
        }
        let elo = t.elo();
        assert_eq!(elo[0], 0.0);
        assert!((elo[1] - ecart(0.75)).abs() < 1.0, "{elo:?}");
    }

    #[test]
    fn trois_bots_les_ecarts_s_additionnent() {
        // A bat B à 76 % (≈ +200), B bat C à 76 %, A bat C à 91 % (≈ +400).
        let mut t = Tableau::new(&["C", "B", "A"]);
        let p200 = 1.0 / (1.0 + 10f64.powf(-0.5));
        let p400 = 1.0 / (1.0 + 10f64.powf(-1.0));
        let n = 100_000.0;
        t.points[1][0] = p200 * n;
        t.points[0][1] = (1.0 - p200) * n;
        t.points[2][1] = p200 * n;
        t.points[1][2] = (1.0 - p200) * n;
        t.points[2][0] = p400 * n;
        t.points[0][2] = (1.0 - p400) * n;
        for (i, ligne) in t.parties.iter_mut().enumerate() {
            for (j, case) in ligne.iter_mut().enumerate() {
                if i != j {
                    *case = n;
                }
            }
        }
        let elo = t.elo();
        assert!((elo[1] - 200.0).abs() < 2.0, "{elo:?}");
        assert!((elo[2] - 400.0).abs() < 2.0, "{elo:?}");
    }

    #[test]
    fn un_bot_qui_gagne_tout_a_un_elo_fini() {
        let mut t = Tableau::new(&["B0", "B1"]);
        for _ in 0..100 {
            t.ajouter(1, 0, 1.0);
        }
        let elo = t.elo();
        assert!(elo[1].is_finite() && elo[1] > 400.0, "{elo:?}");
    }
}
