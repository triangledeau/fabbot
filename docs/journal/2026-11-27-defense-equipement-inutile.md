# 2026-11-27 — Le bot ne défend plus avec un équipement inutile

- Agent : Claude (Claude Code), branche `claude/project-thread-j7cvuu`.
- Signalé par le propriétaire : contre une Chute d'étoile (3) d'Aurora, le
  bot a défendu avec une carte + Nullrune Gloves + Nullrune Hood (défense 0).

## Cause
- B2, B3 et le niveau réfléchi (recherche ISMCTS) traitent « défendre avec
  la capuche Nullrune » comme une option à part. Elle donne exactement le
  même résultat que « passer » : les deux ont le même score aux bruits des
  parties simulées près, et la plus visitée gagne à pile ou face.
- B1 posait aussi ces équipements face à une attaque mortelle (score
  « 100 + dégâts évités », même avec 0 dégât évité).

## Correction
- `fab_ai::equipement_sans_defense` : équipement de défense 0 sans
  déclencheur, ni modification de sa propre défense, ni remplacement, ni
  capacité activée (prudent : un équipement avec une capacité activée reste
  envisagé). Vrais équipements concernés : les 4 Nullrune, Arcane Lantern,
  Aetherstorm Wellingtons.
- La recherche ne crée plus ces options (`options_par_cle`) ; B1 leur donne
  un score négatif. Défendre avec reste légal dans le moteur (pour l'humain).
- Tests : `crates/cards/tests/bot_defense_inutile.rs` (4 tests ; celui de
  B3 et celui de B1 échouaient avant la correction).

## Mesures
- 30 parties B3 contre B3 entre vrais decks (mêmes graines) : avant,
  64 équipements inutiles posés en défense sur 1 291 défenseurs (environ
  2 par partie) ; après, 0. Temps de calcul inchangé (113 s contre 115 s).
- Tournoi court B3 contre B2 (81 paires × 2) : 50,7 % après, 50,0 % avant.
  Trop court pour conclure ; aucun gain de niveau n'est attendu (les
  options retirées valaient « passer »), seulement des itérations mieux
  employées et un jeu plus propre.
