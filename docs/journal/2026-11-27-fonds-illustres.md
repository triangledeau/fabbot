# 2026-11-27 — Fonds illustrés d'Arsenal

- Agent : Claude (Claude Code), branche `claude/project-thread-9bjzi4`.
- Demande d'Antoine : des illustrations de cartes en haute définition,
  représentant des lieux sans personnage, fondues dans le fond sombre, tirées
  au hasard à l'ouverture et à chaque changement de page.

## Travail fait
- `crates/ui/web/commun/fonds.js` (nouveau) : 15 illustrations de lieux
  (ArtStation, publiées par les artistes ; 1 300 à 2 500 px de large contre
  546 px pour les scans officiels). Noms de carte et artistes vérifiés dans
  le fichier officiel LSS (sauf « Seat of the Emperor », illustration
  officielle qui n'est pas une carte). Tirage par `crypto.getRandomValues`,
  jamais deux fois la même de suite, fondu enchaîné à chaque changement
  d'adresse (#…). Légende « Illustration : carte · artiste » en bas à droite.
- `commun/arsenal.css` : règles `#fond` (image assombrie, vignette et dégradé
  haut/bas vers `--bg`) ; plus discret sur l'écran de partie ; masqué à
  l'impression. `accueil/accueil.css` : vignette de l'accueil allégée.
- Écartées faute de lieu « vide » : Sink Below, Visit Anvilheim, Territorial
  Domain, Clash of Mountains (personnages), Channel the Tranquil Domain
  (image encadrée, 920 px), Light of Sol (abstraite).
- `docs/interface.md` (« Fonds illustrés »), `docs/rapatriement-interface.md`
  (H6 repris).

## Vérification
- Appli réelle compilée depuis la branche sur le PC d'Antoine (dossier à
  part `fabbot-fonds`), captures 1440×900 de l'accueil (3 tirages), Jouer,
  Entraîner, Apprendre, Imprimer : illustrations HD chargées, textes lisibles.
- Non vérifié en image : le tapis en cours de partie (fond à 30 %).
