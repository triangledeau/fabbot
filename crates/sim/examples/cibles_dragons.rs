//! Mesure : quand l'adversaire de Dromai peut attaquer un dragon, que fait le bot ?
use fab_ai::side::creer_side;
use fab_ai::{creer_agent, Agent};
use fab_cards::{decks, real};
use fab_engine::{
    Choice, Decision, DecisionKind, FormatConfig, GameState, Outcome, PlayerId, Subtypes, Target,
};
use fab_match::{lancer_manche, preparer_manche};

#[derive(Default, Debug)]
struct Compte {
    decisions_cible: u64,
    vise_heros: u64,
    vise_dragon: u64,
    autre: u64,
}

struct Espion {
    bot: Box<dyn Agent + Send>,
    c: Compte,
}
impl Agent for Espion {
    fn name(&self) -> &str {
        "espion"
    }
    fn choose(&mut self, g: &GameState, d: &Decision) -> usize {
        let i = self.bot.choose(g, d);
        if d.kind == DecisionKind::AttackTarget {
            let dragon = |c: &Choice| matches!(c, Choice::Target(Target::Object(r)) if g.def(r.id).subtypes.has(Subtypes::DRAGON));
            if d.options.iter().any(dragon) {
                self.c.decisions_cible += 1;
                match d.options[i] {
                    Choice::Target(Target::Player(_)) => self.c.vise_heros += 1,
                    ref c if dragon(c) => self.c.vise_dragon += 1,
                    _ => self.c.autre += 1,
                }
            }
        }
        i
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let bot = a.get(1).cloned().unwrap_or("b1".into());
    let adv = a.get(2).cloned().unwrap_or("ira".into());
    let n: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(200);
    let db = std::sync::Arc::new(real::build());
    let pool = |nom: &str| real::card_pool(&db, &decks::liste(nom).unwrap()).unwrap();
    let (pa, pd) = (pool(&adv), pool("dromai"));
    let mut total = Compte::default();
    let mut victoires_dromai = 0;
    for seed in 0..n {
        let mut s0 = creer_side("liste", &db, None, seed).unwrap();
        let mut s1 = creer_side("liste", &db, None, seed ^ 1).unwrap();
        let taille = FormatConfig::silver_age().deck_size as usize;
        let chooser = PlayerId((seed % 2) as u8);
        let m = preparer_manche(&db, [&pa, &pd], [&mut *s0, &mut *s1], chooser, taille).unwrap();
        let mut g = lancer_manche(db.clone(), &m, seed, 200).unwrap();
        let mut e = Espion {
            bot: creer_agent(&bot, seed ^ 7).unwrap(),
            c: Compte::default(),
        };
        let mut dr = creer_agent(&bot, seed ^ 9).unwrap();
        fab_ai::play_out(&mut g, [&mut e, &mut *dr], false).unwrap();
        if g.outcome == Some(Outcome::Winner(PlayerId(1))) {
            victoires_dromai += 1;
        }
        total.decisions_cible += e.c.decisions_cible;
        total.vise_heros += e.c.vise_heros;
        total.vise_dragon += e.c.vise_dragon;
        total.autre += e.c.autre;
    }
    println!("{bot} ({adv}) contre {bot} (dromai), {n} parties : Dromai gagne {victoires_dromai}");
    println!("{total:?}");
}
