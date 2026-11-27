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

**Deck Aurora (2026-11-27, en cours)** : les cartes nouvelles du deck Aurora
sont `Tested` (code : `crates/cards/src/real/lightning.rs`, tests :
`crates/cards/tests/cartes_lightning.rs`). Les jetons **Embodiment of
Lightning** et **Lightning Flow** sont aussi codés et testés.

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

**Étape 3, deck Levia (2026-11-23)** : toutes les cartes du deck Levia sont
`Tested` (code : `crates/cards/src/real/shadow.rs`, token Agility dans
`generic.rs` ; tests : `crates/cards/tests/cartes_shadow.rs` et
`regles_etape3_levia.rs`). Les tokens **Agility**, **Gate to i'Arathael** et
**Blasmophet, the Insatiable Hunger** sont aussi codés et testés. Darkest
Hour (cherchée par Countdown to Extinction) n'est dans aucun de nos decks :
elle n'est pas codée.

**Étape 3, deck Dromai (2026-11-23)** : toutes les cartes du deck Dromai
sont `Tested` (code : `crates/cards/src/real/draconic.rs` et `generic.rs` ;
tests : `crates/cards/tests/cartes_draconic.rs` et
`regles_etape3_dromai.rs`). Les huit dragons (faces arrière des Invoke :
Azvolai, Cromai, Kyloria, Miragai, Nekria, Ouvia, Themai, Yendurai) et les
tokens **Ash**, **Aether Ashwing** et **Spectral Shield** sont aussi codés et
testés. Nimblism et Oasis Respite servent aussi au deck Arakni.

**Étape 3, deck Arakni (2026-11-23)** : toutes les cartes du deck Arakni
sont `Tested` (code : `crates/cards/src/real/assassin.rs`, cartes génériques
Public Bounty, Pursue et Relentless Pursuit dans `generic.rs` ; tests :
`crates/cards/tests/cartes_assassin.rs` et `regles_etape3_arakni.rs`). Les
six **Agents of Chaos** (demi-héros qu'Arakni peut devenir : Black Widow,
Funnel Web, Orb-Weaver, Redback, Tarantula, Trap-Door) et les tokens
**Graphene Chelicera** et **Bloodrot Pox** sont aussi codés et testés. Toutes
les couleurs officielles des cartes du deck sont codées (le deck n'utilise
souvent que la rouge).

