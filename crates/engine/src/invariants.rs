//! Contrôles de cohérence de l'état de partie, utilisés par les tests et les
//! simulations massives (« fuzzing ») : si l'un d'eux échoue, le moteur a un bug.

use crate::ids::{ObjectId, PlayerId};
use crate::state::GameState;
use crate::zones::Zone;

impl GameState {
    /// Vérifie la cohérence de l'état. Renvoie la liste des problèmes.
    pub fn check_invariants(&self) -> Vec<String> {
        let mut errs = Vec::new();
        let n = self.objects.len();
        // Combien de fois chaque objet apparaît dans un conteneur.
        let mut seen = vec![0u8; n];
        let mut mark = |id: ObjectId, where_: &str, expected: Zone, errs: &mut Vec<String>| {
            seen[id.0 as usize] += 1;
            let z = self.obj(id).zone;
            if z != expected && !(expected.in_arena() && z == Zone::CombatChain) {
                errs.push(format!(
                    "objet {} listé en {where_} mais sa zone est {z:?}",
                    id.0
                ));
            }
        };
        for p in PlayerId::BOTH {
            let ps = self.player(p);
            for &c in &ps.deck {
                mark(c, "deck", Zone::Deck, &mut errs);
            }
            for &c in &ps.hand {
                mark(c, "main", Zone::Hand, &mut errs);
            }
            for &c in &ps.pitch {
                mark(c, "pitch", Zone::Pitch, &mut errs);
            }
            for &c in &ps.graveyard {
                mark(c, "cimetière", Zone::Graveyard, &mut errs);
            }
            for &c in &ps.banished {
                mark(c, "banni", Zone::Banished, &mut errs);
            }
            if let Some(c) = ps.arsenal {
                mark(c, "arsenal", Zone::Arsenal, &mut errs);
            }
            for &c in &ps.weapons {
                mark(c, "arme", Zone::Weapon, &mut errs);
            }
            for (c, z) in [
                (ps.head, Zone::Head),
                (ps.chest, Zone::Chest),
                (ps.arms, Zone::Arms),
                (ps.legs, Zone::Legs),
            ] {
                if let Some(c) = c {
                    mark(c, "équipement", z, &mut errs);
                }
            }
            mark(ps.hero, "héros", Zone::Hero, &mut errs);
            if ps.life < 0 {
                errs.push(format!("vie négative pour {p:?}"));
            }
            if p != self.turn_player && ps.action_points > 0 {
                errs.push(format!("{p:?} a des points d'action hors de son tour"));
            }
            if ps.hand.len() > 60 {
                errs.push("main anormalement grande".into());
            }
        }
        // Objets sur la pile ou la chaîne de combat : pas dans un conteneur de joueur.
        for (i, o) in self.objects.iter().enumerate() {
            let id = ObjectId(i as u32);
            let equipped = self.player(o.owner).weapons.contains(&id)
                || self.player(o.owner).equipment().any(|e| e == id);
            match o.zone {
                Zone::Stack | Zone::CombatChain if !equipped => {
                    if seen[i] != 0 {
                        errs.push(format!(
                            "objet {i} sur pile/chaîne mais aussi listé ailleurs"
                        ));
                    }
                    seen[i] = 1;
                }
                Zone::Gone | Zone::Permanent => seen[i] = 1,
                _ => {}
            }
            if seen[i] != 1 {
                errs.push(format!("objet {i} ({:?}) présent {} fois", o.zone, seen[i]));
            }
        }
        // Chaque objet en zone pile est soit une couche, soit dans la file d'attente.
        for (i, o) in self.objects.iter().enumerate() {
            if o.zone == Zone::Stack {
                let id = ObjectId(i as u32);
                let on_stack = self.stack.iter().any(|l| l.object.id == id)
                    || self.combat.queue.iter().any(|(a, _)| a.id == id);
                if !on_stack {
                    errs.push(format!("objet {i} en zone pile sans couche"));
                }
            }
            if o.zone == Zone::CombatChain && !self.combat.open {
                errs.push(format!("objet {i} sur la chaîne alors qu'elle est fermée"));
            }
        }
        errs
    }
}
