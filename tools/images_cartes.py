"""Associe chaque vraie carte codée (crates/cards/src/real) à son image
officielle et à son nom français (lot 1 du plan de rapatriement,
docs/rapatriement-arsenal.md).

Source : le fichier officiel LSS (bash tools/fetch_cards.sh), qui donne pour
chaque impression sa langue, son nom imprimé et l'adresse de son image.
Choix de l'image, dans l'ordre :
  1. impression française normale (non foil) : la « VF bord noir » ;
  2. sinon impression anglaise normale, hors « Unlimited » (bord blanc) ;
  parmi elles, une impression Silver Age d'abord, puis la plus récente du
  fichier.
Le résultat, data/cards/images.tsv, est versionné : il ne contient que des
codes d'impression et des noms (les images restent chargées en ligne par la
page, elles ne sont pas dans le dépôt).

Usage : python3 tools/images_cartes.py   (après bash tools/fetch_cards.sh)
Fichier produit par un programme : en cas de conflit, le refaire.
"""
import csv
import json
import os
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CSV = os.path.join(ROOT, "data/cards/sources/public_card_data.csv")
OUT = os.path.join(ROOT, "data/cards/images.tsv")
PREFIXE = "https://legendstory-production-s3-public.s3.amazonaws.com/media/cards/large/"

out = subprocess.run(
    ["cargo", "run", "-q", "-p", "fab-cards", "--bin", "card-dump"],
    cwd=ROOT, check=True, capture_output=True, text=True,
).stdout
codees = [json.loads(l) for l in out.splitlines() if l.strip()]

csv.field_size_limit(10**9)
# (nom anglais, couleur) -> liste d'impressions (ordre du fichier)
impressions = {}
with open(CSV, encoding="utf-8-sig", newline="") as fh:
    for r in csv.DictReader(fh):
        cle = (r["face_1_true_name"], (r["face_1_true_color"] or "").lower())
        impressions.setdefault(cle, []).append(r)


def code_image(url):
    """`.../large/FR_SBR016.webp` -> `FR_SBR016`."""
    if not url.startswith(PREFIXE) or not url.endswith(".webp"):
        return None
    return url[len(PREFIXE):-len(".webp")]


def meilleure(rs, langue, face):
    bonnes = [
        r for r in rs
        if r["print_language"] == langue
        and r[face + "finish_type"] == "regular"
        and not r["print_id"].startswith("U-")
        and code_image(r[face + "thumbnail_large"])
    ]
    if not bonnes:
        return None
    sa = [r for r in bonnes if "Silver Age" in r["product_name"]]
    return (sa or bonnes)[-1]


lignes = []
manquantes = []
for c in codees:
    nom = c["name"].replace(" // ", "||")
    couleur = (c["color"] or "").lower()
    face = "face_1_"
    if c.get("back_of"):
        # Face arrière : on cherche la carte par sa face avant.
        rs = [r for r in impressions.get((c["back_of"], "red"), []) + impressions.get((c["back_of"], ""), [])
              if r["face_2_true_name"] == c["name"]]
        face = "face_2_"
    else:
        rs = impressions.get((nom, couleur), [])
    fr = meilleure(rs, "fr", face)
    en = meilleure(rs, "en", face)
    choisie = fr or en
    if choisie is None:
        manquantes.append(c["name"] + (f" ({c['color']})" if c["color"] else ""))
        continue
    nom_fr = fr[face + "name"] if fr else ""
    lignes.append((c["name"], c["color"] or "", code_image(choisie[face + "thumbnail_large"]), nom_fr))

lignes = sorted(set(lignes))
with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# Image officielle et nom français de chaque carte codée.\n")
    fh.write("# Produit par tools/images_cartes.py (ne pas modifier à la main).\n")
    fh.write("# nom anglais\tcouleur\tcode de l'image (FR_ = version française)\tnom français\n")
    for l in lignes:
        fh.write("\t".join(l) + "\n")

fr = sum(1 for l in lignes if l[2].startswith("FR_"))
print(f"{len(lignes)} cartes : {fr} en français, {len(lignes) - fr} en anglais, {len(manquantes)} sans image")
for m in manquantes:
    print("  sans image :", m)
sys.exit(0)
