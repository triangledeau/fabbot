# Sources des règles

Le moteur suit **uniquement** les sources officielles de Legend Story Studios.

| Source | Où la trouver | Consultée le |
|---|---|---|
| Comprehensive Rules (CR) | https://rules.fabtcg.com/en/cr/ | 2026-11-22 |
| Tournament Rules and Policy (TRP), dont 7.4 Silver Age | https://rules.fabtcg.com/en/trp/ | 2026-11-22 |
| Liste bannies / restreintes / héros au banc | fabtcg.com → Politique de légalité des cartes | copiée à la main par le propriétaire le 2026-11-22 (`config/silver_age.toml`) ; à mettre à jour à chaque saison |
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
| CR 1.10.2d / 6.6.6 | Déclenchements : cibles, ordre choisi par les joueurs | `flow.rs`, `events.rs` | ✅ testé |
| CR 1.7.4 / 5.4 | Effets statiques (carte, héros, équipements, permanents) | `state.rs` | ✅ testé |
| CR 1.7.5 | Modes (« Choose 1 ») | `play.rs` | ✅ testé |
| CR 1.8.5 / 1.8.9 | Cibles, cible devenue illégale | `play.rs`, `flow.rs` | ✅ testé |
| CR 1.8.6 | Choix pendant la résolution | `script.rs`, `flow.rs` | ✅ testé |
| CR 1.3.3 / 5.3.6a | Permanents (auras, objets) | `flow.rs` | ✅ testé |
| CR 5.1.3 | Coûts optionnels, X | `play.rs` | ✅ testé (X : pas encore de test) |
| CR 5.1.8-5.1.9 | Coûts d'effet | `play.rs` | ✅ testé |
| CR 5.2 | Capacités activées (équipement, objet, main, héros) | `play.rs` | ✅ testé |
| CR 6.4.10 / 6.5.5 | Prévention des dégâts : boucliers, préventions fixes, ordre au choix du joueur | `effects.rs` | ✅ testé |
| CR 8.3.8 / 8.3.15 / 8.3.20 | Arcane barrier (payer), spellvoid (détruire, optionnel), ward (obligatoire) | `effects.rs` | ✅ testé |
| CR 6.4 / 6.5 | Effets de remplacement (« instead ») : cimetière, dessus du deck, dégâts, création de tokens ; statiques ou créés par un effet ; optionnels (« you may ») | `replace.rs` | ✅ testé |
| CR 8.3.21 | Ephemeral (retiré de la partie au lieu du cimetière) | `replace.rs` + carte | ✅ testé |
| CR 1.4.5 / 7.3.2a / 8.1.3a | Cible d'attaque au choix (héros, allié, spectra) ; personne ne défend un allié | `play.rs`, `flow.rs` | ✅ testé |
| CR 2.5.3 / 1.10.2b | Vie des alliés, allié à 0 PV « clear » | `state.rs`, `flow.rs` | ✅ testé |
| CR 8.3.14 | Spectra | `play.rs`, `flow.rs` | ✅ testé |
| CR 6.3.6 | « Loses all abilities » (capacités imprimées seulement) | `state.rs`, `events.rs`, `play.rs` | ✅ testé |
| CR 6.2.2a | Durée « until the end of their next turn » ; « during their next turn » | `state.rs`, `effects.rs` | ✅ testé |
| CR 8.5.3 | Dégâts arcaniques | `effects.rs` | ✅ testé |
| CR 1.11 | Priorité, passer, résolution | `flow.rs` | ✅ testé |
| CR 1.13 / 1.14 | Points d'action, ressources, pitch | `play.rs` | ✅ testé |
| CR 2.0.3c / 2.5.3e | Pas de valeur négative | `state.rs`, `effects.rs` | ✅ |
| CR 2.7.1 | Unicité d'une carte (nom + pitch) | `cards/src/decklist.rs` | ✅ testé |
| CR 3 | Zones, nouvel objet au changement de zone (3.0.9) | `state.rs` | ✅ |
| CR 4.1 | Mise en place, choix du premier joueur | `setup.rs` | ✅ testé (équipement/deck fournis, choix à l'étape 6) |
| CR 4.2 à 4.4 | Phases du tour, arsenal, pitch sous le deck, pioche | `flow.rs` | ✅ testé |
| CR 4.5.4d | Partie nulle par blocage (stalemate) | `flow.rs` | ⚠️ cas simple seulement (decks, mains, arsenaux vides) |
| CR 5.1 | Jouer une carte | `play.rs` | ✅ testé |
| CR 5.3 | Résolution, go again, réactions de défense | `flow.rs` | ✅ testé |
| CR 7.0 à 7.7 | Chaîne de combat, 6 étapes + fermeture (déclenchements résolus pendant l'étape de fermeture, avant le retour des permanents) | `flow.rs` | ✅ testé |
| CR 8.1 | Types : action, réactions, instant, bloc, ressource, arme, équipement | `play.rs`, `flow.rs` | ✅ testé |
| CR 8.2.1-2, 8.2.10 | 1H / 2H / off-hand | `setup.rs` | ✅ |
| CR 8.3.2 / 8.3.3 / 8.3.10 / 8.3.34 | Battleworn, Blade Break, Temper, Guardwell (vrais déclenchements de fermeture) | `flow.rs` | ✅ testé |
| CR 8.3.4 | Dominate | `flow.rs`, `play.rs` | ✅ testé |
| CR 8.3.5 | Go again | `flow.rs` | ✅ testé |
| CR 8.5.3 / 8.5.6 / 8.5.7 | Dégâts, pioche, gain de vie | `effects.rs`, `setup.rs` | ✅ testé |
| CR 8.5.45 / 8.5.17 | Clash (cartes révélées, puissance imprimée, gagnant) ; « when you win a clash revealing this » (CR 1.7.4i) | `effects.rs`, `events.rs` | ✅ testé |
| CR 8.4.2 | Crush (déclenché par les **dégâts** infligés, pas le hit) | `cards/src/real/guardian.rs` | ✅ testé |
| CR 8.3.18 / 5.4.7b | Heave (capacité déclenchée cachée depuis la main) | `events.rs` + carte | ✅ testé |
| CR 8.3.42 | Suspense (2 compteurs, un retiré à chaque début de tour, détruit à 0) | `state.rs`, `flow.rs` | ✅ testé |
| CR 5.1.6 | Réduction « your next … costs {r} less » (Seismic Surge) | `play.rs`, `effects.rs` | ✅ testé |
| CR 8.3.4 | « Loses and can't gain dominate » | `state.rs` | ✅ testé |
| CR 4.1.3 | Qui choisit le premier joueur (match) | `crates/fab-match` | ✅ testé |
| TRP 7.4 | Format Silver Age | `config/silver_age.toml` | ✅ testé |

### Simplifications et manques connus

- **Ordre de plusieurs remplacements obligatoires** sur le même événement
  (CR 6.5.1-6.5.4) : le moteur applique ceux des cartes puis ceux des effets,
  dans l'ordre de création, sans demander. Aucun cas dans les 9 decks où
  l'ordre change le résultat.
- **Remplacements optionnels** : seulement pour la création de tokens (ex.
  Smoldering Scales). Les déplacements (cimetière, deck) et les dégâts n'en
  acceptent que d'obligatoires (aucune carte des 9 decks n'en a d'autres).
- **Préventions des dégâts infligés à un allié** : non gérées (aucune carte
  des 9 decks n'en crée).
- **Arcane barrier / ward / spellvoid de même type sur un même objet** (valeur
  imprimée + valeur accordée) : additionnées en une seule capacité.
- **Prévention au choix** : quand les dégâts restants tombent à 0, plus aucune
  prévention n'est appliquée (un ward n'est pas détruit pour rien).
- **Capacités accordées** : un effet peut donner des mots-clés et des valeurs
  (ward, arcane barrier, spellvoid), pas encore des capacités activées ou
  déclenchées (ex. Storm of Sandikai : « Dragon allies you control get
  "Once per Turn Action — 0: Attack" ») ni spectra.
- **« Loses all abilities »** : l'ordre chronologique entre la perte et une
  capacité accordée plus tôt n'est pas pris en compte (CR 6.3.4).
- **Fermeture de la chaîne par un effet** (CR 7.7.2c-d : attaque détruite
  avant les dégâts, effet « close the combat chain ») : pas encore gérée.
- **Coût d'effet impossible à payer** (CR 1.14.4b, 5.1.8a) : pas de retour en
  arrière ; chaque carte doit vérifier à l'avance qu'elle peut payer
  (`play_condition`).
- **Informations de dernier état** (CR 1.2.3) : seulement pour go again à
  l'étape de résolution (CR 7.6.2a).
- **Partie nulle par blocage** (CR 4.5.4d) : seul le cas simple est détecté ;
  sinon le simulateur arrête la partie après N tours (« limite de tours »).
- **Heave** (CR 5.4.7b) : le choix « déclencher ou non » de la capacité
  cachée est posé à la résolution (la pile de la phase de fin se résout sans
  priorité, personne ne peut répondre entre les deux). Si le joueur refuse,
  l'adversaire a vu passer la carte : petite fuite d'information.
- **« Can't gain {p} »** (Chokeslam) : les bonus de puissance (effets, auras,
  « this gets +1{p} ») sont ignorés, les malus restent ; les compteurs +1{p}
  ne sont pas concernés (aucune carte d'attaque des 9 decks n'en reçoit).
- Mélange des decks fait avant le choix du premier joueur au lieu d'après
  (CR 4.1.8) : sans conséquence, le choix ne dépend pas de l'ordre du deck.
