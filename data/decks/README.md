# Listes des decks

Les 9 decks du projet, un fichier par deck (`01_ira.txt` … `09_arakni.txt`).
Le fichier d'origine fourni par le propriétaire est conservé dans `source/`.

Format :

```
# Titre libre
Hero: <nom exact du héros>
Class: <classe>
Source: <tournoi, place>

Arena:
1 <nom> [Weapon|Head|Chest|Arms|Legs|Off-Hand]

Deck:
2 <nom> (red|yellow|blue)
```

Les tests (`cargo test -p fab-cards`) vérifient que chaque liste se lit et
respecte le format Silver Age : 55 cartes au plus, au moins 40 cartes de deck,
2 copies maximum par carte unique (nom + couleur).

Statistiques : `cargo run -p fab-cards --bin deck-stats`.

| # | Héros | Classe | Arène | Deck | Source |
|---|---|---|---|---|---|
| 1 | Ira, Crimson Haze | Ninja | 7 | 48 | Sunday Showdown Sydney, 2e |
| 2 | Hala | Warrior | 8 | 47 | Sunday Showdown Sydney, 5e |
| 3 | Dorinthea | Warrior | 9 | 46 | Sunday Showdown Mexico City, 1re |
| 4 | Bravo, Flattering Showman | Guardian | 10 | 45 | Sunday Showdown Mexico City, 3e |
| 5 | Iyslander | Elemental Wizard | 10 | 45 | SAGE Rabblelander |
| 6 | Florian | Elemental Runeblade | 12 | 43 | Sunday Showdown Bangkok, 2e |
| 7 | Levia | Shadow Brute | 8 | 47 | Sunday Showdown Atlanta, 5e |
| 8 | Dromai | Draconic Illusionist | 7 | 48 | Sunday Showdown Paris, 5e |
| 9 | Arakni, Web of Deceit | Assassin | 8 | 47 | Sunday Showdown Jakarta, 5e |
