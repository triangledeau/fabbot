# 2026-11-26 — Arsenal, lot 9 (extras) et classes des decks maison

- Agent : Claude (Claude Code), branche `claude/project-thread-lcawsx`.
- Plan : `docs/rapatriement-arsenal.md`, lot 9. Fin de la phase 1.

## Fait
- Classes et talents des decks maison (PR #31) : le héros doit avoir les
  classes et talents de chaque carte ; carte hybride : un côté suffit
  (réponse du propriétaire) ; « Essence of X » ajoute le talent X (sans
  quoi Iyslander ne passait pas). Deux hypothèses à confirmer, notées dans
  `docs/suivi.md`.
- `crates/ui/src/extras.rs` : `GET /api/conseil`, `GET /api/evaluation`,
  `POST /api/auto`, `POST /api/bug`, `GET /api/bugs`.
- Conseil : le bot choisi décide pour le joueur humain (il ne lit que la
  vue de ce joueur), graine fixée par la position.
- Évaluation : `fab_ai::b3::Evaluation::estimer` en moyenne sur 24 mondes
  `determiniser` (rien de caché n'influence la barre) ; `None` pendant un
  choix du bot.
- Page : `web/partie/extras.js` et `extras.css` ; `afficher()` émet
  `arsenal:etat`.
- Tests : `crates/ui/tests/extras.rs` (conseil valide et stable, barre
  entre 0 et 1, manche entière en Bot contre Bot, bug enregistré puis
  partie rechargée depuis le signalement).
