# Decks maison

Les decks importés ou construits dans la page (volet « Mes decks » de
`fab-web`), un fichier `<nom>.txt` par deck, au même format que les decks
du projet (`../README.md`). Ils sont lus au moment où on les demande : pas
besoin de recompiler.

Un deck maison est **jouable contre le bot** quand il respecte le format
Silver Age et que toutes ses cartes sont codées dans `crates/cards/`. Il ne
compte pas dans le tournoi des bots ni dans les tables apprises (seuls les
9 decks du projet y entrent).

Ces fichiers sont personnels : Git les ignore (seul ce README est suivi).
