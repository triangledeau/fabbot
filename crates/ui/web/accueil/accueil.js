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
    <nav class="tiles3" aria-label="Sections">
      <div class="ht" style="--c:#c9483c" id="a-jouer" role="link" tabindex="0" aria-label="Jouer">
        <div class="pic"><div class="art" id="a-art-jouer"></div><span class="sym" aria-hidden="true">⚔</span></div>
        <div class="txt"><h2>Jouer</h2><ul><li>Affronter le bot</li><li>Créer et modifier tes decks</li><li>Imprimer un deck (proxies)</li></ul></div>
        <div class="foot resume" id="a-resume" hidden></div>
      </div>
      <a class="ht" style="--c:#4f8fe0" href="#apprendre">
        <div class="pic"><div class="art" id="a-art-apprendre"></div><span class="sym" aria-hidden="true">✦</span></div>
        <div class="txt"><h2>Apprendre</h2><ul><li>Didacticiel : les règles pas à pas</li><li>Puzzles de value (à venir)</li></ul></div>
      </a>
      <a class="ht" style="--c:#ffd84a" href="#entrainer">
        <div class="pic"><div class="art" id="a-art-ent1"></div><span class="sym" aria-hidden="true">♜</span></div>
        <div class="txt"><h2>Entraîner</h2><ul><li>Tournoi des bots, decks contre decks</li><li>Comparer deux niveaux de bot</li><li>Faire apprendre les bots</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">Dernier tournoi</span><span id="a-tournoi">…</span></div>
          <div class="pbar" id="a-tournoi-barre"><i></i></div></div>
      </a>
    </nav>
    <div class="homefoot"><span>Moteur <b>fabbot</b></span><span id="a-textes"></span></div>
  </div></div>`;

  // Match en cours (le serveur en lance un au démarrage) : on propose de le reprendre.
  async function rafraichir() {
    try {
      const [d, e] = await Promise.all([
        fetch("/api/decks").then((r) => r.json()),
        fetch("/api/etat").then((r) => r.json()),
      ]);
      document.getElementById("a-textes").textContent = d.textes ? "Textes officiels des cartes chargés" : "Textes officiels des cartes absents : relancez Jouer.bat avec Internet pour les télécharger.";
      const noms = d.noms || {};
      const nomDeck = (id) => (noms[id] || nom(id)).split(",")[0];
      const r = document.getElementById("a-resume");
      if (e && e.reglages && !e.erreur) {
        const m = e.match || {};
        const fini = m.gagnant ? " (terminé)" : "";
        const titre = `Match en cours${fini} : ${nomDeck(e.reglages.deck)} contre ${nomDeck(e.reglages.deck_bot)} (bot ${String(e.reglages.niveau).toUpperCase()})`;
        r.innerHTML = `<div class="row"><span class="lib" title="${esc(titre)}"><b>${esc(nomDeck(e.reglages.deck))} – ${esc(nomDeck(e.reglages.deck_bot))}</b> <span class="num">${m.vous ?? 0}–${m.bot ?? 0}</span></span>
          <a class="btn small primary" href="#partie">Reprendre</a><a class="btn small quiet" href="#nouveau-match" title="Nouveau match">Nouveau</a></div>`;
        r.hidden = false;
      } else r.hidden = true;
      illustrations(e && e.vous && e.vous.heros_image);
      dernierTournoi();
    } catch (err) { /* serveur injoignable : l'accueil reste affiché sans chiffres */ }
  }
  // Illustrations des volets, comme dans l'Arsenal JS : Dorinthea pour Jouer,
  // Kano pour Apprendre, Bravo seul pour Entraîner (portraits
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
    poser("a-art-ent1", choix("bravo"));
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
  // Volet Jouer : un div (il contient les liens du match en cours), qui mène
  // à #decks sauf clic sur un de ces liens ou dans le cadre du match.
  const jouer = document.getElementById("a-jouer");
  jouer.addEventListener("click", (ev) => { if (!ev.target.closest(".resume")) location.hash = "#decks"; });
  jouer.addEventListener("keydown", (ev) => { if (ev.key === "Enter" && ev.target === jouer) location.hash = "#decks"; });
  document.addEventListener("arsenal:vue", (ev) => { if (ev.detail.vue === "accueil") rafraichir(); });
})();
