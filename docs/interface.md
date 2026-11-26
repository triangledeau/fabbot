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

Comme en tournoi (CR 4.1.3 à 4.1.6), avant chaque manche :

1. **Qui commence ?** Le joueur désigné choisit : à la première manche, il
   est tiré au sort ; ensuite, c'est le perdant de la manche précédente. Le
   bot choisit toujours de commencer.
2. **Votre équipement** : cliquez une carte par zone (cliquer à nouveau la
   retire ; une zone peut rester vide). Pour les armes, deux mains au plus
   et un seul off-hand. Le bot choisit le sien **sans voir** le vôtre.
3. **Votre deck** : l'équipement du bot est maintenant montré. Chaque
   carte est une **pile** de ses exemplaires : clic = un exemplaire de plus
   dans le deck, clic droit = un de moins (les exemplaires hors du deck
   sont grisés). Le compteur doit indiquer **40 / 40** pour valider.

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

- **Jouer** : ouvre la fenêtre « Nouveau match » (decks, niveau du bot,
  manches, graine), puis l'écran de partie ;
- **Apprendre** (didacticiel, puzzles) et **Entraîner** (tournoi des bots)
  sont marqués « Bientôt » : ils arrivent avec les lots suivants du plan
  `docs/rapatriement-arsenal.md` ;
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

### Mes decks (decks maison)

Depuis l'accueil, **Mes decks** : collez la liste d'un deck copiée sur
FaBrary (« Copy deck list », texte) ou au format des fichiers du projet,
donnez-lui un nom, puis **Vérifier** ou **Importer**. La page dit si le
deck respecte le format Silver Age (55 cartes au plus, 40 de deck au moins,
2 exemplaires, cartes bannies, héros « Young »), quelles cartes ne sont pas
légales et lesquelles le bot ne sait pas encore jouer (pas codées). Le deck
est enregistré dans `data/decks/maison/<nom>.txt` ; s'il est jouable, il
apparaît dans la fenêtre **Nouveau match** (« Decks maison »), pour vous
comme pour le bot.

**Éditeur de deck** : « Nouveau deck » (choisir un héros Young),
« Modifier » un deck maison, ou « Dupliquer » un deck du projet (l'original
ne change pas). À gauche la collection Silver Age (recherche en français ou
en anglais, filtres pitch, type, « pour ce héros », « jouables par le bot »,
« dans le deck », tri) ; les cartes que le bot ne sait pas encore jouer sont
grisées. Clic : +1 ; clic droit : −1 ; Ctrl+Z : annuler. À droite : le
card-pool (55 au plus, 40 cartes de deck au moins), la répartition du
pitch, la courbe de coût, la liste par équipement et deck, et la
vérification du serveur à chaque changement. « Enregistrer » écrit le deck
maison ; « Copier la liste » donne le texte au format FaBrary. Un deck
maison se range dans les **archives** (« Archiver ») ; on ne supprime
qu'un deck archivé. Le filtre « pour ce héros » garde les cartes que le
héros peut avoir (même règle que la vérification ci-dessous).

La légalité vient de la base communautaire (`bash tools/fetch_cards.sh`) ;
sans elle, seuls le format et les cartes codées sont vérifiés. Avec elle,
la vérification contrôle aussi les classes et talents : le héros doit
avoir toutes les classes et tous les talents de chaque carte (Generic :
pour tous) ; pour une carte hybride (« Brute / Warrior »), un seul des deux
côtés suffit (réponse du propriétaire, 2026-11-26) ; « Essence of Ice »
(Iyslander), « Essence of Earth, Ice, and Lightning » (Bravo)… ajoutent
ces talents au héros.

Pour les développeurs : `crates/ui/src/api_decks.rs` (routes
`/api/collection`, `/api/decks-maison`, `/api/deck-maison`,
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

Depuis l'accueil, **Entraîner**. **Tournoi** : cochez les decks (du projet
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

- **En haut** : le bot (héros, PV, points d'action, ressources, taille du
  deck, nombre de cartes en main, équipement, arsenal, permanents, pitch).
- **Au milieu** : la chaîne de combat (attaque, cible, puissance,
  défenseurs) et la pile.
- **En bas** : vous, avec votre main.
- **À droite** : la question posée et ses options, la fiche de la carte
  survolée (avec le **texte officiel**), et le journal de la partie.
- « Cimetière · Bannies » : cliquez pour déplier le détail.

Sur une carte : coût en haut à droite, ⚔ puissance, ◆ pitch, ⛨ défense,
la bande de couleur indique sa couleur de pitch. Une carte penchée est
engagée ; un bord pointillé = carte face cachée (que vous seul voyez).

### Jouer

- Les cartes **surlignées en jaune** ont une option : cliquez pour la
  choisir (s'il y en a plusieurs, un petit menu s'ouvre). Vous pouvez aussi
  cliquer une option dans la liste de droite.
- Clavier : `1` à `9` = option, `P` = passer, `A` ou `Ctrl+Z` = annuler.
- **↶ Annuler** revient avant votre dernier choix (et avant les réponses du
  bot qui ont suivi), autant de fois que voulu. Le nombre entre parenthèses
  dit combien de choix on peut annuler. Après une annulation, le bot peut
  répondre autrement (B0 et B2 ont une part de hasard).
- Avec B2, chaque décision du bot prend une fraction de seconde : « Le bot
  réfléchit… » s'affiche en bas à droite.

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
