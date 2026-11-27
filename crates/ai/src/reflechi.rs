//! Le niveau **réfléchi** : la recherche de B3, sur tous les cœurs de la
//! machine, avec un temps de réflexion qui dépend de la décision.
//!
//! B3 fait 300 itérations par décision (quelques millisecondes). Le niveau
//! réfléchi garde exactement la même recherche ([`crate::b2`], estimation
//! apprise de [`crate::b3`]) mais la laisse tourner bien plus longtemps :
//! - **rien** quand une seule option compte (deux exemplaires identiques en
//!   main ne font qu'une option) ;
//! - un **temps prévu** qui grandit avec le nombre d'options et le genre de
//!   décision ([`ReglagesReflechi::temps_prevu`]) : un ordre de pitch vaut
//!   peu, une attaque ou une défense beaucoup ; plafonné à
//!   [`ReglagesReflechi::duree_max`] (2 minutes par défaut) ;
//! - un **arrêt anticipé** quand le choix est joué d'avance : l'option la
//!   plus visitée a tant d'avance que les itérations encore prévues ne
//!   peuvent plus la rattraper (la recherche joue l'option la plus visitée,
//!   continuer ne changerait rien) ;
//! - un arrêt **à la demande** (bouton « Joue maintenant » de la page) :
//!   [`Suivi::jouer_maintenant`] ; le bot joue alors le meilleur choix
//!   trouvé jusque-là.
//!
//! ## Plusieurs cœurs
//!
//! Chaque fil (thread) fait **sa propre recherche** sur la même position,
//! avec son propre hasard (« parallélisation à la racine », Chaslot,
//! Winands et van den Herik, 2008) ; on additionne ensuite, option par
//! option, les visites et les résultats de tous les fils. Les fils ne se
//! partagent rien pendant la recherche : pas d'attente entre eux. Avec un
//! arbre limité à la décision présente (profondeur 1, le réglage mesuré de
//! B3), cela revient à faire autant d'itérations d'un seul arbre.
//!
//! ## Suivre la réflexion
//!
//! [`Suivi`] est partagé (`Arc`) avec l'interface : il donne plusieurs fois
//! par seconde le temps écoulé, le temps prévu, le nombre d'itérations et
//! les options avec leurs chances estimées ([`Progression`]).
//!
//! Les résultats ne sont pas reproductibles à l'identique : ils dépendent
//! de la vitesse de la machine et de l'ordre des fils. Pour les tournois et
//! l'apprentissage, on garde B3 (rapide et reproductible).
//!
//! Information : chaque fil est un [`IsmctsAgent`] ordinaire ; il ne lit
//! l'information cachée que pour la redistribuer au hasard
//! ([`crate::determinisation`]).

use crate::b2::{cles_des_options, meilleure_option, IsmctsAgent, ReglagesB2, StatOption};
use crate::Agent;
use fab_engine::{Decision, DecisionKind, GameState};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Réglages du niveau réfléchi.
#[derive(Clone, Copy, Debug)]
pub struct ReglagesReflechi {
    /// La recherche de chaque fil (B3 par défaut ; `iterations` et
    /// `duree_max` y sont ignorés : c'est le temps prévu qui compte).
    pub recherche: ReglagesB2,
    /// Nombre de fils ; 0 : autant que de cœurs logiques (12 sur la Surface).
    pub fils: usize,
    /// Temps de réflexion par option en plus de la première, pour une
    /// décision « normale » (multiplié par le poids du genre de décision).
    pub par_option: Duration,
    /// Temps minimal d'une décision à plusieurs options.
    pub duree_min: Duration,
    /// Temps maximal d'une décision.
    pub duree_max: Duration,
    /// Arrêter dès que le choix ne peut plus changer.
    pub arret_anticipe: bool,
    /// Banque de temps pour toute la partie (`None` : pas de banque, seul
    /// le temps prévu par décision compte). Voir
    /// [`ReglagesReflechi::temps_avec_banque`].
    pub banque: Option<Duration>,
}

impl Default for ReglagesReflechi {
    fn default() -> Self {
        ReglagesReflechi {
            recherche: ReglagesB2::b3(),
            fils: 0,
            par_option: Duration::from_secs(3),
            duree_min: Duration::from_millis(300),
            duree_max: Duration::from_secs(120),
            arret_anticipe: true,
            banque: None,
        }
    }
}

