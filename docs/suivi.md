# Suivi du projet

Ce fichier est **le point de départ de chaque nouvelle conversation**. Il dit
où en est le projet, ce qui reste en suspens et quoi faire ensuite. Claude le
met à jour à la fin de chaque tâche.

## Où en est-on

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles + système d'effets | ✅ (manques connus listés dans `docs/rules/README.md`) |
| 2 | Deux premiers decks complets + interface texte | ⏳ |
| 3 | Les 7 autres decks + simulateur rapide | ⏳ |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |

**Branche la plus récente :** `claude/etape-1-moteur` (contient tout le travail
précédent, dont `claude/etape-1-suite`). Tant qu'elle n'est pas fusionnée dans
`main`, partir de cette branche.

**Chiffres actuels :** 95 tests au vert ; 200 000 parties aléatoires sans
incohérence (dont 100 000 avec les nouveaux mécanismes, `--mecanismes`).
Vitesse mesurée dans le conteneur de Claude (4 cœurs) : environ 7 000
parties/s, contre 8 000 pour la version précédente sur la même machine
(≈ 7 % d'instructions en plus, coût des nouveaux mécanismes). Le chiffre de
18 000 parties/s noté avant a sans doute été mesuré dans un autre conteneur.

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

## Prochaine tâche

Étape 2 : coder les vraies cartes, en commençant par **Hala et Dorinthea**
(deux Warriors qui partagent beaucoup de cartes et peu de mécaniques
exotiques). Avant Dromai, il faudra ajouter au moteur les **capacités
accordées par un effet** (Storm of Sandikai donne une capacité d'attaque aux
dragons) ; avant Hala/Dorinthea, vérifier les mécaniques « clash »,
« sharpen » et « crush » au fil des cartes.

## Journal des sessions

| Date | Travail | Branche |
|---|---|---|
| 2026-11-22 | Étape 0 : squelette, règles Silver Age officielles, 9 listes de decks, source officielle des cartes | `claude/squelette-projet`, `claude/listes-decks` |
| 2026-11-22 | Étape 1 : cœur des règles (tour, priorité, combat), puis système d'effets suspendable | `claude/etape-1-regles`, `claude/etape-1-suite` |
| 2026-11-22 | Fin de l'étape 1 : arcane barrier / ward / spellvoid, remplacements, attaques contre alliés et spectra, perte des capacités, durées « prochain tour », étape de fermeture avec déclenchements | `claude/etape-1-moteur` |
