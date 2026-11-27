# 2026-11-27 — Correctifs Iyslander, Hala, Ira (B1, side, B3)

Agent : Claude, branche `claude/project-thread-j807fx`. Suite de l'analyse
du 2026-11-26 (`docs/analyses/2026-11-26-iyslander-hala-ira-b3.md`, section
« Correctifs appliqués » : tous les chiffres).

## Travail fait
- B1 (`crates/ai/src/b1.rs`, `vue.rs`) :
  - go again prévu pour Zenith Blade aiguisée, les combos d'Ira et Scar for a Scar ;
  - usage des équipements d'Ira ;
  - Frostbite de Frost Spike chez l'adversaire ;
  - sorts arcaniques notés selon les dégâts qui passent l'arcane barrier adverse ;
  - renforts (Crucible, amp, Stir) avant un sort ;
  - carte Ice bleue en arsenal, jouée au tour adverse ;
  - ordre du pitch sous le deck.
- Tests : `bot_b1_go_again.rs` (9), `bot_b1_sorts.rs` (6).
- `fab-sim --apprendre-side --side-adverse appris` (nouvelle option) ;
  `data/side/b1.txt` réappris (20 000 parties par paire, 4 min).
- Poids B3 4e génération : 46,9 % contre la 3e (−22 Elo), non retenus.
- Tournoi B3 contre B3 refait : Ira 60 → 70 %, Hala 42 → 47 %, Iyslander 21 → 41 %.
