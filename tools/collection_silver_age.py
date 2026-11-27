"""Collection Silver Age de l'éditeur de decks : toutes les cartes légales du
format, avec leur image et leur nom français.

Source : le fichier officiel LSS (`bash tools/fetch_cards.sh`), seul : la
base communautaire n'est plus nécessaire. Légalité (config/silver_age.toml,
TRP 7.4) :
  - la carte a été imprimée au moins une fois en rareté basic, common ou
    rare (n'importe quelle langue, n'importe quelle impression) ;
  - un héros doit être « Young » ; les héros mis sur la touche
    (`benched_heroes`) sont retirés ;
  - les cartes bannies (`banned`) et les jetons sont retirés.

Choix de l'image (demande du propriétaire, 2026-11-27) : version française,
bord noir, illustration standard.
  - « illustration standard » : l'artiste le plus fréquent parmi les
    impressions ordinaires de la carte (raretés basic à fabled, hors promo,
    marvel, gold) ; les illustrations alternatives et étendues (marvel) sont
    écartées ;
  - ordre : FR non foil d'illustration standard ; sinon EN non foil
    d'illustration standard, hors « Unlimited » (bord blanc, codes U- et R-) ;
    sinon FR ou EN non foil quelle que soit l'illustration ; sinon FR puis
    EN en foil ; sinon une autre langue ;
  - à égalité, une impression Silver Age d'abord, puis la plus récente du
    fichier.

Le résultat, data/cards/collection.tsv, est versionné : statistiques,
ligne de type, mots-clés et codes d'image, **sans le texte des cartes**
(le serveur lit le texte dans le fichier officiel local s'il est là).
Les images restent chargées en ligne par la page (serveur d'images LSS).

Usage : python3 -I tools/collection_silver_age.py [chemin du CSV]
Fichier produit par un programme : en cas de conflit, le refaire.
"""
import collections
import csv
import os
import re
import sys
import tomllib
import unicodedata

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CSV = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "data/cards/sources/public_card_data.csv")
OUT = os.path.join(ROOT, "data/cards/collection.tsv")
PREFIXE = "https://legendstory-production-s3-public.s3.amazonaws.com/media/cards/large/"

with open(os.path.join(ROOT, "config/silver_age.toml"), "rb") as fh:
    FORMAT = tomllib.load(fh)
RARETES = {r.lower() for r in FORMAT["legal_rarities"]}
BANNIES = set(FORMAT["banned"])
SUR_LA_TOUCHE = set(FORMAT["benched_heroes"])
# Raretés d'une impression « ordinaire » (pour trouver l'illustration standard).
# Les héros sont notés « token » dans leurs impressions de booster.
ORDINAIRES = {"basic", "common", "rare", "super-rare", "majestic", "legendary", "fabled", "token"}
# Mots-clés gardés pour les filtres de l'éditeur (en gras dans le texte LSS).
MOTS_CLES = [
    "Go again", "Dominate", "Crush", "Intimidate", "Arcane Barrier", "Blade Break",
    "Battleworn", "Temper", "Phantasm", "Spellvoid", "Opt", "Ward", "Boost", "Combo",
    "Reprise", "Blood Debt", "Clash", "Stealth", "Mark", "Legendary", "Guardwell",
    "Quell", "Overpower", "Piercing", "Spectra", "Unity", "Heave", "Amp",
]
TALENTS = {"Chaos", "Draconic", "Earth", "Elemental", "Ice", "Light", "Lightning",
           "Mystic", "Revered", "Reviled", "Royal", "Shadow"}

csv.field_size_limit(10**9)
par_carte = collections.defaultdict(list)  # (nom, couleur) -> impressions
with open(CSV, encoding="utf-8-sig", newline="") as fh:
    for r in csv.DictReader(fh):
        par_carte[(r["face_1_true_name"], r["face_1_true_color"].lower())].append(r)


def code_image(r):
    url = r["face_1_thumbnail_large"]
    if not url.startswith(PREFIXE) or not url.endswith(".webp"):
        return None
    return url[len(PREFIXE):-len(".webp")]


def blanc(r):
    """Bord blanc : « Unlimited » et « Unlimited Revised »."""
    code = r["print_id"]
    return code.startswith("U-") or code.startswith("R-")


def artiste(r):
    """Nom de l'artiste sans accents (le fichier écrit « Lâp » ou « Lập »)."""
    t = unicodedata.normalize("NFKD", r["face_1_artist"])
    return "".join(c for c in t if not unicodedata.combining(c)).strip().lower()


