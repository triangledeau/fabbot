// Didacticiel « Apprendre » : le contenu des leçons (phase 2 du plan de
// rapatriement, docs/didacticiel.md). Public : débutant complet.
//
// Chaque phrase de règle cite sa source, au format CR x.y.z : les
// Comprehensive Rules officielles (rules.fabtcg.com, copie locale dans
// docs/rules/sources/, non versionnée). Rien n'est écrit de mémoire.
//
// Une leçon est une suite d'étapes :
//   { texte: "…", cr: ["4.5.3a"] }                       explication
//   { cartes: [["Brutal Assault", "bleu"]], texte: "…" }   explication avec des cartes
//   { scene: "attaquer", consigne: "…", reussi: (e) => …, bravo: "…" }
//                                                         scène jouée sur le moteur
//   { quiz: "…", choix: ["…", "…"], bonne: 0, pourquoi: "…", cr: [...] }
// Dans une scène, `e` est l'état renvoyé par /api/apprendre/etat (le même
// que celui de l'écran de partie) ; `e.debut` est l'état au lancement.
"use strict";
const LECONS = [
  {
    no: 1, titre: "Le but du jeu",
    lecons: [
      {
        id: "1-1", titre: "Ton héros et ses points de vie",
        etapes: [
          { texte: "Chaque joueur joue un <b>héros</b>. C'est une carte posée devant lui pendant toute la partie.", cr: ["1.1.2", "4.1.2"] },
          { texte: "Le héros a des <b>points de vie</b> (PV, <i>life</i>). Le nombre est en général imprimé en bas à droite de la carte du héros.", cr: ["2.5.1", "2.5.2"] },
          { texte: "<b>Tu perds la partie quand les PV de ton héros tombent à 0.</b> Tu gagnes quand ton adversaire a perdu.", cr: ["4.5.3a", "4.5.2a"] },
          { texte: "Si les deux héros tombent à 0 au même moment, la partie est nulle.", cr: ["4.5.4a"] },
          { quiz: "Quand perds-tu la partie ?", choix: ["Quand les PV de mon héros tombent à 0", "Quand je n'ai plus de cartes en main", "Quand mon adversaire a plus de PV que moi à la fin du tour"], bonne: 0,
            pourquoi: "On perd quand les PV de son héros sont réduits à 0. Une main vide ne fait pas perdre.", cr: ["4.5.3a"] },
        ],
      },
      {
        id: "1-2", titre: "Faire perdre des PV : attaquer",
        etapes: [
          { texte: "Pour faire baisser les PV adverses, on <b>attaque</b>. Une carte d'attaque a une <b>puissance</b> : le nombre imprimé en général en bas à gauche de la carte.", cr: ["2.9.1", "2.9.2"] },
          { texte: "Si personne ne se défend, l'attaque touche le héros adverse et lui inflige autant de dégâts que sa puissance. Il perd autant de PV.", cr: ["7.5.2"] },
          { scene: "attaquer",
            consigne: "C'est ton tour. Joue <b>Raging Onslaught</b> (la carte rouge) pour attaquer. Le jeu te demandera ensuite de payer son coût : choisis une carte bleue (le paiement est expliqué au chapitre 5). Quand il n'y a rien d'autre à faire, le jeu passe tout seul. Le journal, en bas à gauche, raconte ce qui s'est passé.",
            reussi: (e) => e.bot.vie < e.debut.bot.vie,
            bravo: (e) => `L'adversaire ne s'est pas défendu : il perd ${e.debut.bot.vie - e.bot.vie} PV, la puissance de Raging Onslaught rouge. Il lui reste ${e.bot.vie} PV.` },
          { quiz: "Une attaque de puissance 5 n'est pas défendue. Combien de PV perd le héros adverse ?", choix: ["5", "0", "Cela dépend de ses PV"], bonne: 0,
            pourquoi: "Sans défense, les dégâts sont égaux à la puissance de l'attaque.", cr: ["7.5.2"] },
        ],
      },
      {
        id: "1-3", titre: "Ta main et l'intellect",
        etapes: [
          { texte: "Le héros a aussi un <b>intellect</b> : le nombre de cartes que tu as en main après avoir pioché.", cr: ["2.4.1"] },
          { texte: "Au début de la partie, chaque joueur pioche jusqu'à avoir autant de cartes en main que l'intellect de son héros.", cr: ["4.1.10"] },
          { texte: "À la fin de <b>ton</b> tour, tu repioches jusqu'à ton intellect. Ton adversaire ne repioche pas à la fin de ton tour, sauf au tout premier tour de la partie.", cr: ["4.4.3f"] },
          { quiz: "Ton héros a 4 d'intellect. À la fin de ton tour, il te reste 1 carte en main. Combien en pioches-tu ?", choix: ["3", "4", "1"], bonne: 0,
            pourquoi: "Tu pioches jusqu'à avoir 4 cartes en main : 4 − 1 = 3 cartes.", cr: ["4.4.3f"] },
        ],
      },
    ],
  },
  {
    no: 2, titre: "Lire une carte",
    lecons: [
      {
        id: "2-1", titre: "Les nombres d'une carte",
        etapes: [
          { cartes: [["Raging Onslaught", "rouge"]], texte: "Voici <b>Raging Onslaught</b> rouge. Ses nombres sont imprimés aux quatre coins, toujours aux mêmes places en général." },
          { texte: "En haut à droite, le <b>coût</b> : les ressources à payer pour jouer la carte. En haut à gauche, le <b>pitch</b> : les ressources que la carte te donne si tu t'en sers pour payer.", cr: ["2.2.1", "2.2.2", "2.8.1", "2.8.2"] },
          { texte: "En bas à gauche, la <b>puissance</b> : la force de l'attaque. En bas à droite, la <b>défense</b> : ce que la carte arrête quand elle défend.", cr: ["2.9.2", "2.3.1", "2.3.2"] },
          { texte: "Une carte sans un de ces nombres n'a pas cette caractéristique. Un 0 imprimé, lui, est une vraie valeur.", cr: ["2.2.2", "2.3.2", "2.9.2"] },
          { quiz: "Raging Onslaught rouge coûte 3 et a 7 de puissance. Combien de ressources faut-il payer pour la jouer ?", choix: ["3", "7", "1"], bonne: 0,
            pourquoi: "Le coût (en haut à droite) dit ce qu'il faut payer : 3. La puissance sert au combat.", cr: ["2.2.1"] },
          { quiz: "Où lit-on en général la défense d'une carte ?", choix: ["En bas à droite", "En bas à gauche", "En haut à gauche"], bonne: 0,
            pourquoi: "La défense est en bas à droite, la puissance en bas à gauche.", cr: ["2.3.2", "2.9.2"] },
        ],
      },
      {
        id: "2-2", titre: "La couleur et le pitch",
        etapes: [
          { cartes: [["Brutal Assault", "rouge"], ["Brutal Assault", "jaune"], ["Brutal Assault", "bleu"]], texte: "La <b>couleur</b> d'une carte est la bande en haut de la carte. Beaucoup de cartes existent en trois couleurs." , cr: ["2.1.2"] },
          { texte: "En général, une carte rouge a un pitch de 1, une jaune de 2, une bleue de 3. Couleur et pitch restent deux choses distinctes : c'est le pitch imprimé qui compte.", cr: ["2.1.2a", "2.8.2"] },
          { texte: "Le plus souvent, plus le pitch est haut, plus l'attaque est faible : Brutal Assault a 6 de puissance en rouge, 5 en jaune, 4 en bleu. Tu choisis entre une carte forte et une carte qui paie bien." },
          { texte: "Le nom et le pitch ensemble font l'identité d'une carte : Brutal Assault rouge et Brutal Assault bleu sont deux cartes différentes.", cr: ["2.7.1", "2.8.1"] },
          { quiz: "Une carte bleue, pitchée, donne en général combien de ressources ?", choix: ["3", "1", "2"], bonne: 0,
            pourquoi: "Rouge 1, jaune 2, bleu 3, en général : c'est le pitch imprimé qui fait foi.", cr: ["2.1.2a"] },
        ],
      },
      {
        id: "2-3", titre: "La ligne de type",
        etapes: [
          { cartes: [["Raging Onslaught", "rouge"], ["Steelblade Shunt", "rouge"], ["Dorinthea", ""]], texte: "En bas de la carte, la <b>ligne de type</b> se lit ainsi : [classe ou talent] [type] — [sous-types]. Par exemple « Generic Action – Attack », « Warrior Defense Reaction », « Warrior Hero – Young ». Sur les cartes françaises, la ligne est traduite : « Guerrier Réaction de défense » se lit « Warrior Defense Reaction ».", cr: ["2.14.1"] },
          { texte: "Les premiers mots sont la <b>classe</b> (Warrior, Guardian, Ninja…) et le <b>talent</b> (Draconic, Ice, Shadow…), s'il y en a. Ils décident si la carte peut aller dans ton deck : ton héros doit avoir toutes les classes et tous les talents écrits. « Generic » veut dire : aucune classe, n'importe quel héros peut la jouer.", cr: ["2.11.1", "2.11.6", "2.14.1a", "1.1.3"] },
          { texte: "Ensuite vient le <b>type</b> : Action, Attack Reaction, Defense Reaction, Instant, Equipment, Weapon, Hero… Il dit comment et quand la carte se joue. Les chapitres suivants les présentent un par un.", cr: ["2.15.1", "2.15.6a"] },
          { texte: "Après le tiret viennent les <b>sous-types</b>. Certains ajoutent des règles : « Attack » fait de la carte une attaque. D'autres ne font que décrire, comme « Young » ou « Sword ».", cr: ["2.10.1", "2.10.6"] },
          { quiz: "Steelblade Shunt est une « Warrior Defense Reaction ». Quelle est sa classe ?", choix: ["Warrior", "Defense Reaction", "Elle n'en a pas"], bonne: 0,
            pourquoi: "La classe vient avant le type (ici Defense Reaction).", cr: ["2.14.1"] },
          { quiz: "Une carte « Generic » peut-elle aller dans le deck de n'importe quel héros ?", choix: ["Oui", "Non, seulement d'un héros Generic"], bonne: 0,
            pourquoi: "Generic veut dire sans classe ni talent : rien ne l'interdit à un héros.", cr: ["2.14.1a", "1.1.3"] },
        ],
      },
      {
        id: "2-4", titre: "Le texte de la carte",
        etapes: [
          { cartes: [["Sharpen Steel", "rouge"]], texte: "Au milieu de la carte, sous l'illustration, se trouve le <b>texte</b>. Il peut contenir trois choses.", cr: ["2.12.1", "2.12.2"] },
          { texte: "Le <b>texte de règles</b>, en caractères droits : c'est lui qui agit sur la partie. Un paragraphe correspond en général à une capacité.", cr: ["2.12.2", "2.12.3"] },
          { texte: "Le <b>texte de rappel</b>, en italique entre parenthèses, et le <b>texte d'ambiance</b>, en italique sous un trait : ils n'ont aucun effet sur le jeu.", cr: ["2.12.2", "2.12.3"] },
          { texte: "Le nom imprimé compte toujours dans sa version anglaise, même sur une carte française.", cr: ["2.7.4"] },
          { quiz: "Le texte d'ambiance (en italique, sous un trait) change-t-il quelque chose à la partie ?", choix: ["Non", "Oui, comme le texte de règles"], bonne: 0,
            pourquoi: "Seul le texte de règles agit. Le texte de rappel et le texte d'ambiance n'ont pas d'effet.", cr: ["2.12.3"] },
        ],
      },
    ],
  },
  { no: 3, titre: "Le tapis et les zones", lecons: [] },
  { no: 4, titre: "Le déroulement d'un tour", lecons: [] },
  { no: 5, titre: "Payer : coût et pitch", lecons: [] },
  { no: 6, titre: "Attaquer : la chaîne de combat", lecons: [] },
  { no: 7, titre: "Défendre", lecons: [] },
  { no: 8, titre: "Réactions, instants et priorité", lecons: [] },
  { no: 9, titre: "Armes et équipements", lecons: [] },
  { no: 10, titre: "Les effets", lecons: [] },
  { no: 11, titre: "Les mots-clés", lecons: [] },
  { no: 12, titre: "Arcane, prévention, jetons, cas limites", lecons: [] },
  { no: 13, titre: "Le format Silver Age", lecons: [] },
];

