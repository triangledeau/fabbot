#!/usr/bin/env bash
# Télécharge les données des cartes dans data/cards/sources/ (non versionné).
#
# 1. SOURCE OFFICIELLE (référence) : fichier « Download Card Data » publié par
#    Legend Story Studios sur cardvault.fabtcg.com. Textes à jour, errata compris
#    (colonnes *_true_textbox).
# 2. Base communautaire the-fab-cube (complément) : donne notamment la légalité
#    par format (silver_age_legal, silver_age_banned), absente du fichier officiel.
#
# Usage : bash tools/fetch_cards.sh
set -euo pipefail
cd "$(dirname "$0")/.."
out=data/cards/sources
mkdir -p "$out"
curl -sS -L --fail -o "$out/public_card_data.csv" \
  https://d1pn9y8e99aays.cloudfront.net/public_card_data.csv
echo "ok  source officielle : $out/public_card_data.csv"
curl -sS -L --fail -o "$out/fabcube_card.json" \
  https://raw.githubusercontent.com/the-fab-cube/flesh-and-blood-cards/develop/json/english/card.json
echo "ok  base communautaire : $out/fabcube_card.json"
date -u +"Téléchargé le %Y-%m-%d %H:%M UTC" > "$out/DATE.txt"
