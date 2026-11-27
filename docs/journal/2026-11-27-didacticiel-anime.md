# 2026-11-27 — Didacticiel : refonte animée

- Agent : Claude Code, branche `claude/project-thread-0p5cpk`.
- Demande : le propriétaire a jugé la 1re version « très bof » (notions
  non expliquées avant usage, pas d'icônes officielles, cartes peu
  montrées, moche, pas animé).
- Fait : tapis animé plein écran ; chapitres 1 à 3 réécrits (duel, lire une
  carte, payer) ; zooms sur les coins des cartes (anneau, trait et étiquette
  du même côté) ; icônes officielles chargées depuis rules.fabtcg.com ;
  test `ordre_des_notions` ; quiz sans solution en cas d'erreur ; carte
  montrée pendant le paiement dans les scènes. Les leçons des anciens
  chapitres 3 à 7 sont reprises telles quelles dans les chapitres 4 à 7
  (sans animation), en attendant leur refonte.
- Vérifié : les 24 leçons parcourues de bout en bout dans un navigateur
  sans tête, sans erreur JavaScript ; 8 tests du didacticiel au vert.
- Pas fait : animation des chapitres 4 à 7 ; chapitres 8 à 12 ; vérification
  des icônes officielles dans l'appli d'Antoine (le serveur cloud n'a pas
  accès au site des règles).
