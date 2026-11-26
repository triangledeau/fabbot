# Jouer contre le bot

Deux interfaces, avec les mêmes règles, la même annulation et le même
principe : vous ne voyez que ce que votre joueur a le droit de voir.

## Dans le navigateur (recommandé)

**Double-cliquez sur `Jouer.bat`** à la racine du projet. Il :

1. met le jeu à jour (`git pull`) ;
2. télécharge les textes officiels des cartes s'ils manquent ;
3. prépare le jeu (quelques minutes la première fois et après une mise à
   jour, quelques secondes ensuite) ;
4. ouvre la page du jeu dans votre navigateur.

La fenêtre noire doit rester ouverte pendant la partie ; la fermer arrête
le jeu. Le serveur n'écoute que sur votre ordinateur.

Les decks, le niveau du bot (B1 par défaut), le nombre de manches à gagner
(2 par défaut : au meilleur des 3) et la graine se choisissent dans la page,
avec le bouton **Nouveau match**.

Pour les curieux, la commande équivalente (options : `--deck`, `--bot`,
`--niveau`, `--graine`, `--manches`, `--port`, `--sans-navigateur`) :

```
cargo run --release -p fab-ui --bin fab-web -- --deck levia --bot arakni --niveau b2
```

Si le port 8080 est déjà pris, le jeu prend le suivant libre (8081…).

### Avant chaque manche : équipement et deck

Comme en tournoi (CR 4.1.3 à 4.1.6), avant chaque manche, une fenêtre
d'ajustement (reprise de l'Arsenal JS) occupe la place du tapis :

- **En-tête** : « Hala contre Dorinthea », la pastille « tiré au sort » et
  le bouton **Retirer au sort** si le deck du bot a été tiré au hasard
  (première manche seulement : le match est refait avec un autre deck),
  **Ce que joue le bot ▾** (son héros, ses armes et les équipements de son
  card-pool ; pas son choix, qu'il fait sans voir le vôtre) et le point de
  départ (voir plus bas).
- **Qui commence ?** Le joueur désigné choisit : à la première manche, il
  est tiré au sort ; ensuite, c'est le perdant de la manche précédente. Le
  bot choisit toujours de commencer.
- **Le deck** : chaque carte est une **pile** de ses exemplaires : clic (ou
  Entrée, +) = un exemplaire de plus dans le deck, clic droit (ou Suppr, −)
  = un de moins ; les exemplaires hors du deck sont grisés, un message dit
  quand ce n'est pas possible. Le deck se règle dès l'ouverture et le reste
  jusqu'au lancement.
- **L'équipement** : une rangée par zone, la carte choisie encadrée ; une
  carte par zone (cliquer à nouveau la retire), deux mains au plus pour les
  armes. **Valider l'équipement** le présente au bot (qui a choisi le sien
  sans voir le vôtre) ; l'équipement du bot s'affiche alors à côté du
  vôtre, et **Lancer la partie** s'active quand le compteur indique
  **40 / 40**.
- **Pied de fenêtre** : le compteur, **Rythme** (Normal / Rapide : le film
  des coups va deux fois plus vite ; aussi dans Options en partie), le
  niveau du bot (modifiable avant la première manche seulement : le match
  est refait) et **Retour** (écran Jouer ; le match est gardé).
- Les cartes prennent la plus grande taille qui laisse tout tenir sans
  défiler ; le panneau de droite montre la carte survolée en grand.

Point de départ (boutons « Départ ») : **Dernier réglage** (votre dernier
choix contre ce deck du bot, gardé par le navigateur ; c'est le départ par
défaut), **Plan conseillé** (le choix de la table apprise des bots pour
votre card-pool contre ce héros) ou **Proposition** (votre choix de la
manche précédente ; à la première manche, l'équipement et les 40 premières
cartes de la liste).
Survolez une carte pour son texte officiel. Le bot choisit avec sa table
apprise (`docs/bots.md`, « La phase de side »).

