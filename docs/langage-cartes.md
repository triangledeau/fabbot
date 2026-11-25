# Langage de cartes (prototype du 2026-11-25)

But : encoder les ~2 200 noms du pool Silver Age vite, sans perdre en exactitude.
Une carte s'écrit en quelques lignes de données au lieu de code Rust, et un
outil la **réécrit en anglais** pour la comparer au texte officiel LSS.

## Où

- Cartes : `crates/cards/cartes/*.cartes` (pour l'instant `warrior.cartes`, les
  25 cartes des decks Hala et Dorinthea).
- Lecture, construction pour le moteur, réécriture : `crates/cards/src/langage/`.
- Morceaux impossibles à dire dans le langage : `native NOM "texte"` dans le
  fichier, code Rust dans le module de la classe (`real/warrior.rs`).
- Moteur : deux instructions en plus, `Op::Run(f, arg)` et `Op::IfArg(f, arg, n)`
  (comme `Do` et `If`, avec un argument qui désigne une phrase du fichier).

## Exemple

```text
card "Sharpen Steel" red yellow blue
  type Warrior Action
  stats cost 0 defense 3
  resolve
    next_attack weapon gets power 3/2/1
  keyword go_again
```

## Outil

Il faut le fichier officiel (`bash tools/fetch_cards.sh`).

| Commande | Rôle |
| --- | --- |
| `cargo run -p fab-cards --bin langage -- verifier` | Réécrit chaque carte en anglais et la compare au texte officiel, à la ligne de type et aux valeurs imprimées. |
| `cargo run -p fab-cards --bin langage -- deviner "Nom"` | Encode une carte sans IA depuis le texte officiel (étage 1), ou donne la phrase qui bloque. |
| `cargo run -p fab-cards --bin langage -- comparer` | Encode automatiquement les cartes du fichier et compare au fichier écrit à la main. |
| `cargo run --release -p fab-cards --bin langage -- couverture` | Part du pool encodée sans IA, et phrases bloquantes les plus fréquentes. |

## Résultats du prototype (Warrior, 25 cartes)

- 760 tests verts avec les cartes construites depuis le fichier ; vitesse de
  simulation inchangée (≈ 7 000 parties/s Hala contre Dorinthea, `--check`).
- Réécriture : 25 cartes sur 25 identiques au texte officiel. Une erreur
  volontaire (une valeur, une condition) est bien signalée.
- Encodage sans IA : 16 cartes sur 25 produites par le script, identiques au
  fichier écrit à la main ; 9 demandent du Rust (« native »), à relire.
- Pool entier, vocabulaire Warrior seul : 65 noms sur 2 189 (3 %). Les phrases
  bloquantes les plus fréquentes sont surtout des mots-clés (Stealth, Boost,
  Cloaked, Legendary, Blade Break…) : voir `couverture`.
