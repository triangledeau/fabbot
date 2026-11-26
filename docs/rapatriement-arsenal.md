# Plan de rapatriement des fonctionnalités dans Arsenal

Copie du plan validé par le propriétaire le 2026-11-26 (document d'origine :
https://claude.ai/artifact/2DSrttadbwSwBWaemTPdqM). L'**avancement** de
chaque lot est tenu à jour dans le tableau « Les lots » ci-dessous.

Les fonctionnalités viennent de l'Arsenal JavaScript (dossier
`Documents\CLAUDE\fleshandblood` sur le PC du propriétaire). Le plan les
ramène dans fabbot en 8 lots, environ 14 sessions, puis deux chantiers à
repenser plus tard (didacticiel, puis puzzles), **sans reprendre une seule
règle du moteur JS**.

## Version retenue

C'est l'Arsenal JavaScript du dossier `Documents\CLAUDE\fleshandblood`
(1er au 26 nov.), pas ANTIGRAV.

- **fleshandblood** contient : plateau animé (`outils-bot/src/page/finition.js`),
  impression (`proxies-de-rathe.html`, `impression/séparateurs`), éditeur et
  ajout de decks (`src/page/editeur/`), puzzles de value (`puzzles/value/`),
  didacticiel, tournoi, atelier. Tout est documenté dans
  `outils-bot/docs/PLAN-*.md`.
- **ANTIGRAV** est une reprise plus courte de la même charte (logo Arsenal,
  accueil Jouer / Entraîner / Apprendre) sur un serveur Rust. Il apporte
  quelques extras : signalement de bugs dans la page, conseil du bot et
  barre d'évaluation, sons, banc SPRT et Bot contre Bot dans la page.

Le plan prend donc fleshandblood comme source principale, et ANTIGRAV pour
ses extras (lot 9).

## Inventaire

Au départ (2026-11-26), fabbot a une page web simple
(`crates/ui/web/index.html`, 632 lignes) : un match contre le bot, l'écran
de side avant chaque manche, l'annulation et le texte officiel au survol.
Ni image de carte, ni animation, ni écran d'accueil.

