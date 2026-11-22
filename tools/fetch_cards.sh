#!/usr/bin/env bash
# Télécharge la base communautaire des cartes (the-fab-cube/flesh-and-blood-cards)
# dans data/cards/sources/ (non versionné). Voir docs/cards/README.md.
set -euo pipefail
cd "$(dirname "$0")/.."
out=data/cards/sources
mkdir -p "$out"
curl -sS -L --fail -o "$out/card.json" \
  https://raw.githubusercontent.com/the-fab-cube/flesh-and-blood-cards/develop/json/english/card.json
date -u +"Téléchargé le %Y-%m-%d %H:%M UTC" > "$out/DATE.txt"
echo "Base téléchargée : $out/card.json"
