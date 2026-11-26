# Tâches en cours

Tableau de coordination entre les agents (sessions Claude, Antigravity…).
Règles : `AGENTS.md`, « Travailler à plusieurs ». Chaque agent ajoute sa
ligne en commençant (et la fait arriver sur `main` tout de suite), puis la
retire en finissant.

## En cours

| Début | Agent | Branche | Tâche | Zone touchée (crates, fichiers) |
|---|---|---|---|---|
| — | — | — | (aucune) | — |

## Découpage conseillé des prochaines tâches

Tâches qui peuvent avancer **en parallèle** sans se gêner (zones
différentes). La liste détaillée est dans `docs/suivi.md`, « Prochaine
tâche ».

| Tâche | Zone principale | Peut tourner en même temps que |
|---|---|---|
| B3 : évaluation apprise par auto-jeu | `crates/ai/` (nouveau module), `docs/bots.md` | procédure « nouveau deck », corrections de cartes |
| Procédure « nouveau deck » (10e deck) | `docs/`, `crates/cards/src/real/`, `data/decks/` | B3 |
| Corrections de cartes signalées par le propriétaire | `crates/cards/src/real/<classe>.rs`, `crates/cards/tests/` | tout, sauf un autre agent sur le même deck |
| Améliorations de B2 (Levia, vitesse) | `crates/ai/src/b2.rs`, `determinisation.rs` | procédure « nouveau deck » ; **pas** avec B3 si B3 modifie la même évaluation |
| Phase de side apprise avec B2 | `crates/ai/src/side.rs`, `data/side/` | tout ce qui ne touche pas `data/side/` |
| Interface web | `crates/ui/` | tout |
