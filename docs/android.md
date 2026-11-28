# Arsenal sur Android : brief pour l'agent qui fait l'appli

Ce document est **le point de départ unique** du travail sur l'appli
Android d'Arsenal. Il a été écrit le 2026-11-28 pour un agent Claude qui
travaille depuis un autre compte d'Antoine, sans accès aux discussions où
le projet a été organisé. Tout ce qu'il faut savoir est ici ou dans les
fichiers du dépôt cités. Si une information manque, demande-la à Antoine
plutôt que de la deviner.

À lire avant de commencer, dans cet ordre : ce document en entier, puis
`AGENTS.md`, `CLAUDE.md`, `docs/suivi.md` (seulement « Où en est-on ») et
`docs/taches.md`. `docs/interface.md` décrit l'interface PC que tu vas
porter.

Ton périmètre : **uniquement l'appli Android**. Pas de travail sur le
moteur, les cartes, les bots ni l'interface PC.

## 1. Le projet en deux minutes

- **fabbot** est un programme en Rust qui joue à Flesh and Blood au format
  Silver Age avec 9 decks (Ira, Hala, Dorinthea, Bravo, Iyslander,
  Florian, Levia, Dromai, Arakni), plus les decks qu'Antoine crée lui-même
  (« decks maison »).
- **Arsenal** est le nom de l'application : sur PC, Antoine double-clique
  sur `Jouer.bat`, qui compile et lance `fab-web`. C'est un petit serveur
  web local (sur `127.0.0.1`), qui ouvre le navigateur sur une page en
  HTML, CSS et JavaScript. La page affiche l'accueil, la partie contre le
  bot, les decks, l'éditeur de deck, l'impression des proxies,
  l'entraînement des bots et le didacticiel « Apprendre ».
- **Antoine** est le propriétaire. Il connaît très bien le jeu, mais il
  débute en programmation et avec Git. Explique-lui les choses en
  français, simplement, sans jargon inutile, et dis-lui clairement ce que
  tu attends de lui quand tu as besoin de lui (avec le chemin exact du
  bouton ou du fichier).
- D'autres agents travaillent sur le même dépôt en même temps (d'autres
  sessions Claude, une session Antigravity). Ils ne savent pas que tu
  existes, sauf par `docs/taches.md`. La section 6 explique comment
  cohabiter avec eux.

### Les crates (dossier `crates/`)

| Crate | Rôle | Toi |
|---|---|---|
| `engine` (`fab-engine`) | règles du jeu, état de la partie, points de décision | tu l'utilises, tu n'y touches pas |
| `cards` (`fab-cards`) | cartes et decks | tu l'utilises ; une seule exception, section 5.1 |
| `ai` (`fab-ai`) | les bots B0 à B3 et le niveau « réfléchi » | tu l'utilises, tu n'y touches pas |
| `fab-match` | matchs en plusieurs manches, phase de side | tu l'utilises, tu n'y touches pas |
| `sim` (`fab-sim`) | simulateur en ligne de commande | rien à voir avec toi |
| `ui` (`fab-ui`) | interface PC : `fab-web` (serveur + page) et l'interface texte | tu l'utilises comme bibliothèque, tu n'y touches pas |

Aucune de ces crates n'a de dépendance extérieure lourde (seulement
`serde`, `serde_json`, `toml`) : elles se compilent pour Android sans
difficulté.

## 2. L'objectif

Une appli Android, installée sur le téléphone d'Antoine à partir d'un
fichier **APK** (pas de Play Store), qui offre **les mêmes écrans
qu'Arsenal sur PC**, adaptés à un écran de téléphone tactile.

- Le moteur et le bot tournent **sur le téléphone**, sans serveur sur
  Internet et sans compte. Seules les images des cartes viennent
  d'Internet (comme sur PC, voir 4.4).
- Le bot sera plus lent que sur la Surface d'Antoine. C'est accepté. Il
  faut seulement que l'appli reste fluide pendant que le bot réfléchit et
  que la lenteur soit mesurée et annoncée (jalon 5).
- L'interface PC ne change pas. Quand un autre agent améliore la page PC,
  l'appli Android doit en profiter sans travail de ta part, ou presque.

## 3. Architecture retenue (décidée avec Antoine)

```
┌───────────────────────── Téléphone ──────────────────────────┐
│                                                              │
│  Appli Android (Kotlin, une seule activité)                  │
│   └─ WebView plein écran ── http://127.0.0.1:<port>/ ──┐     │
│                                                        │     │
│  Bibliothèque Rust libfab_android.so (crates/android)  │     │
│   ├─ petit serveur local (127.0.0.1) ◄─────────────────┘     │
│   │    ├─ /android/…  : fichiers propres au téléphone        │
│   │    └─ tout le reste → fab_ui::serveur::repondre_avec_corps│
│   └─ fab-ui → fab-match, fab-ai, fab-cards, fab-engine       │
│                                                              │
│  Dossier privé de l'appli : decks, decks maison, sauvegardes │
└──────────────────────────────────────────────────────────────┘
```

En une phrase : l'appli Android est une **coquille** qui démarre le même
serveur qu'Arsenal sur PC, mais à l'intérieur du téléphone, et l'affiche
dans une WebView. La page web PC est servie telle quelle, avec en plus une
feuille de style et un script « téléphone » qui l'adaptent au tactile et
au petit écran.

### Pourquoi ce choix

La page d'Arsenal fait déjà tout : plus de 6 500 lignes de Rust côté
serveur (`crates/ui/src/`) et environ 700 Ko de HTML, CSS et JavaScript
(`crates/ui/web/`), avec des dizaines de routes `/api/…`. Le moteur parle
à la page uniquement par ces routes, en JSON.

- **WebView + serveur local (retenu).** On réutilise 100 % de la page et
  de l'API. Une amélioration de la page PC arrive sur Android à la
  prochaine compilation. Le code propre à Android reste petit : une
  activité Kotlin, une bibliothèque Rust de quelques centaines de lignes,
  une feuille de style et un script d'adaptation.
- **Réécrire l'interface en Kotlin / Jetpack Compose, Flutter ou en Rust
  (egui, Slint) : écarté.** Il faudrait refaire chaque écran, puis tenir
  deux interfaces à jour en parallèle. Elles divergeraient en quelques
  semaines, alors que plusieurs agents modifient la page PC chaque jour.
- **Tauri 2 (version mobile) : écarté.** Il fait lui aussi une WebView,
  mais il impose son propre système d'échanges entre la page et Rust, qui
  ne correspond pas aux routes `/api/…` existantes. Il ajoute un gros
  outillage (Node, CLI Tauri) pour un gain nul ici.
- **Intercepter les requêtes de la WebView sans serveur
  (`shouldInterceptRequest`) : écarté.** Android ne donne pas le corps des
  requêtes `POST` à cette fonction, or la page en envoie (chargement d'une
  sauvegarde, decks maison).

