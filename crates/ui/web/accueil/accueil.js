// Accueil Arsenal (volet « accueil », adresse #accueil ; lot 0 du plan de
// rapatriement). Repris de renderHome() de l'Arsenal JS : logo et trois
// volets. Jouer mène à l'écran de partie ; Apprendre (didacticiel, puzzles)
// arrive plus tard (phases 2 et 3) ; Entraîner mène au volet du lot 8.
"use strict";
(() => {
  const sec = document.getElementById("v-accueil");
  const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const nom = (id) => id ? id.charAt(0).toUpperCase() + id.slice(1) : "?";

  sec.innerHTML = `<div class="homepg"><div class="home">
    <div class="brand"><small>FLESH AND BLOOD · SILVER AGE</small>
      <h1><img class="logo" src="commun/logo_arsenal.webp" alt="Arsenal" width="1200" height="531"></h1></div>
    <div class="resume" id="a-resume" hidden></div>
    <nav class="tiles3" aria-label="Sections">
      <a class="ht" style="--c:#c9483c" href="#decks">
        <div class="pic"><div class="art" id="a-art-jouer"></div><span class="sym" aria-hidden="true">⚔</span></div>
        <div class="txt"><h2>Jouer</h2><ul><li>Affronter le bot</li><li>Créer et modifier tes decks</li><li>Imprimer un deck (proxies)</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">Decks</span><span id="a-decks">…</span></div>
          <div class="row"><span class="muted">Bots</span><span id="a-bots">…</span></div></div>
      </a>
      <a class="ht" style="--c:#4f8fe0" href="#apprendre">
        <div class="pic"><div class="art" id="a-art-apprendre"></div><span class="sym" aria-hidden="true">✦</span></div>
        <div class="txt"><h2>Apprendre</h2><ul><li>Didacticiel : les règles pas à pas</li><li>Puzzles de value (à venir)</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">Pour</span><span>débuter de zéro</span></div></div>
      </a>
      <a class="ht" style="--c:#ffd84a" href="#entrainer">
        <div class="pic"><div class="art" id="a-art-ent1"></div><div class="art" id="a-art-ent2" hidden></div><span class="vs" id="a-vs" hidden>VS</span><span class="sym" aria-hidden="true">♜</span></div>
        <div class="txt"><h2>Entraîner</h2><ul><li>Tournoi des bots, decks contre decks</li><li>Comparer deux niveaux de bot</li><li>Faire apprendre les bots</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">Dernier tournoi</span><span id="a-tournoi">…</span></div>
          <div class="pbar" id="a-tournoi-barre"><i></i></div></div>
      </a>
    </nav>
    <div class="homefoot"><a class="btn small" href="#decks">Mes decks</a><a class="btn small" href="#impression">Imprimer</a><span>Moteur <b>fabbot</b></span><span id="a-textes"></span></div>
  </div></div>`;

  // Match en cours (le serveur en lance un au démarrage) : on propose de le reprendre.
  async function rafraichir() {
    try {
      const [d, e] = await Promise.all([
        fetch("/api/decks").then((r) => r.json()),
        fetch("/api/etat").then((r) => r.json()),
      ]);
      const nm = (d.decks_maison || []).length;
      document.getElementById("a-decks").innerHTML = `<b class="num">${d.decks.length + nm}</b> (${nm} à toi)`;
      document.getElementById("a-bots").textContent = d.niveaux.map((n) => n.toUpperCase()).join(" · ");
      document.getElementById("a-textes").textContent = d.textes ? "Textes officiels des cartes chargés" : "Textes officiels des cartes absents : relancez Jouer.bat avec Internet pour les télécharger.";
      const noms = d.noms || {};
      const nomDeck = (id) => (noms[id] || nom(id)).split(",")[0];
      const r = document.getElementById("a-resume");
      if (e && e.reglages && !e.erreur) {
        const m = e.match || {};
        const fini = m.gagnant ? " (terminé)" : "";
        r.innerHTML = `<span><b>Match en cours${fini}</b> : ${esc(nomDeck(e.reglages.deck))} contre ${esc(nomDeck(e.reglages.deck_bot))} (bot ${esc(String(e.reglages.niveau).toUpperCase())}), score <span class="num">${m.vous ?? 0}–${m.bot ?? 0}</span></span>
          <a class="btn small primary" href="#partie">Reprendre</a><a class="btn small quiet" href="#nouveau-match">Nouveau match</a>`;
        r.hidden = false;
      } else r.hidden = true;
      illustrations(e && e.vous && e.vous.heros_image);
      dernierTournoi();
    } catch (err) { /* serveur injoignable : l'accueil reste affiché sans chiffres */ }
  }
  // Illustrations des volets, comme dans l'Arsenal JS : Dorinthea pour Jouer,
  // Kano pour Apprendre, Grindstone contre Iyslander pour Entraîner (portraits
  // des héros des decks du bot, images chargées en ligne).
  let portraits = null;
  async function illustrations(monHeros) {
    if (!portraits) {
      try {
        const p = await fetch("/api/decks-projet").then((r) => r.json());
        portraits = {};
        for (const d of p.decks || []) if (d.heros_image) portraits[d.nom] = d.heros_image;
      } catch (err) { portraits = {}; }
    }
    const ids = Object.keys(portraits);
    const choix = (...noms) => portraits[noms.find((n) => portraits[n]) || ids[0]];
    const poser = (id, img) => { const el = document.getElementById(id); if (!el || !img) return; el.style.setProperty("--art", Images.fond(img)); el.classList.add("illu"); el.hidden = false; };
    poser("a-art-jouer", choix("dorinthea") || monHeros);
    poser("a-art-apprendre", choix("kano"));
    const g = choix("grindstone", "bravo"), d = choix("iyslander", "kano");
    if (g && d && g !== d) {
      poser("a-art-ent1", g); poser("a-art-ent2", d);
      document.getElementById("a-art-ent1").classList.add("split1");
      document.getElementById("a-art-ent2").classList.add("split2");
      document.getElementById("a-vs").hidden = false;
    } else poser("a-art-ent1", g || d);
  }
  // Pied du volet Entraîner : le tournoi en cours, sinon le dernier fini.
  async function dernierTournoi() {
    const t = document.getElementById("a-tournoi"), barre = document.querySelector("#a-tournoi-barre i");
    try {
      const v = await fetch("/api/tournoi").then((r) => r.json());
      if (v && v.en_cours) {
        t.innerHTML = `en cours, <span class="num">${v.fait} / ${v.total}</span>`;
        barre.style.width = (v.total ? (100 * v.fait) / v.total : 0).toFixed(1) + "%";
        return;
      }
      const h = await fetch("/api/tournois").then((r) => r.json());
      const d = (h.tournois || [])[0];
      if (!d) { t.textContent = "aucun"; barre.style.width = "0"; return; }
      const quand = new Date(d.date * 1000).toLocaleDateString("fr-FR", { day: "numeric", month: "short" });
      t.innerHTML = `${esc(quand)}, <span class="num">${d.fait}</span> parties${d.arret ? " (arrêté)" : ""}`;
      barre.style.width = "100%";
    } catch (err) { t.textContent = "—"; }
  }
  document.addEventListener("arsenal:vue", (ev) => { if (ev.detail.vue === "accueil") rafraichir(); });
})();
