# Les bots et leur classement Elo

## Les bots

| Nom | Code | Comment il joue |
|---|---|---|
| **B0** | `RandomAgent` (`crates/ai/src/lib.rs`) | au hasard parmi les options légales ; sert de référence (Elo 0) et de testeur de règles |
| **B1** | `HeuristicAgent` (`crates/ai/src/b1.rs`) | règles fixes, sans recherche (détail ci-dessous) |
| **B2** | `IsmctsAgent` (`crates/ai/src/b2.rs`) | recherche ISMCTS : mondes imaginés + parties simulées par B1 (détail ci-dessous) |
| **B3** | `IsmctsAgent` avec `ReglagesB2::b3()` (`crates/ai/src/b3.rs`) | la recherche de B2, avec une évaluation apprise par auto-jeu (détail ci-dessous) |

### Un bot ne voit que l'information de son joueur

B1 ne lit la partie qu'à travers une **`Vue`** (`crates/ai/src/vue.rs`).
L'état complet y est un champ privé : le code de B1 ne peut pas y toucher.
La `Vue` montre la main du joueur, ses cartes face cachée hors du deck, les
cartes face visible, et des nombres publics (vies, tailles de main, combat,
pile). Lire une carte cachée (main adverse, arsenal adverse, deck) fait
**paniquer** le programme : un test le vérifie, et les parties de test
complètes sur les 9 decks prouvent qu'aucune lecture interdite n'a lieu.

### Les règles de B1

- **À son tour** : il joue d'abord les cartes avec go again (pour garder
  l'action), les actions non-attaque avec go again avant les attaques
  (souvent un bonus pour la prochaine attaque), puis la plus grosse attaque
  (carte ou arme). Il joue ses réactions d'attaque, et ses instants quand il
  attaque.
- **Au tour adverse** : il défend une carte à la fois. Une carte est posée
  si les dégâts évités (+1 si l'attaque est entièrement bloquée, ×1,5 sous
  10 PV) dépassent ce qu'elle coûte : 2,5 pour une carte de la main
  (+0,25 par point de puissance, +0,75 par carte de la main déjà posée),
  0,8 × défense pour un équipement avec blade break, 1 avec battleworn ou
  temper, 0,5 sinon. Contre une attaque mortelle, il pose tout ce qu'il
  faut. Il joue une réaction de défense si elle évite au moins 2 dégâts.
  Il ne défend jamais avec un équipement qui n'ajoute aucune défense
  (défense 0, sans capacité qui en dépende : les Nullrune, Arcane
  Lantern, Aetherstorm Wellingtons) ; B2, B3 et le niveau réfléchi ne
  l'essaient pas non plus (`fab_ai::equipement_sans_defense`).
- **Pitch** : la carte qui paie le reste au plus juste, en gardant les
  cartes les plus fortes.
- **Arsenal** : la carte d'attaque la plus forte (Iyslander : une carte
  Ice bleue non-attaque d'abord).
- **Cible d'une attaque** (CR 1.4.5) : le héros adverse, sauf si l'attaque
  peut détruire un allié adverse (un dragon de Dromai…) dont 2 attaques
  feraient plus de dégâts qu'elle (2 × puissance de l'allié > puissance de
  l'attaque) ; jamais un allié si l'attaque peut tuer le héros. Réglage
  `tours_d_allie` (2). Mesure du 2026-11-26 (tournoi B1 contre B1,
  1 000 parties par paire) : avant, B1 n'attaquait **jamais** un dragon
  (0 sur ≈ 7 000 occasions) ; Dromai contre Ira passe de 90 % à 81 %, les
  autres matchups de Dromai bougent de 1 point au plus (moyenne de Dromai
  87 % → 86 %). Avec 1 ou 3 au lieu de 2 : 86 % et 85 %. B0, B2 et B3
  attaquaient déjà les dragons (61 %, 38 %, 45 % des occasions).
