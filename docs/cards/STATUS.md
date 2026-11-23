# Suivi de validation des cartes

Cycle de chaque carte : **NotCoded → Coded → Tested → Validated**.
Seul le propriétaire du projet passe une carte en **Validated**, après relecture
de son rapport de tests.

Tableau généré par `cargo run -p fab-cards --bin deck-stats -- --status`
(9 héros, 61 cartes d'arène, 191 cartes de deck nom+couleur = 154 noms).
Les versions rouge / jaune / bleue d'une carte partagent le même code d'effet
mais sont suivies séparément, car leurs valeurs (attaque, défense...) diffèrent.

**Étape 2 (2026-11-22)** : toutes les cartes des decks Hala et Dorinthea sont
`Tested` (code : `crates/cards/src/real/`, tests :
`crates/cards/tests/cartes_warrior.rs` et `cartes_generiques.rs`, valeurs
imprimées vérifiées par `python3 tools/check_coded_cards.py`). Le token
**Flurry** (créé par Edict of Steel et Toe the Line) est aussi codé et testé.

**Étape 3, deck Ira (2026-11-22)** : toutes les cartes du deck Ira sont
`Tested` (code : `crates/cards/src/real/ninja.rs` et `generic.rs`, tests :
`crates/cards/tests/cartes_ninja.rs` et `cartes_generiques.rs`). La carte
**Crouching Tiger** (créée par Aspect of Tiger, Blessing of Qi, Pouncing Paws
et Tiger Eye Reflex) est aussi codée et testée : ce n'est **pas** un token
(ligne de type officielle « Ninja Action - Attack »), mais une carte avec
ephemeral.

**Étape 3, deck Bravo (2026-11-22)** : toutes les cartes du deck Bravo sont
`Tested` (code : `crates/cards/src/real/guardian.rs` et `generic.rs`, tests :
`crates/cards/tests/cartes_guardian.rs`). Les tokens **Seismic Surge**,
**Might**, **Vigor** et **Quicken** (créés par les cartes du deck) sont aussi
codés et testés.

**Étape 3, deck Iyslander (2026-11-22)** : toutes les cartes du deck
Iyslander sont `Tested` (code : `crates/cards/src/real/wizard.rs` et
`generic.rs`, tests : `crates/cards/tests/cartes_wizard.rs` et
`regles_etape3_iyslander.rs`). Les tokens **Frostbite** (créé par Iyslander,
Ice Eternal et Frost Spike) et **Ponder** (créé par Turn to Mindfire) sont
aussi codés et testés.

**Étape 3, deck Florian (2026-11-23)** : toutes les cartes du deck Florian
sont `Tested` (code : `crates/cards/src/real/runeblade.rs`, tests :
`crates/cards/tests/cartes_runeblade.rs` et `regles_etape3_florian.rs`). Les
tokens **Runechant** et **Embodiment of Earth** sont aussi codés et testés.
Blossom of Spring sert aussi au deck Arakni.

| Carte | Couleur | Utilisée par | Statut |
|---|---|---|---|
| **Ira, Crimson Haze** | héros | Ira, Crimson Haze | Tested |
| **Hala** | héros | Hala | Tested |
| **Dorinthea** | héros | Dorinthea | Tested |
| **Bravo, Flattering Showman** | héros | Bravo, Flattering Showman | Tested |
| **Iyslander** | héros | Iyslander | Tested |
| **Florian** | héros | Florian | Tested |
| **Levia** | héros | Levia | NotCoded |
| **Dromai** | héros | Dromai | NotCoded |
| **Arakni, Web of Deceit** | héros | Arakni, Web of Deceit | NotCoded |
| Aetherstorm Wellingtons | arène | Iyslander | Tested |
| Annals of Sutcliffe | arène | Florian | Tested |
| Arcane Lantern | arène | Florian | Tested |
| Blade Beckoner Boots | arène | Dorinthea, Iyslander | Tested |
| Blade Beckoner Gauntlets | arène | Iyslander | Tested |
| Blade Beckoner Helm | arène | Bravo, Flattering Showman, Ira, Crimson Haze, Iyslander | Tested |
| Blade Beckoner Plating | arène | Iyslander | Tested |
| Blood Scent | arène | Ira, Crimson Haze | Tested |
| Bloodied Oval | arène | Florian | Tested |
| Blossom of Spring | arène | Arakni, Web of Deceit, Florian | Tested |
| Buzzard Helm | arène | Levia | NotCoded |
| Civic Steps | arène | Bravo, Flattering Showman | Tested |
| Crown of Dichotomy | arène | Florian | Tested |
| Crucible of Aetherweave | arène | Iyslander | Tested |
| Danger Digits | arène | Arakni, Web of Deceit | NotCoded |
| Decimator Great Axe | arène | Dorinthea | Tested |
| Deep Blue | arène | Dromai | NotCoded |
| Double Cross Strap | arène | Ira, Crimson Haze | Tested |
| Edge Laden Plate | arène | Hala | Tested |
| Edge of Autumn | arène | Ira, Crimson Haze | Tested |
| Embraforged Gauntlet | arène | Levia | NotCoded |
| Gauntlet of Boulderhold | arène | Bravo, Flattering Showman | Tested |
| Gauntlets of Unity | arène | Dorinthea, Hala | Tested |
| Hell Hammer | arène | Levia | NotCoded |
| Helm of Lignum Vitae | arène | Florian | Tested |
| Helm of Unity | arène | Dorinthea, Hala | Tested |
| Hex Gauntlet | arène | Levia | NotCoded |
| Hooves of the Shadowbeast | arène | Levia | NotCoded |
| Mark of the Huntsman | arène | Arakni, Web of Deceit | NotCoded |
| Mask of the Swarming Claw | arène | Ira, Crimson Haze | Tested |
| Miller's Grindstone | arène | Bravo, Flattering Showman | Tested |
| Nebula Blade | arène | Florian | Tested |
| Nullrune Boots | arène | Bravo, Flattering Showman, Hala | Tested |
| Nullrune Gloves | arène | Bravo, Flattering Showman, Dorinthea, Florian, Hala, Iyslander | Tested |
| Nullrune Hood | arène | Bravo, Flattering Showman, Dorinthea, Hala, Iyslander, Levia | Tested |
| Nullrune Robe | arène | Arakni, Web of Deceit, Dorinthea | Tested |
| Pillar of Unity | arène | Hala | Tested |
| Plating of Unity | arène | Dorinthea | Tested |
| Plume of Evergrowth | arène | Florian | Tested |
| Pouncing Paws | arène | Ira, Crimson Haze | Tested |
| Predatory Plating | arène | Levia | NotCoded |
| Prey Spotters | arène | Arakni, Web of Deceit | NotCoded |
| Refraction Bolters | arène | Dorinthea | Tested |
| Richter Scale | arène | Bravo, Flattering Showman | Tested |
| Scepter of Pain | arène | Florian | Tested |
| Seeker's Leggings | arène | Dromai | NotCoded |
| Silken Shroud | arène | Dromai | NotCoded |
| Skera Strapping | arène | Levia | NotCoded |
| Smoldering Scales | arène | Dromai | NotCoded |
| Spellfire Cloak | arène | Iyslander | Tested |
| Stalker's Steps | arène | Arakni, Web of Deceit | NotCoded |
| Steelbraid Buckler | arène | Bravo, Flattering Showman | Tested |
| Storm of Sandikai | arène | Dromai | NotCoded |
| Tearing Shuko | arène | Ira, Crimson Haze | Tested |
| Time Skippers | arène | Dromai | NotCoded |
| Topsy Turvy | arène | Arakni, Web of Deceit | NotCoded |
| Unflinching Foothold | arène | Bravo, Flattering Showman, Iyslander | Tested |
| Unyielding Grip | arène | Florian | Tested |
| Wave of Reality | arène | Dromai | NotCoded |
| Well Grounded | arène | Florian | Tested |
| Zenith Blade | arène | Hala | Tested |
| Aethersling | red | Iyslander | Tested |
| Agile Windup | yellow | Levia | NotCoded |
| Amplify the Arknight | blue | Florian | Tested |
| Amplify the Arknight | red | Florian | Tested |
| Arcane Polarity | red | Arakni, Web of Deceit, Bravo, Flattering Showman, Dorinthea, Florian, Hala, Ira, Crimson Haze, Iyslander | Tested |
| Arcane Seeds // Life | red | Florian | Tested |
| Arcane Twining | blue | Iyslander | Tested |
| Art of Desire: Body | red | Arakni, Web of Deceit | NotCoded |
| Aspect of Tiger: Body | red | Ira, Crimson Haze | Tested |
| Autumn's Touch | blue | Florian | Tested |
| Autumn's Touch | red | Florian | Tested |
| Autumn's Touch | yellow | Florian | Tested |
| Billowing Mirage | red | Dromai | NotCoded |
| Bite | red | Arakni, Web of Deceit | NotCoded |
| Bittering Thorns | red | Ira, Crimson Haze | Tested |
| Blaze Headlong | red | Dromai | NotCoded |
| Blessing of Qi | red | Ira, Crimson Haze | Tested |
| Bluster Buff | red | Ira, Crimson Haze | Tested |
| Boneyard Marauder | red | Levia | NotCoded |
| Boulder Drop | blue | Bravo, Flattering Showman | Tested |
| Boulder Drop | red | Bravo, Flattering Showman | Tested |
| Brutal Assault | blue | Hala | Tested |
| Buckling Blow | blue | Bravo, Flattering Showman | Tested |
| Chest Puff | red | Hala, Iyslander | Tested |
| Chokeslam | blue | Bravo, Flattering Showman | Tested |
| Chokeslam | red | Bravo, Flattering Showman | Tested |
| Chorus of Rotwood | red | Florian | Tested |
| Clash of Agility | blue | Levia | NotCoded |
| Clash of Might | blue | Bravo, Flattering Showman | Tested |
| Clash of Mountains | blue | Bravo, Flattering Showman | Tested |
| Clash of Vigor | blue | Bravo, Flattering Showman | Tested |
| Cleave the Heavens | blue | Levia | NotCoded |
| Cleave the Heavens | red | Levia | NotCoded |
| Colors of Aria | red | Florian | Tested |
| Concoct Disorder | red | Arakni, Web of Deceit | NotCoded |
| Consuming Strength | yellow | Levia | NotCoded |
| Countdown to Extinction | red | Levia | NotCoded |
| Crash and Bash | red | Bravo, Flattering Showman | Tested |
| Crush Confidence | red | Bravo, Flattering Showman | Tested |
| Cut n' Carve | blue | Hala | Tested |
| Dam the Shadowake | red | Levia | NotCoded |
| Deadwood Rumbler | blue | Levia | NotCoded |
| Debilitate | blue | Bravo, Flattering Showman | Tested |
| Debilitate | red | Bravo, Flattering Showman | Tested |
| Disable | blue | Bravo, Flattering Showman | Tested |
| Display of Craftsmanship | blue | Dorinthea, Hala | Tested |
| Display of Craftsmanship | red | Hala | Tested |
| Double Trouble | red | Arakni, Web of Deceit | NotCoded |
| Dread Screamer | blue | Levia | NotCoded |
| Dread Screamer | red | Levia | NotCoded |
| Dread Screamer | yellow | Levia | NotCoded |
| Dunebreaker Cenipai | red | Dromai | NotCoded |
| Dustup | red | Dromai | NotCoded |
| Edict of Steel | blue | Hala | Tested |
| Edict of Steel | red | Hala | Tested |
| Edict of Steel | yellow | Hala | Tested |
| Endless Maw | red | Levia | NotCoded |
| Engulfing Shadows | yellow | Levia | NotCoded |
| Essence of Ancestry: Body | red | Dromai | NotCoded |
| Fatal Engagement | blue | Hala | Tested |
| Fault Line | red | Bravo, Flattering Showman | Tested |
| Feasting Shadowbeast | red | Levia | NotCoded |
| Feeding Frenzy | red | Levia | NotCoded |
| Feign Vengeance | blue | Ira, Crimson Haze | Tested |
| Felling Swing | blue | Dorinthea | Tested |
| Felling Swing | red | Dorinthea | Tested |
| Felling Swing | yellow | Dorinthea | Tested |
| Flatten the Field | blue | Bravo, Flattering Showman | Tested |
| Flying Kick | blue | Ira, Crimson Haze | Tested |
| Flying Kick | red | Ira, Crimson Haze | Tested |
| Frost Spike | blue | Iyslander | Tested |
| Frosting | blue | Iyslander | Tested |
| Fruits of the Forest | blue | Florian | Tested |
| Fruits of the Forest | yellow | Florian | Tested |
| Fyendal's Fighting Spirit | blue | Dorinthea | Tested |
| Fyendal's Fighting Spirit | red | Dorinthea, Hala, Iyslander | Tested |
| Goremass Summoning | blue | Levia | NotCoded |
| Graveling Growl | red | Levia | NotCoded |
| Graveling Growl | yellow | Levia | NotCoded |
| Growl | red | Ira, Crimson Haze | Tested |
| Harvest Season | red | Florian | Tested |
| Hit and Run | blue | Dorinthea | Tested |
| Humble | red | Hala, Ira, Crimson Haze | Tested |
| Hurl | red | Arakni, Web of Deceit | NotCoded |
| Ice Bolt | blue | Iyslander | Tested |
| Ice Bolt | red | Iyslander | Tested |
| Ice Eternal | blue | Iyslander | Tested |
| Infect | red | Arakni, Web of Deceit | NotCoded |
| Invoke Azvolai | red | Dromai | NotCoded |
| Invoke Cromai | red | Dromai | NotCoded |
| Invoke Kyloria | red | Dromai | NotCoded |
| Invoke Miragai | red | Dromai | NotCoded |
| Invoke Nekria | red | Dromai | NotCoded |
| Invoke Ouvia | red | Dromai | NotCoded |
| Invoke Themai | red | Dromai | NotCoded |
| Invoke Yendurai | red | Dromai | NotCoded |
| Ironsong Response | blue | Dorinthea | Tested |
| Ironsong Response | red | Hala | Tested |
| Jack Be Quick | red | Dromai | NotCoded |
| Lair of the Spider | red | Arakni, Web of Deceit | NotCoded |
| Lay Low | yellow | Dorinthea | Tested |
| Legacy of Ikaru | blue | Ira, Crimson Haze | Tested |
| Look Tuff | red | Dorinthea, Iyslander | Tested |
| Macho Grande | blue | Bravo, Flattering Showman | Tested |
| Mark of the Black Widow | red | Arakni, Web of Deceit | NotCoded |
| Mark of the Funnel Web | red | Arakni, Web of Deceit | NotCoded |
| Mark the Prey | red | Arakni, Web of Deceit | NotCoded |
| Night's Embrace | blue | Arakni, Web of Deceit | NotCoded |
| Nimblism | red | Arakni, Web of Deceit, Dromai | NotCoded |
| Oasis Respite | red | Arakni, Web of Deceit, Dromai | NotCoded |
| Oath of the Arknight | blue | Florian | Tested |
| On the Horizon | red | Dorinthea, Dromai, Florian, Hala, Ira, Crimson Haze, Iyslander | Tested |
| Orb-Weaver Spinneret | red | Arakni, Web of Deceit | NotCoded |
| Outed | red | Dromai | NotCoded |
| Overpower | blue | Dorinthea | Tested |
| Overpower | red | Dorinthea | Tested |
| Passing Mirage | blue | Dromai | NotCoded |
| Photon Splicing | blue | Iyslander | Tested |
| Pick Up the Point | red | Arakni, Web of Deceit | NotCoded |
| Pouncing Qi | blue | Ira, Crimson Haze | Tested |
| Pouncing Qi | red | Ira, Crimson Haze | Tested |
| Public Bounty | red | Arakni, Web of Deceit | NotCoded |
| Pummel | red | Bravo, Flattering Showman | Tested |
| Pursue to the Edge of Oblivion | red | Arakni, Web of Deceit | NotCoded |
| Pursue to the Pits of Despair | red | Arakni, Web of Deceit | NotCoded |
| Pyroglyphic Protection | blue | Iyslander | Tested |
| Qi Unleashed | blue | Ira, Crimson Haze | Tested |
| Raging Onslaught | blue | Dorinthea | Tested |
| Rake the Embers | red | Dromai | NotCoded |
| Ravenous Rabble | red | Arakni, Web of Deceit, Dromai, Iyslander | Tested |
| Razor Reflex | red | Arakni, Web of Deceit, Ira, Crimson Haze | Tested |
| Razor's Edge | red | Arakni, Web of Deceit | NotCoded |
| Read the Runes | red | Florian | Tested |
| Reaper's Call | blue | Arakni, Web of Deceit | NotCoded |
| Reduce to Runechant | blue | Florian | Tested |
| Reduce to Runechant | red | Florian | Tested |
| Reduce to Runechant | yellow | Florian | Tested |
| Reinforce the Line | red | Ira, Crimson Haze | Tested |
| Relentless Pursuit | blue | Arakni, Web of Deceit | NotCoded |
| Rootbound Carapace | red | Florian | Tested |
| Rootbound Carapace | yellow | Florian | Tested |
| Runeblood Incantation | red | Florian | Tested |
| Sand Cover | red | Dromai | NotCoded |
| Scar for a Scar | red | Dromai, Ira, Crimson Haze, Iyslander | Tested |
| Seek Vengeance | blue | Ira, Crimson Haze | Tested |
| Seek Vengeance | red | Ira, Crimson Haze | Tested |
| Sharp Incline | blue | Hala | Tested |
| Sharp Incline | red | Hala | Tested |
| Sharpen Steel | blue | Dorinthea | Tested |
| Sharpen Steel | red | Dorinthea | Tested |
| Shove Off | blue | Hala | Tested |
| Shred | blue | Arakni, Web of Deceit | NotCoded |
| Shred | red | Arakni, Web of Deceit | NotCoded |
| Sift | blue | Dorinthea | Tested |
| Sigil of Silphidae | blue | Florian | Tested |
| Sirens of Safe Harbor | blue | Dorinthea, Hala, Ira, Crimson Haze | Tested |
| Sirens of Safe Harbor | red | Dorinthea, Hala, Ira, Crimson Haze | Tested |
| Sirens of Safe Harbor | yellow | Hala, Ira, Crimson Haze | Tested |
| Sit! | red | Bravo, Flattering Showman | Tested |
| Smashback Alehorn | blue | Levia | NotCoded |
| Snatch | red | Dromai | NotCoded |
| Soul Harvest | blue | Levia | NotCoded |
| Stains of the Redback | red | Arakni, Web of Deceit | NotCoded |
| Staunch Response | red | Bravo, Flattering Showman | Tested |
| Steelblade Shunt | blue | Dorinthea | Tested |
| Steelblade Shunt | red | Dorinthea, Hala | Tested |
| Steelblade Shunt | yellow | Dorinthea | Tested |
| Stir the Aetherwinds | blue | Iyslander | Tested |
| Sweeping Blow | red | Dromai | NotCoded |
| Swordmaster's Path | blue | Hala | Tested |
| Swordmaster's Path | red | Hala | Tested |
| Test of Agility | red | Levia | NotCoded |
| Test of Vigor | red | Bravo, Flattering Showman | Tested |
| The Suspense is Killing Me | blue | Bravo, Flattering Showman | Tested |
| Thunder Quake | blue | Bravo, Flattering Showman | Tested |
| Thunk | blue | Bravo, Flattering Showman | Tested |
| Tiger Eye Reflex | yellow | Ira, Crimson Haze | Tested |
| Timekeeper's Whim | blue | Iyslander | Tested |
| Timesnap Potion | blue | Dromai, Iyslander | Tested |
| Toe the Line | red | Hala | Tested |
| Turn to Mindfire | red | Iyslander | Tested |
| Unmovable | red | Iyslander | Tested |
| Untamed | red | Ira, Crimson Haze | Tested |
| Vigorous Smashup | blue | Levia | NotCoded |
| Vigorous Smashup | red | Levia | NotCoded |
| Vigorous Smashup | yellow | Levia | NotCoded |
| Voltic Bolt | blue | Iyslander | Tested |
| Voltic Bolt | red | Iyslander | Tested |
| Winter's Bite | blue | Iyslander | Tested |
| Wounded Bull | red | Iyslander | Tested |
| Wrecker Romp | blue | Levia | NotCoded |