En fin de manche, le bouton **Manche suivante** lance la préparation de la
suivante ; le score du match est affiché en haut. Annuler un coup après la
fin d'une manche la fait reprendre (le résultat n'est plus compté).

### L'accueil Arsenal

La page s'ouvre sur l'**accueil Arsenal** (logo, trois volets) :

- chaque volet porte le portrait d'un héros des decks du bot (images
  chargées en ligne) ;
- **Jouer** : l'écran Jouer (choix des decks, éditeur, impression) ; en
  bas, le nombre de decks (dont les vôtres) et les niveaux du bot ;
- **Apprendre** : le didacticiel ;
- **Entraîner** : trois volets (tournoi des bots, apprentissage des bots,
  entraîner le bot sur un deck « bientôt ») ; en bas, le dernier tournoi
  (ou celui en cours) avec sa barre d'avancement ;
- le bandeau **Match en cours** (au-dessus des volets) donne le score et
  propose **Reprendre** ou **Nouveau match**.

Dans la partie, le bouton **Accueil** (en haut à gauche) revient à
l'accueil sans arrêter le match. Chaque écran a sa propre adresse
(`#accueil`, `#partie`, `#nouveau-match`) : le bouton Précédent du
navigateur marche, et on peut mettre un écran en favori.

Les polices d'Arsenal (Cinzel, Sofia Sans, IBM Plex Mono) sont chargées
depuis Internet ; sans connexion, la page prend des polices de secours et
marche pareil.

Pour les développeurs : la page est découpée en un dossier par volet dans
`crates/ui/web/` (`commun/` = charte et navigation, `accueil/`,
`partie/`), et chaque fichier est intégré au programme par
`crates/ui/src/fichiers.rs` (un fichier ajouté = une ligne là, un test le
vérifie).

### Apprendre (didacticiel des règles)

Le volet **Apprendre** de l'accueil (`#apprendre`) explique les règles pas à
pas, pour débuter de zéro : leçons courtes, scènes jouées sur le vrai moteur,
questions éclair, et un tableau de couverture des règles officielles. Voir
`docs/didacticiel.md`.

### Images des cartes

Les cartes s'affichent avec leur **image officielle**, en **version
française** quand elle existe (343 cartes sur 396), sinon en anglais ; la
fiche de la carte survolée montre la grande image et le nom français, puis
le texte officiel (anglais, avec errata). Une puissance ou une défense
changée en jeu, un compteur, « face cachée » s'affichent en pastilles sur
l'image. Les images viennent du serveur d'images de Legend Story Studios :
sans Internet, chaque carte garde son cadre dessiné.

Pour les développeurs : `data/cards/images.tsv` (produit par
`tools/images_cartes.py`, à relancer quand une carte est ajoutée) ;
`crates/ui/src/images.rs` ajoute `image` et `nom_fr` au JSON des cartes ;
`crates/ui/web/commun/cartes.js` construit l'adresse.

### L'écran Jouer (choix des decks, decks maison)

Depuis l'accueil, le volet **Jouer** (adresse `#decks`) reprend l'écran
Jouer de l'Arsenal JS :

- **Mes decks** (decks maison, `data/decks/maison/`) et **Decks du bot**
  (`data/decks/`) en tuiles, avec l'illustration du héros et des pastilles :
  « réglé pour le bot », « jouable contre le bot », « héros pas encore
  codé », « n cartes pas encore jouables », « n hors règles », taille du
  card-pool (« 55 / 55 »).
- **Clic** sur une tuile : la choisir ; **double-clic** : l'ouvrir dans
  l'éditeur ; **Entrée / Espace** au clavier : la choisir.
