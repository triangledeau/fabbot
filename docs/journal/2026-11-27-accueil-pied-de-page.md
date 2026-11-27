# 2026-11-27 — Accueil : pied de page retiré

- Agent : Claude, branche `claude/project-thread-gfw7zg`.
- Demande du propriétaire : la mention « Moteur fabbot · Textes officiels des cartes chargés » en bas de l'accueil ne sert à rien.
- Fait : la mention est retirée (`crates/ui/web/accueil/accueil.js`, `accueil.css`). Le pied de page ne s'affiche plus que si les textes des cartes sont absents, avec le conseil de relancer Jouer.bat avec Internet.
- Vérifié dans la page (Chromium) : rien en bas de l'accueil quand les textes sont chargés ; l'avertissement apparaît quand ils manquent.