impl ReglagesReflechi {
    /// Lit le nom d'un niveau réfléchi : `reflechi` (2 minutes au plus par
    /// décision), `reflechi-30s`, `reflechi-5m`, `reflechi-500ms` (temps
    /// maximal par décision). Le temps par option suit le temps maximal
    /// (1/40e), pour qu'un niveau plus court reste proportionné.
    pub fn depuis_nom(nom: &str) -> Option<Self> {
        let mut r = ReglagesReflechi::default();
        let reste = match nom.strip_prefix("reflechi") {
            Some("") => return Some(r),
            Some(x) => x.strip_prefix('-')?,
            None => return None,
        };
        // `reflechi-partie-10m` : une banque de 10 minutes pour la partie.
        let (banque, reste) = match reste.strip_prefix("partie-") {
            Some(x) => (true, x),
            None => (false, reste),
        };
        let chiffres: String = reste.chars().take_while(|c| c.is_ascii_digit()).collect();
        let n: u64 = chiffres.parse().ok().filter(|&n| n > 0)?;
        let max = match &reste[chiffres.len()..] {
            "ms" => Duration::from_millis(n),
            "s" => Duration::from_secs(n),
            "m" | "min" => Duration::from_secs(60 * n),
            _ => return None,
        };
        if banque {
            r.banque = Some(max);
            return Some(r);
        }
        r.duree_max = max;
        r.par_option = max / 40;
        r.duree_min = r.duree_min.min(max);
        Some(r)
    }

    /// Poids d'un genre de décision dans le temps prévu : les décisions qui
    /// décident souvent d'une partie (jouer une carte ou passer, défendre,
    /// l'arsenal) comptent plus qu'un ordre de pitch ou de déclenchements.
    pub fn poids(genre: DecisionKind) -> f32 {
        match genre {
            DecisionKind::Priority | DecisionKind::Defend => 2.0,
            DecisionKind::Arsenal | DecisionKind::Pitch | DecisionKind::FirstPlayer => 1.0,
            DecisionKind::PitchOrder | DecisionKind::TriggerOrder | DecisionKind::TriggerFirst => {
                0.25
            }
            _ => 0.5,
        }
    }

    /// Temps prévu pour une décision de ce genre à `options` options
    /// distinctes (0 s s'il n'y en a qu'une).
    pub fn temps_prevu(&self, genre: DecisionKind, options: usize) -> Duration {
        if options <= 1 {
            return Duration::ZERO;
        }
        let t = self
            .par_option
            .mul_f32(Self::poids(genre) * (options - 1) as f32);
        t.clamp(self.duree_min, self.duree_max)
    }

    /// Temps prévu avec une banque où il reste `restant` : une part de la
    /// banque d'autant plus grande que la décision est complexe. L'unité est
    /// 1/150e de ce qui reste ; une décision vaut `poids × (options − 1) / 2`
    /// unités (6 options distinctes comptées au plus) : 1/75e de la banque
    /// pour une priorité à 3 options (8 s sur 10 min), 1/25e pour une
    /// priorité à 7 options ou plus (24 s), jamais plus que
    /// [`ReglagesReflechi::duree_max`]. La banque fond de décision en
    /// décision sans s'épuiser d'un coup, et l'arrêt anticipé en rend une
    /// partie. Banque vide : le temps minimal
    /// ([`ReglagesReflechi::duree_min`], 0,3 s, encore bien plus que les
    /// 300 itérations de B3) ; la partie peut alors dépasser un peu la banque.
    pub fn temps_avec_banque(
        &self,
        genre: DecisionKind,
        options: usize,
        restant: Duration,
    ) -> Duration {
        if options <= 1 {
            return Duration::ZERO;
        }
        let unites = Self::poids(genre) * (options - 1).min(6) as f32 / 2.0;
        let t = (restant / 150).mul_f32(unites);
        t.min(self.duree_max).max(self.duree_min)
    }

    fn nombre_de_fils(&self) -> usize {
        if self.fils > 0 {
            self.fils
        } else {
            std::thread::available_parallelism().map_or(1, |n| n.get())
        }
    }
}

/// Une option suivie pendant la réflexion.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptionSuivie {
    /// Index de l'option dans la décision.
    pub index: usize,
    /// Parties simulées passées par elle (tous fils confondus).
    pub visites: u64,
    /// Chances de gagner estimées en la choisissant (0 à 1).
    pub chances: f32,
}

