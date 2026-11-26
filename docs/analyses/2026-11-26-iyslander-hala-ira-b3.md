# Iyslander, Hala et Ira avec B3 : analyse (2026-11-26)

Agent : Claude, branche `claude/project-thread-j807fx`. Analyse seulement :
aucun changement des bots n'est appliqué (variantes mesurées dans
`2026-11-26-experiences-b1.patch`, à côté de ce fichier).

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

## Correctifs proposés (en attente de l'accord du propriétaire)

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
