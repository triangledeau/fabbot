# fabbot

Un bot qui joue à **Flesh and Blood** au format **Silver Age** avec 9 decks :
contre lui-même pour simuler beaucoup de parties, ou contre un humain avec
annulation illimitée.

> État : **étape 1 en cours**. Le cœur des règles fonctionne avec des cartes
> d'essai fictives : des parties complètes se jouent entre bots aléatoires.
> Les vraies cartes arrivent à l'étape 2.

## Jouer contre le bot

**Double-cliquez sur `Jouer.bat`** (dans le dossier `fabbot`). C'est tout :
le jeu se met à jour, se prépare et s'ouvre dans votre navigateur. La
fenêtre noire qui s'ouvre doit rester ouverte pendant la partie ; fermez-la
pour arrêter le jeu. La première fois (et après une mise à jour), la
préparation prend quelques minutes.

Mode d'emploi de la page : `docs/interface.md`.

## Installer sur Windows (Surface Pro 9)

1. **Installer Git** : https://git-scm.com/download/win (options par défaut).
2. **Installer Rust** : télécharger et lancer `rustup-init.exe` depuis
   https://rustup.rs. S'il propose d'installer les « Visual Studio Build Tools »,
   accepter : Rust en a besoin sur Windows.
3. Ouvrir un **nouveau** terminal (PowerShell) et vérifier :
   ```
   cargo --version
   ```
4. Récupérer le projet :
   ```
   git clone https://github.com/triangledeau/fabbot
   cd fabbot
   ```

Rust fonctionne aussi bien sur les Surface Pro 9 Intel que sur la version ARM (SQ3).

## Commandes utiles

| Je veux… | Commande |
|---|---|
| Lancer tous les tests | `cargo test --workspace` |
| Lancer le simulateur (mode optimisé) | `cargo run --release -p fab-sim -- --games 100000` |
| Simuler en vérifiant la cohérence à chaque coup | `cargo run --release -p fab-sim -- --games 100000 --check` |
| Jouer contre le bot dans le navigateur | double-cliquer sur `Jouer.bat` |
| Jouer contre le bot dans le terminal | `cargo run --release -p fab-ui` (voir `docs/interface.md`) |

La première compilation prend une ou deux minutes, les suivantes sont rapides.
Toujours utiliser `--release` pour simuler : c'est 10 à 50 fois plus rapide.

## Organisation du projet

```
crates/
  engine/     Moteur de règles (état de partie, tours, combat, couches)
  cards/      Base de cartes : données + effets + statut de validation
  ai/         Les bots (aléatoire → heuristique → ISMCTS → évaluation apprise)
  fab-match/  Matchs en plusieurs manches et phase de side
  sim/        Simulateur de parties en masse
  ui/         Interface pour jouer contre le bot
config/       Paramètres du format Silver Age (vérifiés, TRP 7.4)
data/decks/   Les listes des 9 decks
docs/rules/   Sources officielles des règles et suivi de couverture
tools/        Scripts (ex. télécharger les règles officielles)
docs/cards/   Tableau de validation des cartes
```

## Feuille de route

| Étape | Contenu | Statut |
|---|---|---|
| 0 | Mise en place, squelette | ✅ |
| 1 | Cœur des règles (zones, tour, ressources, combat, couches) | ✅ |
| 2 | Deux premiers decks complets + interface texte | ✅ |
| 3 | Les 7 autres decks + simulateur rapide | ⏳ |
| 4 | Bots B1 (heuristique) et B2 (ISMCTS) + classement Elo | ⏳ |
| 5 | Interface web locale + annulation | ⏳ |
| 6 | Matchs en plusieurs manches + phase de side | ⏳ |
| 7 | Bot B3 (évaluation apprise) + procédure « nouveau deck » | ⏳ |
