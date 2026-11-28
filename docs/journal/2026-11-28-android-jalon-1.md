# 2026-11-28 — Appli Android, jalon 1 : la crate Rust et l'aperçu sur PC

- **Agent** : Claude (compte Android), session dans le cloud.
- **Branche** : `claude/laughing-ramanujan-pnd3nr` (nom imposé par la session).
- **Brief** : `docs/android.md` (point de reprise : section 10).

## Fait

- Nouvelle crate `crates/android` (`fab-android`, bibliothèque `cdylib`
  + `rlib`), ajoutée au workspace. Elle ne modifie aucune autre crate.
- `demarrer(dossier, textes, port)` : notes de plantage
  (`sauvegardes/plantages.txt` et journal d'Android), dossier courant =
  dossier privé de l'appli, decks du projet installés dans
  `<dossier>/decks/` à chaque démarrage (un deck retiré du projet est
  effacé, `decks/maison/` n'est jamais touché), `FABBOT_DECKS`, port
  réservé, puis un fil « serveur » qui reprend la partie sauvegardée ou
  crée le match par défaut et lit les textes des cartes. Un second appel
  rend le même port.
- Port fixe 38080 : la WebView range les réglages de la page
  (`localStorage`) par adresse, port compris ; un port changeant les
  ferait perdre.
- Boucle de serveur propre à l'appli (copie de `fab_ui::serveur::servir`,
  à garder alignée) ; routes `/android/mobile.css` et `/android/mobile.js` ;
  les deux balises ajoutées avant `</head>` ; dos de carte embarqué servi
  par l'appli s'il était présent à la compilation.
- `build.rs` : les decks `data/decks/*.txt` sont embarqués
  automatiquement (le 10e deck, Aurora, est déjà pris).
- Lien Kotlin par JNI (`fr.arsenal.fabbot.Moteur.demarrer`), compilé
  seulement pour Android : `cargo check` et `cargo clippy` passent pour
  `aarch64-linux-android`. L'édition de liens pour Android (NDK) n'a pas
  pu être faite dans la session : ce sera le rôle du workflow GitHub du
  jalon 2.
- `mobile.css` et `mobile.js` : la base (classe `android` sur `<html>`,
  toucher sans délai, pas de rebond de page). L'adaptation de l'écran de
  partie est le jalon 3.
- Binaire `apercu` : `cargo run -p fab-android --bin apercu`.
- Icône : l'emblème dessiné par Gemini, fourni par Antoine, détouré (fond
  vert retiré) et recadré au carré : `android/icone/arsenal-embleme.png`.
- Antoine a donné sa version d'Android : 17.

## Tests (18, noms en français)

- `tests/demarrage.rs` : démarrage dans un dossier vide, decks copiés sans
  écraser les decks maison (et un deck abîmé réparé, un deck retiré
  effacé), reprise d'une partie sauvegardée, textes lus dans le dossier de
  l'appli, injection des deux balises (`/` et `/index.html`), fichiers
  `mobile.*` servis, hôte non local refusé (403), sauvegarde écrite après
  un `POST` réussi. Les tests passent par de vraies connexions TCP locales
  et ne changent ni le dossier courant ni les variables d'environnement.
- `tests/selecteurs.rs` : chaque classe ou identifiant de la page PC visé
  par `mobile.css` / `mobile.js` existe encore dans `crates/ui/web/`, plus
  trois tests de l'outil de lecture des sélecteurs.

## Essai dans le navigateur

- Aperçu lancé, textes officiels des cartes chargés, page servie en
  entier avec les deux balises, classe `android` présente.
- Match contre B3 (Hala contre Dorinthea, 2 manches gagnantes) joué par
  un script Chromium qui clique dans la page (cartes jouables et boutons
  d'options, écran de préparation, « Manche suivante ») : RESULTAT_MATCH.
- Erreurs de la page pendant l'essai : seulement des ressources
  extérieures refusées par le réseau de la session (polices Google, images
  des cartes) et le 404 attendu du dos de carte absent.
- Au format du Pixel 9 (412 × 923, rapport de pixels 2,625, tactile) :
  l'accueil s'affiche correctement ; l'écran de partie déborde à droite
  et la grande carte, ouverte au toucher, couvre le plateau. À traiter au
  jalon 3.

## Non repris

Rien de la version PC n'est retiré à ce jalon (il n'y a pas encore
d'appli installable).
