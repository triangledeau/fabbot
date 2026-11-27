# 2026-11-27 — Toutes les cartes Silver Age dans l'éditeur

Agent : Claude Code. Branche : `claude/project-thread-dp1xpr`.

## Constat
L'éditeur ne montrait sur le PC du propriétaire que les cartes codées : la
collection complète dépendait de `data/cards/sources/fabcube_card.json`
(base communautaire), que `Jouer.bat` ne télécharge pas. L'Arsenal JS
lisait le même genre de fichier (`cards.json`, local). Écart oublié lors de
la reprise de l'interface (ajouté : E16 de `docs/rapatriement-interface.md`).

## Fait
- `tools/collection_silver_age.py` : table `data/cards/collection.tsv`
  (versionnée, 360 Ko, sans le texte des cartes) depuis le seul fichier
  officiel LSS. Légalité TRP 7.4 d'après `config/silver_age.toml` : imprimée
  au moins une fois en basic / common / rare, héros Young, hors bannies,
  hors héros mis sur la touche, hors jetons. **4 045 cartes, 2 166 noms.**
- Image : VF non foil d'illustration standard (artiste le plus fréquent des
  impressions ordinaires), sinon EN bord noir (hors Unlimited `U-` / `R-`),
  puis replis. **3 401 images françaises**, 644 anglaises, 0 sans image.
  Les images restent chargées en ligne (≈ 4 000 images, plusieurs centaines
  de Mo : pas embarquées) ; le navigateur les garde en cache.
- `api_decks.rs` lit la table (plus de base communautaire) ; le texte vient
  du fichier officiel local s'il est là. `images.rs` prend l'image de la
  table en priorité (même image dans l'éditeur et en partie).
- CR 8.3.7 (spécialisation) : vérifiée dans `verifier` et dans le filtre
  « pour ce héros » de l'éditeur.
- Une carte non codée s'affiche, se met dans un deck et s'enregistre ; un
  deck qui en contient n'est pas jouable contre le bot (comme avant).

## Pas repris
- Le texte des cartes n'est pas dans la table (données LSS non republiées) :
  sans `public_card_data.csv` local, pas de recherche dans le texte.
- Pour une carte dont la VF n'existe qu'en illustration alternative, l'image
  standard anglaise est préférée à la VF alternative (ex. Flying Kick bleu).
