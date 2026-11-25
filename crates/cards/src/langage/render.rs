//! Réécriture d'une carte en anglais, pour la comparer au texte officiel
//! (colonne `true_textbox` du fichier LSS, errata compris).

use super::*;

/// Le texte d'une carte, pour la couleur n° `ci` (0 rouge, 1 jaune, 2 bleu ;
/// 0 pour une carte sans couleur), sous la même forme que [`normalize_official`].
pub fn render(file: &File, card: &CardSpec, ci: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    for item in &card.items {
        match item {
            Item::Keyword(k) => parts.push(keyword(k)),
            Item::PlayOnlyIf(PlayCond::AttackActionDefending) => parts.push(
                "Play this only if an attack action card is defending this chain link.".into(),
            ),
            Item::Resolve(lines) => parts.push(lines_text(file, card.target, lines, ci)),
            Item::Activated(a) => parts.push(activated(file, a, ci)),
            Item::Trigger {
                label,
                event,
                lines,
            } => {
                let label = match label {
                    TriggerLabel::Unity => "Unity - ",
                };
                let when = match event {
                    EventKind::DefendsWithHandCard => {
                        "When this defends together with a card from hand, "
                    }
                };
                let body = lines_text(file, None, lines, ci);
                parts.push(format!("{label}{when}{}", lower_first(&body)));
            }
            Item::NativeTrigger { text, .. } | Item::NativeStatic { text, .. } => {
                parts.push(text.clone())
            }
        }
    }
    parts.join(" ")
}

/// Le texte officiel ramené à la même forme : sans mise en forme (`**`), sans
/// texte de rappel (`_( … )_`), retours à la ligne `{br}` remplacés par des espaces.
pub fn normalize_official(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("_(") {
        out.push_str(&rest[..i]);
        match rest[i..].find(")_") {
            Some(j) => rest = &rest[i + j + 2..],
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    let out = out.replace("**", "").replace("{br}", " ");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn keyword(k: &Keyword) -> String {
    match k {
        Keyword::GoAgain => "Go again".into(),
        Keyword::Temper => "Temper".into(),
        Keyword::Battleworn => "Battleworn".into(),
        Keyword::Dominate => "Dominate".into(),
        Keyword::Specialization(h) => format!("{h} Specialization"),
    }
}

fn resources(n: u8) -> String {
    "{r}".repeat(n as usize)
}

fn activated(file: &File, a: &Activated, ci: usize) -> String {
    let mut s = String::new();
    if a.once_per_turn {
        s.push_str("Once per Turn ");
    }
    s.push_str(match a.speed {
        Speed::Action => "Action -- ",
        Speed::Instant => "Instant -- ",
    });
    let mut costs = Vec::new();
    if a.cost > 0 {
        costs.push(resources(a.cost));
    }
    if a.tap {
        costs.push("{t}".into());
    }
    if a.destroy_this {
        costs.push("Destroy this".into());
    }
    s.push_str(&costs.join(", "));
    s.push_str(": ");
    let mut body = Vec::new();
    if a.attack {
        body.push("Attack".to_string());
    }
    if !a.lines.is_empty() {
        body.push(lines_text(file, a.target, &a.lines, ci));
    }
    if let Some(ActivateCond::SharpenedSwordThisTurn) = a.only_if {
        body.push("Activate this only if you've sharpened a sword this turn.".into());
    }
    if a.go_again {
        body.push("Go again".into());
    }
    s.push_str(&body.join(" "));
    s
}

/// Les phrases d'un bloc, chacune en phrase complète (majuscule, point).
fn lines_text(file: &File, target: Option<TargetKind>, lines: &[Line], ci: usize) -> String {
    lines
        .iter()
        .map(|l| {
            let stmt = stmt(&file.stmts[l.stmt as usize], target, ci);
            match l.cond {
                None => upper_first(&stmt),
                Some(c) => format!(
                    "{}, {}",
                    cond(file.conds[c as usize], ci),
                    lower_first(&stmt)
                ),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn cond(c: Cond, ci: usize) -> String {
    match c {
        Cond::AttackedWithWeaponThisTurn => "If you've attacked with a weapon this turn".into(),
        Cond::Reprise => reprise().into(),
        Cond::TargetCountersAtLeast(v) => {
            format!("If it has {} or more +1{{p}} counters", v[ci])
        }
        Cond::DefendsWeaponAttack => "If this defends a weapon attack".into(),
    }
}

fn reprise() -> &'static str {
    "Reprise - If the defending hero has defended with a card from their hand this chain link"
}

fn target_words(t: Option<TargetKind>) -> &'static str {
    match t {
        Some(TargetKind::Attack) => "target attack",
        Some(TargetKind::WeaponAttack) => "target weapon attack",
        Some(TargetKind::SwordYouControl) => "target sword you control",
        None => "target ???",
    }
}

fn gets(g: &Gets, ci: usize) -> String {
    match g {
        Gets::Power(v) => format!("+{}{{p}}", v[ci]),
        Gets::Keyword(k) => keyword(k).to_lowercase(),
    }
}

/// Une phrase, terminée par un point (sauf texte natif, recopié tel quel).
fn stmt(s: &Stmt, target: Option<TargetKind>, ci: usize) -> String {
    match s {
        Stmt::NextAttack { filter, gets: g } => {
            let f = match filter {
                AttackFilter::Any => "",
                AttackFilter::Weapon => "weapon ",
                AttackFilter::Axe => "axe ",
                AttackFilter::Sword => "sword ",
            };
            format!("your next {f}attack this turn gets {}.", gets(g, ci))
        }
        Stmt::TargetGets {
            power,
            instead_if_reprise,
        } => {
            let mut s = format!("{} gets +{}{{p}}.", target_words(target), power[ci]);
            if let Some(m) = instead_if_reprise {
                s.push_str(&format!(" {}, instead it gets +{}{{p}}.", reprise(), m[ci]));
            }
            s
        }
        Stmt::SharpenTarget => format!("sharpen {}.", target_words(target)),
        Stmt::TargetNextAttackGets(k) => {
            format!(
                "its next attack this turn gets {}.",
                keyword(k).to_lowercase()
            )
        }
        Stmt::TargetNextAttackCostsLess(n) => format!(
            "your next attack with it this turn costs {} less to activate.",
            resources(*n)
        ),
        Stmt::CreateToken(name) => format!("create a {name} token."),
        Stmt::DealDamageAttackingHero(n) => format!("deal {n} damage to the attacking hero."),
        Stmt::GainResources(n) => format!("gain {}.", resources(*n)),
        Stmt::ThisGetsDefense(n) => format!("this gets +{n}{{d}} until end of turn."),
        Stmt::Native { text, .. } => text.clone(),
    }
}

fn upper_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
