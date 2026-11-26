# 2026-11-26 — Analyse Iyslander, Hala et Ira avec B3

- Agent : Claude (session cloud), branche `claude/project-thread-j807fx`.
- Travail : tournoi B3 contre B3 (1 620 parties, 0 erreur) ; nouvel outil
  `fab-sim --bin analyse` (dégâts par source, cartes jouées / pitchées /
  défendues, X, fusions, capacités, attaques par tour ; `SIDE_ETUDIE`,
  `SIDE_ADVERSE`) ; variantes de B1 mesurées puis retirées du code (gardées
  dans `docs/analyses/2026-11-26-experiences-b1.patch`).
- Chiffres : Iyslander 21 %, Hala 42 %, Ira 60 % (B3 contre B3). Iyslander
  sans Nullrune adverses : 29,2 % → 51,3 % (B1, 3 600 parties). Hala avec
  B1 qui voit le go again de Zenith Blade : 42,0 % → 45,4 % (B1, 3 600) et
  42,6 % → 52,8 % (B3, 108). Variantes Ice Eternal / arsenal / Frost Spike :
  29,2 % → 29,8 % (non significatif).
- Rapport : `docs/analyses/2026-11-26-iyslander-hala-ira-b3.md`. Aucun
  correctif appliqué (accord du propriétaire attendu).
