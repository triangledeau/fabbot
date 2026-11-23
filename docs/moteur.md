# Le moteur en bref

Ce document explique comment le moteur est organisé, pour pouvoir coder les
cartes (étape 2) et pour que chaque nouvelle session de Claude Code s'y retrouve.

## Le principe : des décisions

Le moteur avance seul jusqu'à ce qu'un joueur doive choisir. Il expose alors
une `Decision` (qui choisit, quel type de choix, quelles options). Le bot ou
l'humain répond avec l'index d'une option (`GameState::apply`). Quand la seule
option est « passer », le moteur passe tout seul.

## La pile de tâches

Certaines actions demandent plusieurs choix successifs. Jouer une carte, par
exemple : coût optionnel, X, mode, cibles, cartes à pitcher. Une capacité peut
aussi poser une question pendant sa résolution. Le moteur garde donc une
**pile de tâches** (`GameState::tasks`) ; la tâche du dessus passe toujours
avant le reste. Une tâche peut s'arrêter pour attendre une réponse, puis
reprendre exactement où elle en était.

## Écrire une carte

Une carte est une `CardDef` (dans `crates/engine/src/card.rs`) :

| Champ | Sert à | Exemple réel |
|---|---|---|
| `pitch`, `cost`, `power`, `defense`, `types`, `subtypes`, `supertypes`, `keywords` | valeurs imprimées | toutes |
| `targets` | cibles déclarées en jouant la carte (CR 5.1.4) | « Target attack gets +3 » |
| `modes` | « Choose 1 » (CR 1.7.5) | Razor Reflex, Pummel |
| `additional_cost`, `effect_cost` | coûts additionnels (CR 5.4.4a) | Hurl, Wrecker Romp, Boneyard Marauder |
| `x_cost` | coût X | Ice Eternal |
| `play_condition` | « Play this only if … » | Graveling Growl, Fatal Engagement |
| `cost_mod` | « This costs {r} less … » | Amplify the Arknight |
| `resolve` | effets à la résolution | Sift, Frosting |
| `triggers` | « When … », « At the beginning of … » | Snatch, Sirens of Safe Harbor |
| `activated` | « Instant — Destroy this: … », attaques d'armes | Timesnap Potion, Edge of Autumn |
| `self_mod` | « this gets +N / go again if … » | Blaze Headlong, Flying Kick |
| `aura` | effets statiques sur d'autres objets (héros, équipements, auras) | Ira, Blade Beckoner |
| `arcane_barrier`, `ward`, `spellvoid` | préventions au choix, valeur imprimée | Nullrune Hood, Silken Shroud, Mask of the Swarming Claw |
| `replacements` | « If … would …, instead … » (remplacements statiques) | Embraforged Gauntlet, Florian, Smoldering Scales |

Les effets sont de petits programmes (`Op`, voir `crates/engine/src/script.rs`) :

- `Op::Do(|g, c| …)` : du code Rust qui modifie la partie avec les primitives
  de `crates/engine/src/effects.rs` (`draw`, `deal_damage`, `destroy`,
  `banish`, `modify`, `next_attack`, `prevent_next`, `create_token`…) ;
