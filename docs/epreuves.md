# Les épreuves : mesurer le niveau du bot

Chantier 1 de `docs/feuille-de-route-top8.md`. Les Elo de `docs/bots.md`
comparent les bots entre eux ; les épreuves cherchent **où** le bot perd
des points, décision par décision. Elles tournent sans session Claude (liste
de nuit `tools/nuit/travaux.txt`, voir `docs/nuit.md`).

## Épreuve A : puzzles à solution exacte

Programme : `cargo run --release -p fab-sim --bin epreuve-a`. Solveur :
`fab_ai::tour_exact`.

### Principe

Pendant son tour, un joueur ne découvre presque rien (la pioche arrive en
fin de tour). Si on **fixe la défense** de l'adversaire, le tour devient un
problème à un seul joueur, entièrement déterminé : le solveur **essaie
toutes les façons de jouer le tour** (cartes, ordre, pitch, arme, cibles,
réponses aux questions des cartes) et garde la meilleure.

1. Le programme joue des parties B3 contre B3 (phase de side apprise, comme
   le tournoi) et garde la position du **début de la phase d'action** de
   chaque tour (pile vide, pas de combat).
2. Pour chaque position et chaque défense, le solveur trouve le maximum :
   **tuer** l'adversaire ce tour si c'est possible, sinon lui faire perdre le
   plus de points de vie. Le tour s'arrête à la décision d'arsenal (CR
   4.4.3b, après les déclenchements de début de phase de fin).
3. B3 joue le **même** tour, dans le **même** monde (même main adverse, même
   ordre des decks, même hasard), contre la même défense. B3, lui, ne voit
   que son information ordinaire.

Les deux défenses :

| Défense | L'adversaire… | Ce que le puzzle mesure |
|---|---|---|
| `passive` | ne défend avec aucune carte, ne joue rien ; ses autres choix (arcane barrier, ordre des déclenchements) sont faits par B1 | l'enchaînement pur : ordre des attaques, go again, pitch, arme, combos |
| `b1` | prend toutes ses décisions avec B1 (qui ne voit que son information) | le tour contre une défense réaliste et fixe |

### Lire le résultat

Une table par défense, une ligne par deck (le deck du joueur dont c'est le
tour) :

- **puzzles** : positions testées ; **complets** : le solveur a tout essayé
  (sinon le budget `--budget`, en noeuds, est atteint ; ces puzzles sont
  écartés des taux) ;
- **max = 0** : puzzles où aucune ligne ne retire de point de vie ;
- **résolus au mieux** : B3 fait aussi bien que le maximum (létal si le
  létal existe, sinon autant de dégâts) ;
- **manque moyen** : points de vie que B3 laisse en moyenne par rapport au
  maximum (0 quand il trouve le létal) ;
- **létaux trouvés** : puzzles où un létal existait, et combien B3 en a
  joué ;
- puis les **pires puzzles**, létaux manqués d'abord, avec la meilleure
  ligne et la ligne de B3 en clair (graine et tour pour les retrouver).

`--details fichier.tsv` écrit une ligne par puzzle (tableur).

### Ce qu'il faut garder en tête

- L'objectif du puzzle (dégâts de ce tour) **n'est pas** tout le jeu : garder
  une carte pour défendre ou pour l'arsenal peut valoir mieux que 2 dégâts.
  Un puzzle « non résolu » n'est donc pas toujours une erreur. Les **létaux
  manqués contre la défense `b1`** sont les plus parlants : tuer vaut
  toujours mieux.
- Contre la défense `passive`, un létal manqué n'est souvent pas une erreur :
  un vrai adversaire défendrait. Cette défense sert à voir si B3 **sait**
  enchaîner ses cartes.
- Le solveur connaît la vraie main adverse et l'ordre des decks (c'est ce qui
  rend la solution exacte) ; B3 non. Contre la défense `b1`, une partie de
  l'écart vient donc de l'information cachée. Une carte qui pioche pendant le
  tour donne aussi au solveur une information que B3 n'a pas.
- Réductions du solveur : deux exemplaires identiques en main ne sont essayés
  qu'une fois ; pour un même paiement, chaque **ensemble** de cartes pitchées
  n'est essayé qu'une fois (l'ordre de pitch n'est pas essayé : seul l'ordre
  dans la zone de pitch, sans effet avant la fin du tour, est perdu).
- Le seuil « top 8 » proposé (feuille de route) : ≥ 95 % des puzzles
  d'attaque résolus au mieux, 100 % des létaux trouvés.

### Options

| Option | Défaut | Rôle |
|---|---|---|
| `--parties N` | 81 | parties jouées (81 = chaque paire de decks une fois) |
| `--bot nom` | `b3` | le bot testé (aussi celui des parties) |
| `--defense liste` | `passive,b1` | défenses, séparées par des virgules |
| `--budget N` | 200000 | noeuds au plus par puzzle |
| `--decks liste` | les 9 de `data/decks/` | decks utilisés |
| `--seed N` | 1 | graine (mêmes graines et même code : mêmes puzzles) |
| `--threads N` | tous | threads |
| `--details fichier` | — | une ligne par puzzle (TSV) |

### Pas encore fait

- Les 15 puzzles de value de l'Arsenal JS (lot 6 du rapatriement).
- Les puzzles de **défense** (que bloquer contre une attaque connue).
- Une défense « parfaite » (l'adversaire choisit au mieux contre chaque
  ligne, recherche min-max) : donnerait des **létaux forcés** exacts.

### Première mesure (2026-11-27)

81 parties B3 contre B3 (chaque paire une fois), budget 200 000 noeuds,
1 452 puzzles par défense, 96 % complets (journal :
`docs/journal/2026-11-27-epreuve-a.md`).

| Défense | résolus au mieux | manque moyen | létaux trouvés |
|---|---|---|---|
| passive | 61,4 % | 1,67 PV | 80,1 % (249 / 311) |
| b1 | 61,1 % | 1,26 PV | 80,0 % (76 / 95) |