- **Capacités activées** (hors attaques d'arme) : seulement celles du
  héros, des armes et des permanents. Pas « Destroy this » sur un
  équipement, ni « Banish this from your hand », sauf les équipements
  d'Ira (ci-dessous).
- **Go again qui n'est pas écrit sur la carte** (2026-11-26) : B1 lit les
  mots-clés de la carte, alors qu'une partie du go again n'est donnée qu'en
  jeu. Il prévoit désormais : Zenith Blade aiguisée ce tour (première
  attaque du tour), les combos d'Ira (Seek Vengeance après Edge of Autumn,
  Pouncing Qi et Qi Unleashed après Crouching Tiger, Aspect of Tiger: Body
  après une attaque rouge) et Scar for a Scar quand il a moins de vie que
  l'adversaire.
- **Équipements d'Ira** : Pouncing Paws quand un combo du Tigre est en
  main, Tearing Shuko juste avant de jouer le Crouching Tiger, Blood Scent
  quand une carte de la main coûte plus que ses ressources.
- **Sorts arcaniques (Iyslander)** : un sort vaut les dégâts qui passent
  l'arcane barrier adverse (total public de ses équipements, plafonné par
  ce qu'il peut payer). Un sort entièrement arrêté est gardé (défense,
  pitch). Avant un sort, il ajoute les renforts qui servent : Crucible of
  Aetherweave (+1, s'il reste de quoi payer), amp 1 d'Arcane Twining /
  Photon Splicing ou Stir the Aetherwinds quand ils font passer la
  barrière. Il garde en arsenal une carte Ice bleue non-attaque et la joue
  au tour adverse (gelure chez l'adversaire). Frost Spike met sa gelure
  chez l'adversaire.
- **Ordre du pitch sous le deck (pitch stacking)** : il alterne une carte
  à jouer et une carte qui paie, pour que les mains suivantes aient un sort
  et de quoi le payer (effet mesuré faible : le deck de 40 cartes est
  rarement épuisé).
- **Questions des cartes** : « oui », le premier mode, le X maximum. Pour
  une cible : son attaque en cours, puis le héros adverse, puis un objet
  adverse.

**Limites connues** : B1 ne lit pas le texte des cartes. Il ne sait donc pas
si une cible est amie ou ennemie, ni éviter les pièges de son propre deck.
Exemple mesuré : avec Levia, il perd en moyenne **plus de 12 PV par partie
à cause du blood debt** (contre Hala B0), d'où ses mauvais résultats. B2 (la
recherche) doit corriger ce genre de défaut.

### Réglages essayés (B1 modifié contre B1, 81 paires × 200 parties)

| Variante | Écart d'Elo |
|---|---|
| coût de base d'une carte de la main 2,5 au lieu de 2 | **+5** (gardé) |
| coût de base 3 / 1,5 / 4 | +2 / −16 / −56 |
| pas de supplément par point de puissance | −60 |
| répondre « non » aux questions des cartes | −24 |
| ne jamais jouer d'instant | −27 |
| ne jamais mettre en arsenal | +1 (sans effet) |
| activer toutes les capacités non-attaque | +3 (sans effet, la version prudente est gardée) |

### B2 : la recherche (ISMCTS)

Pour chaque décision qui a au moins deux options différentes, B2 répète
300 fois :

1. **imaginer un monde possible** (`crates/ai/src/determinisation.rs`) :
   une copie de la partie où les cartes cachées de l'adversaire (main, deck,
   arsenal face cachée) sont tirées au hasard dans son **card-pool** (ses
   cartes de deck parmi les 55), moins les cartes de son deck déjà vues ;
   l'ordre de son propre deck est mélangé, et le hasard futur (pioches,
   tirages) change de graine ;
2. **choisir une option à essayer** dans ce monde (formule UCB : les
   options au meilleur score moyen, ou encore peu essayées) ;
3. **continuer la partie pendant 4 tours** avec B1 pour les deux joueurs,
   puis **estimer** le résultat : vie + 1,5 par carte en main ou en arsenal
   + 0,25 par carte du deck, l'écart entre les deux joueurs étant ramené
   entre 0 et 1 (une partie finie avant compte 1, ½ ou 0) ;
4. noter le résultat pour l'option essayée.

Il joue ensuite l'option la plus essayée (c'est-à-dire celle qui a le
meilleur score, la formule UCB y revenant le plus souvent). Deux
exemplaires identiques d'une carte en main comptent comme une seule
option.

Le code sait construire un **arbre** de décisions (les siennes et celles de
l'adversaire, variante « un seul observateur » de l'ISMCTS) ; le réglage
`profondeur` le limite. Les mesures ci-dessous montrent qu'avec ce budget,
l'arbre limité à la décision présente (profondeur 1) est le plus fort : dans
un arbre plus profond, l'adversaire imaginé joue moins bien que B1.

