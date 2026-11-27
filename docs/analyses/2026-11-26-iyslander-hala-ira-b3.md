# Iyslander, Hala et Ira avec B3 : analyse (2026-11-26)

Agent : Claude, branche `claude/project-thread-j807fx`. Les variantes de
l'analyse sont dans `2026-11-26-experiences-b1.patch`, à côté de ce fichier.
Les correctifs ont ensuite été appliqués avec l'accord du propriétaire :
voir « Correctifs appliqués » à la fin.

## En bref

Sur 1 620 parties B3 contre B3, Iyslander gagne 21 % (8e sur 9), Hala 42 %
(6e) et Ira 60 % (4e) : seuls Iyslander et Hala sont vraiment en dessous.
Les causes ne sont pas les mêmes.

- **Iyslander** perd surtout parce que ses adversaires prennent leurs
  équipements Arcane Barrier (Nullrune) contre lui : une bleue pitchée paie
  trois Arcane Barrier 1 et annule un sort entier. Sans ces équipements
  adverses, il passe de 29 % à 51 % (B1, 3 600 parties). Sa table de side a
  été apprise contre des adversaires au choix « liste », donc sans Nullrune.
- **Hala** est freinée par un défaut de B1 : il ne voit pas le go again de
  Zenith Blade aiguisée (le go again est donné à l'attaque-proxy, B1 lit les
  mots-clés de l'arme). Corrigé à l'essai : 42,0 % → 45,4 % avec B1 (3 600
  parties), 42,6 % → 52,8 % avec B3 (108 parties).
- **Ira** n'est pas en dessous de la moyenne ; il perd contre Dromai, Bravo
  et Florian.
- Double Ice Eternal, X d'Ice Eternal, fusion : presque rien (+0,6 point,
  non significatif), comme dans l'ancienne Arsenal JS.

## Matrice B3 contre B3

`fab-sim --tournoi --bots b3 b3 --games 20 --check --seed 7` (0 erreur, 36 min,
4 cœurs). Score (%) du deck de la ligne contre celui de la colonne.

| deck | ira | hala | dorinthea | bravo | iyslander | florian | levia | dromai | arakni | moy. |
|---|---|---|---|---|---|---|---|---|---|---|
| dromai | 80 | 100 | 100 | 20 | 100 | 90 | 95 | 45 | 100 | 81 |
| bravo | 55 | 85 | 25 | 60 | 85 | 25 | 100 | 100 | 95 | 70 |
| florian | 65 | 100 | 80 | 45 | 85 | 50 | 95 | 15 | 90 | 69 |
| dorinthea | 55 | 100 | 25 | 65 | 100 | 15 | 95 | 0 | 90 | 61 |
| ira | 58 | 72 | 50 | 32 | 90 | 25 | 90 | 35 | 85 | 60 |
| hala | 25 | 60 | 0 | 10 | 95 | 0 | 100 | 0 | 90 | 42 |
| arakni | 5 | 5 | 5 | 10 | 55 | 15 | 90 | 0 | 55 | 27 |
| iyslander | 15 | 0 | 0 | 5 | 45 | 5 | 70 | 0 | 50 | 21 |
| levia | 5 | 0 | 0 | 5 | 35 | 5 | 65 | 5 | 15 | 15 |

Parties détaillées (108 parties B3 par deck, outil `analyse`) :

| deck | victoires | dégâts par tour à soi | tours à 0 dégât | tours sans carte jouée |
|---|---|---|---|---|
| Iyslander | 31 % | 2,2 (dont 1,3 arcane) | 42 % | 24 % |
| Hala | 43 % | 1,2 | 61 % | 45 % |
| Ira | 57 % | 2,1 | 35 % | 11 % |

## Mesures

