# fabbot — consignes pour Claude Code

Les consignes communes à tous les agents (règles du projet, travail à
plusieurs, commandes) sont dans `AGENTS.md`, chargé ici :

@AGENTS.md

## Propre à Claude Code
- Claude crée lui-même la pull request et la **fusionne dans `main`** quand
  fmt, clippy, tous les tests et la vérification « CI » de GitHub sont au
  vert (accord du propriétaire du 2026-11-26). Si la fusion échoue
  (conflit), le signaler au propriétaire au lieu de forcer. Le propriétaire
  débute avec Git : dire simplement en fin de tâche ce qui a été fusionné.
- Une session Claude peut aussi, à la demande du propriétaire, créer et
  fusionner la pull request d'une branche poussée par un autre agent
  (`antigravity/...`), après avoir relancé fmt, clippy et les tests dessus.
- Le nom de branche imposé par la session (`claude/<mot>-<mot>-<code>`)
  convient : il commence bien par `claude/`.
