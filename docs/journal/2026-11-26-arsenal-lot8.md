# 2026-11-26 — Arsenal, lot 8 : Entraîner

- Agent : Claude (Claude Code), branche `claude/project-thread-lcawsx`.
- Plan : `docs/rapatriement-arsenal.md`, lot 8.

## Fait
- `crates/ui/src/api_tournoi.rs` : tournoi en arrière-plan dans `fab-web`
  (mêmes parties que `fab-sim --tournoi` : phase de side, places et
  joueur qui choisit alternés), sur les decks choisis, decks maison
  compris. Les parties sont prises tour par tour (une par paire, puis la
  deuxième…) : la matrice se remplit partout à la fois et un arrêt donne
  des résultats équilibrés. Les numéros de graine diffèrent donc de
  `fab-sim` pour un même réglage.
- Banc SPRT : log-rapport de vraisemblance (approximation normale), bornes
  ±2,94 (5 % d'erreur), pas de conclusion avant 30 parties.
- Historique `sauvegardes/tournois/tournoi-<date>.json`, export JSON / CSV.
- Tâches : `fab-sim --apprendre-side` / `--apprendre-b3` lancés comme
  programmes à part, sortie affichée ; confirmation avant (le fichier de
  table ou de poids est réécrit).
- Volet `web/entrainer/`, tuile « Entraîner » de l'accueil activée.
- Tests : `crates/ui/tests/entrainer.rs` (tournoi complet B0 contre B0 sur
  2 decks, historique, SPRT qui conclut, réglages refusés).

## Mesuré
- Tournoi B0 contre B0, 9 decks, 1 partie par paire (81 parties) : moins
  d'une seconde dans la page.

## Pas repris
- Apprendre sur un deck maison : `fab-sim` n'apprend que sur les decks du
  projet et la table apprise est compilée dans le programme.
