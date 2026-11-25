//! Outil du langage de cartes (voir `docs/langage-cartes.md`).
//!
//! - `cargo run -p fab-cards --bin langage -- verifier` : réécrit chaque carte
//!   des fichiers `.cartes` en anglais et la compare au texte officiel
//!   (ligne de type, valeurs imprimées, texte). Code de sortie 1 si un écart.
//! - `… -- deviner "Nom" ["Nom" …]` : encode une carte depuis le texte
//!   officiel, sans IA (étage 1), ou dit quelle phrase bloque.
//! - `… -- comparer` : encode automatiquement chaque carte des fichiers
//!   `.cartes` et compare au fichier écrit à la main.
//! - `… -- couverture [--detail N]` : part du pool Silver Age que l'étage 1
//!   encode seul, et les phrases inconnues les plus fréquentes.
//!
//! Il faut le fichier officiel : `bash tools/fetch_cards.sh`.

use fab_cards::langage::officiel::{Base, Officielle};
use fab_cards::langage::{self, deviner, CardSpec, File, Item, ProgramId};
use fab_engine::card::Color;
use std::collections::HashMap;

fn color_name(c: Color) -> &'static str {
    match c {
        Color::Red => "red",
        Color::Yellow => "yellow",
        Color::Blue => "blue",
    }
}

fn versions<'a>(base: &'a Base, card: &CardSpec) -> Vec<(usize, Option<&'a Officielle>)> {
    if card.colors.is_empty() {
        vec![(0, base.get(&card.name, ""))]
    } else {
        card.colors
            .iter()
            .map(|&c| {
                let ci = match c {
                    Color::Red => 0,
                    Color::Yellow => 1,
                    Color::Blue => 2,
                };
                (ci, base.get(&card.name, color_name(c)))
            })
            .collect()
    }
}

fn natives(card: &CardSpec, file: &File) -> usize {
    let mut n = 0;
    for it in &card.items {
        match it {
            Item::NativeTrigger { .. } | Item::NativeStatic { .. } => n += 1,
            Item::Resolve(lines) => {
                n += lines
                    .iter()
                    .filter(|l| matches!(file.stmts[l.stmt as usize], langage::Stmt::Native { .. }))
                    .count()
            }
            _ => {}
        }
    }
    n
}

fn stat(v: Option<[i16; 3]>, ci: usize) -> String {
    v.map(|a| a[ci].to_string()).unwrap_or_default()
}

fn verifier(base: &Base, file: &File) -> bool {
    let (mut ok, mut ecarts, mut avec_natif) = (0, 0, 0);
    for card in &file.cards {
        let nat = natives(card, file);
        let mut errs = Vec::new();
        for (ci, o) in versions(base, card) {
            let Some(o) = o else {
                errs.push(format!("absente du fichier officiel (couleur {ci})"));
                continue;
            };
            if o.typebox != card.typebox {
                errs.push(format!(
                    "type : « {} » au lieu de « {} »",
                    card.typebox, o.typebox
                ));
            }
            let st = card.stats;
            for (k, mine, off) in [
                ("cost", stat(st.cost, ci), &o.cost),
                ("power", stat(st.power, ci), &o.power),
                ("defense", stat(st.defense, ci), &o.defense),
                (
                    "life",
                    st.life.map(|x| x.to_string()).unwrap_or_default(),
                    &o.life,
                ),
                (
                    "intellect",
                    st.intellect.map(|x| x.to_string()).unwrap_or_default(),
                    &o.intellect,
                ),
            ] {
                if &mine != off {
                    errs.push(format!("{k} : {mine:?} au lieu de {off:?} ({})", o.color));
                }
            }
            let mine = langage::render(file, card, ci);
            let off = langage::normalize_official(&o.textbox);
            if mine != off {
                errs.push(format!(
                    "texte ({}) :\n      encodé  : {mine}\n      officiel: {off}",
                    if o.color.is_empty() { "-" } else { &o.color }
                ));
            }
        }
        let tag = if nat > 0 {
            avec_natif += 1;
            format!(" [{nat} morceau(x) Rust à relire]")
        } else {
            String::new()
        };
        if errs.is_empty() {
            ok += 1;
            println!("ok     {}{tag}", card.name);
        } else {
            ecarts += 1;
            println!("ÉCART  {}{tag}", card.name);
            for e in errs {
                println!("    {e}");
            }
        }
    }
    println!(
        "\n{} cartes : {ok} identiques au texte officiel, {ecarts} avec écart ; {avec_natif} contiennent du Rust à relire.",
        file.cards.len()
    );
    ecarts == 0
}

