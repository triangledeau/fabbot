# 2026-11-27 — Niveau réfléchi (chantier 4 de la feuille de route)

Agent : Claude. Branche : `claude/project-thread-lip8f5`.

## Fait
- `crates/ai/src/reflechi.rs` (nouveau) : `ReflechiAgent`, la recherche de
  B3 lancée sur tous les cœurs (une recherche indépendante par fil, visites
  additionnées option par option à la racine). Temps prévu = 3 s par option
  distincte en plus de la première × poids du genre (priorité et défense 2,
  pitch / arsenal 1, ordres 0,25, autres 0,5), entre 0,3 s et 2 min ; rien
  quand une seule option compte. Arrêt anticipé quand l'option la plus
  visitée ne peut plus être rattrapée dans le temps restant. `Suivi`
  partagé : avancement, « Joue maintenant ».
- Noms : `reflechi` (2 min max), `reflechi-30s`, `reflechi-5m`… ; pas dans
  `BOTS` (trop lent pour les tournois).
- `b2.rs` : `choose` découpé en méthodes publiques (`preparer`, `iterer`,
  `racine`, `meilleure_option`, `cles_des_options`) ; même comportement,
  estimation non touchée.
- Page : le bot réfléchit dans un fil à part (`Session::poursuivre`), jauge,
  option préférée (cartes cachées non nommées), bouton « Joue maintenant »,
  « Annuler » interrompt. Routes `GET /api/reflexion`, `POST /api/joue-maintenant`.
- Tests : `crates/cards/tests/bot_reflechi.rs`, `crates/ui/tests/reflechi.rs`.

## Mesuré
- Vitesse (conteneur cloud, 4 cœurs, release) : ≈ 6 800 parties imaginées
  par seconde et par fil ; B3 en fait 300 par décision. Sur la Surface
  (12 fils), à mesurer : une décision de 30 s devrait en faire plusieurs
  centaines de milliers.

## Pas fait
- Courbe force / temps de réflexion (à faire dans les épreuves de mesure :
  le niveau réfléchi se mesure position par position, pas en Elo).
- Réglage des temps par genre de décision : valeurs de départ, non mesurées.
- Arbre plus profond / parties simulées plus longues avec ce budget :
  à mesurer avant de changer les réglages de la recherche.
- Jauge dans l'interface texte (le niveau y marche, sans jauge).
- Cœurs performants / économes de la Surface : tous les fils sont égaux.