- **Clic droit** sur une tuile, ou son bouton **⋯** (utile au doigt) : Jouer
  avec ce deck, Le bot joue ce deck, Modifier, Dupliquer, Imprimer
  (proxies), Exporter la liste, Archiver. Flèches haut / bas dans le menu,
  Échap pour le fermer. Un deck du bot ne se modifie pas : « Modifier » en
  fait une copie (deck maison), l'original et ses réglages ne changent pas.
- Panneau **Partie contre le bot** : votre héros contre celui du bot (« ? »
  si au hasard), deck du bot (au hasard, un deck du bot ou un de vos decks
  jouables), niveau du bot, manches, graine (« Options avancées »), les
  raisons pour lesquelles un deck n'est pas jouable, **Ajuster et lancer**
  (ouvre la préparation de la manche) et **Charger une partie (.json)**.
- Barre du haut : **Nouveau deck** (adresse `#nouveau-deck` : grille des
  héros Young, filtres Tous / Jouables par le bot / par classe), **Importer
  une liste** (export texte de FaBrary ou format des fichiers du projet :
  Vérifier, puis Créer le deck), **Imprimer un deck** (menu « Imprimer quel
  deck ? »), **Decks archivés** (adresse `#archives` : Désarchiver, et
  Supprimer… pour un deck maison). Archiver affiche « Annuler » pendant
  quelques secondes. Les decks du bot archivés sont seulement cachés (dans
  ce navigateur).

La vérification d'un deck dit s'il respecte le format Silver Age (55 cartes
au plus, 40 de deck au moins, 2 exemplaires, cartes bannies, héros
« Young »), quelles cartes ne sont pas légales et lesquelles le bot ne sait
pas encore jouer (pas codées). Le deck est enregistré dans
`data/decks/maison/<nom>.txt` ; s'il est jouable, il se choisit pour vous
comme pour le bot.

