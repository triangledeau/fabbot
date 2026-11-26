# 2026-11-26 — Étape 7 : bot B3 et procédure « nouveau deck »

- Agent : Claude Code ; branche `claude/wizardly-hawking-xf43w4`.
- **B3** (`crates/ai/src/b3.rs`) : la recherche de B2 avec une estimation
  apprise. Évaluation par caractéristiques (vie, cartes, équipements, blood
  debt, héros, vie et main propres au héros, permanents carte par carte),
  σ(note₀ − note₁) ; régression logistique (Newton, ridge λ = 10) ; poids
  par nom de carte dans `data/b3/evaluation.txt`. Commandes :
  `fab-sim --apprendre-b3` (`--bots`, `--sortie`, `--lambda`), bots `b3`,
  `b3-…`, `b3:fichier`.
- Mesures contre B2 (81 paires, 0 erreur) : 1re génération (40 500
  parties B1 contre B1) +9 Elo (324 parties), +19 sans la force du héros
  dans la recherche, −24 avec 1 tour simulé et 800 itérations ; **2e
  génération (4 536 parties B3-100 contre B3-100) : 53,7 % sur 648
  parties, +26 Elo (−1 à +53), retenue**. Détail : `docs/bots.md`.
- **Nouveau deck** : liste unique des decks `fab_cards::decks` (fichiers
  `data/decks/NN_nom.txt`, utilisée par le simulateur, l'interface et les
  tests ; decks en cours dans `data/decks/en_cours/`, non lus) ; outil
  `verifier-deck` (format, cartes à coder, lignes de STATUS.md) ;
  `check_cards.py <fichier>` ; procédure `docs/nouveau-deck.md`.
- 12 nouveaux tests (794 au total), fmt et clippy sans avertissement.
