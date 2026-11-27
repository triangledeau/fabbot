@echo off
rem ==========================================================
rem  fabbot : travaux de nuit (tournois, mesures).
rem  Double-cliquer pour lancer tout de suite la liste de nuit.
rem  Pour un essai rapide (quelques minutes) : nuit.bat -Essai
rem  Resultats : resultats\nuit\AAAA-MM-JJ\resume.md
rem  Mode d'emploi : docs\nuit.md
rem ==========================================================
title fabbot - travaux de nuit (ne pas fermer)
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0nuit.ps1" %*
echo.
echo Travaux termines. Resultats dans resultats\nuit\
pause
