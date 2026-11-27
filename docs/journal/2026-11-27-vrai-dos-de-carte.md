# 2026-11-27 — Vrai dos de carte partout

- Agent : Claude (session Claude Code), branche `claude/project-thread-f2dihz`.
- Demande d'Antoine : le dos affiché doit être le vrai dos officiel Flesh and
  Blood partout où il est visible.

## Recensement des dos visibles

| Endroit | Élément | Avant |
|---|---|---|
| Partie : deck des deux joueurs | `.z .dos` (`partie.js`) | dos dessiné |
| Partie : main du bot (éventail) | `.dos-main .dos` | dos dessiné |
| Partie : carte cachée (arsenal du bot face cachée, carte bannie face cachée…) | `.carte.cachee` | dos dessiné avec « ? » |
| Didacticiel : main de l'adversaire, cartes posées de dos | `.ct.dos` (`apprendre.js`) | dos dessiné |

Le vrai dos ne s'affichait que si l'image était posée à la main dans
`data/outils/` ; elle ne l'était pas sur la Surface.

## Changement

Une seule règle dans `commun/arsenal.css` pour tous ces éléments, en trois
couches : image locale `data/outils/dos-de-carte.*` si présente, sinon le dos
officiel en ligne (même adresse que l'Arsenal JS), sinon le dos dessiné.
L'ancien interrupteur `vrai-dos` de `cartes.js` est retiré.

## Non repris

- Impression (outil simple et Proxies de Rathe) : les cartes à une face sont
  imprimées avec un verso vierge ; aucun dos officiel n'est imprimé.
- Écrans Jouer, éditeur, Entraîner, accueil : aucune carte de dos n'y est
  affichée.
