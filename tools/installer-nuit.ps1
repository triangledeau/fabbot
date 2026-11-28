# fabbot : programme tools\nuit.ps1 chaque soir (tâche planifiée Windows
# « fabbot nuit », pour l'utilisateur actuel, sans droits administrateur).
# Lancé par installer-nuit.bat. Mode d'emploi : docs\nuit.md
param([string]$Heure = "23:00", [string]$Fin = "07:30")
$ErrorActionPreference = "Stop"
$script = Join-Path $PSScriptRoot "nuit.ps1"
$action = New-ScheduledTaskAction -Execute "powershell.exe" `
    -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$script`" -Planifie -Heure $Heure -Fin $Fin" `
    -WorkingDirectory (Split-Path -Parent $PSScriptRoot)
# Trois déclencheurs : l'heure prévue, puis, parce qu'une Surface en veille
# moderne ne se réveille pas toujours à l'heure, le déverrouillage de la
# session et la sortie de veille. nuit.ps1 -Planifie ne fait rien hors de la
# plage horaire ni si la nuit a déjà été lancée.
$quand = New-ScheduledTaskTrigger -Daily -At $Heure
$espace = "ROOT\Microsoft\Windows\TaskScheduler"
$classeSession = Get-CimClass -Namespace $espace -ClassName MSFT_TaskSessionStateChangeTrigger
$deverrouillage = New-CimInstance -CimClass $classeSession -ClientOnly -Property @{
    StateChange = [uint32]8   # 8 = déverrouillage de la session
    UserId = "$env:USERDOMAIN\$env:USERNAME"
    Enabled = $true
}
$classeEvenement = Get-CimClass -Namespace $espace -ClassName MSFT_TaskEventTrigger
$filtre = "*[System[(Provider[@Name='Microsoft-Windows-Kernel-Power'] and EventID=507) or (Provider[@Name='Microsoft-Windows-Power-Troubleshooter'] and EventID=1)]]"
$reveil = New-CimInstance -CimClass $classeEvenement -ClientOnly -Property @{
    Subscription = "<QueryList><Query Id=`"0`" Path=`"System`"><Select Path=`"System`">$filtre</Select></Query></QueryList>"
    Enabled = $true
}
# Réveille le PC s'il dort ; rattrape un départ manqué ; ne démarre pas sur
# batterie (réglage Windows par défaut) ; au plus 10 heures ; jamais deux
# nuits en même temps.
$reglages = New-ScheduledTaskSettingsSet -WakeToRun -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Hours 10) -MultipleInstances IgnoreNew
$description = "fabbot : tournois et mesures de nuit (tools\nuit.ps1)"
try {
    Register-ScheduledTask -TaskName "fabbot nuit" -Action $action -Trigger @($quand, $deverrouillage, $reveil) -Settings $reglages `
        -Description $description -Force | Out-Null
} catch {
    # Sans droits administrateur, Windows peut refuser le déclencheur « réveil » :
    # on garde 23 h et le déverrouillage.
    Write-Host "Déclencheur « sortie de veille » refusé par Windows ($($_.Exception.Message)) : 23 h et déverrouillage seulement."
    Register-ScheduledTask -TaskName "fabbot nuit" -Action $action -Trigger @($quand, $deverrouillage) -Settings $reglages `
        -Description $description -Force | Out-Null
}
$info = Get-ScheduledTask -TaskName "fabbot nuit" | Get-ScheduledTaskInfo
Write-Host ("Travaux de nuit programmés chaque soir à $Heure (arrêt à $Fin). Prochain lancement : " + $info.NextRunTime)
Write-Host "Si la Surface dort à $Heure, la nuit part dès qu'elle se réveille ou qu'on la déverrouille (jusqu'à 1 h avant $Fin)."
Write-Host "Laisser la Surface branchée sur secteur et la session Windows ouverte (l'écran peut être verrouillé)."
