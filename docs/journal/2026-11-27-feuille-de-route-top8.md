# 2026-11-27 — Feuille de route vers un bot top 8

Agent : Claude, branche `claude/project-thread-s5ckuh`.

## Travail fait
- `docs/feuille-de-route-top8.md` : état mesuré (B0 → B3, matrice B3
  contre B3), faiblesses connues, cinq épreuves pour mesurer « top 8 » sans
  joueurs humains (puzzles exacts du tour, regret contre un bot lent, avis
  du propriétaire, parties contre lui, exploitabilité), chantiers par ordre
  d'impact avec sessions et calcul local.
- Cadre fixé par le propriétaire : seulement les 10 decks connus du bot,
  pas de calcul loué dans le cloud (calcul lourd sur la Surface, la nuit).
- Aucun code changé. Tâche de documentation courte : pas de ligne dans
  `docs/taches.md` (ajoutée et retirée dans la même pull request, elle
  n'aurait servi à personne).

## Chiffres
Aucune nouvelle mesure : les chiffres viennent de `docs/bots.md` et de
`docs/analyses/2026-11-26-iyslander-hala-ira-b3.md`. Les coûts sont des
estimations, à refaire après l'étape « Mesurer ».

## Mise à jour (même jour)
Le propriétaire accepte un niveau de bot qui réfléchit jusqu'à quelques
minutes par tour selon la complexité : ajouté (section « Le niveau
réfléchi », chantier 4 élargi, total ≈ 17 à 20 sessions).
Ajout : jauge de progression de la réflexion dans la page, avec un bouton « Joue maintenant ».