**Éditeur de deck** (repris de l'Arsenal JS) : « Nouveau deck » (choisir
un héros Young), « Modifier » un deck maison, ou « Modifier » / « Dupliquer »
un deck du bot (copie : l'original ne change pas, un bandeau le rappelle).

- Barre du haut : ← Jouer, nom du deck (modifiable), état (« enregistré » ou
  « ● modifications non enregistrées »), Annuler, Importer (remplace la
  liste), Exporter (format FaBrary), Imprimer (ouvre Imprimer avec ce deck,
  une fois enregistré), Enregistrer (Ctrl+S).
- À gauche, la collection Silver Age : recherche (« / » pour y aller, en
  français ou en anglais), filtres Pitch, Coût, Type, Mots-clés, Classe /
  talent, Afficher (légales pour le héros, jouables par le bot, déjà dans
  le deck, jouables d'abord), tri, taille des cartes (mémorisée). Les cartes
  que le bot ne sait pas encore jouer sont grisées. Survol : grande image et
  texte, posés à côté de la carte.
- Clic : +1 au deck ; ⇧ clic : +1 au side ; Alt clic : +1 aux envies ;
  Entrée au clavier. **Clic droit** (ou touche Menu) sur une carte de la
  grille ou de la liste : ajouter, déplacer (1 ou tous), retirer, voir en
  grand, les autres couleurs. On peut aussi **glisser** une ligne vers une
  autre catégorie (⇧ en lâchant : tous les exemplaires). Ctrl+Z annule
  (60 pas) ; Échap ferme menu et aperçu.
- À droite : héros et compteurs Deck (40) / Side / Équipement / Envies,
  pitch et courbe de coût, puis trois onglets : **Liste** (catégories
  repliables Deck, Side, Équipement et armes, Liste d'envies), **Équilibrage**
  (pitch, courbe, rôles des cartes comparés à la moyenne des 9 decks du
  bot), **Vérifications** (règles de construction, plus l'avis du serveur).
- Dans le fichier enregistré, les 40 premières cartes de deck forment le
  deck de départ, la suite est le side (ligne `// side`). La liste d'envies
  n'est pas dans le fichier : elle reste dans le navigateur.

Un deck maison se range dans les **archives** (« Archiver ») ; on ne
supprime qu'un deck archivé. Le filtre « légales pour le héros » garde les
cartes que le héros peut avoir (même règle que la vérification ci-dessous).

La légalité vient de la base communautaire (`bash tools/fetch_cards.sh`) ;
sans elle, seuls le format et les cartes codées sont vérifiés. Avec elle,
la vérification contrôle aussi les classes et talents : le héros doit
avoir toutes les classes et tous les talents de chaque carte (Generic :
pour tous) ; pour une carte hybride (« Brute / Warrior »), un seul des deux
côtés suffit (réponse du propriétaire, 2026-11-26) ; « Essence of Ice »
(Iyslander), « Essence of Earth, Ice, and Lightning » (Bravo)… ajoutent
ces talents au héros.

Pour les développeurs : `crates/ui/src/api_decks.rs` (routes
`/api/collection`, `/api/decks-projet`, `/api/decks-maison`, `/api/deck-maison`,
`/api/deck-maison/verifier`, `/api/deck-maison/importer`),
`fab_cards::import` (lecture de l'export FaBrary), `fab_cards::decks`
(decks maison). Page : `crates/ui/web/decks/`.

### Imprimer des proxies

Depuis l'accueil, **Imprimer** : choisissez un deck (du projet ou maison),
le papier (A4 ou Letter), avec ou sans héros et équipement, traits de
coupe, espace entre les cartes. La page montre les feuilles (9 cartes de
63 × 88 mm par page, images officielles en VF quand elles existent) ;
**Imprimer…** ouvre l'impression du navigateur (ou « Enregistrer en
PDF »). Imprimez à 100 %, sans « ajuster à la page ». Page :
`crates/ui/web/impression/`.

### Aide du bot et signalement de bug

Dans l'écran de partie, sous les options : **Conseil** (ou touche C)
montre le coup que jouerait le bot choisi (B1, B2 ou B3) à votre place ;
il ne voit que ce que vous voyez. La barre **Vous** donne vos chances de
gagner selon l'évaluation apprise de B3 (sans regarder la main du bot :
moyenne sur des mains tirées au hasard). **Bot contre Bot** fait jouer ce
bot à votre place jusqu'à la fin de la manche (re-cliquez pour reprendre
la main ; « Annuler » marche toujours). **Signaler un bug** (en haut)
enregistre votre texte avec la sauvegarde du match dans
`sauvegardes/bugs/` : on rejoue la partie coup par coup.

### Entraîner (tournoi, SPRT, apprentissage)

Depuis l'accueil, **Entraîner** : trois volets, **Tournoi des bots**
(adresse `#tournoi`), **Faire apprendre les bots** (`#apprentissage-bots`)
et « Entraîner le bot sur un deck » (bientôt). « ← Entraîner » revient aux
volets. **Tournoi** : cochez les decks (du projet
et maison), choisissez le bot et la façon de choisir l'équipement (side)
de A et de B, le nombre de parties par paire de decks, et lancez. La
matrice se remplit pendant les parties (score de A en %, ligne = deck de
A, colonne = deck de B), avec le score total, l'écart d'Elo et son
intervalle à 95 %, et un classement des decks (utile quand A et B sont le
même bot). **Banc SPRT** : le tournoi s'arrête dès que le test tranche
entre « A n'a pas d'avance » et « A a au moins l'avance demandée »
(5 % d'erreur de chaque côté). Chaque tournoi fini va dans
`sauvegardes/tournois/` (liste « Historique », bouton Voir), et s'exporte
en JSON ou en CSV (matrice). **Apprentissage** : lance
`fab-sim --apprendre-side` ou `--apprendre-b3` (le `fab-sim` compilé à
côté de `fab-web` : `cargo build --release -p fab-sim`) et montre sa
sortie ; le fichier réécrit ne sert qu'après avoir recompilé. Page :
`crates/ui/web/entrainer/`, serveur : `crates/ui/src/api_tournoi.rs`.

### Tapis de jeu

Chaque joueur a son **tapis**, disposé comme le playmat officiel (repris de
l'Arsenal JS). De gauche à droite, pour vous : équipement (tête, torse,
jambes, et bras collé au torse), arme, **héros** (PV gravés dessus) avec
l'**arsenal** dessous, seconde arme ou off-hand avec les **permanents**
dessous, **pitch** collé au deck, puis cimetière, deck et banni. Le tapis du
bot est le même, tourné à 180° (vu d'en face). Cliquez le cimetière ou le
banni pour voir toutes leurs cartes. Ressources et points d'action sont à
côté du tapis ; votre main dessous.

### Plateau animé

Après chacun de vos choix, la page **rejoue les coups** un par un (votre
choix, puis les réponses du bot) au lieu de sauter directement au résultat :

- les cartes **volent** d'une zone à l'autre (main → chaîne, pitch,
  cimetière…) ; une carte jouée par le bot part de sa main ;
- une attaque qui touche **fonce sur le héros** : impact, secousse, dégâts
  en gros chiffres, PV qui **défilent** ; une attaque entièrement bloquée
  affiche « Bloqué » ;
- un **bandeau** annonce « Votre tour » / « Tour du bot » et chaque étape du
  combat ; le pitch envoie des étincelles vers les ressources ;
- **tirer une carte** surlignée : une chaîne de laiton la relie à la zone
  visée ; lâchez-la sur la chaîne de combat (jouer, défendre), sur votre
  pitch ou sur votre arsenal. Le clic marche toujours.

Dans le panneau de droite : **Animations** (tout couper) et **Pause avant
les dégâts** (le film s'arrête avant chaque coup porté ; « Continuer » ou
Espace). Le bouton **Passer l'animation** (ou Échap) va directement à la fin.
Ces réglages sont gardés par le navigateur. Si votre système demande moins
d'animations, elles sont coupées.

Pour les développeurs : le serveur ajoute à l'état un `film` (plateau après
chaque pas du moteur + événements `tour`, `joue`, `attaque`, `defense`,
`touche`, `bloque`, `degats`, `pitch`, `chaine_fermee`) et `film_no` ; chaque
option a une `action` (`jouer`, `pitch`, `defendre`, `arsenal`, `activer`,
`autre`). Page : `crates/ui/web/partie/finition.js`.

### Sauvegarde et reprise

Le match en cours est **sauvegardé tout seul** après chacun de vos choix,
dans `sauvegardes/en-cours.json` (dossier de fabbot). Si vous fermez la
fenêtre, le prochain lancement de `fab-web` **reprend le match au même
point** (manches gagnées, équipement choisi, coups joués). Pour repartir de
zéro : `fab-web --nouvelle-partie`, ou « Nouveau match » dans la page. Un
match terminé n'est pas repris.

Dans l'en-tête de la partie : **Exporter** télécharge la sauvegarde (un
fichier `.json`) ; **Importer** reprend un match depuis un tel fichier. Un
fichier abîmé ou fait pour d'autres cartes est refusé, et le match en
cours ne change pas.

Pour les développeurs : la sauvegarde ne contient que les réglages et les
choix (premier joueur, équipement, deck, puis tous les coups de chaque
manche, ceux du bot compris) ; la reprise rejoue ces coups dans le moteur
sans appeler le bot, donc à l'identique. Format `arsenal-fabbot`, version 1
(`crates/ui/src/sauvegarde.rs`). Routes : `GET /api/sauvegarde`,
`POST /api/charger` (corps : la sauvegarde).

### La page

L'écran de partie reprend celui de l'Arsenal JS (26/11/2026, liste des
écarts : `docs/rapatriement-interface.md`). Il occupe tout l'écran :

- **Le tapis**, à gauche : le bot en haut (zones tournées à 180°), vous en
  bas, l'illustration de chaque héros en fond de sa moitié. À côté de chaque
  tapis : les **ressources** (symboles de pitch, 3 par symbole), les **points
  d'action** (losanges) et les **effets en cours** (marqué, bonus de la
  prochaine attaque, réductions de coût, préventions, cartes jouables depuis
  le banni…). Main du bot en haut à droite (dos de cartes), votre main en bas
  à droite, en carré.
- **Au milieu** : « Tour N », les étapes du combat (Layer · Attaque ·
  Défense · Réactions · Dégâts · Résolution, l'étape en cours soulignée),
  qui a la priorité, puis la **chaîne de combat** : chaque maillon avec sa
  carte, « Attaque / Défense » en gros pour le maillon en cours, les
  bloqueurs à côté, « Touche » ou « Bloquée ». À droite, la **pile** (la carte
  de devant se résout en premier).
- **Le ruban**, au-dessus de vos zones : une phrase qui dit la situation
  (« Le bot · Growl 3 — défense 3, vous subissez 0 ») et les boutons du
  choix.
- **À droite** : Accueil, **Options** (animations, pause avant les dégâts,
  nouveau match, importer une partie, signaler un bug, abandonner), le score
  du match, le conseil du bot et la barre d'évaluation, la carte survolée en
  grand avec son texte, le journal, puis « Annuler mon dernier choix » et
  « Exporter la partie ».

Sur une carte : l'image officielle (ou un cadre dessiné : coût en haut à
droite, ⚔ puissance, ◆ pitch, ⛨ défense). Une carte penchée est engagée ; un
bord pointillé = carte face cachée (que vous seul voyez).

### Jouer

- **Votre tour** : cliquez une carte surlignée de votre main (ou de votre
  arsenal) pour la **sélectionner** : ses façons de la jouer apparaissent en
  boutons dans le ruban. L'arme, le héros, l'équipement : un clic active la
  capacité. Vous pouvez aussi **tirer** une carte vers la chaîne, le pitch ou
  l'arsenal. Le ruban propose « Fin de ma phase d'action », « Passer (fermer
  la chaîne) » ou « Passer », et pendant le tour du bot « Passer jusqu'à la
  fin du tour ».
- **Défendre** : touchez les cartes qui bloquent (main, équipement,
  arsenal) ; elles se posent sur la chaîne, le total de défense et « vous
  subissez N » se mettent à jour. Un nouveau clic reprend une carte.
  « Valider la défense » (ou « Ne pas bloquer ») envoie tout d'un coup ; une
  seule annulation revient avant la défense.
- **Payer** : touchez les cartes à pitcher (« Payez N, reste M »).
- **Arsenal** en fin de tour : touchez la carte, ou « Ne rien mettre en
  arsenal ». **Ordre du pitch** : touchez les cartes dans l'ordre (numéro
  affiché), puis « Valider cet ordre » ; ou « Garder l'ordre de pitch ».
