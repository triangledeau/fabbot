//! Lecture d'un fichier `.cartes`.
//!
//! Format, ligne par ligne (`#` = commentaire) :
//! ```text
//! card "Nom" [red] [yellow] [blue]          (colonne 0)
//!   type <ligne de type officielle>          (2 espaces)
//!   stats cost 0 defense 6/5/4 …
//!   target attack | weapon_attack | sword_you_control
//!   keyword go_again | temper | battleworn | specialization <Héros>
//!   play_only_if attack_action_defending
//!   resolve
//!     <phrase>                               (4 espaces)
//!     if <condition> then <phrase>
//!   activated action|instant [cost N] [tap] [once_per_turn] [go_again]
//!             [attack] [destroy_this] [only_if <cond>] [target <cible>]
//!     <phrase>
//!   trigger unity defends_with_hand_card
//!     <phrase>
//!   native_trigger NOM "texte"  /  native_static NOM "texte"
//! ```
//! Phrases : voir [`parse_stmt`].

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ligne {} : {}", self.line, self.msg)
    }
}

type R<T> = Result<T, String>;

/// Lit tout un fichier.
pub fn parse(src: &str) -> Result<File, ParseError> {
    let mut file = File::default();
    // Bloc ouvert au niveau 4 (resolve, activated, trigger) : on y range les phrases.
    let mut open_block = false;
    for (i, raw) in src.lines().enumerate() {
        let line = i + 1;
        let err = |msg: String| ParseError { line, msg };
        let text = raw.trim_end();
        if text.trim().is_empty() || text.trim_start().starts_with('#') {
            continue;
        }
        let indent = text.len() - text.trim_start().len();
        let body = text.trim_start();
        match indent {
            0 => {
                let rest = body
                    .strip_prefix("card ")
                    .ok_or_else(|| err("une carte commence par `card \"Nom\"`".into()))?;
                let (name, rest) = quoted(rest).map_err(err)?;
                let mut colors = Vec::new();
                for w in rest.split_whitespace() {
                    colors.push(match w {
                        "red" => Color::Red,
                        "yellow" => Color::Yellow,
                        "blue" => Color::Blue,
                        _ => return Err(err(format!("couleur inconnue : {w}"))),
                    });
                }
                file.cards.push(CardSpec {
                    name,
                    colors,
                    line,
                    ..Default::default()
                });
                open_block = false;
            }
            2 => {
                let card = file
                    .cards
                    .last_mut()
                    .ok_or_else(|| err("ligne hors d'une carte".into()))?;
                open_block = parse_item(card, body).map_err(err)?;
            }
            4 => {
                if !open_block {
                    return Err(err(
                        "phrase hors d'un bloc resolve / activated / trigger".into()
                    ));
                }
                let l = parse_line(&mut file, body).map_err(err)?;
                let card = file.cards.last_mut().unwrap();
                match card.items.last_mut() {
                    Some(Item::Resolve(lines))
                    | Some(Item::Activated(Activated { lines, .. }))
                    | Some(Item::Trigger { lines, .. }) => lines.push(l),
                    _ => unreachable!(),
                }
            }
            _ => return Err(err(format!("indentation inattendue ({indent} espaces)"))),
        }
    }
    Ok(file)
}

/// Une ligne de niveau 2. Renvoie `true` si elle ouvre un bloc de phrases.
fn parse_item(card: &mut CardSpec, body: &str) -> R<bool> {
    let (head, rest) = body.split_once(' ').unwrap_or((body, ""));
    match head {
        "type" => card.typebox = rest.trim().to_string(),
        "stats" => card.stats = parse_stats(rest)?,
        "target" => card.target = Some(target_kind(rest.trim())?),
        "keyword" => card.items.push(Item::Keyword(keyword(rest.trim())?)),
        "play_only_if" => card.items.push(Item::PlayOnlyIf(match rest.trim() {
            "attack_action_defending" => PlayCond::AttackActionDefending,
            w => return Err(format!("condition de jeu inconnue : {w}")),
        })),
        "resolve" => {
            card.items.push(Item::Resolve(Vec::new()));
            return Ok(true);
        }
        "activated" => {
            card.items.push(Item::Activated(parse_activated(rest)?));
            return Ok(true);
        }
        "trigger" => {
            let mut w = rest.split_whitespace();
            let label = match w.next() {
                Some("unity") => TriggerLabel::Unity,
                x => return Err(format!("déclencheur inconnu : {x:?}")),
            };
            let event = match w.next() {
                Some("defends_with_hand_card") => EventKind::DefendsWithHandCard,
                x => return Err(format!("événement inconnu : {x:?}")),
            };
            card.items.push(Item::Trigger {
                label,
                event,
                lines: Vec::new(),
            });
            return Ok(true);
        }
        "native_trigger" | "native_static" => {
            let (name, rest) = rest.split_once(' ').ok_or("native : NOM \"texte\"")?;
            let (text, _) = quoted(rest)?;
            let name = name.to_string();
            card.items.push(if head == "native_trigger" {
                Item::NativeTrigger { name, text }
            } else {
                Item::NativeStatic { name, text }
            });
        }
        _ => return Err(format!("mot inconnu : {head}")),
    }
    Ok(false)
}

