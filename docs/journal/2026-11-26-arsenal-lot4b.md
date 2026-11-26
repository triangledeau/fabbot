# 2026-11-26 — Arsenal, lot 4b (éditeur de deck)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Source** : `outils-bot/src/page/editeur/editeur.js` et `editeur.css` de
  l'Arsenal JS (apparence et gestes), réécrits autour de l'API.

## Fait

- Page : `web/decks/editeur.js` (adresse `#editeur`) et styles dans
  `decks.css` ; le volet « Mes decks » gagne « Nouveau deck », « Modifier »,
  « Dupliquer » (decks du projet), « Archiver », archives (« Restaurer »,
  « Supprimer »).
- Serveur (`api_decks.rs`) : `GET /api/deck?nom=` (deck du projet ou
  maison), `POST /api/deck-maison/archiver|restaurer|supprimer` ; la
  collection donne classes, talents, mots-clés, texte, `heros`,
  `emplacement` (une carte colorée, comme les équipements Evo, reste une
  carte de deck). Tokens retirés de la collection.
- `fab-cards::decks` : `archives_maison`, `archiver_maison`,
  `restaurer_maison`, `supprimer_archive_maison` (seul un deck archivé se
  supprime).
- Question ouverte : vérification classe / talent des cartes d'un deck
  (règle de construction) ; le filtre « pour ce héros » n'est qu'une aide.

## Tests

- 3 nouveaux tests (`tests/decks_maison.rs`) : lecture d'un deck du projet
  pour le dupliquer, cycle archiver / restaurer / supprimer, emplacements et
  héros dans la collection.
- Playwright : dupliquer Hala, filtrer, ajouter, limite de 2 exemplaires,
  Ctrl+Z, enregistrer ; aucune erreur JavaScript.
