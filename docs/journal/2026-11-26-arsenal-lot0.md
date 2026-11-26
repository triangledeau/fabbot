# 2026-11-26 — Arsenal, lot 0 (socle)

- **Agent** : Claude (session cloud), branche `claude/project-thread-lcawsx`.
- **Plan** : `docs/rapatriement-arsenal.md` (copié du plan validé par le propriétaire).

## Fait

- Page web découpée : `crates/ui/web/index.html` (coquille) + `commun/`
  (charte Arsenal reprise d'`arsenal.css` / `gabarit.html` de l'Arsenal JS,
  navigation par `#`, logo `logo_arsenal.webp`), `accueil/` (logo, bandeau
  « Match en cours », volets Jouer / Apprendre / Entraîner, ces deux
  derniers « Bientôt »), `partie/` (l'ancien écran de `fab-web`, même code,
  rhabillé : thème sombre, laiton, polices Cinzel / Sofia Sans / IBM Plex Mono).
- Serveur : `crates/ui/src/fichiers.rs` intègre les fichiers au programme
  (`include_bytes!`) ; `Reponse.corps` devient des octets (`texte()` pour le
  lire en texte) pour servir le logo.
- Aucune règle touchée, aucun fichier de Talishar ni image d'ArtStation.

## Tests

- 3 nouveaux tests (`crates/ui/tests/web.rs`) : chaque fichier de `web/`
  est servi avec le bon type et le bon contenu ; la page ne référence que des
  fichiers servis ; chemin inconnu ou qui remonte (`..`) = 404.
- Contrôle visuel Playwright (accueil 1400 × 860 et 390 px de large sans
  défilement horizontal, fenêtre « Nouveau match », plateau en partie) :
  aucune erreur JavaScript ; le bouton Précédent ramène à l'accueil.