def choisir(rs):
    artistes = collections.Counter(artiste(r) for r in rs if r["rarity"] in ORDINAIRES)
    standard = artistes.most_common(1)[0][0] if artistes else None

    def parmi(f):
        bonnes = [r for r in rs if code_image(r) and f(r)]
        if not bonnes:
            return None
        sa = [r for r in bonnes if "Silver Age" in r["product_name"]]
        return (sa or bonnes)[-1]

    normal = lambda r: r["face_1_finish_type"] == "regular" and not blanc(r) and r["rarity"] in ORDINAIRES | {"promo"}
    std = lambda r: artiste(r) == standard and r["rarity"] in ORDINAIRES
    return (
        parmi(lambda r: r["print_language"] == "fr" and normal(r) and std(r))
        or parmi(lambda r: r["print_language"] == "en" and normal(r) and std(r))
        or parmi(lambda r: r["print_language"] == "fr" and normal(r))
        or parmi(lambda r: r["print_language"] == "en" and normal(r))
        or parmi(lambda r: r["print_language"] == "fr" and std(r))
        or parmi(lambda r: r["print_language"] == "en" and std(r))
        or parmi(lambda r: r["print_language"] in ("fr", "en"))
        or parmi(lambda r: True)
    )


def moniker(nom):
    """CR 2.7.3 : le surnom d'un nom de personne (avant la virgule)."""
    return nom.split(",")[0].strip()


lignes = []
for (nom, couleur), rs in par_carte.items():
    if not any(r["rarity"] in RARETES for r in rs):
        continue
    en = next((r for r in rs if r["print_language"] == "en"), rs[0])
    types = en["face_1_types"].split(";")
    if "Token" in types or "Token" in en["face_1_true_typebox"]:
        continue
    nom_liste = nom.replace("||", " // ")
    if nom in BANNIES or nom_liste in BANNIES:
        continue
    if "Hero" in types:
        if "Young" not in en["face_1_subtypes"].split(";") or moniker(nom) in SUR_LA_TOUCHE:
            continue
    texte = en["face_1_true_textbox"]
    gras = re.findall(r"\*\*([^*]+)\*\*", texte)
    mots = [k for k in MOTS_CLES if any(g.strip().lower().startswith(k.lower()) for g in gras)]
    if "Legendary" in mots and "Legendary" not in [g.strip() for g in gras]:
        mots.remove("Legendary")
    # CR 8.3.7 : « [HERO] Specialization ».
    spe = sorted({m.group(1) for g in gras for m in [re.fullmatch(r"(.+) Specialization", g.strip())] if m})
    # « Essence of Earth, Ice, and Lightning » : talents permis au héros.
    essences = sorted({t for g in gras if g.startswith("Essence of") for t in re.findall(r"[A-Za-z]+", g) if t in TALENTS})
    choisie = choisir(rs)
    fr = next((r for r in rs if r["print_language"] == "fr"), None)
    nom_fr = fr["face_1_name"].replace("||", " // ") if fr else ""
    lignes.append([
        nom_liste,
        couleur.capitalize(),
        en["face_1_true_pitch"], en["face_1_true_cost"], en["face_1_true_power"],
        en["face_1_true_defense"], en["face_1_true_life"], en["face_1_true_intellect"],
        en["face_1_true_typebox"].replace("||", " // "),
        ";".join(mots), ";".join(spe), ";".join(essences),
        code_image(choisie) if choisie else "",
        nom_fr,
    ])

lignes.sort(key=lambda l: (l[0], l[2]))
with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# Collection Silver Age de l'éditeur (cartes légales, hors jetons).\n")
    fh.write("# Produit par tools/collection_silver_age.py depuis le fichier officiel LSS (ne pas modifier à la main).\n")
    fh.write("# nom\tcouleur\tpitch\tcoût\tpuissance\tdéfense\tvie\tintellect\tligne de type\tmots-clés\tspécialisation\tessences\timage\tnom français\n")
    for l in lignes:
        fh.write("\t".join(x.replace("\t", " ").replace("\n", " ") for x in l) + "\n")

noms = {l[0] for l in lignes}
fr = sum(1 for l in lignes if l[12].startswith("FR_"))
sans = sum(1 for l in lignes if not l[12])
print(f"{len(lignes)} cartes ({len(noms)} noms) : {fr} images FR, {len(lignes) - fr - sans} autres, {sans} sans image")
