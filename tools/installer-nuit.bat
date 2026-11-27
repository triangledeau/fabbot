@echo off
rem ==========================================================
rem  fabbot : programme les travaux de nuit chaque soir.
rem  Double-cliquer une fois. Par defaut : tous les soirs a 23:00,
rem  arret des travaux a 07:30.
rem  Autres heures : installer-nuit.bat -Heure 22:30 -Fin 08:00
rem  Pour arreter la programmation : desinstaller-nuit.bat
rem ==========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0installer-nuit.ps1" %*
if errorlevel 1 echo [echec : la tache n'a pas pu etre programmee]
pause
