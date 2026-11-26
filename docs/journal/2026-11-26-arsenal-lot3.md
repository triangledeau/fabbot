# 2026-11-26 — Arsenal, lot 3 (sauvegarde et reprise)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Source** : idée de la sauvegarde de l'Arsenal JS (graine + choix) ;
  rien du moteur JS repris.

## Fait

- `web.rs` : la session garde tous les coups appliqués (humain et bot) ;
  `Session::rejouer` reconstruit une partie en rejouant ces coups, sans
  appeler le bot, et refuse une sauvegarde qui ne colle pas au moteur.
- `match_web.rs` : le match note ses étapes (premier joueur, équipement,
  deck, coups de chaque manche, passage à la manche suivante).
- `sauvegarde.rs` (nouveau) : format `arsenal-fabbot` v1, export, import,
  écriture sûre (fichier temporaire puis renommage).
- `serveur.rs` : `GET /api/sauvegarde`, `POST /api/charger`, sauvegarde
  automatique après chaque POST réussi ; `fab-web` reprend
  `sauvegardes/en-cours.json` au lancement (`--nouvelle-partie` pour
  l'éviter). Dossier `sauvegardes/` ignoré par Git.
- Page : boutons Exporter / Importer dans l'en-tête de la partie.

## Tests

- 4 nouveaux tests (`tests/web.rs`) : partie rechargée au même point (état
  identique, suite identique) ; match rechargé après une manche finie garde
  le score ; sauvegarde abîmée refusée sans toucher au match ; fichier de
  sauvegarde automatique relu à l'identique.
- Playwright : export puis import dans la page, sans erreur.