| essai | bot | parties | victoires |
|---|---|---|---|
| Iyslander, référence | B1 | 3 600 | 29,2 % |
| Iyslander, adversaires au side « liste » (pas de Nullrune) | B1 | 3 600 | 51,3 % |
| Iyslander, adversaires qui ne paient l'Arcane Barrier qu'avec des ressources déjà là | B1 | 3 600 | 44,5 % |
| Iyslander, référence | B3 | 108 | 31,5 % |
| Iyslander, adversaires au side « liste » | B3 | 108 | 38,0 % |
| Iyslander, B1 : actions bleues de l'arsenal au tour adverse | B1 | 3 600 | 29,2 % |
| Iyslander, B1 : arsenal = action bleue non-attaque | B1 | 3 600 | 28,8 % |
| Iyslander, B1 : Frost Spike chez l'adversaire | B1 | 3 600 | 29,2 % |
| Iyslander, B1 : X d'Ice Eternal qui garde une carte Ice (fusion) | B1 | 3 600 | 29,5 % |
| Iyslander, B1 : tout ensemble | B1 | 3 600 | 29,8 % |
| Iyslander, tout ensemble dans les parties simulées de B3 | B3 | 108 | 27,8 % |
| Iyslander joué par B2 (contre B3) | B2 | 108 | 34,3 % |
| Iyslander, son propre side « liste » | B3 | 108 | 23,1 % |
| Hala, référence | B1 | 3 600 | 42,0 % |
| Hala, B1 voit le go again de Zenith Blade aiguisée | B1 | 3 600 | 45,4 % |
| Hala, référence | B3 | 108 | 42,6 % |
| Hala, même B1 dans les parties simulées de B3 | B3 | 108 | 52,8 % |
| Hala jouée par B2 (contre B3) | B2 | 108 | 37,0 % |
| Ira, référence | B3 | 108 | 56,5 % |
| Ira joué par B2 (contre B3) | B2 | 108 | 46,8 % |

Incertitude : ± 1,6 point sur 3 600 parties, ± 9 points sur 108 ; mêmes
graines d'une variante à l'autre.

Ice Eternal : B1 la joue 0,4 fois par partie, dans 75 % des cas sans autre
carte Ice en main ; B3 la joue 0,04 fois et la pitche 0,75 fois.

## Vérifié et écarté

- Textes des cartes : `check_coded_cards.py`, 396 cartes, 0 écart ; textes
  relus (Iyslander, Ice Eternal, Frost Spike, Frostbite, Ravenous Rabble,
  Stir the Aetherwinds, Hala, Zenith Blade, Sharpen, Flurry, Nullrune).
- Arcane Barrier (CR 8.3.8) et fin des ressources en fin de tour
  (CR 4.4.3e) : conformes.
- Poids de B3 : B2 ne fait pas mieux pour ces decks.
- Choix de règle en attente qui touche ces decks : Aether Ashwing (AB1 qui
  protège le héros de Dromai), non mesuré à part.
- Sans le matchup Dromai : Iyslander 24 %, Hala 48 %, Ira 63 %.

## Correctifs proposés (appliqués le 2026-11-27, voir plus bas)

1. B1 voit le go again de Zenith Blade aiguisée (`FAB_B1_EXP=zenith` du
   patch) ; confirmer par un tournoi B3 complet.
2. B1 estime le go again des combos d'Ira (Seek Vengeance, Pouncing Qi…) :
   le moteur ne l'ajoute que sur la pile ou la chaîne (`ninja.rs`). À mesurer.
3. Refaire `--apprendre-side` contre un adversaire qui utilise la table
   (Iyslander apprendrait à jouer contre AB3/AB4).
4. 4e génération des poids de B3 après 1 et 2.
5. Frost Spike : B1 met le Frostbite chez l'adversaire (aujourd'hui la
   première zone exposée, souvent la sienne).

