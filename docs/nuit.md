# Travaux de nuit sur la Surface

Chantier 0 de `docs/feuille-de-route-top8.md`. Le calcul lourd (tournois,
mesures, plus tard les épreuves et les apprentissages) tourne **la nuit
sur la Surface**, sans session Claude ouverte : il ne coûte aucun token.
Le lendemain, une session Claude lit le fichier de résultats (accès au
dossier `C:\Users\abiot\fabbot`), ou le propriétaire l'ouvre lui-même.

## Mise en place (une seule fois)

1. Le dossier `fabbot` doit être sur `main` (comme pour `Jouer.bat`).
2. Double-cliquer sur **`tools\installer-nuit.bat`**. Il crée la tâche
   planifiée Windows « fabbot nuit » : tous les soirs à 23 h 00, arrêt des
   travaux à 7 h 30. Autres heures :
   `tools\installer-nuit.bat -Heure 22:30 -Fin 08:00`.
   Pas besoin de droits administrateur.
3. Essai tout de suite (quelques minutes) : `tools\nuit.bat -Essai`. Le
   résultat est dans `resultats\nuit\AAAA-MM-JJ-essai\resume.md`.

Pour arrêter la programmation : `tools\desinstaller-nuit.bat`. Pour lancer
la liste de nuit à la main, à n'importe quelle heure : double-cliquer sur
`tools\nuit.bat`.

**Conditions** pour que la nuit tourne :
- la Surface **branchée sur secteur** (Windows ne lance pas la tâche sur
  batterie, et l'arrête si le câble est débranché) ;
- la **session Windows ouverte** (l'écran peut être verrouillé : la tâche
  tourne pour l'utilisateur connecté) ;
- le PC allumé ou en veille : la tâche demande à réveiller le PC
  (« réveil programmé »). Sur certaines Surface, le réveil depuis la veille
  ne marche pas : le plus sûr est de laisser le PC allumé, écran verrouillé.
  Pendant les travaux, le script empêche lui-même la mise en veille
  (l'écran peut s'éteindre).

## Ce que fait le script (`tools\nuit.ps1`)

1. Met le jeu à jour (`git pull`) **seulement** si le dossier est sur
   `main` et sans fichier suivi modifié ; sinon il le note et continue avec
   la version présente.
2. Compile en mode optimisé (`cargo build --release -p fab-sim --bins`).
3. Lance les travaux de la liste **un par un** (chacun utilise tous les
   threads), et note pour chacun : heure de début, durée, statut, sortie.
4. À l'heure limite, le travail en cours est arrêté et les suivants ne sont
   pas lancés (pour que le PC soit libre le matin).

Résultats dans `resultats\nuit\AAAA-MM-JJ\` (dossier ignoré par git, propre
à la machine) :
- `resume.md` : **le fichier à lire le matin** : version jouée, une
  section par travail (commande, durée, statut, fin de la sortie), puis un
  bilan ;
- `NN-nom.txt` : la sortie complète de chaque travail ;
- `00-compilation.txt` : la sortie de la compilation ;
- `travaux.txt` : la liste qui a été lancée.

Si le script est lancé deux fois le même jour, le second dossier s'appelle
`AAAA-MM-JJ-2`.

## La liste des travaux

Une ligne par travail, `nom | programme | arguments` ; les lignes qui
commencent par `#` sont ignorées. `programme` est un programme du paquet
`fab-sim` (`fab-sim`, `analyse`…).

- **Liste par défaut** : `tools\nuit\travaux.txt` (suivie par git ; une
  session Claude la change par pull request).
- **Liste du soir** : si le fichier `resultats\nuit\a-faire.txt` existe, il
  remplace la liste par défaut **pour une nuit** (il est déplacé dans le
  dossier du jour). Une session Claude peut le déposer directement dans le
  dossier de la Surface, sans pull request : c'est la façon la moins chère
  de commander un calcul pour la nuit suivante.

**Règle** : un travail de nuit n'écrit jamais dans un fichier suivi par
git (`data\b3\evaluation.txt`, `data\side\b1.txt`…), sinon la mise à jour
du lendemain est bloquée. Les apprentissages écrivent dans `resultats\`
(`--sortie resultats\nuit\poids.txt`) ; une session Claude reprend ensuite
le fichier dans une pull request si le résultat est meilleur.

Options de `nuit.bat` / `nuit.ps1` : `-Essai` (liste courte
`tools\nuit\essai.txt`), `-Fin 07:30` (heure limite), `-Discret` (priorité
basse, pour garder le PC utilisable dans la journée), `-SansMiseAJour`.

## Liste par défaut actuelle

| Travail | Pourquoi | Durée estimée (à mesurer) |
|---|---|---|
| `vitesse-b3` : B3 contre B3, 4 parties par paire | chronométrer la Surface (parties par seconde) | quelques minutes |
| `controle-b1` : B1 contre B1, 1 000 parties par paire, vérifiées | détecter une erreur de règles après une mise à jour | quelques minutes |
| `b3-contre-b2` : 16 parties par paire | suivre le niveau de B3 | ≈ 10 à 20 min |
| `matrice-b3` : B3 contre B3, 100 parties par paire | matrice des matchups (± 10 points par case) | ≈ 1 à 2 h |
