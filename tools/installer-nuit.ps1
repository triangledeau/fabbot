# fabbot : programme tools\nuit.ps1 chaque soir (tâche planifiée Windows
# « fabbot nuit », pour l'utilisateur actuel, sans droits administrateur).
# Lancé par installer-nuit.bat. Mode d'emploi : docs\nuit.md
param([string]$Heure = "23:00", [string]$Fin = "07:30")
$ErrorActionPreference = "Stop"
$script = Join-Path $PSScriptRoot "nuit.ps1"
$action = New-ScheduledTaskAction -Execute "powershell.exe" `
    -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Minimized -File `"$script`" -Fin $Fin" `
    -WorkingDirectory (Split-Path -Parent $PSScriptRoot)
$quand = New-ScheduledTaskTrigger -Daily -At $Heure
# Réveille le PC s'il dort ; ne démarre pas sur batterie (réglage Windows
# par défaut) ; au plus 10 heures ; jamais deux nuits en même temps.
$reglages = New-ScheduledTaskSettingsSet -WakeToRun -ExecutionTimeLimit (New-TimeSpan -Hours 10) -MultipleInstances IgnoreNew
Register-ScheduledTask -TaskName "fabbot nuit" -Action $action -Trigger $quand -Settings $reglages `
    -Description "fabbot : tournois et mesures de nuit (tools\nuit.ps1)" -Force | Out-Null
$info = Get-ScheduledTask -TaskName "fabbot nuit" | Get-ScheduledTaskInfo
Write-Host ("Travaux de nuit programmés chaque soir à $Heure (arrêt à $Fin). Prochain lancement : " + $info.NextRunTime)
Write-Host "Laisser la Surface branchée sur secteur et la session Windows ouverte (l'écran peut être verrouillé)."