**Information** : B2 ne connaît que le **card-pool** de l'adversaire (ses
55 cartes), pas les 40 qu'il a choisies (décision du propriétaire,
2026-11-23 ; plus tard, on pondérera par la fréquence des cartes dans les
decks du méta). Il ne lit pas les cartes cachées : il les **remplace** par
des cartes imaginées. Un test le prouve : deux parties qui ne diffèrent que
par la main adverse et l'ordre des decks donnent exactement le même choix
(`b2_fait_le_meme_choix_quelle_que_soit_la_vraie_main_adverse`).
Approximations : il oublie les cartes cachées vues un instant (cartes
révélées puis reprises) ; les cartes bannies face cachée de l'adversaire
comptent comme vues.

**Réglages** (dans le nom du bot, après `b2-`) : nombre d'itérations
(`b2-1000`) ou durée par décision (`b2-500ms`), puis `t4` (tours simulés,
`t0` : jusqu'à la fin), `p1` (profondeur de l'arbre, `p0` : sans limite),
`c0.4` (exploration), `v1.5` (valeur d'une carte), `a` (parties simulées
par B0). `b2` = `b2-300t4p1c0.4v1.5`. Une durée rend les parties non
reproductibles (elles dépendent de la vitesse de la machine) : les tournois
utilisent un nombre d'itérations.

**Coût** : ≈ 2 s de calcul par partie complète (sur un cœur), contre
0,5 ms pour B1.

#### Réglages essayés (B2 contre B1, 81 paires × 2 ou 4 parties)

| Variante | Score | Elo |
|---|---|---|
| 100 itérations, parties simulées jusqu'à la fin, arbre complet | 59 % | +65 |
| idem, exploration 0,4 / 1,2 | 57 % / 59 % | +52 / +65 |
| idem, parties simulées par B0 | 36 % | −99 |
| 100 itérations, **4 tours simulés**, arbre complet | 77 % | +205 |
| 2 / 3 / 6 tours simulés | 63 % / 68 % / 70 % | +94 / +134 / +145 |
| 4 tours, carte = 3 PV / 0,75 PV | 69 % / 70 % | +141 / +150 |
| 300 itérations, 4 tours, arbre complet | 78 % | +218 |
| 100 itérations, 4 tours, **profondeur 1** | 78 % | +221 |
| idem, profondeur 2 / 3 | 75 % / 75 % | +192 / +187 |
| idem, 5 tours | 78 % | +224 |
| idem, exploration 0,4 / 1,2 | 81 % / 78 % | +250 / +218 |
| 30 itérations, 4 tours, profondeur 1 | 67 % | +122 |
| **300 itérations, 4 tours, profondeur 1** | **84 %** | **+283** |

### B3 : l'évaluation apprise (étape 7)

B3 cherche exactement comme B2 (300 itérations, 4 tours simulés,
profondeur 1). Seule change l'**estimation** des parties simulées arrêtées
après 4 tours : à la place de la formule fixe (vie + 1,5 par carte…), une
évaluation dont les poids sont **appris** sur des parties jouées par les
bots eux-mêmes.

**L'évaluation** : chaque joueur reçoit une note = somme de ses
caractéristiques × leur poids ; les chances de gagner du joueur 0 sont
σ(note₀ − note₁). Caractéristiques : vie, vie sous 10 et sous 5, cartes en
main, arsenal, deck, équipements et leur défense, cartes à blood debt face
visible dans la zone de bannissement, cimetière, bannissement, « c'est son
tour » ; le héros (force du deck) et, propres à chaque héros, sa vie et ses
cartes (main + arsenal) ; chaque permanent contrôlé, **carte par carte**
(Runechant, Frostbite, Ash, dragons, Might…) ; depuis la 3e génération,
le **contenu de la main** (main + arsenal, valeurs imprimées) : nombre
d'attaques, leur puissance totale, défense totale, réactions de défense,
ressources (pitch), et chaque carte, **carte par carte** (« en main
Command and Conquer (red) = … »). Les poids sont lus par nom de
carte dans `data/b3/evaluation.txt` (fichier lisible) : un nouveau deck a
des poids nuls tant qu'on n'a pas refait l'apprentissage.