Déconseillé : double Ice Eternal, pitch stacking (gain nul ici et dans
l'Arsenal JS).

## Reproduire

```bash
cargo run --release -p fab-sim --bin analyse -- iyslander --bots b3 b3 --games 12 --seed 3
SIDE_ADVERSE=liste cargo run --release -p fab-sim --bin analyse -- iyslander --bots b1 b1 --games 400 --seed 11
git apply docs/analyses/2026-11-26-experiences-b1.patch   # variantes, puis FAB_B1_EXP=zenith …
```

## Correctifs appliqués (2026-11-27)

Demandés par le propriétaire, avec trois précisions : AB3 contre Iyslander
est réaliste (tout le monde le prend contre un mage qui fait souvent 3
dégâts d'arcane d'une source) ; Ira domine la méta et devrait être plus
haut ; Iyslander doit privilégier des séquences à gros dégâts que AB3 ne
peut pas prévenir entièrement, construites par pitch stacking, en créant
des gelures pendant le tour adverse. Règles de B1 : `docs/bots.md`.
Tests : `crates/cards/tests/bot_b1_go_again.rs`, `bot_b1_sorts.rs`.

| correctif | mesure | avant | après |
|---|---|---|---|
| Hala : go again de Zenith Blade aiguisée | B1, 3 600 parties | 42,0 % | 45,6 % |
| Ira : go again des combos (Seek Vengeance, Pouncing Qi…) | B1, 3 600 parties | 72,7 % | 72,8 % |
| Ira : Scar for a Scar (go again si moins de vie) | B1, 3 600 parties | 72,8 % | 74,4 % |
| Ira : Pouncing Paws, Tearing Shuko, Blood Scent | B1, 3 600 parties | 74,4 % | 76,7 % |
| Ira : go again (Zenith des adversaires compris) | B3, 108 parties | 56,5 % | 66,7 % |
| Iyslander : sorts qui passent l'arcane barrier, renforts (Crucible, amp, Stir) | B1, 3 600 parties | 29,8 % | 49,2 % |
| Iyslander : + gelures au tour adverse (carte Ice bleue en arsenal) | B1, 3 600 parties | 49,2 % | 50,4 % |
| Iyslander : + pitch stacking | B1, 3 600 parties | 49,2 % | 49,5 % |
| Iyslander : tout ensemble | B1, 3 600 parties (graine 11 / 12) | 29,8 % | 51,4 % / 51,7 % |
| Iyslander : tout ensemble | B3, 108 parties | 23,1 % | 42,6 % |
| Table de side réapprise contre la table (`--side-adverse appris`) | B1, Iyslander | 51,4 % | 51,4 % |
| Poids de B3, 4e génération | B3 contre B3 3e gén., 1 296 parties | — | 46,9 % (−22 Elo) : **non retenue** |

Le pitch stacking n'apporte presque rien (+0,3 à +0,9 point) : en 40
cartes, le deck est rarement épuisé avant la fin de la partie, donc les
cartes remises sous le deck reviennent peu. Les gelures au tour adverse
apportent +1 à +2 points.

### Tournoi B3 contre B3 après les correctifs

Même commande que plus haut (`--games 20 --check --seed 7`, 0 erreur,
36 min). Moyenne de chaque deck, avant → après :

| deck | avant | après |
|---|---|---|
| Ira | 60 | **70** |
| Dromai | 81 | 70 |
| Dorinthea | 61 | 69 |
| Florian | 69 | 68 |
| Bravo | 70 | 63 |
| Hala | 42 | **47** |
| Iyslander | 21 | **41** |
| Arakni | 27 | 20 |
| Levia | 15 | 11 |

Matrice complète :

| deck | ira | hala | dorinthea | bravo | iyslander | florian | levia | dromai | arakni | moy. |
|---|---|---|---|---|---|---|---|---|---|---|
| ira | 60 | 80 | 90 | 40 | 60 | 40 | 100 | 60 | 100 | 70 |
| dromai | 65 | 95 | 100 | 5 | 75 | 50 | 100 | 40 | 100 | 70 |
| dorinthea | 30 | 90 | 60 | 95 | 90 | 60 | 100 | 0 | 95 | 69 |
| florian | 60 | 85 | 60 | 45 | 100 | 55 | 100 | 20 | 90 | 68 |
| bravo | 75 | 50 | 5 | 60 | 55 | 45 | 90 | 95 | 95 | 63 |
| hala | 30 | 60 | 35 | 25 | 65 | 10 | 100 | 0 | 95 | 47 |
| iyslander | 45 | 30 | 35 | 40 | 25 | 5 | 75 | 20 | 90 | 41 |
| arakni | 10 | 15 | 0 | 0 | 25 | 5 | 80 | 0 | 48 | 20 |
| levia | 10 | 0 | 0 | 0 | 25 | 0 | 40 | 0 | 25 | 11 |

Incertitude : ± 7 points sur la moyenne d'un deck (180 parties). Ira est
maintenant à égalité en tête, comme dans la méta réelle selon le
propriétaire. Arakni et Levia perdent un peu : leurs adversaires jouent mieux.
