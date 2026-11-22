# Suivi du projet

Ce fichier est **le point de départ de chaque nouvelle conversation**. Il dit
où en est le projet, ce qui reste en suspens et quoi faire ensuite. Claude le
met à jour à la fin de chaque tâche.

## Où en est-on

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles + système d'effets | ✅ (manques connus listés dans `docs/rules/README.md`) |
| 2 | Deux premiers decks complets + interface texte | 🟡 cartes Hala et Dorinthea faites ; interface texte à faire |
| 3 | Les 7 autres decks + simulateur rapide | ⏳ |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |

**Branche la plus récente :** `claude/quirky-ritchie-a0y1et` (cartes de Hala
et Dorinthea, partie de `main` après la fusion de `claude/etape-1-moteur`).
Tant qu'elle n'est pas fusionnée dans `main`, partir de cette branche.

**Chiffres actuels :** 169 tests au vert. Vrais decks Hala contre Dorinthea
(`--vrais`) : 100 000 parties aléatoires sans incohérence, ≈ 7 600 parties/s
dans le conteneur de Claude (4 cœurs) ; Hala 49,8 %, Dorinthea 46,5 % entre
bots **aléatoires** (ce chiffre ne dit rien du vrai matchup). Decks d'essai :
100 000 + 50 000 (`--mecanismes`) parties sans incohérence.

**Choix provisoire d'équipement et de deck** (`real::default_loadout`, en
attendant la phase de side de l'étape 6) : la première carte de chaque
emplacement d'arène dans l'ordre de la liste (Hala : Helm of Unity, Edge Laden
Plate, Gauntlets of Unity, Pillar of Unity ; Dorinthea : Helm of Unity,
Plating of Unity, Gauntlets of Unity, Refraction Bolters), et les 40 premières
cartes de la liste.

## Questions ouvertes pour le propriétaire

1. **CR 7.0.1a ou CR 8.1.1b ?** À l'étape de résolution du combat, peut-on
   jouer une action **non-attaque** sans que la chaîne de combat se ferme
   d'abord ? Le moteur applique pour l'instant la lecture la plus stricte
   (7.0.1a : seulement des attaques). Voir `crates/engine/src/play.rs`
   (`timing_ok`).
2. **Liste officielle des bannissements Silver Age** (fabtcg.com, inaccessible
   aux outils automatiques) : à copier-coller si on veut la confirmer.
3. **Détruit puis banni à la place** : si un objet avec « If this would be put
   into your graveyard, instead banish it » (Embraforged Gauntlet) est
   détruit, compte-t-il comme « détruit » (déclencheurs « when this is
   destroyed ») ? Le moteur dit oui. Voir `effects.rs` (`destroy`).
4. **Ward quand il ne reste plus de dégâts** : avec deux wards (ou un bouclier
   et un ward) et une attaque déjà entièrement prévenue, le moteur ne détruit
   pas le ward restant. Est-ce correct (CR 6.4.2 : une prévention n'est
   « active » que s'il reste quelque chose à prévenir) ?
5. **Steelblade Shunt** (« If this defends a weapon attack, deal 1 damage to
   the attacking hero ») : la condition est vérifiée à la résolution de la
   carte, juste avant qu'elle devienne une carte qui défend, en regardant
   l'attaque du maillon actif. Correct ?
6. **Shove Off** (« Choose a non-equipment card defending a sword attack you
   control ») : le moteur propose aussi les cartes qui défendent sur les
   **maillons précédents** (CR 7.0.5a : une carte défend tant qu'elle est sur
   la chaîne). Ou seulement le maillon actif ?
7. **Decimator Great Axe** (« halve the base {d} of target defending card ») :
   la cible peut être n'importe quelle carte qui défend sur la chaîne, y
   compris un équipement ou une carte d'un maillon précédent (lecture
   littérale). Correct ?
8. **Flurry + Dorinthea** : le moteur applique CR 5.2.3e — Flurry fixe la
   limite à 2 attaques, Dorinthea en ajoute 1 → 3 attaques possibles avec la
   même arme ce tour. Correct ?
9. **On the Horizon** (« look at the top card of your deck ») : pour l'instant
   sans effet dans le moteur (regarder ne change pas l'état). Il faudra
   l'information « ce que chaque joueur sait » pour les bots B2+ et
   l'interface.

## Prochaine tâche

Étape 2 (fin) : **interface texte** pour jouer contre le bot (crate `fab-ui`) :
afficher l'état visible du joueur, la décision en cours et ses options,
répondre par un numéro ; bot B0 (aléatoire) en face, decks Hala ou
Dorinthea. Le propriétaire peut en profiter pour vérifier les cartes et les
passer en `Validated`.

Ensuite (étape 3) : les 7 autres decks. Avant Dromai, il faudra ajouter au
moteur les **capacités accordées par un effet** (Storm of Sandikai donne une
capacité d'attaque aux dragons) ; vérifier « clash » et « crush » au fil des
cartes (aucune carte de Hala ou Dorinthea ne les utilise).

## Journal des sessions

| Date | Travail | Branche |
|---|---|---|
| 2026-11-22 | Étape 0 : squelette, règles Silver Age officielles, 9 listes de decks, source officielle des cartes | `claude/squelette-projet`, `claude/listes-decks` |
| 2026-11-22 | Étape 1 : cœur des règles (tour, priorité, combat), puis système d'effets suspendable | `claude/etape-1-regles`, `claude/etape-1-suite` |
| 2026-11-22 | Fin de l'étape 1 : arcane barrier / ward / spellvoid, remplacements, attaques contre alliés et spectra, perte des capacités, durées « prochain tour », étape de fermeture avec déclenchements | `claude/etape-1-moteur` |
| 2026-11-22 | Étape 2 : toutes les cartes de Hala et Dorinthea (55 lignes de STATUS.md + token Flurry) codées et testées (73 tests de cartes) ; moteur : sharpen, effets « prochaine attaque de cette arme », coût d'activation réduit, attaques supplémentaires (CR 5.2.3), paiement « unless you pay », renvoi d'une carte qui défend, fin de la marque (CR 9.3.3) ; `tools/check_coded_cards.py` ; `fab-sim --vrais` | `claude/quirky-ritchie-a0y1et` |
