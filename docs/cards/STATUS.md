# Suivi de validation des cartes

Cycle de chaque carte : **NotCoded → Coded → Tested → Validated**.
Seul le propriétaire du projet passe une carte en **Validated**, après relecture
de son rapport de tests.

Tableau généré par `cargo run -p fab-cards --bin deck-stats -- --status`
(9 héros, 61 cartes d'arène, 191 cartes de deck nom+couleur = 154 noms).
Les versions rouge / jaune / bleue d'une carte partagent le même code d'effet
mais sont suivies séparément, car leurs valeurs (attaque, défense...) diffèrent.

| Carte | Couleur | Utilisée par | Statut |
|---|---|---|---|
| **Ira, Crimson Haze** | héros | Ira, Crimson Haze | NotCoded |
| **Hala** | héros | Hala | NotCoded |
| **Dorinthea** | héros | Dorinthea | NotCoded |
| **Bravo, Flattering Showman** | héros | Bravo, Flattering Showman | NotCoded |
| **Iyslander** | héros | Iyslander | NotCoded |
| **Florian** | héros | Florian | NotCoded |
| **Levia** | héros | Levia | NotCoded |
| **Dromai** | héros | Dromai | NotCoded |
| **Arakni, Web of Deceit** | héros | Arakni, Web of Deceit | NotCoded |
| Aetherstorm Wellingtons | arène | Iyslander | NotCoded |
| Annals of Sutcliffe | arène | Florian | NotCoded |
| Arcane Lantern | arène | Florian | NotCoded |
| Blade Beckoner Boots | arène | Dorinthea, Iyslander | NotCoded |
| Blade Beckoner Gauntlets | arène | Iyslander | NotCoded |
| Blade Beckoner Helm | arène | Bravo, Flattering Showman, Ira, Crimson Haze, Iyslander | NotCoded |
| Blade Beckoner Plating | arène | Iyslander | NotCoded |
| Blood Scent | arène | Ira, Crimson Haze | NotCoded |
| Bloodied Oval | arène | Florian | NotCoded |
| Blossom of Spring | arène | Arakni, Web of Deceit, Florian | NotCoded |
| Buzzard Helm | arène | Levia | NotCoded |
| Civic Steps | arène | Bravo, Flattering Showman | NotCoded |
| Crown of Dichotomy | arène | Florian | NotCoded |
| Crucible of Aetherweave | arène | Iyslander | NotCoded |
| Danger Digits | arène | Arakni, Web of Deceit | NotCoded |
| Decimator Great Axe | arène | Dorinthea | NotCoded |
| Deep Blue | arène | Dromai | NotCoded |
| Double Cross Strap | arène | Ira, Crimson Haze | NotCoded |
| Edge Laden Plate | arène | Hala | NotCoded |
| Edge of Autumn | arène | Ira, Crimson Haze | NotCoded |
| Embraforged Gauntlet | arène | Levia | NotCoded |
| Gauntlet of Boulderhold | arène | Bravo, Flattering Showman | NotCoded |
| Gauntlets of Unity | arène | Dorinthea, Hala | NotCoded |
| Hell Hammer | arène | Levia | NotCoded |
| Helm of Lignum Vitae | arène | Florian | NotCoded |
| Helm of Unity | arène | Dorinthea, Hala | NotCoded |
| Hex Gauntlet | arène | Levia | NotCoded |
| Hooves of the Shadowbeast | arène | Levia | NotCoded |
| Mark of the Huntsman | arène | Arakni, Web of Deceit | NotCoded |
| Mask of the Swarming Claw | arène | Ira, Crimson Haze | NotCoded |
| Miller's Grindstone | arène | Bravo, Flattering Showman | NotCoded |
| Nebula Blade | arène | Florian | NotCoded |
| Nullrune Boots | arène | Bravo, Flattering Showman, Hala | NotCoded |
| Nullrune Gloves | arène | Bravo, Flattering Showman, Dorinthea, Florian, Hala, Iyslander | NotCoded |
| Nullrune Hood | arène | Bravo, Flattering Showman, Dorinthea, Hala, Iyslander, Levia | NotCoded |
| Nullrune Robe | arène | Arakni, Web of Deceit, Dorinthea | NotCoded |
| Pillar of Unity | arène | Hala | NotCoded |
| Plating of Unity | arène | Dorinthea | NotCoded |
| Plume of Evergrowth | arène | Florian | NotCoded |
| Pouncing Paws | arène | Ira, Crimson Haze | NotCoded |
| Predatory Plating | arène | Levia | NotCoded |
| Prey Spotters | arène | Arakni, Web of Deceit | NotCoded |
| Refraction Bolters | arène | Dorinthea | NotCoded |
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
| Tearing Shuko | arène | Ira, Crimson Haze | NotCoded |
| Time Skippers | arène | Dromai | NotCoded |
| Topsy Turvy | arène | Arakni, Web of Deceit | NotCoded |
| Unflinching Foothold | arène | Bravo, Flattering Showman, Iyslander | NotCoded |
| Unyielding Grip | arène | Florian | NotCoded |
| Wave of Reality | arène | Dromai | NotCoded |
| Well Grounded | arène | Florian | NotCoded |
| Zenith Blade | arène | Hala | NotCoded |
| Aethersling | red | Iyslander | NotCoded |
| Agile Windup | yellow | Levia | NotCoded |
| Amplify the Arknight | blue | Florian | NotCoded |
| Amplify the Arknight | red | Florian | NotCoded |
| Arcane Polarity | red | Arakni, Web of Deceit, Bravo, Flattering Showman, Dorinthea, Florian, Hala, Ira, Crimson Haze, Iyslander | NotCoded |
| Arcane Seeds // Life | red | Florian | NotCoded |
| Arcane Twining | blue | Iyslander | NotCoded |
| Art of Desire: Body | red | Arakni, Web of Deceit | NotCoded |
| Aspect of Tiger: Body | red | Ira, Crimson Haze | NotCoded |
| Autumn's Touch | blue | Florian | NotCoded |
| Autumn's Touch | red | Florian | NotCoded |
| Autumn's Touch | yellow | Florian | NotCoded |
| Billowing Mirage | red | Dromai | NotCoded |
| Bite | red | Arakni, Web of Deceit | NotCoded |
| Bittering Thorns | red | Ira, Crimson Haze | NotCoded |
| Blaze Headlong | red | Dromai | NotCoded |
| Blessing of Qi | red | Ira, Crimson Haze | NotCoded |
| Bluster Buff | red | Ira, Crimson Haze | NotCoded |
| Boneyard Marauder | red | Levia | NotCoded |
| Boulder Drop | blue | Bravo, Flattering Showman | NotCoded |
| Boulder Drop | red | Bravo, Flattering Showman | NotCoded |
| Brutal Assault | blue | Hala | NotCoded |
| Buckling Blow | blue | Bravo, Flattering Showman | NotCoded |
| Chest Puff | red | Hala, Iyslander | NotCoded |
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
| Cut n' Carve | blue | Hala | NotCoded |
| Dam the Shadowake | red | Levia | NotCoded |
| Deadwood Rumbler | blue | Levia | NotCoded |
| Debilitate | blue | Bravo, Flattering Showman | NotCoded |
| Debilitate | red | Bravo, Flattering Showman | NotCoded |
| Disable | blue | Bravo, Flattering Showman | NotCoded |
| Display of Craftsmanship | blue | Dorinthea, Hala | NotCoded |
| Display of Craftsmanship | red | Hala | NotCoded |
| Double Trouble | red | Arakni, Web of Deceit | NotCoded |
| Dread Screamer | blue | Levia | NotCoded |
| Dread Screamer | red | Levia | NotCoded |
| Dread Screamer | yellow | Levia | NotCoded |
| Dunebreaker Cenipai | red | Dromai | NotCoded |
| Dustup | red | Dromai | NotCoded |
| Edict of Steel | blue | Hala | NotCoded |
| Edict of Steel | red | Hala | NotCoded |
| Edict of Steel | yellow | Hala | NotCoded |
| Endless Maw | red | Levia | NotCoded |
| Engulfing Shadows | yellow | Levia | NotCoded |
| Essence of Ancestry: Body | red | Dromai | NotCoded |
| Fatal Engagement | blue | Hala | NotCoded |
| Fault Line | red | Bravo, Flattering Showman | NotCoded |
| Feasting Shadowbeast | red | Levia | NotCoded |
| Feeding Frenzy | red | Levia | NotCoded |
| Feign Vengeance | blue | Ira, Crimson Haze | NotCoded |
| Felling Swing | blue | Dorinthea | NotCoded |
| Felling Swing | red | Dorinthea | NotCoded |
| Felling Swing | yellow | Dorinthea | NotCoded |
| Flatten the Field | blue | Bravo, Flattering Showman | NotCoded |
| Flying Kick | blue | Ira, Crimson Haze | NotCoded |
| Flying Kick | red | Ira, Crimson Haze | NotCoded |
| Frost Spike | blue | Iyslander | NotCoded |
| Frosting | blue | Iyslander | NotCoded |
| Fruits of the Forest | blue | Florian | NotCoded |
| Fruits of the Forest | yellow | Florian | NotCoded |
| Fyendal's Fighting Spirit | blue | Dorinthea | NotCoded |
| Fyendal's Fighting Spirit | red | Dorinthea, Hala, Iyslander | NotCoded |
| Goremass Summoning | blue | Levia | NotCoded |
| Graveling Growl | red | Levia | NotCoded |
| Graveling Growl | yellow | Levia | NotCoded |
| Growl | red | Ira, Crimson Haze | NotCoded |
| Harvest Season | red | Florian | NotCoded |
| Hit and Run | blue | Dorinthea | NotCoded |
| Humble | red | Hala, Ira, Crimson Haze | NotCoded |
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
| Ironsong Response | blue | Dorinthea | NotCoded |
| Ironsong Response | red | Hala | NotCoded |
| Jack Be Quick | red | Dromai | NotCoded |
| Lair of the Spider | red | Arakni, Web of Deceit | NotCoded |
| Lay Low | yellow | Dorinthea | NotCoded |
| Legacy of Ikaru | blue | Ira, Crimson Haze | NotCoded |
| Look Tuff | red | Dorinthea, Iyslander | NotCoded |
| Macho Grande | blue | Bravo, Flattering Showman | NotCoded |
| Mark of the Black Widow | red | Arakni, Web of Deceit | NotCoded |
| Mark of the Funnel Web | red | Arakni, Web of Deceit | NotCoded |
| Mark the Prey | red | Arakni, Web of Deceit | NotCoded |
| Night's Embrace | blue | Arakni, Web of Deceit | NotCoded |
| Nimblism | red | Arakni, Web of Deceit, Dromai | NotCoded |
| Oasis Respite | red | Arakni, Web of Deceit, Dromai | NotCoded |
| Oath of the Arknight | blue | Florian | NotCoded |
| On the Horizon | red | Dorinthea, Dromai, Florian, Hala, Ira, Crimson Haze, Iyslander | NotCoded |
| Orb-Weaver Spinneret | red | Arakni, Web of Deceit | NotCoded |
| Outed | red | Dromai | NotCoded |
| Overpower | blue | Dorinthea | NotCoded |
| Overpower | red | Dorinthea | NotCoded |
| Passing Mirage | blue | Dromai | NotCoded |
| Photon Splicing | blue | Iyslander | NotCoded |
| Pick Up the Point | red | Arakni, Web of Deceit | NotCoded |
| Pouncing Qi | blue | Ira, Crimson Haze | NotCoded |
| Pouncing Qi | red | Ira, Crimson Haze | NotCoded |
| Public Bounty | red | Arakni, Web of Deceit | NotCoded |
| Pummel | red | Bravo, Flattering Showman | NotCoded |
| Pursue to the Edge of Oblivion | red | Arakni, Web of Deceit | NotCoded |
| Pursue to the Pits of Despair | red | Arakni, Web of Deceit | NotCoded |
| Pyroglyphic Protection | blue | Iyslander | NotCoded |
| Qi Unleashed | blue | Ira, Crimson Haze | NotCoded |
| Raging Onslaught | blue | Dorinthea | NotCoded |
| Rake the Embers | red | Dromai | NotCoded |
| Ravenous Rabble | red | Arakni, Web of Deceit, Dromai, Iyslander | NotCoded |
| Razor Reflex | red | Arakni, Web of Deceit, Ira, Crimson Haze | NotCoded |
| Razor's Edge | red | Arakni, Web of Deceit | NotCoded |
| Read the Runes | red | Florian | NotCoded |
| Reaper's Call | blue | Arakni, Web of Deceit | NotCoded |
| Reduce to Runechant | blue | Florian | NotCoded |
| Reduce to Runechant | red | Florian | NotCoded |
| Reduce to Runechant | yellow | Florian | NotCoded |
| Reinforce the Line | red | Ira, Crimson Haze | NotCoded |
| Relentless Pursuit | blue | Arakni, Web of Deceit | NotCoded |
| Rootbound Carapace | red | Florian | NotCoded |
| Rootbound Carapace | yellow | Florian | NotCoded |
| Runeblood Incantation | red | Florian | NotCoded |
| Sand Cover | red | Dromai | NotCoded |
| Scar for a Scar | red | Dromai, Ira, Crimson Haze, Iyslander | NotCoded |
| Seek Vengeance | blue | Ira, Crimson Haze | NotCoded |
| Seek Vengeance | red | Ira, Crimson Haze | NotCoded |
| Sharp Incline | blue | Hala | NotCoded |
| Sharp Incline | red | Hala | NotCoded |
| Sharpen Steel | blue | Dorinthea | NotCoded |
| Sharpen Steel | red | Dorinthea | NotCoded |
| Shove Off | blue | Hala | NotCoded |
| Shred | blue | Arakni, Web of Deceit | NotCoded |
| Shred | red | Arakni, Web of Deceit | NotCoded |
| Sift | blue | Dorinthea | NotCoded |
| Sigil of Silphidae | blue | Florian | NotCoded |
| Sirens of Safe Harbor | blue | Dorinthea, Hala, Ira, Crimson Haze | NotCoded |
| Sirens of Safe Harbor | red | Dorinthea, Hala, Ira, Crimson Haze | NotCoded |
| Sirens of Safe Harbor | yellow | Hala, Ira, Crimson Haze | NotCoded |
| Sit! | red | Bravo, Flattering Showman | NotCoded |
| Smashback Alehorn | blue | Levia | NotCoded |
| Snatch | red | Dromai | NotCoded |
| Soul Harvest | blue | Levia | NotCoded |
| Stains of the Redback | red | Arakni, Web of Deceit | NotCoded |
| Staunch Response | red | Bravo, Flattering Showman | NotCoded |
| Steelblade Shunt | blue | Dorinthea | NotCoded |
| Steelblade Shunt | red | Dorinthea, Hala | NotCoded |
| Steelblade Shunt | yellow | Dorinthea | NotCoded |
| Stir the Aetherwinds | blue | Iyslander | NotCoded |
| Sweeping Blow | red | Dromai | NotCoded |
| Swordmaster's Path | blue | Hala | NotCoded |
| Swordmaster's Path | red | Hala | NotCoded |
| Test of Agility | red | Levia | NotCoded |
| Test of Vigor | red | Bravo, Flattering Showman | NotCoded |
| The Suspense is Killing Me | blue | Bravo, Flattering Showman | NotCoded |
| Thunder Quake | blue | Bravo, Flattering Showman | NotCoded |
| Thunk | blue | Bravo, Flattering Showman | NotCoded |
| Tiger Eye Reflex | yellow | Ira, Crimson Haze | NotCoded |
| Timekeeper's Whim | blue | Iyslander | NotCoded |
| Timesnap Potion | blue | Dromai, Iyslander | NotCoded |
| Toe the Line | red | Hala | NotCoded |
| Turn to Mindfire | red | Iyslander | NotCoded |
| Unmovable | red | Iyslander | NotCoded |
| Untamed | red | Ira, Crimson Haze | NotCoded |
| Vigorous Smashup | blue | Levia | NotCoded |
| Vigorous Smashup | red | Levia | NotCoded |
| Vigorous Smashup | yellow | Levia | NotCoded |
| Voltic Bolt | blue | Iyslander | NotCoded |
| Voltic Bolt | red | Iyslander | NotCoded |
| Winter's Bite | blue | Iyslander | NotCoded |
| Wounded Bull | red | Iyslander | NotCoded |
| Wrecker Romp | blue | Levia | NotCoded |
