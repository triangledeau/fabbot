# fabbot — consignes pour Claude Code

Bot jouant 9 decks de Flesh and Blood au format **Silver Age**. Le propriétaire
du projet est un expert du jeu, mais débute en programmation : explications en
**français**, claires, sans jargon inutile.

## Priorités (dans cet ordre)
1. **Exactitude des règles et des cartes.** Ne jamais deviner une règle ou un
   texte de carte. En cas de doute : laisser un `TODO(règles)` et poser la question.
2. **Performance** du moteur (simulation massive sur une Surface Pro 9 **Intel** :
   10 cœurs / 12 threads, dont 2 cœurs performants et 8 cœurs économes).
3. **Niveau de jeu** du bot, toujours *mesuré* (Elo, matrice de matchups), jamais supposé.

## Architecture (workspace Rust dans `crates/`)
- `engine` (`fab-engine`) : état de partie, règles, points de décision, RNG déterministe, historique d'annulation.
- `cards` (`fab-cards`) : données des cartes + effets + statut de validation.
- `ai` (`fab-ai`) : trait `Agent` et les bots B0 → B3.
- `fab-match` : matchs en plusieurs manches, phase de side.
- `sim` (`fab-sim`) : simulateur en ligne de commande.
- `ui` (`fab-ui`) : interface pour jouer contre le bot.

Principe central : le moteur avance jusqu'à une `Decision` ; un `Agent` (bot ou
humain) répond par l'index d'une option. Tout choix de joueur passe par là.

## Début et fin de conversation
- **Au début** : lire `docs/suivi.md` (état d'avancement, questions ouvertes,
  prochaine tâche) et partir de la branche indiquée.
- **À la fin de chaque tâche** : mettre à jour `docs/suivi.md` (statut, journal,
  questions ouvertes, prochaine tâche), committer et pousser.
- **Rappeler au propriétaire de changer de conversation** quand une tâche est
  finie ou que la conversation devient longue (cela réduit la consommation),
  et lui fournir à chaque fois **le message à coller** dans la nouvelle
  conversation, au format :
  > Projet fabbot. Lis `CLAUDE.md` puis `docs/suivi.md`. Pars de la branche
  > `<branche>`. Tâche : <prochaine tâche>.
- Le propriétaire débute avec Git : quand une pull request est utile,
  lui expliquer pas à pas comment la faire sur github.com.

## Règles de travail
- **Sources de vérité** : Comprehensive Rules officiel + textes officiels des
  cartes avec errata (voir `docs/rules/README.md`). Citer la section du CR en
  commentaire, au format `// CR x.y.z`, dans le code de règles.
- Pour consulter les règles : `bash tools/fetch_rules.sh`, puis chercher dans
  `docs/rules/sources/*.txt`. rules.fabtcg.com est accessible ; fabtcg.com
  bloque les accès automatisés (403) : ne pas chercher à contourner, demander
  au propriétaire de copier le contenu.
- Textes des cartes : **fichier officiel LSS** (`bash tools/fetch_cards.sh`,
  colonne `face_*_true_textbox` = texte à jour avec errata). La base
  communautaire ne sert que de complément (légalité par format). Voir
  `docs/cards/README.md`. Vérification : `python3 tools/check_cards.py`, et
  pour les cartes codées `python3 tools/check_coded_cards.py`.
- Vraies cartes : `crates/cards/src/real/` (texte officiel en commentaire
  au-dessus de chaque carte).
- Les 9 decks sont dans `data/decks/` ; `cargo run -p fab-cards --bin deck-stats`.
- En Silver Age il n'y a pas de sideboard séparé : avant chaque manche, on choisit
  équipement + deck de 40 dans un card-pool de 55 (CR 4.1).
- **Chaque carte** suit le cycle : `NotCoded` → `Coded` → `Tested` → `Validated`.
  Seul le propriétaire passe une carte en `Validated`. Tenir `docs/cards/STATUS.md` à jour.
- **Un test par phrase du texte d'une carte**, plus les cas limites. Noms de
  tests en français, descriptifs (`attaque_avec_go_again_rend_un_point_d_action`).
- **Tout bug de règle signalé devient d'abord un test** qui échoue, puis on corrige.
- Pas de valeur de format non vérifiée : `config/silver_age.toml` reste
  commenté tant que `verified = false`.
- Le hasard passe **uniquement** par `fab_engine::Rng` (déterminisme, rejeu).
- `GameState` doit rester bon marché à copier : pas de `String`, des
  identifiants numériques. Seule exception : la base de cartes immuable,
  partagée par `Arc<CardDb>`.
- Le moteur passe la priorité automatiquement quand « passer » est la seule
  option légale : un test qui attend une décision doit en tenir compte.
- Tests de règles : `crates/cards/tests/regles_*.rs`, un test par article du
  CR, nommé `cr_x_y_z_description`. Cartes fictives : `fab_cards::test_set`.
- Test massif : `cargo run --release -p fab-sim -- --games 100000 --check`
  (`--vrais` : vrais decks Hala contre Dorinthea ; `--decks ira hala` : deux vrais decks au choix ;
  `--bots b1 b0` : bots des deux joueurs).
- Classement Elo des bots : `cargo run --release -p fab-sim -- --tournoi --bots b1 b0 --games 1000`
  (81 paires de decks ; voir `docs/bots.md`). Les bots lisent la partie via `fab_ai::vue::Vue`.
- Organisation du moteur et façon d'écrire une carte : `docs/moteur.md`.
- Harnais de test commun : `crates/cards/tests/commun/mod.rs` (`partie`, `choisir`,
  `passer_jusqua`, `finir_le_tour`).
- Un bot n'a accès qu'à l'information de son joueur.
- Interface texte (`fab-ui`) : `docs/interface.md`. Tout `Op::Ask` d'une carte
  porte un `label` (la question affichée au joueur).

## Commandes
- Compiler : `cargo build --workspace`
- Tests : `cargo test --workspace`
- Lint : `cargo clippy --workspace --all-targets -- -D warnings`
- Format : `cargo fmt --all`
- Simulation (mode optimisé) : `cargo run --release -p fab-sim`

Avant chaque commit : fmt, clippy sans avertissement, tous les tests au vert.

## Git
- Ne pas travailler directement sur `main` : une branche par tâche, puis PR.
- Messages de commit en français.
