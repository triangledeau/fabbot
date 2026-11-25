#!/usr/bin/env python3
"""Mesure d'un modèle local (LM Studio, llama.cpp…) sur le langage de cartes.

Pour chaque carte de crates/cards/cartes/warrior.cartes, le modèle reçoit la
référence du langage, les AUTRES cartes du fichier comme exemples, et le texte
officiel de la carte. Sa réponse est vérifiée par
`cargo run --release -p fab-cards --bin langage -- verifier --fichier …`.
En cas d'écart, l'erreur lui est renvoyée (3 essais au plus).

Usage (serveur LM Studio démarré, un modèle chargé) :
    python tools/bench_modele_local.py [--url http://localhost:1234/v1] [--modele NOM] [--cartes "Nom" …]
Résultat : out/bench_modele_local.json et un résumé à l'écran.
"""
import argparse
import csv
import json
import os
import subprocess
import sys
import time
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CARTES = os.path.join(ROOT, "crates", "cards", "cartes", "warrior.cartes")
CSV = os.path.join(ROOT, "data", "cards", "sources", "public_card_data.csv")
PARSE_RS = os.path.join(ROOT, "crates", "cards", "src", "langage", "parse.rs")

csv.field_size_limit(10**9)


def blocs(texte):
    """Découpe le fichier en blocs de cartes (nom -> texte)."""
    out, cur, nom = {}, [], None
    for line in texte.splitlines():
        if line.startswith("card "):
            if nom:
                out[nom] = "\n".join(cur).rstrip()
            nom = line.split('"')[1]
            cur = [line]
        elif nom is not None and not line.startswith("#"):
            cur.append(line)
    if nom:
        out[nom] = "\n".join(cur).rstrip()
    return out


def reference():
    """La documentation du format, prise en tête de parse.rs et de parse_stmt."""
    src = open(PARSE_RS, encoding="utf-8").read()
    doc = [l[4:] for l in src.splitlines() if l.startswith("//! ")]
    stmts, on = [], False
    for l in src.splitlines():
        if l.startswith("/// Les phrases du langage"):
            on = True
        elif on and l.startswith("/// "):
            stmts.append(l[4:])
        elif on:
            break
    conds = [
        "Conditions (if <condition> then <phrase>) : attacked_with_weapon_this_turn, reprise,",
        "defends_weapon_attack, target_counters_at_least N.",
    ]
    return "\n".join(doc + ["", "Phrases :"] + stmts + conds)


def officiel(nom):
    vs = {}
    with open(CSV, encoding="utf-8") as f:
        for r in csv.DictReader(f):
            if r["print_language"] == "en" and r["face_1_true_name"] == nom:
                vs.setdefault(r["face_1_true_color"], r)
    lignes = []
    for c in ["red", "yellow", "blue", ""]:
        if c in vs:
            r = vs[c]
            stats = ", ".join(
                f"{k} {r['face_1_true_' + k]}"
                for k in ["cost", "power", "defense", "life", "intellect"]
                if r["face_1_true_" + k]
            )
            lignes.append(
                f"- {c or 'sans couleur'} | type: {r['face_1_true_typebox']} | {stats}\n"
                f"  texte: {r['face_1_true_textbox']}"
            )
    return "\n".join(lignes)


def appel(url, modele, messages):
    body = json.dumps(
        {"model": modele, "messages": messages, "temperature": 0, "max_tokens": 400}
    ).encode()
    req = urllib.request.Request(
        url + "/chat/completions", body, {"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=900) as r:
        d = json.load(r)
    return d["choices"][0]["message"]["content"], d.get("usage", {})


def nettoyer(rep):
    """Garde le bloc de carte (enlève les ``` éventuels et le texte autour)."""
    lines = [l for l in rep.splitlines() if not l.strip().startswith("```")]
    start = next((i for i, l in enumerate(lines) if l.startswith("card ")), None)
    if start is None:
        return rep
    out = [lines[start]]
    for l in lines[start + 1 :]:
        if l.startswith("  ") or not l.strip():
            out.append(l)
        else:
            break
    return "\n".join(out).rstrip() + "\n"


def verifier(texte):
    path = os.path.join(ROOT, "out", "bench_carte.cartes")
    open(path, "w", encoding="utf-8").write(texte)
    p = subprocess.run(
        ["cargo", "run", "--release", "-q", "-p", "fab-cards", "--bin", "langage", "--",
         "verifier", "--fichier", path],
        cwd=ROOT, capture_output=True, text=True, encoding="utf-8",
    )
    return p.returncode == 0, (p.stdout + p.stderr).strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", default="http://localhost:1234/v1")
    ap.add_argument("--modele", default=None)
    ap.add_argument("--essais", type=int, default=3)
    ap.add_argument("--cartes", nargs="*")
    a = ap.parse_args()
    os.makedirs(os.path.join(ROOT, "out"), exist_ok=True)
    if not a.modele:
        with urllib.request.urlopen(a.url + "/models", timeout=30) as r:
            a.modele = json.load(r)["data"][0]["id"]
    print("Modèle :", a.modele)
    tout = blocs(open(CARTES, encoding="utf-8").read())
    ref = reference()
    noms = a.cartes or list(tout)
    res = []
    for nom in noms:
        exemples = "\n\n".join(b for n, b in tout.items() if n != nom)
        system = (
            "Tu encodes des cartes Flesh and Blood dans un petit langage. Réponds UNIQUEMENT "
            "par le bloc de la carte, sans commentaire. N'utilise jamais `native` ni "
            "`native_trigger` ni `native_static` : si une phrase n'existe pas dans le "
            "langage, réponds seulement IMPOSSIBLE.\n\nFormat :\n" + ref +
            "\n\nExemples :\n\n" + exemples
        )
        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": f'Carte "{nom}" :\n{officiel(nom)}'},
        ]
        t0 = time.time()
        ok, essais, tokens, sortie = False, 0, 0, ""
        for essais in range(1, a.essais + 1):
            rep, usage = appel(a.url, a.modele, messages)
            tokens += usage.get("completion_tokens", 0)
            if "IMPOSSIBLE" in rep:
                sortie = "IMPOSSIBLE"
                break
            texte = nettoyer(rep)
            if "native" in texte:
                sortie = "native interdit"
                break
            ok, sortie = verifier(texte)
            if ok:
                break
            messages += [
                {"role": "assistant", "content": rep},
                {"role": "user", "content": "Écart avec le texte officiel :\n" + sortie +
                 "\nCorrige et renvoie seulement le bloc."},
            ]
        dt = time.time() - t0
        natif = "native" in tout[nom]
        res.append({"carte": nom, "ok": ok, "essais": essais, "secondes": round(dt, 1),
                    "tokens_sortie": tokens, "attendu_impossible": natif, "sortie": sortie[-400:]})
        print(f"{'OK ' if ok else 'NON'} {nom:28} {essais} essai(s) {dt:6.1f} s"
              f"{'  (Rust attendu)' if natif else ''}")
    json.dump(res, open(os.path.join(ROOT, "out", "bench_modele_local.json"), "w",
                        encoding="utf-8"), ensure_ascii=False, indent=1)
    faisables = [r for r in res if not r["attendu_impossible"]]
    reussies = [r for r in faisables if r["ok"]]
    temps = sum(r["secondes"] for r in res)
    print(f"\nCartes faisables dans le langage : {len(reussies)} / {len(faisables)} réussies ; "
          f"temps total {temps:.0f} s, moyenne {temps / max(1, len(res)):.0f} s par carte.")


if __name__ == "__main__":
    sys.exit(main())
