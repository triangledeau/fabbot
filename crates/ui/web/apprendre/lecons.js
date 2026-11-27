// Didacticiel « Apprendre » : le contenu (phase 2 du plan de rapatriement,
// docs/didacticiel.md). Public : débutant complet.
//
// Règles d'écriture (vérifiées par le test `ordre_des_notions` de
// crates/ui/src/api_apprendre.rs) :
// - chaque mot du jeu s'écrit {{notion}} (ou {{notion|forme affichée}}) ;
// - une notion n'est employée qu'après le plan qui l'introduit
//   (`introduit: [...]`, écrit AVANT `dit` dans le plan) ;
// - chaque phrase de règle cite sa source : `cr: ["4.5.3a"]`, les
//   Comprehensive Rules officielles (copie locale : docs/rules/sources/).
//
// Une leçon est une suite de PLANS joués sur le tapis (apprendre.js) :
//   introduit  notions expliquées dans ce plan
//   dit        ce que dit le guide (HTML court, {{notions}})
//   fait       actions animées sur le tapis, dans l'ordre (voir ACTIONS dans apprendre.js)
//   clic       id d'une carte que l'élève doit cliquer ; `apres` : actions qui suivent le clic,
//              `ensuite` : ce que dit le guide après le clic
//   quiz       { q, choix, bonne, pourquoi }
//   scene      partie sur le vrai moteur : { id, consigne, reussi(e), bravo(e) }
"use strict";

// Les notions : mot affiché, icône officielle (p puissance, d défense,
// r ressource, h vie, i intellect) et définition courte (bulle au survol).
const NOTIONS = {
  heros: { mot: "héros", def: "La grande carte qui te représente pendant toute la partie." },
  pv: { mot: "PV", icone: "h", def: "Points de vie du héros. À 0, son joueur perd." },
  main: { mot: "main", def: "Les cartes que tu tiens. Toi seul les vois." },
  attaque: { mot: "attaque", def: "Une carte qui frappe le héros adverse." },
  puissance: { mot: "puissance", icone: "p", def: "La force d'une attaque, en bas à gauche de la carte." },
  degats: { mot: "dégâts", def: "Ce que perd un héros touché : la puissance moins la défense." },
  defense: { mot: "défense", icone: "d", def: "Ce qu'une carte arrête quand elle défend, en bas à droite." },
  defendre: { mot: "défendre", def: "Poser des cartes de sa main face à une attaque." },
  cout: { mot: "coût", icone: "r", def: "Les ressources à payer pour jouer la carte, en haut à droite." },
  ressource: { mot: "ressource", icone: "r", def: "Ce qui sert à payer les coûts. Perdues en fin de tour." },
  pitch: { mot: "pitch", icone: "r", def: "Les ressources que donne une carte quand on s'en sert pour payer, en haut à gauche." },
  pitcher: { mot: "pitcher", def: "Poser une carte de sa main dans sa zone de pitch pour gagner ses ressources." },
  couleur: { mot: "couleur", def: "La bande en haut de la carte : rouge, jaune ou bleue." },
  intellect: { mot: "intellect", icone: "i", def: "Le nombre de cartes que tu as en main après avoir pioché." },
  deck: { mot: "deck", def: "Ta pioche, face cachée. Tu ne peux pas la regarder." },
  type: { mot: "ligne de type", def: "La ligne en bas de la carte : classe, type et sous-types." },
  action: { mot: "point d'action", def: "Il en faut un pour jouer une carte d'action. Tu en as 1 au début de ton tour." },
};

