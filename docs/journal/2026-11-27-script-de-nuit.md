# 2026-11-27 — Script de nuit pour la Surface (chantier 0)

Agent : Claude, branche `claude/project-thread-4x9bjp`.

## Fait

- `tools/nuit.ps1` (lancé par `tools/nuit.bat`) : mise à jour si le dossier
  est sur `main` sans modification, compilation, travaux un par un, sortie
  de chacun dans `resultats/nuit/AAAA-MM-JJ/NN-nom.txt`, résumé
  `resume.md`, empêche la veille, heure limite (`-Fin`), `-Essai`,
  `-Discret`, `-SansMiseAJour`.
- Liste par défaut `tools/nuit/travaux.txt` ; liste d'une nuit
  `resultats/nuit/a-faire.txt` (une session Claude peut la déposer par
  l'accès au dossier, sans pull request).
- `tools/installer-nuit.bat` / `desinstaller-nuit.bat` : tâche planifiée
  Windows « fabbot nuit » (23 h 00 par défaut, réveil du PC, pas sur
  batterie, 10 h au plus).
- `resultats/` ignoré par git. Mode d'emploi : `docs/nuit.md`.

## Essai sur la Surface (i5-1235U, 12 threads), dans un dossier à part

- `-Essai -SansMiseAJour` : compilation release 59,7 s ; B1 contre B0
  (324 parties) 1,1 s ; B3 contre B1 (81 parties, 1 par paire) 44,4 s ;
  0 erreur ; accents corrects dans `resume.md` et les sorties.
- `-Discret -Fin` (heure limite 1 minute après) : le travail B3 est arrêté
  à l'heure limite, aucun `fab-sim.exe` ne reste en marche.
- Bug trouvé et corrigé : la variable `$fin` écrasait le paramètre `-Fin`
  (PowerShell ignore la casse), la ligne « Fin : … » du résumé manquait.

## Pas fait / pas vérifié

- **Tâche planifiée non essayée** : la création de tâche Windows a été
  refusée par les permissions de la session sur la Surface. À faire par le
  propriétaire : double-clic sur `tools\installer-nuit.bat`.
- Réveil depuis la veille (`WakeToRun`) non vérifié sur la Surface.
- Priorité basse (`-Discret`) : lancée sans erreur, effet non mesuré.
- Durées de la liste par défaut : estimées, à lire dans le premier
  `resume.md`.
