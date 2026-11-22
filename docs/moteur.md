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
- `Op::Ask(|g, c| …)` : pose une question ; la réponse arrive dans `c.answer` ;
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

## Les événements et le journal du tour

Tout ce qui se passe (carte jouée, attaque, défense, touche, pitch, dégâts,
destruction, bannissement…) est un `Event`. Il est ajouté au journal du tour
(`turn_log`) et déclenche les capacités concernées. Les conditions du type
« si vous avez joué une autre carte rouge ce tour » lisent ce journal.

## Vérifier

- Tests de règles : `crates/cards/tests/regles_*.rs` (un test par article du CR).
- Test massif : `cargo run --release -p fab-sim -- --games 100000 --check`
  (ajouter `--mecanismes` pour les decks d'essai qui utilisent préventions,
  remplacements, alliés et spectra).
