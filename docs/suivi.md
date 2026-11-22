# Suivi du projet

Ce fichier est **le point de départ de chaque nouvelle conversation**. Il dit
où en est le projet, ce qui reste en suspens et quoi faire ensuite. Claude le
met à jour à la fin de chaque tâche.

## Où en est-on

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles + système d'effets | 🔶 presque fini (voir « Prochaine tâche ») |
| 2 | Deux premiers decks complets + interface texte | ⏳ |
| 3 | Les 7 autres decks + simulateur rapide | ⏳ |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |

**Branche la plus récente :** `claude/etape-1-suite` (contient tout le travail
précédent). Tant qu'elle n'est pas fusionnée dans `main`, partir de cette branche.

**Chiffres actuels :** 62 tests au vert ; 500 000 parties aléatoires sans
incohérence ; environ 18 000 parties/s (bots aléatoires, 4 cœurs).

## Questions ouvertes pour le propriétaire

1. **CR 7.0.1a ou CR 8.1.1b ?** À l'étape de résolution du combat, peut-on
   jouer une action **non-attaque** sans que la chaîne de combat se ferme
   d'abord ? Le moteur applique pour l'instant la lecture la plus stricte
   (7.0.1a : seulement des attaques). Voir `crates/engine/src/play.rs`
   (`timing_ok`).
2. **Liste officielle des bannissements Silver Age** (fabtcg.com, inaccessible
   aux outils automatiques) : à copier-coller si on veut la confirmer.

## Prochaine tâche

Compléter le moteur avant de coder les vraies cartes (détails dans
`docs/rules/README.md`, section « Simplifications et manques connus ») :

1. Préventions au choix du joueur : **arcane barrier**, **ward**, **spellvoid**.
2. **Effets de remplacement** (« instead »), indispensables pour Levia.
3. **Attaquer autre chose qu'un héros** (alliés, dragons, spectra).
4. **« Loses all abilities »** et effets « jusqu'à la fin de leur prochain tour ».
5. **Déclenchements à la fermeture de la chaîne** (CR 7.7.3-7.7.4), ex. Hell Hammer.

Ensuite : étape 2, coder les vraies cartes, en commençant par **Hala et
Dorinthea** (deux Warriors qui partagent beaucoup de cartes et peu de
mécaniques exotiques).

## Journal des sessions

| Date | Travail | Branche |
|---|---|---|
| 2026-11-22 | Étape 0 : squelette, règles Silver Age officielles, 9 listes de decks, source officielle des cartes | `claude/squelette-projet`, `claude/listes-decks` |
| 2026-11-22 | Étape 1 : cœur des règles (tour, priorité, combat), puis système d'effets suspendable | `claude/etape-1-regles`, `claude/etape-1-suite` |
