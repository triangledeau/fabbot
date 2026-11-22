//! Tests de l'interface texte : parties complètes, annulation, information cachée.

use fab_ai::{Agent, RandomAgent};
use fab_engine::{DecisionKind, Event, GameState, ObjectId, PlayerId, Zone};
use fab_ui::jeu::{Fin, Interface};
use fab_ui::{nouvelle_partie, vue};
use std::io::Cursor;

const P0: PlayerId = PlayerId::P0;

/// Joue avec des réponses tapées à l'avance ; renvoie la fin et le texte affiché.
fn jouer(g: &mut GameState, reponses: &str, graine_bot: u64) -> (Fin, String) {
    let mut bot = RandomAgent::new(graine_bot);
    let mut ui = Interface {
        humain: P0,
        bot: &mut bot,
        textes: None,
    };
    let mut entree = Cursor::new(reponses.to_string());
    let mut sortie = Vec::new();
    let fin = ui.jouer(g, &mut entree, &mut sortie).expect("pas d'erreur");
    (fin, String::from_utf8(sortie).unwrap())
}

/// Les cartes que P0 n'a pas le droit de voir.
fn cachees_pour_p0(g: &GameState) -> Vec<ObjectId> {
    (0..g.objects.len() as u32)
        .map(ObjectId)
        .filter(|&id| !vue::visible(g, P0, id))
        .collect()
}

fn ne_montre_pas(texte: &str, cachees: &[ObjectId], contexte: &str) {
    for id in cachees {
        assert!(
            !texte.contains(&format!("[{}]", id.0)),
            "{contexte} : la carte cachée [{}] est affichée :\n{texte}",
            id.0
        );
    }
}

#[test]
fn une_partie_complete_se_joue_en_tapant_toujours_1() {
    for (moi, bot) in [
        ("hala", "dorinthea"),
        ("dorinthea", "hala"),
        ("ira", "hala"),
    ] {
        for graine in 1..=3 {
            let mut g = nouvelle_partie(moi, bot, graine).unwrap();
            let (fin, texte) = jouer(&mut g, &"1\n".repeat(50_000), graine);
            assert_eq!(fin, Fin::Terminee, "{moi} contre {bot}, graine {graine}");
            assert!(g.is_over());
            assert!(
                texte.contains("Partie") || texte.contains("Victoire") || texte.contains("Défaite")
            );
        }
    }
}

#[test]
fn une_partie_complete_avec_choix_varies() {
    // Réponses « 2, 1, 3, 1, 2… » : les numéros trop grands sont refusés
    // et redemandés, ce qui teste aussi les réponses invalides.
    let mut g = nouvelle_partie("hala", "dorinthea", 11).unwrap();
    let reponses: String = (0..200_000)
        .map(|i| format!("{}\n", 1 + (i * 7) % 3))
        .collect();
    let (fin, _) = jouer(&mut g, &reponses, 5);
    assert_eq!(fin, Fin::Terminee);
}

#[test]
fn annuler_revient_a_la_decision_precedente() {
    let mut g = nouvelle_partie("hala", "dorinthea", 3).unwrap();
    // Avancer jusqu'à la première décision de l'humain.
    let mut bot = RandomAgent::new(1);
    while g.pending_decision().unwrap().player != P0 {
        let d = g.pending_decision().unwrap().clone();
        let i = bot.choose(&g, &d);
        g.apply(i).unwrap();
    }
    let avant = g.pending_decision().unwrap().clone();
    let main_avant = g.player(P0).hand.clone();
    let (fin, texte) = jouer(&mut g, "1\na\nq\n", 2);
    assert_eq!(fin, Fin::Abandon);
    assert!(texte.contains("Choix annulé."));
    assert_eq!(g.pending_decision(), Some(&avant));
    assert_eq!(g.player(P0).hand, main_avant);
}

#[test]
fn annuler_sans_choix_precedent_ne_fait_rien() {
    let mut g = nouvelle_partie("hala", "dorinthea", 3).unwrap();
    let (fin, texte) = jouer(&mut g, "a\nq\n", 2);
    assert_eq!(fin, Fin::Abandon);
    assert!(texte.contains("Rien à annuler."));
}

#[test]
fn reponse_hors_des_options_est_redemandee() {
    let mut g = nouvelle_partie("hala", "dorinthea", 3).unwrap();
    let (_, texte) = jouer(&mut g, "99\nbonjour\nq\n", 2);
    assert_eq!(texte.matches("Réponse non comprise").count(), 2);
}

