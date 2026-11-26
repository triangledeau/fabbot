# 2026-11-26 — Arsenal, lot 5 (impression des proxies)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Source** : `proxies-de-rathe.html` de l'Arsenal JS (idée, format des
  planches) ; page réécrite, sans les données de cartes embarquées.

## Fait

- Page : volet « Imprimer » (`#impression`, `web/impression/`), lien depuis
  l'accueil. Deck du projet ou maison (`/api/deck`), images de la
  collection (`/api/collection`), 9 cartes de 63 × 88 mm par page A4 ou
  Letter, traits de coupe, espace réglable, héros et équipement en option ;
  `@page` à la taille du papier, sans marge, impression du navigateur.
- Pas repris : versions de cartes (foil, full art…), agrandissement
  d'image, fond perdu, versos, séparateurs, boîtes de deck.

## Tests

- Le test des fichiers servis couvre les nouveaux fichiers.
- Playwright : deck Ira (56 cartes) → 7 pages A4 au format PDF
  (595 × 842 points), aucune erreur JavaScript.
