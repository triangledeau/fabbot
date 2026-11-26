# Outils locaux (non publiés)

Ce dossier reçoit des outils qu'on ne met pas dans le dépôt public, parce
qu'ils embarquent une base de cartes (éditions, liens d'images).

- `proxies-de-rathe.html` : l'outil d'impression complet de l'Arsenal JS
  (versions et langues des cartes, fond perdu, liseré, recto verso, export
  PDF…). S'il est ici, l'écran **Imprimer** d'Arsenal l'affiche, avec le
  deck choisi déjà chargé. Sans lui, Imprimer garde l'outil simple de
  fabbot. Copier le fichier tel quel (celui à côté de l'Arsenal JS) ; le
  serveur pose lui-même le mode intégré.

- `dos-de-carte.webp` (ou `.png`, `.jpg`) : le vrai dos des cartes Flesh and
  Blood. S'il est ici, Arsenal l'affiche partout où une carte est de dos
  (deck, main du bot, cartes cachées) ; sinon, un dos dessiné.
