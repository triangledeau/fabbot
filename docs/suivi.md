# Suivi du projet

Ce fichier est **le point de départ de chaque nouvelle conversation**. Il dit
où en est le projet, ce qui reste en suspens et quoi faire ensuite. L'agent
qui finit une tâche le met à jour (seulement les lignes concernées).
**Plusieurs agents** (sessions Claude, Antigravity) peuvent travailler en
même temps : règles dans `AGENTS.md`, qui fait quoi dans `docs/taches.md`.

## Où en est-on

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles + système d'effets | ✅ (manques connus listés dans `docs/rules/README.md`) |
| 2 | Deux premiers decks complets + interface texte | ✅ (cartes à valider par le propriétaire avec l'interface) |
| 3 | Les 7 autres decks + simulateur rapide | ✅ (Ira, Bravo, Iyslander, Florian, Levia, Dromai, Arakni ; cartes à valider par le propriétaire) |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ✅ B0 = 0, B1 = +511, B2 = +785 |
| 5 | Interface web locale + annulation | ✅ (`fab-web`, à essayer par le propriétaire) |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ✅ bots (choix appris, +96 Elo) et page web (écran avant chaque manche) ; à essayer par le propriétaire |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ✅ B3 (3e génération) = B2 + 53 Elo (+34 à +73) ; procédure : `docs/nouveau-deck.md` |
| 8 | Rapatriement de l'Arsenal JS dans la page web (plan : `docs/rapatriement-arsenal.md`) | 🔄 lots 0 (accueil Arsenal, page en plusieurs fichiers, navigation par #), 1 (images des cartes en VF, noms français), 2 (plateau animé, tapis de jeu), 3 (sauvegarde et reprise), 4a (decks maison, import FaBrary), 4b (éditeur de deck) et 4c (ajustement avant la manche) faits ; avancement lot par lot dans le plan |

**Branche de départ :** `main` (depuis le 2026-11-26, Claude crée et
fusionne lui-même la pull request à la fin de chaque tâche ; l'étape 7 est
fusionnée).

**Jouer dans le navigateur :** double-cliquer sur **`Jouer.bat`** (mise à
jour, textes des cartes, compilation et ouverture du navigateur en un
clic ; plateau, clic sur les cartes, bouton
« Annuler », texte officiel au survol, journal ; bot B1 par défaut,
`--niveau b2` ou bouton « Nouveau match » pour changer). **Étape 6 :** un
match se joue en plusieurs manches (2 manches gagnantes par défaut) et,
avant chaque manche, un écran fait choisir qui commence (si c'est à vous),
votre équipement, puis vos 40 cartes parmi les 55. Mode d'emploi :
`docs/interface.md`.

**Jouer contre le bot dans le terminal :** `cargo run --release -p fab-ui` (mode d'emploi :
`docs/interface.md`) ; `--niveau b1` pour le bot heuristique, `--niveau b2`
pour le bot qui cherche (une fraction de seconde par décision ; `b2-1000` ou
`b2-500ms` pour le rendre plus fort).

**Bots et Elo :** `docs/bots.md` (règles de B1, tournoi, résultats).
Tournoi : `cargo run --release -p fab-sim -- --tournoi --bots b1 b0 --games 1000 --check`.

**Étape 7 :** **B3** = la recherche de B2 avec une estimation **apprise**
par auto-jeu (poids lisibles dans `data/b3/evaluation.txt`, appris sur
des parties de B3 contre B3 ; **3e génération** depuis le 2026-11-26,
avec le contenu de la main) : **B3 contre B2 : 57,6 % sur 1 296 parties
(+53 Elo, intervalle +34 à +73), 0 erreur** ; contre la 2e génération :
52,0 % sur 2 592 parties (+14 Elo, +1 à +28) ; `--niveau b3` dans
`fab-ui`, menu « niveau » de la page web. Gain modeste : détail et pistes
dans `docs/bots.md`. **Ajouter un deck** : `docs/nouveau-deck.md` (liste
dans `data/decks/en_cours/`, `verifier-deck`, cartes, puis le fichier passe
dans `data/decks/` et il est pris en compte partout ; tables des bots à
refaire).

**Chiffres actuels :** 797 tests au vert. **B2 contre B1 : 83,9 % sur les
81 paires des 9 decks (1 620 parties, 0 erreur) → +287 Elo** ; B2 contre
B0 : 98,8 %. Classement commun : **B0 = 0, B1 = +511, B2 = +796**. B2
imagine la main adverse dans le **card-pool de 55** (pas les 40 choisies).
Il ne gagne que 37 % avec Levia (blood debt toujours mal compris) et 74 %
avec Arakni, 82 à 99 % avec les autres decks. Détail et réglages essayés :
`docs/bots.md`.

Étape 4, B1 : **B1 contre B0 : 95,0 % sur les
81 paires des 9 decks (81 000 parties, 0 erreur) → B1 = +512 Elo**
(intervalle à 95 % : +506 à +517). B1 gagne 96 à 100 % avec 8 decks, mais
seulement 64 % avec Levia : il perd plus de 12 PV par partie à cause du
blood debt (B1 ne lit pas le texte des cartes). Détail : `docs/bots.md`.

Chiffres de l'étape 3 : Simulateur, bots **aléatoires**
(ces pourcentages ne disent rien des vrais matchups) : Arakni contre chacun
des 8 autres decks et en miroir, 100 000 parties vérifiées (`--check`) par
paire, **0 erreur** ; ≈ 7 000 à 17 000 parties/s dans le conteneur de Claude
(4 cœurs). Arakni gagne 21 % contre Ira, 49 % contre Hala, 54 % contre
Dorinthea, 52 % contre Bravo, 99 % contre Iyslander, 51 % contre Florian,
98 % contre Levia, 7 % contre Dromai. Nouveau : le simulateur compte à part
les nulles par **blocage** (CR 4.5.4d) ; elles étaient comptées avec « 2
héros à 0 ». Détection élargie : decks et mains vides et aucune vie qui
change pendant 6 tours (avant, ces parties allaient jusqu'à la limite de 200
tours, surtout avec Arakni et Dromai dont le héros a des capacités
déclenchées). Avec les bots aléatoires, 3 à 11 % des parties d'Arakni
finissent ainsi (une carte injouable reste en arsenal).

**Phase de side des bots (étape 6)** : chaque bot choisit son équipement
et ses 40 cartes avec une **table apprise** par simulation
(`data/side/b1.txt`, une entrée par paire de héros, lisible ; méthode et
mesures : `docs/bots.md`, « La phase de side »). **B1 avec la table contre
B1 avec le choix « liste » : 63,5 % sur 324 000 parties (+96 Elo, 0
erreur).** **B2 avec la table contre B2 « liste » : 62,0 % sur 1 620 parties (+85 Elo, intervalle +68 à +103).** Les tournois du simulateur utilisent la table par
défaut (`--side liste liste` pour l'ancien choix).

**Choix « liste »** (`real::default_loadout`, ancien choix provisoire,
toujours utilisé par les tests et par l'humain dans le terminal) : la première carte de chaque
emplacement d'arène dans l'ordre de la liste (Hala : Helm of Unity, Edge Laden
Plate, Gauntlets of Unity, Pillar of Unity ; Dorinthea : Helm of Unity,
Plating of Unity, Gauntlets of Unity, Refraction Bolters), et les 40 premières
cartes de la liste. Les mains d'armes sont respectées (CR 8.2.2b) : Florian
prend Nebula Blade (2H) et donc aucun off-hand (Annals of Sutcliffe, listée
« Off-Hand », est en fait une arme 2H). Une arme à une main listée en 2
exemplaires est équipée deux fois si les mains le permettent (Arakni : deux
Mark of the Huntsman).

## Questions ouvertes pour le propriétaire

**Decks maison (Arsenal, lot 4a) :** la vérification d'un deck importé ne
contrôle pas encore que chaque carte est de la classe (ou du talent) du
héros. `TODO(règles)` : il faut la règle exacte de construction (CR,
section sur le card-pool), que la session cloud n'a pas pu lire
(rules.fabtcg.com renvoie 403). Pouvez-vous copier ce passage, ou
`bash tools/fetch_rules.sh` sur votre PC ? D'ici là, l'éditeur a seulement
un filtre « pour ce héros » (classe, talent ou Generic), sans valeur de
règle.

**Étape 6, à regarder :** le fichier `data/side/b1.txt` (choix appris des
bots, paire par paire, avec l'écart estimé de chaque carte laissée hors du
deck). Ces choix reflètent ce qui réussit **à B1** : par exemple B1 retire
souvent Arcane Polarity, Reinforce the Line ou Oasis Respite, sans doute
parce qu'il les joue mal. Si un choix vous paraît absurde pour un humain,
dites-le : c'est une piste pour améliorer B1. Une question de règle :
CR 4.1.4a dit « up to one arena-card for each of their … weapon zones » ;
le moteur accepte deux armes à une main (Arakni : deux Mark of the
Huntsman), comme le choix provisoire le faisait déjà. Est-ce bien ça ?

Aucune question bloquante. Bots : par défaut, `fab-ui` garde le bot B0
(au hasard), plus utile pour vérifier les cartes ; `--niveau b1` / `b2`
donnent les bots plus forts. Préférez-vous B2 par défaut ?

**B2 et l'information cachée** (réponse du 2026-11-23, appliquée) : B2
imagine les cartes cachées de l'adversaire dans son card-pool de 55, moins
les cartes déjà vues. Plus tard : pondérer par la fréquence des cartes dans
les decks du méta.

1. **Agents of Chaos** : « a random Agent of Chaos » tire au hasard (chances
   égales) parmi les **six** demi-héros du trait (Black Widow, Funnel Web,
   Orb-Weaver, Redback, Tarantula, Trap-Door), même s'ils ne sont pas dans le
   card-pool de 55 (CR 2.13.4). Le total de vie ne change pas.
2. **Mark of the Huntsman** + Bite / Hurl / Danger Digits : la dague a
   touché puis elle est détruite ; sa capacité « When this hits a hero, you
   may choose to destroy this and mark them » se résout ensuite. Le moteur
   laisse quand même **marquer** le héros (« destroy this » échoue). Est-ce
   bien ça ? (`TODO(règles)` dans le code.)
3. **« When this hits a marked hero »** (Mark of the Black Widow / Funnel
   Web) et le go again d'**Arakni** : la marque disparaît pendant la touche
   (CR 9.3.3) ; le moteur regarde si le héros était marqué **juste avant**
   la touche, donc ces effets fonctionnent.
4. **Bite** (« you may have target dagger … ») : le « you may » est le choix
   de la cible ; sans cible, rien ne se passe.
5. **Concoct Disorder** : un héros dont l'arsenal est déjà occupé (ou le
   deck vide) ne met rien en arsenal ; seules les cartes vraiment mises
   comptent pour le go again.
6. **Public Bounty** : « the next time you attack a marked hero » s'applique
   quand l'attaque est déclarée (cible connue) ; si la prochaine attaque vise
   un héros non marqué, l'effet attend une attaque suivante du tour.
7. **Arakni, Trap-Door** : la carte trouvée est bannie face cachée ; si c'est
   un piège (Lair of the Spider), elle peut être jouée pendant le reste de ce
   tour et pendant le tour de l'adversaire.
8. **Double Trouble** : compte les réactions d'attaque **jouées ou
   activées par vous** sur le maillon de Double Trouble (cartes, capacités des
   Agents, Danger Digits, Prey Spotters, Stalker's Steps).

Sept choix faits pour Dromai, à confirmer :

1. **Phantasm sur une attaque de dragon** : CR 8.3.13 dit « destroy this ».
   Pour une attaque d'allié (attaque-proxy), le moteur détruit **l'allié**
   (le dragon va au cimetière, son Ash disparaît, la chaîne se ferme si
   c'était avant les dégâts, CR 7.7.2c). Est-ce bien ça ?
2. **Aether Ashwing** (Arcane Barrier 1) et **Sand Cover** (ward 4 sur une
   Ash) : « you » est le contrôleur du permanent (CR 1.1.2b) ; ils protègent
   donc le **héros**, pas l'allié lui-même. Une Ash avec ward est détruite
   pour prévenir des dégâts faits à votre héros.
3. **Invoke X** dont l'Ash ciblée a disparu : la carte entre quand même dans
   l'arène, face dragon, sans Ash dessous (donc sans phantasm) (CR 5.3.6a,
   8.2.13a). Une Invoke ne peut pas être jouée sans Ash à cibler.
4. **Miragai / Passing Mirage** : « your first dragon (Illusionist) attack
   each turn » = la première attaque de ce type du tour, même si elle a eu
   lieu avant que Miragai / Passing Mirage soit en jeu (alors aucune autre
   attaque n'en profite ce tour).
5. **Cromai** (« Once per turn, when Cromai attacks or leaves the arena ») :
   s'il a déjà attaqué ce tour, il ne redonne pas de point d'action en
   quittant l'arène le même tour.
6. **Oasis Respite** : « a source of your choice » est choisie à la
   résolution parmi les objets publics de l'arène et de la pile (des deux
   joueurs).
7. **Plusieurs wards sur les mêmes dégâts** : si détruire l'un d'eux
   déclenche une capacité (Silken Shroud, Wave of Reality, Essence of
   Ancestry: Body), le joueur choisit l'ordre, même si tous seront utilisés
   (pour Essence of Ancestry, l'ordre décide si une autre aura Illusionist
   est encore là quand elle quitte l'arène).

Six choix faits pour Levia, à confirmer :

1. **Puissance « 6 or more » hors de l'arène** : pour une carte de la main,
   du deck, du cimetière ou du pitch (défaussée, bannie, pitchée), on lit la
   puissance **imprimée** (CR 1.7.4 : ses capacités statiques ne
   fonctionnent pas hors de l'arène). Ex. un Feasting Shadowbeast jaune (5)
   n'est pas une carte de 6 quand on le bannit, même si un 6 a déjà été
   banni ce tour. Dans l'arène (Hell Hammer, carte sur la chaîne), on lit la
   puissance actuelle.
2. **« You've banished » / « put into your banished zone »** : le premier
   compte les cartes que **vous** avez bannies (Feasting Shadowbeast, Feeding
   Frenzy, Goremass Summoning, Blasmophet), le second les cartes arrivées
   dans **votre** zone de bannissement, quel que soit le responsable (Levia,
   Graveling Growl, Hooves). Une carte bannie « au lieu d'aller au
   cimetière » (Embraforged Gauntlet, Engulfing Shadows) est bannie par son
   propriétaire.
3. **Hex Gauntlet** peut se retourner lui-même face cachée (il est déjà dans
   la zone de bannissement quand sa capacité se résout).
4. **Blasmophet, the Insatiable Hunger** n'a pas de capacité « Attack » dans
   le texte officiel : il ne peut pas attaquer (mais on peut l'attaquer).
   Unique (CR 8.3.48a) : si Goremass Summoning en crée un 2e, le joueur
   choisit celui qu'il garde.
5. **Gate to i'Arathael + Blasmophet** (CR 5.1.3d) : une carte qui a une
   permission « this turn » (Gate) l'utilise ; la permission « once per
   turn » de Blasmophet reste disponible pour une autre carte.
6. **Countdown to Extinction** : la question « chercher un Darkest Hour ? »
   est posée à chaque touche, même sans Darkest Hour dans le deck (aucun de
   nos decks n'en a) ; si oui, le deck est mélangé.

Correction de règle au passage : on ne gagne plus de point d'action hors de
sa phase d'action (CR 1.13.2b), ex. Hooves détruit pendant la phase de fin.

Cinq choix faits pour Florian, à confirmer :

1. **Meld** (Arcane Seeds // Life) : jouée fusionnée, la carte a les types
   Action et Instant (CR 8.3.38c). Le moteur la traite comme une **action**
   (pile vide, pendant votre tour, 1 point d'action, CR 8.1.1). Pendant le
   tour adverse, seul le côté Life est proposé. Est-ce bien ça ?
2. **Florian** : « If you would create … aura tokens » ne concerne que les
   tokens que **Florian crée** lui-même : un Frostbite créé chez lui par
   Iyslander (CR 8.5.2c, contrôlé par Florian) n'est pas doublé.
3. **Cartes Earth bannies face cachée** : elles comptent pour Florian, Helm
   of Lignum Vitae et Well Grounded (on lit leur texte imprimé). Colors of
   Aria n'est Earth que face visible. Faut-il plutôt ne compter que les
   cartes face visible ?
4. **Annals of Sutcliffe** : « pitched this way » = seulement les cartes
   pitchées pour payer cette activation (des ressources restées d'un
   paiement précédent ne comptent pas).
5. **Sigil of Silphidae** : la cible (« target hero ») est choisie quand la
   capacité se déclenche, même s'il n'y a aucune autre aura à bannir ; le
   choix de bannir se fait à la résolution.

Trois choix faits pour Iyslander, à confirmer :

1. **Frost Spike** : « Create a Frostbite token in an exposed head, chest,
   arms, or legs zone ». Le joueur de Frost Spike choisit à la résolution
   (ce n'est pas une cible) **n'importe quelle** zone exposée, la sienne ou
   celle de l'adversaire ; le Frostbite entre sous le contrôle du joueur à
   qui appartient la zone (CR 8.5.2c) et occupe la zone jusqu'à sa
   destruction. Est-ce bien ça ?
2. **« As though it were an instant »** (Iyslander depuis l'arsenal, Stir the
   Aetherwinds) : le moteur utilise toujours la permission quand elle existe
   (elle évite seulement de dépenser un point d'action). Pour Stir, la
   permission et le bonus de dégâts sont « consommés » par la prochaine
   action Wizard non-attaque jouée, qu'elle ait été jouée comme instant ou non.
3. **Ice Eternal** : la fusion (révéler une carte Ice) est demandée après le
   paiement des ressources (CR 5.1.9 : les coûts d'effet se paient après) ;
   seules les cartes Ice encore en main peuvent être révélées.

Deux choix faits pour Bravo, à confirmer en jouant :

1. **Heave** (Thunder Quake) : CR 5.4.7b dit que le joueur décide de
   déclencher la capacité cachée, puis la couche se résout. Le moteur pose une
   seule question (« payer 3 ? ») à la résolution, et ne déclenche heave que
   si c'est possible (arsenal vide, 3 ressources sans pitcher la carte
   elle-même). Si vous refusez, l'adversaire a vu passer la carte.
2. **Bravo** : sa capacité n'est proposée que s'il y a une carte face cachée
   dans l'arsenal (l'activer sans carte serait légal, mais ne ferait rien
   d'autre que coûter 2 ressources).

Questions réglées le 2026-11-22 (réponses du propriétaire, citées en
commentaire dans le code) : action non-attaque à l'étape de résolution
(interdite, CR 7.0.1a), détruit puis banni (reste « détruit », CR 1.9.3a),
ward sans dégâts restants (CR 1.9.1b), Steelblade Shunt, Decimator Great Axe,
Flurry + Dorinthea, On the Horizon, cartes bannies face cachée (visibles par
leur propriétaire, CR 3.0.3a), combo « last attack », Crouching Tiger (une
carte, pas un token), Blessing of Qi (CR 6.2.2a), Aspect of Tiger: Body.
Liste officielle Silver Age (bannies, héros mis sur la touche) copiée dans
`config/silver_age.toml` : aucune carte de nos decks n'est concernée.
Correction : **Shove Off** ne peut plus viser une carte d'un maillon
précédent si l'épée a quitté ce maillon (notes de Mastery Pack Warrior).

## Prochaine tâche

**Pour le propriétaire :** jouer quelques parties avec
`cargo run --release -p fab-ui`, vérifier les cartes de Hala et Dorinthea
(commande `t <n>` pour le texte officiel) et passer celles qui sont justes en
`Validated` dans `docs/cards/STATUS.md` ; signaler tout comportement faux.

Vérifier aussi les cartes d'Ira (`cargo run --release -p fab-ui -- --deck ira`),
de Bravo (`--deck bravo`), d'Iyslander (`--deck iyslander`), de Florian
(`--deck florian`), de Levia (`--deck levia`), de Dromai (`--deck dromai`)
et d'Arakni (`--deck arakni`) ; `t <n>` donne le texte officiel (pour un
dragon, celui de sa face arrière).

Essayer aussi le bot B1 (`--niveau b1`) et dire s'il fait des choix
absurdes à corriger simplement.

Essayer aussi le bot B2 (`--niveau b2`) et noter ses erreurs flagrantes.

**Pour le propriétaire (étape 5) :** essayer l'interface web
(double-cliquer sur `Jouer.bat`) et dire ce qui gêne : lisibilité du plateau, informations manquantes,
clics peu pratiques. C'est désormais le moyen le plus simple de vérifier
les cartes.

**Pour le propriétaire (étape 6) :** jouer un match dans la page web
(`Jouer.bat`) et essayer l'écran de choix avant chaque manche ; dire ce
qui gêne. Relire `data/side/b1.txt`.

**Pour le propriétaire (étape 7) :** essayer B3 (`--niveau b3`, ou le
menu de la page web) ; relire `docs/nouveau-deck.md` et dire si un 10e deck
est voulu (donner sa liste).

**Pour le propriétaire (étape 8) :** ouvrir `Jouer.bat` : la page
commence par l'accueil Arsenal ; dire si l'habillage de la partie convient.

**Pour Claude (étape 8) :** suivre `docs/rapatriement-arsenal.md`, lot par
lot dans l'ordre validé (colonne « Avancement ») ; prochain lot : 5
(impression).

**Pour Claude :** l'étape 7 est finie ; il n'y a plus d'étape prévue.
Prochaine tâche proposée (B3 3e génération faite le 2026-11-26 : les
gains diminuent, pistes dans `docs/bots.md`) : **améliorer B2/B3 avec
Levia** (B3 ne gagne que 12 % avec Levia contre B2 : blood debt), ou **un 10e deck** si le propriétaire en donne la liste (suivre
`docs/nouveau-deck.md`). Pistes pour la phase de side : apprendre avec B2
plutôt que B1, tenir compte du premier joueur et de l'arène adverse
(variables croisées), refaire l'apprentissage contre l'adversaire qui
utilise lui-même la table. Pistes d'amélioration de B2, à mesurer plus tard : une estimation qui compte le
blood debt et les cartes bannies (Levia), garder l'arbre d'une décision à la
suivante, aller plus vite (plus d'itérations pour le même temps), pondérer
les cartes imaginées par leur fréquence dans les decks du méta.

## Journal des sessions

Une session = un fichier `docs/journal/AAAA-MM-JJ-<sujet>.md` (voir
`AGENTS.md`, « Travailler à plusieurs »). Historique jusqu'au 2026-11-26 :
`docs/journal/historique.md`.
