# 2026-11-26 — Ergonomie d'Arsenal (revue des 46 points)

- Agent : Claude (session projet « Améliorer l'ergonomie d'Arsenal »)
- Branche : `claude/project-thread-4sgqn2` (PR #37, puis lot 2)
- Revue : https://claude.ai/artifact/Dga7oQMursMjqAmwxksNqR (46 points, faite
  en lançant l'ancien Arsenal JS et fab-web, captures à l'appui).

## Partage avec le fil de reprise de l'interface

Le fil « Rapatrier les fonctionnalités dans Arsenal » réécrit `partie/*`,
`decks/*`, la préparation de manche et l'éditeur : il intègre les tableaux
Partie, Préparation, Éditeur, Jouer et Fin de partie de la revue, ainsi
que les points transverses qui vivent dans ces écrans (tactile, Échap,
contraste des zones, cartes sans image, couleur écrite). Il a corrigé le
bug « Niveau du bot » vide (PR #35).

## Fait ici

- Lot 1 (PR #37) : plus de numéros internes « [34] » dans la page (l'interface
  texte les garde : `vue::montrer_numeros`) ; `{p}` brut → « puissance » ;
  `Symboles.html` / `Symboles.texte` (`commun/cartes.js`) pour {r} {p} {d}
  {h}/{g} {i} {t} {u} {c}, sens vérifié dans `public_card_data.csv` ; focus
  clavier visible partout ; `/api/decks` renvoie `noms` (héros de chaque deck).
- Lot 2 : Entraîner en mots de joueur (niveaux B0–B3 expliqués, sides
  nommés, graine / parties en même temps / test SPRT sous « Réglages
  avancés », decks en puces au nom du héros, durée estimée avant de lancer
  d'après la vitesse du dernier tournoi des mêmes bots, parties en même temps
  = cœurs − 1 par défaut, message actionnable si fab-sim manque) ;
  Impression (noms des héros, clic = « déjà possédée » : grisée, sur des
  feuilles à part jamais imprimées, bouton « Tout réimprimer ») ; Accueil
  (compteurs d'Entraîner sans jargon, message des textes officiels qui dit
  quoi faire, héros dans le bandeau « Match en cours »).

## Pas fait ici

- « Les 40 du dernier réglage » à l'impression : le réglage est gardé par
  la page de partie (en réécriture) sous forme d'indices ; à reprendre
  quand l'écran de préparation sera stable.
- Portraits dans les puces de decks d'Entraîner (il faudrait l'image du
  héros dans `/api/decks`).
- Cache local des images (hors ligne).
- `{x}`, `{v}`, `{^}` du fichier LSS : sens non vérifié, laissés bruts
  (TODO règles dans `commun/cartes.js`).
