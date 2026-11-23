# Les bots et leur classement Elo

## Les bots

| Nom | Code | Comment il joue |
|---|---|---|
| **B0** | `RandomAgent` (`crates/ai/src/lib.rs`) | au hasard parmi les options légales ; sert de référence (Elo 0) et de testeur de règles |
| **B1** | `HeuristicAgent` (`crates/ai/src/b1.rs`) | règles fixes, sans recherche (détail ci-dessous) |
| B2 | à venir | recherche ISMCTS |
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

`--bots b1 b0` marche aussi avec `--decks a b` (une seule paire).

### Résultats (2026-11-23, 81 000 parties, 0 erreur)

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
