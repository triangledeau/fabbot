# Brief de reprise : didacticiel, chapitres 4 à 7 animés

**Projet** fabbot (https://github.com/triangledeau/fabbot), application Arsenal, bouton « Apprendre ».
**Branche à reprendre** `claude/project-thread-0p5cpk` (commit `e8d5139`, poussé, **non fusionné**). Partir de cette branche, puis ramener `main` dedans avant la PR.

## La demande d'Antoine
Aucune étape du didacticiel ne doit laisser le tapis vide. Chaque étape doit montrer visuellement ce qu'elle explique : chaque zone citée est montrée et surlignée sur le tapis. Les chapitres 1 à 3 sont faits et fusionnés (PR #85). Les chapitres 4 à 7 étaient du texte seul.

## Déjà fait sur la branche (non relu à l'écran)
- `crates/ui/web/apprendre/apprendre.js` :
  - un **plan complet des zones** (`PLAN`, clés `plan.*`). Les cartes y sont à 75 %, le placement passe par `plan.main.N`, `plan.chaine.N`, `plan.cimetiere.N` et `plan.armes` (arme à deux mains) ;
  - des **nouvelles actions** : `plan`, `voir` (met des zones en avant et pâlit les autres), `etiquette` (« Publique », « Privée », « Hors arène »), `retirer`, `cacher`, `tirer` (pioche), `sous_deck`, `frise` (phases ou étapes du combat, à droite), `note`, `flot`, `lueur` ;
  - de **nouvelles zones** : `pile` [300,262], `maillon.0/1`, et `moi.arsenal` déplacé en [1110,262] (il chevauchait la main). Les compteurs de ressources et de points d'action sont passés à gauche (left 24px).
- `apprendre.css` : styles `.ap-slot.vu/.terne/.tag`, `.ap-frise`, `.ap-note`, `.ct.lueur`.
- `lecons.js` :
  - 13 notions ajoutées (zone, équipement, arme, arsenal, cartes bannies, chaîne de combat, pile, permanent, publique, privée, arène, go again, maillon) ;
  - des aides avant `LECONS` (`DUEL`, `MAIN`, `PLAN_COMPLET`, `EQUIPE`, `PHASES`, `ETAPES`, `BRAVO_ATTAQUE`) ;
  - les **chapitres 4 à 7 entièrement réécrits**, une idée par étape, chacun avec son animation. Les règles ont été vérifiées dans `docs/rules/sources/cr.txt`, les CR sont cités.
- `crates/ui/src/api_apprendre.rs` : nouveau test `chaque_lecon_commence_par_remplir_le_tapis`. fmt et les tests de fab-ui passent.

## Bug connu, à corriger en premier
`ETAPES` est déjà déclaré dans `crates/ui/web/partie/partie.js:826`. Les deux scripts sont chargés dans la même page, donc `lecons.js` plante (« Identifier 'ETAPES' has already been declared ») et le didacticiel ne s'affiche plus. **Renommer la constante dans lecons.js** (par exemple `ETAPES_COMBAT`) ; vérifier aussi `PHASES`, `MAIN`, `DUEL`, `Z`, `TOUTES`, `EQUIPE`, `PLAN_COMPLET` (`grep` dans `crates/ui/web`).

## Reste à faire
1. Corriger le bug ci-dessus, recompiler (`cargo build -p fab-ui --bin fab-web` : les fichiers web sont intégrés à la compilation), puis relancer le serveur.
2. Capturer **chaque étape** des leçons 4-1 à 7-1 en 1366 × 720 et 1280 × 750 avec `tools/revue_didacticiel.js` (Playwright ; usage `node tools/revue_didacticiel.js <dossier> <l> <h> 4-1 4-2 …`, serveur sur le port 8091). Le script écrit « VIDE … » pour toute étape au tapis vide.
3. Examiner chaque capture une à une :
   - contraste, étiquettes en français sur fond sombre ;
   - rien de coupé ni de chevauché (frise à droite contre la main adverse, notes, étiquettes « Hors arène » trop larges…) ;
   - nom des cartes identique au scan montré ;
   - zoom `texte` sur Épines amères (cadre calibré pour une autre carte : à vérifier).
4. Vérifier les scènes 4-3, 5-2, 6-3 et 6-4 (moteur), qui utilisent la nouvelle position de l'arsenal.
5. fmt, clippy, tous les tests, puis PR vers `main` et fusion quand la vérification « verifier » est verte. Écrire l'entrée de journal (`docs/journal/2026-11-28-didacticiel-ch4-7.md`) et mettre à jour le tableau d'avancement de `docs/didacticiel.md`.
6. Livrer à Antoine avec la liste de ce qui a été vérifié et de ce qui reste imparfait.

## Exigences d'Antoine (rappel)
- Écrire en français naturel, sans tics d'IA, et ne jamais employer une notion avant de l'avoir expliquée.
- Une idée par étape ; l'élève fait lui-même les gestes.
- Utiliser les vraies cartes et les icônes officielles.
- Faire une revue de design complète par captures **avant** chaque livraison : il ne veut plus signaler les erreurs une par une.
- Ne jamais deviner une règle : se fier au CR.