Pour relier Kotlin et Rust : la crate `jni` (version 0.21) côté Rust, et
une seule fonction exposée à Kotlin, du genre
`demarrer(dossier_donnees: String) -> port: Int`. Pour compiler Rust pour
Android : `cargo-ndk`.

## 4. Ce qu'il faut savoir avant d'écrire une ligne

J'ai lu le code de `fab-ui` pour repérer les pièges. Les numéros de ligne
sont ceux du 2026-11-28 ; vérifie-les, le code bouge.

### 4.1 Comment `fab-web` démarre (à reproduire dans `crates/android`)

Lis `crates/ui/src/bin/web.rs` (120 lignes). Il fait, dans l'ordre :

1. lire ses réglages (`fab_ui::web::Reglages` : deck, deck du bot, niveau
   `b3` par défaut, graine, manches) ;
2. reprendre la partie sauvegardée `sauvegardes/en-cours.json` si elle
   existe et n'est pas finie (`fab_ui::sauvegarde::lire`), sinon créer un
   match (`fab_ui::match_web::MatchWeb::nouveau`) ;
3. charger les textes officiels des cartes (`fab_ui::textes::Textes`) ;
4. réserver un port (`fab_ui::serveur::ecouter(8080)`, qui essaie les 20
   ports suivants s'il est pris) ;
5. construire `fab_ui::serveur::Etat` et appeler
   `fab_ui::serveur::servir_toujours`.

Ta fonction `demarrer` fait la même chose, dans un fil à elle (elle doit
rendre la main tout de suite à Kotlin avec le numéro de port).

### 4.2 Les chemins de fichiers : le vrai piège

Plusieurs fichiers sont lus **pendant que le programme tourne**, à un
chemin fixé **au moment de la compilation** avec
`env!("CARGO_MANIFEST_DIR")`. Ce chemin est celui de l'ordinateur qui a
compilé (par exemple `/home/user/fabbot/crates/cards`). Il n'existe pas sur
le téléphone.

| Quoi | Où dans le code | Sur Android | Que faire |
|---|---|---|---|
| Les 9 decks `data/decks/*.txt` et les decks maison `data/decks/maison/` | `crates/cards/src/decks.rs`, fonction `dossier()` (vers la ligne 33) ; `tous()` fait un `expect` | **l'appli plante au démarrage** | correctif partagé, section 5.1 |
| Textes officiels des cartes (`data/cards/sources/public_card_data.csv`, 33 Mo, non versionné) | `crates/ui/src/textes.rs`, `Textes::charger()` | fichier introuvable : page sans texte des cartes | ne pas appeler `charger()` : lire le fichier toi-même et appeler `Textes::depuis_csv(&texte)` (fonction publique) |
| Vrai dos de carte (`data/outils/dos-de-carte.webp`, non versionné) | `crates/ui/src/fichiers.rs`, `dos_de_carte()` | absent : la page dessine un dos de remplacement | ton serveur répond lui-même à `GET /outils/dos-de-carte` avec le fichier embarqué |
| Outil Proxies de Rathe (`data/outils/proxies-de-rathe.html`, non versionné) | `fichiers.rs`, `chemin_proxies()` | absent : l'impression complète est indisponible | voir jalon 6 ; le fichier ne doit jamais entrer dans le dépôt (il est privé) |

Les sauvegardes, elles, utilisent des chemins **relatifs au dossier
courant** : `sauvegardes/en-cours.json` (`sauvegarde.rs`),
`sauvegardes/bugs/` (`extras.rs`), `sauvegardes/tournois/`
(`api_tournoi.rs`). Il suffit donc que `demarrer` fasse
`std::env::set_current_dir(dossier_donnees)` avant tout le reste, avec
`dossier_donnees` = le dossier privé de l'appli (`context.filesDir` côté
Kotlin).

Les poids du bot B3 (`data/b3/evaluation.txt`), la table de side
(`data/side/b1.txt`), les tables d'images et de collection
(`data/cards/*.tsv`), le format (`config/silver_age.toml`) et tous les
fichiers de la page (`crates/ui/web/`) sont **intégrés au programme** à
la compilation (`include_str!` / `include_bytes!`) : rien à faire.

### 4.3 Le serveur : écrire ta propre boucle

La boucle qui lit les requêtes HTTP (`servir`, vers la ligne 465 de
`crates/ui/src/serveur.rs`) est privée. La fonction qui calcule la
réponse est publique :

```rust
pub fn repondre_avec_corps(methode: &str, requete: &str, corps: &[u8], etat: &mut Etat) -> Reponse
```

Écris dans `crates/android` ta propre boucle (une soixantaine de lignes),
en copiant la logique de `servir` : lecture de la première ligne et des
en-têtes `Content-Length` et `Host`, refus si l'hôte n'est pas local,
corps limité à 4 Mo, verrou sur l'`Etat`, et **écriture de la sauvegarde
après chaque `POST` réussi** (`fab_ui::sauvegarde::ecrire`). Mets en
commentaire en tête : « copie de `fab_ui::serveur::servir` du
2026-11-xx, à garder alignée ».

Avoir ta boucle te permet de servir tes propres routes avant de passer la
main à `fab-ui` :

- `GET /android/mobile.css` et `GET /android/mobile.js` : l'adaptation au
  téléphone (section 4.5), rangées dans `crates/android/web/` et intégrées
  avec `include_bytes!` ;
- `GET /` et `GET /index.html` : tu prends la réponse de `fab-ui` et tu
  ajoutes juste avant `</head>` les deux balises `<link>` et `<script>`
  vers ces fichiers ;
- `GET /outils/dos-de-carte` : le dos de carte embarqué, si Antoine le
  fournit.

### 4.4 Réseau, images et sécurité

- Les images des cartes viennent du serveur d'images de LSS
  (`https://legendstory-production-s3-public.s3.amazonaws.com/media/cards/…`,
  constante `IMAGES` dans `crates/ui/src/images.rs`). Sans Internet,
  l'appli marche mais affiche les cadres dessinés. Pas de téléchargement
  massif d'images dans l'APK sans accord d'Antoine (droits et taille).
- Il faut la permission `INTERNET` (pour les images et pour la connexion
  locale) et un `network_security_config.xml` qui autorise le HTTP non
  chiffré **seulement** vers `127.0.0.1` et `localhost`.
- Le serveur n'écoute que sur `127.0.0.1`. D'autres applis du téléphone
  pourraient en théorie s'y connecter. Pour une appli personnelle c'est
  acceptable ; si tu veux fermer cette porte, ajoute un jeton secret tiré
  au hasard au démarrage, passé dans l'adresse et vérifié par ta boucle.

### 4.5 La page n'est pas faite pour un téléphone

La page PC a une balise `viewport` et quelques `@media` (largeur maximale
900 px, `hover: none`), mais elle a été conçue pour l'écran de la Surface
(environ 1 920 × 1 280, souris, clavier). Points repérés dans
`crates/ui/web/` :

