# 2026-11-28 — Appli Android, jalon 2 : premier APK, construit par GitHub

- **Agent** : Claude (compte Android), session dans le cloud.
- **Branche** : `claude/lucid-heisenberg-nxwfjz` (nom imposé par la session).
- **Brief** : `docs/android.md` (point de reprise : section 10).

## Fait

- Projet Gradle `android/` : Gradle 8.11.1 (wrapper committé), plugin
  Android 8.7.3, Kotlin 2.0.21, `compileSdk`/`targetSdk` 35, `minSdk` 26,
  identifiant `fr.arsenal.fabbot`, nom « Arsenal », seule dépendance
  `androidx.appcompat`.
- Activité unique `ActiviteArsenal` : WebView plein écran sur
  `http://127.0.0.1:<port>/`, `localStorage` actif, bouton Retour (page
  précédente, sinon quitter), liens extérieurs dans le navigateur,
  `alert`/`confirm` en boîtes de dialogue Android en français, marges
  pour les barres du système, message « Chargement d'Arsenal… » pendant
  le démarrage, message d'erreur en français si le moteur ne démarre pas.
  Portrait verrouillé (choix d'Antoine).
- Textes officiels des cartes : rangés dans l'APK (non compressés), copiés
  dans le dossier de l'appli au premier lancement et après chaque mise à
  jour, là où le moteur les attend (`fab_android::chemin_textes`).
- HTTP non chiffré limité à `127.0.0.1` et `localhost`.
- Icône adaptative tirée de l'emblème (marge pour le rognage en cercle),
  version monochrome pour les icônes à thème d'Android 13+.
- `android/construire.sh` / `construire.ps1` (bibliothèque Rust par
  `cargo ndk` avec pages de 16 Ko et sans symboles, textes, APK).
- `.github/workflows/android.yml` : APK en artefact `arsenal-<n>`, sur
  les pull requests qui touchent l'appli, sur `main` et à la demande.
  Secrets facultatifs : clé de signature, dos de carte.
- Clé de signature créée pour Antoine (RSA 4096, alias `arsenal`, validité
  100 ans) et envoyée directement ; elle n'est pas dans le dépôt.

## Mesuré

- Construction GitHub : MESURE_DUREE.
- Taille de l'APK : MESURE_TAILLE.

## Pas vérifié

- L'installation et le démarrage sur un vrai téléphone : la session cloud
  n'a ni SDK ni émulateur (`dl.google.com` refusé par son réseau). C'est
  l'essai demandé à Antoine sur son Pixel 9 ; le jalon sera coché après
  son retour.
- Captures : seulement l'aperçu PC au format du Pixel 9 (accueil en
  portrait, préparation de la manche). Rien ne déborde à l'accueil.

## Non repris (à ce jalon)

- Paysage : écarté par Antoine (portrait pour toute l'appli).
- Téléchargements, import de fichier, impression, presse-papiers :
  jalons 4 et 6.
