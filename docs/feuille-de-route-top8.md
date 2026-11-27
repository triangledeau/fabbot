# Feuille de route : vers un bot de niveau top 8 Pro Tour (2026-11-27)

Agent : Claude, branche `claude/project-thread-s5ckuh`. Document de plan :
aucun code n'est changé.

## Le cadre (décisions du propriétaire)

- **Périmètre limité aux cartes que le bot connaît** : les 9 decks de
  `data/decks/` et Aurora (`data/decks/en_cours/10_aurora.txt`), soit
  10 decks et 100 paires. Pas d'encodage complet du pool Silver Age.
  « Top 8 Pro Tour » veut donc dire ici : **jouer ces 10 decks, les uns
  contre les autres, comme un joueur de top 8**.
- **Pas de location de calcul dans le cloud.** Le calcul lourd tourne sur
  la Surface Pro 9 du propriétaire (i5-1235U, 2 cœurs performants + 8 cœurs
  économes, 12 threads, 16 Go, pas de carte graphique), de préférence la
  nuit, ou dans les sessions Claude déjà en cours (4 cœurs).
- **Critères** : le temps et le quota Claude. Le calcul local ne coûte
  aucun token : tout ce qui peut devenir un script lancé sur la Surface
  passe avant ce qui demande une session Claude.

## 1. Où on en est (chiffres mesurés)

| Mesure | Résultat | Source |
|---|---|---|
| B1 contre B0 | 95,0 %, B1 = +512 Elo | `docs/bots.md` |
| B2 contre B1 | 83,9 %, B2 = +796 Elo | `docs/bots.md` |
| B3 (3e gén.) contre B2 | 57,6 %, +53 Elo (+34 à +73) | `docs/bots.md` |
| B3 3e gén. contre 2e gén. | 52,0 %, +14 Elo (+1 à +28) | `docs/bots.md` |
| B3 4e gén. contre 3e gén. | 46,9 %, −22 Elo : non retenue | `docs/bots.md` |
| Vitesse | B1 ≈ 0,5 ms par partie ; B3 ≈ 2,5 s par partie et par bot (≈ 5 s pour B3 contre B3) | `docs/bots.md` |

Tournoi B3 contre B3 du 2026-11-27, après les correctifs de B1 (moyenne
de chaque deck, ± 7 points) : Ira 70, Dromai 70, Dorinthea 69, Florian 68,
Bravo 63, Hala 47, Iyslander 41, Arakni 20, Levia 11
(`docs/analyses/2026-11-26-iyslander-hala-ira-b3.md`).

### Faiblesses connues

1. **B3 cherche avec B1.** Les parties simulées de la recherche, et les
   décisions de l'adversaire imaginé, sont jouées par B1, un bot à règles
   fixes qui ne lit pas le texte des cartes. Chaque défaut de B1 se
   retrouve dans B3. Preuve : corriger le go again de Zenith Blade dans B1
   a fait gagner +10 points à Hala jouée par B3 (108 parties).
2. **L'évaluation est apprise contre un ancien B1.** Les poids de la 3e
   génération ont été appris avant les correctifs de B1 du 2026-11-27 ; la
   4e génération, apprise après, est moins bonne (−22 Elo). La méthode
   (régression linéaire sur des caractéristiques simples) semble arrivée à
   son plafond : +46, puis +14, puis −22.
3. **Horizon court.** 300 itérations et 4 tours simulés par décision. La
   recherche ne regarde que la décision présente (profondeur 1) : elle ne
   planifie pas un tour entier (ordre des attaques, pitch, arsenal) comme
   un tout.
4. **Iyslander contre Arcane Barrier 3** : passé de 21 % à 41 % avec les
   sorts qui visent les dégâts qui passent la barrière et les gelures
   posées au tour adverse. Le pitch stacking, lui, n'apporte presque rien
   avec la façon dont B1 le fait (+0,3 à +0,9 point) : le vrai pitch
   stacking (préparer la main d'après pour un gros tour que AB3 ne peut pas
   couvrir) demande de planifier sur deux tours, ce que le bot ne fait pas.
5. **Levia** (11 %) : le blood debt est mal compris (tâche en cours dans un
   autre fil, `docs/taches.md`). **Arakni** (20 %) : pas encore analysé.
