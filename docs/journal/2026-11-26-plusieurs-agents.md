# 2026-11-26 — Organisation pour plusieurs agents

- Agent : Claude Code ; branche `claude/beautiful-cerf-kzalhm`.
- fabbot devient la version principale du bot, travaillée par plusieurs
  sessions Claude et une session Antigravity.
- `AGENTS.md` : consignes communes à tous les agents (contenu de l'ancien
  `CLAUDE.md` + « Travailler à plusieurs » : branches par agent, tableau
  des tâches, journal en fichiers séparés, conflits, fichiers à un seul
  agent à la fois). `CLAUDE.md` le charge (`@AGENTS.md`) et garde ce qui est
  propre à Claude Code ; `GEMINI.md` : propre à Antigravity (dossier à part,
  pull requests).
- `docs/taches.md` : tâches en cours et découpage conseillé en tâches
  parallèles.
- Journal : l'ancien tableau de `docs/suivi.md` est dans
  `docs/journal/historique.md` ; une session = un fichier.
- `.github/workflows/ci.yml` : vérification automatique (fmt, clippy,
  tests) de chaque pull request.