/// L'avancement de la réflexion en cours (ou de la dernière).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progression {
    /// Le bot est-il en train de réfléchir ?
    pub en_cours: bool,
    /// Numéro de la décision (change à chaque nouvelle réflexion).
    pub numero: u64,
    pub ecoule: Duration,
    pub prevu: Duration,
    pub iterations: u64,
    pub fils: usize,
    /// Les options distinctes, de la plus visitée à la moins visitée.
    pub options: Vec<OptionSuivie>,
    /// La réflexion s'est arrêtée parce que le choix ne pouvait plus changer.
    pub choix_acquis: bool,
    /// Ce qu'il reste dans la banque de temps de la partie (`None` : pas de
    /// banque).
    pub banque: Option<Duration>,
}

/// Ce que l'interface partage avec le bot : l'avancement, et le bouton
/// « Joue maintenant ».
#[derive(Debug, Default)]
pub struct Suivi {
    /// Numéro de la réflexion à arrêter ; `u64::MAX` : toutes, jusqu'à
    /// [`Suivi::reprendre`].
    arret: AtomicU64,
    progression: Mutex<Progression>,
}

impl Suivi {
    /// Une copie de l'avancement.
    pub fn progression(&self) -> Progression {
        self.progression
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    /// Arrête la réflexion en cours : le bot joue le meilleur choix trouvé.
    /// Si aucune réflexion n'est en cours, arrête la **prochaine** (celle
    /// dont le fil vient d'être lancé mais n'a pas encore commencé) ; les
    /// suivantes ne sont pas touchées.
    pub fn jouer_maintenant(&self) {
        let p = self.progression();
        let cible = if p.en_cours { p.numero } else { p.numero + 1 };
        let _ = self.arret.fetch_max(cible, Ordering::Relaxed);
    }

    /// Arrête la réflexion en cours **et** celles qui commenceraient avant
    /// [`Suivi::reprendre`] (l'interface abandonne le coup du bot).
    pub fn arreter_tout(&self) {
        self.arret.store(u64::MAX, Ordering::Relaxed);
    }

    /// Annule [`Suivi::arreter_tout`].
    pub fn reprendre(&self) {
        self.arret.store(0, Ordering::Relaxed);
    }

    fn doit_arreter(&self, numero: u64) -> bool {
        let a = self.arret.load(Ordering::Relaxed);
        a == u64::MAX || a == numero
    }

    fn ecrire(&self, f: impl FnOnce(&mut Progression)) {
        f(&mut self.progression.lock().unwrap_or_else(|p| p.into_inner()));
    }
}

/// Le bot du niveau réfléchi.
pub struct ReflechiAgent {
    pub reglages: ReglagesReflechi,
    /// Une recherche par fil (gardée d'une décision à l'autre : son hasard
    /// continue, son évaluation reste rangée).
    recherches: Vec<IsmctsAgent>,
    suivi: Arc<Suivi>,
    numero: u64,
    /// Ce qu'il reste dans la banque de temps (`None` : pas de banque). Un
    /// bot est créé pour chaque partie : la banque repart à chaque manche.
    restant: Option<Duration>,
    /// Statistiques : itérations faites depuis la création du bot.
    pub iterations_faites: u64,
}

impl ReflechiAgent {
    pub fn new(graine: u64, reglages: ReglagesReflechi) -> Self {
        let n = reglages.nombre_de_fils();
        let recherches = (0..n as u64)
            .map(|k| {
                IsmctsAgent::avec(
                    graine ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15),
                    reglages.recherche,
                )
            })
            .collect();
        ReflechiAgent {
            reglages,
            recherches,
            suivi: Arc::new(Suivi::default()),
            numero: 0,
            restant: reglages.banque,
            iterations_faites: 0,
        }
    }

    /// Le suivi de la réflexion, à garder par l'interface.
    pub fn suivi(&self) -> Arc<Suivi> {
        Arc::clone(&self.suivi)
    }
}

/// Visites et résultats additionnés sur tous les fils, dans l'ordre des clés.
fn additionner(parts: &[Mutex<(Vec<StatOption>, u64)>], n: usize) -> (Vec<StatOption>, u64) {
    let mut total = vec![StatOption::default(); n];
    let mut iterations = 0;
    for p in parts {
        let p = p.lock().unwrap_or_else(|e| e.into_inner());
        iterations += p.1;
        for (t, s) in total.iter_mut().zip(&p.0) {
            t.visites += s.visites;
            t.gains += s.gains;
        }
    }
    (total, iterations)
}