- Clavier : `1` à `9` = boutons du ruban dans l'ordre, `P` = passer (ou
  valider la défense), `A` ou `Ctrl+Z` = annuler, `C` = conseil, `Échap` =
  désélectionner.
- **Annuler mon dernier choix** revient avant votre dernier choix (et avant
  les réponses du bot qui ont suivi), autant de fois que voulu.
- **Fin de partie** : une fenêtre donne le vainqueur, les PV, les dégâts
  infligés et subis, vos défenses ; puis Manche suivante, Rejouer, Changer de
  deck, Voir le journal, Annuler, Exporter.

## Dans le terminal (interface texte)

```
cargo run --release -p fab-ui
```

Par défaut, vous jouez **Hala** et le bot joue **Dorinthea**. Options :

| Option | Effet |
|---|---|
| `--deck dorinthea` | vous jouez Dorinthea (le bot prend alors Hala) |
| `--deck ira` | vous jouez Ira, Crimson Haze (le bot prend alors Hala) |
| `--deck bravo` | vous jouez Bravo, Flattering Showman (le bot prend alors Hala) |
| `--deck iyslander` | vous jouez Iyslander (le bot prend alors Hala) |
| `--deck florian` | vous jouez Florian (le bot prend alors Hala) |
| `--deck levia` | vous jouez Levia (le bot prend alors Hala) |
| `--deck dromai` | vous jouez Dromai (le bot prend alors Hala) |
| `--deck arakni` | vous jouez Arakni, Web of Deceit (le bot prend alors Hala) |
| `--bot hala` | deck du bot : `ira`, `hala`, `dorinthea`, `bravo`, `iyslander`, `florian`, `levia`, `dromai` ou `arakni` (on peut faire un miroir : `--deck hala --bot hala`) |
| `--niveau b1` | niveau du bot : `b0` (au hasard, par défaut), `b1` (heuristique) `b2` (recherche ISMCTS ; variantes `b2-500ms`, `b2-1000`…) ou `b3` (recherche avec évaluation apprise), voir `docs/bots.md` |
| `--graine 42` | rejouer exactement la même partie (mêmes mélanges) |