| Fonction de la page | Fichiers | Problème sur téléphone | Piste |
|---|---|---|---|
| Texte officiel de la carte **au survol** | `partie/partie.js` (`mouseenter`) | pas de survol au doigt | appui long ou toucher → grande carte en surimpression (`mobile.js`) |
| Glisser-déposer HTML5 (chaîne de laiton, éditeur) | `partie/finition.js`, `decks/editeur.js` (`dragstart`) | la WebView ne déclenche pas `dragstart` au doigt | vérifier qu'un clic suffit ; sinon équivalent au toucher dans `mobile.js` |
| Menus au clic droit | `partie.js`, `decks.js`, `editeur.js` (`contextmenu`) | le clic droit devient l'appui long | vérifier que l'appui long ouvre bien le menu |
| Raccourcis clavier (Ctrl+Z, Échap…) | `finition.js`, `partie.js`, `editeur.js`, `decks.js`… (`keydown`) | pas de clavier | vérifier qu'un bouton existe pour chaque raccourci utile ; le bouton Retour d'Android peut faire « Échap » |
| `confirm(…)` | `partie.js`, `entrainer.js` | la WebView ne l'affiche pas sans aide | `WebChromeClient.onJsConfirm` → boîte de dialogue Android |
| Téléchargement de fichiers (`a.download`, liens `blob:`) : sauvegarde, export de deck, export de tournoi | `partie.js`, `decks.js`, `editeur.js`, `entrainer.js` | rien ne se passe dans une WebView | pont JavaScript → Kotlin qui enregistre dans Téléchargements |
| Import de fichier (`<input type="file">`) | `index.html` | rien ne s'ouvre sans aide | `WebChromeClient.onShowFileChooser` |
| `window.print()` | `impression/impression.js` | rien ne se passe | pont vers `PrintManager` d'Android (impression ou PDF) |
| Presse-papiers | `decks.js`, `editeur.js`, `impression.js` | peut être refusé | vérifier, et prévoir un message clair |
| `localStorage` (réglages mémorisés) | plusieurs fichiers | désactivé par défaut dans une WebView | `settings.domStorageEnabled = true` |

Règle : **toute adaptation passe par `crates/android/web/mobile.css` et
`mobile.js`**, jamais par une modification de `crates/ui/web/`. Ces deux
fichiers visent les classes et les identifiants de la page PC ; si un
agent les renomme, ton adaptation casse sans bruit. Ajoute donc dans
`crates/android` un test qui vérifie que chaque sélecteur utilisé par
`mobile.css` et `mobile.js` existe encore dans les fichiers de
`crates/ui/web/` (simple recherche de texte). Ce test tourne dans la CI
de tout le monde et prévient l'autre agent avant qu'il fusionne.

Si un écran ne peut vraiment pas s'adapter de l'extérieur, ne force pas :
écris à Antoine ce qu'il faudrait changer dans la page PC et pourquoi
(section 5.2).

### 4.6 Le bot sur le téléphone

- B3 et le niveau « réfléchi » cherchent pendant un temps donné ou un
  nombre de simulations donné. Le niveau réfléchi utilise
  `available_parallelism()` fils de calcul (`crates/ai/src/reflechi.rs`)
  et tourne dans un fil à part (`crates/ui/src/web.rs`, vers la ligne
  438), avec une jauge (`GET /api/reflexion`) et un bouton « Joue
  maintenant ». Rien à changer : vérifie seulement que la jauge s'affiche
  et que l'appli reste fluide.
- Le volet « Entraîner » lance des tournois et des apprentissages
  (`crates/ui/src/api_tournoi.rs`). Sur un téléphone, ces calculs sont
  longs, chauffent et vident la batterie. Ils doivent rester possibles,
  mais avec un avertissement (jalon 6).
- Le niveau de bot par défaut reste B3, comme sur PC, sauf avis contraire
  d'Antoine après la mesure du jalon 5.

### 4.7 Le cycle de vie d'Android

- Quand l'appli passe en arrière-plan, Android peut tuer le processus à
  tout moment, y compris pendant que le bot réfléchit. Ce n'est pas grave
  parce que la partie est sauvegardée après chaque coup : au redémarrage,
  `demarrer` reprend `sauvegardes/en-cours.json`. Teste-le (jalon 3).