/// Les versions officielles d'un nom : rouge, jaune, bleu, ou la version sans couleur.
fn official_versions<'a>(base: &'a Base, name: &str) -> Vec<&'a Officielle> {
    let colored: Vec<&Officielle> = ["red", "yellow", "blue"]
        .iter()
        .filter_map(|c| base.get(name, c))
        .collect();
    if colored.is_empty() {
        base.get(name, "").into_iter().collect()
    } else {
        colored
    }
}

/// Forme comparable d'une carte : chaque élément, phrases et conditions en clair.
fn canonique(file: &File, card: &CardSpec) -> String {
    let mut s = format!(
        "{:?} {:?} {} {:?} {:?}\n",
        card.name, card.colors, card.typebox, card.target, card.stats
    );
    let lines = |ls: &[langage::Line]| -> String {
        ls.iter()
            .map(|l| {
                format!(
                    "{:?} => {:?}",
                    l.cond.map(|c| file.conds[c as usize]),
                    file.stmts[l.stmt as usize]
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    for it in &card.items {
        let line = match it {
            Item::Resolve(ls) => format!("resolve [{}]", lines(ls)),
            Item::Activated(a) => format!(
                "activated {:?} {} {} {} {} {} {} {:?} {:?} [{}]",
                a.speed,
                a.cost,
                a.tap,
                a.once_per_turn,
                a.go_again,
                a.attack,
                a.destroy_this,
                a.only_if,
                a.target,
                lines(&a.lines)
            ),
            Item::Trigger {
                label,
                event,
                lines: ls,
            } => format!("trigger {label:?} {event:?} [{}]", lines(ls)),
            other => format!("{other:?}"),
        };
        s.push_str(&line);
        s.push('\n');
    }
    s
}

fn comparer(base: &Base) {
    let file = langage::program(ProgramId::Warrior);
    let (mut same, mut diff, mut fail) = (0, 0, 0);
    for card in &file.cards {
        let vs = official_versions(base, &card.name);
        let vs: Vec<&Officielle> = if card.colors.is_empty() {
            vs
        } else {
            vs.into_iter()
                .filter(|o| card.colors.iter().any(|&c| color_name(c) == o.color))
                .collect()
        };
        match deviner::deviner(&vs) {
            Err(e) => {
                fail += 1;
                println!("à encoder  {} — phrase inconnue : « {e} »", card.name);
            }
            Ok(text) => {
                let auto = langage::parse(&text).expect("le texte deviné doit se relire");
                let a = canonique(&auto, &auto.cards[0]);
                let b = canonique(file, card);
                if a == b {
                    same += 1;
                    println!("auto = main {}", card.name);
                } else {
                    diff += 1;
                    println!("DIFFÉRENT  {}\n  auto : {a}\n  main : {b}", card.name);
                }
            }
        }
    }
    println!(
        "\n{} cartes : {same} encodées automatiquement à l'identique, {diff} différentes, {fail} à encoder par un modèle ou à la main.",
        file.cards.len()
    );
}

/// Bannies et rareté : même règle que `config/silver_age.toml` (TRP 7.4).
fn banned() -> Vec<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/silver_age.toml");
    let t = std::fs::read_to_string(p).unwrap_or_default();
    let Some(i) = t.find("banned = [") else {
        return vec![];
    };
    let end = t[i..].find(']').map(|j| i + j).unwrap_or(t.len());
    t[i..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// Remplace les nombres par N (pour regrouper les phrases).
fn gabarit(s: &str) -> String {
    let mut out = String::new();
    let mut prev_digit = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            if !prev_digit {
                out.push('N');
            }
            prev_digit = true;
        } else {
            out.push(c);
            prev_digit = false;
        }
    }
    out
}

fn couverture(base: &Base, detail: usize) {
    let banned = banned();
    let mut names: Vec<&str> = base
        .cards
        .values()
        .filter(|o| {
            let head = o.typebox.split(" - ").next().unwrap_or("");
            let is = |w: &str| head.split_whitespace().any(|x| x == w);
            if is("Token") {
                return false;
            }
            if is("Hero") {
                return o.typebox.contains("Young");
            }
            o.rarities
                .iter()
                .any(|r| r == "basic" || r == "common" || r == "rare")
                && !banned.contains(&o.name)
        })
        .map(|o| o.name.as_str())
        .collect();
    names.sort();
    names.dedup();
    let mut ok = 0;
    let mut unknown: HashMap<String, (usize, String)> = HashMap::new();
    for name in &names {
        let vs = official_versions(base, name);
        match deviner::deviner(&vs) {
            Ok(_) => ok += 1,
            Err(e) => {
                let first = e.split(". ").next().unwrap_or(&e).to_string();
                let k = gabarit(&first);
                let entry = unknown.entry(k).or_insert((0, name.to_string()));
                entry.0 += 1;
            }
        }
    }
    println!(
        "Pool Silver Age (approx.) : {} noms ; encodés sans IA par l'étage 1 : {ok} ({:.1} %).",
        names.len(),
        100.0 * ok as f64 / names.len().max(1) as f64
    );
    let mut v: Vec<_> = unknown.into_iter().collect();
    v.sort_by_key(|e| std::cmp::Reverse(e.1 .0));
    println!("\nPhrases bloquantes les plus fréquentes (prochains gabarits) :");
    for (k, (n, ex)) in v.iter().take(detail) {
        println!("{n:5}  {k}   (ex. {ex})");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(base) = Base::charger() else {
        eprintln!("Fichier officiel absent : lancer `bash tools/fetch_cards.sh`.");
        std::process::exit(2);
    };
    match args.first().map(String::as_str) {
        Some("verifier") => {
            // `--fichier F` : vérifier un autre fichier (sortie d'un modèle).
            let file: &File = match args.iter().position(|a| a == "--fichier") {
                Some(i) => {
                    let path = args.get(i + 1).expect("--fichier CHEMIN");
                    let text = std::fs::read_to_string(path).expect("fichier illisible");
                    match langage::parse(&text) {
                        Ok(f) => Box::leak(Box::new(f)),
                        Err(e) => {
                            println!("ERREUR DE LECTURE {e}");
                            std::process::exit(3);
                        }
                    }
                }
                None => langage::program(ProgramId::Warrior),
            };
            if !verifier(&base, file) {
                std::process::exit(1);
            }
        }
        Some("deviner") => {
            for name in &args[1..] {
                match deviner::deviner(&official_versions(&base, name)) {
                    Ok(t) => println!("{t}\n"),
                    Err(e) => println!("# {name} : phrase inconnue « {e} »\n"),
                }
            }
        }
        Some("comparer") => comparer(&base),
        Some("couverture") => {
            let n = args
                .iter()
                .position(|a| a == "--detail")
                .and_then(|i| args.get(i + 1))
                .and_then(|x| x.parse().ok())
                .unwrap_or(30);
            couverture(&base, n);
        }
        _ => eprintln!(
            "usage : langage verifier | deviner \"Nom\"… | comparer | couverture [--detail N]"
        ),
    }
}
