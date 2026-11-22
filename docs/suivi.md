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
| 3 | Les 7 autres decks + simulateur rapide | ⏳ (Ira ✅, Bravo ✅ ; restent Iyslander, Florian, Levia, Dromai, Arakni) |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs + choix équipement/deck (« phase de side ») | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |

**Branche la plus récente :** `claude/wizardly-pasteur-r91h2w` (deck Bravo,
partie de `main`). Tant qu'elle n'est pas fusionnée dans `main`, partir de
cette branche.

**Jouer contre le bot :** `cargo run --release -p fab-ui` (mode d'emploi :
`docs/interface.md`).

**Chiffres actuels :** 317 tests au vert. Simulateur, bots **aléatoires**
(ces pourcentages ne disent rien des vrais matchups), 100 000 parties
vérifiées (`--check`) sans incohérence pour chaque paire, ≈ 8 000 à 9 500
parties/s dans le conteneur de Claude (4 cœurs) :
`--decks bravo ira` (Bravo 16,0 %), `--decks bravo hala` (Bravo 59,5 %),
`--decks bravo dorinthea` (Bravo 61,8 %), `--decks bravo bravo`,
`--vrais` (Hala 49,8 %, Dorinthea 46,5 %, inchangé). Ira contre Dorinthea :
83,5 % (session précédente).

**Choix provisoire d'équipement et de deck** (`real::default_loadout`, en
attendant la phase de side de l'étape 6) : la première carte de chaque
emplacement d'arène dans l'ordre de la liste (Hala : Helm of Unity, Edge Laden
Plate, Gauntlets of Unity, Pillar of Unity ; Dorinthea : Helm of Unity,
Plating of Unity, Gauntlets of Unity, Refraction Bolters), et les 40 premières
cartes de la liste.

## Questions ouvertes pour le propriétaire

Aucune question bloquante. Deux choix faits pour Bravo, à confirmer en jouant :

1. **Heave** (Thunder Quake) : CR 5.4.7b dit que le joueur décide de
   déclencher la capacité cachée, puis la couche se résout. Le moteur pose une
   seule question (« payer 3 ? ») à la résolution, et ne déclenche heave que
   si c'est possible (arsenal vide, 3 ressources sans pitcher la carte
   elle-même). Si vous refusez, l'adversaire a vu passer la carte.
2. **Bravo** : sa capacité n'est proposée que s'il y a une carte face cachée
   dans l'arsenal (l'activer sans carte serait légal, mais ne ferait rien
   d'autre que coûter 2 ressources).

Questions réglées le 2026-11-22 (réponses du propriétaire, citées en
commentaire dans le code) : action non-attaque à l'étape de résolution
(interdite, CR 7.0.1a), détruit puis banni (reste « détruit », CR 1.9.3a),
ward sans dégâts restants (CR 1.9.1b), Steelblade Shunt, Decimator Great Axe,
Flurry + Dorinthea, On the Horizon, cartes bannies face cachée (visibles par
leur propriétaire, CR 3.0.3a), combo « last attack », Crouching Tiger (une
carte, pas un token), Blessing of Qi (CR 6.2.2a), Aspect of Tiger: Body.
Liste officielle Silver Age (bannies, héros mis sur la touche) copiée dans
`config/silver_age.toml` : aucune carte de nos decks n'est concernée.
Correction : **Shove Off** ne peut plus viser une carte d'un maillon
précédent si l'épée a quitté ce maillon (notes de Mastery Pack Warrior).

## Prochaine tâche

**Pour le propriétaire :** jouer quelques parties avec
`cargo run --release -p fab-ui`, vérifier les cartes de Hala et Dorinthea
(commande `t <n>` pour le texte officiel) et passer celles qui sont justes en
`Validated` dans `docs/cards/STATUS.md` ; signaler tout comportement faux.

Vérifier aussi les cartes d'Ira (`cargo run --release -p fab-ui -- --deck ira`)
et de Bravo (`cargo run --release -p fab-ui -- --deck bravo`) ; `t <n>` donne
le texte officiel.

**Pour Claude (étape 3, suite) :** coder le deck suivant de la liste,
**Iyslander** (`data/decks/05_iyslander.txt`). Puis Florian, Levia, Dromai,
Arakni. Le moteur sait maintenant accorder des
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
| 2026-11-22 | Réponses aux questions 1 et 3 à 14 : commentaires des règles confirmées, correction de Shove Off (épée partie du maillon), cartes bannies face cachée visibles par leur propriétaire dans l'interface, test Legacy of Ikaru sur une 2e attaque d'Edge of Autumn | `claude/intelligent-mayer-kvrq5b` |
| 2026-11-22 | Liste officielle Silver Age (40 cartes bannies, 4 héros mis sur la touche) dans `config/silver_age.toml`, noms français traduits par le fichier officiel ; aucune carte de nos decks concernée | `claude/intelligent-mayer-kvrq5b` |
| 2026-11-22 | Étape 3, deck Bravo : 35 lignes de STATUS.md + tokens Seismic Surge, Might, Vigor, Quicken codés et testés (61 tests de cartes, 7 tests de règles) ; moteur : clash (CR 8.5.45), « win a clash revealing this » depuis le deck, crush (CR 8.4.2, sur les dégâts), heave (capacité cachée depuis la main), suspense (CR 8.3.42), réductions « your next … costs {r} less », « can't gain {p} » (Chokeslam), « loses and can't gain dominate » ; `fab-ui --deck bravo` | `claude/wizardly-pasteur-r91h2w` |
