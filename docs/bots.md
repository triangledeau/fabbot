# Les bots et leur classement Elo

## Les bots

| Nom | Code | Comment il joue |
|---|---|---|
| **B0** | `RandomAgent` (`crates/ai/src/lib.rs`) | au hasard parmi les options légales ; sert de référence (Elo 0) et de testeur de règles |
| **B1** | `HeuristicAgent` (`crates/ai/src/b1.rs`) | règles fixes, sans recherche (détail ci-dessous) |
| **B2** | `IsmctsAgent` (`crates/ai/src/b2.rs`) | recherche ISMCTS : mondes imaginés + parties simulées par B1 (détail ci-dessous) |
| B3 | à venir (étape 7) | évaluation apprise |

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
- **Pitch** : la carte qui paie le reste au plus juste, en gardant les
  cartes les plus fortes.
- **Arsenal** : la carte d'attaque la plus forte.
- **Capacités activées** (hors attaques d'arme) : seulement celles du
  héros, des armes et des permanents. Pas « Destroy this » sur un
  équipement, ni « Banish this from your hand ».
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
| B2 | **+785** | 1 620 parties contre B1 (83,6 %, +283 ± 23) et 1 620 contre B0 (98,1 %) |

Calcul commun par Bradley-Terry sur les trois confrontations (contre B0
seul, B2 obtient +690 : à 98 %, l'écart est mal mesuré).

**B2 contre B1 : 83,6 %, 0 erreur (vérification `--check`).** Score de B2
(%), ligne = deck de B2, colonne = deck de B1 (20 parties par case) :

| B2 \ B1 | ira | hala | dorint. | bravo | iyslan. | florian | levia | dromai | arakni | moy. |
|---|---|---|---|---|---|---|---|---|---|---|
| ira | 100 | 85 | 100 | 80 | 100 | 100 | 100 | 70 | 100 | 93 |
| hala | 80 | 95 | 80 | 85 | 100 | 95 | 100 | 50 | 100 | 87 |
| dorinthea | 100 | 100 | 100 | 100 | 100 | 75 | 100 | 55 | 100 | 92 |
| bravo | 100 | 100 | 100 | 100 | 100 | 75 | 100 | 100 | 100 | 97 |
| iyslander | 95 | 90 | 95 | 90 | 95 | 75 | 95 | 25 | 100 | 84 |
| florian | 100 | 95 | 95 | 90 | 100 | 100 | 100 | 80 | 100 | 96 |
| levia | 30 | 15 | 10 | 15 | 65 | 40 | 90 | 10 | 35 | 34 |
| dromai | 100 | 100 | 100 | 95 | 100 | 100 | 100 | 100 | 100 | 99 |
| arakni | 90 | 50 | 35 | 65 | 100 | 55 | 100 | 30 | 100 | 69 |

Points faibles restants : **Levia** (34 %, le blood debt : B1, qui joue
les décisions suivantes et les parties simulées, ne le comprend pas, et
4 tours simulés sont trop courts pour en voir le coût) et **Arakni**
(69 %). Dromai reste très fort, aussi entre les mains de B1.

**B2 contre B0 : 98,1 %** (1 590 victoires, 30 défaites, 0 erreur), dont
84 % avec Levia ; 100 % avec 7 decks.

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
