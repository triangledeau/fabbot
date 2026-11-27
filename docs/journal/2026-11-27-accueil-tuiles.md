# 2026-11-27 — Accueil : tuiles simplifiées

- Agent : Claude Code, branche `claude/project-thread-e1u5xk`.
- Tuile Entraîner : seule l'illustration de Bravo, en pleine largeur (plus de
  partage en diagonale ni de rond « VS »). La page Entraîner garde son propre
  « VS » (styles `split1`/`split2`/`.vs` conservés).
- Tuiles Jouer et Apprendre : cartouches « Decks / Bots » et « Pour débuter
  de zéro » retirés.
- Bandeau « Match en cours » déplacé dans la tuile Jouer (cartouche du bas).
  La tuile Jouer devient un `div` cliquable (vers `#decks`) pour ne pas
  mettre de lien dans un lien ; Reprendre et Nouveau match gardent leur lien.
- Cartouches : marge basse fixée à 14px, égale aux marges gauche/droite
  (mesuré 15/15/15 px à 720 et 950 px de haut).
- Vérifié dans Chromium (captures 1500×720 et 1500×950, aucun message
  d'erreur JS, clic tuile → `#decks`, clic Reprendre → `#partie`).