const LECONS = [
  {
    no: 1, titre: "Le duel",
    lecons: [
      {
        id: "1-1", titre: "Deux héros face à face",
        plans: [
          { introduit: ["heros"], dit: "Une partie oppose deux joueurs. Chacun joue un {{heros}} : la grande carte posée devant lui.", cr: ["1.1.2", "4.1.2"],
            fait: [["poser", "adv", { nom: "Bravo, Flattering Showman" }, "adv.heros"], ["poser", "moi", { nom: "Dorinthea" }, "moi.heros"]] },
          { introduit: ["pv"], dit: "Ton héros, Dorinthea, a 20 {{pv}}. C'est le nombre en bas à droite de sa carte.", cr: ["2.5.1", "2.5.2"],
            fait: [["zoom", "moi", "vie", "20 PV"], ["vie", "moi", 20], ["vie", "adv", 20]] },
          { dit: "Quand les {{pv}} d'un héros tombent à 0, son joueur <b>perd la partie</b>. Si les deux tombent à 0 en même temps, c'est une partie nulle.", cr: ["4.5.3a", "4.5.4a"],
            fait: [["vie", "adv", 0, { lent: true }], ["perd", "adv"]] },
          { dit: "Ton but est donc simple : faire tomber les {{pv}} du héros adverse à 0 avant que les tiens n'y tombent.", cr: ["4.5.2a"],
            fait: [["releve", "adv"], ["vie", "adv", 20]] },
          { quiz: { q: "Quand perds-tu la partie ?", choix: ["Quand les PV de mon héros tombent à 0", "Quand je n'ai plus de cartes en main", "Quand mon adversaire a plus de PV que moi"], bonne: 0,
            pourquoi: "On perd quand les PV de son héros sont réduits à 0. Une main vide ne fait pas perdre." }, cr: ["4.5.3a"] },
        ],
      },
      {
        id: "1-2", titre: "Attaquer",
        plans: [
          { fait: [["poser", "adv", { nom: "Bravo, Flattering Showman" }, "adv.heros", { vite: true }], ["poser", "moi", { nom: "Dorinthea" }, "moi.heros", { vite: true }], ["vie", "moi", 20], ["vie", "adv", 20],
              ["poser", "a1", { nom: "Raging Onslaught", couleur: "rouge" }, "moi.main.0"], ["poser", "a2", { nom: "Brutal Assault", couleur: "rouge" }, "moi.main.1"], ["poser", "a3", { nom: "Brutal Assault", couleur: "jaune" }, "moi.main.2"]],
            introduit: ["main"], dit: "Voici ta {{main}} : les cartes que tu tiens. Ton adversaire ne les voit pas.", cr: ["3.9.1"] },
          { introduit: ["attaque", "puissance"], dit: "Raging Onslaught est une {{attaque}}. Le nombre en bas à gauche est sa {{puissance}} : 7. C'est la force du coup.", cr: ["2.9.1", "2.9.2"],
            fait: [["zoom", "a1", "puissance", "Puissance 7"]] },
          { dit: "<b>À toi.</b> Clique sur Raging Onslaught pour attaquer Bravo.", clic: "a1",
            apres: [["attaquer", "a1", "adv", 7]],
            introduit: ["degats"], ensuite: "Personne ne s'est défendu : l'attaque touche. Elle inflige autant de {{degats}} que sa {{puissance}}, et Bravo perd 7 {{pv}}.", cr: ["7.5.2"] },
          { dit: "Dans une vraie partie, une attaque se paie avant d'être jouée. Tu l'apprendras au chapitre 3. Pour l'instant, retiens la {{puissance}}.",
            fait: [["cimetiere", "a1"]] },
          { quiz: { q: "Une attaque de puissance 5 touche un héros qui ne se défend pas. Combien de PV perd-il ?", choix: ["5", "0", "Cela dépend de ses PV"], bonne: 0,
            pourquoi: "Sans défense, les dégâts sont égaux à la puissance." }, cr: ["7.5.2"] },
        ],
      },
      {
        id: "1-3", titre: "Défendre",
        plans: [
          { fait: [["poser", "adv", { nom: "Bravo, Flattering Showman" }, "adv.heros", { vite: true }], ["poser", "moi", { nom: "Dorinthea" }, "moi.heros", { vite: true }], ["vie", "moi", 20], ["vie", "adv", 20],
              ["poser", "b1", { nom: "Brutal Assault", couleur: "bleu" }, "moi.main.0"], ["poser", "b2", { nom: "Brutal Assault", couleur: "bleu" }, "moi.main.1"], ["poser", "b3", { nom: "Raging Onslaught", couleur: "rouge" }, "moi.main.2"],
              ["dos", "x1", "adv.main.0"], ["dos", "x2", "adv.main.1"], ["dos", "x3", "adv.main.2"],
              ["jouer", "x2", { nom: "Brutal Assault", couleur: "rouge" }, "chaine"]],
            dit: "C'est le tour de Bravo. Il t'attaque avec Brutal Assault : {{puissance}} 6.", cr: ["2.9.2"] },
          { introduit: ["defense"], dit: "Regarde tes cartes : en bas à droite se trouve leur {{defense}}. Ce Brutal Assault bleu a 3 de {{defense}}.", cr: ["2.3.1", "2.3.2"],
            fait: [["zoom", "b1", "defense", "Défense 3"]] },
          { introduit: ["defendre"], dit: "Tu peux {{defendre}} : poser des cartes de ta {{main}} face à l'attaque. <b>À toi :</b> clique sur un Brutal Assault bleu.", cr: ["7.3.2a"], clic: "b1",
            apres: [["defendre", "b1", 0], ["impact", "moi", 3]],
            ensuite: "6 − 3 = 3 : tu ne perds que 3 {{pv}} au lieu de 6. Défendre n'est pas jouer une carte : tu ne paies rien.", cr: ["7.5.2", "7.3.2"] },
          { dit: "Les {{defense|défenses}} s'additionnent. Avec deux cartes de {{defense}} 3, la {{defense}} totale est 6 : l'attaque ne touche pas.", cr: ["7.5.2"],
            fait: [["retour", "b1", "moi.main.0"], ["vie", "moi", 20], ["defendre", "b1", 0], ["defendre", "b2", 1], ["bouclier"]] },
          { dit: "Mais chaque carte qui défend quitte ta {{main}} : tu en auras moins pour attaquer à ton tour. Défendre est toujours un choix.",
            fait: [["cimetiere", "b1"], ["cimetiere", "b2"], ["cimetiere", "x2"]] },
          { quiz: { q: "Une attaque de puissance 7 est défendue par une carte de défense 3. Combien de PV perds-tu ?", choix: ["4", "7", "0"], bonne: 0,
            pourquoi: "7 − 3 = 4." }, cr: ["7.5.2"] },
        ],
      },
    ],
  },
  {
    no: 2, titre: "Lire une carte",
    lecons: [
      {
        id: "2-1", titre: "Les quatre coins",
        plans: [
          { fait: [["poser", "c", { nom: "Raging Onslaught", couleur: "rouge" }, "centre"]],
            dit: "Reprenons Raging Onslaught. Chaque coin de la carte donne une information, toujours au même endroit." },
          { dit: "En bas à gauche, tu connais déjà la {{puissance}} : 7. En bas à droite, la {{defense}} : 3.", cr: ["2.9.2", "2.3.2"],
            fait: [["zoom", "c", "puissance", "Puissance 7"], ["zoom", "c", "defense", "Défense 3"]] },
          { introduit: ["cout", "ressource"], dit: "En haut à droite, le {{cout}} : 3. Pour jouer la carte, il faut payer 3 {{ressource|ressources}}.", cr: ["2.2.1", "2.2.2", "1.13.3"],
            fait: [["zoom", "c", "cout", "Coût 3"]] },
          { introduit: ["pitch"], dit: "En haut à gauche, le {{pitch}} : les {{ressource|ressources}} que la carte te donne si tu t'en sers pour payer une autre carte. Ici 1.", cr: ["2.8.1", "2.8.2"],
            fait: [["zoom", "c", "pitch", "Pitch 1"]] },
          { quiz: { q: "Raging Onslaught coûte 3 et a 7 de puissance. Combien de ressources faut-il pour la jouer ?", choix: ["3", "7", "1"], bonne: 0,
            pourquoi: "Le coût, en haut à droite, dit ce qu'il faut payer : 3." }, cr: ["2.2.1"] },
        ],
      },
      {
        id: "2-2", titre: "Trois couleurs",
        plans: [
          { fait: [["poser", "r", { nom: "Brutal Assault", couleur: "rouge" }, "trio.0"], ["poser", "j", { nom: "Brutal Assault", couleur: "jaune" }, "trio.1"], ["poser", "b", { nom: "Brutal Assault", couleur: "bleu" }, "trio.2"]],
            introduit: ["couleur"], dit: "Beaucoup de cartes existent en trois {{couleur|couleurs}}. La {{couleur}} est la bande en haut de la carte.", cr: ["2.1.2"] },
          { dit: "En général, une carte rouge a un {{pitch}} de 1, une jaune de 2, une bleue de 3. C'est le {{pitch}} imprimé qui compte.", cr: ["2.1.2a", "2.8.2"],
            fait: [["bulle", "r", "pitch", "1"], ["bulle", "j", "pitch", "2"], ["bulle", "b", "pitch", "3"]] },
          { dit: "En échange, la {{puissance}} baisse : 6 en rouge, 5 en jaune, 4 en bleu. Une carte forte paie mal, une carte qui paie bien frappe moins fort.",
            fait: [["bulle", "r", "puissance", "6"], ["bulle", "j", "puissance", "5"], ["bulle", "b", "puissance", "4"]] },
          { dit: "Pour le jeu, Brutal Assault rouge et Brutal Assault bleu sont deux cartes différentes : le nom et le {{pitch}} ensemble font l'identité d'une carte.", cr: ["2.7.1", "2.8.1"] },
          { quiz: { q: "Une carte bleue donne en général combien de ressources quand on la pitche ?", choix: ["3", "1", "2"], bonne: 0,
            pourquoi: "Rouge 1, jaune 2, bleu 3, en général." }, cr: ["2.1.2a"] },
        ],
      },
      {
        id: "2-3", titre: "La carte du héros",
        plans: [
          { fait: [["poser", "moi", { nom: "Dorinthea" }, "centre"]],
            dit: "La carte du {{heros}} se lit aussi par ses coins. En bas à droite, tu connais ses {{pv}}." , cr: ["2.5.2"] },
          { introduit: ["intellect"], dit: "En bas à gauche, son {{intellect}} : 4. C'est le nombre de cartes que tu as en {{main}} après avoir pioché.", cr: ["2.4.1"],
            fait: [["zoom", "moi", "intellect", "Intellect 4"]] },
          { introduit: ["deck"], dit: "Tes autres cartes forment ton {{deck}}, face cachée. Au début de la partie, chaque joueur pioche jusqu'à son {{intellect}}.", cr: ["3.7.1", "3.7.4", "4.1.10"],
            fait: [["deplacer", "moi", "moi.heros"], ["deck", "moi"], ["piocher", 4]] },
          { dit: "À la fin de <b>ton</b> tour, tu repioches jusqu'à ton {{intellect}}. Ton adversaire ne repioche pas à la fin de ton tour, sauf au tout premier tour de la partie.", cr: ["4.4.3f"] },
          { quiz: { q: "Intellect 4. À la fin de ton tour, il te reste 1 carte en main. Combien en pioches-tu ?", choix: ["3", "4", "1"], bonne: 0,
            pourquoi: "Tu pioches jusqu'à avoir 4 cartes : 4 − 1 = 3." }, cr: ["4.4.3f"] },
        ],
      },
      {
        id: "2-4", titre: "Type et texte",
        plans: [
          { fait: [["poser", "s", { nom: "Steelblade Shunt", couleur: "rouge" }, "centre"]],
            introduit: ["type"], dit: "En bas de la carte, la {{type}} : « Warrior Defense Reaction ». Sur une carte française : « Guerrier Réaction de défense ».", cr: ["2.14.1"] },
          { fait: [["zoom", "s", "type", "Warrior Defense Reaction"]],
            dit: "Avant le type viennent la <b>classe</b> (Warrior, Guardian, Ninja…) et parfois un <b>talent</b> (Draconic, Ice…). Ton {{heros}} doit les avoir pour que la carte entre dans ton {{deck}}. « Generic » veut dire : aucune classe, tout héros peut la jouer.", cr: ["2.11.6", "1.1.3", "2.14.1a"] },
          { dit: "Vient ensuite le <b>type</b>, qui dit quand la carte se joue. Les chapitres suivants les présentent un par un.", cr: ["2.15.1"] },
          { fait: [["zoom", "s", "texte", "Texte de règles"]],
            dit: "Au milieu, le texte. Seul le texte de règles agit sur la partie. Le texte en italique entre parenthèses (rappel) ou sous un trait (ambiance) n'a aucun effet.", cr: ["2.12.2", "2.12.3"] },
          { quiz: { q: "Steelblade Shunt est une « Warrior Defense Reaction ». Quelle est sa classe ?", choix: ["Warrior", "Defense Reaction", "Elle n'en a pas"], bonne: 0,
            pourquoi: "La classe vient avant le type." }, cr: ["2.14.1"] },
        ],
      },
    ],
  },
  {
    no: 3, titre: "Payer",
    lecons: [
      {
        id: "3-1", titre: "Pitcher pour payer",
        plans: [
          { fait: [["poser", "adv", { nom: "Bravo, Flattering Showman" }, "adv.heros", { vite: true }], ["poser", "moi", { nom: "Dorinthea" }, "moi.heros", { vite: true }], ["vie", "moi", 20], ["vie", "adv", 20],
              ["poser", "r", { nom: "Raging Onslaught", couleur: "rouge" }, "moi.main.0"], ["poser", "b", { nom: "Brutal Assault", couleur: "bleu" }, "moi.main.1"], ["poser", "j", { nom: "Brutal Assault", couleur: "jaune" }, "moi.main.2"]],
            dit: "Tu veux jouer Raging Onslaught. Son {{cout}} est 3 : il te faut 3 {{ressource|ressources}}.", cr: ["2.2.1"] },
          { introduit: ["pitcher"], dit: "Pour en avoir, tu vas {{pitcher}} une carte : la poser dans ta zone de pitch et gagner son {{pitch}}. <b>À toi :</b> clique sur le Brutal Assault bleu ({{pitch}} 3).", cr: ["1.14.2d", "1.14.3"], clic: "b",
            apres: [["pitcher", "b", 3], ["payer", 3], ["jouer", "r", null, "chaine"], ["attaquer", "r", "adv", 7]],
            ensuite: "Le bleu donne 3 {{ressource|ressources}}, qui paient le {{cout}}. Raging Onslaught attaque : Bravo perd 7 {{pv}}." },
          { introduit: ["action"], dit: "Jouer une attaque coûte aussi 1 {{action}}. Tu en as 1 au début de ton tour : sans aide, une seule attaque par tour.", cr: ["8.1.1c", "4.3.2"] },
          { dit: "Les {{ressource|ressources}} non dépensées sont perdues à la fin du tour. Les cartes pitchées, elles, vont sous ton {{deck}} : elles reviendront plus tard.", cr: ["4.4.3e", "4.4.3c"] },
          { quiz: { q: "Peux-tu pitcher une carte sans avoir de coût à payer ?", choix: ["Non", "Oui, à tout moment"], bonne: 0,
            pourquoi: "On ne pitche que pour payer un coût, ou si un effet l'ordonne." }, cr: ["1.14.3b"] },
        ],
      },
      {
        id: "3-2", titre: "À toi de jouer",
        plans: [
          { dit: "Maintenant, une vraie partie sur le moteur d'Arsenal. Tu as Raging Onslaught ({{cout}} 3) et trois Brutal Assault : rouge, jaune et bleu.",
            scene: { id: "payer",
              consigne: "Joue Raging Onslaught, puis paie-la sans utiliser la carte bleue : il te faut 3 {{ressource|ressources}}.",
              reussi: (e) => e.bot.vie < e.debut.bot.vie,
              bravo: (e) => e.vous.pitch.length >= 2 ? "Jaune (2) + rouge (1) = 3. Tu gardes la bleue." : "La bleue a payé seule : 3 d'un coup. Ça marche aussi !" } },
          { quiz: { q: "Tu pitches une bleue (pitch 3) pour une carte de coût 2. Que devient la ressource restante ?", choix: ["Elle reste jusqu'à la fin du tour", "Elle est perdue tout de suite", "Le paiement est refusé"], bonne: 0,
            pourquoi: "Le coût est retiré de tes ressources ; le reste est gardé pour ce tour, puis perdu en fin de tour." }, cr: ["1.14.2", "4.4.3e"] },
        ],
      },
    ],
  },
  { no: 4, titre: "Le tour", lecons: [] },
  { no: 5, titre: "Le tapis et les zones", lecons: [] },
  { no: 6, titre: "La chaîne de combat", lecons: [] },
  { no: 7, titre: "Réactions et priorité", lecons: [] },
  { no: 8, titre: "Armes et équipements", lecons: [] },
  { no: 9, titre: "Les effets", lecons: [] },
  { no: 10, titre: "Les mots-clés", lecons: [] },
  { no: 11, titre: "Arcane, jetons, cas limites", lecons: [] },
  { no: 12, titre: "Le format Silver Age", lecons: [] },
];