Pendant la recherche, le terme « force du héros » est retiré : il est le
même pour tous les coups d'une décision, et dans un matchup déséquilibré il
écrasait toutes les estimations près de 0 (Levia : −3,3).

**L'apprentissage** (`--apprendre-b3`) : une position au début de chaque
tour de chaque partie, avec le résultat final ; régression logistique
(Newton, pénalité ridge). Une partie sur 10 est gardée pour mesurer
la qualité des prédictions (perte logarithmique, 0,693 = toujours 50 %).
Trois pénalités sont essayées (λ/4, λ, 4λ ; λ = 10 par défaut) et la
meilleure sur ces parties de test est gardée. Sur les mêmes parties, le
programme apprend aussi des poids **sans** le contenu de la main et les
écrit à côté (`…_sans_main.txt`), pour comparer.

```bash
cargo run --release -p fab-sim -- --apprendre-b3 --games 500
cargo run --release -p fab-sim -- --apprendre-b3 --games 112 --bots b3-100 b3-100
cargo run --release -p fab-sim -- --tournoi --bots b3 b2 --games 16 --check
cargo run --release -p fab-sim -- --tournoi --bots b3 b3:data/b3/generation2.txt --games 32 --check
```

Variantes : `b3-800t1` (mêmes réglages que `b2-…`), `b3:fichier` (autre
fichier de poids).

#### Mesures (2026-11-26, contre B2, 81 paires)

| Poids | Appris sur | Perte (test) | Parties | Score | Elo (intervalle à 95 %) |
|---|---|---|---|---|---|
| formule fixe de B2 | — | 0,595 | | | |
| 1re génération | 40 500 parties B1 contre B1 (900 000 positions, 10 s) | 0,346 | 324 | 51,2 % | +9 (−29 à +47) |
| idem, sans la force du héros dans la recherche | | | 324 | 52,8 % | +19 (−18 à +58) |
| idem, 1 tour simulé, 800 itérations (même temps que B2) | | | 324 | 46,6 % | −24 (−62 à +14) |
| 2e génération | 4 536 parties B3-100 contre B3-100 (88 000 positions, 35 min) | 0,391 | 648 | 53,7 % | +26 (−1 à +53) |
| 2e génération, mesure refaite | | | 1 296 | 56,6 % | +46 (+27 à +65) |
| 3e génération, sans le contenu de la main | 9 072 parties B3-100 (2e gén.) contre B3-100 (153 000 positions, 42 min) | 0,377 | 1 296 | 57,6 % | +53 (+34 à +73) |
| **3e génération, avec le contenu de la main (retenue)** | mêmes parties, λ = 40 | **0,372** | 1 296 | **57,6 %** | **+53 (+34 à +73)** |

Face-à-face direct, plus sensible : **B3 3e génération contre B3 2e
génération : 52,0 % sur 2 592 parties, +14 Elo (intervalle +1 à +28)**,
0 erreur.

**4e génération (2026-11-27, non retenue)** : apprise après les correctifs
de B1 (go again de Zenith Blade et des combos d'Ira, sorts d'Iyslander) sur
9 072 parties B3-100 contre B3-100 (152 637 positions, 63 min ; perte de
test 0,426, non comparable à la 3e génération car les parties ont changé).
Face-à-face contre la 3e génération : **46,9 % sur 1 296 parties, −22 Elo
(−41 à −3)**, 0 erreur. La 3e génération reste dans `data/b3/evaluation.txt`.

