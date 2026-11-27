# 2026-11-27 — Intégration du deck Aurora (étapes 4 et 5 de `docs/nouveau-deck.md`)

Agent : Claude Code. Branche : `claude/project-thread-v8uk72` (reprend
`claude/project-thread-0jwqpo`, qui avait déplacé le deck dans
`data/decks/10_aurora.txt` et ajouté les images).

## Fait
- `verifier-deck` : deck complet, 0 problème ; `check_cards.py` : 34 entrées, 0 problème.
- Parties au hasard vérifiées (`--bots b0 b0 --games 100000 --check`) :
  Aurora contre Ira, Levia et Arakni, **0 erreur** sur 300 000 parties.
- Table de side refaite (`--apprendre-side --games 20000 --side-adverse appris`,
  100 paires, 2 000 000 parties, 252 s).
- Poids de B3 refaits (`--apprendre-b3 --games 112 --bots b3-100 b3-100 --lambda 40`,
  11 200 parties, 69 min, 0 erreur, perte de test 0,4727). Face-à-face contre
  la 3e génération : 48,7 % sur 1 600 parties, −9 Elo (−26 à +8), non
  significatif. Retenus (seuls à connaître Aurora) ; l'ancienne 3e
  génération est dans `data/b3/generation3.txt`.
- Deux tests supposaient 9 decks : la route `/api/decks` (`crates/ui/tests/web.rs`)
  compte maintenant `decks::noms()` ; le test des poids de B3 s'appelle
  `…_connait_tous_les_heros`.

## Mesures
- Tournoi B1 contre B1, choix « liste » des deux côtés, 400 parties par paire :
  Aurora 57 % (+51 Elo), dans le haut du peloton (Dromai 86 % contre elle,
  Bravo 63 %).
- Tournoi B1 côté side appris contre B1 côté liste : 66,6 % au total ; Aurora
  avec le side appris 76 %.

## Pas fait
- Pas de nouveau tournoi Elo complet (`docs/bots.md` garde les chiffres à 9 decks).
- Point de règle ouvert inchangé : Beckoning Haunt, X sans cible légale.
- Les 24 cartes restent `Tested` : validation par le propriétaire en jouant le deck.
