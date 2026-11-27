# Tâches en cours

Tableau de coordination entre les agents (sessions Claude, Antigravity…).
Règles : `AGENTS.md`, « Travailler à plusieurs ». Chaque agent ajoute sa
ligne en commençant (et la fait arriver sur `main` tout de suite), puis la
retire en finissant.

## En cours

| Début | Agent | Branche | Tâche | Zone touchée (crates, fichiers) |
|---|---|---|---|---|
| 2026-11-27 | Claude | `claude/project-thread-v8uk72` | Finir l'intégration d'Aurora : table de side refaite (fait, poussée), poids B3 à refaire avec 10 héros (`--apprendre-b3 --games 112 --bots b3-100 b3-100 --lambda 40`, ≈ 1 h), puis comparer aux anciens poids et fusionner | `data/side/b1.txt`, `data/b3/evaluation.txt`, `docs/bots.md` (chiffres) |
| 2026-11-26 | Claude | `claude/project-thread-0p5cpk` | Didacticiel « Apprendre » des règles (phase 2 du rapatriement) | `crates/ui/web/apprendre/` (nouveau), `crates/ui/src/api_apprendre.rs` (nouveau), quelques lignes dans `serveur.rs`, `index.html`, `accueil.js` |
| 2026-11-26 | Claude | `claude/vigilant-fermat-iq5cjs` | B2/B3 : estimation qui comprend le blood debt et les cartes bannies (Levia), mesurée contre B2 | `crates/ai/src/b2.rs`, `b3.rs` (recherche seulement), `docs/bots.md` |

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
