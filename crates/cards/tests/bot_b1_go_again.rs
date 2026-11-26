//! B1 : go again qui n'est pas écrit sur la carte en main ou sur l'arme
//! (Zenith Blade aiguisée, combos d'Ira, Scar for a Scar), et zone du
//! Frostbite de Frost Spike. Mesures : `docs/analyses/2026-11-26-iyslander-hala-ira-b3.md`.

mod commun;
use commun::*;
use fab_ai::{Agent, HeuristicAgent};
use fab_engine::{Choice, DecisionKind, PlayerId, Zone};

/// Le choix de B1 pour la décision en cours.
fn choix_b1(t: &Partie) -> Choice {
    let d = t.g.pending_decision().unwrap().clone();
    let i = HeuristicAgent::new().choose(&t.g, &d);
    d.options[i]
}

/// Avance jusqu'à la prochaine priorité de P0 pendant sa phase d'action,
/// pile vide et chaîne fermée ou à l'étape de résolution (là où B1 choisit
/// sa prochaine action). P1 passe ou ne défend pas ; les autres décisions de
/// P0 pitchent un Essai Rempart si possible, sinon prennent la première option.
fn jusqu_a_ma_prochaine_action(t: &mut Partie) {
    use fab_engine::state::CombatStep;
    for _ in 0..100 {
        let d = t.g.pending_decision().unwrap().clone();
        let libre = t.g.stack.is_empty()
            && (t.g.combat.links.is_empty() || t.g.combat.step == Some(CombatStep::Resolution));
        if d.player == P0 && d.kind == DecisionKind::Priority && libre {
            return;
        }
        let c = if d.kind == DecisionKind::Priority || d.player == PlayerId(1) {
            d.index_of(Choice::Pass).unwrap_or(0)
        } else {
            // Pitch : de préférence une carte de remplissage.
            d.options
                .iter()
                .position(|o| matches!(o, Choice::Pitch(c) if t.g.def(*c).name == "Essai Rempart"))
                .unwrap_or(0)
        };
        t.g.apply(c).unwrap();
    }
    panic!("pas de nouvelle action pour P0");
}

#[test]
fn b1_attaque_avec_zenith_blade_aiguisee_avant_une_attaque_sans_go_again() {
    // Edict of Steel aiguise Zenith Blade : sa première attaque du tour a go
    // again. B1 doit attaquer avec l'épée avant Brutal Assault (6, sans go again).
    let mut t = partie_vraie(
        ["Hala", "Hala"],
        &[
            ("Edict of Steel", R),
            ("Brutal Assault", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Zenith Blade"],
        &[],
    );
    let zenith = t.equipement(P0, "Zenith Blade");
    let edict = t.en_main(P0, "Edict of Steel", R);
    t.choisir(Choice::Play(edict));
    jusqu_a_ma_prochaine_action(&mut t);
    assert!(t.g.sharpened_this_turn(zenith));
    assert!(matches!(choix_b1(&t), Choice::Activate(a, _) if a == zenith));
}

#[test]
fn b1_prefere_la_grosse_attaque_si_zenith_blade_n_est_pas_aiguisee() {
    // Sans aiguisage, pas de go again : Brutal Assault (6) passe avant l'épée (3).
    let t = partie_vraie(
        ["Hala", "Hala"],
        &[
            ("Brutal Assault", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Zenith Blade"],
        &[],
    );
    let brutal = t.en_main(P0, "Brutal Assault", R);
    assert_eq!(choix_b1(&t), Choice::Play(brutal));
}

#[test]
fn b1_voit_le_go_again_du_combo_de_seek_vengeance_apres_edge_of_autumn() {
    // Après une attaque d'Edge of Autumn, Seek Vengeance (4) a go again : B1
    // la joue avant Humble (6, sans go again).
    let mut t = partie_vraie(
        ["Ira, Crimson Haze", "Ira, Crimson Haze"],
        &[
            ("Seek Vengeance", R),
            ("Humble", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Edge of Autumn"],
        &[],
    );
    let edge = t.equipement(P0, "Edge of Autumn");
    t.choisir(Choice::Activate(edge, 0));
    jusqu_a_ma_prochaine_action(&mut t);
    let seek = t.en_main(P0, "Seek Vengeance", R);
    assert_eq!(choix_b1(&t), Choice::Play(seek));
}

#[test]
fn b1_prefere_humble_quand_le_combo_de_seek_vengeance_n_est_pas_rempli() {
    let t = partie_vraie(
        ["Ira, Crimson Haze", "Ira, Crimson Haze"],
        &[
            ("Seek Vengeance", R),
            ("Humble", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let humble = t.en_main(P0, "Humble", R);
    assert_eq!(choix_b1(&t), Choice::Play(humble));
}

#[test]
fn b1_met_le_frostbite_de_frost_spike_chez_l_adversaire() {
    // Les deux joueurs ont des zones exposées ; avant, B1 prenait la
    // première de la liste, la sienne.
    let mut t = partie_vraie(
        ["Iyslander", "Iyslander"],
        &[("Frost Spike", B)],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let spike = t.en_main(P0, "Frost Spike", B);
    t.choisir(Choice::Play(spike));
    for _ in 0..20 {
        if t.kind() == DecisionKind::Effect {
            break;
        }
        t.choisir(Choice::Pass);
    }
    assert_eq!(t.kind(), DecisionKind::Effect);
    assert!(t.options().contains(&Choice::Zone(P0, Zone::Head)));
    assert!(matches!(choix_b1(&t), Choice::Zone(p, _) if p == P1));
}

#[test]
fn b1_voit_le_go_again_de_scar_for_a_scar_quand_il_a_moins_de_vie() {
    // Moins de vie que l'adversaire : Scar for a Scar (4) aura go again, B1
    // la joue avant Humble (6, sans go again).
    let mut t = partie_vraie(
        ["Ira, Crimson Haze", "Ira, Crimson Haze"],
        &[
            ("Scar for a Scar", R),
            ("Humble", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    t.g.player_mut(P0).life = 10;
    let scar = t.en_main(P0, "Scar for a Scar", R);
    assert_eq!(choix_b1(&t), Choice::Play(scar));
}

#[test]
fn b1_prefere_humble_a_scar_for_a_scar_a_vie_egale() {
    let t = partie_vraie(
        ["Ira, Crimson Haze", "Ira, Crimson Haze"],
        &[
            ("Scar for a Scar", R),
            ("Humble", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &[],
        &[],
    );
    let humble = t.en_main(P0, "Humble", R);
    assert_eq!(choix_b1(&t), Choice::Play(humble));
}

#[test]
fn b1_detruit_pouncing_paws_quand_il_a_un_combo_du_tigre_en_main() {
    let t = partie_vraie(
        ["Ira, Crimson Haze", "Ira, Crimson Haze"],
        &[
            ("Pouncing Qi", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Pouncing Paws", "Tearing Shuko"],
        &[],
    );
    let paws = t.equipement(P0, "Pouncing Paws");
    assert!(matches!(choix_b1(&t), Choice::Activate(a, _) if a == paws));
}

#[test]
fn b1_garde_pouncing_paws_sans_combo_du_tigre() {
    let t = partie_vraie(
        ["Ira, Crimson Haze", "Ira, Crimson Haze"],
        &[
            ("Humble", R),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
            ("Essai Rempart", B),
        ],
        RIEN,
        RIEN,
        RIEN,
        &["Pouncing Paws"],
        &[],
    );
    let paws = t.equipement(P0, "Pouncing Paws");
    assert!(!matches!(choix_b1(&t), Choice::Activate(a, _) if a == paws));
}
