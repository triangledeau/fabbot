# Reprise de l'interface de l'Arsenal JS : liste exhaustive des écarts

26/11/2026. Comparaison faite dans les **deux applis lancées** (Arsenal JS :
`arsenal.html` du dossier fleshandblood ; fabbot : `fab-web` de `main`), écran
par écran, puis dans le code de l'Arsenal JS pour les menus, raccourcis et
comportements qu'une capture ne montre pas. Captures côte à côte :
`/mnt/project-files/arsenal/comparaison/*.png` (dossier du projet).

Légende : ✅ déjà dans fabbot · 🟡 en partie · ❌ absent · ⛔ non repris
(volontairement, raison donnée).

Ce qui reste exclu (décisions déjà prises) : fonds ArtStation de l'accueil,
code Talishar, règles du moteur JS (le moteur fabbot reste la référence), son.
Didacticiel et puzzles : phases 2 et 3 du plan.

## 1. Écran de partie (le plus gros écart)

| # | Arsenal JS | fabbot (après la reprise de l'écran de partie) | État |
|---|---|---|---|
| P1 | Tapis plein écran : le bot en haut (zones tournées à 180°), vous en bas, chaîne au milieu ; colonne de droite séparée | repris : tapis plein écran, sans en-tête, colonne à droite | ✅ |
| P2 | Illustration du héros de chaque joueur en fond de sa moitié du tapis (image de carte chargée en ligne) | repris (image de la carte du héros, chargée en ligne comme les autres images) | ✅ |
| P3 | Équipement en colonnes (tête, torse, jambes ; bras collé au torse côté centre) | repris | ✅ |
| P4 | Arme / Héros (+ PV en grosse pastille) / Arme ou Off-hand ; Arsenal sous le héros ; Permanents (ou « Alliés (n) ») sous l'off-hand | repris | ✅ |
| P5 | Pitch collé au deck ; Cimetière / Deck / Banni avec compteurs | ✅ (compteurs présents) | ✅ |
| P6 | Clic sur le cimetière ou le banni : fenêtre qui montre toute la zone | ✅ (fenêtre de la zone) | ✅ |
| P7 | Bloc « état » de chaque joueur : ressources en symboles de pitch (cercles rouges, 3 par symbole), points d'action en losanges, « Effets en cours » en pastilles (auras, jetons, marqué, bonus de la prochaine attaque…) avec aperçu au survol | repris : symboles de pitch, losanges, pastilles « Effets en cours » (marqué, capacités du héros perdues, prochaine attaque, réductions de coût, « comme un instant », préventions, cartes jouables depuis le banni) ; pas d'aperçu au survol des pastilles | ✅ |
| P8 | Pendant un paiement : les symboles montrent « Paie N, reste M », contour doré pour ce qui manque | repris (« Paie N · reste M », contour doré) | ✅ |
| P9 | Main du bot : dos de cartes en éventail avec « Main du bot (n) » | ✅ | ✅ |
| P10 | Votre main en carré sous la pile (2 colonnes jusqu'à 4 cartes, 3 au-delà), « Ta main (n) » | repris | ✅ |
| P11 | Barre des étapes du combat : Layer · Attaque · Défense · Réactions · Dégâts · Résolution, l'étape en cours soulignée ; « Tour N » ; « priorité : toi / bot » | repris (+ « priorité : vous / bot ») | ✅ |
| P12 | Chaîne de combat : chaque maillon avec la carte, l'encadré « Attaque X / Défense Y » en gros, les bloqueurs à côté, les réactions jouées, « Maillon n · héros », modificateurs, Go again / Dominate, résultat « Touche : N » ou « Bloquée » ; flèches entre maillons ; défilement vers le dernier | repris ; un maillon déjà résolu montre « Touche » ou « Bloquée » mais pas sa puissance finale (le moteur ne la garde pas) ni la liste des modificateurs | 🟡 |
| P13 | Pile (« la carte de devant se résout en premier ») : couches empilées avec leur type (déclencheur, attaque, capacité) et leur propriétaire ; la carte en cours de paiement y attend (« à payer ») | repris (couches avec carte, genre, propriétaire) ; la carte en cours de paiement n'y est pas montrée à part | ✅ |
| P14 | **Ruban d'invite** sous la chaîne : une phrase qui dit la situation (« Iyslander · Scar for a Scar 4 — défense 0, tu subis 4 ») et les boutons du choix (« Ne pas bloquer » / « Valider la défense ») | repris | ✅ |
| P15 | Priorité : clic sur une carte de la main la sélectionne, ses façons de la jouer apparaissent en boutons dans le ruban ; capacités d'arme, de héros, d'équipement en boutons ; « Fin de ma phase d'action » / « Passer (fermer la chaîne) » / « Passer » ; « Passer jusqu'à la fin du tour » | repris : clic = sélection, boutons dans le ruban, « Fin de ma phase d'action » / « Passer (fermer la chaîne) » / « Passer », « Passer jusqu'à la fin du tour » | ✅ |
| P16 | Clic sur l’arme, le héros, un équipement, un allié jouable : active sa capacité directement | ✅ (carte surlignée cliquable) | ✅ |
| P17 | **Défense au clic** : on touche les cartes de la main / l'équipement / l'arsenal (Ambush) pour les poser sur la chaîne, un nouveau clic les reprend ; le total de défense et « tu subis N » se mettent à jour ; notes (Dominate : 1 carte max, Unity, Crush…) puis « Valider la défense » | repris : brouillon de défense (main, équipement, arsenal), total et « vous subissez N » à jour, envoyé d'un coup et annulé d'un coup ; note Dominate et arsenal (les autres notes de l'Arsenal JS venaient de son propre moteur) | ✅ |
| P18 | **Paiement au clic** : on touche les cartes à pitcher ; dès que c'est assez, c'est payé ; « Annuler » rend la carte | clic = la carte est pitchée tout de suite (le moteur demande une carte à la fois) ; « Payez N (reste M) » dans le ruban et sur les symboles | 🟡 |
| P19 | Fin des réactions : le ruban résume « Fin des réactions. Le bot bloque avec X : 4 contre 3. Tu subis 1. » + « Continuer » | « Pause avant les dégâts » (case) | 🟡 |
| P20 | Choix (modes, cibles) : boutons dans le ruban, le premier en évidence | repris | ✅ |
| P21 | Choisir une carte hors main (cimetière…) : petites cartes cliquables dans le ruban | repris (petites cartes cliquables dans le ruban) | ✅ |
| P22 | Choisir plusieurs cartes : on les touche, « sélection n / max », « Valider » / « Ne rien mettre » | on touche les cartes (ou leurs boutons dans le ruban) une par une, chacune part tout de suite ; pas de compteur « n / max » | 🟡 |
| P23 | Remettre des cartes sur le deck : on les touche dans l'ordre (numéro affiché sur la carte) | on touche les cartes une par une, chacune part tout de suite ; pas de numéro | 🟡 |
| P24 | Arsenal en fin de tour : « touche une carte de ta main », « Ne rien mettre en arsenal » | repris | ✅ |
| P25 | Ordre des cartes pitchées sous le deck : on les touche dans l'ordre, « Valider cet ordre » / « Garder l'ordre de pitch » / « Recommencer » | repris : on touche dans l'ordre (numéro sur la carte), Valider / Garder l'ordre de pitch / Recommencer | ✅ |
| P26 | Tirer une carte (glisser) vers la chaîne, le héros adverse (anneau de visée), le pitch ou l'arsenal, avec une chaîne de laiton qui suit | ✅ (lot 2) | ✅ |
| P27 | Survol d'une carte de la main : elle s'incline vers le curseur | repris (avec les animations) | ✅ |
| P28 | Colonne de droite : bouton Accueil, bouton « Options » (Animations oui/non ; « Abandonner et revenir au choix des decks ») ; aperçu de la carte survolée **en grand avec son image** et son texte ; journal ; en bas « Annuler mon dernier choix » (Ctrl+Z) et « Exporter la partie » | repris : Accueil, Options (Animations, Pause avant les dégâts, Nouveau match, Importer, Signaler un bug, Abandonner), aperçu en grand, journal, Annuler / Exporter en bas ; le conseil et la barre d'évaluation de fabbot restent | ✅ |
| P29 | Journal : lignes par tour (« — Tour 1 : Iyslander (18 PV) — »), le plan du bot en encadré (« Plan de Iyslander : … value estimée 14,5 »), reste collé en bas sauf si on remonte | titres de tour et dégâts en rouge ; le plan du bot n'est pas affiché (le bot fabbot ne l'écrit pas) | 🟡 |
| P30 | Bannière « Tour N · à toi / héros » à chaque nouveau tour | ✅ (« Votre tour » / « Tour du bot », lot 2) | ✅ |
| P31 | Rythme de lisibilité : rien ne quitte l'écran avant d'avoir été lu (pile : 1,1 s pour une couche du bot, 0,35 s pour la vôtre ; chaîne : 1,3 s avant de se fermer) | animations du lot 2 | 🟡 |
| P32 | Fin de partie : fenêtre « Victoire ! / Victoire du bot / Égalité », nombre de tours, PV finaux, **dégâts infligés, dégâts empêchés en bloquant (cartes de la main, blocs d'équipement)** ; boutons Rejouer (nouveau tirage si bot au hasard), Changer de deck, Voir le journal, Annuler mon dernier choix, Exporter | repris : fenêtre de fin, tours, PV, dégâts infligés et subis, défenses (main / autres), boutons ; « dégâts empêchés » remplacé par « dégâts subis » (non calculé par le moteur) | 🟡 |
| P33 | Cartes du bot jouables depuis le banni (Crouching Tiger, Nuu) affichées dans un groupe « Banni » | repris dans les « Effets en cours » (« Jouable depuis le banni ») | ✅ |

fabbot a en plus (à garder) : match en plusieurs manches, préparation
équipement + deck avant chaque manche, conseil du bot, barre d'évaluation, Bot
contre Bot, signalement de bug, Importer une partie.

## 2. Écran « Jouer » (choix des decks)

| # | Arsenal JS | fabbot aujourd'hui | État |
|---|---|---|---|
| J1 | Barre du haut : Accueil, « JOUER », « Choisis ton deck, celui du bot, ou crée le tien », boutons Relier le dossier, + Nouveau deck, Importer une liste, Imprimer un deck (menu : « Imprimer quel deck ? »), Decks archivés (n) | écran « Mes decks » : listes déroulantes et boutons | ✅ (sauf « Relier le dossier » : voir J11) |
| J2 | « MES DECKS » : tuile « ＋ Nouveau deck — choisir un héros Young », puis une tuile par deck maison | liste déroulante | ✅ (tuile « Nouveau deck » compacte tant que la rangée est vide, proposition du document d'ergonomie) |
| J3 | « DECKS DU BOT » : une tuile par deck avec l'illustration du héros, nom, héros · source, pastilles (« réglé pour le bot » / « liste modifiée » / « jouable contre le bot » / « héros pas encore codé » / « n cartes pas encore jouables » / « n hors règles », « 55 / 55 » ou « n / 40 ») | liste de lignes « ira — Dupliquer » | ✅ (pastilles : réglé pour le bot / jouable / héros pas encore codé / n cartes pas encore jouables / n hors règles, et « n / 55 ») |
| J4 | Clic sur une tuile : la choisir (cadre doré) ; double-clic : l'ouvrir dans l'éditeur ; Entrée / Espace au clavier : la choisir | non | ✅ |
| J5 | **Clic droit sur une tuile, ou bouton ⋯** : menu « ▶ Jouer avec ce deck · ✎ Modifier (la liste du bot) · ⧉ Dupliquer · ⎙ Imprimer (proxies) · ⇩ Exporter la liste · ↺ Revenir à la liste d'origine (si modifiée) · 🗄 Archiver » ; flèches haut/bas dans le menu, Échap pour fermer | « Dupliquer » seulement | ✅ ; en plus « Le bot joue ce deck » (document d'ergonomie). Pour un deck du bot, « Modifier » ouvre une copie (voir J10) |
| J6 | Panneau « PARTIE CONTRE LE BOT » : votre héros VS celui du bot (« ? » si au hasard), « Deck du bot » (Au hasard ou un deck), avertissements (deck à 39 cartes, carte non jouable…), « AJUSTER ET LANCER », « Charger une partie (.json) » | fenêtre « Nouveau match » | ✅ ; en plus Niveau du bot (libellés de joueur), Manches, graine sous « Options avancées » |
| J7 | Archiver avec « Annuler » dans le message ; écran « Decks archivés » : Désarchiver, Supprimer… (confirmation) | non | ✅ (decks du bot archivés dans le navigateur, decks maison dans `archives/`) |
| J8 | Écran « Nouveau deck » : grille des héros Young légaux avec leur image, filtres Tous / Jouables par le bot / par classe, pastille « bot ✓ / ✗ » ; clic = deck créé et ouvert dans l'éditeur | liste déroulante de héros + « Nouveau deck » | ✅ |
| J9 | Exporter la liste : fenêtre avec le texte (copier / télécharger) | non | ✅ (texte du fichier fabbot : Copier, Fichier .txt) |
| J10 | Revenir à la liste d'origine d'un deck du bot modifié (confirmation) | non | ⛔ les decks du bot sont des fichiers du projet (réglages et table de side appris dessus) : on ne les modifie pas depuis Arsenal, « Modifier » en fait une copie. À rediscuter si Antoine veut modifier les listes du bot |
| J11 | Relier le dossier fleshandblood (decks en .json) | ⛔ fabbot enregistre déjà sur le disque (`data/decks/maison/`) | ⛔ |

## 3. Fenêtre « Ajuster et lancer » (avant la partie)

| # | Arsenal JS | fabbot aujourd'hui | État |
|---|---|---|---|
| A1 | Fenêtre plein écran : « Bravo contre Ira », pastille « tiré au sort », bouton « Retirer au sort » | préparation de manche dans l'écran de partie | ✅ (fenêtre à la place du tapis, panneau Carte gardé à droite ; Retirer au sort refait le match, manche 1 seulement) |
| A2 | « Départ : … » + choix Dernier réglage / Plan conseillé / Deck par défaut (grisés si absents) | ✅ (Dernier réglage, Plan conseillé, Proposition) | ✅ |
| A3 | « Ce que joue le bot ▾ » : déplie ce que joue le deck adverse | non | 🟡 héros, armes et équipements du card-pool du bot ; **pas** son side contre votre deck (le bot choisit sans voir le vôtre, on ne montre pas son choix à l'avance : à confirmer avec Antoine) |
| A4 | Le pool en **piles de cartes** (une pile par carte, exemplaires décalés), « 2/2 » ; **clic gauche active un exemplaire, clic droit le désactive** ; clavier Entrée/+ et Suppr/− ; message si impossible | grille de cartes | ✅ (piles, clic / clic droit, Entrée + / Suppr −, message si impossible) |
| A5 | Compteur « Deck 40 / 40 prêt / manque N / N de trop », « n exemplaires désactivés (side) » | compteur présent | ✅ |
| A6 | Équipement et arme : une rangée par emplacement, la carte choisie encadrée | ✅ (étape « Valider l'équipement ») | ✅ (une rangée par zone ; l'équipement du bot apparaît à côté après validation, CR 4.1.5) |
| A7 | Bas de fenêtre : **Rythme** (Normal / Rapide) et **Réflexion** (Rapide / Normale / Sans limite) du bot | non | 🟡 Rythme ✅ (Normal / Rapide) ; Réflexion = niveau du bot (B0 à B3, B2 ½ s, B2 1 000 essais), modifiable avant la première manche seulement (changer en cours de match casserait la reprise d'une partie sauvegardée) |
| A8 | Taille des cartes ajustée pour que tout tienne sans défiler | non | ✅ |
| A9 | Retour / LANCER LA PARTIE (actif seulement à 40) | ✅ (Valider le deck) | ✅ |

## 4. Éditeur de deck

| # | Arsenal JS | fabbot aujourd'hui | État |
|---|---|---|---|
| E1 | Barre : ← Jouer, nom du deck (modifiable), état « enregistré · … » / « ● modifications non enregistrées », Importer, Exporter, Imprimer, Enregistrer (Ctrl+S) | ✅ (Imprimer ouvre Imprimer avec ce deck, après enregistrement) | ✅ |
| E2 | Recherche (« / » pour y aller) ; filtres Pitch, Coût 0/1/2/3+, Type, **Mots-clés (+ ajouter…)**, Classe / talent, Afficher (légales pour le héros, jouables par le bot, déjà dans le deck, jouables d'abord), Tri | ✅ | ✅ |
| E3 | **Taille des cartes** (curseur, mémorisée) | ✅ | ✅ |
| E4 | Clic = +1 au deck ; **⇧ clic = +1 au side ; Alt clic = +1 aux envies** ; Entrée au clavier | ✅ | ✅ |
| E5 | **Clic droit sur une carte** (grille ou liste) : menu « Ajouter un exemplaire (au deck, au side, aux envies) · Déplacer (1 ou tous, Deck ↔ Side ↔ Envies) · Retirer (1, toutes les copies) · 🔍 Voir en grand · ◐ Les autres couleurs de cette carte » ; touche Menu ou ⇧F10 au clavier | ✅ | ✅ |
| E6 | Glisser une carte vers une catégorie (⇧ en lâchant = tous les exemplaires) | ✅ | ✅ |
| E7 | **Quatre catégories** : Deck (40), Side (reste des 55), Équipement et armes, **Liste d'envies** (hors réserve) ; repliables | ✅ (Deck = 40 premières cartes du fichier, Side = la suite ; envies gardées dans le navigateur, pas dans le fichier) | ✅ |
| E8 | En-tête du héros (illustration, PV, intellect), compteurs Deck / Side / Équipement / Envies | 🟡 illustration, nom, classe et compteurs ; pas de PV ni d'intellect (la collection ne les donne pas) | 🟡 |
| E9 | Pitch en barre, ressources par main de 4, courbe de coût | ✅ | ✅ |
| E10 | Onglets **Liste / Équilibrage / Vérifications ✓** ; Équilibrage = rôles des cartes comparés à la moyenne des decks du bot ; Vérifications = liste des problèmes, clic = montrer les cartes en cause | 🟡 trois onglets et repère des 9 decks du bot ; le clic ne montre que les cartes pas encore jouables par le bot | 🟡 |
| E11 | Lignes de la liste avec l'illustration de la carte en fond, boutons + / − | ✅ | ✅ |
| E12 | Aperçu au survol (grande image + texte + « jouable par le bot ») | ✅ (posé à côté de la carte, jamais sur le panneau) | ✅ |
| E13 | Bandeau « Deck du bot » + « Dupliquer plutôt » | ✅ bandeau « Copie du deck du bot » (Modifier fait déjà une copie : pas de « Dupliquer plutôt ») | ✅ |
| E14 | Annuler (Ctrl+Z, 60 pas) | ✅ | ✅ |
| E15 | Échap ferme menu et aperçu | ✅ | ✅ |

## 5. Accueil

| # | Arsenal JS | fabbot aujourd'hui | État |
|---|---|---|---|
| H1 | Trois volets avec **l'illustration d'un héros** (Dorinthea pour Jouer, Kano pour Apprendre, Bravo VS Iyslander pour Entraîner) | ✅ portraits des héros des decks du bot (Dorinthea ; Apprendre : Ira, Kano n'étant pas un deck du bot ; Entraîner : Bravo contre Iyslander) | ✅ |
| H2 | « Partie en cours : X contre Y, tour N » + Reprendre / L'oublier | ✅ (match en cours, Reprendre / Nouveau match) | ✅ |
| H3 | Bas du volet Jouer : « Decks 14 (0 à toi) », « Réflexion rapide · normale · sans limite » | ✅ « Decks 9 (0 à toi) » ; la ligne Réflexion devient « Bots B0 · B1 · B2 · B3 » (fabbot règle le niveau, pas un temps de réflexion) | ✅ |
| H4 | Volet Entraîner : « Dernier tournoi » + barre d'avancement | ✅ « Dernier tournoi » (ou tournoi en cours) + barre | ✅ |
| H5 | Pied : version du moteur, nombre de cartes de l'éditeur | ✅ (texte proche) | ✅ |
| H6 | Fond d'accueil ArtStation | ⛔ exclu | ⛔ |

## 6. Imprimer

| # | Arsenal JS | fabbot aujourd'hui | État |
|---|---|---|---|
| I1 | Outil « Proxies de Rathe » complet : liste Fabrary ou libre, tokens liés, exemplaires, **Deck + side / Deck seul / Avec les envies**, langue (français si disponible), version par défaut, éviter les foils, choix de la version carte par carte, taille, fond perdu, liseré, rognage, page, export PDF | ✅ l'outil lui-même, intégré dans Imprimer avec le deck chargé et les trois portées, s'il est copié dans `data/outils/` (pas dans le dépôt public : il embarque une base de cartes) ; sinon l'outil simple, avec les trois portées | ✅ |

## 7. Entraîner et Tournoi

| # | Arsenal JS | fabbot aujourd'hui | État |
|---|---|---|---|
| T1 | Page Entraîner = trois volets illustrés (Tournoi des bots, Atelier, Entraîner le bot sur un deck « bientôt ») | ✅ trois volets illustrés : Tournoi des bots (#tournoi), Faire apprendre les bots (#apprentissage-bots, l'équivalent fabbot de l'atelier), Entraîner le bot sur un deck (bientôt) | ✅ |
| T2 | Tournoi : decks en pastilles avec l'avatar du héros, parties par matchup, Lancer / Pause / Remettre à zéro, « En direct » (parties qui défilent), matrice, classement, historique par version du moteur, Exporter (.md) | 🟡 lot 8 + portraits des héros dans les pastilles ; pas de « En direct » (parties qui défilent) | 🟡 |

## Ordre de travail proposé

1. Écran de partie (P1 à P33) : c'est là que vous jouez.
2. Écran Jouer + Ajuster (J1 à J10, A1 à A8).
3. Éditeur (E1 à E15).
4. Accueil (H1, H3, H4), Entraîner (T1, T2), Imprimer (I1).

Chaque lot : une pull request, vérifiée dans l'appli lancée avec des captures,
et la liste de ce qui est repris et de ce qui ne l'est pas.
