# 2026-11-28 — Appli Android, jalon 0 (préparation)

- **Agent** : Claude (compte Android), session cloud.
- **Branche** : `claude/stoic-galileo-dk0tet`.

## Fait

- Ligne « Appli Android » ajoutée dans « En cours » de `docs/taches.md`
  (pull request triangledeau/fabbot#88, fusionnée).
- Correctif partagé de la section 5.1 de `docs/android.md` : `dossier()`
  de `crates/cards/src/decks.rs` prend `FABBOT_DECKS` si la variable
  existe et n'est pas vide ; deux tests sur `dossier_selon`, qui ne touche
  pas l'environnement du processus (pull request triangledeau/fabbot#89,
  fusionnée, CI verte).
- Réponse d'Antoine notée : le téléphone est un Pixel 9.

## Pas fait

- Questions 2 à 6 de la section 12 (orientation, transfert de l'APK, où
  construire, dos de carte et Proxies de Rathe, identifiant et icône) :
  pas encore de réponse. À reposer au début de la session suivante.
- Aucune mesure : pas de code Android à ce jalon.