Exemple : `cargo run --release -p fab-ui -- --deck dorinthea --graine 42`
(les deux tirets seuls séparent les options de cargo de celles du jeu).

Par défaut, le bot est **B0 : il choisit au hasard** parmi les coups
légaux. Il sert à vérifier les règles et les cartes (il essaie tout, même
les coups absurdes). Avec `--niveau b1`, le bot **B1** joue « normalement »
(il attaque à son tour, défend au vôtre) : plus utile pour voir les cartes
dans de vraies situations. Il reste faible (voir `docs/bots.md`).

## Pendant la partie

Le moteur s'arrête à chaque choix que vous devez faire et affiche les options
numérotées. Il passe tout seul quand « passer » est le seul choix possible.

| Commande | Effet |
|---|---|
| `3` | choisir l'option 3 |
| `a` | annuler votre dernier choix (autant de fois que voulu) |
| `e` | afficher tout l'état, cimetières et cartes bannies compris |
| `t 12` | fiche et **texte officiel** de la carte `[12]` |
| `?` | aide |
| `q` | quitter |

Chaque carte affichée porte un numéro entre crochets, par exemple
`Edict of Steel (bleu) [9]` : c'est ce numéro qu'on donne à `t`.

La ligne « Permanents » donne, pour un allié (dragon, Aether Ashwing…), sa
puissance et ses PV actuels, ses compteurs (endurance, vie), les cartes
qui sont dessous (ex. « dessous : Ash ») et phantasm ; pour une aura ou une
Ash, son ward. Pour un dragon, `t` affiche le texte de sa face arrière.

Le texte officiel (avec errata) vient du fichier de Legend Story Studios ;
il faut l'avoir téléchargé une fois avec `bash tools/fetch_cards.sh`.

Vous ne voyez que ce que votre joueur a le droit de voir : la main et
l'arsenal du bot (face cachée) n'apparaissent que par leur nombre, et sa
pioche n'est pas révélée.

## Valider les cartes

C'est l'occasion de vérifier les cartes de Hala et Dorinthea en situation :
jouez-les, regardez le texte officiel avec `t`, puis notez dans
`docs/cards/STATUS.md` celles qui se comportent correctement (statut
`Validated`). Tout comportement faux : le signaler à Claude, il deviendra
d'abord un test qui échoue, puis sera corrigé.

## Limites actuelles

- Terminal : pas d'écran de choix ; vous jouez l'équipement et les 40
  premières cartes de la liste, le bot son choix appris. Le choix complet
  avant chaque manche est dans la page web.
- « Look at the top card of your deck » (On the Horizon) n'affiche encore rien.
- Page web : pas d'images de cartes (le nom, la couleur et les valeurs
  seulement) ; une seule partie à la fois (un nouvel onglet montre la même).
