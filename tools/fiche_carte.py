#!/usr/bin/env python3
"""Prépare la fiche compacte d'une carte à encoder (pour un agent) :
texte officiel LSS par couleur, valeurs imprimées, et les 3 cartes déjà
codées les plus proches (code Rust de crates/cards/src/real/).

Usage : python3 tools/fiche_carte.py "Nom de la carte"
"""
import csv, glob, re, sys

csv.field_size_limit(10**9)
CSV = "data/cards/sources/public_card_data.csv"
MOTS_VIDES = set("the a an of to this that you your it its if and or is are be by for from on in with".split())


def officiel(nom):
    vu = {}
    for row in csv.DictReader(open(CSV, encoding="utf-8-sig")):
        if row["print_language"] != "en" or row["face_1_true_name"] != nom:
            continue
        vu.setdefault(row["face_1_true_color"], row)
    return vu


def mots(t):
    return {m for m in re.findall(r"[a-z{}]+", t.lower()) if m not in MOTS_VIDES}


def blocs_codes():
    """(nom, commentaire, code) pour chaque carte déjà codée."""
    out = []
    for f in sorted(glob.glob("crates/cards/src/real/*.rs")):
        lignes = open(f, encoding="utf-8").read().split("\n")
        i = 0
        while i < len(lignes):
            m = re.match(r"\s*// (.+?) \((rouge|jaune|bleu|[^)]*)\) — ", lignes[i]) or re.match(
                r"\s*// (.+?) — ", lignes[i])
            if m:
                j = i
                while j < len(lignes) and "db.add(" not in lignes[j] and j - i < 80:
                    j += 1
                bloc = "\n".join(lignes[i : j + 1])
                com = " ".join(l.strip().lstrip("/") for l in lignes[i:j] if l.strip().startswith("//"))
                if j - i < 80:
                    out.append((m.group(1), com, bloc, f))
                i = j + 1
            else:
                i += 1
    return out


def main():
    nom = sys.argv[1]
    rows = officiel(nom)
    if not rows:
        sys.exit(f"absent du fichier officiel : {nom}")
    print(f"# Carte à encoder : {nom}\n")
    textes = []
    for col, r in sorted(rows.items()):
        vals = ", ".join(f"{k} {r['face_1_true_' + k]}" for k in ("pitch", "cost", "power", "defense", "life", "intellect") if r["face_1_true_" + k])
        print(f"- {col or 'sans couleur'} : {r['face_1_true_typebox']} ; {vals}")
        print(f"  texte : {r['face_1_true_textbox']}")
        if r["face_2_true_name"]:
            print(f"  face 2 « {r['face_2_true_name']} » : {r['face_2_true_typebox']} ; {r['face_2_true_textbox']}")
        textes.append(r["face_1_true_textbox"])
    cible = mots(" ".join(textes))
    scores = []
    for n, com, bloc, f in blocs_codes():
        if n == nom:
            continue
        m = mots(com)
        s = len(cible & m) / (len(cible | m) or 1)
        scores.append((s, n, bloc, f))
    scores.sort(reverse=True)
    pris = set()
    print("\n# Cartes proches déjà codées (modèles à imiter)\n")
    for s, n, bloc, f in scores:
        if n in pris:
            continue
        pris.add(n)
        print(f"## {n} ({f})\n```rust\n{bloc}\n```\n")
        if len(pris) == 3:
            break


main()