6. **Information cachée** : B2/B3 imaginent la main adverse au hasard dans
   le card-pool de 55, sans tenir compte de ce que l'adversaire a montré
   (ce qu'il a bloqué, pitché, gardé en main).
7. **Side** : la table `data/side/b1.txt` est apprise avec B1, pas avec B3.
8. **Ira** domine la méta réelle ; il était 4e en simulation, il est
   maintenant à égalité en tête (70 %) depuis les correctifs.

### Ce qu'on ne sait pas

**Personne ne sait aujourd'hui à quel niveau humain correspond B3.** Les
Elo ci-dessus comparent les bots entre eux, pas à des joueurs. Mon
estimation, **non mesurée** : la liste des faiblesses ci-dessus (surtout 1
et 3) laisse penser qu'un bon joueur de magasin bat B3 souvent. La
première tâche est donc de mesurer.

## 2. Comment mesurer « niveau top 8 » sans tournoi humain

Aucune mesure seule ne suffit. Je propose **cinq épreuves**, dont quatre
tournent sans le propriétaire et sans token une fois écrites.

| Épreuve | Ce qu'elle mesure | Seuil proposé « top 8 » | Coût après mise en place |
|---|---|---|---|
| **A. Puzzles à solution exacte** | jouer son tour au mieux : maximum de dégâts, létal trouvé, ordre des attaques, pitch | ≥ 95 % des puzzles d'attaque résolus au mieux ; 100 % des létaux trouvés | calcul local, quelques minutes |
| **B. Regret contre un « bot lent »** | qualité de chaque décision : sur une banque de positions de vraies parties, perte de chances de gagner par rapport au choix d'une recherche 30 à 100 fois plus longue | perte moyenne < 1 point de % par décision importante | calcul local, une nuit |
| **C. Avis du propriétaire** | ce qu'aucun calcul ne voit : le plan de jeu, les pièges, la gestion des ressources | sur 50 positions choisies, le bot fait le coup du propriétaire (ou un coup jugé aussi bon) dans ≥ 90 % des cas | temps du propriétaire, 0 token |
| **D. Parties contre le propriétaire** | le niveau réel contre un expert du jeu | ≥ 50 % de victoires en matchs en 2 manches gagnantes (indicatif : 30 matchs donnent ± 18 points) | temps du propriétaire, 0 token |
| **E. Exploitabilité** | le bot a-t-il un trou qu'un adversaire simple exploite ? | aucun bot « spécialisé contre B3 » ne dépasse 55 % | calcul local, une nuit |

Détails :

- **A. Puzzles.** Pendant son propre tour, la seule inconnue est la pioche
  (qui n'arrive qu'en fin de tour). On peut donc **énumérer toutes les
  façons de jouer le tour** (cartes, ordre, pitch, arme, arsenal) et
  connaître la meilleure contre une défense donnée. Les puzzles sont
  produits automatiquement à partir de positions de vraies parties : pas
  besoin de les écrire à la main, ni de tokens pour les faire tourner. Les
  15 puzzles de value de l'Arsenal JS (lot 6 du rapatriement) peuvent y
  entrer pour les cartes connues. Des puzzles de défense (que bloquer
  contre une attaque connue) suivent la même idée, avec une valeur estimée
  par simulation.
- **B. Regret.** On garde 2 000 positions de parties B3 contre B3. Pour
  chacune, une recherche très longue (30 000 itérations, parties simulées
  jusqu'à la fin) note toutes les options ; on mesure de combien le choix
  de B3 normal s'en écarte. Cela donne un chiffre par type de décision
  (défense, pitch, arsenal, cible…) et montre **où** le bot perd.
- **C. Avis du propriétaire.** La page web a déjà le « conseil du bot » et
  le signalement de bug (lot 9). Il suffit d'un écran « position à
  juger » : la page montre une position et le choix du bot, le propriétaire
  dit « d'accord » ou donne son coup. Chaque désaccord devient un test
  (règle du projet : un défaut signalé devient d'abord un test).
- **D. Parties contre le propriétaire.** Déjà possible avec `Jouer.bat`
  (bot niveau 3 par défaut). Il faudrait seulement que la page enregistre
  les résultats (deck, adversaire, gagnant) dans un fichier, pour suivre le
  score dans le temps.
- **E. Exploitabilité.** On apprend des poids de B3 **uniquement** sur des
  parties contre B3 figé ; si ce « B3 spécialisé » gagne nettement, B3 a un
  défaut systématique à corriger.

Une 6e source, gratuite, pour juger les decks plutôt que le jeu : **les
matchups réels selon le propriétaire.** S'il remplit la matrice 10 × 10
des matchups qu'il observe en tournoi (« Ira contre Dromai : 55 % »), on
la compare à la matrice B3 contre B3. Un écart de plus de 15 points sur une
case désigne un deck que le bot joue mal.

## 3. Les chantiers, par ordre d'impact

Unité de coût : une **session** = un fil court de Claude Code qui se
termine par une pull request. Le calcul local est donné en heures de
Surface (à confirmer : la Surface n'a pas encore été chronométrée ; je
compte ≈ 5 000 parties B3 contre B3 par heure sur ses 12 threads, contre
≈ 2 700 par heure dans le conteneur de Claude). Pour voir un gain de
+20 Elo avec certitude, il faut ≈ 2 600 parties, soit environ 30 minutes
de Surface.

| # | Chantier | Pourquoi | Sessions | Calcul local | Gain attendu (à mesurer) |
|---|---|---|---|---|---|
| 0 | **Script de nuit sur la Surface** (`tools/nuit.bat` + tâche planifiée Windows) : tournois, apprentissages, épreuves A/B/E, résultats écrits dans un fichier daté | tout le reste en dépend ; supprime les tokens dépensés à attendre des calculs | 1 | — | aucun directement ; divise le coût des chantiers suivants |
| 1 | **Épreuves A et B** (puzzles exacts du tour, banque de positions, regret contre bot lent) | sans mesure, on ne sait pas où le bot perd ni s'il progresse | 2 | 1 nuit | aucun directement ; donne la liste des défauts |
| 2 | **Planifier son tour en entier** : le solveur exact de l'épreuve A sert aussi à jouer (B3 choisit la meilleure séquence du tour au lieu d'une décision à la fois) | c'est le cœur du jeu à haut niveau en FaB (ordre des attaques, go again, pitch, garder une carte pour bloquer ou pour l'arsenal) ; règle aussi une partie du pitch stacking d'Iyslander | 2 à 3 | quelques nuits de mesures | le plus grand gain probable : la recherche actuelle ne fait que 4 tours avec B1 |
| 3 | **Remplacer B1 dans les parties simulées** par une politique apprise en imitant B3 (on enregistre les choix de B3 en auto-jeu, on apprend à les prédire ; aussi rapide que B1) | faiblesse n° 1 : B3 ne peut pas voir plus loin que la qualité de B1 | 2 à 3 | 2 à 4 nuits (auto-jeu + apprentissage) | important : chaque correctif de B1 a déjà donné +5 à +20 points sur un deck |
| 4 | **Niveau « réfléchi »** (voir plus bas) : jauge de réflexion à l'écran, recherche sur les 12 threads, temps de réflexion selon la complexité de la position (jusqu'à quelques minutes par tour), courbe force / temps de réflexion | accepté par le propriétaire le 2026-11-27 : un humain aussi prend son temps ; c'est le levier le moins cher | 1 à 1,5 | 1 à 2 nuits | à mesurer ; B2 passait de +122 à +283 Elo entre 30 et 300 itérations, et un tour de quelques minutes donne 100 à 1 000 fois le budget actuel |
| 5 | **Lire l'information montrée par l'adversaire** : main imaginée selon ce qu'il a bloqué, pitché, gardé ; card-pool pondéré par les listes du méta | un joueur de top 8 « lit » la main adverse | 1 à 2 | quelques heures | moyen |
| 6 | **Decks faibles** : Levia (en cours dans un autre fil), Arakni (analyse comme celle d'Iyslander), Aurora (tables et poids à faire quand le deck passe dans `data/decks/`) | un deck à 11-20 % fausse toute la matrice et l'apprentissage | 1 par deck | 1 nuit par deck | fort sur ces decks |
| 7 | **Évaluation plus riche** : paires de caractéristiques (« peut-il finir ce tour ? »), puis petit réseau de neurones sur processeur | la régression linéaire plafonne (+46, +14, −22) | 2 à 3 | plusieurs nuits | moyen, incertain |
| 8 | **Épreuves C, D, E** (écran « position à juger », journal des matchs, B3 spécialisé) | le juge final est le propriétaire | 1 à 2 | 1 nuit | aucun directement |
| 9 | **Side appris avec B3** au lieu de B1 (en partant de la table actuelle, sur les cartes qui changent vraiment le résultat) | la table actuelle reflète ce qui réussit à B1 | 1 | **coûteux** : refaire 20 000 parties par paire avec B3 prendrait des semaines ; version réduite (≈ 1 000 parties, 15 cartes en doute par paire) ≈ 4 à 6 nuits | faible à moyen |

### Le niveau « réfléchi » (accord du propriétaire, 2026-11-27)

Le propriétaire accepte un niveau de bot qui réfléchit **jusqu'à quelques
minutes par tour**, selon la complexité de la position. Cela change la
feuille de route sur six points.

1. **Deux bots au lieu d'un.** Un **B3 rapide** (≈ 2,5 s par partie, comme
   aujourd'hui) sert aux parties simulées, aux tournois et à
   l'apprentissage. Un **niveau réfléchi** joue contre le propriétaire et
   passe les épreuves. Tout ce qui améliore le bot rapide améliore aussi le
   niveau réfléchi, puisque l'un fait les parties simulées de l'autre.
2. **Le budget.** Aujourd'hui, une décision prend quelques millisecondes
   (300 itérations). Sur les 12 threads de la Surface, une minute par
   décision importante donne de l'ordre de **100 à 1 000 fois plus**
   d'itérations (à mesurer). Le temps est réparti selon la complexité :
   rien quand une seule option compte, quelques secondes pour un pitch
   évident, une à quelques minutes pour un tour d'attaque à beaucoup de
   séquences ou une défense contre une attaque mortelle.
3. **Ce qui redevient possible** avec ce budget, et doit être remesuré :
   un arbre plus profond (profondeur 2 ou 3, moins bon aujourd'hui faute
   d'itérations), des parties simulées plus longues (jusqu'à la fin), et
   surtout la **planification du tour entier** (chantier 2), qui peut
   énumérer toutes les séquences même pour Ira. Le pitch stacking
   d'Iyslander sur deux tours entre aussi dans ce budget.
4. **La mesure coûte plus cher.** Une partie complète au niveau réfléchi
   dure de 20 minutes à 1 heure : une nuit donne une dizaine de parties
   seulement, trop peu pour un Elo. Le niveau réfléchi se mesure donc
   **position par position** (épreuves A et B, sur des banques de positions
   gardées), et par les parties contre le propriétaire (épreuve D). Le
   « bot lent » de l'épreuve B doit alors réfléchir encore plus longtemps
   (une nuit pour quelques centaines de positions).
5. **Un nouveau levier : apprendre du niveau réfléchi.** La nuit, le niveau
   réfléchi joue des positions ; ses choix servent à entraîner le bot
   rapide (chantier 3 : imiter le niveau réfléchi plutôt que B3). C'est la
   méthode d'AlphaZero, en version modeste, sur processeur. Le bot rapide
   s'améliore, donc les parties simulées du niveau réfléchi aussi.

6. **Une jauge de réflexion à l'écran** (demande du propriétaire) : pendant
   que le bot réfléchit, la page montre une barre qui avance (temps écoulé
   sur le temps prévu pour cette décision) et, en petit, l'option que le
   bot préfère pour l'instant avec ses chances estimées. Le serveur
   `fab-web` envoie l'avancement de la recherche plusieurs fois par
   seconde ; un bouton « Joue maintenant » arrête la réflexion et prend le
   meilleur choix trouvé.

Coût : +1 à 1,5 session environ (recherche sur plusieurs threads, temps
selon la complexité, jauge et bouton « Joue maintenant » dans la page,
réglage dans `fab-ui`) ; le calcul est local.
Le niveau réfléchi passe dans l'étape « Mesurer », car il est le moins cher
des gros leviers.

À ne **pas** faire (déjà mesuré comme inutile) : double Ice Eternal, pitch
stacking « simple » de B1, 4e génération de poids sur la même méthode,
parties simulées par B0, arbre de recherche plus profond avec B1.

## 4. Ordre proposé et total

| Étape | Chantiers | Sessions | Calcul local | Point d'arrêt |
|---|---|---|---|---|
| **Mesurer** | 0, 1, 4 | ≈ 4 à 5 | 2 à 3 nuits | on connaît le taux de puzzles réussis et le regret par type de décision ; on décide la suite sur ces chiffres |
| **Gros leviers** | 2, 3 | ≈ 5 | 1 à 2 semaines de nuits | gain mesuré contre B3 actuel et sur les épreuves A/B |
| **Finitions** | 5, 6, 7 | ≈ 5 à 7 | 1 à 2 semaines de nuits | matrice B3 contre B3 proche de la matrice réelle du propriétaire |
| **Juger** | 8, puis 9 si besoin | ≈ 2 à 3 | 1 semaine de nuits | épreuves C, D, E atteintes |
| **Total** | | **≈ 17 à 20 sessions** | **≈ 4 à 6 semaines de nuits sur la Surface** | |

Repères : ces nombres sont des estimations, pas des mesures. La moitié du
travail (étape « Mesurer ») sert à savoir si la suite est nécessaire telle
quelle : après elle, la feuille de route sera refaite avec de vrais
chiffres. Le risque principal est le chantier 2 (planifier un tour entier
peut être lent pour les decks à beaucoup de choix, Ira par exemple) ; s'il
coûte trop cher, on le limite aux décisions d'attaque.

## 5. Ce qui ne coûte aucun token

- les nuits de calcul sur la Surface (une fois le script du chantier 0
  écrit) ;
- la matrice des matchups réels, les positions à juger et les parties
  contre le bot, par le propriétaire ;
- les analyses avec l'outil existant
  `cargo run --release -p fab-sim --bin analyse -- <deck>`.

Une session Claude n'est utile que pour écrire du code ou lire des
résultats. Pour économiser le quota : lancer le calcul la nuit, et ouvrir
une session le lendemain seulement pour lire le fichier de résultats.