impl Agent for ReflechiAgent {
    fn name(&self) -> &str {
        "B3-reflechi"
    }

    fn suivi(&self) -> Option<Arc<Suivi>> {
        Some(self.suivi())
    }

    fn choose(&mut self, state: &GameState, decision: &Decision) -> usize {
        if decision.options.len() <= 1 {
            return 0;
        }
        let cles = cles_des_options(state, decision);
        if cles.len() == 1 {
            return cles[0].1;
        }
        let restant = self.restant;
        let prevu = match restant {
            Some(r) => self
                .reglages
                .temps_avec_banque(decision.kind, cles.len(), r),
            None => self.reglages.temps_prevu(decision.kind, cles.len()),
        };
        let fils = self.recherches.len();
        self.numero += 1;
        let numero = self.numero;
        self.suivi.ecrire(|p| {
            *p = Progression {
                en_cours: true,
                numero,
                prevu,
                fils,
                banque: restant,
                ..Progression::default()
            }
        });
        let moi = decision.player;
        let debut = Instant::now();
        let stop = AtomicBool::new(false);
        let parts: Vec<Mutex<(Vec<StatOption>, u64)>> =
            (0..fils).map(|_| Mutex::new((Vec::new(), 0))).collect();
        let suivi = &self.suivi;
        let reglages = self.reglages;
        let mut acquis = false;
        std::thread::scope(|s| {
            for (r, part) in self.recherches.iter_mut().zip(&parts) {
                let (stop, cles) = (&stop, &cles);
                s.spawn(move || {
                    r.preparer(state);
                    let mut faites = 0u64;
                    // Toujours au moins un paquet : même arrêtée tout de
                    // suite, la recherche a quelque chose à proposer.
                    loop {
                        r.iterer(state, moi, 16);
                        faites += 16;
                        *part.lock().unwrap_or_else(|e| e.into_inner()) = (r.racine(cles), faites);
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                });
            }
            loop {
                std::thread::sleep(Duration::from_millis(50));
                let ecoule = debut.elapsed();
                let (total, iterations) = additionner(&parts, cles.len());
                let mut options: Vec<OptionSuivie> = total
                    .iter()
                    .zip(&cles)
                    .map(|(t, &(_, i))| OptionSuivie {
                        index: i,
                        visites: t.visites as u64,
                        chances: if t.visites > 0 {
                            t.gains / t.visites as f32
                        } else {
                            0.0
                        },
                    })
                    .collect();
                options.sort_by_key(|o| std::cmp::Reverse(o.visites));
                // Arrêt anticipé : l'avance de la première option est plus
                // grande que les itérations encore prévues.
                if reglages.arret_anticipe && ecoule >= reglages.duree_min && options.len() >= 2 {
                    let restant = prevu.saturating_sub(ecoule).as_secs_f64();
                    let vitesse = iterations as f64 / ecoule.as_secs_f64().max(1e-3);
                    let avance = options[0].visites.saturating_sub(options[1].visites);
                    if avance as f64 > vitesse * restant {
                        acquis = true;
                    }
                }
                let fini = acquis || ecoule >= prevu || suivi.doit_arreter(numero);
                suivi.ecrire(|p| {
                    p.ecoule = ecoule;
                    p.iterations = iterations;
                    p.options = options;
                    p.choix_acquis = acquis;
                    p.banque = restant.map(|r| r.saturating_sub(ecoule));
                });
                if fini {
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
            }
        });
        let (total, iterations) = additionner(&parts, cles.len());
        self.iterations_faites += iterations;
        suivi.ecrire(|p| {
            p.en_cours = false;
            p.ecoule = debut.elapsed();
            p.iterations = iterations;
        });
        let ecoule = debut.elapsed();
        self.restant = restant.map(|r| r.saturating_sub(ecoule));
        suivi.ecrire(|p| p.banque = self.restant);
        meilleure_option(&total, &cles).unwrap_or(cles[0].1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_nom_regle_le_temps_maximal() {
        let r = ReglagesReflechi::depuis_nom("reflechi").unwrap();
        assert_eq!(r.duree_max, Duration::from_secs(120));
        let r = ReglagesReflechi::depuis_nom("reflechi-30s").unwrap();
        assert_eq!(r.duree_max, Duration::from_secs(30));
        assert_eq!(r.par_option, Duration::from_millis(750));
        let r = ReglagesReflechi::depuis_nom("reflechi-5m").unwrap();
        assert_eq!(r.duree_max, Duration::from_secs(300));
        let r = ReglagesReflechi::depuis_nom("reflechi-200ms").unwrap();
        assert_eq!(r.duree_max, Duration::from_millis(200));
        assert_eq!(r.duree_min, Duration::from_millis(200));
        let r = ReglagesReflechi::depuis_nom("reflechi-partie-10m").unwrap();
        assert_eq!(r.banque, Some(Duration::from_secs(600)));
        assert_eq!(r.duree_max, Duration::from_secs(120));
        for faux in [
            "reflechi-",
            "reflechi-0s",
            "reflechi-3h",
            "reflechix",
            "b3",
            "reflechi-partie-",
            "reflechi-partie-0m",
        ] {
            assert!(ReglagesReflechi::depuis_nom(faux).is_none(), "{faux}");
        }
    }

    #[test]
    fn le_temps_prevu_suit_les_options_et_le_genre_de_decision() {
        let r = ReglagesReflechi::default();
        assert_eq!(r.temps_prevu(DecisionKind::Priority, 1), Duration::ZERO);
        // 2 options de priorité : 3 s × 2 × 1.
        assert_eq!(
            r.temps_prevu(DecisionKind::Priority, 2),
            Duration::from_secs(6)
        );
        // Un ordre de pitch à 2 options : 3 s × 0,25 = 0,75 s.
        assert_eq!(
            r.temps_prevu(DecisionKind::PitchOrder, 2),
            Duration::from_millis(750)
        );
        // Beaucoup d'options : plafonné à 2 minutes.
        assert_eq!(
            r.temps_prevu(DecisionKind::Defend, 40),
            Duration::from_secs(120)
        );
        assert!(r.temps_prevu(DecisionKind::Defend, 5) > r.temps_prevu(DecisionKind::Pitch, 5));
    }

    #[test]
    fn la_banque_se_repartit_selon_la_complexite_sans_s_epuiser() {
        let r = ReglagesReflechi::default();
        let dix = Duration::from_secs(600);
        // Une seule option : rien.
        assert_eq!(
            r.temps_avec_banque(DecisionKind::Priority, 1, dix),
            Duration::ZERO
        );
        // Priorité à 3 options : 1/75e de la banque (8 s sur 10 min).
        assert_eq!(
            r.temps_avec_banque(DecisionKind::Priority, 3, dix),
            Duration::from_secs(8)
        );
        // Priorité à beaucoup d'options : 1/25e (24 s), même au-delà de 7.
        assert_eq!(
            r.temps_avec_banque(DecisionKind::Priority, 30, dix),
            Duration::from_secs(24)
        );
        // Un ordre de pitch à 2 options : 1/1200e (0,5 s).
        assert_eq!(
            r.temps_avec_banque(DecisionKind::PitchOrder, 2, dix),
            Duration::from_millis(500)
        );
        // Jamais plus de 2 minutes.
        assert_eq!(
            r.temps_avec_banque(DecisionKind::Priority, 30, Duration::from_secs(7200)),
            Duration::from_secs(120)
        );
        // Banque vide : le temps minimal.
        assert_eq!(
            r.temps_avec_banque(DecisionKind::Priority, 3, Duration::ZERO),
            r.duree_min
        );
        // 60 grosses décisions d'affilée (sans arrêt anticipé) laissent
        // encore plus d'un vingtième de la banque (52 s sur 10 min).
        let mut reste = dix;
        for _ in 0..60 {
            reste = reste.saturating_sub(r.temps_avec_banque(DecisionKind::Priority, 7, reste));
        }
        assert!(reste > dix / 20, "{reste:?}");
    }

    #[test]
    fn joue_maintenant_vise_une_seule_reflexion() {
        let s = Suivi::default();
        // Avant la première réflexion : c'est elle qui s'arrêtera.
        s.jouer_maintenant();
        assert!(s.doit_arreter(1));
        assert!(!s.doit_arreter(2));
        s.arreter_tout();
        assert!(s.doit_arreter(7));
        s.reprendre();
        assert!(!s.doit_arreter(7));
    }
}
