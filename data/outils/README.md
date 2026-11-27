# Outils locaux (non publiés)

Ce dossier reçoit des outils qu'on ne met pas dans le dépôt public, parce
qu'ils embarquent une base de cartes (éditions, liens d'images).

- `proxies-de-rathe.html` : l'outil d'impression complet de l'Arsenal JS
  (versions et langues des cartes, fond perdu, liseré, recto verso, export
  PDF…). S'il est ici, l'écran **Imprimer** d'Arsenal l'affiche, avec le
  deck choisi déjà chargé. Sans lui, Imprimer garde l'outil simple de
  fabbot. Copier le fichier tel quel (celui à côté de l'Arsenal JS) ; le
  serveur pose lui-même le mode intégré.

- `dos-de-carte.webp` (ou `.png`, `.jpg`) : facultatif. Arsenal affiche
  déjà le vrai dos des cartes Flesh and Blood partout où une carte est de dos
  (deck, main du bot, cartes cachées, didacticiel), en le chargeant en ligne
  comme les images des cartes. Une image posée ici passe avant (utile hors
  ligne). Sans réseau et sans image ici : un dos dessiné.
