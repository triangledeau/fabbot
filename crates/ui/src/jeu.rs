//! La boucle de jeu : le moteur avance, le bot répond à ses décisions,
//! l'humain répond aux siennes en tapant un numéro.
//!
//! Lecture et écriture passent par des flux génériques, ce qui permet de
//! tester l'interface sans terminal.

use crate::textes::Textes;
use crate::vue;
use fab_ai::Agent;
use fab_engine::{DecisionKind, GameState, History, ObjectId, PlayerId};
use std::io::{self, BufRead, Write};

/// Comment la partie s'est arrêtée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fin {
    /// La partie est allée à son terme.
    Terminee,
    /// Le joueur a quitté (commande `q` ou fin de l'entrée).
    Abandon,
}

pub const AIDE: &str = "\
Commandes :
  <numéro>  choisir cette option
  a         annuler votre dernier choix (autant de fois que voulu)
  e         afficher tout l'état (cimetières et cartes bannies compris)
  t <n>     fiche et texte officiel de la carte [n]
  ?         cette aide
  q         quitter
";

pub struct Interface<'a> {
    pub humain: PlayerId,
    pub bot: &'a mut dyn Agent,
    pub textes: Option<&'a Textes>,
}

impl Interface<'_> {
    /// Joue la partie jusqu'au bout (ou jusqu'à l'abandon).
    pub fn jouer(
        &mut self,
        g: &mut GameState,
        entree: &mut impl BufRead,
        sortie: &mut impl Write,
    ) -> io::Result<Fin> {
        let h = self.humain;
        if g.record.is_none() {
            g.record = Some(Vec::new());
        }
        let mut historique: History<GameState> = History::new();
        let mut tour = 0u32;
        let mut afficher_etat = true;
        writeln!(sortie, "{AIDE}")?;
        loop {
            // 1. Ce qui s'est passé depuis la dernière fois.
            let evenements = g.record.as_mut().map(std::mem::take).unwrap_or_default();
            for e in &evenements {
                let Some(l) = vue::evenement(g, h, e) else {
                    continue;
                };
                if matches!(e, fab_engine::Event::StartOfTurn(_)) {
                    tour += 1;
                    writeln!(sortie, "\n[tour {tour}] {l}")?;
                } else {
                    writeln!(sortie, "  · {l}")?;
                }
                afficher_etat = true;
            }

            // 2. La décision en attente.
            let Some(d) = g.pending_decision().cloned() else {
                writeln!(sortie)?;
                write!(sortie, "{}", vue::etat(g, h, true))?;
                writeln!(sortie, "{}", vue::resultat(g, h))?;
                return Ok(Fin::Terminee);
            };
            if d.player != h {
                let i = self.bot.choose(g, &d);
                g.apply(i)
                    .map_err(|e| io::Error::other(format!("choix du bot refusé : {e:?}")))?;
                continue;
            }
            if afficher_etat
                && matches!(
                    d.kind,
                    DecisionKind::Priority | DecisionKind::Defend | DecisionKind::Arsenal
                )
            {
                writeln!(sortie)?;
                write!(sortie, "{}", vue::etat(g, h, false))?;
                afficher_etat = false;
            }
            write!(sortie, "{}", vue::decision(g, &d))?;

            // 3. La réponse de l'humain.
            loop {
                write!(sortie, "> ")?;
                sortie.flush()?;
                let mut ligne = String::new();
                if entree.read_line(&mut ligne)? == 0 {
                    writeln!(sortie)?;
                    return Ok(Fin::Abandon);
                }
                let ligne = ligne.trim();
                let mut mots = ligne.split_whitespace();
                match mots.next().unwrap_or("") {
                    "q" | "quitter" => return Ok(Fin::Abandon),
                    "?" | "aide" | "h" => write!(sortie, "{AIDE}")?,
                    "e" | "etat" | "état" => write!(sortie, "{}", vue::etat(g, h, true))?,
                    "t" => match mots.next().and_then(|n| n.parse::<u32>().ok()) {
                        Some(n) => self.carte(g, ObjectId(n), sortie)?,
                        None => writeln!(sortie, "Usage : t <numéro de carte entre crochets>")?,
                    },
                    "a" | "annuler" => match historique.undo() {
                        Some(avant) => {
                            *g = avant;
                            writeln!(sortie, "Choix annulé.")?;
                            afficher_etat = true;
                            break;
                        }
                        None => writeln!(sortie, "Rien à annuler.")?,
                    },
                    "" => {}
                    n => match n.parse::<usize>() {
                        Ok(n) if (1..=d.options.len()).contains(&n) => {
                            historique.record(g);
                            g.apply(n - 1).map_err(|e| {
                                io::Error::other(format!("choix refusé par le moteur : {e:?}"))
                            })?;
                            break;
                        }
                        _ => writeln!(
                            sortie,
                            "Réponse non comprise : tapez un numéro entre 1 et {}, ou « ? » pour l'aide.",
                            d.options.len()
                        )?,
                    },
                }
            }
        }
    }

    /// Commande `t` : fiche de la carte et texte officiel, si l'humain a le
    /// droit de la voir.
    fn carte(&self, g: &GameState, id: ObjectId, sortie: &mut impl Write) -> io::Result<()> {
        if id.0 as usize >= g.objects.len() {
            return writeln!(sortie, "Pas de carte [{}].", id.0);
        }
        if !vue::visible(g, self.humain, id) {
            return writeln!(sortie, "Cette carte est cachée pour vous.");
        }
        writeln!(sortie, "{}", vue::fiche(g, id))?;
        let d = g.def(id);
        match self.textes.and_then(|t| t.texte(&d.name, d.color)) {
            Some((typ, texte)) => {
                writeln!(sortie, "  {typ}")?;
                for l in texte.lines() {
                    writeln!(sortie, "  │ {l}")?;
                }
            }
            None if self.textes.is_none() => writeln!(
                sortie,
                "  (texte officiel indisponible : lancez « bash tools/fetch_cards.sh »)"
            )?,
            None => writeln!(sortie, "  (texte officiel introuvable)")?,
        }
        Ok(())
    }
}
