# 2026-11-26 — Arsenal, tapis de jeu

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Demande** : le propriétaire voulait retrouver la disposition du tapis
  de l'Arsenal JS (`outils-bot/src/page/interface.js`, `fieldHTML`, et
  `gabarit.html`), qui n'avait pas été reprise au lot 2.

## Fait

- Serveur : chaque arme ou équipement porte son `emplacement` (`arme`,
  `tete`, `torse`, `bras`, `jambes`), lu dans la zone du moteur.
- Page (`partie/partie.js`, `partie.css`) : tapis par joueur, zones à leur
  place (équipement, arme, héros + arsenal, seconde arme ou off-hand +
  permanents, pitch en éventail collé au deck, cimetière / deck / banni) ;
  tapis du bot tourné à 180° ; PV gravés sur le héros ; fenêtre pour voir
  tout le cimetière ou les bannies ; main du bot en dos de cartes.
- Les ancres du film (lot 2) sont gardées : vols, impacts et glisser-déposer
  marchent comme avant.

## Tests

- 1 nouveau test : chaque arme et équipement a un emplacement valide, un
  seul par emplacement d'équipement, rien pour les cartes en main.
- Playwright : 25 coups joués sur le tapis, aucune erreur JavaScript.