| Carte | Couleur | Utilisée par | Statut |
|---|---|---|---|
| **Aurora** | héros | Aurora | Tested |
| **Ira, Crimson Haze** | héros | Ira, Crimson Haze | Tested |
| **Hala** | héros | Hala | Tested |
| **Dorinthea** | héros | Dorinthea | Tested |
| **Bravo, Flattering Showman** | héros | Bravo, Flattering Showman | Tested |
| **Iyslander** | héros | Iyslander | Tested |
| **Florian** | héros | Florian | Tested |
| **Levia** | héros | Levia | Tested |
| **Dromai** | héros | Dromai | Tested |
| **Arakni, Web of Deceit** | héros | Arakni, Web of Deceit | Tested |
| Aetherstorm Wellingtons | arène | Iyslander | Tested |
| Annals of Sutcliffe | arène | Florian | Tested |
| Arcane Lantern | arène | Florian | Tested |
| Beckoning Haunt | arène | Aurora | Tested |
| Blade Beckoner Boots | arène | Dorinthea, Iyslander, Aurora | Tested |
| Blade Beckoner Gauntlets | arène | Iyslander | Tested |
| Blade Beckoner Helm | arène | Bravo, Flattering Showman, Ira, Crimson Haze, Iyslander | Tested |
| Blade Beckoner Plating | arène | Iyslander | Tested |
| Blood Scent | arène | Ira, Crimson Haze | Tested |
| Bloodied Oval | arène | Florian | Tested |
| Blossom of Spring | arène | Arakni, Web of Deceit, Florian, Aurora | Tested |
| Buzzard Helm | arène | Levia | Tested |
| Civic Steps | arène | Bravo, Flattering Showman | Tested |
| Crown of Dichotomy | arène | Florian | Tested |
| Crucible of Aetherweave | arène | Iyslander | Tested |
| Danger Digits | arène | Arakni, Web of Deceit | Tested |
| Decimator Great Axe | arène | Dorinthea | Tested |
| Deep Blue | arène | Dromai | Tested |
| Double Cross Strap | arène | Ira, Crimson Haze | Tested |
| Edge Laden Plate | arène | Hala | Tested |
| Edge of Autumn | arène | Ira, Crimson Haze | Tested |
| Embraforged Gauntlet | arène | Levia | Tested |
| Flash of Brilliance | arène | Aurora | Tested |
| Gauntlet of Boulderhold | arène | Bravo, Flattering Showman | Tested |
| Gauntlets of Unity | arène | Dorinthea, Hala | Tested |
| Hell Hammer | arène | Levia | Tested |
| Helm of Lignum Vitae | arène | Florian | Tested |
| Helm of Might and Magic | arène | Aurora | Tested |
| Helm of Unity | arène | Dorinthea, Hala | Tested |
| Hex Gauntlet | arène | Levia | Tested |
| Hooves of the Shadowbeast | arène | Levia | Tested |
| Mark of the Huntsman | arène | Arakni, Web of Deceit | Tested |
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
| Predatory Plating | arène | Levia | Tested |
| Prey Spotters | arène | Arakni, Web of Deceit | Tested |
| Refraction Bolters | arène | Dorinthea | Tested |
| Richter Scale | arène | Bravo, Flattering Showman | Tested |
| Scepter of Pain | arène | Florian | Tested |
| Seeker's Leggings | arène | Dromai | Tested |
| Silken Shroud | arène | Dromai | Tested |
| Skera Strapping | arène | Levia | Tested |
| Smoldering Scales | arène | Dromai | Tested |
| Spellfire Cloak | arène | Iyslander | Tested |
| Stalker's Steps | arène | Arakni, Web of Deceit | Tested |
| Star Fall | arène | Aurora | Tested |
| Steelbraid Buckler | arène | Bravo, Flattering Showman | Tested |
| Storm of Sandikai | arène | Dromai | Tested |
| Tearing Shuko | arène | Ira, Crimson Haze | Tested |
| Time Skippers | arène | Dromai | Tested |
| Topsy Turvy | arène | Arakni, Web of Deceit | Tested |
| Twinkle Toes | arène | Aurora | Tested |
| Unflinching Foothold | arène | Bravo, Flattering Showman, Iyslander | Tested |
| Unyielding Grip | arène | Florian | Tested |
| Wave of Reality | arène | Dromai | Tested |
| Well Grounded | arène | Florian | Tested |
| Zenith Blade | arène | Hala | Tested |
| Aethersling | red | Iyslander | Tested |
| Agile Windup | yellow | Levia | Tested |
| Amplify the Arknight | blue | Florian | Tested |
| Amplify the Arknight | red | Florian | Tested |
| Arcane Polarity | red | Arakni, Web of Deceit, Bravo, Flattering Showman, Dorinthea, Florian, Hala, Ira, Crimson Haze, Iyslander, Aurora | Tested |
| Arcane Seeds // Life | red | Florian | Tested |
| Arcane Twining | blue | Iyslander | Tested |
| Arcanic Shockwave | red | Aurora | Tested |
| Art of Desire: Body | red | Arakni, Web of Deceit | Tested |
| Aspect of Tiger: Body | red | Ira, Crimson Haze | Tested |
| Autumn's Touch | blue | Florian | Tested |
| Autumn's Touch | red | Florian | Tested |
| Autumn's Touch | yellow | Florian | Tested |
| Billowing Mirage | red | Dromai | Tested |
| Bite | red | Arakni, Web of Deceit | Tested |
| Bittering Thorns | red | Ira, Crimson Haze | Tested |
| Blaze Headlong | red | Dromai | Tested |
| Blessing of Qi | red | Ira, Crimson Haze | Tested |
| Bluster Buff | red | Ira, Crimson Haze | Tested |
| Boneyard Marauder | red | Levia | Tested |
| Boulder Drop | blue | Bravo, Flattering Showman | Tested |
| Boulder Drop | red | Bravo, Flattering Showman | Tested |
| Brutal Assault | blue | Hala | Tested |
| Buckling Blow | blue | Bravo, Flattering Showman | Tested |
| Burn Up // Shock | red | Aurora | Tested |
| Chest Puff | red | Hala, Iyslander | Tested |
| Chokeslam | blue | Bravo, Flattering Showman | Tested |
| Chokeslam | red | Bravo, Flattering Showman | Tested |
| Chorus of Rotwood | red | Florian | Tested |
| Clash of Agility | blue | Levia | Tested |
| Clash of Might | blue | Bravo, Flattering Showman | Tested |
| Clash of Mountains | blue | Bravo, Flattering Showman | Tested |
| Clash of Vigor | blue | Bravo, Flattering Showman | Tested |
| Cleave the Heavens | blue | Levia | Tested |
| Cleave the Heavens | red | Levia | Tested |
| Cloud Cover | red | Aurora | Tested |
| Colors of Aria | red | Florian | Tested |
| Concoct Disorder | red | Arakni, Web of Deceit | Tested |
| Consuming Strength | yellow | Levia | Tested |
| Countdown to Extinction | red | Levia | Tested |
| Crash and Bash | red | Bravo, Flattering Showman | Tested |
| Crush Confidence | red | Bravo, Flattering Showman | Tested |
| Cut n' Carve | blue | Hala | Tested |
| Dam the Shadowake | red | Levia | Tested |
| Deadwood Rumbler | blue | Levia | Tested |
| Debilitate | blue | Bravo, Flattering Showman | Tested |
| Debilitate | red | Bravo, Flattering Showman | Tested |
| Disable | blue | Bravo, Flattering Showman | Tested |
| Display of Craftsmanship | blue | Dorinthea, Hala | Tested |
| Display of Craftsmanship | red | Hala | Tested |
| Double Trouble | red | Arakni, Web of Deceit | Tested |
| Dread Screamer | blue | Levia | Tested |
| Dread Screamer | red | Levia | Tested |
| Dread Screamer | yellow | Levia | Tested |
| Dunebreaker Cenipai | red | Dromai | Tested |
| Dustup | red | Dromai | Tested |
| Edict of Steel | blue | Hala | Tested |
| Edict of Steel | red | Hala | Tested |
| Edict of Steel | yellow | Hala | Tested |
| Electrostatic Discharge | red | Aurora | Tested |
| Endless Maw | red | Levia | Tested |
| Engulfing Shadows | yellow | Levia | Tested |
| Entwine Lightning | red | Aurora | Tested |
| Essence of Ancestry: Body | red | Dromai | Tested |
| Fatal Engagement | blue | Hala | Tested |
| Fault Line | red | Bravo, Flattering Showman | Tested |
| Feasting Shadowbeast | red | Levia | Tested |
| Feeding Frenzy | red | Levia | Tested |
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
| Fyendal's Fighting Spirit | red | Dorinthea, Hala, Iyslander, Aurora | Tested |
| Goremass Summoning | blue | Levia | Tested |
| Graveling Growl | red | Levia | Tested |
| Graveling Growl | yellow | Levia | Tested |
| Growl | red | Ira, Crimson Haze | Tested |
| Harness Lightning | red | Aurora | Tested |
| Harvest Season | red | Florian | Tested |
| Hit and Run | blue | Dorinthea | Tested |
| Humble | red | Hala, Ira, Crimson Haze | Tested |
| Hurl | red | Arakni, Web of Deceit | Tested |
| Ice Bolt | blue | Iyslander | Tested |
| Ice Bolt | red | Iyslander | Tested |
| Ice Eternal | blue | Iyslander | Tested |
| Infect | red | Arakni, Web of Deceit | Tested |
| Invoke Azvolai | red | Dromai | Tested |
| Invoke Cromai | red | Dromai | Tested |
| Invoke Kyloria | red | Dromai | Tested |
| Invoke Miragai | red | Dromai | Tested |
| Invoke Nekria | red | Dromai | Tested |
| Invoke Ouvia | red | Dromai | Tested |
| Invoke Themai | red | Dromai | Tested |
| Invoke Yendurai | red | Dromai | Tested |
| Ironsong Response | blue | Dorinthea | Tested |
| Ironsong Response | red | Hala | Tested |
| Jack Be Quick | red | Dromai, Aurora | Tested |
| Lair of the Spider | red | Arakni, Web of Deceit | Tested |
| Lay Low | yellow | Dorinthea | Tested |
| Legacy of Ikaru | blue | Ira, Crimson Haze | Tested |
| Lightning Surge | red | Aurora | Tested |
| Look Tuff | red | Dorinthea, Iyslander | Tested |
| Macho Grande | blue | Bravo, Flattering Showman | Tested |
| Mark of the Black Widow | red | Arakni, Web of Deceit | Tested |
| Mark of the Funnel Web | red | Arakni, Web of Deceit | Tested |
| Mark the Prey | red | Arakni, Web of Deceit | Tested |
| Night's Embrace | blue | Arakni, Web of Deceit | Tested |
| Nimblism | red | Arakni, Web of Deceit, Dromai, Aurora | Tested |
| Oasis Respite | red | Arakni, Web of Deceit, Dromai | Tested |
| Oath of the Arknight | blue | Florian | Tested |
| On the Horizon | red | Dorinthea, Dromai, Florian, Hala, Ira, Crimson Haze, Iyslander | Tested |
| Orb-Weaver Spinneret | red | Arakni, Web of Deceit | Tested |
| Outed | red | Dromai | Tested |
| Overpower | blue | Dorinthea | Tested |
| Overpower | red | Dorinthea | Tested |
| Passing Mirage | blue | Dromai | Tested |
| Path of Same Ends | red | Aurora | Tested |
| Photon Splicing | blue | Iyslander | Tested |
| Pick Up the Point | red | Arakni, Web of Deceit | Tested |
| Pouncing Qi | blue | Ira, Crimson Haze | Tested |
| Pouncing Qi | red | Ira, Crimson Haze | Tested |
| Public Bounty | red | Arakni, Web of Deceit | Tested |
| Pummel | red | Bravo, Flattering Showman | Tested |
| Pursue to the Edge of Oblivion | red | Arakni, Web of Deceit | Tested |
| Pursue to the Pits of Despair | red | Arakni, Web of Deceit | Tested |
| Pyroglyphic Protection | blue | Iyslander | Tested |
| Qi Unleashed | blue | Ira, Crimson Haze | Tested |
| Quick Succession | yellow | Aurora | Tested |
| Raging Onslaught | blue | Dorinthea | Tested |
| Rake the Embers | red | Dromai | Tested |
| Ravenous Rabble | red | Arakni, Web of Deceit, Dromai, Iyslander, Aurora | Tested |
| Razor Reflex | red | Arakni, Web of Deceit, Ira, Crimson Haze | Tested |
| Razor's Edge | red | Arakni, Web of Deceit | Tested |
| Read the Runes | red | Florian | Tested |
| Reaper's Call | blue | Arakni, Web of Deceit | Tested |
| Reduce to Runechant | blue | Florian | Tested |
| Reduce to Runechant | red | Florian | Tested |
| Reduce to Runechant | yellow | Florian | Tested |
| Reinforce the Line | red | Ira, Crimson Haze | Tested |
| Relentless Pursuit | blue | Arakni, Web of Deceit | Tested |
| Rootbound Carapace | red | Florian | Tested |
| Rootbound Carapace | yellow | Florian | Tested |
| Runeblood Incantation | red | Florian | Tested |
| Rush of Power | red | Aurora | Tested |
| Sand Cover | red | Dromai | Tested |
| Scar for a Scar | red | Dromai, Ira, Crimson Haze, Iyslander, Aurora | Tested |
| Second Strike | red | Aurora | Tested |
| Seek Vengeance | blue | Ira, Crimson Haze | Tested |
| Seek Vengeance | red | Ira, Crimson Haze | Tested |
| Sharp Incline | blue | Hala | Tested |
| Sharp Incline | red | Hala | Tested |
| Sharpen Steel | blue | Dorinthea | Tested |
| Sharpen Steel | red | Dorinthea | Tested |
| Shove Off | blue | Hala | Tested |
| Shred | blue | Arakni, Web of Deceit | Tested |
| Shred | red | Arakni, Web of Deceit | Tested |
| Sift | blue | Dorinthea | Tested |
| Sigil of Silphidae | blue | Florian | Tested |
| Sigil of Voltaris | blue | Aurora | Tested |
| Sirens of Safe Harbor | blue | Dorinthea, Hala, Ira, Crimson Haze | Tested |
| Sirens of Safe Harbor | red | Dorinthea, Hala, Ira, Crimson Haze | Tested |
| Sirens of Safe Harbor | yellow | Hala, Ira, Crimson Haze | Tested |
| Sit! | red | Bravo, Flattering Showman | Tested |
| Sizzle | red | Aurora | Tested |
| Smashback Alehorn | blue | Levia | Tested |
| Snatch | red | Dromai, Aurora | Tested |
| Soul Harvest | blue | Levia | Tested |
| Stains of the Redback | red | Arakni, Web of Deceit | Tested |
| Static Shock | red | Aurora | Tested |
| Staunch Response | red | Bravo, Flattering Showman | Tested |
| Steelblade Shunt | blue | Dorinthea | Tested |
| Steelblade Shunt | red | Dorinthea, Hala | Tested |
| Steelblade Shunt | yellow | Dorinthea | Tested |
| Stinging Sprite | blue | Aurora | Tested |
| Stinging Sprite | red | Aurora | Tested |
| Stir the Aetherwinds | blue | Iyslander | Tested |
| Sweeping Blow | red | Dromai | Tested |
| Swordmaster's Path | blue | Hala | Tested |
| Swordmaster's Path | red | Hala | Tested |
| Test of Agility | red | Levia | Tested |
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
| Vigorous Smashup | blue | Levia | Tested |
| Vigorous Smashup | red | Levia | Tested |
| Vigorous Smashup | yellow | Levia | Tested |
| Visionary of Orbits | red | Aurora | Tested |
| Voltbound Duality | blue | Aurora | Tested |
| Voltic Bolt | blue | Iyslander | Tested |
| Voltic Bolt | red | Iyslander | Tested |
| Weave Lightning | red | Aurora | Tested |
| Winter's Bite | blue | Iyslander | Tested |
| Wounded Bull | red | Iyslander | Tested |
| Wrecker Romp | blue | Levia | Tested |
