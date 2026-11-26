# 2026-11-26 — B1 attaque les dragons de Dromai

- Agent : Claude Code, branche `claude/project-thread-2y4214`.
- Question d'Antoine : Dromai gagne beaucoup ; le bot pense-t-il à détruire
  les dragons ?

## Constat (mesuré, `crates/sim/examples/cibles_dragons.rs`)

Part des attaques dirigées vers un dragon quand c'était possible, contre
Dromai (adversaire Ira sauf mention) :

| Bot | Sur un dragon | Parties |
|---|---|---|
| B0 | 61 % (1 416 / 2 339) | 300 |
| B1 (avant) | **0 %** (0 / ≈ 7 000, Ira + Bravo + Florian) | 900 |
| B2 | 38 % (111 / 293) | 20 |
| B2-200a | 23 % (31 / 132) | 15 |
| B2-1000 | 26 % (25 / 98) | 6 |
| B3 | 45 % (78 / 175) | 15 |

## Correctif

B1 (`cible_d_attaque`) vise un allié adverse si l'attaque peut le
détruire et si 2 × sa puissance dépasse la puissance de l'attaque ; jamais
si l'attaque peut tuer le héros. `Vue::ma_carte_sur_la_pile`,
`Vue::vie_objet`. Tests : `crates/cards/tests/bot_b1_cibles.rs`.

Tournoi B1 contre B1, 1 000 parties par paire : Dromai contre Ira 90 % → 81 %,
les autres matchups de Dromai à ±1 point, moyenne 87 % → 86 %. Le manque
n'explique donc qu'une petite partie de la force de Dromai.

## À refaire / pistes

- `data/side/b1.txt` et les poids de B3 ont été appris avec l'ancien B1 :
  à refaire un jour (fichiers à un seul agent).
- Iyslander (mesuré, non corrigé, à la demande d'Antoine on s'est arrêté) :
  B1 ne fait jamais de pitch stacking (ordre du pitch toujours le premier),
  joue Ice Eternal surtout à X = 0 ou 1, la fusionne presque jamais, jamais
  deux Ice Eternal le même tour. L'ancien Arsenal JS avait une ligne
  « double Ice Eternal » dédiée.
