# 2026-11-26 — Arsenal, lot 2 (plateau animé)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Source** : `outils-bot/src/page/finition.js` et `finition.css` de
  l'Arsenal JS (habillage seulement, réécrit autour de l'API).

## Fait

- Serveur (`crates/ui/src/web.rs`) : après chaque choix, `film` = le plateau
  après chaque pas du moteur (le choix, puis chaque réponse du bot), avec les
  événements à animer, construits avec la même règle de visibilité que l'état
  (`vue::visible`) ; `film_no` change à chaque choix et à chaque annulation ;
  chaque option porte son `action` (glisser-déposer). Aucune règle touchée.
- Page : `partie/finition.js` (FX et Film) et `finition.css` : vols de cartes,
  frappe et impact, « Bloqué », PV qui défilent, bandeaux de tour et d'étape,
  pitch, chaîne de laiton, pause « Continuer », « Passer l'animation »,
  réglages Animations / Pause gardés dans le navigateur.

## Tests

- 2 nouveaux tests (`tests/web.rs`) : un film par choix, rien de caché dans
  le film, dernier plateau du film = plateau affiché, événements connus et
  présents sur toute une partie ; chaque option a une action valide.
- Tests existants adaptés : l'annulation et la vérification « rien de caché »
  comparent l'état sans le film (le film montre le passé).
- Playwright : défense par glisser-déposer sur la chaîne, film du tour du bot
  avec pause « Continuer » puis impact (−5 PV), aucune erreur JavaScript.
