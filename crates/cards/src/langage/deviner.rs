//! **Encodage automatique** (étage 1, sans IA) : écrit une carte dans le
//! langage de cartes à partir du texte officiel, quand chaque phrase du texte
//! correspond à un gabarit connu. Sinon, renvoie la première phrase inconnue :
//! la carte part à l'étage suivant (modèle local, puis Claude).
//!
//! Les gabarits ci-dessous sont l'inverse exact de `render.rs` : une phrase
//! encodée ici se réécrit à l'identique.

use super::officiel::Officielle;

/// Un morceau de gabarit.
#[derive(Clone, Copy)]
enum P {
    /// Texte exact.
    L(&'static str),
    /// Un nombre.
    N,
    /// Des symboles de ressource `{r}{r}…` (renvoie leur nombre).
    R,
    /// Un mot (nom de token, de héros).
    W,
    /// Un mot-clé accordé (`go again`, `dominate`).
    K,
}

use P::*;

/// Essaie un gabarit au début de `s`. Renvoie les trous remplis et la suite.
fn try_pat<'a>(pat: &[P], mut s: &'a str) -> Option<(Vec<String>, &'a str)> {
    let mut holes = Vec::new();
    for p in pat {
        match *p {
            L(t) => s = s.strip_prefix(t)?,
            N => {
                let n = s.chars().take_while(|c| c.is_ascii_digit()).count();
                if n == 0 {
                    return None;
                }
                holes.push(s[..n].to_string());
                s = &s[n..];
            }
            R => {
                let mut n = 0;
                while let Some(r) = s.strip_prefix("{r}") {
                    s = r;
                    n += 1;
                }
                if n == 0 {
                    return None;
                }
                holes.push(n.to_string());
            }
            W => {
                let n = s
                    .chars()
                    .take_while(|c| c.is_ascii_alphabetic() || *c == '\'' || *c == '-')
                    .count();
                if n == 0 {
                    return None;
                }
                holes.push(s[..n].to_string());
                s = &s[n..];
            }
            K => {
                let (k, rest) = if let Some(r) = s.strip_prefix("go again") {
                    ("go_again", r)
                } else {
                    ("dominate", s.strip_prefix("dominate")?)
                };
                holes.push(k.into());
                s = rest;
            }
        }
    }
    Some((holes, s))
}

/// Une phrase reconnue : la ligne du langage, et la cible qu'elle déclare.
struct Found {
    dsl: String,
    target: Option<&'static str>,
}