// Les sections des Comprehensive Rules (titres officiels, en anglais) : la
// couverture montre, pour chacune, les leçons qui la citent.
const CR_SECTIONS = [
  ["1.0", "General"], ["1.1", "Players"], ["1.2", "Objects"], ["1.3", "Cards"], ["1.4", "Attacks"], ["1.5", "Macros"], ["1.6", "Layers"], ["1.7", "Abilities"], ["1.8", "Effects"], ["1.9", "Events"], ["1.10", "Game State"], ["1.11", "Priority"], ["1.12", "Numbers and Symbols"], ["1.13", "Assets"], ["1.14", "Costs"], ["1.15", "Counters"],
  ["2.0", "General"], ["2.1", "Color"], ["2.2", "Cost"], ["2.3", "Defense"], ["2.4", "Intellect"], ["2.5", "Life"], ["2.6", "Metatype"], ["2.7", "Name"], ["2.8", "Pitch"], ["2.9", "Power"], ["2.10", "Subtypes"], ["2.11", "Supertypes"], ["2.12", "Text Box"], ["2.13", "Traits"], ["2.14", "Type Box"], ["2.15", "Types"],
  ["3.0", "General"], ["3.1", "Arena"], ["3.2", "Arms"], ["3.3", "Arsenal"], ["3.4", "Banished"], ["3.5", "Chest"], ["3.6", "Combat Chain"], ["3.7", "Deck"], ["3.8", "Graveyard"], ["3.9", "Hand"], ["3.10", "Head"], ["3.11", "Hero"], ["3.12", "Legs"], ["3.13", "Permanent"], ["3.14", "Pitch"], ["3.15", "Stack"], ["3.16", "Weapon"],
  ["4.0", "General"], ["4.1", "Starting a Game"], ["4.2", "Start Phase"], ["4.3", "Action Phase"], ["4.4", "End Phase"], ["4.5", "Ending a Game"],
  ["5.0", "General"], ["5.1", "Playing Cards"], ["5.2", "Activated Abilities"], ["5.3", "Resolution Abilities & Resolving Layers"], ["5.4", "Static Abilities"],
  ["6.0", "General"], ["6.1", "Discrete Effects"], ["6.2", "Continuous Effects"], ["6.3", "Continuous Effect Interactions"], ["6.4", "Replacement Effects"], ["6.5", "Replacement Effect Interactions"], ["6.6", "Triggered Effects"],
  ["7.0", "General"], ["7.1", "Layer Step"], ["7.2", "Attack Step"], ["7.3", "Defend Step"], ["7.4", "Reaction Step"], ["7.5", "Damage Step"], ["7.6", "Resolution Step"], ["7.7", "Close Step"],
  ["8.0", "General"], ["8.1", "Type Keywords"], ["8.2", "Subtype Keywords"], ["8.3", "Ability Keywords"], ["8.4", "Label Keywords"], ["8.5", "Effect Keywords"], ["8.6", "Token Keywords"],
  ["9.0", "General"], ["9.1", "Double-Faced Cards"], ["9.2", "Split-Cards"], ["9.3", "Marked"], ["9.4", "Living Legend"],
];
