# Sources des cartes

## 1. Source officielle (référence)

Legend Story Studios publie toutes ses cartes dans un fichier CSV, lien
« Download Card Data » en bas de https://cardvault.fabtcg.com (la base officielle,
qui a remplacé cards.fabtcg.com). Environ 49 000 impressions, toutes langues.

Colonnes utiles (par face de carte, `face_1_…` / `face_2_…`) :
- `true_name`, `true_color`, `true_pitch`, `true_cost`, `true_power`,
  `true_defense`, `true_life`, `true_intellect` : valeurs **à jour** ;
- `true_typebox` : classe, type et sous-types ;
- `true_textbox` : **texte de règles à jour, errata compris** — c'est ce texte
  qui fait foi pour coder une carte ;
- `rules_text` : texte imprimé d'origine (peut être obsolète) ;
- `rarity`, `set_code`, `print_id` : impression.

Les cartes « meld » à deux moitiés sont notées `Arcane Seeds||Life` (nos listes
écrivent `Arcane Seeds // Life`).

## 2. Base communautaire (complément)

[the-fab-cube/flesh-and-blood-cards](https://github.com/the-fab-cube/flesh-and-blood-cards).
Utilisée **uniquement** pour ce que le fichier officiel ne donne pas, notamment
la légalité par format (`silver_age_legal`, `silver_age_banned`). En cas de
désaccord, le fichier officiel l'emporte.

## Commandes

- `bash tools/fetch_cards.sh` : télécharge les deux sources dans
  `data/cards/sources/` (non versionné : on ne republie pas les données de LSS).
- `python3 tools/check_cards.py` : vérifie que chaque carte des 9 listes existe
  dans le fichier officiel, avec la bonne couleur.

## Vérification des 9 decks (2026-11-22, fichier officiel mis à jour le même jour)

- **Les 224 cartes sont présentes** dans le fichier officiel, avec les bonnes couleurs.
- Les 9 héros sont de sous-type **Young**. Vie 20 (Iyslander 18), intellect 4.
- Selon la base communautaire, aucune carte n'est illégale ou bannie en Silver Age.
  La liste officielle des bannissements (fabtcg.com) reste à confirmer à la main.
- Le fichier officiel classe Florian « Elemental Runeblade » (la liste d'origine
  indique « Earth Runeblade »).

## Mots-clés présents dans les 9 decks

Ambush, Amp, Arcane Barrier, Battleworn, Blade Break, Blood Debt, Clash, Combo,
Crush, Decompose, Dominate, Essence of Earth, Essence of Ice, Go again,
Guardwell, Heave, Ice Fusion, Legendary, Mark, Meld, Opt, Phantasm, Reprise,
Retrieve, Sharpen, Spectra, Spellvoid, Specialization, Steal, Stealth, Suspense,
Temper, Transform, Unity, Ward — d'après la base communautaire, sans les cartes
de Levia les plus récentes (qui ajoutent au moins Blood Debt et des tokens Gate).
Liste à recalculer depuis le fichier officiel à l'étape 2.
