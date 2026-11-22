# Suivi du projet

Ce fichier est **le point de départ de chaque nouvelle conversation**. Il dit
où en est le projet, ce qui reste en suspens et quoi faire ensuite. Claude le
met à jour à la fin de chaque tâche.

## Où en est-on

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles + système d'effets | ✅ (manques connus listés dans `docs/rules/README.md`) |
| 2 | Deux premiers decks complets + interface texte | ✅ (cartes à valider par le propriétaire avec l'interface) |
| 3 | Les 7 autres decks + simulateur rapide | ⏳ (Ira ✅ ; restent Bravo, Iyslander, Florian, Levia, Dromai, Arakni) |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |

**Branche la plus récente :** `claude/intelligent-mayer-kvrq5b` (deck Ira,
partie de `main`). Tant qu'elle n'est pas fusionnée dans `main`, partir de
cette branche.

**Jouer contre le bot :** `cargo run --release -p fab-ui` (mode d'emploi :
`docs/interface.md`).

**Chiffres actuels :** 245 tests au vert. Simulateur, bots **aléatoires**
(ces pourcentages ne disent rien des vrais matchups), 100 000 parties
vérifiées (`--check`) sans incohérence pour chaque paire, ≈ 7 500 à 8 700
parties/s dans le conteneur de Claude (4 cœurs) :
`--decks ira dorinthea` (Ira 83,5 %), `--decks ira ira`, `--vrais` (Hala
49,8 %, Dorinthea 46,5 %, inchangé). Decks d'essai : 50 000 parties
(normal et `--mecanismes`) sans incohérence.

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
10. **Cartes bannies face cachée** : leur propriétaire peut-il les regarder ?
    L'interface les cache pour tout le monde en attendant
    (`crates/ui/src/vue.rs`, `visible`). Aucune carte de Hala ou Dorinthea
    n'est concernée.
11. **Combo, « the last attack this combat chain »** : le moteur prend
    l'attaque du **maillon précédent** celui de la carte (sinon Seek Vengeance
    ne pourrait jamais avoir son combo, puisqu'elle est elle-même la dernière
    attaque). Même lecture pour la capacité donnée par **Legacy of Ikaru**
    (« When this hits, if Edge of Autumn was the last attack… ») : Legacy sur
    Seek Vengeance après Edge of Autumn → pioche ; Legacy sur l'attaque
    d'Edge of Autumn elle-même → pas de pioche. Correct ? Voir
    `crates/cards/src/real/ninja.rs` (`last_attack_before`).
12. **Crouching Tiger n'est pas un token** (ligne de type officielle « Ninja
    Action - Attack », mot-clé Ephemeral) : le moteur le crée comme une carte
    dans la zone de bannissement ; après son attaque il est retiré de la
    partie (ephemeral). Correct ?
13. **Blessing of Qi** : « It gets +3{p} » n'a pas de durée écrite ; le moteur
    applique le bonus jusqu'à la fin du tour (le tigre ne peut être joué que
    ce tour). Correct ?
14. **Aspect of Tiger: Body** (« if a red attack action card was the last
    attack ») : une attaque d'arme (Edge of Autumn) ne compte pas, une carte
    d'attaque rouge (ex. Scar for a Scar rouge) compte, le Crouching Tiger
    (sans couleur) ne compte pas. Correct ?

## Prochaine tâche

**Pour le propriétaire :** jouer quelques parties avec
`cargo run --release -p fab-ui`, vérifier les cartes de Hala et Dorinthea
(commande `t <n>` pour le texte officiel) et passer celles qui sont justes en
`Validated` dans `docs/cards/STATUS.md` ; signaler tout comportement faux.

Vérifier aussi les cartes d'Ira (`t <n>` dans l'interface donne le texte
officiel) et répondre aux questions 11 à 14.

**Pour Claude (étape 3, suite) :** coder le deck suivant de la liste,
**Bravo, Flattering Showman** (`data/decks/04_bravo.txt`). Puis Iyslander,
Florian, Levia, Dromai, Arakni. Le moteur sait maintenant accorder des
capacités **déclenchées** (`CardDef::granted`) ; pour Dromai il faudra aussi
accorder une capacité **activée** (Storm of Sandikai donne une capacité
d'attaque aux dragons). Vérifier « clash » et « crush » au fil des cartes.

## Journal des sessions

| Date | Travail | Branche |
|---|---|---|
| 2026-11-22 | Étape 0 : squelette, règles Silver Age officielles, 9 listes de decks, source officielle des cartes | `claude/squelette-projet`, `claude/listes-decks` |
| 2026-11-22 | Étape 1 : cœur des règles (tour, priorité, combat), puis système d'effets suspendable | `claude/etape-1-regles`, `claude/etape-1-suite` |
| 2026-11-22 | Fin de l'étape 1 : arcane barrier / ward / spellvoid, remplacements, attaques contre alliés et spectra, perte des capacités, durées « prochain tour », étape de fermeture avec déclenchements | `claude/etape-1-moteur` |
| 2026-11-22 | Étape 2 : toutes les cartes de Hala et Dorinthea (55 lignes de STATUS.md + token Flurry) codées et testées (73 tests de cartes) ; moteur : sharpen, effets « prochaine attaque de cette arme », coût d'activation réduit, attaques supplémentaires (CR 5.2.3), paiement « unless you pay », renvoi d'une carte qui défend, fin de la marque (CR 9.3.3) ; `tools/check_coded_cards.py` ; `fab-sim --vrais` | `claude/quirky-ritchie-a0y1et` |
| 2026-11-22 | Fin de l'étape 2 : interface texte `fab-ui` (état visible du joueur, options numérotées, annulation illimitée, texte officiel des cartes avec `t`, journal des événements) ; moteur : `GameState::record` (journal complet facultatif), libellé des questions des cartes (`Ask::label`) ; `docs/interface.md` | `claude/fervent-ride-8osack` |
| 2026-11-22 | Étape 3, deck Ira : 27 lignes de STATUS.md + Crouching Tiger codées et testées (45 tests Ninja, 12 tests génériques, 8 tests de règles) ; moteur : jouer depuis la zone de bannissement (permission), cartes créées dans le bannissement, ephemeral, ambush, capacités déclenchées accordées (`CardDef::granted`), effets « this combat chain » sur la prochaine attaque, modes proposés seulement s'ils ont une cible ; `fab-sim --decks a b` | `claude/intelligent-mayer-kvrq5b` |
