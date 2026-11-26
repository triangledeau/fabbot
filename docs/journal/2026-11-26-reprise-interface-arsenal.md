# 2026-11-26 — Reprise complète de l'interface de l'Arsenal JS

- Agent : Claude (Claude Code)
- Branche : `claude/project-thread-lcawsx`
- Pull requests fusionnées : #35 (écran de partie), #39 (écran Jouer),
  #40 (fenêtre d'ajustement), #41 (éditeur de deck), #42 (accueil et
  Entraîner), #43 (Imprimer), et la dernière (finitions de l'éditeur, ce
  journal).

## Pourquoi

Le propriétaire a lancé l'appli après la phase 1 du rapatriement :
l'interface n'était pas reprise en entier. Demande : être beaucoup plus
rigoureux, tout reprendre (points 1 à 9), attention au clic droit de
l'écran des decks.

## Méthode

Liste exhaustive des écarts, écran par écran et bouton par bouton, dans
les deux applis lancées, puis dans le code de l'Arsenal JS pour les menus
et raccourcis : `docs/rapatriement-interface.md` (une ligne par écart,
état ✅ / 🟡 / ❌ / ⛔). Chaque lot a été vérifié dans l'appli lancée
(Playwright, captures dans le dossier du projet, aucune erreur de page)
avant d'être fusionné.

## Fait

- Écran de partie (P1 à P32), écran Jouer (J1 à J9, menus du clic droit,
  archives, nouveau deck), fenêtre d'ajustement avant la manche (A1 à A9),
  éditeur de deck (E1 à E15), accueil illustré (H1 à H5), Entraîner en
  trois volets (T1, portraits dans T2), Imprimer (I1 : trois portées,
  outil complet Proxies de Rathe s'il est copié dans `data/outils/`).
- Corrigé au passage : la collection proposait « Arcane Seeds » et « Life »
  seuls (côtés d'une carte double) ; test ajouté.
- Tests : 854 au vert ; fmt et clippy sans avertissement.

## Pas repris (raisons dans `docs/rapatriement-interface.md`)

- J10 (revenir à la liste d'origine d'un deck du bot) : « Modifier » fait
  une copie, validé par le propriétaire.
- A3 : le side du bot contre votre deck n'est pas montré (à confirmer).
- A7 : « Réflexion » = niveau du bot, changeable avant la première manche.
- T2 : pas de « En direct » (parties qui défilent pendant un tournoi).
- P12, P18, P19, P22, P23, P29, P31, P32 : limites du moteur (une carte à
  la fois, puissance finale non gardée, plan du bot non écrit).
- Exclus par décision : fonds ArtStation, code Talishar, règles du moteur
  JS, son.
