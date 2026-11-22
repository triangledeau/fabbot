"""Vérifie que chaque carte des listes data/decks/*.txt existe dans le fichier
officiel de LSS, avec la bonne couleur.

Usage : python3 tools/check_cards.py   (après bash tools/fetch_cards.sh)
"""
import csv
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CSV = os.path.join(ROOT, "data/cards/sources/public_card_data.csv")
DECKS = os.path.join(ROOT, "data/decks")

# Le fichier officiel note les cartes « meld » à deux moitiés « A||B »,
# les listes de decks les notent « A // B ».
def official_name(name):
    return name.replace(" // ", "||")

colors = {}
with open(CSV, encoding="utf-8-sig", newline="") as fh:
    for r in csv.DictReader(fh):
        if r["print_language"] != "en":
            continue
        colors.setdefault(r["face_1_true_name"], set()).add(r["face_1_true_color"])

problems = 0
checked = set()
for fn in sorted(os.listdir(DECKS)):
    if not fn.endswith(".txt"):
        continue
    for line in open(os.path.join(DECKS, fn), encoding="utf-8"):
        m = (re.match(r"Hero: (.+)", line)
             or re.match(r"\d+ (.+?) \[", line)
             or re.match(r"\d+ (.+?) \((red|yellow|blue)\)", line))
        if not m:
            continue
        name = official_name(m.group(1).strip())
        color = m.group(2) if m.re.groups == 2 else None
        if (name, color) in checked:
            continue
        checked.add((name, color))
        if name not in colors:
            print(f"ABSENTE   {fn}: {name}")
            problems += 1
        elif color and color not in colors[name]:
            print(f"COULEUR   {fn}: {name} ({color}) — couleurs officielles : {sorted(colors[name])}")
            problems += 1

print(f"{len(checked)} entrées vérifiées, {problems} problème(s).")
sys.exit(1 if problems else 0)
