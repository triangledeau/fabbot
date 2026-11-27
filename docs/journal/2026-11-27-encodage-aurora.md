# 2026-11-27 — Essai d'encodage du deck Aurora

- Agent : Claude Code (session d'orchestration + sous-agents Haiku et Sonnet).
- Branche : `claude/project-thread-0jwqpo` (part de `claude/prototype-langage-cartes` + `main`).
- Deck : `data/decks/en_cours/10_aurora.txt` (Sunday Showdown Sydney, 5e).

## Méthode

1. Fiche compacte par carte : `python3 tools/fiche_carte.py "Nom"` (texte LSS
   par couleur, valeurs imprimées, 3 cartes déjà codées les plus proches).
2. Un sous-agent Haiku par carte : code dans `crates/cards/src/real/lightning.rs`,
   tests dans `crates/cards/tests/cartes_lightning.rs`, `cargo test` jusqu'au vert.
3. Relecture de chaque carte par la session principale (diff du code).
4. Reprise par Sonnet des cartes fausses ou incomplètes.

## Chiffres mesurés

- Étage sans IA (`langage deviner`) : 0 carte sur 24 (vocabulaire Warrior seul).
- 9 cartes génériques du deck étaient déjà codées.
- Haiku : 24 cartes (+ 3 jetons), 2 400 000 tokens, 100 000 par carte en
  moyenne, 1 à 2,5 min par carte. 17 justes à la relecture (71 %),
  4 partielles bien signalées (brique moteur manquante), 3 fausses non vues
  (Rush of Power, Star Fall, Quick Succession).
- Sonnet : les 7 cartes en 5 passages, 509 000 tokens. Toutes complétées.
  Ajouts au moteur : `CardDef::attack_mod` (effet statique d'une arme sur ses
  attaques), `keywords_sans_self_mod`, fusion mémorisée sur l'objet
  (`is_fused`), effet « N prochaines attaques », capacité différée à usage
  unique (`add_delayed_trigger`), capacité activée à coût X (`x_per`).
- Part qui a demandé du Rust dans le moteur : 6 cartes sur 24.
- Tokens de la session principale (relecture, orchestration) : non mesurés.
- Tests : 1 007 verts, clippy et fmt propres. Simulation `--games 20000
  --check --vrais` : 0 erreur ; 2,94 s et 2,97 s contre 2,80 s et 2,87 s sur
  `main` (environ 4 % plus lent, mesuré dans le conteneur).

## Reste à faire

- Relecture des 24 cartes par le propriétaire (statut `Tested`).
- Point de règle ouvert : Beckoning Haunt, un X sans cible légale reste
  payable sans effet (comportement moteur existant pour une cible absente).
- Passer le deck dans `data/decks/` (étape 4 de `docs/nouveau-deck.md`), puis
  refaire la table de side et le tournoi Elo.
