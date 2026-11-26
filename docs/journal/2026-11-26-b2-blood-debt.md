# 2026-11-26 — B2/B3 et le blood debt de Levia (en pause)

Agent : Claude Code. Branche : `claude/vigilant-fermat-iq5cjs` (non fusionnée).

## Fait
- Estimation avec le blood debt à venir : réglages `d<tours>` et `g<part>`
  dans le nom du bot (`b2-300d2`, `b3-300d2g0.5`), `fab_ai::b2::dette_a_venir`.
- Parties simulées qui évitent le blood debt (réglage `l`) :
  `fab_ai::b2::eviter_le_blood_debt` (Cleave the Heavens rouge, Hex Gauntlet).
- `fab-sim --tournoi … --avec levia` : seulement les 17 paires avec Levia.
- Tests dans `crates/cards/tests/bot_b2.rs`. Réglages par défaut inchangés.

## Mesures (contre B2, 17 paires avec Levia, 16 parties par paire)
Score de B2 quand il joue Levia (144 parties, ± 6 points environ) :
B2 actuel 16 % ; `d1` 12 % ; `d2` 15 % ; `d3` 13 % ; `l` 12 %.
**Aucun gain mesurable.**

## Diagnostic
Avec B2, Levia ne perd que ≈ 2 à 3,5 PV par partie au blood debt (elle
évite la perte ≈ 9 fois sur 10 en bannissant une carte de 6) ; B1 seul ne
l'évite que 2 fois sur 3. Elle perd surtout la course aux dégâts, souvent
de peu (adversaire à 1–6 PV). Piste : regarder les dégâts par tour et les
cartes injouables en début de partie (3 cartes au cimetière exigées).
Mesure finale sur les 81 paires : pas encore faite.