- `Op::Ask(|g, c| …)` : pose une question (`Ask { player, label, options }`, le
  `label` est le texte affiché par l'interface) ; la réponse arrive dans `c.answer`.
  La fonction doit rester **pure** (sans effet sur la partie) : l'interface la
  rappelle pour retrouver le libellé ;
- `Op::If(condition, n)` / `Op::Back(condition, n)` : sauter / boucler.

Exemples complets : les **vraies cartes** de `crates/cards/src/real/`
(`generic.rs`, `warrior.rs`), chacune précédée de son texte officiel, et les
cartes fictives de `crates/cards/src/test_set.rs`.

Pour une carte qui existe en rouge / jaune / bleu, un seul code sert aux trois
couleurs : `by_color(g, c, [rouge, jaune, bleu])` lit la valeur selon la
couleur de la carte (ex. « +3{p} / +2{p} / +1{p} »).

Après avoir codé une carte : `python3 tools/check_coded_cards.py` compare ses
valeurs imprimées (pitch, coût, puissance, défense, types, mots-clés) au
fichier officiel.

## Dégâts, préventions et remplacements

`deal_damage` / `deal_damage_to` ne modifient pas la vie tout de suite : les
dégâts passent d'abord par les **remplacements** (« deals that much plus 3 »),
puis par les **préventions** (boucliers, arcane barrier, ward, spellvoid). Si
le joueur protégé a un choix à faire (payer l'arcane barrier, détruire pour le
spellvoid, ordre des préventions), les dégâts attendent sa réponse. Le
résultat (« if damage is dealt this way ») est dans `g.last_damage`, à lire
dans l'instruction **suivante** du script. Même chose pour `create_tokens` et
`g.last_created`.

Les remplacements créés par une capacité (Topsy Turvy, Stir the Aetherwinds…)
s'ajoutent avec `add_replacement` ; les préventions avec `prevent_next`
(« prevent the next N »), `prevent_next_time` (« the next time … prevent N »)
ou `add_prevention` (filtre de source, « if you prevent damage this way »).

## Attaques, alliés, perte des capacités

- En jouant une attaque, le joueur choisit sa cible (`DecisionKind::AttackTarget`)
  s'il y a un allié ou un objet avec spectra en face. `ChainLink::target_obj`
  et les événements `Attack` / `Hit` / `Damage` (`target_obj`) disent si
  c'est un héros (`None`) ou un objet (« when this hits a hero »).
- « Loses all abilities » : `modify` avec `Delta { lose_abilities: true, .. }`.
- « Until the end of their next turn » : `Until::EndOfTurnNumber(g.next_turn_of(p))`,
  ou `g.end_of_next_turn(p)`. « Their first attack during their next turn » :
  `next_attack_on_turn`.
- Fermeture de la chaîne : les capacités « when the combat chain closes »
  (Hell Hammer, blade break…) se résolvent **avant** que les permanents
  reviennent dans leurs zones (CR 7.7.4) ; `g.combat.links` est encore
  disponible pour savoir qui a attaqué ou défendu.

## Armes, sharpen, effets de tour

- `sharpen(p, arme)` (CR 8.5.58) : un compteur +1{p}, retiré au début de la
  phase de fin (déclenchement retardé `RuleTrigger::Unsharpen`). Passe par les
  remplacements (`ReplaceKind::Sharpen`, ex. Swordmaster's Path : un sharpen
  de plus). « Has been sharpened this turn » : `sharpened_this_turn(id)`,
  `player_sharpened_this_turn(p, filtre)`.
- Une attaque d'arme est une **attaque-proxy** (`obj.proxy_of = Some(arme)`) ;
  `is_weapon_attack(id)`. L'événement `Attack` / `Hit` d'une proxy désigne
  l'**arme** ; l'événement `Defend` désigne la **proxy**. Pour une proxy,
  `obj.counters` = numéro de l'attaque de cette arme ce tour (1 = première).
- « Its next attack this turn » : `next_attack_with(p, arme, delta)`.
- `add_weapon_effect(arme, …)` (jusqu'à la fin du tour) : `CostLess(n)`
  (« your next attack with it costs {r} less », consommé au paiement),
  `AttackTwice` (Flurry, CR 5.2.3e), `AdditionalAttack` (Dorinthea, CR 5.2.3).
  Voir `activation_cost` et `activation_limit`.
- « … unless you pay {r} » : `pay(p, n)` (le joueur pitche si besoin).
- Renvoyer une carte qui défend dans la main : `return_to_hand(id)`.
- Marque : CR 9.3.3, un héros marqué touché par une source adverse n'est plus marqué.

## Bannissement, capacités accordées, ambush (étape 3)

- « Create a Crouching Tiger in your banished zone » : `create_card_banished(p, carte)`
  (une carte créée, pas un token) ; « you may play it this turn / during your
  next turn » : `permit_play(p, objet, tour)` (`g.turn` ou `g.next_turn_of(p)`).
  Les cartes permises apparaissent dans les options de priorité (`Choice::Play`).
- Ephemeral (`Keywords::EPHEMERAL`, CR 8.3.21) : au lieu d'aller au cimetière,
  la carte est retirée de la partie (`Zone::Gone`).
- Ambush (`Keywords::AMBUSH`, CR 8.3.28) : la carte de l'arsenal est proposée
  pour défendre.
- « Target attack gets "When this hits, …" » : la capacité accordée est écrite
  dans `CardDef::granted` de la carte qui l'accorde, puis
  `grant_trigger(cible, c.card, index, durée)`. Sa couche est
  `LayerKind::Granted` ; `c.source` y désigne l'objet qui a reçu la capacité.
- « The next Crouching Tiger you play this combat chain » : `next_attack_this_chain`.
- Carte à modes dont la cible dépend du mode (Razor Reflex) : le filtre de
  cible lit `c.mode` ; seuls les modes qui ont une cible sont proposés.

## Clash, crush, heave, suspense (étape 3, deck Bravo)

- **Clash** (CR 8.5.45) : `g.clash(a, b)` révèle le dessus des deux decks
  (événements `Revealed`), compare la puissance **imprimée** (CR 1.7.4) et
  renvoie le gagnant (événement `ClashWon`), ou `None`.
- « When you win a clash revealing this » (Thunk) : déclenchement `anywhere`
  sur `Event::ClashWon { card == me }` ; une capacité `anywhere` fonctionne
  aussi dans une zone privée (main, deck) et vérifie elle-même sa zone.
- **Crush** (CR 8.4.2) : capacité déclenchée sur `Event::Damage` (dégâts
  réellement infligés, ≥ 4, à un héros). Le drapeau `Keywords::CRUSH` dit
  qu'une carte « a crush » (Bravo, Crash and Bash).
- **Heave** (CR 8.3.18) : déclenchement `anywhere` depuis la main au début de
  la phase de fin (voir Thunder Quake) ; `put_in_arsenal` puis `turn_face_up`.
- **Suspense** (`Keywords::SUSPENSE`, CR 8.3.42) : géré par le moteur
  (compteurs dans `Object::counters`).
- « Your next Guardian attack action card costs {r} less » :
  `next_card_costs_less(p, filtre, n)` (Seismic Surge).
- « Can't gain {p} during their next action phase » :
  `forbid_power_gain_next_action_phase(p)` (Chokeslam).
- « Loses and can't gain dominate » : `Delta { loses: Keywords::DOMINATE, .. }`.

## Instant, Frostbite, amp, tap (étape 3, deck Iyslander)

- **« As though it were an instant »** (CR 8.1.1d) : capacité statique
  `CardDef::play_as_instant` (Iyslander : source, carte) ou permission
  `next_card_as_instant(p, filtre)` (Stir the Aetherwinds, utilisée par la
  prochaine carte jouée qui vérifie le filtre). `Ctx::as_instant` : la carte
  ne coûte pas de point d'action.
- **Surcoût** « cards and abilities cost you an additional {r} » :
  `CardDef::cost_tax` (Frostbite), ajouté dans `play_cost` et
  `activation_cost` avant les réductions (CR 5.1.6a).
- **Coût « XX »** : `CardDef::x_twice` (Ice Eternal, X+X).
- **« The next card you play this turn … deals that much plus N »** :
  remplacement de dégâts avec `Replacement::bind` (filtre) ; la carte est
  mémorisée dans `data` quand elle est jouée, `value` porte le +N
  (Crucible of Aetherweave, Stir the Aetherwinds). `amp(p, n)` (CR 8.5.47).
- **Tap** : `tap(id)` renvoie `false` si l'objet est déjà engagé (CR 8.5.55a).
- **Zones exposées** : `exposed_zones()`, `create_token_in_zone(p, carte,
  zone)` (Frost Spike, CR 8.5.2c) ; le choix d'une zone est `Choice::Zone`.
- **Prévention statique** « If you would be dealt arcane damage, prevent N » :
  `CardDef::prevent_arcane` (Pyroglyphic Protection), `PreventRef::Static`.
- « Any target » / « target hero » : `wizard::ANY_TARGET`, `wizard::TARGET_HERO`.
- « Has an arcane damage effect » : `CardDef::arcane_damage`, à cocher sur
  chaque carte qui inflige des dégâts arcaniques.

## Cartes doubles, meld, decompose, Runechant (étape 3, deck Florian)

- **Carte double** (split-card, CR 9.2, ex. Arcane Seeds // Life) : quatre
  définitions (la carte entière, le côté gauche, le côté droit, la version
  fusionnée), reliées par `CardDef::split` ([`Split`]). Hors de la pile, la
  carte a les propriétés des deux côtés (CR 9.2.2). En la jouant, le joueur
  choisit un côté jouable (`Task::PlaySide`, réponse `Choice::Mode`, libellés
  dans `modes` de la carte entière) ; sur la pile, l'objet pointe vers la
  définition du côté choisi (CR 9.2.3) et revient à la carte entière dès
  qu'il quitte la pile (`move_to`).
- **Meld** (CR 8.3.38) : la version fusionnée a les types des deux côtés ;
  c'est une action (1 point d'action, pile vide). `Ctx::melded` : elle se
  résout d'abord avec le côté droit, le joueur actif reçoit la priorité,
  puis le côté gauche (CR 5.3.4d, dans `finish_resolve`).
- `Event::Played { def, .. }` garde la définition **telle qu'elle a été
  jouée** ; « if you've played a non-attack action card this turn » :
  `played_def_this_turn`.
- **Cibles dans un cimetière** (CR 1.8.5a) : `TargetSpec::in_graveyard`.
- **Supertypes variables** : `GameState::supertypes(id)` ajoute
  `CardDef::face_up_supertypes` tant que la carte est face visible (Colors
  of Aria est Earth, Ice et Lightning).
- **Compteurs à l'entrée** : `CardDef::enter_counters` (« enters the arena
  with N verse counters », compteurs dans `Object::counters`).
- **Créateur d'un token** : `create_tokens_by(créateur, contrôleur, …)` ;
  `Pending::by` permet à Florian de ne doubler que les tokens qu'il crée
  lui-même (CR 8.5.2c).
- **« Pitched this way »** (Annals of Sutcliffe) : `Ctx::log_start` donne
  l'index du journal du tour au moment où la carte est jouée ou la capacité
  activée ; les événements `Pitched` qui suivent viennent du paiement.
- **Decompose** (CR 8.4.14) : `runeblade::decompose()` ; seules les cartes
  qui permettent encore de compléter « 2 Earth + 1 action » (3 cartes
  distinctes) sont proposées.

## Les événements et le journal du tour

Tout ce qui se passe (carte jouée, attaque, défense, touche, pitch, dégâts,
destruction, bannissement…) est un `Event`. Il est ajouté au journal du tour
(`turn_log`) et déclenche les capacités concernées. Les conditions du type
« si vous avez joué une autre carte rouge ce tour » lisent ce journal.

## Interface

`GameState::record` : journal complet facultatif des événements (`None` par
défaut, donc sans coût pour les simulations). L'interface texte (`crates/ui`,
voir `docs/interface.md`) l'active pour raconter la partie, y compris ce qui
se passe en fin de tour, quand `turn_log` est vidé.

## Vérifier

- Tests de règles : `crates/cards/tests/regles_*.rs` (un test par article du CR).
- Test massif : `cargo run --release -p fab-sim -- --games 100000 --check`
  (ajouter `--mecanismes` pour les decks d'essai qui utilisent préventions,
  remplacements, alliés et spectra ; `--decks ira hala` pour deux vrais decks).
