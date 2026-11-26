// Volet « Mes decks » (adresse #decks ; lot 4a du plan de rapatriement).
// Importer un deck depuis l'export texte de FaBrary (ou le format des
// fichiers de data/decks/), le vérifier (format Silver Age, légalité,
// cartes codées) et l'enregistrer dans data/decks/maison/. Un deck dont
// toutes les cartes sont codées se choisit ensuite dans « Nouveau match ».
// L'éditeur de deck complet viendra au lot 4b.
"use strict";
(() => {
  const sec = document.getElementById("v-decks");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  sec.innerHTML = `<header class="dk-tete">
      <a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a>
      <h1>Mes decks</h1>
      <span class="muted" id="dk-collection"></span>
    </header>
    <div class="dk-corps">
      <section class="dk-bloc" aria-labelledby="dk-t-liste">
        <h2 id="dk-t-liste">Decks maison</h2>
        <p class="muted petit">Enregistrés dans <span class="num" id="dk-dossier">data/decks/maison/</span>. Un deck dont toutes les cartes sont codées est jouable contre le bot (fenêtre « Nouveau match »).</p>
        <div id="dk-liste" class="dk-liste"></div>
      </section>
      <section class="dk-bloc" aria-labelledby="dk-t-import">
        <h2 id="dk-t-import">Importer une liste</h2>
        <p class="muted petit">Sur FaBrary : « Copy » → « Copy deck list » (texte), puis collez ici. Le format des fichiers du projet (<span class="num">Arena:</span> / <span class="num">Deck:</span>) marche aussi.</p>
        <label class="dk-champ">Nom du deck <input id="dk-nom" placeholder="ex. hala-agressif" autocomplete="off" spellcheck="false"></label>
        <textarea id="dk-texte" rows="14" spellcheck="false" placeholder="Hero: Hala&#10;&#10;Arena cards&#10;1x Zenith Blade&#10;…&#10;&#10;Deck cards&#10;2x Edict of Steel (red)&#10;…"></textarea>
        <div class="dk-boutons">
          <button class="btn" id="dk-verifier">Vérifier</button>
          <button class="btn primary" id="dk-importer">Importer</button>
        </div>
        <div id="dk-resultat" class="dk-resultat" aria-live="polite"></div>
      </section>
    </div>`;
  const $ = (id) => document.getElementById(id);

  // « Hala agressif ! » → « hala-agressif » (nom de fichier accepté par le serveur)
  const slug = (s) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40);

  function bilan(v) {
    const t = [];
    if (v.jouable) t.push(`<span class="dk-ok">✓ Jouable contre le bot</span>`);
    else t.push(`<span class="dk-ko">✗ Pas encore jouable</span>`);
    t.push(`<span class="muted">${esc(v.heros)}${v.classe ? " · " + esc(v.classe) : ""} · ${v.arene ?? "?"} carte(s) d'arène + ${v.deck ?? "?"} de deck</span>`);
    const l = [];
    for (const p of v.problemes || []) l.push(`<li>${esc(p)}</li>`);
    if ((v.non_codees || []).length) l.push(`<li>${v.non_codees.length} carte(s) pas encore codée(s) pour le bot : ${v.non_codees.map(esc).join(", ")}</li>`);
    for (const r of v.remarques || []) l.push(`<li class="muted">${esc(r)}</li>`);
    if (v.legalite_verifiee === false) l.push(`<li class="muted">Légalité Silver Age non vérifiée (base communautaire absente : bash tools/fetch_cards.sh).</li>`);
    return t.join(" ") + (l.length ? `<ul>${l.join("")}</ul>` : "");
  }

  async function lister() {
    try {
      const [v, c] = await Promise.all([
        fetch("/api/decks-maison").then((r) => r.json()),
        fetch("/api/collection").then((r) => r.json()),
      ]);
      $("dk-collection").textContent = c.source === "fabcube"
        ? `Collection Silver Age : ${c.nombre} cartes, dont ${c.codees} jouables par le bot`
        : `Collection : ${c.codees} cartes codées (base Silver Age absente)`;
      const l = $("dk-liste");
      l.replaceChildren();
      if (!v.decks.length) { l.innerHTML = `<p class="muted">Aucun deck maison pour l'instant.</p>`; return; }
      for (const d of v.decks) {
        const e = document.createElement("div");
        e.className = "dk-deck" + (d.jouable ? " jouable" : "");
        e.innerHTML = `<span class="ava"></span><div><b>${esc(d.nom)}</b>${d.titre ? ` <span class="muted">— ${esc(d.titre)}</span>` : ""}<div>${bilan(d)}</div></div>`;
        if (d.heros_image) e.querySelector(".ava").style.setProperty("--art", Images.fond(d.heros_image));
        l.appendChild(e);
      }
    } catch (err) { $("dk-liste").textContent = "Serveur injoignable."; }
  }

  async function envoyer(importer) {
    const res = $("dk-resultat");
    const texte = $("dk-texte").value;
    const nom = slug($("dk-nom").value);
    if (importer && !nom) { res.innerHTML = `<span class="dk-ko">Donnez un nom au deck.</span>`; $("dk-nom").focus(); return; }
    $("dk-nom").value = nom || $("dk-nom").value;
    const url = importer ? "/api/deck-maison/importer?nom=" + encodeURIComponent(nom) : "/api/deck-maison/verifier";
    res.textContent = "…";
    try {
      const r = await fetch(url, { method: "POST", body: texte, headers: { "Content-Type": "text/plain; charset=utf-8" } });
      const v = await r.json();
      if (!r.ok || v.erreur) { res.innerHTML = `<span class="dk-ko">✗ ${esc(v.erreur || "erreur " + r.status)}</span>`; return; }
      res.innerHTML = (importer ? `<b>Enregistré : ${esc(v.nom)}</b><br>` : "") + bilan(v);
      if (importer) lister();
    } catch (err) { res.innerHTML = `<span class="dk-ko">Serveur injoignable.</span>`; }
  }
  $("dk-verifier").addEventListener("click", () => envoyer(false));
  $("dk-importer").addEventListener("click", () => envoyer(true));
  document.addEventListener("arsenal:vue", (ev) => { if (ev.detail.vue === "decks") lister(); });
})();