#[test]
fn la_commande_t_refuse_une_carte_cachee() {
    let mut g = nouvelle_partie("hala", "dorinthea", 3).unwrap();
    let cachee = g.player(PlayerId::P1).deck[0];
    let (_, texte) = jouer(&mut g, &format!("t {}\nq\n", cachee.0), 2);
    assert!(texte.contains("Cette carte est cachée pour vous."));
}

#[test]
fn la_commande_t_montre_une_carte_de_sa_main() {
    let mut g = nouvelle_partie("hala", "dorinthea", 3).unwrap();
    let mut bot = RandomAgent::new(1);
    while g.player(P0).hand.is_empty() || g.pending_decision().unwrap().player != P0 {
        let d = g.pending_decision().unwrap().clone();
        let i = bot.choose(&g, &d);
        g.apply(i).unwrap();
    }
    let carte = g.player(P0).hand[0];
    let nom = g.def(carte).name.clone();
    let (_, texte) = jouer(&mut g, &format!("t {}\nq\n", carte.0), 2);
    assert!(texte.contains(&nom));
}

/// Parties aléatoires : à chaque décision, ni l'état affiché à l'humain,
/// ni ses options, ni le récit des événements ne montrent une carte cachée
/// (main et arsenal adverses, decks).
#[test]
fn l_humain_ne_voit_jamais_les_cartes_cachees() {
    for graine in 1..=40u64 {
        let mut g = nouvelle_partie("hala", "dorinthea", graine).unwrap();
        let mut a = RandomAgent::new(graine);
        let mut b = RandomAgent::new(graine + 1000);
        while let Some(d) = g.pending_decision().cloned() {
            let cachees = cachees_pour_p0(&g);
            ne_montre_pas(&vue::etat(&g, P0, true), &cachees, "état");
            // Toutes les décisions sont décrites (affichage de chaque option),
            // et chaque question de carte a son libellé.
            let texte = vue::decision(&g, &d);
            if d.kind == DecisionKind::Effect {
                assert!(!texte.contains("Choix demandé par une capacité"), "{texte}");
            }
            if d.player == P0 {
                ne_montre_pas(&texte, &cachees, "décision");
            }
            let i = if d.player == P0 {
                a.choose(&g, &d)
            } else {
                b.choose(&g, &d)
            };
            g.apply(i).unwrap();
            // Une carte peut devenir publique puis cachée dans la même
            // décision (pitchée puis remise sous le deck) : pour les
            // événements, on vérifie la pioche adverse.
            let evenements = std::mem::take(g.record.as_mut().unwrap());
            for e in &evenements {
                if let Event::Drew { player, card } = *e {
                    let l = vue::evenement(&g, P0, e).unwrap_or_default();
                    if player != P0 {
                        ne_montre_pas(&l, &[card], "pioche adverse");
                        assert!(!l.contains(&g.def(card).name), "{l}");
                    }
                }
            }
        }
    }
}

#[test]
fn la_main_adverse_n_affiche_que_le_nombre_de_cartes() {
    let mut g = nouvelle_partie("hala", "dorinthea", 3).unwrap();
    // Faire piocher les mains de départ.
    let mut bot = RandomAgent::new(1);
    while g.player(PlayerId::P1).hand.is_empty() {
        let d = g.pending_decision().unwrap().clone();
        let i = bot.choose(&g, &d);
        g.apply(i).unwrap();
    }
    let n = g.player(PlayerId::P1).hand.len();
    let etat = vue::etat(&g, P0, true);
    assert!(etat.contains(&format!("Main : {n} carte(s)")));
    assert!(g
        .player(PlayerId::P1)
        .hand
        .iter()
        .all(|&c| g.obj(c).zone == Zone::Hand && !vue::visible(&g, P0, c)));
}

#[test]
fn une_carte_bannie_face_cachee_est_visible_par_son_proprietaire_seulement() {
    // CR 3.0.3a : un joueur peut regarder un objet privé qu'il possède, sauf
    // dans le deck.
    let mut g = nouvelle_partie("hala", "dorinthea", 1).unwrap();
    while g.player(P0).hand.is_empty() {
        let d = g.pending_decision().unwrap().clone();
        let i = RandomAgent::new(1).choose(&g, &d);
        g.apply(i).unwrap();
    }
    let c = g.player(P0).hand[0];
    g.banish(c, true);
    assert_eq!(g.obj(c).zone, Zone::Banished);
    assert!(vue::visible(&g, P0, c));
    assert!(!vue::visible(&g, PlayerId::P1, c));
    let haut = g.top_of_deck(P0).unwrap();
    assert!(!vue::visible(&g, P0, haut), "pas le deck");
}
