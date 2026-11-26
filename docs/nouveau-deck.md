# Procédure « nouveau deck »

Comment ajouter un 10e deck (ou un 11e…) au projet, de la liste jusqu'aux
bots qui savent le jouer. Chaque étape dit **qui** la fait et **quelle
commande** vérifie qu'elle est finie. Une étape peut s'étaler sur plusieurs
conversations : `docs/suivi.md` dit où on en est.

Exemple utilisé ci-dessous : un deck Kayo, fichier `10_kayo.txt`.

## 1. La liste (propriétaire, puis Claude)

Le propriétaire fournit la liste (copie d'un site de decks, ou texte libre).
Claude l'écrit au format du projet (`data/decks/README.md`) dans
**`data/decks/en_cours/10_kayo.txt`** : le numéro suivant, un tiret bas, le
nom court du héros en minuscules. Tant qu'il est dans `en_cours/`, le deck
n'est vu par **aucun** programme ni test : `main` reste au vert pendant le
codage.

Vérifications :

```bash
bash tools/fetch_cards.sh
```

```bash
python3 tools/check_cards.py data/decks/en_cours/10_kayo.txt
```

```bash
cargo run -p fab-cards --bin verifier-deck -- data/decks/en_cours/10_kayo.txt
```

- `check_cards.py` : chaque carte existe dans le **fichier officiel LSS**,
  avec la bonne couleur (orthographe exacte).
- `verifier-deck` : format Silver Age (55 cartes, au moins 40 cartes de deck,
  2 copies au plus, cartes bannies, héros au banc, `config/silver_age.toml`),
  puis la liste des **cartes à coder** (celles qui ne le sont pas déjà pour un
  autre deck) et des lignes manquantes dans `docs/cards/STATUS.md`.
- Légalité Silver Age des raretés : à vérifier dans la base communautaire
  (`silver_age_legal`, voir `docs/cards/README.md`) ; en cas de doute, poser
  la question au propriétaire.

## 2. Le suivi des cartes (Claude)

Ajouter dans `docs/cards/STATUS.md` une ligne par carte nouvelle (héros,
arène, chaque couleur des cartes de deck), au statut `NotCoded` ;
`cargo run -p fab-cards --bin deck-stats -- --status` donne le tableau
complet si le deck est déjà dans `data/decks/` (sinon, écrire les lignes à
la main en suivant `verifier-deck`). Les cartes partagées avec un autre deck
ne changent pas de statut : ajouter seulement le héros dans la colonne
« Utilisée par ».

## 3. Les cartes (Claude, une ou plusieurs conversations)

Pour **chaque** carte à coder (méthode : `docs/moteur.md`, « Écrire une
carte ») :

1. lire le texte à jour (colonne `face_*_true_textbox` du fichier officiel)
   et le recopier en commentaire au-dessus du code ;
2. coder la carte dans `crates/cards/src/real/<classe>.rs` (un nouveau
   fichier pour une nouvelle classe, ajouté à `real::build`) ; un mécanisme
   nouveau va dans le moteur, avec ses tests de règles
   (`crates/cards/tests/regles_*.rs`, `cr_x_y_z_description`) ;
3. **un test par phrase** du texte, plus les cas limites, dans
   `crates/cards/tests/cartes_<classe>.rs`, noms de tests en français ;
4. toute question de règle : un `TODO(règles)` et une question au
   propriétaire dans `docs/suivi.md`, jamais une supposition ;
5. passer la ligne de `STATUS.md` à `Tested`.

Vérifications à chaque fois :

```bash
python3 tools/check_coded_cards.py
```

```bash
cargo run -p fab-cards --bin verifier-deck -- data/decks/en_cours/10_kayo.txt
```

`check_coded_cards.py` compare les valeurs imprimées des cartes codées
(coût, pitch, puissance, défense, types, mots-clés) au fichier officiel.

## 4. Le deck entre dans le projet (Claude)

Quand `verifier-deck` répond « deck complet » :

```bash
git mv data/decks/en_cours/10_kayo.txt data/decks/10_kayo.txt
```

Le deck est alors lu **partout** par `fab_cards::decks` (la seule liste des
decks) : simulateur, tournois, interface texte (`--deck kayo`), page web
(menu des decks), tests. Mettre à jour le tableau de `data/decks/README.md`.

Vérifications :

```bash
cargo test --workspace
```

```bash
cargo run --release -p fab-sim -- --decks kayo ira --bots b0 b0 --games 100000 --check
```

```bash
cargo run --release -p fab-sim -- --tournoi --bots b1 b0 --games 200 --check
```

- les tests `decks_du_projet.rs` (format, cartes toutes codées), `bots.rs`
  (B1 joue des parties complètes avec chaque deck, sans lire d'information
  cachée) et `regles_side.rs` (card-pool de 55, choix légaux) couvrent le
  nouveau deck sans rien changer ;
- 100 000 parties au hasard contre un autre deck, vérifiées après chaque
  décision (`--check`) : **0 erreur** attendu ; recommencer contre 2 ou 3
  autres decks (surtout ceux qui interagissent : auras, alliés, marques…) ;
- un tournoi B1 contre B0 rapide : le nouveau deck ne doit pas être très
  en dessous des autres (sinon, B1 le joue mal : voir `docs/bots.md`).

## 5. Les bots apprennent le deck (Claude)

Deux tables apprises dépendent des decks. Sans elles, le nouveau deck
fonctionne déjà, mais moins bien : la phase de side prend le choix
« liste », et l'évaluation de B3 n'a pas de poids propres au héros ni à ses
permanents (ils valent 0).

```bash
cargo run --release -p fab-sim -- --apprendre-side --games 20000
```

```bash
cargo run --release -p fab-sim -- --apprendre-b3 --games 500
```

Puis recompiler (les tables sont compilées dans le programme) et mesurer :

```bash
cargo run --release -p fab-sim -- --tournoi --bots b1 b1 --side appris liste --games 2000 --check
```

```bash
cargo run --release -p fab-sim -- --tournoi --bots b3 b2 --games 10 --check
```

Les deux apprentissages refont **toutes** les paires (le nouveau deck change
aussi les choix des autres decks contre lui). Durées (conteneur de Claude,
4 cœurs) : ≈ 4 min pour le side avec 10 decks, ≈ 15 s pour B3 ; les
tournois de B3 sont plus longs (≈ 1 min par partie et par cœur… voir
`docs/bots.md`).

## 6. Le propriétaire vérifie

- jouer le deck dans la page web (`Jouer.bat`, menu des decks) ou dans le
  terminal (`cargo run --release -p fab-ui -- --deck kayo`, `t <n>` pour le
  texte officiel) ;
- passer en `Validated` dans `docs/cards/STATUS.md` les cartes qui se
  comportent bien, signaler les autres (chaque bug signalé devient d'abord
  un test qui échoue) ;
- relire les choix appris du deck dans `data/side/b1.txt`.

## Résumé

| Étape | Fait quand | Commande |
|---|---|---|
| 1. Liste dans `data/decks/en_cours/` | cartes officielles, format Silver Age | `check_cards.py`, `verifier-deck` |
| 2. Lignes de `STATUS.md` | plus de « ligne absente » | `verifier-deck` |
| 3. Cartes codées et testées | plus de « carte à coder » | `verifier-deck`, `check_coded_cards.py`, `cargo test` |
| 4. Fichier dans `data/decks/` | 0 erreur | `cargo test`, `fab-sim --check` |
| 5. Tables apprises | refaites et mesurées | `--apprendre-side`, `--apprendre-b3`, `--tournoi` |
| 6. Validation | cartes `Validated` | propriétaire |
