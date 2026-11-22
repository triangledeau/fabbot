# Jouer contre le bot (interface texte)

```
cargo run --release -p fab-ui
```

Par défaut, vous jouez **Hala** et le bot joue **Dorinthea**. Options :

| Option | Effet |
|---|---|
| `--deck dorinthea` | vous jouez Dorinthea (le bot prend alors Hala) |
| `--bot hala` | deck du bot (on peut faire un miroir : `--deck hala --bot hala`) |
| `--graine 42` | rejouer exactement la même partie (mêmes mélanges) |

Exemple : `cargo run --release -p fab-ui -- --deck dorinthea --graine 42`
(les deux tirets seuls séparent les options de cargo de celles du jeu).

Le bot est pour l'instant **B0 : il choisit au hasard** parmi les coups
légaux. Il sert à vérifier les règles et les cartes, pas à s'entraîner.

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

- Bot aléatoire seulement (les bots B1 et B2 arrivent à l'étape 4).
- Équipement et deck de 40 choisis automatiquement (les premières cartes de
  la liste) ; le choix avant chaque manche arrive à l'étape 6.
- « Look at the top card of your deck » (On the Horizon) n'affiche encore rien.
- Une page web avec plateau arrive à l'étape 5.
