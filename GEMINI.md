# fabbot — consignes pour Antigravity

**Lire d'abord `AGENTS.md`** : toutes les règles du projet et la façon de
travailler à plusieurs y sont. Ce fichier ne contient que ce qui est propre
à Antigravity.

## Propre à Antigravity
- Antigravity tourne **sur l'ordinateur du propriétaire**. Travailler dans
  un **dossier à part** (par exemple `fabbot-antigravity`, un second
  `git clone` du projet), jamais dans le dossier `fabbot` où se trouve
  `Jouer.bat` : ce dossier-là doit rester sur `main` pour que le jeu se
  mette à jour correctement.
- Branches : `antigravity/<sujet>`, créées depuis `main` à jour.
- Avant de pousser : `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, tout au vert.
- Pull request : si l'outil `gh` (GitHub en ligne de commande) est
  installé et connecté, créer la pull request vers `main` et la fusionner
  quand la vérification « CI » est verte. Sinon, pousser la branche et dire
  au propriétaire : « branche `antigravity/<sujet>` poussée, à fusionner » ;
  il peut demander à une session Claude de la vérifier et de la fusionner.
- Ne pas modifier `CLAUDE.md` (consignes propres à Claude Code) ; une règle
  qui vaut pour tous va dans `AGENTS.md`.
