# 2026-11-26 — Didacticiel « Apprendre » (phase 2), début

- Agent : Claude (session cloud), branche `claude/project-thread-0p5cpk`.
- Structure proposée au propriétaire et public choisi (débutant complet) :
  voir `docs/didacticiel.md`.
- Règles officielles : rules.fabtcg.com et le PDF de fabtcg.com renvoient 403
  aux accès automatiques ; copie texte faite avec le navigateur de l'appli
  Claude, rangée dans `docs/rules/sources/` (non versionné), avec l'accord du
  propriétaire.
- Fait : volet `#apprendre` (sommaire, leçons, questions, tableau de
  couverture du CR), scènes sur le moteur (`api_apprendre.rs`, bot scripté,
  `Session::avec_agent`), chapitre 1 (3 leçons, 1 scène, 3
  questions), chapitre 2 (4 leçons : nombres, couleur et pitch, ligne de type, texte), tuile Apprendre de l'accueil active.
- Mesuré : couverture 18 sections du CR sur 87 (1.1, 2.1 à 2.5, 2.7 à 2.12, 2.14, 2.15, 4.1, 4.4,
  4.5, 7.5 — voir le tableau dans la page).
- Pas fait : chapitres 3 à 13, mémos, rappel éclair d'une leçon précédente.