**Avec Aurora (2026-11-27, retenue)** : le 10e deck oblige à refaire les
poids (le héros Aurora n'avait pas de poids). 11 200 parties B3-100
(3e génération) contre B3-100 sur les 100 paires (196 147 positions,
69 min ; perte de test 0,4727, λ = 40). Face-à-face contre la 3e génération :
**48,7 % sur 1 600 parties, −9 Elo (−26 à +8)**, 0 erreur : écart non
significatif. Ces poids sont dans `data/b3/evaluation.txt` car eux seuls
connaissent Aurora ; la 3e génération est gardée dans
`data/b3/generation3.txt` (`b3:data/b3/generation3.txt`).

0 erreur. Les poids de 1re génération prédisent bien la fin d'une partie
**jouée par B1** (perte 0,35 contre 0,60 pour la formule fixe), mais
n'aident presque pas la recherche. Ceux de 2e génération, appris sur des
parties de B3, donnent une valeur bien plus grande aux cartes (une carte en
main ≈ 3 PV, contre ≈ 0,4 PV en 1re génération) ; c'est le premier
réglage qui dépasse B2, de peu (la mesure refaite sur deux fois plus de
parties donne +46 : la 1re mesure était basse par hasard). La 3e
génération apporte encore un peu (+14 en face-à-face). Le contenu de la
main améliore un peu les prédictions (perte 0,372 contre 0,377), mais pas
le score contre B2, sur ces 1 296 parties (sans doute parce que les cartes en main comptent
déjà beaucoup par leur nombre, et que la recherche joue de toute façon ces
cartes dans ses parties simulées). Poids notables : une Passing Mirage
(blue) en main vaut −1,2, une Dustup (red) +0,8 (par rapport à une carte
quelconque, qui vaut déjà ≈ +0,3 + le poids du héros). Gardés dans le
dépôt : `data/b3/evaluation.txt` (poids avec Aurora, utilisés par `b3`), `data/b3/generation3.txt` (3e génération),
`data/b3/generation2.txt` (`b3:data/b3/generation2.txt`) et
`data/b3/generation1_b1.txt` (1re génération, `b3:data/b3/generation1_b1.txt`).
Coût : le même que B2 (≈ 2,5 s par partie sur un cœur).

Pistes : les gains d'une génération à l'autre diminuent (+46, puis +14) ;
une 4e génération n'apporterait sans doute pas beaucoup plus. Plus
prometteur : une évaluation qui regarde des **paires** de caractéristiques
(ex. vie de l'adversaire × puissance en main : « peut-il finir ? »), ou
apprendre directement ce que la recherche doit choisir (comme AlphaZero),
ou garder les poids pendant tout le tour au lieu d'évaluer seulement au
début des tours.

## La phase de side (étape 6)

Avant chaque partie, chaque joueur choisit son équipement et les 40 cartes
de son deck dans son card-pool de 55 (CR 4.1.4 à 4.1.6, TRP 7.4), dans
l'ordre officiel : premier joueur (CR 4.1.3), arène sans voir celle de
l'adversaire, présentation des arènes (CR 4.1.5), puis deck. Code :
`fab_match::preparer_manche` et le trait `Side` ; façons de choisir des
bots : `crates/ai/src/side.rs`.

| Side | Règle |
|---|---|
| `liste` | la première carte de chaque zone qui trouve sa place, les 40 premières cartes de la liste (l'ancien choix provisoire) |
| `appris` | la table apprise `data/side/b1.txt`, propre à chaque paire de héros (par défaut) |
| `general` | la même table, mais l'entrée « contre n'importe qui » du héros (sert à mesurer) |
| `hasard` | tout au hasard (sert à apprendre) |

**Apprentissage** (`--apprendre-side`, ≈ 3 minutes sur 4 cœurs) : pour
chacune des 81 paires, 20 000 parties B1 contre B1 où le premier deck tire
son équipement (une option par zone) et ses 40 cartes au hasard, l'autre
gardant le choix `liste` (ou, avec `--side-adverse appris`, le choix de
la table actuelle : c'est ainsi qu'a été faite la table du 2026-11-26, pour
qu'un deck apprenne à jouer contre les équipements que l'adversaire prend
vraiment, comme les Nullrune contre Iyslander). Une régression linéaire estime ce que chaque
exemplaire de carte et chaque équipement change aux chances de gagner
(avec « je commence » comme variable à part). L'estimation d'une paire est
tirée vers l'estimation générale du deck (pénalité « ridge » λ = 1 000).
Le choix appris garde le meilleur équipement de chaque zone et les 40
exemplaires les mieux notés.

```bash
cargo run --release -p fab-sim -- --apprendre-side --games 20000 --side-adverse appris
cargo run --release -p fab-sim -- --tournoi --bots b1 b1 --side appris liste --games 4000 --check
```

La table est un fichier texte lisible : pour chaque paire, l'arène choisie,
les cartes laissées hors du deck et, entre crochets, l'écart estimé (en
points de % de victoire par exemplaire) par rapport à la meilleure option de
la zone ou à la moins bonne carte gardée. ⚠️ Ces écarts mesurent ce qui
réussit **à B1** : une carte que B1 joue mal (Arcane Polarity, par exemple)
est mal notée même si elle est bonne entre les mains d'un humain.

### Résultats (2026-11-25, graines différentes de l'apprentissage)

| Mesure | Parties | Score | Écart d'Elo (intervalle à 95 %) |
|---|---|---|---|
| B1 `appris` contre B1 `liste` | 324 000 | **63,5 %** | **+96** (+95 à +98) |
| B1 `general` contre B1 `liste` | 162 000 | 62,0 % | +85 (+83 à +87) |
| B1 `appris` contre B1 `general` (λ = 5 000) | 324 000 | 51,8 % | +12 (+11 à +14) |
| B1 `appris` contre B1 `general` (λ = 1 000, retenu) | 324 000 | 52,0 % | +14 (+13 à +15) |
| B1 `appris` contre B1 `general` (λ = 250) | 324 000 | 52,0 % | +14 (+13 à +15) |
| B2 `appris` contre B2 `liste` | 1 620 | **62,0 %** | **+85** (+68 à +103) |

0 erreur. Le gain vient surtout de l'équipement (ex. Florian : Nebula Blade
plutôt qu'Annals of Sutcliffe, −12,8 points estimés pour Annals ; Ira : Blade
Beckoner Helm plutôt que Mask of the Swarming Claw) et de quelques cartes
que B1 joue mal. B1 `appris` gagne même ses 9 miroirs contre B1 `liste`
(60 à 87 %).

Les tournois (`--tournoi`) et `--decks` utilisent maintenant `--side appris
appris` par défaut ; les chiffres B0 / B1 / B2 plus haut ont été mesurés
avec le choix `liste` des deux côtés.

## Le classement Elo

Si A a `d` points d'Elo de plus que B, A marque en moyenne
`1 / (1 + 10^(−d/400))` point par partie (victoire 1, nulle ½) :
+100 → 64 %, +200 → 76 %, +400 → 91 %, +500 → 95 %.
Calcul : `crates/ai/src/elo.rs` (modèle de Bradley-Terry ; B0 vaut 0).

### Mesurer

```bash
cargo run --release -p fab-sim -- --tournoi --bots b1 b0 --games 1000 --check
```

Le premier bot joue chacun des 9 decks contre le second avec chacun des 9
decks (81 paires, miroirs compris), `--games` parties par paire, en
alternant les places et le joueur qui choisit qui commence. Le simulateur
affiche l'écart d'Elo avec son intervalle à 95 % et la matrice des scores.

`--bots b1 b0` marche aussi avec `--decks a b` (une seule paire). Pour B2 :

```bash
cargo run --release -p fab-sim -- --tournoi --bots b2 b1 --games 20 --check
cargo run --release -p fab-sim -- --tournoi --bots b2 b0 --games 20 --check
```

(≈ 15 et 9 minutes dans le conteneur de Claude, 4 cœurs.)

### Classement commun (2026-11-23)

| Bot | Elo | Mesuré sur |
|---|---|---|
| B0 | 0 | référence |
| B1 | **+511** | 81 000 parties contre B0 |
| B2 | **+796** | 1 620 parties contre B1 (83,9 %, +287 ± 23) et 1 620 contre B0 (98,8 %) |

Calcul commun par Bradley-Terry sur les trois confrontations (contre B0
seul, B2 obtient +770 : à 99 %, l'écart est mal mesuré). Ces mesures sont
faites avec B2 qui imagine la main adverse dans le **card-pool** ; quand il
connaissait les 40 cartes exactes, il faisait 83,6 % contre B1 (même force,
à la précision près).

**B2 contre B1 : 83,9 %, 0 erreur (vérification `--check`), 2 nulles par
blocage.** Score de B2 (%), ligne = deck de B2, colonne = deck de B1
(20 parties par case) :

| B2 \ B1 | ira | hala | dorint. | bravo | iyslan. | florian | levia | dromai | arakni | moy. |
|---|---|---|---|---|---|---|---|---|---|---|
| ira | 100 | 85 | 95 | 90 | 100 | 100 | 100 | 65 | 100 | 93 |
| hala | 85 | 98 | 95 | 90 | 100 | 90 | 100 | 35 | 100 | 88 |
| dorinthea | 100 | 100 | 100 | 100 | 100 | 85 | 100 | 40 | 100 | 92 |
| bravo | 100 | 100 | 100 | 95 | 100 | 95 | 100 | 100 | 100 | 99 |
| iyslander | 80 | 85 | 90 | 85 | 100 | 70 | 95 | 30 | 100 | 82 |
| florian | 100 | 80 | 100 | 100 | 100 | 100 | 100 | 55 | 100 | 93 |
| levia | 50 | 5 | 0 | 30 | 70 | 25 | 80 | 15 | 55 | 37 |
| dromai | 100 | 100 | 100 | 95 | 100 | 100 | 100 | 90 | 100 | 98 |
| arakni | 100 | 52 | 40 | 75 | 100 | 65 | 100 | 35 | 100 | 74 |

Points faibles restants : **Levia** (37 %, le blood debt : B1, qui joue
les décisions suivantes et les parties simulées, ne le comprend pas, et
4 tours simulés sont trop courts pour en voir le coût) et **Arakni**
(74 %). Dromai reste très fort, aussi entre les mains de B1.

**B2 contre B0 : 98,8 %** (1 600 victoires, 18 défaites, 2 nulles,
0 erreur), dont 91 % avec Levia.

### Résultats de B1 (2026-11-23, 81 000 parties, 0 erreur)

**B1 contre B0 : 95,0 % → B1 = +512 Elo** (intervalle à 95 % : +506 à +517).

Score de B1 (%), ligne = deck de B1, colonne = deck de B0 :

| B1 \ B0 | ira | hala | dorint. | bravo | iyslan. | florian | levia | dromai | arakni | moy. |
|---|---|---|---|---|---|---|---|---|---|---|
| ira | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| hala | 100 | 100 | 99 | 100 | 100 | 100 | 100 | 99 | 100 | 100 |
| dorinthea | 99 | 98 | 96 | 98 | 100 | 98 | 100 | 80 | 100 | 96 |
| bravo | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| iyslander | 98 | 99 | 97 | 99 | 100 | 98 | 100 | 88 | 99 | 98 |
| florian | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 96 | 100 | 100 |
| levia | 62 | 44 | 35 | 59 | 96 | 47 | 98 | 61 | 69 | 64 |
| dromai | 99 | 100 | 99 | 100 | 98 | 98 | 100 | 98 | 100 | 99 |
| arakni | 100 | 99 | 99 | 100 | 100 | 98 | 100 | 96 | 100 | 99 |

**B1 contre B1** (score du deck de la ligne contre celui de la colonne) :
ce sont les matchups « joués par B1 ». Ils reflètent surtout ce que B1 sait
faire de chaque deck, **pas** la vraie force des decks.

| ligne \ colonne | ira | hala | dorint. | bravo | iyslan. | florian | levia | dromai | arakni | moy. |
|---|---|---|---|---|---|---|---|---|---|---|
| ira | 50 | 80 | 79 | 23 | 89 | 49 | 96 | 12 | 98 | 64 |
| hala | 19 | 50 | 41 | 5 | 66 | 28 | 100 | 3 | 99 | 46 |
| dorinthea | 22 | 59 | 50 | 20 | 54 | 20 | 100 | 3 | 94 | 47 |
| bravo | 78 | 94 | 81 | 50 | 80 | 50 | 99 | 42 | 99 | 75 |
| iyslander | 11 | 35 | 46 | 19 | 50 | 10 | 94 | 1 | 73 | 38 |
| florian | 51 | 74 | 80 | 52 | 91 | 53 | 96 | 10 | 96 | 67 |
| levia | 5 | 0 | 0 | 1 | 6 | 5 | 50 | 2 | 7 | 8 |
| dromai | 85 | 98 | 97 | 56 | 98 | 89 | 98 | 52 | 99 | 86 |
| arakni | 2 | 1 | 6 | 1 | 27 | 4 | 93 | 1 | 50 | 21 |

Vitesse : ≈ 7 000 parties/s (B1 contre B0) dans le conteneur de Claude
(4 cœurs) ; un tournoi complet de 81 000 parties prend ≈ 12 s.
