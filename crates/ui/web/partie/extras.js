// Extras de l'écran de partie (lot 9 du plan de rapatriement, d'après les
// ajouts « ANTIGRAV » de l'Arsenal JS) : conseil du bot, barre
// d'évaluation, Bot contre Bot, signalement de bug. Le serveur fait tout
// (crates/ui/src/extras.rs) ; ce fichier utilise etat, action et afficher
// de partie.js.
"use strict";
(() => {
  const zone = $("aide-bot");
  zone.innerHTML = `<div class="ab-ligne">
      <label title="Bot qui conseille ou qui joue à votre place">Bot <select id="ab-niveau"><option value="b1">B1</option><option value="b2" selected>B2</option><option value="b3">B3</option></select></label>
      <button id="ab-conseil" title="Le coup que jouerait ce bot à votre place (touche C). Il ne voit que ce que vous voyez.">Conseil</button>
      <button id="ab-auto" aria-pressed="false" title="Bot contre Bot : ce bot joue à votre place jusqu'à la fin de la manche (cliquez à nouveau pour reprendre la main)">▶ Bot contre Bot</button>
    </div>
    <div id="ab-conseil-texte" class="ab-conseil"></div>
    <div class="ab-eval" title="Vos chances de gagner selon l'évaluation apprise de B3 (moyenne sur des mondes tirés au hasard pour ce que vous ne voyez pas)">
      <span>Vous</span><div class="ab-barre"><i id="ab-eval"></i></div><span id="ab-eval-txt" class="num">–</span>
    </div>`;
  let auto = false;
  let derniereEval = null;
  const niveau = () => $("ab-niveau").value;

  async function conseiller() {
    if (!etat || !etat.decision) return;
    const t = $("ab-conseil-texte");
    t.textContent = "Le bot réfléchit…";
    try {
      const c = await api("GET", "/api/conseil?niveau=" + niveau());
      // Le bouton du ruban de cette option, ou la carte du plateau qui la porte.
      document.querySelectorAll("#v-partie .conseil").forEach((b) => b.classList.remove("conseil"));
      const o = etat.decision.options.find((x) => x.i === c.i);
      const b = document.querySelector(`#options button[data-i="${c.i}"]`) || (o && o.carte !== null ? document.querySelector(`#plateau [data-k="${o.carte}"]`) : null);
      if (b) b.classList.add("conseil");
      t.textContent = `${c.niveau.toUpperCase()} conseille : ${c.texte}`;
    } catch (e) { t.textContent = e.message; }
  }

  async function evaluer() {
    if (!etat || etat.phase === "preparation" || !(etat.decision || etat.fini)) return;
    try {
      const v = await api("GET", "/api/evaluation");
      if (v.vous === null || v.vous === undefined) return;
      derniereEval = v.vous;
      $("ab-eval").style.width = (100 * v.vous).toFixed(1) + "%";
      $("ab-eval-txt").textContent = Math.round(100 * v.vous) + " %";
    } catch (e) { /* pas de manche en cours */ }
  }

  async function boucleAuto() {
    while (auto && Arsenal.vue() === "partie" && etat && etat.decision && !etat.fini) {
      await action("POST", "/api/auto?niveau=" + niveau());
      await new Promise((r) => setTimeout(r, 250));
    }
    arreterAuto();
  }
  function arreterAuto() {
    auto = false;
    $("ab-auto").setAttribute("aria-pressed", "false");
    $("ab-auto").textContent = "▶ Bot contre Bot";
  }

  $("ab-conseil").addEventListener("click", conseiller);
  $("ab-auto").addEventListener("click", () => {
    if (auto) { arreterAuto(); return; }
    auto = true;
    $("ab-auto").setAttribute("aria-pressed", "true");
    $("ab-auto").textContent = "■ Reprendre la main";
    boucleAuto();
  });
  document.addEventListener("keydown", (e) => {
    if (Arsenal.vue() !== "partie" || e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;
    if ($("d-nouvelle").open || $("d-bug").open) return;
    if (e.key === "c" || e.key === "C") conseiller();
  });

  // Après chaque affichage : plus de conseil (la position a changé),
  // nouvelle évaluation ; les boutons restent actifs pendant Bot contre Bot.
  document.addEventListener("arsenal:etat", () => {
    const prep = !etat || etat.aucune_partie || etat.phase === "preparation";
    zone.hidden = prep;
    $("ab-conseil-texte").textContent = "";
    $("ab-conseil").disabled = auto || !etat.decision;
    $("ab-auto").disabled = !auto && (!etat.decision || etat.fini);
    if (!prep) evaluer();
  });

  // ---- signalement de bug
  $("b-bug").addEventListener("click", () => {
    $("bug-msg").textContent = "";
    $("d-bug").showModal();
  });
  $("f-bug").addEventListener("submit", async (e) => {
    if (e.submitter && e.submitter.value !== "ok") return;
    e.preventDefault();
    const texte = $("t-bug").value;
    try {
      const r = await fetch("/api/bug", { method: "POST", body: texte });
      const v = await r.json();
      if (!r.ok) throw new Error(v.erreur);
      $("t-bug").value = "";
      $("bug-msg").textContent = `Enregistré : ${v.dossier}/${v.nom}.json. Merci !`;
      setTimeout(() => $("d-bug").close(), 1500);
    } catch (err) { $("bug-msg").textContent = "Non enregistré : " + err.message; }
  });
})();
