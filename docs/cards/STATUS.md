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

| Carte | Couleur | Utilisée par | Statut |
|---|---|---|---|
| **Ira, Crimson Haze** | héros | Ira, Crimson Haze | Tested |
| **Hala** | héros | Hala | Tested |
| **Dorinthea** | héros | Dorinthea | Tested |
| **Bravo, Flattering Showman** | héros | Bravo, Flattering Showman | NotCoded |
| **Iyslander** | héros | Iyslander | NotCoded |
| **Florian** | héros | Florian | NotCoded |
| **Levia** | héros | Levia | NotCoded |
| **Dromai** | héros | Dromai | NotCoded |
| **Arakni, Web of Deceit** | héros | Arakni, Web of Deceit | NotCoded |
| Aetherstorm Wellingtons | arène | Iyslander | NotCoded |
| Annals of Sutcliffe | arène | Florian | NotCoded |
| Arcane Lantern | arène | Florian | NotCoded |
| Blade Beckoner Boots | arène | Dorinthea, Iyslander | Tested |
| Blade Beckoner Gauntlets | arène | Iyslander | NotCoded |
| Blade Beckoner Helm | arène | Bravo, Flattering Showman, Ira, Crimson Haze, Iyslander | Tested |
| Blade Beckoner Plating | arène | Iyslander | NotCoded |
| Blood Scent | arène | Ira, Crimson Haze | Tested |
| Bloodied Oval | arène | Florian | NotCoded |
| Blossom of Spring | arène | Arakni, Web of Deceit, Florian | NotCoded |
| Buzzard Helm | arène | Levia | NotCoded |
| Civic Steps | arène | Bravo, Flattering Showman | NotCoded |
| Crown of Dichotomy | arène | Florian | NotCoded |
| Crucible of Aetherweave | arène | Iyslander | NotCoded |
| Danger Digits | arène | Arakni, Web of Deceit | NotCoded |
| Decimator Great Axe | arène | Dorinthea | Tested |
| Deep Blue | arène | Dromai | NotCoded |
| Double Cross Strap | arène | Ira, Crimson Haze | Tested |
| Edge Laden Plate | arène | Hala | Tested |
| Edge of Autumn | arène | Ira, Crimson Haze | Tested |
| Embraforged Gauntlet | arène | Levia | NotCoded |
| Gauntlet of Boulderhold | arène | Bravo, Flattering Showman | NotCoded |
| Gauntlets of Unity | arène | Dorinthea, Hala | Tested |
| Hell Hammer | arène | Levia | NotCoded |
| Helm of Lignum Vitae | arène | Florian | NotCoded |
| Helm of Unity | arène | Dorinthea, Hala | Tested |
| Hex Gauntlet | arène | Levia | NotCoded |
| Hooves of the Shadowbeast | arène | Levia | NotCoded |
| Mark of the Huntsman | arène | Arakni, Web of Deceit | NotCoded |
| Mask of the Swarming Claw | arène | Ira, Crimson Haze | Tested |
| Miller's Grindstone | arène | Bravo, Flattering Showman | NotCoded |
| Nebula Blade | arène | Florian | NotCoded |
| Nullrune Boots | arène | Bravo, Flattering Showman, Hala | Tested |
| Nullrune Gloves | arène | Bravo, Flattering Showman, Dorinthea, Florian, Hala, Iyslander | Tested |
| Nullrune Hood | arène | Bravo, Flattering Showman, Dorinthea, Hala, Iyslander, Levia | Tested |
| Nullrune Robe | arène | Arakni, Web of Deceit, Dorinthea | Tested |
| Pillar of Unity | arène | Hala | Tested |
| Plating of Unity | arène | Dorinthea | Tested |
| Plume of Evergrowth | arène | Florian | NotCoded |
| Pouncing Paws | arène | Ira, Crimson Haze | Tested |
| Predatory Plating | arène | Levia | NotCoded |
| Prey Spotters | arène | Arakni, Web of Deceit | NotCoded |
| Refraction Bolters | arène | Dorinthea | Tested |
| Richter Scale | arène | Bravo, Flattering Showman | NotCoded |
| Scepter of Pain | arène | Florian | NotCoded |
| Seeker's Leggings | arène | Dromai | NotCoded |
| Silken Shroud | arène | Dromai | NotCoded |
| Skera Strapping | arène | Levia | NotCoded |
| Smoldering Scales | arène | Dromai | NotCoded |
| Spellfire Cloak | arène | Iyslander | NotCoded |
| Stalker's Steps | arène | Arakni, Web of Deceit | NotCoded |
| Steelbraid Buckler | arène | Bravo, Flattering Showman | NotCoded |
| Storm of Sandikai | arène | Dromai | NotCoded |
| Tearing Shuko | arène | Ira, Crimson Haze | Tested |
| Time Skippers | arène | Dromai | NotCoded |
| Topsy Turvy | arène | Arakni, Web of Deceit | NotCoded |
| Unflinching Foothold | arène | Bravo, Flattering Showman, Iyslander | NotCoded |
| Unyielding Grip | arène | Florian | NotCoded |
| Wave of Reality | arène | Dromai | NotCoded |
| Well Grounded | arène | Florian | NotCoded |
| Zenith Blade | arène | Hala | Tested |
| Aethersling | red | Iyslander | NotCoded |
| Agile Windup | yellow | Levia | NotCoded |
| Amplify the Arknight | blue | Florian | NotCoded |
| Amplify the Arknight | red | Florian | NotCoded |
| Arcane Polarity | red | Arakni, Web of Deceit, Bravo, Flattering Showman, Dorinthea, Florian, Hala, Ira, Crimson Haze, Iyslander | Tested |
| Arcane Seeds // Life | red | Florian | NotCoded |
| Arcane Twining | blue | Iyslander | NotCoded |
| Art of Desire: Body | red | Arakni, Web of Deceit | NotCoded |
| Aspect of Tiger: Body | red | Ira, Crimson Haze | Tested |
| Autumn's Touch | blue | Florian | NotCoded |
| Autumn's Touch | red | Florian | NotCoded |
| Autumn's Touch | yellow | Florian | NotCoded |
| Billowing Mirage | red | Dromai | NotCoded |
| Bite | red | Arakni, Web of Deceit | NotCoded |
| Bittering Thorns | red | Ira, Crimson Haze | Tested |
| Blaze Headlong | red | Dromai | NotCoded |
| Blessing of Qi | red | Ira, Crimson Haze | Tested |
| Bluster Buff | red | Ira, Crimson Haze | Tested |
| Boneyard Marauder | red | Levia | NotCoded |
| Boulder Drop | blue | Bravo, Flattering Showman | NotCoded |
| Boulder Drop | red | Bravo, Flattering Showman | NotCoded |
| Brutal Assault | blue | Hala | Tested |
| Buckling Blow | blue | Bravo, Flattering Showman | NotCoded |
| Chest Puff | red | Hala, Iyslander | Tested |
| Chokeslam | blue | Bravo, Flattering Showman | NotCoded |
| Chokeslam | red | Bravo, Flattering Showman | NotCoded |
| Chorus of Rotwood | red | Florian | NotCoded |
| Clash of Agility | blue | Levia | NotCoded |
| Clash of Might | blue | Bravo, Flattering Showman | NotCoded |
| Clash of Mountains | blue | Bravo, Flattering Showman | NotCoded |
| Clash of Vigor | blue | Bravo, Flattering Showman | NotCoded |
| Cleave the Heavens | blue | Levia | NotCoded |
| Cleave the Heavens | red | Levia | NotCoded |
| Colors of Aria | red | Florian | NotCoded |
| Concoct Disorder | red | Arakni, Web of Deceit | NotCoded |
| Consuming Strength | yellow | Levia | NotCoded |
| Countdown to Extinction | red | Levia | NotCoded |
| Crash and Bash | red | Bravo, Flattering Showman | NotCoded |
| Crush Confidence | red | Bravo, Flattering Showman | NotCoded |
| Cut n' Carve | blue | Hala | Tested |
| Dam the Shadowake | red | Levia | NotCoded |
| Deadwood Rumbler | blue | Levia | NotCoded |
| Debilitate | blue | Bravo, Flattering Showman | NotCoded |
| Debilitate | red | Bravo, Flattering Showman | NotCoded |
| Disable | blue | Bravo, Flattering Showman | NotCoded |
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
| Fault Line | red | Bravo, Flattering Showman | NotCoded |
| Feasting Shadowbeast | red | Levia | NotCoded |
| Feeding Frenzy | red | Levia | NotCoded |
| Feign Vengeance | blue | Ira, Crimson Haze | Tested |
| Felling Swing | blue | Dorinthea | Tested |
| Felling Swing | red | Dorinthea | Tested |
| Felling Swing | yellow | Dorinthea | Tested |
| Flatten the Field | blue | Bravo, Flattering Showman | NotCoded |
| Flying Kick | blue | Ira, Crimson Haze | Tested |
| Flying Kick | red | Ira, Crimson Haze | Tested |
| Frost Spike | blue | Iyslander | NotCoded |
| Frosting | blue | Iyslander | NotCoded |
| Fruits of the Forest | blue | Florian | NotCoded |
| Fruits of the Forest | yellow | Florian | NotCoded |
| Fyendal's Fighting Spirit | blue | Dorinthea | Tested |
| Fyendal's Fighting Spirit | red | Dorinthea, Hala, Iyslander | Tested |
| Goremass Summoning | blue | Levia | NotCoded |
| Graveling Growl | red | Levia | NotCoded |
| Graveling Growl | yellow | Levia | NotCoded |
| Growl | red | Ira, Crimson Haze | Tested |
| Harvest Season | red | Florian | NotCoded |
| Hit and Run | blue | Dorinthea | Tested |
| Humble | red | Hala, Ira, Crimson Haze | Tested |
| Hurl | red | Arakni, Web of Deceit | NotCoded |
| Ice Bolt | blue | Iyslander | NotCoded |
| Ice Bolt | red | Iyslander | NotCoded |
| Ice Eternal | blue | Iyslander | NotCoded |
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
| Macho Grande | blue | Bravo, Flattering Showman | NotCoded |
| Mark of the Black Widow | red | Arakni, Web of Deceit | NotCoded |
| Mark of the Funnel Web | red | Arakni, Web of Deceit | NotCoded |
| Mark the Prey | red | Arakni, Web of Deceit | NotCoded |
| Night's Embrace | blue | Arakni, Web of Deceit | NotCoded |
| Nimblism | red | Arakni, Web of Deceit, Dromai | NotCoded |
| Oasis Respite | red | Arakni, Web of Deceit, Dromai | NotCoded |
| Oath of the Arknight | blue | Florian | NotCoded |
| On the Horizon | red | Dorinthea, Dromai, Florian, Hala, Ira, Crimson Haze, Iyslander | Tested |
| Orb-Weaver Spinneret | red | Arakni, Web of Deceit | NotCoded |
| Outed | red | Dromai | NotCoded |
| Overpower | blue | Dorinthea | Tested |
| Overpower | red | Dorinthea | Tested |
| Passing Mirage | blue | Dromai | NotCoded |
| Photon Splicing | blue | Iyslander | NotCoded |
| Pick Up the Point | red | Arakni, Web of Deceit | NotCoded |
| Pouncing Qi | blue | Ira, Crimson Haze | Tested |
| Pouncing Qi | red | Ira, Crimson Haze | Tested |
| Public Bounty | red | Arakni, Web of Deceit | NotCoded |
| Pummel | red | Bravo, Flattering Showman | NotCoded |
| Pursue to the Edge of Oblivion | red | Arakni, Web of Deceit | NotCoded |
| Pursue to the Pits of Despair | red | Arakni, Web of Deceit | NotCoded |
| Pyroglyphic Protection | blue | Iyslander | NotCoded |
| Qi Unleashed | blue | Ira, Crimson Haze | Tested |
| Raging Onslaught | blue | Dorinthea | Tested |
| Rake the Embers | red | Dromai | NotCoded |
| Ravenous Rabble | red | Arakni, Web of Deceit, Dromai, Iyslander | NotCoded |
| Razor Reflex | red | Arakni, Web of Deceit, Ira, Crimson Haze | Tested |
| Razor's Edge | red | Arakni, Web of Deceit | NotCoded |
| Read the Runes | red | Florian | NotCoded |
| Reaper's Call | blue | Arakni, Web of Deceit | NotCoded |
| Reduce to Runechant | blue | Florian | NotCoded |
| Reduce to Runechant | red | Florian | NotCoded |
| Reduce to Runechant | yellow | Florian | NotCoded |
| Reinforce the Line | red | Ira, Crimson Haze | Tested |
| Relentless Pursuit | blue | Arakni, Web of Deceit | NotCoded |
| Rootbound Carapace | red | Florian | NotCoded |
| Rootbound Carapace | yellow | Florian | NotCoded |
| Runeblood Incantation | red | Florian | NotCoded |
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
| Sigil of Silphidae | blue | Florian | NotCoded |
| Sirens of Safe Harbor | blue | Dorinthea, Hala, Ira, Crimson Haze | Tested |
| Sirens of Safe Harbor | red | Dorinthea, Hala, Ira, Crimson Haze | Tested |
| Sirens of Safe Harbor | yellow | Hala, Ira, Crimson Haze | Tested |
| Sit! | red | Bravo, Flattering Showman | NotCoded |
| Smashback Alehorn | blue | Levia | NotCoded |
| Snatch | red | Dromai | NotCoded |
| Soul Harvest | blue | Levia | NotCoded |
| Stains of the Redback | red | Arakni, Web of Deceit | NotCoded |
| Staunch Response | red | Bravo, Flattering Showman | NotCoded |
| Steelblade Shunt | blue | Dorinthea | Tested |
| Steelblade Shunt | red | Dorinthea, Hala | Tested |
| Steelblade Shunt | yellow | Dorinthea | Tested |
| Stir the Aetherwinds | blue | Iyslander | NotCoded |
| Sweeping Blow | red | Dromai | NotCoded |
| Swordmaster's Path | blue | Hala | Tested |
| Swordmaster's Path | red | Hala | Tested |
| Test of Agility | red | Levia | NotCoded |
| Test of Vigor | red | Bravo, Flattering Showman | NotCoded |
| The Suspense is Killing Me | blue | Bravo, Flattering Showman | NotCoded |
| Thunder Quake | blue | Bravo, Flattering Showman | NotCoded |
| Thunk | blue | Bravo, Flattering Showman | NotCoded |
| Tiger Eye Reflex | yellow | Ira, Crimson Haze | Tested |
| Timekeeper's Whim | blue | Iyslander | NotCoded |
| Timesnap Potion | blue | Dromai, Iyslander | NotCoded |
| Toe the Line | red | Hala | Tested |
| Turn to Mindfire | red | Iyslander | NotCoded |
| Unmovable | red | Iyslander | NotCoded |
| Untamed | red | Ira, Crimson Haze | Tested |
| Vigorous Smashup | blue | Levia | NotCoded |
| Vigorous Smashup | red | Levia | NotCoded |
| Vigorous Smashup | yellow | Levia | NotCoded |
| Voltic Bolt | blue | Iyslander | NotCoded |
| Voltic Bolt | red | Iyslander | NotCoded |
| Winter's Bite | blue | Iyslander | NotCoded |
| Wounded Bull | red | Iyslander | NotCoded |
| Wrecker Romp | blue | Levia | NotCoded |
