# Didacticiel « Apprendre »

Phase 2 du plan de rapatriement (`docs/rapatriement-arsenal.md`) : le
didacticiel des règles, repensé. Choix du propriétaire (2026-11-26) : public
**débutant complet**.

## Principes

- **Deux niveaux** : un *parcours* progressif (13 chapitres, une notion par
  leçon) et une *couverture* des règles officielles : le sommaire montre,
  pour chaque section des Comprehensive Rules, les leçons qui la citent.
  Une section sans leçon n'est pas encore expliquée.
- **Rien n'est écrit de mémoire** : chaque phrase de règle cite sa source
  (`CR x.y.z`). Copie locale des règles : `docs/rules/sources/` (non
  versionnée ; rules.fabtcg.com refuse les accès automatiques, la copie se
  fait avec un navigateur).
- **Scènes sur le vrai moteur**, en règles complètes : une petite partie
  dont tout est fixé (mains, ordre des decks, qui commence) et où
  l'adversaire suit un script. L'élève décide, le moteur applique la règle,
  la page vérifie que le but de la scène est atteint. Pas de règles
  réduites, donc aucun changement de `crates/engine/`.
- **Une leçon** = des étapes : explication courte, scène, question éclair
  (réponses mélangées, explication et numéro du CR après la réponse).
  Progression gardée dans le navigateur (`localStorage`).

## Fichiers

| Fichier | Rôle |
|---|---|
| `crates/ui/web/apprendre/lecons.js` | Le contenu : chapitres, leçons, étapes, sections du CR |
| `crates/ui/web/apprendre/apprendre.js`, `.css` | Le volet `#apprendre` : sommaire, leçon, scène |
| `crates/ui/src/api_apprendre.rs` | Les scènes (`SCENES`), le bot scripté, les routes `/api/apprendre/…` |

## Ajouter une leçon

1. Lire la section du CR dans `docs/rules/sources/`.
2. Si la leçon a une scène : l'ajouter à `SCENES` (`api_apprendre.rs`), avec
   un test qui joue la bonne réponse et vérifie le résultat.
3. Écrire la leçon dans `lecons.js`, avec `cr: [...]` sur chaque étape de
   règle.

## Refonte animée (2026-11-27)

Le propriétaire a rejeté la première version (notions employées avant d'être
expliquées, pas d'icônes du jeu, cartes peu montrées, pas assez animé). La
leçon se joue maintenant sur un **tapis animé** plein écran (1280×720, mis à
l'échelle) :

- les vraies cartes, en grand ; un **zoom** agrandit une carte et entoure un
  coin (anneau + trait + étiquette avec l'icône, du côté du coin) ;
- les cartes se déplacent (main → combat → cimetière), les PV défilent, les
  dégâts s'affichent ;
- les **icônes officielles** (`rules.fabtcg.com/en/assets/images/icon_*.png`,
  chargées par le navigateur, rien n'est versionné ; symbole de secours si
  le site ne répond pas) ;
- chaque mot de jeu s'écrit `{{notion}}` dans `lecons.js` : icône +
  définition au survol. Une notion doit être déclarée (`introduit: [...]`)
  avant d'être employée : le test `ordre_des_notions` le vérifie ;
- un plan de leçon = `dit` (texte du guide), `fait` (animations : `poser`,
  `zoom`, `attaquer`, `pitcher`…), `clic` (carte que l'élève doit cliquer),
  `quiz` ou `scene` (partie sur le moteur). Le quiz ne donne pas la
  solution en cas d'erreur : l'élève réessaie.

## Avancement

| Chapitre | État |
|---|---|
| 1. Le duel | 3 leçons animées |
| 2. Lire une carte | 4 leçons animées (zooms sur les coins) |
| 3. Payer | 2 leçons animées, 1 scène (payer) |
| 4. Le tapis et les zones | 4 leçons reprises de la 1re version, 1 scène (arsenal) — animation à faire |
| 5. Le tour | 5 leçons reprises de la 1re version, 1 scène (un point d'action) — animation à faire |
| 6. La chaîne de combat | 5 leçons reprises de la 1re version, 2 scènes (go again, défendre) — animation à faire |
| 7. Réactions et priorité | 1 leçon reprise (réactions de défense) |
| 8 à 12 | à écrire |
