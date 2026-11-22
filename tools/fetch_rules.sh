#!/usr/bin/env bash
# Télécharge les règles officielles (Comprehensive Rules + Tournament Rules)
# depuis rules.fabtcg.com et les convertit en texte lisible.
# Les fichiers vont dans docs/rules/sources/ (non versionné : on ne republie
# pas les documents de LSS, on les retélécharge quand on en a besoin).
#
# Usage : bash tools/fetch_rules.sh
set -euo pipefail
cd "$(dirname "$0")/.."
out=docs/rules/sources
mkdir -p "$out"
pages=(
  cr cr/01-game-concepts cr/02-object-properties cr/03-zones
  cr/04-game-structure cr/05-layers-cards-abilities cr/06-effects
  cr/07-combat cr/08-keywords cr/09-additional-rules cr/glossary
  trp/01-tournament-information trp/03-tournament-logistics
  trp/05-gameplay-logistics trp/07-constructed-formats
)
for p in "${pages[@]}"; do
  f="$out/$(echo "$p" | tr / _).html"
  curl -sS -L --fail -o "$f" "https://rules.fabtcg.com/en/$p/"
  echo "ok  $p"
done
python3 -I tools/html_to_text.py "$out"/*.html
date -u +"Téléchargé le %Y-%m-%d %H:%M UTC" > "$out/DATE.txt"
echo "Textes disponibles dans $out/*.txt"