fn parse_stats(rest: &str) -> R<Stats> {
    let mut s = Stats::default();
    let w: Vec<&str> = rest.split_whitespace().collect();
    if !w.len().is_multiple_of(2) {
        return Err("stats : des paires `nom valeur`".into());
    }
    for p in w.chunks(2) {
        let v = by_color(p[1])?;
        match p[0] {
            "cost" => s.cost = Some(v),
            "power" => s.power = Some(v),
            "defense" => s.defense = Some(v),
            "life" => s.life = Some(v[0] as u8),
            "intellect" => s.intellect = Some(v[0] as u8),
            x => return Err(format!("valeur inconnue : {x}")),
        }
    }
    Ok(s)
}

fn parse_activated(rest: &str) -> R<Activated> {
    let mut w = rest.split_whitespace();
    let speed = match w.next() {
        Some("action") => Speed::Action,
        Some("instant") => Speed::Instant,
        x => return Err(format!("activated : action ou instant, pas {x:?}")),
    };
    let mut a = Activated {
        speed,
        cost: 0,
        tap: false,
        once_per_turn: false,
        go_again: false,
        attack: false,
        destroy_this: false,
        only_if: None,
        target: None,
        lines: Vec::new(),
    };
    while let Some(x) = w.next() {
        match x {
            "cost" => a.cost = num(w.next())?,
            "tap" => a.tap = true,
            "once_per_turn" => a.once_per_turn = true,
            "go_again" => a.go_again = true,
            "attack" => a.attack = true,
            "destroy_this" => a.destroy_this = true,
            "only_if" => {
                a.only_if = Some(match w.next() {
                    Some("sharpened_sword_this_turn") => ActivateCond::SharpenedSwordThisTurn,
                    x => return Err(format!("only_if inconnu : {x:?}")),
                })
            }
            "target" => a.target = Some(target_kind(w.next().unwrap_or(""))?),
            _ => return Err(format!("activated : mot inconnu {x}")),
        }
    }
    Ok(a)
}

/// Une ligne de phrase : `<phrase>` ou `if <condition> then <phrase>`.
fn parse_line(file: &mut File, body: &str) -> R<Line> {
    let (cond, stmt) = if let Some(rest) = body.strip_prefix("if ") {
        let (c, s) = rest
            .split_once(" then ")
            .ok_or("if <condition> then <phrase>")?;
        (Some(parse_cond(c.trim())?), s.trim())
    } else {
        (None, body)
    };
    let cond = cond.map(|c| {
        file.conds.push(c);
        (file.conds.len() - 1) as u16
    });
    let stmt = parse_stmt(stmt)?;
    file.stmts.push(stmt);
    Ok(Line {
        cond,
        stmt: (file.stmts.len() - 1) as u16,
    })
}

fn parse_cond(s: &str) -> R<Cond> {
    let w: Vec<&str> = s.split_whitespace().collect();
    Ok(match w.as_slice() {
        ["attacked_with_weapon_this_turn"] => Cond::AttackedWithWeaponThisTurn,
        ["reprise"] => Cond::Reprise,
        ["defends_weapon_attack"] => Cond::DefendsWeaponAttack,
        ["target_counters_at_least", v] => Cond::TargetCountersAtLeast(by_color(v)?),
        _ => return Err(format!("condition inconnue : {s}")),
    })
}

