//! Les zones de jeu (CR 3).

/// Zone où se trouve un objet. Les zones « par joueur » (main, deck...) sont
/// celles du propriétaire de l'objet (CR 3.0.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Zone {
    Deck,
    Hand,
    Arsenal,
    Pitch,
    Graveyard,
    Banished,
    Hero,
    Weapon,
    Head,
    Chest,
    Arms,
    Legs,
    /// La chaîne de combat (CR 3.6).
    CombatChain,
    /// La pile des couches, file d'attente des attaques comprise (CR 3.15).
    Stack,
    /// Zone permanente partagée (CR 3.13).
    Permanent,
    /// L'objet a cessé d'exister (token sorti du jeu, attaque-proxy terminée...).
    Gone,
}

impl Zone {
    /// L'arène (CR 3.1.1).
    pub fn in_arena(self) -> bool {
        matches!(
            self,
            Zone::Hero
                | Zone::Weapon
                | Zone::Head
                | Zone::Chest
                | Zone::Arms
                | Zone::Legs
                | Zone::CombatChain
                | Zone::Permanent
        )
    }
}
