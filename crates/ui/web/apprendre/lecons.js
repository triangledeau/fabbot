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
  {
    no: 3, titre: "Le tapis et les zones",
    lecons: [
      {
        id: "3-1", titre: "Les zones de jeu",
        etapes: [
          { texte: "Pendant la partie, chaque carte se trouve dans une <b>zone</b>. Chaque joueur a les siennes : héros, deux zones d'arme, tête, torse, bras, jambes, deck, main, arsenal, pitch, cimetière et zone des cartes bannies.", cr: ["3.0.1", "3.0.2"] },
          { texte: "Trois zones sont communes aux deux joueurs : la <b>chaîne de combat</b> (les attaques en cours), la <b>pile</b> (les cartes en train d'être jouées) et la zone des <b>permanents</b>.", cr: ["3.0.2", "3.6.1", "3.13.1", "3.15.1"] },
          { texte: "Certaines zones sont <b>publiques</b> : tout le monde voit les cartes. C'est le cas du cimetière, de la zone de pitch et des cartes bannies.", cr: ["3.0.3", "3.0.4", "3.8.1", "3.14.1", "3.4.1"] },
          { texte: "D'autres sont <b>privées</b> : la main, le deck et l'arsenal. Tu peux regarder ta main et ton arsenal, mais pas ton propre deck.", cr: ["3.9.1", "3.7.1", "3.3.1", "3.0.3a", "3.7.4"] },
          { quiz: "Peux-tu regarder les cartes de ton propre deck pendant la partie ?", choix: ["Non, sauf si une règle ou un effet le permet", "Oui, à tout moment", "Oui, mais seulement pendant ton tour"], bonne: 0,
            pourquoi: "Le deck est la seule de tes zones privées que tu ne peux pas regarder librement.", cr: ["3.0.3a", "3.7.4"] },
          { quiz: "Ton adversaire voit-il les cartes de ton cimetière ?", choix: ["Oui, c'est une zone publique", "Non, c'est une zone privée"], bonne: 0,
            pourquoi: "Le cimetière est public, comme la zone de pitch et les cartes bannies.", cr: ["3.8.1"] },
        ],
      },
      {
        id: "3-2", titre: "L'arène et l'équipement",
        etapes: [
          { texte: "L'<b>arène</b> regroupe le héros, les armes, l'équipement (tête, torse, bras, jambes), la chaîne de combat et les permanents. La main, le deck, l'arsenal, le pitch, le cimetière, les cartes bannies et la pile n'en font pas partie.", cr: ["3.1.1", "3.1.1a"] },
          { texte: "Chaque zone d'équipement (tête, torse, bras, jambes) contient au plus une carte. Il y a deux zones d'arme : une arme à deux mains (2H) les occupe toutes les deux.", cr: ["3.2.2", "3.16.2", "3.16.2a"] },
          { texte: "Armes et équipements sont choisis avant la partie et commencent déjà en place. Ils ne vont jamais dans le deck.", cr: ["3.2.3", "3.16.3", "4.1.4a", "4.1.6a"] },
          { quiz: "Combien de cartes d'équipement peux-tu avoir sur la tête ?", choix: ["Une au plus", "Deux", "Autant que tu veux"], bonne: 0,
            pourquoi: "Chaque zone d'équipement ne contient qu'une carte au plus.", cr: ["4.1.4a"] },
        ],
      },
      {
        id: "3-3", titre: "L'arsenal",
        etapes: [
          { texte: "L'<b>arsenal</b> est une zone privée qui garde <b>une seule carte</b> de ton deck, face cachée, pour plus tard.", cr: ["3.3.1", "3.3.2"] },
          { texte: "À la fin de ton tour, si ton arsenal est vide, tu peux y mettre une carte de ta main. Tu pourras la jouer depuis l'arsenal comme depuis ta main.", cr: ["4.4.3b", "3.3.4", "5.1.1a"] },
          { scene: "attaquer",
            consigne: "Joue Raging Onslaught en payant avec une carte bleue, comme au chapitre 1. Puis, quand le jeu te le propose en fin de tour, <b>mets une carte en arsenal</b>.",
            reussi: (e) => !!e.vous.arsenal,
            bravo: "Ta carte est en arsenal : elle ne compte plus dans ta main, et tu pourras la jouer à un prochain tour. Ton adversaire ne la voit pas." },
          { quiz: "Ton arsenal contient déjà une carte. Peux-tu en ajouter une autre à la fin de ton tour ?", choix: ["Non, il faut qu'il soit vide", "Oui, jusqu'à deux"], bonne: 0,
            pourquoi: "L'arsenal ne contient qu'une carte, et on n'y met une carte en fin de tour que s'il est vide.", cr: ["3.3.2", "4.4.3b"] },
        ],
      },
      {
        id: "3-4", titre: "Où vont les cartes jouées",
        etapes: [
          { texte: "Une carte que tu pitches pour payer va dans ta <b>zone de pitch</b>. À la fin du tour, chaque joueur remet ses cartes pitchées <b>sous son deck</b>, dans l'ordre de son choix, sans le montrer.", cr: ["1.14.3", "4.4.3c"] },
          { texte: "Une carte d'action sans attaque va au <b>cimetière</b> une fois résolue. Une attaque reste sur la chaîne de combat jusqu'à la fin du combat, puis va au cimetière, avec les cartes de main qui l'ont défendue. Un équipement qui a défendu retourne, lui, à sa place.", cr: ["5.3.7", "3.0.12", "7.7.5", "7.7.6"] },
          { quiz: "Que deviennent tes cartes pitchées à la fin du tour ?", choix: ["Elles vont sous ton deck", "Elles vont au cimetière", "Elles retournent dans ta main"], bonne: 0,
            pourquoi: "Les cartes de la zone de pitch vont sous le deck, dans l'ordre choisi par leur propriétaire.", cr: ["4.4.3c"] },
        ],
      },
    ],
  },
  {
    no: 4, titre: "Le déroulement d'un tour",
    lecons: [
      {
        id: "4-1", titre: "Les trois phases",
        etapes: [
          { texte: "Les joueurs jouent chacun leur tour. Un tour a trois phases, dans l'ordre : <b>début</b>, <b>action</b>, <b>fin</b>.", cr: ["4.0.3a", "4.0.3b"] },
          { texte: "<b>Début</b> : les effets « au début du tour » se déclenchent. Personne ne joue de carte pendant cette phase.", cr: ["4.2.1", "4.2.2"] },
          { texte: "<b>Action</b> : le joueur du tour reçoit 1 point d'action et peut jouer. Elle se termine quand plus rien n'est en cours et que les deux joueurs passent l'un après l'autre.", cr: ["4.3.2", "4.3.3", "4.3.4"] },
          { texte: "<b>Fin</b> : arsenal (si vide), cartes pitchées sous le deck, perte des points d'action et des ressources restants, puis le joueur du tour repioche jusqu'à son intellect. Le tour passe à l'adversaire.", cr: ["4.4.3b", "4.4.3c", "4.4.3e", "4.4.3f", "4.4.4"] },
          { quiz: "À la fin de ton tour, il te reste 2 ressources non utilisées. Que deviennent-elles ?", choix: ["Elles sont perdues", "Tu les gardes pour le tour suivant", "Elles deviennent des PV"], bonne: 0,
            pourquoi: "Tous les joueurs perdent leurs points d'action et leurs ressources en fin de tour.", cr: ["4.4.3e"] },
        ],
      },
      {
        id: "4-2", titre: "Les points d'action",
        etapes: [
          { texte: "Jouer une carte d'<b>action</b> (comme une attaque) coûte, en plus de son coût, <b>1 point d'action</b>. Au début de ta phase d'action, tu n'en as qu'un.", cr: ["8.1.1c", "4.3.2"] },
          { scene: "un-point",
            consigne: "Essaie de jouer tes <b>deux</b> Brutal Assault rouges. Paie chacune avec une carte bleue.",
            reussi: (e) => e.bot.vie < e.debut.bot.vie && !(e.decision?.options || []).some((o) => o.action === "jouer"),
            bravo: "La première attaque a touché, mais tu n'as plus de point d'action : la seconde ne peut plus se jouer ce tour-ci." },
          { texte: "Certaines cartes ont <b>go again</b> (« rejouer ») : une fois résolues, elles te rendent 1 point d'action. Pour une attaque, il revient au début de l'étape de résolution du combat. C'est ainsi qu'on enchaîne plusieurs actions dans un tour.", cr: ["5.3.5", "8.3.5a", "8.3.5b"] },
          { texte: "Une action se joue seulement quand la pile est vide, et pas pendant un combat, sauf à son étape de résolution.", cr: ["8.1.1a", "8.1.1b"] },
          { quiz: "Tu as 1 point d'action et deux attaques sans go again. Combien peux-tu en jouer ce tour-ci ?", choix: ["Une", "Deux", "Aucune"], bonne: 0,
            pourquoi: "La première coûte ton seul point d'action et ne le rend pas : la seconde ne peut plus se jouer.", cr: ["8.1.1c"] },
        ],
      },
      {
        id: "4-3", titre: "Le premier tour",
        etapes: [
          { texte: "Avant la partie, un joueur choisit qui joue le premier tour. Pour la première partie d'un match, ce joueur est tiré au sort. Ensuite, c'est celui qui a perdu la partie précédente (en cas de nulle, le même que la fois d'avant).", cr: ["4.1.3"] },
          { texte: "Chaque joueur commence avec une main égale à l'intellect de son héros.", cr: ["4.1.10"] },
          { texte: "À la fin du tout premier tour, <b>tous</b> les joueurs repiochent jusqu'à leur intellect, pas seulement le joueur du tour.", cr: ["4.4.3f"] },
          { quiz: "Ton adversaire commence et tu défends avec 2 cartes pendant son premier tour. À la fin de ce tour, repioches-tu ?", choix: ["Oui, jusqu'à ton intellect", "Non, seul le joueur du tour repioche"], bonne: 0,
            pourquoi: "Au premier tour de la partie, tous les joueurs repiochent jusqu'à leur intellect.", cr: ["4.4.3f"] },
        ],
      },
    ],
  },
  {
    no: 5, titre: "Payer : coût et pitch",
    lecons: [
      {
        id: "5-1", titre: "Payer en pitchant",
        etapes: [
          { texte: "Pour jouer une carte, tu paies son <b>coût</b> en <b>ressources</b>. Une carte d'action coûte en plus 1 point d'action.", cr: ["2.2.1", "1.13.3", "5.1.6b", "5.1.6c"] },
          { texte: "Pour avoir des ressources, tu <b>pitches</b> des cartes de ta main : tu les poses dans ta zone de pitch et tu gagnes autant de ressources que leur pitch. Tu les pitches une par une, jusqu'à avoir assez.", cr: ["1.14.2d", "1.14.3"] },
          { scene: "payer",
            consigne: "Joue <b>Raging Onslaught</b> (coût 3). Pour payer, essaie cette fois la carte <b>jaune</b> (pitch 2) puis la <b>rouge</b> (pitch 1). Tu peux recommencer pour essayer la bleue seule (pitch 3).",
            reussi: (e) => e.bot.vie < e.debut.bot.vie,
            bravo: (e) => e.vous.pitch.length >= 2 ? "2 + 1 = 3 : le coût est payé avec deux cartes. Tu gardes la bleue en main." : "3 : la bleue paie le coût à elle seule. Tu gardes la jaune et la rouge en main." },
          { texte: "Tu ne peux pitcher une carte que si tu as besoin de ses ressources pour payer, et une carte sans pitch ne peut pas être pitchée.", cr: ["1.14.3a", "1.14.3b"] },
          { quiz: "Tu veux jouer une carte de coût 2. Tu pitches une carte bleue (pitch 3). Que deviennent les ressources ?", choix: ["2 paient le coût, 1 te reste jusqu'à la fin du tour", "Les 3 sont dépensées", "Le paiement est refusé : il faut l'appoint exact"], bonne: 0,
            pourquoi: "Le coût est retiré de tes ressources ; le reste est gardé pour ce tour, puis perdu en fin de tour.", cr: ["1.14.2", "4.4.3e"] },
          { quiz: "Peux-tu pitcher une carte « pour rien », sans avoir de coût à payer ?", choix: ["Non", "Oui, à tout moment"], bonne: 0,
            pourquoi: "On ne pitche que pour payer un coût qui demande ces ressources, ou si un effet l'ordonne.", cr: ["1.14.3b"] },
        ],
      },
      {
        id: "5-2", titre: "Une carte, trois usages",
        etapes: [
          { texte: "Chaque carte de ta main peut servir de trois façons : la <b>jouer</b> (attaquer, par exemple), la <b>pitcher</b> pour payer une autre carte, ou <b>défendre</b> avec, pendant le tour adverse.", cr: ["5.1.1a", "1.14.3", "7.3.2a"] },
          { texte: "Défendre avec une carte n'est pas la jouer : on ne paie pas son coût.", cr: ["7.3.2"] },
          { texte: "Toute la difficulté du jeu est là : une carte pitchée ne défendra pas, une carte gardée pour défendre ne paiera pas. Le chapitre 7 revient sur la défense." },
          { quiz: "Tu défends avec une carte de coût 3. Combien de ressources paies-tu ?", choix: ["0", "3"], bonne: 0,
            pourquoi: "Déclarer une carte en défense n'est pas la jouer : son coût n'est pas payé.", cr: ["7.3.2"] },
        ],
      },
      {
        id: "5-3", titre: "Les cartes pitchées reviennent",
        etapes: [
          { texte: "À la fin du tour, les cartes de ta zone de pitch vont <b>sous ton deck</b>. Si tu en as plusieurs, tu choisis leur ordre, et ton adversaire ne le connaît pas.", cr: ["4.4.3c", "8.5.15c"] },
          { texte: "Une carte pitchée n'est donc pas perdue : elle reviendra quand tu auras pioché tout le reste de ton deck. Une carte qui va au cimetière n'en sort que si un effet le permet." },
          { quiz: "Qui voit l'ordre dans lequel tu ranges tes cartes pitchées sous ton deck ?", choix: ["Personne d'autre que toi", "Ton adversaire aussi"], bonne: 0,
            pourquoi: "Cet ordre est une information cachée.", cr: ["4.4.3c"] },
        ],
      },
    ],
  },
  {
    no: 6, titre: "Attaquer : la chaîne de combat",
    lecons: [
      {
        id: "6-1", titre: "Les étapes du combat",
        etapes: [
          { texte: "Quand tu joues une attaque, elle va d'abord sur la <b>pile</b>, comme toute carte jouée. Quand elle se résout, le <b>combat</b> commence : l'attaque devient un <b>maillon</b> de la <b>chaîne de combat</b>.", cr: ["5.1.2", "7.0.2", "7.0.3", "7.2.2"] },
          { texte: "Un maillon passe par des étapes, dans l'ordre : <b>Couche</b> (Layer), <b>Attaque</b>, <b>Défense</b>, <b>Réaction</b>, <b>Dégâts</b>, <b>Résolution</b>. Puis vient la <b>Fermeture</b> (Close) quand le combat se termine.", cr: ["7.0.1", "7.7.1"] },
          { texte: "<b>Attaque</b> : l'attaque est posée sur la chaîne, les effets « quand elle attaque » se déclenchent. <b>Défense</b> : le défenseur choisit ses cartes de défense. <b>Réaction</b> : chacun peut jouer des réactions.", cr: ["7.2.2", "7.2.3", "7.3.2", "7.4.1"] },
          { texte: "<b>Dégâts</b> : si la puissance dépasse la défense totale, l'attaque <b>touche</b> et inflige la différence. <b>Résolution</b> : le maillon est terminé ; si l'attaque a go again, son joueur gagne 1 point d'action.", cr: ["7.5.2", "7.6.2"] },
          { quiz: "Dans quel ordre viennent ces étapes ?", choix: ["Défense, puis Réaction, puis Dégâts", "Réaction, puis Défense, puis Dégâts", "Dégâts, puis Défense, puis Réaction"], bonne: 0,
            pourquoi: "Couche, Attaque, Défense, Réaction, Dégâts, Résolution, puis Fermeture.", cr: ["7.0.1"] },
        ],
      },
      {
        id: "6-2", titre: "Qui joue quoi pendant un combat",
        etapes: [
          { texte: "Pendant un combat, on ne joue pas de carte d'action, sauf une nouvelle attaque à l'étape de <b>Résolution</b>. Une carte <b>Instant</b>, elle, se joue dès qu'on a la priorité.", cr: ["7.0.1a", "7.6.3a", "8.1.1b"] },
          { texte: "À l'étape de <b>Réaction</b>, l'attaquant peut jouer des <b>Attack Reactions</b> (réactions d'attaque) et le défenseur des <b>Defense Reactions</b> (réactions de défense).", cr: ["7.4.2a", "7.4.2b", "8.1.2a", "8.1.3a"] },
          { quiz: "Ton adversaire t'attaque. Peux-tu jouer une Attack Reaction ?", choix: ["Non, seul l'attaquant le peut", "Oui, à l'étape de Réaction"], bonne: 0,
            pourquoi: "Une Attack Reaction ne se joue que par le joueur qui contrôle l'attaque, à l'étape de Réaction.", cr: ["8.1.2a"] },
        ],
      },
      {
        id: "6-3", titre: "Enchaîner les attaques",
        etapes: [
          { texte: "À l'étape de Résolution, une attaque avec <b>go again</b> te rend 1 point d'action. Tu peux alors jouer une autre attaque : elle forme un nouveau maillon sur la même chaîne de combat.", cr: ["7.6.2", "7.6.3a"] },
          { texte: "Quand tout le monde passe à l'étape de Résolution sans rien jouer, la chaîne se <b>ferme</b> : le combat est fini et les cartes de la chaîne vont au cimetière.", cr: ["7.7.2a", "7.7.6"] },
          { scene: "go-again",
            consigne: "Tu joues Ira, un héros Ninja. Joue <b>Bittering Thorns</b> (go again), puis, quand il touche, joue <b>Brutal Assault</b> rouge. Paie avec la jaune puis la bleue.",
            reussi: (e) => e.debut.bot.vie - e.bot.vie >= 10,
            bravo: (e) => `Deux attaques dans le même tour : ${e.debut.bot.vie - e.bot.vie} PV en tout. Bittering Thorns a rendu un point d'action (go again). La seconde attaque a eu +1 grâce à Bittering Thorns (« When this hits, your next attack this turn gets +1 ») et +1 grâce à la capacité d'Ira (« Your second attack each turn gets +1 »).` },
          { quiz: "Ton attaque n'a pas go again et tu n'as plus de point d'action. Peux-tu jouer une nouvelle attaque à l'étape de Résolution ?", choix: ["Non", "Oui"], bonne: 0,
            pourquoi: "Une attaque d'action coûte toujours 1 point d'action ; sans go again, tu n'en récupères pas.", cr: ["8.1.1c", "7.6.2"] },
        ],
      },
    ],
  },
  {
    no: 7, titre: "Défendre",
    lecons: [
      {
        id: "7-1", titre: "Bloquer une attaque",
        etapes: [
          { texte: "Quand ton héros est attaqué, à l'étape de Défense, tu peux déclarer autant de cartes de ta <b>main</b> que tu veux comme défense (et ton équipement, voir chapitre 9).", cr: ["7.3.2a"] },
          { texte: "Défendre n'est pas jouer : tu ne paies rien. Une carte sans valeur de défense ne peut pas défendre.", cr: ["7.3.2", "7.3.2b"] },
          { texte: "Les défenses s'additionnent. Les dégâts sont égaux à la puissance moins la défense totale. Si la défense est égale ou plus grande, l'attaque ne touche pas.", cr: ["7.5.2"] },
          { scene: "defendre",
            consigne: "Ton adversaire attaque avec <b>Raging Onslaught</b> (puissance 7). Tes cartes ont 3 de défense chacune. Choisis avec combien de cartes tu défends, puis termine ta déclaration.",
            reussi: (e) => e.vous.cimetiere.length > 0 || e.vous.vie < e.debut.vous.vie,
            bravo: (e) => { const n = e.vous.cimetiere.length, p = e.debut.vous.vie - e.vous.vie; const d = 3 * n; return n === 0 ? `Sans défense, l'attaque touche pour 7 : tu perds ${p} PV.` : d >= 7 ? `Ta défense totale (${d}) atteint ou dépasse la puissance (7) : l'attaque ne touche pas, tu ne perds aucun PV. Mais ${n} cartes ont quitté ta main.` : `Tu as défendu avec ${n} carte${n > 1 ? "s" : ""} (défense ${d}) : 7 − ${d} = ${7 - d}. Tu perds ${p} PV.`; } },
          { quiz: "Une attaque de puissance 6 est défendue par deux cartes de défense 3. Combien de dégâts ?", choix: ["0", "3", "6"], bonne: 0,
            pourquoi: "6 − (3 + 3) = 0 : la défense n'est pas dépassée, l'attaque ne touche pas.", cr: ["7.5.2"] },
        ],
      },
      {
        id: "7-2", titre: "Défendre ou garder ses cartes",
        etapes: [
          { texte: "À la fin d'un tour, seul le joueur du tour repioche (sauf au premier tour). Une carte que tu utilises pour défendre pendant le tour adverse ne sera donc pas remplacée avant la fin de <b>ton</b> tour.", cr: ["4.4.3f"] },
          { texte: "Défendre protège tes PV, mais te laisse moins de cartes pour attaquer et payer à ton tour. Chaque défense est un choix." },
          { quiz: "Tu as 4 cartes, intellect 4. Pendant le tour adverse (pas le premier), tu défends avec 2 cartes. Combien de cartes as-tu au début de ton tour ?", choix: ["2", "4"], bonne: 0,
            pourquoi: "Ton adversaire repioche à la fin de son tour, pas toi : tu commences ton tour avec 2 cartes.", cr: ["4.4.3f"] },
        ],
      },
      {
        id: "7-3", titre: "Les réactions de défense",
        etapes: [
          { cartes: [["Steelblade Shunt", "rouge"]], texte: "Une <b>Defense Reaction</b> ne se déclare pas à l'étape de Défense : elle se <b>joue</b> (on paie son coût) à l'étape de Réaction, par le joueur dont le héros est attaqué.", cr: ["7.3.2a", "8.1.3a", "7.4.2b"] },
          { texte: "Quand elle se résout, elle devient une carte de défense de l'attaque en cours, et sa défense s'ajoute.", cr: ["8.1.3b", "7.4.2d"] },
          { quiz: "Pour défendre avec une Defense Reaction, paie-t-on son coût ?", choix: ["Oui, car elle est jouée", "Non, comme une carte déclarée en défense"], bonne: 0,
            pourquoi: "Une Defense Reaction est jouée à l'étape de Réaction : son coût est payé. Seules les cartes déclarées à l'étape de Défense ne coûtent rien.", cr: ["8.1.3a", "7.3.2"] },
        ],
      },
    ],
  },
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