// Les sections des Comprehensive Rules (titres officiels, en anglais) : le
// sommaire montre, pour chacune, les leçons qui la citent. Une section sans
// leçon n'est pas encore expliquée : c'est la mesure de l'exhaustivité.
const CR_SECTIONS = [
  ["1.0", "General"],
  ["1.1", "Players"],
  ["1.2", "Objects"],
  ["1.3", "Cards"],
  ["1.4", "Attacks"],
  ["1.5", "Macros"],
  ["1.6", "Layers"],
  ["1.7", "Abilities"],
  ["1.8", "Effects"],
  ["1.9", "Events"],
  ["1.10", "Game State"],
  ["1.11", "Priority"],
  ["1.12", "Numbers and Symbols"],
  ["1.13", "Assets"],
  ["1.14", "Costs"],
  ["1.15", "Counters"],
  ["2.0", "General"],
  ["2.1", "Color"],
  ["2.2", "Cost"],
  ["2.3", "Defense"],
  ["2.4", "Intellect"],
  ["2.5", "Life"],
  ["2.6", "Metatype"],
  ["2.7", "Name"],
  ["2.8", "Pitch"],
  ["2.9", "Power"],
  ["2.10", "Subtypes"],
  ["2.11", "Supertypes"],
  ["2.12", "Text Box"],
  ["2.13", "Traits"],
  ["2.14", "Type Box"],
  ["2.15", "Types"],
  ["3.0", "General"],
  ["3.1", "Arena"],
  ["3.2", "Arms"],
  ["3.3", "Arsenal"],
  ["3.4", "Banished"],
  ["3.5", "Chest"],
  ["3.6", "Combat Chain"],
  ["3.7", "Deck"],
  ["3.8", "Graveyard"],
  ["3.9", "Hand"],
  ["3.10", "Head"],
  ["3.11", "Hero"],
  ["3.12", "Legs"],
  ["3.13", "Permanent"],
  ["3.14", "Pitch"],
  ["3.15", "Stack"],
  ["3.16", "Weapon"],
  ["4.0", "General"],
  ["4.1", "Starting a Game"],
  ["4.2", "Start Phase"],
  ["4.3", "Action Phase"],
  ["4.4", "End Phase"],
  ["4.5", "Ending a Game"],
  ["5.0", "General"],
  ["5.1", "Playing Cards"],
  ["5.2", "Activated Abilities"],
  ["5.3", "Resolution Abilities & Resolving Layers"],
  ["5.4", "Static Abilities"],
  ["6.0", "General"],
  ["6.1", "Discrete Effects"],
  ["6.2", "Continuous Effects"],
  ["6.3", "Continuous Effect Interactions"],
  ["6.4", "Replacement Effects"],
  ["6.5", "Replacement Effect Interactions"],
  ["6.6", "Triggered Effects"],
  ["7.0", "General"],
  ["7.1", "Layer Step"],
  ["7.2", "Attack Step"],
  ["7.3", "Defend Step"],
  ["7.4", "Reaction Step"],
  ["7.5", "Damage Step"],
  ["7.6", "Resolution Step"],
  ["7.7", "Close Step"],
  ["8.0", "General"],
  ["8.1", "Type Keywords"],
  ["8.2", "Subtype Keywords"],
  ["8.3", "Ability Keywords"],
  ["8.4", "Label Keywords"],
  ["8.5", "Effect Keywords"],
  ["8.6", "Token Keywords"],
  ["9.0", "General"],
  ["9.1", "Double-Faced Cards"],
  ["9.2", "Split-Cards"],
  ["9.3", "Marked"],
  ["9.4", "Living Legend"],
];