- Le bouton Retour d'Android : si la WebView peut revenir en arrière
  (`canGoBack()`), elle revient (la navigation d'Arsenal passe par le `#`
  de l'adresse, voir `crates/ui/web/commun/navigation.js`) ; sinon, on
  quitte.
- L'orientation : le plateau de jeu a besoin de largeur. Propose à
  Antoine le mode paysage pour la partie (ou pour toute l'appli) au jalon
  2, captures à l'appui, et note sa réponse dans ce document.
- Pas d'écran de veille pendant que le bot réfléchit :
  `FLAG_KEEP_SCREEN_ON` tant que `/api/reflexion` ne renvoie pas `null`
  (facultatif, à proposer).

## 5. Ce qui est interdit, et ce qui demande l'accord d'Antoine

### Interdit sans accord écrit d'Antoine

- Modifier `crates/engine`, `crates/cards`, `crates/ai`, `crates/fab-match`,
  `crates/sim` ou `crates/ui` (code, page web, tests), sauf l'exception de
  5.1.
- Modifier les poids du bot B3 (`data/b3/`), la table de side
  (`data/side/b1.txt`), les decks (`data/decks/`), les chiffres d'Elo de
  `docs/bots.md`, `config/silver_age.toml`.
- Modifier `.github/workflows/ci.yml` (tu peux **ajouter** un fichier
  `android.yml` à côté, jalon 7).
- Mettre dans le dépôt un fichier qui n'a pas à y être : le CSV des
  textes de LSS, le dos de carte, l'outil Proxies de Rathe, des images de
  cartes, une clé de signature (`*.jks`, `*.keystore`), un mot de passe.
  Le dépôt est public.
- Pousser sur `main` directement, faire un `git push --force` sur `main`
  ou sur la branche d'un autre agent.
- Publier l'appli (Play Store, site, lien public). Exception : la Release
  GitHub `arsenal-dernier` (section 5.3).

### 5.1 L'exception autorisée d'avance : le dossier des decks

Accord d'Antoine du 2026-11-28 pour **ce changement-là et aucun autre**,
dans une petite pull request à part, avant le reste :

- dans `crates/cards/src/decks.rs`, la fonction `dossier()` regarde
  d'abord une variable d'environnement `FABBOT_DECKS` ; si elle existe et
  n'est pas vide, elle renvoie ce chemin ; sinon elle renvoie le chemin
  actuel, sans aucun changement ;
- un test dans `crates/cards` vérifie que sans la variable le chemin ne
  change pas (attention : les tests tournent en parallèle, ne modifie pas
  la variable d'environnement dans un test qui partage le processus avec
  d'autres ; sépare la logique dans une fonction qui prend la valeur en
  paramètre et teste celle-ci) ;
- une ligne dans la documentation de la fonction.

Côté Android, `demarrer` copie au premier lancement (et à chaque mise à
jour de l'appli) les decks embarqués dans `<dossier privé>/decks/`, sans
jamais toucher `<dossier privé>/decks/maison/`, puis fait
`std::env::set_var("FABBOT_DECKS", …)` **avant** de lancer le moindre
fil.

Pour embarquer les decks sans toucher `data/decks/` : `include_str!` de
chaque fichier depuis `crates/android/src/`, ou un `build.rs` dans
`crates/android` qui parcourt `../../data/decks/*.txt`. Le `build.rs` est
préférable : un 10e deck ajouté par un autre agent arrive tout seul.

Annonce cette pull request dans `docs/taches.md` (zone :
`crates/cards/src/decks.rs`, une fonction) et fusionne-la seule, avant le
reste, pour que les autres agents la voient vite.

### 5.2 Tout autre besoin dans la zone partagée

Si tu as besoin d'autre chose dans `crates/ui`, `crates/cards`… (une
fonction rendue publique, une page PC rendue plus souple, un bug que tu
découvres), **ne le fais pas**. Écris à Antoine un message court :

> Pour l'appli Android, j'aurais besoin de changer <fichier>, <ce qui
> change>, parce que <raison>. Sur PC, ça <ne change rien / change ceci>.
> Tu es d'accord ?

S'il dit oui, fais-le dans une pull request à part, annoncée dans
`docs/taches.md`, et note son accord (date et contenu) dans ce document,
section 5.

Si tu trouves un bug de règle ou de carte en jouant sur le téléphone :
ne le corrige pas, note-le dans ton point de reprise et signale-le à
Antoine (il le confiera à un autre agent).

### 5.3 Accords d'Antoine notés en cours de route

- **2026-11-28 : l'APK publié dans une Release GitHub.** Sur téléphone,
  la page GitHub d'une construction n'affiche pas le lien de
  l'artefact. Antoine a donc accepté que le workflow publie l'APK
  construit depuis `main` dans la Release `arsenal-dernier`
  (préversion, remplacée à chaque construction), avec un lien fixe et
  public :
  `https://github.com/triangledeau/fabbot/releases/download/arsenal-dernier/arsenal.apk`.
  Conséquence : si le secret `ARSENAL_DOS_DE_CARTE` est ajouté, le vrai
  dos de carte est dans cet APK public.

## 6. Travailler sans coordinateur

Antoine coordonne plusieurs agents qui ne se parlent pas. Ton seul moyen
de leur dire ce que tu fais est le dépôt. Suis ces règles à la lettre
(elles complètent « Travailler à plusieurs » d'`AGENTS.md`).

### Ta zone

Tu es seul à travailler dans :

- `crates/android/` (crate Rust `fab-android`, bibliothèque `cdylib`) ;
- `android/` à la racine (projet Gradle : appli Kotlin) ;
- `docs/android.md` (ce document) et `docs/journal/AAAA-MM-JJ-android-*.md` ;
- `.github/workflows/android.yml` (si tu le crées, jalon 7).

Fichiers partagés que tu touches seulement pour ajouter ta ligne :
`Cargo.toml` à la racine (ajouter `"crates/android"` dans `members`),
`Cargo.lock` (produit par cargo : ne jamais le fusionner à la main, voir
`AGENTS.md`), `.gitignore` (ajouter les dossiers produits par Gradle :
`android/.gradle/`, `android/build/`, `android/app/build/`,
`android/local.properties`, `android/app/src/main/jniLibs/`,
`android/app/src/main/assets/donnees/`), `docs/taches.md`,
`docs/suivi.md` (une ligne).

La crate `crates/android` fait partie du workspace pour une raison
précise : si un autre agent change une fonction de `fab-ui` que tu
utilises, la CI échoue sur **sa** pull request et il le voit tout de
suite, au lieu que l'appli casse en silence des semaines plus tard. Elle
doit donc compiler et passer ses tests **sur PC** aussi (la crate `jni`
compile sur toutes les plateformes ; mets ce qui ne sert qu'à Android
derrière `#[cfg(target_os = "android")]` si besoin). `cargo test
--workspace` ne doit pas demander le SDK Android.

### Au début de chaque session

1. `git fetch origin main`, puis repars de `main` à jour.
2. Lis `docs/taches.md` et la section 10 « Point de reprise » de ce
   document.
3. Travaille sur une branche à toi qui commence par `claude/android-`
   (par exemple `claude/android-jalon-2`). Si ta session impose un autre
   nom qui commence par `claude/`, il convient aussi. Une branche par
   jalon, fusionnée à la fin du jalon.
4. Vérifie dans `docs/taches.md` que personne d'autre ne travaille sur
   « Android ». Si une autre ligne touche `crates/ui/web/` en même temps
   que toi, c'est normal (c'est leur zone) : relance simplement ton test
   des sélecteurs après chaque `git merge origin/main`.
5. Ajoute ou mets à jour ta ligne dans « En cours » de `docs/taches.md`,
   par exemple :
   `| 2026-11-28 | Claude (compte Android) | claude/android-jalon-1 | Appli Android, jalon 1 | crates/android/, android/, docs/android.md |`
   et fais-la arriver sur `main` **tout de suite** (petite pull request à
   part, fusionnée dès que la CI est verte).

### Pendant la session

- Committe souvent, avec des messages en français préfixés par
  `Android :` (par exemple `Android : la WebView affiche l'accueil`).
- **Pousse ta branche au moins toutes les 30 minutes de travail**, même
  si ce n'est pas fini. Une session peut être coupée à tout moment par
  un quota ; ce qui n'est pas poussé est perdu.
- Tiens à jour la section 10 « Point de reprise » à chaque commit
  important : ce qui est fait, ce qui reste, la prochaine commande à
  lancer. Une autre session doit pouvoir reprendre sans rien te demander.

### À la fin d'un jalon

1. `git fetch origin main` puis `git merge origin/main` dans ta branche.
2. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace` : tout au vert. Plus la compilation Android
   (section 8) si tu peux la faire là où tu es.
3. Écris ton journal dans `docs/journal/AAAA-MM-JJ-android-<jalon>.md`
   (date, agent, branche, ce qui est fait, chiffres mesurés, ce qui n'est
   pas repris).
4. Coche le jalon dans la section 9, mets à jour le point de reprise, et
   la ligne « Android » de `docs/suivi.md` (une seule ligne, ne
   reformule pas le reste du fichier).
5. Retire ou mets à jour ta ligne dans `docs/taches.md`.
6. Pousse, ouvre la pull request vers `main`, attends que la vérification
   GitHub « CI » soit verte, puis fusionne (accord d'Antoine du
   2026-11-26 pour les sessions Claude, voir `CLAUDE.md`). Si la fusion
   échoue à cause d'un conflit dans un fichier qui n'est pas à toi, ne
   force rien : explique le conflit à Antoine.
7. Dis à Antoine, en deux ou trois phrases simples, ce qui a été fusionné,
   où trouver l'APK, et ce qu'il doit essayer sur son téléphone.
8. Donne-lui le message à coller pour lancer la session suivante (section
   11), et rappelle-lui de **changer de conversation** (cela consomme
   moins).

## 7. Les exigences d'Antoine sur la qualité

Antoine a été déçu par des livraisons annoncées « finies » qui ne
l'étaient pas. Ces règles viennent de ses remarques du 26 et du 27
novembre ; il ne veut pas avoir à signaler les défauts un par un.

1. **Français partout** : chaque texte visible (boutons, messages,
   titres, erreurs, nom de l'appli, boîtes de dialogue Android, message
   d'avertissement). Pas d'anglais oublié, même dans une erreur. Typographie
   française : guillemets « », espace avant `: ; ! ?`.
2. **Langage naturel, sans tics d'IA**, dans l'appli comme dans tes
   messages : pas de « crucial », « robuste », « optimiser », « plonger
   dans », « il est important de noter que » ; pas de listes de trois
   adjectifs ; pas d'emojis décoratifs ; des phrases simples et précises.
   Écris comme un joueur expérimenté parlerait à un autre joueur.
3. **Captures de chaque écran, à la taille du téléphone d'Antoine, et
   relecture par toi-même avant toute livraison.** Pour chaque capture,
   vérifie et note : contraste lisible, rien qui déborde de l'écran, rien
   qui se chevauche, aucun texte coupé, boutons assez grands pour un
   doigt (au moins 48 × 48 dp, soit environ 9 mm), français correct. Tu
   corriges avant de montrer, pas après.
4. **Comparer à l'interface PC, écran par écran et fonction par
   fonction.** Pour chaque écran, une ligne dans le tableau de la section
   9.1 : ce qui marche, ce qui marche autrement (et pourquoi), ce qui ne
   marche pas.
5. **Lister explicitement ce qui n'est pas repris.** Ne jamais dire
   « c'est fini » si une fonction de la version PC manque sans être
   écrite noir sur blanc dans la liste « Non repris ».
6. **Tester dans l'appli réelle**, installée sur un téléphone ou un
   émulateur, compilée depuis `main` après la fusion. Des tests qui
   passent ne suffisent pas.
7. **Ne jamais deviner une règle du jeu ou un texte de carte.** Ce n'est
   pas ton sujet, mais si une question de règle se pose, demande à
   Antoine.

## 8. Construire et tester

### 8.1 Les outils

| Outil | Pour quoi | Installation |
|---|---|---|
| Rust (stable) + cibles Android | compiler `fab-android` | `rustup target add aarch64-linux-android x86_64-linux-android` |
| `cargo-ndk` | appeler le compilateur du NDK pour chaque cible | `cargo install cargo-ndk` |
| SDK Android (outils en ligne de commande), plateforme 35, build-tools | compiler l'appli | Android Studio, ou `cmdline-tools` + `sdkmanager` |
| NDK Android (version LTS, r27 ou plus récent) | compiler du Rust pour le téléphone | `sdkmanager "ndk;27.x.xxxxx"` |
| JDK 17 | Gradle | Temurin 17 |
| Gradle (via le wrapper `android/gradlew`) | assembler l'APK | committer le wrapper |

Versions conseillées pour l'appli : `minSdk 26` (Android 8), `targetSdk`
et `compileSdk` 35, Kotlin, une seule activité, pas de bibliothèque en
plus de `androidx.appcompat` / `androidx.webkit`. Identifiant :
`fr.arsenal.fabbot` (à confirmer avec Antoine). Nom affiché : « Arsenal ».

Le dépôt a un profil `release` très optimisé (`lto = "fat"`,
`codegen-units = 1`) : la compilation Rust est lente, mais le bot est
beaucoup plus rapide. Utilise-le pour toute mesure de vitesse et pour
l'APK donné à Antoine.

### 8.2 Les commandes (à adapter, et à recopier dans le point de reprise une fois vérifiées)

```bash
# 1. Bibliothèque Rust pour téléphone (arm64) et émulateur (x86_64)
cargo ndk -t arm64-v8a -t x86_64 -o android/app/src/main/jniLibs \
    build --release -p fab-android

# 2. Données embarquées non versionnées (textes LSS), si présentes
bash tools/fetch_cards.sh   # télécharge data/cards/sources/public_card_data.csv
mkdir -p android/app/src/main/assets/donnees
cp data/cards/sources/public_card_data.csv android/app/src/main/assets/donnees/

# 3. APK
cd android && ./gradlew assembleRelease
# → android/app/build/outputs/apk/release/app-release.apk
```

Mets ces étapes dans un script `android/construire.sh` (et
`android/construire.ps1` pour la Surface sous Windows), pour qu'Antoine
ou une autre session les lance en une commande.

### 8.3 La signature de l'APK (important)

Android refuse d'installer une mise à jour signée avec une autre clé que
la version installée. Il faudrait alors désinstaller, **ce qui efface les
sauvegardes et les decks maison du téléphone**. Donc :

- une seule clé de signature, créée une fois (`keytool -genkeypair …`),
  gardée par Antoine **hors du dépôt** (par exemple
  `C:\Users\abiot\cles\arsenal.jks`) ;
- `android/app/build.gradle.kts` lit le chemin et les mots de passe dans
  des variables d'environnement (`ARSENAL_CLE`, `ARSENAL_CLE_MDP`…) ; si
  elles manquent, il signe avec la clé de débogage et le dit clairement ;
- explique à Antoine, pas à pas, où ranger le fichier et comment le
  sauvegarder (s'il perd la clé, la prochaine mise à jour demandera de
  désinstaller).

### 8.4 Où compiler

Tu ne sais pas à l'avance où tourne ta session. Regarde au début :

- **Session dans le cloud (claude.ai/code).** Rust est installé. Le SDK
  et le NDK Android se téléchargent depuis `dl.google.com` et
  `maven.google.com` : si le réseau de la session les refuse, ne cherche
  pas à contourner. Tu peux quand même tout faire sauf l'APK : le code,
  `cargo test --workspace`, les captures à la taille du téléphone
  (8.5). L'APK sera alors construit par GitHub (jalon 7) ou sur la
  Surface.
- **Session sur la Surface d'Antoine** (Claude Code ou une session
  pilotée à distance, dans `C:\Users\abiot\fabbot`). Il faut Android
  Studio (qui installe SDK, NDK et JDK) : guide Antoine pour
  l'installation, une étape à la fois, avec les écrans qu'il va voir.
  Attention : le dossier `C:\Users\abiot\fabbot` doit rester sur `main`
  pour que `Jouer.bat` marche (consigne d'Antoine). Pour travailler sur
  ta branche sur la Surface, utilise un second dossier
  (`git worktree add ..\fabbot-android claude/android-…`) ou remets
  `main` à la fin.
- **GitHub Actions** (jalon 7) : un fichier `.github/workflows/android.yml`
  qui construit l'APK sur chaque pull request qui touche `crates/android/`
  ou `android/`, et sur `main`, et le dépose comme « artefact » à
  télécharger. Antoine n'a plus rien à installer. La clé de signature va
  dans les secrets du dépôt (GitHub → Settings → Secrets and variables →
  Actions), encodée en base64 ; c'est Antoine qui l'y met, guide-le.
  Ce workflow ne doit pas ralentir ni casser le workflow `CI` existant.

### 8.5 Captures à la taille du téléphone, sans téléphone

Comme l'interface est une page web, tu peux la voir comme sur le
téléphone avec un navigateur sans écran (Playwright, Chromium), avant
même d'avoir l'APK :

1. lancer ton serveur Android **sur PC** (un petit binaire de test dans
   `crates/android`, par exemple `cargo run -p fab-android --bin
   apercu`, qui démarre la même boucle que l'appli, avec `mobile.css` et
   `mobile.js`) ;
2. ouvrir la page dans Chromium avec la taille d'écran du téléphone
   d'Antoine (largeur et hauteur en points CSS, rapport de pixels,
   `isMobile: true`, `hasTouch: true`), en portrait et en paysage ;
3. capturer chaque écran listé en 9.1, dans chaque état important
   (partie : mon tour, défense, chaîne de combat, fin de manche ; éditeur :
   liste, carte ouverte, menu…) ;
4. ranger les captures dans `docs/android/captures/` **seulement si
   Antoine le souhaite** (elles alourdissent le dépôt), sinon les lui
   envoyer directement.

C'est une approximation fiable de la mise en page, mais pas du tactile,
des boîtes de dialogue Android, de la vitesse ni du cycle de vie : il
faut ensuite vérifier sur l'émulateur ou le vrai téléphone.

### 8.6 Émulateur et vrai téléphone

- **Émulateur** : Android Studio → Device Manager → créer un appareil de
  la même taille que le téléphone d'Antoine, image système x86_64. Les
  sessions cloud n'ont en général pas l'accélération matérielle
  nécessaire : l'émulateur se lance sur la Surface.
- **Vrai téléphone** : Antoine copie l'APK sur le téléphone (câble,
  Google Drive, mail…), l'ouvre, et autorise « Installer des applis
  inconnues » pour l'appli qui l'ouvre. Explique-lui la manœuvre la
  première fois, pour son modèle précis.
- **Captures du vrai téléphone** : avec le débogage USB activé et `adb`
  (installé avec le SDK) sur la Surface :
  `adb exec-out screencap -p > capture.png`. Sinon, Antoine fait les
  captures avec son téléphone et te les envoie.
- **Journal d'erreurs** : `adb logcat -s fab-android` si tu écris les
  messages du Rust dans le journal d'Android (par exemple avec la crate
  `android_logger`, ou en redirigeant les `println!`). Un plantage du
  Rust (`panic`) doit laisser une trace lisible : installe un
  `std::panic::set_hook` qui l'écrit dans le journal et dans
  `sauvegardes/plantages.txt`.

## 9. Les jalons

Chaque jalon finit par une pull request fusionnée dans `main` et un
message à Antoine. Ne commence pas un jalon tant que le précédent n'est
pas fusionné. Coche-les ici au fur et à mesure.

- [x] **Jalon 0 : préparation** (une session courte)
  - Poser à Antoine les questions de la section 12 et noter ses réponses
    dans ce document.
  - Inscrire la tâche dans `docs/taches.md` (pull request à part).
  - Faire et fusionner le correctif partagé de la section 5.1.
  - *Fini quand* : réponses notées, correctif fusionné, CI verte.

- [x] **Jalon 1 : la crate Rust et l'aperçu sur PC** (2026-11-28, journal
  `docs/journal/2026-11-28-android-jalon-1.md`)
  - `crates/android` : `demarrer(dossier)`, copie des decks, lecture des
    textes, reprise de la sauvegarde, ta boucle de serveur, routes
    `/android/…`, injection dans `index.html`, journal des plantages.
  - Binaire `apercu` pour PC.
  - Tests (noms en français, descriptifs) : démarrage dans un dossier
    vide, decks copiés sans écraser les decks maison, reprise d'une
    partie sauvegardée, injection des deux balises, refus d'un hôte non
    local, test des sélecteurs de `mobile.css` / `mobile.js`.
  - *Fini quand* : `cargo run -p fab-android --bin apercu` sert toute la
    page sur PC, une partie complète contre B3 se joue dans le
    navigateur, les tests passent, fmt et clippy sont propres.

- [ ] **Jalon 2 : premier APK qui démarre**
  - Projet Gradle `android/`, activité unique, WebView, appel à
    `demarrer`, bouton Retour, `domStorageEnabled`, réseau limité à
    `127.0.0.1`, icône (l'emblème `android/icone/arsenal-embleme.png`,
    voir section 12, réponse 6), nom « Arsenal », appli verrouillée en
    portrait.
  - Workflow `.github/workflows/android.yml` qui construit l'APK (avancé
    du jalon 7, section 12, réponse 4).
  - Script `android/construire.sh` / `.ps1`.
  - Captures de l'accueil en portrait et en paysage ; proposition
    d'orientation à Antoine.
  - *Fini quand* : l'APK s'installe sur un émulateur ou le téléphone,
    l'accueil s'affiche avec les fonds et le logo, une partie se lance
    et se joue jusqu'au bout (même si l'écran n'est pas encore adapté).

- [ ] **Jalon 3 : la partie jouable au doigt**
  - Adapter l'écran de partie (plateau, main, chaîne de combat, boutons,
    journal, préparation de la manche, fin de manche) dans `mobile.css`
    et `mobile.js` : carte agrandie au toucher, choix au doigt, pas de
    survol nécessaire, `confirm()` géré.
  - Reprise après que l'appli a été tuée (dans le menu développeur
    d'Android : « Ne pas conserver les activités », ou `adb shell am kill
    fr.arsenal.fabbot`).
  - *Fini quand* : un match en 2 manches gagnantes se joue entièrement
    au doigt, Annuler marche, la partie reprend après avoir tué
    l'appli, toutes les captures de la partie ont été relues (section
    7), le tableau 9.1 est rempli pour « Jouer » et « Partie ».

- [ ] **Jalon 4 : les autres écrans**
  - Decks (liste, import FaBrary, archives), éditeur de deck, ajustement
    avant la manche, didacticiel « Apprendre », conseil du bot,
    évaluation, Bot contre Bot, signalement de bug.
  - Téléchargements (pont vers Kotlin), import de fichier, presse-papiers.
  - *Fini quand* : chaque écran du tableau 9.1 a sa ligne remplie, ses
    captures relues, et tout ce qui n'est pas repris est écrit dans
    « Non repris ».

- [ ] **Jalon 5 : la vitesse du bot**
  - Mesurer sur le vrai téléphone le temps de réflexion par décision de
    chaque niveau (B1, B2, B3, réfléchi) sur au moins 3 parties, et le
    comparer à la Surface (même graine, mêmes decks).
  - Vérifier que l'appli reste fluide pendant la réflexion, que la jauge
    s'affiche, et la chauffe sur 20 minutes de jeu.
  - Proposer à Antoine un niveau par défaut sur téléphone, chiffres à
    l'appui. Il choisit.
  - *Fini quand* : chiffres écrits dans le journal du jalon et dans ce
    document, choix d'Antoine noté.

- [ ] **Jalon 6 : Entraîner et Imprimer**
  - Entraîner : tournoi, SPRT, apprentissage. Avertissement clair avant
    de lancer un calcul long (durée estimée, batterie). Savoir ce qui
    arrive si l'appli passe en arrière-plan pendant un tournoi, et le
    dire à Antoine.
  - Imprimer : `window.print()` relié à l'impression Android ; décider
    avec Antoine si l'outil Proxies de Rathe (privé, hors dépôt) doit
    être embarqué dans **son** APK à lui, ajouté au moment de la
    compilation sur sa machine.
  - *Fini quand* : les deux écrans ont leur ligne dans 9.1 et leurs
    captures relues.

- [ ] **Jalon 7 : APK construit par GitHub**
  - `.github/workflows/android.yml` : APK signé, déposé en artefact,
    sans toucher le workflow `CI`.
  - Guide pas à pas pour Antoine : récupérer l'APK sur GitHub, le mettre
    sur le téléphone, faire une mise à jour sans perdre ses données.
  - *Fini quand* : Antoine a installé lui-même une mise à jour à partir
    d'un APK construit par GitHub, et ses sauvegardes sont toujours là.

- [ ] **Jalon 8 : revue finale**
  - Toutes les captures refaites depuis `main`, relues, envoyées à
    Antoine. Tableau 9.1 complet. Liste « Non repris » à jour.
  - *Fini quand* : Antoine a joué une soirée sur son téléphone et dit
    que c'est bon.

### 9.1 Comparaison écran par écran avec la version PC

À remplir au fil des jalons. Les écrans viennent de la navigation
d'Arsenal (`crates/ui/web/commun/navigation.js`, adresses `#…`).

| Écran PC (adresse) | Ce qui marche sur Android | Ce qui marche autrement, et pourquoi | Ce qui ne marche pas | Captures relues |
|---|---|---|---|---|
| Accueil (`#accueil`) | tout s'affiche en portrait (aperçu, jalon 2) | | à vérifier sur le téléphone | portrait, aperçu PC (jalon 2) |
| Nouveau match (`#nouveau-match`) | | | | |
| Préparation de la manche : qui commence, équipement, 40 cartes | | | | |
| Partie (`#partie`) : plateau, main, chaîne de combat, journal, Annuler | | | | |
| Partie : conseil, évaluation, Bot contre Bot, signaler un bug | | | | |
| Partie : sauvegarde, export et import | | | | |
| Fin de manche et de match | | | | |
| Mes decks (`#decks`), archives (`#archives`) | | | | |
| Nouveau deck, import FaBrary (`#nouveau-deck`) | | | | |
| Éditeur de deck (`#editeur`) | | | | |
| Imprimer (`#impression`) | | | | |
| Entraîner (`#entrainer`), tournoi (`#tournoi`), apprentissage (`#apprentissage-bots`) | | | | |
| Apprendre, didacticiel (`#apprendre`) | | | | |

### 9.2 Non repris

À remplir. Une ligne par fonction de la version PC absente ou réduite
sur Android, avec la raison et ce qu'il faudrait pour la reprendre.

| Fonction | Raison | Pour la reprendre |
|---|---|---|
| | | |

## 10. Point de reprise

À mettre à jour à chaque commit important, pour qu'une autre session
reprenne sans rien demander.

- **Dernière mise à jour** : 2026-11-28 (jalon 3 : maquette envoyée à
  Antoine, en attente de son avis avant d'écrire `mobile.css` / `mobile.js`).
- **Jalon en cours** : 3. Branche `claude/vigilant-ptolemy-56q2tv` (nom
  imposé par la session).
- **Retour d'Antoine sur l'APK du jalon 2** (2026-11-28) : « ça se lance ».
  Le reste n'est pas encore adapté, c'est attendu. Il n'a pas encore dit
  s'il a joué une partie jusqu'au bout : le jalon 2 reste à cocher quand il
  l'aura confirmé.
- **Demandes d'Antoine pour le jalon 3** (2026-11-28) :
  - partie : le tapis PC ne tient pas sur un téléphone ; chaque joueur a un
    carré 2 × 2 pour l'équipement, un autre pour deck / pitch / cimetière /
    banni, et entre les deux héros et arsenal ; le bot en miroir. Les deux
    tapis, plus la chaîne de combat et la pile entre eux, tiennent sur
    l'écran. Trouver un moyen d'afficher la carte en grand ; journal derrière
    un bouton. Travailler d'abord sur une maquette aux proportions du Pixel
    9, le livrable s'adaptant à la taille réelle de l'écran ;
  - accueil : les tuiles tiennent sur l'écran sans défilement ;
  - choix des decks : tuiles longues et fines, à gauche l'illustration du
    héros recadrée sur le visage (zoom modéré, pour que les pixels ne se
    voient pas), puis le nom du deck, etc.
- **Maquette** : `docs/android/maquette-jalon-3.html` (page autonome, images
  des cartes chargées depuis LSS ; s'ouvre dans un navigateur, 7 écrans de
  412 × 851). Choix proposés à Antoine :
  - tapis : 4 groupes par joueur, cartes de 58 × 81 points : équipement
    2 × 2, colonne des armes (arme, seconde arme ou off-hand), colonne
    héros / arsenal, carré pitch / cimetière / banni / deck. La colonne des
    armes est un ajout à la proposition d'Antoine ; permanents et effets en
    cours dans la ligne d'état de chaque joueur ;
  - main en bas, puis la question et ses boutons tout en bas (sous le
    pouce) ; Annuler à gauche des boutons ;
  - toucher une carte = son action (comme le clic sur PC) ; appui long sur
    n'importe quelle carte = fiche qui monte du bas (grande carte, texte
    officiel, actions) ;
  - journal : bouton en haut à droite, tiroir depuis la droite ; la dernière
    ligne reste visible en haut de la chaîne ;
  - menu ☰ en haut à gauche : tout le contenu de la colonne de droite du PC ;
  - recadrage des visages : la carte « large » de LSS (546 px de large), un
    carré de 34 % de sa largeur, centré sur un point noté à la main pour
    chacun des 10 héros (table `VIS` de la maquette) ; héros inconnu (deck
    maison) : point par défaut (48 %, 23 %).
- **Version 2 de la maquette** (retour d'Antoine du 2026-11-28) :
  - tuiles de decks plus fines (50 points) ; « réglé pour le bot » devient
    un symbole (tête de robot verte avec une coche) ; l'illustration est
    fondue en dégradé sur la gauche de la tuile, plus zoomée (carte affichée
    à 420 points de large) et centrée sur les yeux (table `YEUX` de la
    maquette, notée à la main pour les 10 héros) ; le compte du card-pool
    n'apparaît que s'il n'est pas complet ;
  - tapis : le héros au centre, une zone d'arme de chaque côté (la seconde
    s'appelle « Off-hand » si c'en est une), l'arsenal dessous ; cartes de
    53 × 74 points, 6 points de marge de chaque côté ;
  - main décalée à gauche ; à droite, les permanents (6 places, « +N » ouvre
    la liste), les exemplaires identiques regroupés avec « ×N » (penser aux
    illusionnistes). Pour le bot : une ligne de permanents au-dessus de son
    tapis, sa main (dos) à droite. Effets en cours dans la ligne d'état de
    chaque joueur.
- **Reste** : avis d'Antoine sur la version 2, puis `mobile.css` /
  `mobile.js` (accueil, choix des decks, partie), test des sélecteurs à
  compléter, captures relues, reprise après que l'appli a été tuée, tableau
  9.1. Puis jalons 4 à 8.
- **Lien de l'APK pour le téléphone** (depuis `main`, Release
  `arsenal-dernier`, section 5.3) :
  `https://github.com/triangledeau/fabbot/releases/download/arsenal-dernier/arsenal.apk`.
- **Signature et dos de carte** : voir le journal du jalon 2 (secrets
  `ARSENAL_CLE_BASE64`, `ARSENAL_CLE_MDP`, `ARSENAL_DOS_DE_CARTE`). Tant que
  la clé n'est pas dans les secrets, chaque nouvel APK demande de
  désinstaller l'ancien.
- **Prochaine commande** : aperçu sur PC `cargo run -p fab-android --bin
  apercu`, puis captures au format du Pixel 9 (412 × 851, rapport 2,625).
- **Questions en attente pour Antoine** : avis sur la maquette (voir
  ci-dessus) ; a-t-il joué une partie jusqu'au bout avec l'APK du jalon 2 ?
- **Bugs de règle ou de carte relevés** : aucun.

## 11. Message pour lancer la session suivante

À la fin de chaque jalon, donne à Antoine ce message rempli :

> Projet fabbot, appli Android. Lis `docs/android.md` en entier (c'est
> ton brief), puis `AGENTS.md`, `CLAUDE.md` et `docs/taches.md`. Pars de
> `main` à jour. Tâche : jalon <N> de `docs/android.md` (<titre>). Le
> point de reprise est en section 10.

## 12. Questions à poser à Antoine au jalon 0

1. Quel est le modèle exact de ton téléphone, et quelle version
   d'Android (Réglages → À propos du téléphone) ? Cela donne la taille
   d'écran pour les captures et la vitesse attendue du bot.
2. Préfères-tu jouer en paysage, en portrait, ou les deux ?
3. Comment veux-tu mettre l'APK sur le téléphone : câble USB, Google
   Drive, mail, autre ?
4. Est-ce que la session tourne sur ta Surface ou dans le cloud ? (Tu
   peux aussi le voir toi-même.) Es-tu prêt à installer Android Studio
   sur la Surface, ou préfères-tu que GitHub construise l'APK pour toi
   (jalon 7, à avancer si besoin) ?
5. Veux-tu embarquer le vrai dos de carte et l'outil Proxies de Rathe
   dans ton APK à toi (ajoutés au moment de la compilation, jamais dans
   le dépôt) ?
6. Identifiant de l'appli `fr.arsenal.fabbot`, nom « Arsenal », icône
   = le logo Arsenal : ça te va ?

### Réponses d'Antoine

1. **Téléphone** (2026-11-28) : Google **Pixel 9**. Écran de 1 080 × 2 424
   pixels ; en taille d'affichage par défaut, cela fait environ 412 × 923
   points CSS avec un rapport de pixels de 2,625 (à confirmer sur le
   téléphone : la taille d'affichage se règle dans Réglages → Écran). Le
   Pixel 9 est sorti sous Android 14, donc bien au-dessus de `minSdk 26`.
   Version d'Android : **Android 17** (donnée le 2026-11-28).
2. **Orientation** (2026-11-28) : **portrait**, pour toute l'appli, partie
   comprise. Le plateau doit donc tenir dans environ 412 points CSS de
   large : c'est la contrainte principale de `mobile.css` (jalon 3). Au
   jalon 2, verrouiller l'activité en portrait au lieu de proposer le
   paysage.
3. **Transfert de l'APK** (2026-11-28) : **téléchargé depuis GitHub**,
   directement sur le téléphone.
4. **Où construire l'APK** (2026-11-28) : **par GitHub**. Pas d'Android
   Studio sur la Surface. Conséquence : le workflow
   `.github/workflows/android.yml` (jalon 7) passe **au jalon 2**, c'est
   lui qui fabrique le premier APK. La clé de signature et le dos de carte
   vont dans les secrets du dépôt (voir 5). Sans émulateur, les essais
   « dans l'appli réelle » (section 7, règle 6) se font sur le Pixel 9
   d'Antoine, avec ses captures.
5. **Dos de carte et Proxies de Rathe** (2026-11-28) : **le vrai dos de
   carte oui**, l'outil Proxies de Rathe **non** (pas besoin de
   l'impression complète sur le téléphone ; à noter dans « Non repris »
   au jalon 6). Le dos de carte ne doit pas entrer dans le dépôt : comme
   l'APK est construit par GitHub, il passe par un secret du dépôt (fichier
   encodé en base64, par exemple `ARSENAL_DOS_DE_CARTE`), qu'Antoine
   ajoute lui-même, guidé pas à pas. Sans ce secret, l'appli dessine le dos
   de remplacement, comme sur PC.
6. **Identifiant, nom, icône** (2026-11-28) : `fr.arsenal.fabbot` et
   « Arsenal » acceptés. Icône : partir du logo Arsenal, mais le
   **retravailler** pour en faire une icône. Le logo est un long bandeau
   (1 200 × 531) avec le mot « ARSENAL » : illisible réduit à une icône
   ronde. Il faut un emblème carré dans le même style (par exemple le
   médaillon rond avec la gemme rouge et un « A »). Antoine choisit qui le
   dessine (Claude ou Gemini) ; l'intégration dans Android (icône
   adaptative, toutes les tailles) est faite au jalon 2.
   **Fait le 2026-11-28** : Antoine a fourni un emblème dessiné par
   Gemini (médaillon rond, « A » en métal, gemmes rouges) sur fond vert.
   Le fond vert a été retiré et l'emblème recadré au carré :
   `android/icone/arsenal-embleme.png` (1 024 × 1 024, fond transparent).
   C'est la source de l'icône adaptative du jalon 2 (prévoir une marge :
   Android rogne le tour de l'icône en cercle ou en carré arrondi).
