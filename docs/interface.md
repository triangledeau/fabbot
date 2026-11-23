# Jouer contre le bot

Deux interfaces, avec les mêmes règles, la même annulation et le même
principe : vous ne voyez que ce que votre joueur a le droit de voir.

## Dans le navigateur (recommandé)

```
cargo run --release -p fab-ui --bin fab-web
```

Puis ouvrez **http://127.0.0.1:8080** dans votre navigateur (Edge, Chrome,
Firefox). Laissez le terminal ouvert pendant la partie ; `Ctrl+C` dans le
terminal arrête le serveur. Le serveur n'écoute que sur votre ordinateur
(personne d'autre sur le réseau ne peut s'y connecter).

Options (les mêmes que l'interface texte, plus `--port`) :
`--deck hala`, `--bot dorinthea`, `--niveau b1` (**B1 par défaut ici**),
`--graine 42`, `--port 8081` (si le port 8080 est déjà pris). Exemple :

```
cargo run --release -p fab-ui --bin fab-web -- --deck levia --bot arakni --niveau b2
```

Le bouton **Nouvelle partie** permet aussi de changer de decks, de niveau
et de graine sans relancer le serveur.

### La page

- **En haut** : le bot (héros, PV, points d'action, ressources, taille du
  deck, nombre de cartes en main, équipement, arsenal, permanents, pitch).
- **Au milieu** : la chaîne de combat (attaque, cible, puissance,
  défenseurs) et la pile.
- **En bas** : vous, avec votre main.
- **À droite** : la question posée et ses options, la fiche de la carte
  survolée (avec le **texte officiel**), et le journal de la partie.
- « Cimetière · Bannies » : cliquez pour déplier le détail.

Sur une carte : coût en haut à droite, ⚔ puissance, ◆ pitch, ⛨ défense,
la bande de couleur indique sa couleur de pitch. Une carte penchée est
engagée ; un bord pointillé = carte face cachée (que vous seul voyez).

### Jouer

- Les cartes **surlignées en jaune** ont une option : cliquez pour la
  choisir (s'il y en a plusieurs, un petit menu s'ouvre). Vous pouvez aussi
  cliquer une option dans la liste de droite.
- Clavier : `1` à `9` = option, `P` = passer, `A` ou `Ctrl+Z` = annuler.
- **↶ Annuler** revient avant votre dernier choix (et avant les réponses du
  bot qui ont suivi), autant de fois que voulu. Le nombre entre parenthèses
  dit combien de choix on peut annuler. Après une annulation, le bot peut
  répondre autrement (B0 et B2 ont une part de hasard).
- Avec B2, chaque décision du bot prend une fraction de seconde : « Le bot
  réfléchit… » s'affiche en bas à droite.

## Dans le terminal (interface texte)

```
cargo run --release -p fab-ui
```

Par défaut, vous jouez **Hala** et le bot joue **Dorinthea**. Options :

| Option | Effet |
|---|---|
| `--deck dorinthea` | vous jouez Dorinthea (le bot prend alors Hala) |
| `--deck ira` | vous jouez Ira, Crimson Haze (le bot prend alors Hala) |
| `--deck bravo` | vous jouez Bravo, Flattering Showman (le bot prend alors Hala) |
| `--deck iyslander` | vous jouez Iyslander (le bot prend alors Hala) |
| `--deck florian` | vous jouez Florian (le bot prend alors Hala) |
| `--deck levia` | vous jouez Levia (le bot prend alors Hala) |
| `--deck dromai` | vous jouez Dromai (le bot prend alors Hala) |
| `--deck arakni` | vous jouez Arakni, Web of Deceit (le bot prend alors Hala) |
| `--bot hala` | deck du bot : `ira`, `hala`, `dorinthea`, `bravo`, `iyslander`, `florian`, `levia`, `dromai` ou `arakni` (on peut faire un miroir : `--deck hala --bot hala`) |
| `--niveau b1` | niveau du bot : `b0` (au hasard, par défaut), `b1` (heuristique) ou `b2` (recherche ISMCTS ; variantes `b2-500ms`, `b2-1000`…), voir `docs/bots.md` |
| `--graine 42` | rejouer exactement la même partie (mêmes mélanges) |

Exemple : `cargo run --release -p fab-ui -- --deck dorinthea --graine 42`
(les deux tirets seuls séparent les options de cargo de celles du jeu).

Par défaut, le bot est **B0 : il choisit au hasard** parmi les coups
légaux. Il sert à vérifier les règles et les cartes (il essaie tout, même
les coups absurdes). Avec `--niveau b1`, le bot **B1** joue « normalement »
(il attaque à son tour, défend au vôtre) : plus utile pour voir les cartes
dans de vraies situations. Il reste faible (voir `docs/bots.md`).

## Pendant la partie

Le moteur s'arrête à chaque choix que vous devez faire et affiche les options
numérotées. Il passe tout seul quand « passer » est le seul choix possible.

| Commande | Effet |
|---|---|
| `3` | choisir l'option 3 |
| `a` | annuler votre dernier choix (autant de fois que voulu) |
| `e` | afficher tout l'état, cimetières et cartes bannies compris |
| `t 12` | fiche et **texte officiel** de la carte `[12]` |
| `?` | aide |
| `q` | quitter |

Chaque carte affichée porte un numéro entre crochets, par exemple
`Edict of Steel (bleu) [9]` : c'est ce numéro qu'on donne à `t`.

La ligne « Permanents » donne, pour un allié (dragon, Aether Ashwing…), sa
puissance et ses PV actuels, ses compteurs (endurance, vie), les cartes
qui sont dessous (ex. « dessous : Ash ») et phantasm ; pour une aura ou une
Ash, son ward. Pour un dragon, `t` affiche le texte de sa face arrière.

Le texte officiel (avec errata) vient du fichier de Legend Story Studios ;
il faut l'avoir téléchargé une fois avec `bash tools/fetch_cards.sh`.

Vous ne voyez que ce que votre joueur a le droit de voir : la main et
l'arsenal du bot (face cachée) n'apparaissent que par leur nombre, et sa
pioche n'est pas révélée.

## Valider les cartes

C'est l'occasion de vérifier les cartes de Hala et Dorinthea en situation :
jouez-les, regardez le texte officiel avec `t`, puis notez dans
`docs/cards/STATUS.md` celles qui se comportent correctement (statut
`Validated`). Tout comportement faux : le signaler à Claude, il deviendra
d'abord un test qui échoue, puis sera corrigé.

## Limites actuelles

- Équipement et deck de 40 choisis automatiquement (les premières cartes de
  la liste) ; le choix avant chaque manche arrive à l'étape 6.
- « Look at the top card of your deck » (On the Horizon) n'affiche encore rien.
- Page web : pas d'images de cartes (le nom, la couleur et les valeurs
  seulement) ; une seule partie à la fois (un nouvel onglet montre la même).
