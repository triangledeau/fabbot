# Suivi du projet

Ce fichier est **le point de départ de chaque nouvelle conversation**. Il dit
où en est le projet, ce qui reste en suspens et quoi faire ensuite. Claude le
met à jour à la fin de chaque tâche.

## Où en est-on

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles + système d'effets | ✅ (manques connus listés dans `docs/rules/README.md`) |
| 2 | Deux premiers decks complets + interface texte | ✅ (cartes à valider par le propriétaire avec l'interface) |
| 3 | Les 7 autres decks + simulateur rapide | ✅ (Ira, Bravo, Iyslander, Florian, Levia, Dromai, Arakni ; cartes à valider par le propriétaire) |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |

**Branche la plus récente :** `claude/dreamy-gates-r3giw6` (deck Arakni,
partie de `main` après la fusion du deck Dromai). Tant qu'elle n'est pas
fusionnée dans `main`, partir de cette branche.

**Jouer contre le bot :** `cargo run --release -p fab-ui` (mode d'emploi :
`docs/interface.md`).

**Chiffres actuels :** 721 tests au vert. Simulateur, bots **aléatoires**
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

**Choix provisoire d'équipement et de deck** (`real::default_loadout`, en
attendant la phase de side de l'étape 6) : la première carte de chaque
emplacement d'arène dans l'ordre de la liste (Hala : Helm of Unity, Edge Laden
Plate, Gauntlets of Unity, Pillar of Unity ; Dorinthea : Helm of Unity,
Plating of Unity, Gauntlets of Unity, Refraction Bolters), et les 40 premières
cartes de la liste. Les mains d'armes sont respectées (CR 8.2.2b) : Florian
prend Nebula Blade (2H) et donc aucun off-hand (Annals of Sutcliffe, listée
« Off-Hand », est en fait une arme 2H). Une arme à une main listée en 2
exemplaires est équipée deux fois si les mains le permettent (Arakni : deux
Mark of the Huntsman).

## Questions ouvertes pour le propriétaire

Aucune question bloquante. Huit choix faits pour Arakni, à confirmer :

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

**Pour Claude (étape 4) :** bot **B1** (heuristique simple, sans recherche)
et classement **Elo** contre B0 (aléatoire), mesuré par le simulateur sur les
9 decks ; puis B2 (ISMCTS). Un bot n'a accès qu'à l'information de son joueur.

## Journal des sessions

| Date | Travail | Branche |
|---|---|---|
| 2026-11-22 | Étape 0 : squelette, règles Silver Age officielles, 9 listes de decks, source officielle des cartes | `claude/squelette-projet`, `claude/listes-decks` |
| 2026-11-22 | Étape 1 : cœur des règles (tour, priorité, combat), puis système d'effets suspendable | `claude/etape-1-regles`, `claude/etape-1-suite` |
| 2026-11-22 | Fin de l'étape 1 : arcane barrier / ward / spellvoid, remplacements, attaques contre alliés et spectra, perte des capacités, durées « prochain tour », étape de fermeture avec déclenchements | `claude/etape-1-moteur` |
| 2026-11-22 | Étape 2 : toutes les cartes de Hala et Dorinthea (55 lignes de STATUS.md + token Flurry) codées et testées (73 tests de cartes) ; moteur : sharpen, effets « prochaine attaque de cette arme », coût d'activation réduit, attaques supplémentaires (CR 5.2.3), paiement « unless you pay », renvoi d'une carte qui défend, fin de la marque (CR 9.3.3) ; `tools/check_coded_cards.py` ; `fab-sim --vrais` | `claude/quirky-ritchie-a0y1et` |
| 2026-11-22 | Fin de l'étape 2 : interface texte `fab-ui` (état visible du joueur, options numérotées, annulation illimitée, texte officiel des cartes avec `t`, journal des événements) ; moteur : `GameState::record` (journal complet facultatif), libellé des questions des cartes (`Ask::label`) ; `docs/interface.md` | `claude/fervent-ride-8osack` |
| 2026-11-22 | Étape 3, deck Ira : 27 lignes de STATUS.md + Crouching Tiger codées et testées (45 tests Ninja, 12 tests génériques, 8 tests de règles) ; moteur : jouer depuis la zone de bannissement (permission), cartes créées dans le bannissement, ephemeral, ambush, capacités déclenchées accordées (`CardDef::granted`), effets « this combat chain » sur la prochaine attaque, modes proposés seulement s'ils ont une cible ; `fab-sim --decks a b` | `claude/intelligent-mayer-kvrq5b` |
| 2026-11-22 | Réponses aux questions 1 et 3 à 14 : commentaires des règles confirmées, correction de Shove Off (épée partie du maillon), cartes bannies face cachée visibles par leur propriétaire dans l'interface, test Legacy of Ikaru sur une 2e attaque d'Edge of Autumn | `claude/intelligent-mayer-kvrq5b` |
| 2026-11-22 | Liste officielle Silver Age (40 cartes bannies, 4 héros mis sur la touche) dans `config/silver_age.toml`, noms français traduits par le fichier officiel ; aucune carte de nos decks concernée | `claude/intelligent-mayer-kvrq5b` |
| 2026-11-22 | Étape 3, deck Bravo : 35 lignes de STATUS.md + tokens Seismic Surge, Might, Vigor, Quicken codés et testés (61 tests de cartes, 7 tests de règles) ; moteur : clash (CR 8.5.45), « win a clash revealing this » depuis le deck, crush (CR 8.4.2, sur les dégâts), heave (capacité cachée depuis la main), suspense (CR 8.3.42), réductions « your next … costs {r} less », « can't gain {p} » (Chokeslam), « loses and can't gain dominate » ; `fab-ui --deck bravo` | `claude/wizardly-pasteur-r91h2w` |
| 2026-11-22 | Étape 3, deck Iyslander : 25 lignes de STATUS.md + tokens Frostbite et Ponder codés et testés (67 tests de cartes, 8 tests de règles) ; moteur : jouer une action « as though it were an instant » (CR 8.1.1d), surcoût Frostbite (CR 5.1.6a), coût « XX », amp (CR 8.5.47), tap (CR 8.5.55), « the next card you play … plus N » (`Replacement::bind`), zones exposées et token créé dans une zone d'équipement (CR 3.0.1a, 8.5.2c), prévention statique (Pyroglyphic Protection) ; `fab-ui --deck iyslander` | `claude/gallant-thompson-jslxg7` |
| 2026-11-23 | Étape 3, deck Levia : 34 lignes de STATUS.md + tokens Agility, Gate to i'Arathael et Blasmophet, the Insatiable Hunger codés et testés (76 tests de cartes, 7 tests de règles) ; moteur : blood debt (CR 8.3.11), qui a banni une carte et puissance imprimée hors de l'arène, cibles dans la zone de bannissement, permission « once per turn » de jouer depuis le bannissement (CR 5.1.3d), déclenchements identiques regroupés pour l'ordre, points d'action seulement en phase d'action (CR 1.13.2b) ; `fab-ui --deck levia` | `claude/nifty-pasteur-apci3j` |
| 2026-11-23 | Étape 3, deck Florian : 32 lignes de STATUS.md + tokens Runechant et Embodiment of Earth codés et testés (64 tests de cartes, 9 tests de règles) ; moteur : cartes doubles et meld (CR 9.2, 8.3.38, 5.3.4d), cibles dans un cimetière (CR 1.8.5a), créateur d'un token (CR 8.5.2c), supertypes « face-up » (Colors of Aria), compteurs à l'entrée, « pitched this way » (`Ctx::log_start`), decompose (CR 8.4.14) ; choix provisoire d'équipement respectant les mains ; `fab-ui --deck florian` | `claude/epic-faraday-pg52sk` |
| 2026-11-23 | Étape 3, deck Dromai : 30 lignes de STATUS.md + 8 dragons (faces arrière) + tokens Ash, Aether Ashwing, Spectral Shield codés et testés (70 tests de cartes, 16 tests de règles) ; moteur : cartes sous un permanent et material (CR 3.0.14, 8.4.5), transform (CR 8.5.36), invocations et cartes retournables (CR 8.2.13, 9.1.3), phantasm (CR 8.3.13), fermeture de la chaîne quand l'attaque disparaît (CR 7.7.2c), alliés qui attaquent (capacité accordée par Storm of Sandikai), vie des alliés remise à zéro en fin de tour (CR 4.4.3a) et compteurs de vie, prévention propre à un allié (Yendurai), steal / gain control, Themai, opt 1 ; interface : alliés, cartes dessous, texte des dragons ; `fab-ui --deck dromai` | `ccr-9b06529e-h1bljg` |
| 2026-11-23 | Étape 3, deck Arakni (fin de l'étape 3) : 28 lignes de STATUS.md + 6 Agents of Chaos + tokens Graphene Chelicera et Bloodrot Pox codés et testés (76 tests de cartes, 11 tests de règles) ; moteur : stealth, marque lue au moment de la touche (`Hit::marked`), dague qui touche par un effet, équiper un token arme, retrieve, become / return to the brood (CR 8.5.25, 8.5.52), traits (CR 2.13), effets sur toutes les attaques du tour, « the next time you attack », go again d'une carte qui quitte la pile (CR 5.3.5a), blocage élargi (CR 4.5.4d) ; deux armes à une main dans le choix provisoire ; `fab-ui --deck arakni` | `claude/dreamy-gates-r3giw6` |
