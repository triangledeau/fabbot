//! Identifiants compacts. Les cartes sont manipulées par des numéros, jamais
//! par du texte : copier un état de partie ne coûte presque rien.

/// Un joueur : 0 ou 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const P0: PlayerId = PlayerId(0);
    pub const P1: PlayerId = PlayerId(1);

    /// L'adversaire de ce joueur.
    #[inline]
    pub fn opponent(self) -> PlayerId {
        PlayerId(1 - self.0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Une *définition* de carte (ex. : « telle carte, version rouge »),
/// index dans la base de cartes (`fab-cards`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(pub u16);

/// Un *exemplaire* physique dans une partie. Deux copies de la même carte
/// ont le même [`CardId`] mais des `ObjectId` différents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adversaire_de_l_adversaire_est_soi_meme() {
        assert_eq!(PlayerId::P0.opponent(), PlayerId::P1);
        assert_eq!(PlayerId::P1.opponent(), PlayerId::P0);
        assert_eq!(PlayerId::P0.opponent().opponent(), PlayerId::P0);
    }
}
