# 2026-11-26 — Arsenal, lot 1 (images des cartes)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Plan** : `docs/rapatriement-arsenal.md`.

## Fait

- `tools/images_cartes.py` : associe chaque carte codée (`card-dump`) à son
  impression dans le fichier officiel LSS ; choisit la VF normale (bord noir),
  sinon l'anglaise hors Unlimited, une impression Silver Age d'abord. Écrit
  `data/cards/images.tsv` (versionné : codes et noms seulement).
- Résultat : **396 cartes, 343 en VF, 53 en anglais, 0 sans image** ; les
  396 adresses répondent (vérifié une fois, code HTTP 200).
- `crates/ui/src/images.rs` : table intégrée au programme ; JSON des cartes
  avec `image`, `nom_fr`, `puissance_imprimee`, `defense_imprimee`, et
  `heros_image` par joueur. Les cartes cachées restent `{"cache": true}`.
- Page : `commun/cartes.js` ; cartes en image (cadre dessiné si l'image ne
  vient pas), pastilles pour les valeurs changées et les compteurs, grande
  image et nom français dans la fiche, portrait du héros, volet Jouer de
  l'accueil illustré par votre héros.

## Tests

- 4 tests unitaires (`images.rs`) et 2 tests (`tests/web.rs`) : toutes les
  cartes des 9 decks ont une image ; l'état donne l'image des cartes visibles
  et rien des cartes cachées.
- Playwright : 46 cartes en image sur le plateau, aucune erreur JavaScript.

## Note

Le fichier officiel n'est pas téléchargeable depuis le conteneur cloud
(403) : il a été copié depuis le PC du propriétaire
(`data/cards/sources/public_card_data.csv`).
