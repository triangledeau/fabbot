@echo off
rem fabbot : supprime la programmation des travaux de nuit.
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Unregister-ScheduledTask -TaskName 'fabbot nuit' -Confirm:$false"
if errorlevel 1 (echo [aucune tache "fabbot nuit" a supprimer]) else (echo Travaux de nuit deprogrammes.)
pause
