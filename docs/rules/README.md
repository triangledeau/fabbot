# Sources des règles

Le moteur suit **uniquement** les sources officielles de Legend Story Studios.

| Source | Où la trouver | Consultée le |
|---|---|---|
| Comprehensive Rules (CR) | https://rules.fabtcg.com/en/cr/ | 2026-11-22 |
| Tournament Rules and Policy (TRP), dont 7.4 Silver Age | https://rules.fabtcg.com/en/trp/ | 2026-11-22 |
| Liste bannies / restreintes / héros au banc | fabtcg.com → Card Legality Policy | **non récupérée** (le site bloque les accès automatisés) |
| Textes officiels des cartes et errata | fabtcg.com / cards.fabtcg.com | à faire (étape 2) |

Le site ne montre pas de numéro de version du CR : on note la date de consultation.

**Copie locale** : `bash tools/fetch_rules.sh` télécharge tout dans
`docs/rules/sources/` (en HTML et en texte). Ce dossier n'est pas versionné :
on ne republie pas les documents de LSS.

## Format Silver Age (TRP 7.4) — vérifié

- Card-pool : **1 héros « young »** + **55 cartes d'arène et de deck au maximum**.
- Héros et cartes de rareté **commune ou rare**. Une carte compte comme commune
  si elle a déjà été imprimée en basic ou common, comme rare si elle a déjà été
  imprimée en rare (et n'est pas commune).
- **2 copies maximum** de chaque carte unique. Unicité = **nom + valeur de pitch** (CR 2.7.1).
- Deck de **exactement 40 cartes** au début de la partie.
- Carte « restricted » : 1 copie ; carte « banned » : interdite (TRP 7).
- Ces règles cèdent devant les capacités méta-statiques des cartes (ex. *Legendary*).

## Conséquences importantes pour le projet

1. **Il n'y a pas de sideboard séparé.** Avant **chaque** manche, y compris la
   première, chaque joueur choisit dans son card-pool de 55 cartes :
   - ses cartes d'arène (armes, équipements : 1 par zone) — CR 4.1.4 ;
   - puis les 40 cartes de son deck — CR 4.1.6 ;
   - le reste forme son « inventaire » — CR 4.1.7.

   Ordre officiel (CR 4.1) : choix du premier joueur → choix des cartes d'arène
   → révélation des cartes d'arène, **à commencer par le premier joueur** →
   choix du deck. Le bot choisit donc son deck **en connaissant** le héros
   adverse, son équipement et qui commence. C'est notre « phase de side ».
2. **Qui choisit le premier joueur** (CR 4.1.3) : tirage au sort à la première
   manche ; ensuite le perdant de la manche précédente ; après une nulle, le
   même joueur que la fois précédente. *(En tournoi, lors de la 1re manche d'une
   phase finale, c'est le mieux classé qui choisit — TRP 3.3.)*
   Implémenté dans `crates/fab-match`.
3. **Nombre de manches** : fixé par le tournoi, pas par le format (TRP 3).
   Les recherches web indiquent que fabtcg.com recommande pour le Silver Age des
   matchs **en une seule manche de 35 minutes**. **Non vérifié** : la page
   officielle n'a pas pu être lue. Le simulateur laissera ce réglage libre.

## Couverture des règles

Chaque section du CR utilisée par le moteur est listée ici, avec le fichier qui
l'implémente et les tests qui la vérifient. Ce tableau se remplit à l'étape 1.

| Section | Sujet | Implémentation | Tests | Statut |
|---|---|---|---|---|
| CR 2.7.1 | Unicité d'une carte (nom + pitch) | — | — | à faire |
| CR 4.1.3 | Qui choisit le premier joueur | `crates/fab-match/src/lib.rs` | 3 tests | ✅ |
| TRP 7.4 | Paramètres du format Silver Age | `config/silver_age.toml`, `crates/engine/src/format.rs` | 2 tests | ✅ |
