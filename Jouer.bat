@echo off
rem ==========================================================
rem  fabbot : double-cliquer sur ce fichier pour jouer.
rem  Il met le jeu a jour, le prepare et ouvre le navigateur.
rem ==========================================================
cd /d "%~dp0"
title fabbot

where cargo >nul 2>nul
if errorlevel 1 goto pas_de_rust

where git >nul 2>nul
if errorlevel 1 goto textes
echo Mise a jour du jeu...
git pull --ff-only -q
if errorlevel 1 echo [mise a jour impossible : on joue avec la version actuelle]

:textes
if exist "data\cards\sources\public_card_data.csv" goto lancer
echo Telechargement des textes officiels des cartes...
if not exist "data\cards\sources" mkdir "data\cards\sources"
curl -sS -L --fail -o "data\cards\sources\public_card_data.csv" https://d1pn9y8e99aays.cloudfront.net/public_card_data.csv
if errorlevel 1 echo [echec : le jeu marchera, mais sans le texte des cartes]

:lancer
echo.
echo Preparation du jeu. La premiere fois, et apres chaque mise a jour,
echo cela peut prendre quelques minutes...
cargo run --release -q -p fab-ui --bin fab-web -- %*
echo.
echo Le jeu s'est arrete.
pause
exit /b

:pas_de_rust
echo Rust n'est pas installe sur cet ordinateur.
echo Installez-le depuis https://rustup.rs puis relancez Jouer.bat.
pause
exit /b 1
