# 2026-11-28 : brief de l'appli Android

- Agent : Claude (fil « Préparer le brief de l'appli Android »)
- Branche : `claude/project-thread-an2c6f`
- Travail : lecture de `fab-ui` (démarrage de `fab-web`, serveur, chemins de fichiers, fonctions de la page qui posent problème au tactile) et écriture de `docs/android.md`, le brief autonome pour l'agent qui fera l'appli depuis un autre compte d'Antoine.
- Choix recommandé : WebView Android + bibliothèque Rust (`crates/android`) qui fait tourner le serveur de `fab-ui` sur `127.0.0.1` ; adaptation au téléphone par une feuille de style et un script ajoutés, sans toucher `crates/ui`.
- Piège repéré : `fab_cards::decks::dossier()` dépend du chemin de compilation ; Antoine a autorisé le 2026-11-28 un seul correctif partagé (variable `FABBOT_DECKS`), décrit en section 5.1 du brief.
- Pas de code dans cette tâche.
