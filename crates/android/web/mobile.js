// Arsenal sur Android : adaptation de la page PC au téléphone.
//
// Servi par l'appli à /android/mobile.js et ajouté à la fin du <head> de
// la page (crates/android/src/serveur.rs). Règle : toute adaptation passe
// par ce fichier et mobile.css, jamais par crates/ui/web/.
//
// Les sélecteurs de la page PC utilisés ici (querySelector, closest,
// getElementById…) sont vérifiés par crates/android/tests/selecteurs.rs.
// Les classes créées ici commencent par « android ».
//
// Jalon 1 : la base seulement (docs/android.md, section 9).
"use strict";

(function () {
  // Marque la page : mobile.css ne s'applique qu'à html.android, la page
  // PC reste donc inchangée hors de l'appli.
  document.documentElement.classList.add("android");
})();
