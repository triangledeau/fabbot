//! Les zones de jeu.
//!
//! TODO(étape 1) : vérifier la liste et la terminologie exactes contre le
//! Comprehensive Rules (section « Zones ») et y ajouter les références.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Zone {
    Deck,
    Hand,
    Arsenal,
    Pitch,
    Graveyard,
    Banished,
    HeroZone,
    WeaponZone,
    /// Zones d'équipement (tête, torse, bras, jambes).
    EquipmentZone,
    /// La chaîne de combat.
    CombatChain,
    /// La pile des couches (« stack »).
    Stack,
    /// Zone permanente générique (tokens, auras, objets...).
    Permanent,
}
