# 2026-11-26 — B3 : 3e génération et contenu de la main

- **Agent :** Claude Code
- **Branche :** `claude/loving-hamilton-itnnhv`

## Travail fait

- **Nouvelles caractéristiques de l'évaluation de B3** (`crates/ai/src/b3.rs`),
  sur le contenu de la main + arsenal (valeurs imprimées) : nombre
  d'attaques, puissance totale des attaques, défense totale, réactions de
  défense, ressources (pitch), et chaque carte **carte par carte**
  (`en main <carte> = poids` dans le fichier des poids).
- `--apprendre-b3` : essaie trois pénalités (λ/4, λ, 4λ) et garde la
  meilleure sur les parties de test ; apprend aussi, sur les mêmes
  parties, des poids **sans** le contenu de la main (`…_sans_main.txt`)
  pour comparer.
- **3e génération** : 9 072 parties B3-100 (poids de 2e génération) contre
  B3-100, 153 000 positions, 42 min sur 4 cœurs. Poids dans
  `data/b3/evaluation.txt` (utilisés par `b3`) ; l'ancien fichier est
  gardé dans `data/b3/generation2.txt`.
- 3 tests ajoutés (`crates/cards/tests/bot_b3.rs`).

## Chiffres mesurés (81 paires de decks, side appris, 0 erreur)

| Poids | Perte (test) | Contre | Parties | Score | Elo (95 %) |
|---|---|---|---|---|---|
| 2e génération (mesure refaite) | — | B2 | 1 296 | 56,6 % | +46 (+27 à +65) |
| 3e génération sans la main | 0,377 | B2 | 1 296 | 57,6 % | +53 (+34 à +73) |
| 3e génération avec la main (retenue) | 0,372 | B2 | 1 296 | 57,6 % | +53 (+34 à +73) |
| 3e génération avec la main | | B3 2e génération | 2 592 | 52,0 % | +14 (+1 à +28) |

Conclusion : petit gain de la 3e génération ; le contenu de la main
améliore un peu les prédictions mais pas le score mesuré. Les gains
diminuent d'une génération à l'autre. B3 reste très faible avec Levia
(12 % contre B2) et Iyslander (28 %).
