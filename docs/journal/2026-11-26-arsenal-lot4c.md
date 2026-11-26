# 2026-11-26 — Arsenal, lot 4c (ajustement avant la manche)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Source** : bloc `openAdjust` d'`editeur.js` de l'Arsenal JS (piles,
  sources de départ), réécrit pour l'écran de préparation de `fab-web`.

## Fait

- Serveur (`match_web.rs`) : la préparation donne `conseil` (arène et deck
  choisis par la table apprise `fab_ai::side` pour le card-pool de
  l'humain contre le héros du bot). Il ne voit que ce que l'humain voit.
- Page (`partie/partie.js`, `partie.css`) : deck en piles (clic +1, clic
  droit −1, clavier Entrée / Suppr), compteur « il en manque N » ; boutons
  de départ « Dernier réglage » (gardé par le navigateur pour chaque paire
  de decks, départ par défaut), « Plan conseillé », « Proposition », aussi
  pour l'équipement.

## Tests

- 1 nouveau test (`tests/web.rs`) : le plan conseillé est un choix complet
  (40 cartes, exemplaires dans les limites) que le serveur accepte.
- Playwright : plan conseillé pour l'équipement et le deck, une pile
  retirée puis remise, validation, dernier réglage gardé ; aucune erreur
  JavaScript.
