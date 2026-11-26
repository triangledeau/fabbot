# fabbot — consignes communes à tous les agents

Ce fichier vaut pour **tous** les agents qui travaillent sur fabbot :
sessions Claude Code (qui le lisent via `CLAUDE.md`), Antigravity (qui lit
`AGENTS.md`) et tout autre outil. Les consignes propres à un outil sont dans
`CLAUDE.md` (Claude Code) ou `GEMINI.md` (Antigravity).

fabbot est **la version principale du bot** : un bot jouant 9 decks de Flesh
and Blood au format **Silver Age**. Le propriétaire du projet est un expert
du jeu, mais débute en programmation : explications en **français**,
claires, sans jargon inutile.

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
- `ui` (`fab-ui`) : interfaces pour jouer contre le bot (texte, et page web locale `fab-web`).

Principe central : le moteur avance jusqu'à une `Decision` ; un `Agent` (bot ou
humain) répond par l'index d'une option. Tout choix de joueur passe par là.

## Travailler à plusieurs

Plusieurs agents (plusieurs sessions Claude, une session Antigravity…)
peuvent travailler **en même temps**. Pour ne pas se marcher dessus :

### Au début d'une tâche
1. Lire `docs/suivi.md` (état, questions ouvertes, prochaines tâches) et
   **`docs/taches.md`** (qui fait quoi en ce moment).
2. Partir de `main` à jour (`git fetch origin main`), sur une **nouvelle
   branche** nommée d'après l'agent : `claude/<sujet>` ou
   `antigravity/<sujet>`. Jamais de travail direct sur `main`.
3. Vérifier que la tâche demandée ne recoupe pas une tâche « en cours » de
   `docs/taches.md` (mêmes fichiers, même crate). Si oui : le dire au
   propriétaire avant de commencer.
4. Ajouter sa ligne dans « En cours » de `docs/taches.md` et la faire
   arriver sur `main` **tout de suite** (petite pull request à part, ou
   le propriétaire le fait), pour que les autres agents la voient.

### Pendant la tâche
- Rester dans la zone annoncée (crates, fichiers). Si on doit toucher une
  zone d'une autre tâche en cours, le signaler au propriétaire.
- Committer souvent, messages de commit en français, préfixés par la zone
  (`Moteur :`, `Cartes Levia :`, `B3 :`, `Docs :`…).

### À la fin de la tâche
1. Écrire son entrée de journal dans **un fichier à soi** :
   `docs/journal/AAAA-MM-JJ-<sujet>.md` (date, agent, branche, travail fait,
   chiffres mesurés). Ne pas écrire de journal dans `docs/suivi.md`.
2. Dans `docs/suivi.md`, ne modifier **que les lignes concernées** (statut
   de l'étape, chiffres, questions ouvertes, prochaines tâches) : ne pas
   reformuler le reste, cela crée des conflits avec les autres agents.
3. Retirer sa ligne de « En cours » dans `docs/taches.md`.
4. **Ramener `main` dans sa branche** (`git fetch origin main` puis
   `git merge origin/main`), relancer fmt, clippy et **tous** les tests.
5. Pousser la branche, puis pull request vers `main` et fusion quand tout
   est au vert (la vérification automatique GitHub « CI » doit aussi être
   verte). Si la fusion échoue (conflit) : voir ci-dessous.

### Conflits
- `docs/suivi.md`, `docs/taches.md`, `docs/cards/STATUS.md` : garder les
  modifications **des deux côtés** (ce sont des listes).
- Fichiers produits par un programme (`Cargo.lock`, `data/side/b1.txt`…) :
  ne pas fusionner à la main ; reprendre la version de `main` puis
  relancer le programme qui le produit.
- Conflit dans du code écrit par un autre agent : **ne pas forcer**,
  expliquer le conflit au propriétaire et lui demander quoi faire.
- Jamais de `git push --force` sur `main` ni sur la branche d'un autre agent.

### Fichiers « à un seul agent à la fois »
Ces fichiers sont refaits d'un bloc ; une seule tâche à la fois doit les
changer (l'annoncer dans `docs/taches.md`) :
- `data/side/b1.txt` (`fab-sim --apprendre-side`) ;
- les chiffres d'Elo de `docs/bots.md` (un tournoi refait = tous les chiffres changent) ;
- les gros changements du cœur du moteur (`crates/engine/src/`) qui
  touchent beaucoup de fichiers.

## Message à coller pour lancer un agent

Le propriétaire lance chaque agent avec ce message (à adapter) :

> Projet fabbot. Lis `AGENTS.md` (ou `CLAUDE.md`), puis `docs/suivi.md` et
> `docs/taches.md`. Pars de la branche `main`. Tâche : <tâche>.

À la fin d'une tâche, l'agent fournit ce message rempli pour la tâche
suivante, et rappelle au propriétaire de **changer de conversation**
(cela réduit la consommation).

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
- Phase de side (équipement + 40 cartes avant chaque partie) : `fab_match::preparer_manche`,
  trait `Side` ; table apprise `data/side/b1.txt` (`fab-sim --apprendre-side`, à refaire
  quand un deck ou B1 change) ; `--side appris liste` pour comparer.
- Organisation du moteur et façon d'écrire une carte : `docs/moteur.md`.
- Harnais de test commun : `crates/cards/tests/commun/mod.rs` (`partie`, `choisir`,
  `passer_jusqua`, `finir_le_tour`).
- Un bot n'a accès qu'à l'information de son joueur.
- Interfaces (`fab-ui` texte, `fab-web` navigateur) : `docs/interface.md`. Tout `Op::Ask` d'une carte
  porte un `label` (la question affichée au joueur).

## Commandes
- Compiler : `cargo build --workspace`
- Tests : `cargo test --workspace`
- Lint : `cargo clippy --workspace --all-targets -- -D warnings`
- Format : `cargo fmt --all`
- Simulation (mode optimisé) : `cargo run --release -p fab-sim`

Avant chaque commit : fmt, clippy sans avertissement, tous les tests au vert.
