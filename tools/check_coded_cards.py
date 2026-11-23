"""Compare les vraies cartes codées (crates/cards/src/real) au fichier
officiel de LSS : pitch, coût, puissance, défense, vie, intellect, mots de la
ligne de type, mots-clés imprimés (go again, dominate, temper...).

Usage : python3 tools/check_coded_cards.py   (après bash tools/fetch_cards.sh)
"""
import csv
import json
import os
import re
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CSV = os.path.join(ROOT, "data/cards/sources/public_card_data.csv")

out = subprocess.run(
    ["cargo", "run", "-q", "-p", "fab-cards", "--bin", "card-dump"],
    cwd=ROOT, check=True, capture_output=True, text=True,
).stdout
coded = [json.loads(l) for l in out.splitlines() if l.strip()]

csv.field_size_limit(10**9)
official = {}
with open(CSV, encoding="utf-8-sig", newline="") as fh:
    for r in csv.DictReader(fh):
        if r["print_language"] != "en":
            continue
        key = (r["face_1_true_name"], (r["face_1_true_color"] or "").lower() or None)
        official.setdefault(key, r)


def num(v):
    v = (v or "").strip()
    # Coût « X » ou « XX » : la partie fixe vaut 0 (CR 1.12.2a), le X est
    # géré par `x_cost` / `x_twice`.
    if v and set(v) == {"X"}:
        return 0
    # Valeur « * » : définie par une capacité de la carte (ex. Bloodied Oval) ;
    # codée comme une base de 0 plus un effet statique.
    if v == "*":
        return 0
    return int(v) if v.isdigit() else None


problems = 0
for c in coded:
    # Carte double : « Arcane Seeds // Life » s'écrit « Arcane Seeds||Life ».
    name = c["name"].replace(" // ", "||")
    r = official.get((name, (c["color"] or "").lower() or None))
    if r is None:
        print(f"ABSENTE  {c['name']} {c['color']}")
        problems += 1
        continue
    for field in ["pitch", "cost", "power", "defense", "life", "intellect"]:
        expected = num(r["face_1_true_" + field])
        if c[field] != expected:
            print(f"ÉCART    {c['name']} {c['color']} {field} : codé {c[field]}, officiel {expected}")
            problems += 1
    typebox = r["face_1_true_typebox"]
    for w in c["typebox"]:
        if w not in typebox:
            print(f"TYPE     {c['name']} {c['color']} : « {w} » absent de « {typebox} »")
            problems += 1
    text = re.sub(r"_\(.*?\)_", "", r["face_1_true_textbox"])  # sans les rappels
    for k in c["keywords"]:
        if k.lower() not in text.lower():
            print(f"MOT-CLÉ  {c['name']} {c['color']} : « {k} » absent du texte")
            problems += 1
    for k in ["Go again", "Dominate", "Battleworn", "Temper", "Guardwell", "Blade Break", "Ambush", "Ephemeral", "Suspense", "Blood Debt"]:
        # « **Blood Debt ** » : certaines cartes ont une espace avant les **.
        printed = re.search(r"\*\*" + k + r"\s*\*\*\s*(\{br\}|$)", text, re.I)
        if printed and k not in c["keywords"]:
            print(f"MOT-CLÉ  {c['name']} {c['color']} : « {k} » imprimé mais pas codé")
            problems += 1
    # Crush est un label (« **Crush** - When … ») : drapeau CRUSH obligatoire.
    if "**crush** -" in text.lower() and "Crush" not in c["keywords"]:
        print(f"MOT-CLÉ  {c['name']} {c['color']} : label « Crush » imprimé mais pas codé")
        problems += 1

print(f"{len(coded)} cartes codées vérifiées, {problems} problème(s).")
sys.exit(1 if problems else 0)
