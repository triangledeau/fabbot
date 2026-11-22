//! Historique pour l'annulation illimitée.
//!
//! À chaque choix de l'humain, on garde une copie de l'état *avant* le choix.
//! Annuler = recharger la copie précédente, autant de fois que voulu.

#[derive(Clone, Debug)]
pub struct History<S: Clone> {
    snapshots: Vec<S>,
}

impl<S: Clone> Default for History<S> {
    fn default() -> Self {
        History {
            snapshots: Vec::new(),
        }
    }
}

impl<S: Clone> History<S> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mémorise l'état courant, juste avant que l'humain fasse un choix.
    pub fn record(&mut self, state: &S) {
        self.snapshots.push(state.clone());
    }

    /// Revient à l'état mémorisé le plus récent. `None` si rien à annuler.
    pub fn undo(&mut self) -> Option<S> {
        self.snapshots.pop()
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annuler_plusieurs_fois_jusqu_au_debut() {
        let mut h = History::new();
        let mut etat = 0;
        for _ in 0..5 {
            h.record(&etat);
            etat += 1;
        }
        assert_eq!(etat, 5);
        for attendu in (0..5).rev() {
            etat = h.undo().unwrap();
            assert_eq!(etat, attendu);
        }
        assert!(h.undo().is_none());
    }
}
