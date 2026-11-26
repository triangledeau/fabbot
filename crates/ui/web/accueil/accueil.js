// Accueil Arsenal (volet « accueil », adresse #accueil ; lot 0 du plan de
// rapatriement). Repris de renderHome() de l'Arsenal JS : logo et trois
// volets. Jouer mène à l'écran de partie ; Apprendre (didacticiel, puzzles)
// et Entraîner (tournoi) arrivent plus tard (phases 2 et 3, lot 8).
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
      <a class="ht" style="--c:#c9483c" href="#nouveau-match">
        <div class="pic"><div class="art" id="a-art-jouer"></div><span class="sym" aria-hidden="true">⚔</span></div>
        <div class="txt"><h2>Jouer</h2><ul><li>Affronter le bot</li><li>Match en plusieurs manches</li><li>Équipement et deck avant chaque manche</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">Decks</span><span id="a-decks">…</span></div>
          <div class="row"><span class="muted">Bots</span><span id="a-bots">…</span></div></div>
      </a>
      <div class="ht" style="--c:#4f8fe0" aria-disabled="true">
        <div class="pic"><div class="art"></div><span class="sym" aria-hidden="true">✦</span><span class="badge">BIENTÔT</span></div>
        <div class="txt"><h2>Apprendre</h2><ul><li>Didacticiel : les règles pas à pas</li><li>Puzzles de value</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">Prévu</span><span>à repenser (phases 2 et 3)</span></div></div>
      </div>
      <div class="ht" style="--c:#ffd84a" aria-disabled="true">
        <div class="pic"><div class="art"></div><span class="sym" aria-hidden="true">♜</span><span class="badge">BIENTÔT</span></div>
        <div class="txt"><h2>Entraîner</h2><ul><li>Tournoi des bots</li><li>Entraîner le bot sur un deck</li></ul></div>
        <div class="foot"><div class="row"><span class="muted">En attendant</span><span class="num">fab-sim --tournoi</span></div></div>
      </div>
    </nav>
    <div class="homefoot"><a class="btn small" href="#decks">Mes decks</a><a class="btn small" href="#impression">Imprimer</a><span>Moteur <b>fabbot</b> (Rust)</span><span id="a-textes"></span></div>
  </div></div>`;

  // Match en cours (le serveur en lance un au démarrage) : on propose de le reprendre.
  async function rafraichir() {
    try {
      const [d, e] = await Promise.all([
        fetch("/api/decks").then((r) => r.json()),
        fetch("/api/etat").then((r) => r.json()),
      ]);
      const nm = (d.decks_maison || []).length;
      document.getElementById("a-decks").innerHTML = `<b class="num">${d.decks.length}</b> jouables` + (nm ? ` + <b class="num">${nm}</b> maison` : "");
      document.getElementById("a-bots").textContent = d.niveaux.map((n) => n.toUpperCase()).join(" · ");
      document.getElementById("a-textes").textContent = d.textes ? "Textes officiels des cartes chargés" : "Textes officiels absents (tools/fetch_cards.sh)";
      const r = document.getElementById("a-resume");
      if (e && e.reglages && !e.erreur) {
        const m = e.match || {};
        const fini = m.gagnant ? " (terminé)" : "";
        r.innerHTML = `<span><b>Match en cours${fini}</b> : ${esc(nom(e.reglages.deck))} contre ${esc(nom(e.reglages.deck_bot))} (${esc(String(e.reglages.niveau).toUpperCase())}), score <span class="num">${m.vous ?? 0}–${m.bot ?? 0}</span></span>
          <a class="btn small primary" href="#partie">Reprendre</a><a class="btn small quiet" href="#nouveau-match">Nouveau match</a>`;
        r.hidden = false;
      } else r.hidden = true;
      // Illustration du volet Jouer : votre héros (images des cartes, lot 1)
      const art = document.getElementById("a-art-jouer");
      const img = e && e.vous && e.vous.heros_image;
      if (img) { art.style.setProperty("--art", Images.fond(img)); art.classList.add("illu"); }
    } catch (err) { /* serveur injoignable : l'accueil reste affiché sans chiffres */ }
  }
  document.addEventListener("arsenal:vue", (ev) => { if (ev.detail.vue === "accueil") rafraichir(); });
})();
