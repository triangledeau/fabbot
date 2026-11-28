# 2026-11-28 — Appli Android, jalon 3 : la partie au doigt

- **Agent** : Claude (session cloud).
- **Branche** : `claude/vigilant-ptolemy-56q2tv` (nom imposé par la session).

## Travail fait

1. **Maquette** au format du Pixel 9 (412 × 851 points) :
   `docs/android/maquette-jalon-3.html`, deux versions, validée par Antoine
   (détail dans `docs/android.md`, section 10).
2. **Adaptation** dans `crates/android/web/mobile.css` et `mobile.js`
   (aucun changement dans `crates/ui/web/`) :
   - accueil : les trois volets tiennent sans défilement ;
   - Jouer : tuiles de 50 points, visage du héros en fondu (point des yeux
     noté à la main pour les 10 héros, table `YEUX` de `mobile.js`), tête
     de robot cochée pour « réglé pour le bot », panneau « Partie contre le
     bot » fixé en bas ;
   - préparation de la manche : liste qui défile, pied compact, toucher =
     +1 exemplaire, appui long = −1 ;
   - partie : le plateau devient une seule grille (les conteneurs du PC en
     `display: contents`, chaque zone posée à sa case) ; héros au centre
     entre ses deux armes, arsenal dessous, équipement en 2 × 2,
     cimetière / banni puis pitch / deck ; le bot tourné d'un demi-tour ;
     chaîne et pile au milieu ; main à gauche et permanents à droite,
     regroupés dès 2 exemplaires identiques (×N) ; question et boutons tout
     en bas ; barre du haut (menu ☰, score, Annuler, journal) ; colonne de
     droite du PC dans un tiroir ; appui long sur une carte = fiche (carte
     en grand et texte officiel) ; Retour d'Android referme fiche et
     tiroirs ;
   - fin de manche : boutons au doigt, « Manche suivante » visible en haut.
3. **Bug de la page PC relevé** (pas corrigé, hors zone) : la règle `.pv`
   de `crates/ui/web/apprendre/apprendre.css` n'est pas limitée au volet
   Apprendre et déforme l'aperçu de carte de la partie (`#detail`) sur PC.

## Vérifié

- Aperçu PC au format du Pixel 9 (Chromium, tactile émulé) : deux manches
  jouées entières contre Dromai (B1), environ 120 captures relues (rien qui
  déborde, rien qui se chevauche, textes en français, boutons d'au moins
  44 points) ; appui long, Fermer, menu, journal, Retour.
- `cargo fmt`, `clippy`, `cargo test --workspace` (dont le test des
  sélecteurs de `mobile.css` / `mobile.js`).

## Pas vérifié

- Sur le téléphone : appui long dans la WebView, bouton Retour, reprise
  après que l'appli a été tuée, match entier au doigt. C'est l'essai
  demandé à Antoine ; le jalon 3 n'est coché qu'après son retour.
