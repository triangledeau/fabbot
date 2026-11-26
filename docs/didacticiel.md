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

## Avancement

| Chapitre | État |
|---|---|
| 1. Le but du jeu | 3 leçons, 1 scène |
| 2. Lire une carte | 4 leçons (cartes montrées avec leur image, sans scène) |
| 3. Le tapis et les zones | 4 leçons, 1 scène (arsenal) |
| 4. Le déroulement d'un tour | 3 leçons, 1 scène (un seul point d'action) |
| 5. Payer : coût et pitch | 3 leçons, 1 scène (payer de deux façons) |
| 6. Attaquer : la chaîne de combat | 3 leçons, 1 scène (go again) |
| 7. Défendre | 3 leçons, 1 scène (bloquer) |
| 8 à 13 | à écrire |
