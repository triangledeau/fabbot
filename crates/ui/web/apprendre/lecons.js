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
          { dit: "Dans une vraie partie, jouer une attaque a un prix : il faut la payer. Tu apprendras comment au chapitre 3. Pour l'instant, retiens la {{puissance}}.",
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
  {
    no: 4, titre: "Le tapis et les zones",
    // Reprise de la première version (texte et scènes), en attendant sa refonte animée.
    lecons: [
      {
        id: "4-1", titre: "Les zones de jeu",
        plans: [
          { dit: "Pendant la partie, chaque carte se trouve dans une <b>zone</b>. Chaque joueur a les siennes : héros, deux zones d'arme, tête, torse, bras, jambes, deck, main, arsenal, pitch, cimetière et zone des cartes bannies.", cr: ["3.0.1","3.0.2"] },
          { dit: "Trois zones sont communes aux deux joueurs : la <b>chaîne de combat</b> (les attaques en cours), la <b>pile</b> (les cartes en train d'être jouées) et la zone des <b>permanents</b>.", cr: ["3.0.2","3.6.1","3.13.1","3.15.1"] },
          { dit: "Certaines zones sont <b>publiques</b> : tout le monde voit les cartes. C'est le cas du cimetière, de la zone de pitch et des cartes bannies.", cr: ["3.0.3","3.0.4","3.8.1","3.14.1","3.4.1"] },
          { dit: "D'autres sont <b>privées</b> : la main, le deck et l'arsenal. Tu peux regarder ta main et ton arsenal, mais pas ton propre deck.", cr: ["3.9.1","3.7.1","3.3.1","3.0.3a","3.7.4"] },
          { quiz: { q: "Peux-tu regarder les cartes de ton propre deck pendant la partie ?", choix: ["Non, sauf si une règle ou un effet le permet","Oui, à tout moment","Oui, mais seulement pendant ton tour"], bonne: 0,
            pourquoi: "Le deck est la seule de tes zones privées que tu ne peux pas regarder librement." }, cr: ["3.0.3a","3.7.4"] },
          { quiz: { q: "Ton adversaire voit-il les cartes de ton cimetière ?", choix: ["Oui, c'est une zone publique","Non, c'est une zone privée"], bonne: 0,
            pourquoi: "Le cimetière est public, comme la zone de pitch et les cartes bannies." }, cr: ["3.8.1"] },
        ],
      },
      {
        id: "4-2", titre: "L'arène et l'équipement",
        plans: [
          { dit: "L'<b>arène</b> regroupe le héros, les armes, l'équipement (tête, torse, bras, jambes), la chaîne de combat et les permanents. La main, le deck, l'arsenal, le pitch, le cimetière, les cartes bannies et la pile n'en font pas partie.", cr: ["3.1.1","3.1.1a"] },
          { dit: "Chaque zone d'équipement (tête, torse, bras, jambes) contient au plus une carte. Il y a deux zones d'arme : une arme à deux mains (2H) les occupe toutes les deux.", cr: ["3.2.2","3.16.2","3.16.2a"] },
          { dit: "Armes et équipements sont choisis avant la partie et commencent déjà en place. Ils ne vont jamais dans le deck.", cr: ["3.2.3","3.16.3","4.1.4a","4.1.6a"] },
          { quiz: { q: "Combien de cartes d'équipement peux-tu avoir sur la tête ?", choix: ["Une au plus","Deux","Autant que tu veux"], bonne: 0,
            pourquoi: "Chaque zone d'équipement ne contient qu'une carte au plus." }, cr: ["4.1.4a"] },
        ],
      },
      {
        id: "4-3", titre: "L'arsenal",
        plans: [
          { dit: "L'<b>arsenal</b> est une zone privée qui garde <b>une seule carte</b> de ton deck, face cachée, pour plus tard.", cr: ["3.3.1","3.3.2"] },
          { dit: "À la fin de ton tour, si ton arsenal est vide, tu peux y mettre une carte de ta main. Tu pourras la jouer depuis l'arsenal comme depuis ta main.", cr: ["4.4.3b","3.3.4","5.1.1a"] },
          { scene: { id: "attaquer",
              consigne: "Joue Raging Onslaught en payant avec une carte bleue, comme au chapitre 3. Puis, quand le jeu te le propose en fin de tour, <b>mets une carte en arsenal</b>.",
              reussi: (e) => !!e.vous.arsenal,
              bravo: "Ta carte est en arsenal : elle ne compte plus dans ta main, et tu pourras la jouer à un prochain tour. Ton adversaire ne la voit pas." } },
          { quiz: { q: "Ton arsenal contient déjà une carte. Peux-tu en ajouter une autre à la fin de ton tour ?", choix: ["Non, il faut qu'il soit vide","Oui, jusqu'à deux"], bonne: 0,
            pourquoi: "L'arsenal ne contient qu'une carte, et on n'y met une carte en fin de tour que s'il est vide." }, cr: ["3.3.2","4.4.3b"] },
        ],
      },
      {
        id: "4-4", titre: "Où vont les cartes jouées",
        plans: [
          { dit: "Une carte que tu pitches pour payer va dans ta <b>zone de pitch</b>. À la fin du tour, chaque joueur remet ses cartes pitchées <b>sous son deck</b>, dans l'ordre de son choix, sans le montrer.", cr: ["1.14.3","4.4.3c"] },
          { dit: "Une carte d'action sans attaque va au <b>cimetière</b> une fois résolue. Une attaque reste sur la chaîne de combat jusqu'à la fin du combat, puis va au cimetière, avec les cartes de main qui l'ont défendue. Un équipement qui a défendu retourne, lui, à sa place.", cr: ["5.3.7","3.0.12","7.7.5","7.7.6"] },
          { quiz: { q: "Que deviennent tes cartes pitchées à la fin du tour ?", choix: ["Elles vont sous ton deck","Elles vont au cimetière","Elles retournent dans ta main"], bonne: 0,
            pourquoi: "Les cartes de la zone de pitch vont sous le deck, dans l'ordre choisi par leur propriétaire." }, cr: ["4.4.3c"] },
        ],
      },
    ],
  },
  {
    no: 5, titre: "Le tour",
    // Reprise de la première version (texte et scènes), en attendant sa refonte animée.
    lecons: [
      {
        id: "5-1", titre: "Les trois phases",
        plans: [
          { dit: "Les joueurs jouent chacun leur tour. Un tour a trois phases, dans l'ordre : <b>début</b>, <b>action</b>, <b>fin</b>.", cr: ["4.0.3a","4.0.3b"] },
          { dit: "<b>Début</b> : les effets « au début du tour » se déclenchent. Personne ne joue de carte pendant cette phase.", cr: ["4.2.1","4.2.2"] },
          { dit: "<b>Action</b> : le joueur du tour reçoit 1 point d'action et peut jouer. Elle se termine quand plus rien n'est en cours et que les deux joueurs passent l'un après l'autre.", cr: ["4.3.2","4.3.3","4.3.4"] },
          { dit: "<b>Fin</b> : arsenal (si vide), cartes pitchées sous le deck, perte des points d'action et des ressources restants, puis le joueur du tour repioche jusqu'à son intellect. Le tour passe à l'adversaire.", cr: ["4.4.3b","4.4.3c","4.4.3e","4.4.3f","4.4.4"] },
          { quiz: { q: "À la fin de ton tour, il te reste 2 ressources non utilisées. Que deviennent-elles ?", choix: ["Elles sont perdues","Tu les gardes pour le tour suivant","Elles deviennent des PV"], bonne: 0,
            pourquoi: "Tous les joueurs perdent leurs points d'action et leurs ressources en fin de tour." }, cr: ["4.4.3e"] },
        ],
      },
      {
        id: "5-2", titre: "Les points d'action",
        plans: [
          { dit: "Jouer une carte d'<b>action</b> (comme une attaque) coûte, en plus de son coût, <b>1 point d'action</b>. Au début de ta phase d'action, tu n'en as qu'un.", cr: ["8.1.1c","4.3.2"] },
          { scene: { id: "un-point",
              consigne: "Essaie de jouer tes <b>deux</b> Brutal Assault rouges. Paie chacune avec une carte bleue.",
              reussi: (e) => e.bot.vie < e.debut.bot.vie && !(e.decision?.options || []).some((o) => o.action === "jouer"),
              bravo: "La première attaque a touché, mais tu n'as plus de point d'action : la seconde ne peut plus se jouer ce tour-ci." } },
          { dit: "Certaines cartes ont <b>go again</b> (« rejouer ») : une fois résolues, elles te rendent 1 point d'action. Pour une attaque, il revient au début de l'étape de résolution du combat. C'est ainsi qu'on enchaîne plusieurs actions dans un tour.", cr: ["5.3.5","8.3.5a","8.3.5b"] },
          { dit: "Une action se joue seulement quand la pile est vide, et pas pendant un combat, sauf à son étape de résolution.", cr: ["8.1.1a","8.1.1b"] },
          { quiz: { q: "Tu as 1 point d'action et deux attaques sans go again. Combien peux-tu en jouer ce tour-ci ?", choix: ["Une","Deux","Aucune"], bonne: 0,
            pourquoi: "La première coûte ton seul point d'action et ne le rend pas : la seconde ne peut plus se jouer." }, cr: ["8.1.1c"] },
        ],
      },
      {
        id: "5-3", titre: "Le premier tour",
        plans: [
          { dit: "Avant la partie, un joueur choisit qui joue le premier tour. Pour la première partie d'un match, ce joueur est tiré au sort. Ensuite, c'est celui qui a perdu la partie précédente (en cas de nulle, le même que la fois d'avant).", cr: ["4.1.3"] },
          { dit: "Chaque joueur commence avec une main égale à l'intellect de son héros.", cr: ["4.1.10"] },
          { dit: "À la fin du tout premier tour, <b>tous</b> les joueurs repiochent jusqu'à leur intellect, pas seulement le joueur du tour.", cr: ["4.4.3f"] },
          { quiz: { q: "Ton adversaire commence et tu défends avec 2 cartes pendant son premier tour. À la fin de ce tour, repioches-tu ?", choix: ["Oui, jusqu'à ton intellect","Non, seul le joueur du tour repioche"], bonne: 0,
            pourquoi: "Au premier tour de la partie, tous les joueurs repiochent jusqu'à leur intellect." }, cr: ["4.4.3f"] },
        ],
      },
      {
        id: "5-4", titre: "Une carte, trois usages",
        plans: [
          { dit: "Chaque carte de ta main peut servir de trois façons : la <b>jouer</b> (attaquer, par exemple), la <b>pitcher</b> pour payer une autre carte, ou <b>défendre</b> avec, pendant le tour adverse.", cr: ["5.1.1a","1.14.3","7.3.2a"] },
          { dit: "Défendre avec une carte n'est pas la jouer : on ne paie pas son coût.", cr: ["7.3.2"] },
          { dit: "Toute la difficulté du jeu est là : une carte pitchée ne défendra pas, une carte gardée pour défendre ne paiera pas. Le chapitre 6 revient sur la défense." },
          { quiz: { q: "Tu défends avec une carte de coût 3. Combien de ressources paies-tu ?", choix: ["0","3"], bonne: 0,
            pourquoi: "Déclarer une carte en défense n'est pas la jouer : son coût n'est pas payé." }, cr: ["7.3.2"] },
        ],
      },
      {
        id: "5-5", titre: "Les cartes pitchées reviennent",
        plans: [
          { dit: "À la fin du tour, les cartes de ta zone de pitch vont <b>sous ton deck</b>. Si tu en as plusieurs, tu choisis leur ordre, et ton adversaire ne le connaît pas.", cr: ["4.4.3c","8.5.15c"] },
          { dit: "Une carte pitchée n'est donc pas perdue : elle reviendra quand tu auras pioché tout le reste de ton deck. Une carte qui va au cimetière n'en sort que si un effet le permet." },
          { quiz: { q: "Qui voit l'ordre dans lequel tu ranges tes cartes pitchées sous ton deck ?", choix: ["Personne d'autre que toi","Ton adversaire aussi"], bonne: 0,
            pourquoi: "Cet ordre est une information cachée." }, cr: ["4.4.3c"] },
        ],
      },
    ],
  },
  {
    no: 6, titre: "La chaîne de combat",
    // Reprise de la première version (texte et scènes), en attendant sa refonte animée.
    lecons: [
      {
        id: "6-1", titre: "Les étapes du combat",
        plans: [
          { dit: "Quand tu joues une attaque, elle va d'abord sur la <b>pile</b>, comme toute carte jouée. Quand elle se résout, le <b>combat</b> commence : l'attaque devient un <b>maillon</b> de la <b>chaîne de combat</b>.", cr: ["5.1.2","7.0.2","7.0.3","7.2.2"] },
          { dit: "Un maillon passe par des étapes, dans l'ordre : <b>Couche</b> (Layer), <b>Attaque</b>, <b>Défense</b>, <b>Réaction</b>, <b>Dégâts</b>, <b>Résolution</b>. Puis vient la <b>Fermeture</b> (Close) quand le combat se termine.", cr: ["7.0.1","7.7.1"] },
          { dit: "<b>Attaque</b> : l'attaque est posée sur la chaîne, les effets « quand elle attaque » se déclenchent. <b>Défense</b> : le défenseur choisit ses cartes de défense. <b>Réaction</b> : chacun peut jouer des réactions.", cr: ["7.2.2","7.2.3","7.3.2","7.4.1"] },
          { dit: "<b>Dégâts</b> : si la puissance dépasse la défense totale, l'attaque <b>touche</b> et inflige la différence. <b>Résolution</b> : le maillon est terminé ; si l'attaque a go again, son joueur gagne 1 point d'action.", cr: ["7.5.2","7.6.2"] },
          { quiz: { q: "Dans quel ordre viennent ces étapes ?", choix: ["Défense, puis Réaction, puis Dégâts","Réaction, puis Défense, puis Dégâts","Dégâts, puis Défense, puis Réaction"], bonne: 0,
            pourquoi: "Couche, Attaque, Défense, Réaction, Dégâts, Résolution, puis Fermeture." }, cr: ["7.0.1"] },
        ],
      },
      {
        id: "6-2", titre: "Qui joue quoi pendant un combat",
        plans: [
          { dit: "Pendant un combat, on ne joue pas de carte d'action, sauf une nouvelle attaque à l'étape de <b>Résolution</b>. Une carte <b>Instant</b>, elle, se joue dès qu'on a la priorité.", cr: ["7.0.1a","7.6.3a","8.1.1b"] },
          { dit: "À l'étape de <b>Réaction</b>, l'attaquant peut jouer des <b>Attack Reactions</b> (réactions d'attaque) et le défenseur des <b>Defense Reactions</b> (réactions de défense).", cr: ["7.4.2a","7.4.2b","8.1.2a","8.1.3a"] },
          { quiz: { q: "Ton adversaire t'attaque. Peux-tu jouer une Attack Reaction ?", choix: ["Non, seul l'attaquant le peut","Oui, à l'étape de Réaction"], bonne: 0,
            pourquoi: "Une Attack Reaction ne se joue que par le joueur qui contrôle l'attaque, à l'étape de Réaction." }, cr: ["8.1.2a"] },
        ],
      },
      {
        id: "6-3", titre: "Enchaîner les attaques",
        plans: [
          { dit: "À l'étape de Résolution, une attaque avec <b>go again</b> te rend 1 point d'action. Tu peux alors jouer une autre attaque : elle forme un nouveau maillon sur la même chaîne de combat.", cr: ["7.6.2","7.6.3a"] },
          { dit: "Quand tout le monde passe à l'étape de Résolution sans rien jouer, la chaîne se <b>ferme</b> : le combat est fini et les cartes de la chaîne vont au cimetière.", cr: ["7.7.2a","7.7.6"] },
          { scene: { id: "go-again",
              consigne: "Tu joues Ira, un héros Ninja. Joue <b>Bittering Thorns</b> (go again), puis, quand il touche, joue <b>Brutal Assault</b> rouge. Paie avec la jaune puis la bleue.",
              reussi: (e) => e.debut.bot.vie - e.bot.vie >= 10,
              bravo: (e) => `Deux attaques dans le même tour : ${e.debut.bot.vie - e.bot.vie} PV en tout. Bittering Thorns a rendu un point d'action (go again). La seconde attaque a eu +1 grâce à Bittering Thorns (« When this hits, your next attack this turn gets +1 ») et +1 grâce à la capacité d'Ira (« Your second attack each turn gets +1 »).` } },
          { quiz: { q: "Ton attaque n'a pas go again et tu n'as plus de point d'action. Peux-tu jouer une nouvelle attaque à l'étape de Résolution ?", choix: ["Non","Oui"], bonne: 0,
            pourquoi: "Une attaque d'action coûte toujours 1 point d'action ; sans go again, tu n'en récupères pas." }, cr: ["8.1.1c","7.6.2"] },
        ],
      },
      {
        id: "6-4", titre: "Bloquer une attaque",
        plans: [
          { dit: "Quand ton héros est attaqué, à l'étape de Défense, tu peux déclarer autant de cartes de ta <b>main</b> que tu veux comme défense (et ton équipement, voir chapitre 8).", cr: ["7.3.2a"] },
          { dit: "Défendre n'est pas jouer : tu ne paies rien. Une carte sans valeur de défense ne peut pas défendre.", cr: ["7.3.2","7.3.2b"] },
          { dit: "Les défenses s'additionnent. Les dégâts sont égaux à la puissance moins la défense totale. Si la défense est égale ou plus grande, l'attaque ne touche pas.", cr: ["7.5.2"] },
          { scene: { id: "defendre",
              consigne: "Ton adversaire attaque avec <b>Raging Onslaught</b> (puissance 7). Tes cartes ont 3 de défense chacune. Choisis avec combien de cartes tu défends, puis termine ta déclaration.",
              reussi: (e) => e.vous.cimetiere.length > 0 || e.vous.vie < e.debut.vous.vie,
              bravo: (e) => { const n = e.vous.cimetiere.length, p = e.debut.vous.vie - e.vous.vie; const d = 3 * n; return n === 0 ? `Sans défense, l'attaque touche pour 7 : tu perds ${p} PV.` : d >= 7 ? `Ta défense totale (${d}) atteint ou dépasse la puissance (7) : l'attaque ne touche pas, tu ne perds aucun PV. Mais ${n} cartes ont quitté ta main.` : `Tu as défendu avec ${n} carte${n > 1 ? "s" : ""} (défense ${d}) : 7 − ${d} = ${7 - d}. Tu perds ${p} PV.`; } } },
          { quiz: { q: "Une attaque de puissance 6 est défendue par deux cartes de défense 3. Combien de dégâts ?", choix: ["0","3","6"], bonne: 0,
            pourquoi: "6 − (3 + 3) = 0 : la défense n'est pas dépassée, l'attaque ne touche pas." }, cr: ["7.5.2"] },
        ],
      },
      {
        id: "6-5", titre: "Défendre ou garder ses cartes",
        plans: [
          { dit: "À la fin d'un tour, seul le joueur du tour repioche (sauf au premier tour). Une carte que tu utilises pour défendre pendant le tour adverse ne sera donc pas remplacée avant la fin de <b>ton</b> tour.", cr: ["4.4.3f"] },
          { dit: "Défendre protège tes PV, mais te laisse moins de cartes pour attaquer et payer à ton tour. Chaque défense est un choix." },
          { quiz: { q: "Tu as 4 cartes, intellect 4. Pendant le tour adverse (pas le premier), tu défends avec 2 cartes. Combien de cartes as-tu au début de ton tour ?", choix: ["2","4"], bonne: 0,
            pourquoi: "Ton adversaire repioche à la fin de son tour, pas toi : tu commences ton tour avec 2 cartes." }, cr: ["4.4.3f"] },
        ],
      },
    ],
  },
  {
    no: 7, titre: "Réactions et priorité",
    // Reprise de la première version (texte et scènes), en attendant sa refonte animée.
    lecons: [
      {
        id: "7-1", titre: "Les réactions de défense",
        plans: [
          { fait: [["vider"], ["poser", "c0", { nom: "Steelblade Shunt", couleur: "rouge" }, "centre"]],
            dit: "Une <b>Defense Reaction</b> ne se déclare pas à l'étape de Défense : elle se <b>joue</b> (on paie son coût) à l'étape de Réaction, par le joueur dont le héros est attaqué.", cr: ["7.3.2a","8.1.3a","7.4.2b"] },
          { dit: "Quand elle se résout, elle devient une carte de défense de l'attaque en cours, et sa défense s'ajoute.", cr: ["8.1.3b","7.4.2d"] },
          { quiz: { q: "Pour défendre avec une Defense Reaction, paie-t-on son coût ?", choix: ["Oui, car elle est jouée","Non, comme une carte déclarée en défense"], bonne: 0,
            pourquoi: "Une Defense Reaction est jouée à l'étape de Réaction : son coût est payé. Seules les cartes déclarées à l'étape de Défense ne coûtent rien." }, cr: ["8.1.3a","7.3.2"] },
        ],
      },
    ],
  },
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
