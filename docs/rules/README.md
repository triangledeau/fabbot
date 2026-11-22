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

Chaque section du CR implémentée, avec le fichier qui l'implémente et les tests
qui la vérifient (`crates/cards/tests/regles_etape1.rs`, tests nommés `cr_…`).

| Section | Sujet | Implémentation | Statut |
|---|---|---|---|
| CR 1.4.3 | Attaque-proxy d'une arme | `engine/src/play.rs`, `state.rs` | ✅ testé |
| CR 1.8.10 | « Votre prochaine attaque » | `engine/src/effects.rs` | ✅ testé |
| CR 1.10.2a | Fin de partie (héros à 0) | `engine/src/flow.rs` | ✅ testé |
| CR 1.10.2d / 6.6.6 | Déclenchements ajoutés à la pile | `flow.rs` | ⚠️ ordre simplifié (voir plus bas) |
| CR 1.11 | Priorité, passer, résolution | `flow.rs` | ✅ testé |
| CR 1.13 / 1.14 | Points d'action, ressources, pitch | `play.rs` | ✅ testé |
| CR 2.0.3c / 2.5.3e | Pas de valeur négative | `state.rs`, `effects.rs` | ✅ |
| CR 2.7.1 | Unicité d'une carte (nom + pitch) | `cards/src/decklist.rs` | ✅ testé |
| CR 3 | Zones, nouvel objet au changement de zone (3.0.9) | `state.rs` | ✅ |
| CR 4.1 | Mise en place, choix du premier joueur | `setup.rs` | ✅ testé (équipement/deck fournis, choix à l'étape 6) |
| CR 4.2 à 4.4 | Phases du tour, arsenal, pitch sous le deck, pioche | `flow.rs` | ✅ testé |
| CR 4.5.4d | Partie nulle par blocage (stalemate) | — | ❌ remplacé par une limite de tours du simulateur |
| CR 5.1 | Jouer une carte | `play.rs` | ✅ testé |
| CR 5.2 | Capacités activées (attaque d'arme) | `play.rs` | ✅ testé (autres capacités : étape 2) |
| CR 5.3 | Résolution, go again, réactions de défense | `flow.rs` | ✅ testé |
| CR 7.0 à 7.7 | Chaîne de combat, 6 étapes + fermeture | `flow.rs` | ✅ testé |
| CR 8.1 | Types : action, réactions, instant, bloc, ressource, arme, équipement | `play.rs`, `flow.rs` | ✅ testé |
| CR 8.2.1-2, 8.2.10 | 1H / 2H / off-hand | `setup.rs` | ✅ |
| CR 8.3.2 / 8.3.3 / 8.3.10 | Battleworn, Blade Break, Temper | `flow.rs` | ✅ testé |
| CR 8.3.4 | Dominate | `flow.rs`, `play.rs` | ✅ testé |
| CR 8.3.5 | Go again | `flow.rs` | ✅ testé |
| CR 8.5.3 / 8.5.6 / 8.5.7 | Dégâts, pioche, gain de vie | `effects.rs`, `setup.rs` | ✅ testé |
| CR 4.1.3 | Qui choisit le premier joueur (match) | `crates/fab-match` | ✅ testé |
| TRP 7.4 | Format Silver Age | `config/silver_age.toml` | ✅ testé |

### Simplifications connues de l'étape 1 (à traiter à l'étape 2)

- **Ordre des déclenchements simultanés** (CR 6.6.6b) : empilés dans l'ordre de
  création, sans choix des joueurs.
- **Blade Break / Battleworn / Temper** appliqués directement à la fermeture de
  la chaîne au lieu de passer par des couches déclenchées. Le résultat est le
  même tant qu'aucun autre déclenchement n'a lieu au même moment.
- **Informations de dernier état** (CR 1.2.3, 7.6.2a) : pas encore gérées.
- **Étape 7.2.2d** (l'attaque cesse d'exister avant les dégâts) : pas encore
  gérée, aucune carte d'essai ne peut provoquer ce cas.
- **Partie nulle par blocage** (CR 4.5.4d) : non détectée ; le simulateur
  arrête la partie après N tours (partie nulle « limite de tours », signalée
  comme telle dans les résultats).
- Mélange des decks fait avant le choix du premier joueur au lieu d'après
  (CR 4.1.8) : sans conséquence, le choix ne dépend pas de l'ordre du deck.
