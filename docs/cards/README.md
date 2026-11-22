# Sources des cartes

## Base de données communautaire (point de départ)

[the-fab-cube/flesh-and-blood-cards](https://github.com/the-fab-cube/flesh-and-blood-cards)
(fichier `json/english/card.json`, ~4 965 cartes). Elle donne pour chaque carte :
nom, couleur, pitch, coût, attaque, défense, vie et intellect des héros, types,
mots-clés, texte (`functional_text`) et légalité Silver Age (`silver_age_legal`,
`silver_age_banned`).

`bash tools/fetch_cards.sh` la télécharge dans `data/cards/sources/` (non versionné).

⚠️ C'est une base **communautaire**, pas une source officielle : chaque texte
doit être confronté au texte officiel (avec errata) avant de passer une carte en
`Validated`.

## Vérification des 9 decks contre cette base (2026-11-22)

- Les 9 héros sont bien **Young**, légaux en Silver Age et non bannis.
  Vie : 20, sauf Iyslander (18). Intellect : 4 pour tous.
- Aucune carte des 9 listes n'est illégale ou bannie en Silver Age selon la base.
- **8 cartes absentes de la base**, toutes du deck Levia (probablement une
  sortie récente) : Cleave the Heavens, Consuming Strength, Countdown to
  Extinction, Dam the Shadowake, Feasting Shadowbeast, Feeding Frenzy,
  Goremass Summoning, Hex Gauntlet. **Leurs textes devront être fournis à la main.**
- Remarque : la base classe Florian « Elemental Runeblade » (la liste d'origine
  indique « Earth Runeblade »).

## Mots-clés présents dans les 9 decks (selon la base)

Ambush, Amp, Arcane Barrier, Battleworn, Blade Break, Blood Debt, Clash, Combo,
Crush, Decompose, Dominate, Essence of Earth, Essence of Ice, Go again,
Guardwell, Heave, Ice Fusion, Legendary, Mark, Meld, Opt, Phantasm, Reprise,
Retrieve, Sharpen, Spectra, Spellvoid, Specialization, Steal, Stealth, Suspense,
Temper, Transform, Unity, Ward.

Chacun sera implémenté depuis sa définition dans le CR (chapitre 8 « Keywords »).
Les 8 cartes absentes peuvent en ajouter d'autres.
