//! Identifiants compacts. Les cartes sont manipulées par des numéros, jamais
//! par du texte : copier un état de partie ne coûte presque rien.

/// Un joueur : 0 ou 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const P0: PlayerId = PlayerId(0);
    pub const P1: PlayerId = PlayerId(1);
    pub const BOTH: [PlayerId; 2] = [PlayerId::P0, PlayerId::P1];

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
/// index dans la base de cartes ([`crate::card::CardDb`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(pub u16);

/// Un emplacement d'objet dans la partie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub u32);

/// Référence à un objet **tel qu'il existe maintenant**.
///
/// CR 3.0.9 : quand un objet entre dans une zone hors de l'arène (sauf la
/// pile), il devient un nouvel objet sans lien avec le précédent. On le
/// modélise en augmentant sa « génération » : un effet qui visait l'ancienne
/// génération ne s'applique plus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjRef {
    pub id: ObjectId,
    pub gen: u32,
}

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
