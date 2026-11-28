# 2026-11-28 — Script de nuit : départ au réveil de la Surface

- Agent : Claude (Claude Code), branche `claude/project-thread-0f6vaa`.
- Constat : première nuit (27 au 28/11) : les 6 travaux ont fini sans
  erreur, mais le départ a eu lieu à 00:52 au lieu de 23:00. Journaux
  Windows : Surface en veille moderne depuis 19:32, sur secteur, minuteurs
  de réveil autorisés ; aucun réveil à 23:00, tâche lancée lors d'un réveil
  automatique à 00:52. Journal du planificateur désactivé.
- Choix du propriétaire : pas de « veille jamais sur secteur »
  (consommation) ; une notification sur son téléphone vers 23 h, il réveille
  la Surface s'il l'a sous la main, sinon la nuit part au premier réveil.
- Fait : `tools/installer-nuit.ps1` ajoute deux déclencheurs
  (déverrouillage de session, sortie de veille : Kernel-Power 507 et
  Power-Troubleshooter 1) et `StartWhenAvailable` ; `tools/nuit.ps1
  -Planifie -Heure` ne fait rien hors de la plage 23:00 → (fin − 1 h) ni si
  la nuit a déjà été lancée (`resultats/nuit/derniere-nuit.txt`).
  `docs/nuit.md` mis à jour.
