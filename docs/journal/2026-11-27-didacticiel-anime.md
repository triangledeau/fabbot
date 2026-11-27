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

## Revue de design complète des chapitres 1 à 3 (même jour, suite)

Le propriétaire a demandé une revue complète avant toute livraison. Trois
passes de captures (84 étapes, en 1366 × 720 puis 1280 × 750), chaque
capture examinée. Corrigé :

- défenseurs qui se chevauchaient sur la chaîne ; cartes du cimetière
  décalées pour qu'on voie qu'il y en a plusieurs ;
- grande carte centrée ; rangée du joueur remontée ; noms des zones sous les
  cases, sur fond sombre, visibles même quand une carte les recouvre ;
- zooms : cadres recalés sur la ligne de type, cadres distincts pour la
  classe, le type, le texte de règles et le texte d'ambiance ; le zoom
  revient à chaque étape qui en parle ;
- compteurs de ressources et de points d'action lisibles, hors de la chaîne,
  retirés en fin de leçon ;
- leçon 3-4 illustrée (point d'action dépensé, carte restée en main,
  ressource perdue en fin de tour) au lieu d'un tapis vide ;
- le tapis se réduit à la hauteur réelle du guide : plus de bouton coupé
  dans la scène 3-5 ;
- Brutal Assault bleu montré en version anglaise, comme le rouge et le
  jaune (pas de VF pour ces deux-là) ;
- consignes du moteur tutoyées, sans « priorité » ni tiret cadratin ;
  espaces insécables avant la ponctuation double.

Reste imparfait : dos de carte de remplacement si `data/outils/dos-de-carte`
manque ; Raging Onslaught et Brutal Assault en anglais (pas de VF) ; la
taille du tapis varie un peu d'une étape à l'autre quand le guide s'allonge.