/// Les phrases du langage :
/// - `next_attack any|weapon|axe|sword gets power N` / `gets go_again`
/// - `target gets power N [instead_if reprise M]`
/// - `sharpen target`
/// - `target_next_attack gets dominate` / `target_next_attack costs_less N`
/// - `create_token NOM`
/// - `deal_damage N attacking_hero`
/// - `gain_resources N`
/// - `this_gets defense N until_end_of_turn`
/// - `native NOM "texte"`
fn parse_stmt(s: &str) -> R<Stmt> {
    if let Some(rest) = s.strip_prefix("native ") {
        let (name, rest) = rest.split_once(' ').ok_or("native NOM \"texte\"")?;
        let (text, _) = quoted(rest)?;
        return Ok(Stmt::Native {
            name: name.into(),
            text,
        });
    }
    let w: Vec<&str> = s.split_whitespace().collect();
    Ok(match w.as_slice() {
        ["next_attack", f, "gets", rest @ ..] => Stmt::NextAttack {
            filter: match *f {
                "any" => AttackFilter::Any,
                "weapon" => AttackFilter::Weapon,
                "axe" => AttackFilter::Axe,
                "sword" => AttackFilter::Sword,
                _ => return Err(format!("filtre d'attaque inconnu : {f}")),
            },
            gets: gets(rest)?,
        },
        ["target", "gets", "power", v] => Stmt::TargetGets {
            power: by_color(v)?,
            instead_if_reprise: None,
        },
        ["target", "gets", "power", v, "instead_if", "reprise", m] => Stmt::TargetGets {
            power: by_color(v)?,
            instead_if_reprise: Some(by_color(m)?),
        },
        ["sharpen", "target"] => Stmt::SharpenTarget,
        ["target_next_attack", "gets", k] => Stmt::TargetNextAttackGets(keyword(k)?),
        ["target_next_attack", "costs_less", n] => Stmt::TargetNextAttackCostsLess(num(Some(*n))?),
        ["create_token", name] => Stmt::CreateToken(name.to_string()),
        ["deal_damage", n, "attacking_hero"] => Stmt::DealDamageAttackingHero(num(Some(*n))?),
        ["gain_resources", n] => Stmt::GainResources(num(Some(*n))?),
        ["this_gets", "defense", n, "until_end_of_turn"] => {
            Stmt::ThisGetsDefense(num(Some(*n))? as i16)
        }
        _ => return Err(format!("phrase inconnue : {s}")),
    })
}

fn gets(w: &[&str]) -> R<Gets> {
    Ok(match w {
        ["power", v] => Gets::Power(by_color(v)?),
        [k] => Gets::Keyword(keyword(k)?),
        _ => return Err(format!("gets : `power N` ou un mot-clé, pas {w:?}")),
    })
}

fn keyword(s: &str) -> R<Keyword> {
    Ok(match s.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["go_again"] => Keyword::GoAgain,
        ["temper"] => Keyword::Temper,
        ["battleworn"] => Keyword::Battleworn,
        ["dominate"] => Keyword::Dominate,
        ["specialization", hero] => Keyword::Specialization(hero.to_string()),
        _ => return Err(format!("mot-clé inconnu : {s}")),
    })
}

fn target_kind(s: &str) -> R<TargetKind> {
    Ok(match s {
        "attack" => TargetKind::Attack,
        "weapon_attack" => TargetKind::WeaponAttack,
        "sword_you_control" => TargetKind::SwordYouControl,
        _ => return Err(format!("cible inconnue : {s}")),
    })
}

/// `3` ou `6/5/4` (rouge / jaune / bleu).
fn by_color(s: &str) -> R<ByColor> {
    let v: Vec<i16> = s
        .split('/')
        .map(|x| {
            x.parse::<i16>()
                .map_err(|_| format!("nombre attendu : {s}"))
        })
        .collect::<R<_>>()?;
    match v.as_slice() {
        [a] => Ok([*a; 3]),
        [a, b, c] => Ok([*a, *b, *c]),
        _ => Err(format!("une valeur ou trois (rouge/jaune/bleu) : {s}")),
    }
}

fn num(s: Option<&str>) -> R<u8> {
    let s = s.ok_or("nombre manquant")?;
    s.parse().map_err(|_| format!("nombre attendu : {s}"))
}

/// `"texte" reste` → (texte, reste).
fn quoted(s: &str) -> R<(String, &str)> {
    let s = s.trim_start();
    let rest = s
        .strip_prefix('"')
        .ok_or("texte entre guillemets attendu")?;
    let end = rest.find('"').ok_or("guillemet fermant manquant")?;
    Ok((rest[..end].to_string(), &rest[end + 1..]))
}