/// Gabarits de phrases (première lettre en minuscule).
fn stmt(s: &str) -> Option<(Found, &str)> {
    let filters: [(&str, &'static str); 4] = [
        ("weapon ", "weapon"),
        ("axe ", "axe"),
        ("sword ", "sword"),
        ("", "any"),
    ];
    for (w, f) in filters {
        let next = [L("your next "), L(w), L("attack this turn gets ")];
        let mut p = next.to_vec();
        p.extend([L("+"), N, L("{p}.")]);
        if let Some((h, r)) = try_pat(&p, s) {
            let dsl = format!("next_attack {f} gets power {}", h[0]);
            return Some((Found { dsl, target: None }, r));
        }
        let mut p = next.to_vec();
        p.extend([K, L(".")]);
        if let Some((h, r)) = try_pat(&p, s) {
            let dsl = format!("next_attack {f} gets {}", h[0]);
            return Some((Found { dsl, target: None }, r));
        }
    }
    let simple: [(&[P], &str, Option<&'static str>); 9] = [
        (
            &[L("target weapon attack gets +"), N, L("{p}.")],
            "target gets power {0}",
            Some("weapon_attack"),
        ),
        (
            &[L("target attack gets +"), N, L("{p}.")],
            "target gets power {0}",
            Some("attack"),
        ),
        (
            &[L("sharpen target sword you control.")],
            "sharpen target",
            Some("sword_you_control"),
        ),
        (
            &[L("its next attack this turn gets "), K, L(".")],
            "target_next_attack gets {0}",
            None,
        ),
        (
            &[
                L("your next attack with it this turn costs "),
                R,
                L(" less to activate."),
            ],
            "target_next_attack costs_less {0}",
            None,
        ),
        (&[L("create a "), W, L(" token.")], "create_token {0}", None),
        (
            &[L("deal "), N, L(" damage to the attacking hero.")],
            "deal_damage {0} attacking_hero",
            None,
        ),
        (&[L("gain "), R, L(".")], "gain_resources {0}", None),
        (
            &[L("this gets +"), N, L("{d} until end of turn.")],
            "this_gets defense {0} until_end_of_turn",
            None,
        ),
    ];
    for (p, out, target) in simple {
        if let Some((h, r)) = try_pat(p, s) {
            let mut dsl = out.to_string();
            for (i, v) in h.iter().enumerate() {
                dsl = dsl.replace(&format!("{{{i}}}"), v);
            }
            return Some((Found { dsl, target }, r));
        }
    }
    None
}

/// Gabarits de conditions « If …, ».
fn cond(s: &str) -> Option<(String, &str)> {
    let conds: [(&[P], &str); 4] = [
        (
            &[L("If you've attacked with a weapon this turn, ")],
            "attacked_with_weapon_this_turn",
        ),
        (
            &[L("Reprise - If the defending hero has defended with a card from their hand this chain link, ")],
            "reprise",
        ),
        (
            &[L("If it has "), N, L(" or more +1{p} counters, ")],
            "target_counters_at_least {0}",
        ),
        (
            &[L("If this defends a weapon attack, ")],
            "defends_weapon_attack",
        ),
    ];
    for (p, out) in conds {
        if let Some((h, r)) = try_pat(p, s) {
            let mut dsl = out.to_string();
            for (i, v) in h.iter().enumerate() {
                dsl = dsl.replace(&format!("{{{i}}}"), v);
            }
            return Some((dsl, r));
        }
    }
    None
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Les phrases d'un morceau de texte. `Err` = la première phrase inconnue.
fn sentences(
    mut s: &str,
    out: &mut Vec<String>,
    target: &mut Option<&'static str>,
) -> Result<(), String> {
    let mut owned;
    loop {
        s = s.trim_start();
        if s.is_empty() {
            return Ok(());
        }
        // « Reprise - …, instead it gets +N{p}. » : modifie la phrase précédente.
        if let Some((h, r)) = try_pat(
            &[
                L("Reprise - If the defending hero has defended with a card from their hand this chain link, instead it gets +"),
                N,
                L("{p}."),
            ],
            s,
        ) {
            match out.last_mut() {
                Some(l) if l.starts_with("target gets power") && !l.contains("instead") => {
                    l.push_str(&format!(" instead_if reprise {}", h[0]));
                    s = r;
                    continue;
                }
                _ => return Err(s.to_string()),
            }
        }
        let (c, rest) = match cond(s) {
            Some((c, r)) => (Some(c), r),
            None => (None, s),
        };
        owned = lower_first(rest);
        let Some((f, r)) = stmt(&owned) else {
            return Err(s.to_string());
        };
        if let Some(t) = f.target {
            *target = Some(t);
        }
        out.push(match c {
            Some(c) => format!("if {c} then {}", f.dsl),
            None => f.dsl,
        });
        let consumed = owned.len() - r.len();
        s = &rest[consumed..];
    }
}

/// Le texte officiel découpé en lignes (`{br}`), sans mise en forme ni rappel.
pub fn lignes_officielles(text: &str) -> Vec<String> {
    text.split("{br}")
        .map(super::render::normalize_official)
        .filter(|l| !l.is_empty())
        .collect()
}

/// Le bloc de langage d'une carte pour UNE couleur (les nombres seuls).
fn une_couleur(o: &Officielle) -> Result<Vec<String>, String> {
    let mut items: Vec<String> = Vec::new();
    let mut card_target: Option<&'static str> = None;
    let mut in_resolve = false;
    for line in lignes_officielles(&o.textbox) {
        let l = line.as_str();
        // Mots-clés seuls sur leur ligne.
        let kw = match l {
            "Go again" => Some("go_again".to_string()),
            "Temper" => Some("temper".into()),
            "Battleworn" => Some("battleworn".into()),
            "Dominate" => Some("dominate".into()),
            _ => l
                .strip_suffix(" Specialization")
                .filter(|h| !h.contains(' '))
                .map(|h| format!("specialization {h}")),
        };
        if let Some(k) = kw {
            items.push(format!("  keyword {k}"));
            in_resolve = false;
            continue;
        }
        if l == "Play this only if an attack action card is defending this chain link." {
            items.push("  play_only_if attack_action_defending".into());
            in_resolve = false;
            continue;
        }
        if let Some(rest) =
            l.strip_prefix("Unity - When this defends together with a card from hand, ")
        {
            let mut out = Vec::new();
            let mut t = None;
            sentences(rest, &mut out, &mut t)?;
            items.push("  trigger unity defends_with_hand_card".into());
            items.extend(out.into_iter().map(|x| format!("    {x}")));
            in_resolve = false;
            continue;
        }
        // Capacité activée.
        let act = [
            ("Once per Turn Action -- ", "action", true),
            ("Action -- ", "action", false),
            ("Instant -- ", "instant", false),
        ]
        .into_iter()
        .find_map(|(p, sp, once)| l.strip_prefix(p).map(|r| (r, sp, once)));
        if let Some((rest, speed, once)) = act {
            let (costs, body) = rest.split_once(": ").ok_or_else(|| l.to_string())?;
            let mut head = format!("  activated {speed}");
            for c in costs.split(", ") {
                if c == "{t}" {
                    head.push_str(" tap");
                } else if c == "Destroy this" {
                    head.push_str(" destroy_this");
                } else if let Some((h, "")) = try_pat(&[R], c) {
                    head.push_str(&format!(" cost {}", h[0]));
                } else {
                    return Err(l.to_string());
                }
            }
            if once {
                head.push_str(" once_per_turn");
            }
            let mut body = body.to_string();
            if let Some(b) = body.strip_suffix(" Go again") {
                body = b.to_string();
                head.push_str(" go_again");
            }
            if let Some(b) =
                body.strip_suffix(" Activate this only if you've sharpened a sword this turn.")
            {
                body = b.to_string();
                head.push_str(" only_if sharpened_sword_this_turn");
            }
            if body == "Attack" {
                head.push_str(" attack");
                body.clear();
            }
            let mut out = Vec::new();
            let mut t = None;
            sentences(&body, &mut out, &mut t)?;
            if let Some(t) = t {
                head.push_str(&format!(" target {t}"));
            }
            items.push(head);
            items.extend(out.into_iter().map(|x| format!("    {x}")));
            in_resolve = false;
            continue;
        }
        // Sinon : effets à la résolution.
        let mut out = Vec::new();
        // « Reprise - … instead it gets » sur sa propre ligne : rattaché à la
        // phrase d'avant.
        if in_resolve && l.starts_with("Reprise - ") && l.contains(", instead it gets") {
            if let Some(last) = items.last_mut() {
                let mut prev = vec![last.trim_start().to_string()];
                sentences(l, &mut prev, &mut card_target)?;
                *last = format!("    {}", prev[0]);
                continue;
            }
        }
        sentences(l, &mut out, &mut card_target)?;
        if !in_resolve {
            items.push("  resolve".into());
            in_resolve = true;
        }
        if out.is_empty() {
            continue;
        }
        items.extend(out.into_iter().map(|x| format!("    {x}")));
    }
    // La cible se déclare avant les capacités.
    if let Some(t) = card_target {
        let pos = items.iter().position(|i| i == "  resolve").unwrap_or(0);
        items.insert(pos, format!("  target {t}"));
    }
    Ok(items)
}

/// Fusionne les versions d'une carte : les nombres qui changent avec la
/// couleur deviennent `rouge/jaune/bleu`.
fn fusionner(versions: &[Vec<String>]) -> Result<Vec<String>, String> {
    let first = &versions[0];
    if versions.iter().any(|v| v.len() != first.len()) {
        return Err("le texte change de forme selon la couleur".into());
    }
    let mut out = Vec::new();
    for (i, line) in first.iter().enumerate() {
        let toks: Vec<Vec<&str>> = versions.iter().map(|v| v[i].split(' ').collect()).collect();
        if toks.iter().any(|t| t.len() != toks[0].len()) {
            return Err(format!(
                "le texte change de forme selon la couleur : {line}"
            ));
        }
        let mut merged = Vec::new();
        for j in 0..toks[0].len() {
            let vals: Vec<&str> = toks.iter().map(|t| t[j]).collect();
            if vals.iter().all(|v| *v == vals[0]) {
                merged.push(vals[0].to_string());
            } else if vals.len() == 3 && vals.iter().all(|v| v.parse::<i32>().is_ok()) {
                merged.push(vals.join("/"));
            } else {
                return Err(format!("le texte change selon la couleur : {line}"));
            }
        }
        out.push(merged.join(" "));
    }
    Ok(out)
}

fn stats(o: &Officielle) -> Vec<String> {
    let mut s = Vec::new();
    for (k, v) in [
        ("cost", &o.cost),
        ("power", &o.power),
        ("defense", &o.defense),
        ("life", &o.life),
        ("intellect", &o.intellect),
    ] {
        if !v.is_empty() {
            s.push(format!("{k} {v}"));
        }
    }
    s
}

/// Écrit la carte dans le langage. `versions` : les couleurs officielles
/// (rouge, jaune, bleu dans cet ordre), ou la seule version d'une carte sans couleur.
pub fn deviner(versions: &[&Officielle]) -> Result<String, String> {
    let o = versions[0];
    let colors: Vec<&str> = versions
        .iter()
        .map(|v| v.color.as_str())
        .filter(|c| !c.is_empty())
        .collect();
    let mut head = format!("card \"{}\"", o.name);
    for c in &colors {
        head.push(' ');
        head.push_str(c);
    }
    let bodies: Vec<Vec<String>> = versions
        .iter()
        .map(|v| {
            let mut b = vec![format!("  type {}", v.typebox)];
            let st = stats(v);
            if !st.is_empty() {
                b.push(format!("  stats {}", st.join(" ")));
            }
            b.extend(une_couleur(v)?);
            Ok(b)
        })
        .collect::<Result<_, String>>()?;
    let body = if bodies.len() == 3 {
        fusionner(&bodies)?
    } else {
        bodies[0].clone()
    };
    let mut out = vec![head];
    out.extend(body);
    Ok(out.join("\n"))
}