| Fonctionnalité | Où dans l'Arsenal JS | Dans fabbot au départ |
|---|---|---|
| Accueil Arsenal (logo, 3 volets Jouer / Apprendre / Entraîner, fond par deck, navigation par #) | `src/page/gabarit.html`, `arsenal.css`, `interface.js` | Absent |
| Images des cartes (VF bord noir en priorité, cadre dessiné si l'image manque) | `src/page/imgkeys.js`, `resolveImg` | Absent (texte seul) |
| Plateau animé : cartes qui volent, chaîne de laiton en glisser-déposer, impacts, PV qui défilent, bandeau des étapes du combat, pause « Continuer » avant les dégâts | `src/page/finition.js` + `.css` | Absent |
| Sauvegarde et reprise de partie, export / import du journal (.json) | `interface.js` | Annulation seulement |
| Ajouter un deck depuis une liste FaBrary (vérification Silver Age) | `tools/nouveau-deck.py`, import de l'éditeur | En ligne de commande (`verifier-deck`, `docs/nouveau-deck.md`) |
| Éditeur de deck (collection filtrable, catégories Deck / Side / Équipement / Envies, stats d'équilibrage, Ctrl+Z) et gestion (dupliquer, archiver, supprimer) | `src/page/editeur/editeur.js`, `decks-stockage.js` | Absent |
| Ajustement avant la partie (piles, dernier réglage, plan conseillé) | bloc `openAdjust` d'`editeur.js` | Écran de side par manche, plus simple |
| Impression : Proxies de Rathe (planches, fond perdu, VF, repères de coupe, PDF), séparateurs, boîtes de deck | `proxies-de-rathe.html`, `impression/` | Absent |
| Puzzles de value (15 positions, 5 niveaux, solveur, brief) | `puzzles/value/`, `src/page/puzzles.js` | Absent |
| Didacticiel « Apprendre » (spirale, 5 leçons du cycle 1, mémos, partie en règles réduites) | `src/page/didacticiel/`, `moteur/apprenti.js` | Absent |
| Tournoi des bots dans la page (matrice, classement, stats, historique, export) | `src/page/tournoi.js` | En ligne de commande (`fab-sim --tournoi`) |
| Atelier et entraînement d'un deck modifié | `atelier/`, `atelier-bot-fab.html` | En ligne de commande (`--apprendre-b3`, `--apprendre-side`) |
| Signalement de bugs, conseil du bot, sons, Bot contre Bot | ANTIGRAV `ui/app.js` | Absent |

Le pool de cartes jouables diffère : l'Arsenal JS jouait Kano, Lexi, Aurora,
Nuu, Riptide, Zane, Bravo Grindstone ; fabbot joue Hala, Florian, Levia,
Arakni. Les puzzles et le didacticiel qui utilisent des cartes absentes de
fabbot attendent l'encodage de ces cartes.

## Ce qu'on ne rapatrie pas

- **Le moteur JS et les cartes JS** : fabbot est la référence. Aucune
  règle, aucun effet de carte n'est recopié du JS.
- **La couche Talishar** (`src/talishar/`, `talishar/`) : code dérivé de
  Talishar, sous licence GPL-3.0. On n'en reprend aucun fichier.
- **Gabarits, ligue JS, réglage SPSA, note de position, rollouts JS** :
  remplacés dans fabbot par B2, B3 et le tournoi Elo.
- **L'ancien format de decks JS** : fabbot a déjà `data/decks/*.txt` et
  `verifier-deck`.
- **Le son** (décision du 2026-11-26).

Le dépôt fabbot est **public** : les fonds d'accueil d'ArtStation et les
images des cartes ne sont pas mis dans le dépôt (la page les charge en
ligne), et aucun fichier dérivé de Talishar n'y entre.

Les fiches de stratégie (`docs-projet/` : value, fatigue, pitch stacking,
plans de jeu) peuvent être copiées telles quelles dans `docs/strategie/`,
dans n'importe quel lot.

## Principes de portage

1. **La page parle au serveur, elle ne joue pas.** Dans l'Arsenal JS, la
   page contenait le moteur. Dans fabbot, la page envoie des choix au
   serveur `fab-web` et affiche l'état JSON qu'il renvoie. On reprend
   l'habillage (HTML, CSS, animations, écrans), pas la logique de jeu.
2. **L'apparence est copiée, le code est réécrit autour de l'API.** La
   charte (couleurs, polices Cinzel / Sofia Sans / IBM Plex Mono, logo) et
   les CSS se reprennent presque tels quels. Le JavaScript est réadapté
   écran par écran.
3. **Une page en plusieurs fichiers.** `crates/ui/web/` est découpé en un
   dossier par volet (`accueil/`, `partie/`, `decks/`, `impression/`,
   `apprendre/`, `entrainer/`) plus `commun/` (charte). Le serveur les
   intègre au binaire : `Jouer.bat` reste un double-clic, sans Internet sauf
   pour les images (et les polices, qui ont des polices de secours).
4. **L'API grandit par modules.** Chaque lot ajoute ses routes dans son
   propre fichier Rust (`api_decks.rs`, `api_tournoi.rs`…), avec une seule
   ligne dans le routeur. Cela évite les conflits entre agents.
5. **Ce dont les animations ont besoin vient du moteur.** Pour faire voler
   une carte d'une zone à l'autre, la page doit savoir quel objet est
   devenu quel autre, et ce qui s'est passé depuis le dernier état (dégâts,
   pitch, blocage). Le serveur lit déjà les événements du moteur : il faut
   les envoyer à la page sans rien changer aux règles.
6. **Chaque lot garde les règles du projet** : fmt, clippy, tous les
   tests, test `crates/ui/tests/` pour chaque route, et contrôle visuel
   Playwright avant fusion.

## Les lots

Huit lots pour la première phase, environ 14 sessions.

**Lot 0, ce qui a été fait (2026-11-26)** : `crates/ui/web/` = `index.html`
(coquille), `commun/` (charte `arsenal.css`, `navigation.js`, logo),
`accueil/`, `partie/` (l'ancien écran, rhabillé). Fichiers intégrés au
programme par `crates/ui/src/fichiers.rs`. Adresses : `#accueil`,
`#partie`, `#nouveau-match` ; un nouveau volet ajoute sa ligne dans
`ADRESSES` (`commun/navigation.js`) et sa règle `.vue` dans
`commun/arsenal.css`. Pas de fond d'ArtStation (dépôt public) : les volets
de l'accueil ont un dégradé en attendant les images (lot 1).

**Lot 1, ce qui a été fait (2026-11-26)** : `tools/images_cartes.py` lit le
fichier officiel LSS et écrit `data/cards/images.tsv` (nom anglais, couleur,
code d'image, nom français ; VF normale d'abord, sinon anglaise hors
Unlimited, Silver Age en priorité) : 396 cartes, 343 en VF, 0 sans image.
`crates/ui/src/images.rs` ajoute `image`, `nom_fr` (et `heros_image`,
`puissance_imprimee`, `defense_imprimee`) au JSON ; `commun/cartes.js`
(`Images.url`, `Images.img`, `Images.fond`) pour tous les volets. Le volet
Jouer de l'accueil montre votre héros. Chaque lot se
termine par une pull request fusionnée et peut être pris par n'importe quel
agent.

| Lot | Contenu | Zone touchée | Dépend de | Effort (sessions) | Avancement |
|---|---|---|---|---|---|
| 0. Socle | Page en plusieurs fichiers, accueil Arsenal (logo, 3 volets, fond), charte, navigation par #. L'écran de partie actuel est gardé et rhabillé. | `crates/ui/web/`, `serveur.rs` | rien | 1 | ✅ fait (2026-11-26) |
| 1. Images | Script qui associe chaque carte codée à son image (VF bord noir d'abord), nom français, champ `image` dans l'API, cadre dessiné en secours | `tools/`, `data/cards/`, `crates/ui/src/vue.rs` | rien | 1 | ✅ fait (2026-11-26) |
| 2. Plateau animé | Reprise de `finition.js` : vols de cartes, glisser-déposer avec chaîne de laiton, impacts, PV qui défilent, bandeau des étapes, pause « Continuer ». Côté serveur : identifiants suivis et événements envoyés à la page | `crates/ui/web/partie/`, `crates/ui/src/web.rs` | 0, 1 | 3 | à faire |
| 3. Sauvegarde | Partie sauvegardée (graine + choix), reprise depuis l'accueil, export / import du journal .json | `crates/ui/src/`, `web/accueil/` | 0 | 1 | à faire |
| 4a. Decks en données | Pool Silver Age avec drapeau « jouable par le bot » (carte codée dans fab-cards), légalité, decks maison dans `data/decks/maison/`, chargement à l'exécution, import d'une liste FaBrary | `crates/cards/` (lecture des decks), `api_decks.rs` | 0 | 1 | à faire |
| 4b. Éditeur de deck | Reprise d'`editeur.js` : collection filtrable, catégories, stats, Ctrl+Z, dupliquer / archiver / supprimer | `web/decks/` | 4a, 1 | 1,5 | à faire |
| 4c. Ajustement avant partie | Fenêtre en piles de l'Arsenal JS à la place de l'écran de side actuel, dernier réglage, plan conseillé par la table apprise | `web/partie/`, `match_web.rs` | 4a, 1 | 0,5 | à faire |
| 5. Impression | Proxies de Rathe, séparateurs et boîtes de deck servis par fab-web, alimentés par la liste d'un deck | `web/impression/` | 0 (4a pour les decks maison) | 1 | à faire |
| 8. Entraîner | Tournoi dans la page (matrice, classement, stats, historique, export), banc SPRT, lancement de `--apprendre-side` / `--apprendre-b3` sur un deck modifié avec progression | `web/entrainer/`, `api_tournoi.rs`, `crates/sim/` | 0 (4a pour les decks maison) | 2 | à faire |
| 9. Extras ANTIGRAV | Signalement de bugs dans la page (enregistrés dans un fichier), conseil du bot et barre d'évaluation, Bot contre Bot | `web/partie/`, `api_bugs.rs` | 0 (2 pour le conseil) | 1 | à faire |

Les numéros 6 et 7 sont gardés pour les chantiers de plus tard (voir
« Ordre conseillé »).

## Ordre conseillé (validé par le propriétaire le 2026-11-26)

Dans une vague, les lots touchent des fichiers différents et peuvent
tourner en même temps sur deux ou trois agents.

1. **Vague 1 : lot 0 (socle) et lot 1 (images).** Rien d'autre ne démarre
   avant le socle, puisqu'il fixe le découpage des fichiers. Les images
   peuvent avancer en parallèle, elles ne touchent que `tools/` et la vue
   JSON.
2. **Vague 2 : lots 2 (plateau animé), 3 (sauvegarde), 4a (decks en
   données), 5 (impression).** L'impression est indépendante : elle convient
   bien à la session Antigravity.
3. **Vague 3 : lots 4b (éditeur), 4c (ajustement), 8 (entraîner), 9
   (extras).** Le signalement de bugs du lot 9 peut remonter en vague 2.

Ensuite, deux chantiers à repenser plutôt qu'à rapatrier tels quels :

- **Phase 2 : didacticiel repensé (lot 7).** On repart des idées (règles
  réduites, bot « apprenti ») avec une nouvelle conception, à valider sur
  maquette. Il touchera le cœur du moteur (`crates/engine/`) : à annoncer
  comme « un seul agent à la fois », drapeaux de règles inactifs par défaut.
- **Phase 3 : puzzles remaniés (lot 6).** Les 15 positions et le solveur de
  l'Arsenal JS servent de point de départ, revérifiés avec les cartes de
  fabbot.

## Travail à plusieurs agents

- **Un lot = une ligne dans `docs/taches.md`**, avec sa zone (dossier
  `web/<volet>/`, fichier `api_<lot>.rs`). Deux lots de la même vague ne
  partagent pas de dossier.
- **Points chauds** : `serveur.rs` (routeur), `vue.rs` (état JSON),
  `crates/ui/web/commun/` (charte). Chaque lot n'y ajoute que des lignes,
  sans réorganiser. En cas de conflit, on garde les deux côtés.
- **Sources à lire** : l'agent lit le plan du chantier dans l'Arsenal JS
  (`outils-bot/docs/PLAN-*.md`) et le fichier source indiqué dans
  l'inventaire. Les sessions cloud n'ont pas ce dossier : le lot commence
  par copier les fichiers utiles dans `docs/arsenal-js/` (CSS, maquettes,
  positions des puzzles), sans le moteur ni la couche Talishar.
- **Fin de lot** : journal dans `docs/journal/`, mise à jour de
  `docs/interface.md`, captures Playwright jointes à la pull request.

Message à coller pour lancer un lot :

> Projet fabbot. Lis `AGENTS.md` (ou `CLAUDE.md`), puis `docs/suivi.md`,
> `docs/taches.md` et `docs/rapatriement-arsenal.md`. Pars de la branche
> `main`. Tâche : lot <n> (<nom du lot>) du plan de rapatriement. Source
> dans l'Arsenal JS : <fichiers de l'inventaire>.
