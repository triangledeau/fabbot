# 2026-11-26 — Arsenal, lot 4a (decks en données)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Source** : idée de `tools/nouveau-deck.py` de l'Arsenal JS (lecture de
  l'export texte FaBrary) ; code réécrit, rien du moteur JS.

## Fait

- `fab-cards` :
  - `decklist::Decklist::to_text` (écrit une liste au format des fichiers) ;
  - `decks` : decks maison dans `data/decks/maison/` (`maison`,
    `enregistrer_maison`, `verifier_nom_maison`), trouvés par `trouver`
    (donc jouables partout où l'on donne un nom de deck) mais absents de
    `tous` (tournoi et tables apprises inchangés) ;
  - `import` (nouveau) : lit l'export texte de FaBrary ou le format du
    projet ; une carte sans couleur est classée (héros ou emplacement)
    par la base de cartes. Le héros doit avoir son nom exact (un héros
    adulte n'est pas remplacé par le jeune du même nom).
- `fab-ui` : `api_decks.rs` (nouveau) : collection Silver Age (base
  communautaire, drapeau `codee`), vérification d'une liste (format,
  légalité, héros Young, cartes codées, équipement de départ), routes des
  decks maison ; `/api/decks` donne aussi `decks_maison` (les jouables).
- Page : volet « Mes decks » (`#decks`, `web/decks/`), lien depuis
  l'accueil ; decks maison dans la fenêtre « Nouveau match ».
- La base communautaire ne connaît pas les cartes les plus récentes (par
  ex. Cleave the Heavens de Levia) : une carte absente de la base n'est pas
  déclarée illégale, et les cartes codées absentes y sont ajoutées.

## Tests

- `crates/ui/tests/decks_maison.rs` (5 tests) : import FaBrary d'un deck du
  projet → même liste, jouable, proposé et jouable contre le bot ; carte non
  codée → gardé mais pas jouable ; 3 copies signalées par « Vérifier » ; noms
  invalides ou déjà pris refusés ; la collection contient toutes les cartes
  des 9 decks, marquées codées.
- `fab-cards` : 5 tests d'import, aller-retour `to_text` / `parse` sur les
  9 decks.
- Playwright : import d'un deck jouable et d'un deck avec une carte non
  codée, liste « Nouveau match », aucune erreur JavaScript.
