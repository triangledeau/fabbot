// Volet « Imprimer » (adresse #impression ; lot 5 du plan de rapatriement,
// d'après Proxies de Rathe de l'Arsenal JS, en plus simple). Planches de
// proxies d'un deck (du projet ou maison) au format carte (63 × 88 mm),
// 9 par page A4 ou Letter, traits de coupe ; images officielles chargées en
// ligne (VF d'abord). Le navigateur imprime (ou enregistre en PDF).
// Si l'outil complet « Proxies de Rathe » est copié dans data/outils/, il
// remplace l'outil simple (cadre intégré, deck chargé par arsenalCharger).
"use strict";
(() => {
  const sec = document.getElementById("v-impression");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  // Portée, comme dans l'Arsenal JS : toute la réserve, le deck seul (les
  // 40 premières cartes du fichier), ou la réserve et la liste d'envies
  // (gardée par l'éditeur dans le navigateur).
  const PORTEES = [["reserve", "Deck + side", "Toute la réserve : deck, side, armes et équipements"], ["deck", "Deck seul", "Les 40 cartes du deck, armes et équipements, sans le side"], ["envies", "Avec les envies", "Toute la réserve et la liste d'envies de l'éditeur"]];
  const lireLS = (k, d) => { try { const v = JSON.parse(localStorage.getItem(k)); return v ?? d; } catch (e) { return d; } };
  const ecrireLS = (k, v) => { try { localStorage.setItem(k, JSON.stringify(v)); } catch (e) { /* stockage indisponible */ } };
  let portee = lireLS("arsenal-impression-portee", "reserve");
  if (!PORTEES.some((p) => p[0] === portee)) portee = "reserve";
  sec.innerHTML = `<header class="dk-tete im-ecran">
      <a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a>
      <h1>Imprimer</h1>
      <span class="muted" id="im-sous-titre">Proxies d'un deck, 63 × 88 mm, à découper</span>
    </header>
    <div class="im-reglages im-ecran">
      <label>Deck <select id="im-deck"></select></label>
      <div class="seg" id="im-portee" role="group" aria-label="Cartes à imprimer">${PORTEES.map(([v, l, t]) => `<button type="button" data-p="${v}" title="${esc(t)}">${l}</button>`).join("")}</div>
      <span class="im-simple">
      <label>Papier <select id="im-papier"><option value="A4">A4</option><option value="Letter">Letter</option></select></label>
      <label><input type="checkbox" id="im-heros" checked> Héros</label>
      <label><input type="checkbox" id="im-arene" checked> Équipement et armes</label>
      <label><input type="checkbox" id="im-traits" checked> Traits de coupe</label>
      <label title="Espace entre les cartes, pour couper plus facilement">Espace <select id="im-espace"><option value="0">0 mm</option><option value="1">1 mm</option><option value="2">2 mm</option></select></label>
      <button class="btn quiet small" id="im-tout" hidden>Tout réimprimer</button>
      <button class="btn primary" id="im-imprimer">Imprimer…</button>
      </span>
      <button class="btn small" id="im-recharger" hidden title="Remplacer la liste de Proxies de Rathe par celle du deck">Recharger la liste</button>
      <span id="im-info" class="muted"></span>
    </div>
    <div id="im-proxies" class="im-proxies im-ecran" hidden></div>
    <p class="muted petit im-ecran">Imprimez à 100 % (« taille réelle », sans « ajuster à la page »). Les cartes sans image en ligne gardent un cadre avec leur nom. Cliquez une carte que vous avez déjà : elle est grisée et ne s'imprime pas (choix gardé par le navigateur).</p>
    <div id="im-planches" class="im-planches"></div>`;
  const $ = (id) => document.getElementById(id);
  const cle = (nom, couleur) => nom + "|" + (couleur || "");
  let images = null;
  // Cartes déjà possédées (exemplaires à ne pas imprimer), par « nom|couleur ».
  const CLE_POSS = "arsenal.impression.possedees";
  const lirePoss = () => { try { return JSON.parse(localStorage.getItem(CLE_POSS)) || {}; } catch (e) { return {}; } };
  const ecrirePoss = (v) => { try { localStorage.setItem(CLE_POSS, JSON.stringify(v)); } catch (e) { /* stockage indisponible */ } };
  let possedees = lirePoss();

  async function remplirDecks() {
    const [d, m] = await Promise.all([fetch("/api/decks").then((r) => r.json()), fetch("/api/decks-maison").then((r) => r.json())]);
    const s = $("im-deck");
    const garde = s.value;
    s.replaceChildren();
    const affiche = d.noms || {};
    const groupe = (lib, noms) => {
      if (!noms.length) return;
      const g = document.createElement("optgroup");
      g.label = lib;
      for (const n of noms) { const o = document.createElement("option"); o.value = n; o.textContent = affiche[n] || n; g.appendChild(o); }
      s.appendChild(g);
    };
    groupe("Decks du projet", d.decks);
    groupe("Decks maison", m.decks.map((x) => x.nom));
    if (garde) s.value = garde;
  }
  async function tableImages() {
    if (images) return images;
    const c = await fetch("/api/collection").then((r) => r.json());
    images = new Map(c.cartes.map((x) => [cle(x.nom, x.couleur), x]));
    return images;
  }

  // Les cartes d'un deck (réponse de /api/deck) selon la portée choisie.
  function listeDe(v) {
    const deck = [], side = [];
    let n = 0;
    for (const e of v.liste.deck) {
      const d = Math.max(0, Math.min(e.n, 40 - n));
      n += d;
      if (d) deck.push({ ...e, n: d });
      if (e.n - d) side.push({ ...e, n: e.n - d });
    }
    const envies = [];
    if (portee === "envies") {
      for (const [k, q] of Object.entries(lireLS("arsenal-envies:" + v.nom, {}))) {
        const [nomC, couleur] = k.split("|");
        if (q > 0) envies.push({ nom: nomC, couleur: couleur || "", n: q });
      }
    }
    return { arene: v.liste.arene, deck, side: portee === "deck" ? [] : side, envies };
  }
  const EN = { rouge: "red", jaune: "yellow", bleu: "blue" };
  // Format Fabrary lu par Proxies de Rathe (les envies sous « Maybeboard »).
  function texteFabrary(v) {
    const L = listeDe(v), t = [`Name: ${v.nom}`, `Hero: ${v.heros}`, ""];
    const sec = (titre, l) => { if (l.length) t.push(titre, ...l.map((e) => `(${e.n}) ${e.nom}${e.couleur ? ` (${EN[e.couleur]})` : ""}`), ""); };
    sec("Arena cards", L.arene); sec("Deck cards", L.deck); sec("Sideboard", L.side); sec("Maybeboard", L.envies);
    return t.join("\n");
  }

  // ---------- Proxies de Rathe (outil complet, s'il est dans data/outils/) ----------
  let outil = null; // { frame, pret }
  let demande = 0;
  async function outilPresent() {
    try { return !!(await fetch("/api/outils").then((r) => r.json())).proxies; } catch (e) { return false; }
  }
  function cadre() {
    if (outil) return outil;
    const f = document.createElement("iframe");
    f.title = "Proxies de Rathe"; f.setAttribute("allow", "clipboard-write");
    const pret = new Promise((res) => f.addEventListener("load", () => res(f.contentWindow), { once: true }));
    f.src = "/outils/proxies-de-rathe.html";
    $("im-proxies").appendChild(f);
    outil = { frame: f, pret };
    return outil;
  }
  async function chargerOutil() {
    const nom = $("im-deck").value; if (!nom) return;
    const n = ++demande;
    $("im-info").textContent = "Chargement…";
    try {
      const [v, w] = await Promise.all([fetch("/api/deck?nom=" + encodeURIComponent(nom)).then((r) => r.json()), cadre().pret]);
      if (v.erreur) throw new Error(v.erreur);
      if (!w || typeof w.arsenalCharger !== "function") throw new Error("Proxies de Rathe ne répond pas");
      const r = await w.arsenalCharger(texteFabrary(v), v.nom);
      if (n !== demande) return;
      $("im-info").textContent = `« ${v.nom} » : ${r.cartes} cartes` + (r.nonReconnues ? ` · ${r.nonReconnues} ligne(s) non reconnue(s) (voir à gauche)` : "");
    } catch (e) { $("im-info").textContent = "Impression indisponible : " + e.message; }
  }
  let complet = false;
  function majPortee() { for (const b of document.querySelectorAll("#im-portee button")) b.setAttribute("aria-pressed", String(b.dataset.p === portee)); }
  $("im-portee").addEventListener("click", (e) => {
    const b = e.target.closest("[data-p]"); if (!b || b.dataset.p === portee) return;
    portee = b.dataset.p; ecrireLS("arsenal-impression-portee", portee); majPortee();
    complet ? chargerOutil() : planches();
  });
  $("im-recharger").addEventListener("click", () => chargerOutil());
  majPortee();
  // Ctrl+P sur Imprimer : imprime les planches de l'outil, pas la page d'Arsenal.
  document.addEventListener("keydown", (e) => {
    if (!complet || Arsenal.vue() !== "impression" || !(e.ctrlKey || e.metaKey) || e.key.toLowerCase() !== "p") return;
    const w = outil && outil.frame.contentWindow;
    if (w && typeof w.arsenalImprimer === "function") { e.preventDefault(); w.arsenalImprimer(); }
  });

  async function planches() {
    if (complet) return chargerOutil();
    const nom = $("im-deck").value;
    if (!nom) return;
    const [v, im] = await Promise.all([fetch("/api/deck?nom=" + encodeURIComponent(nom)).then((r) => r.json()), tableImages()]);
    if (v.erreur) { $("im-info").textContent = v.erreur; return; }
    const cartes = [];
    // Le k-ième exemplaire d'une carte est « possédé » si k < possedees[cle].
    const ajouter = (n, nomC, couleur) => { const k = cle(nomC, couleur); for (let i = 0; i < n; i++) cartes.push({ nom: nomC, couleur, c: im.get(k), cle: k, poss: i < (possedees[k] || 0), max: n }); };
    const L = listeDe(v);
    if ($("im-heros").checked) ajouter(1, v.heros, "");
    if ($("im-arene").checked) for (const e of L.arene) ajouter(e.n, e.nom, "");
    for (const e of [...L.deck, ...L.side, ...L.envies]) ajouter(e.n, e.nom, e.couleur);
    const papier = $("im-papier").value, espace = Number($("im-espace").value);
    document.documentElement.style.setProperty("--im-espace", espace + "mm");
    let st = document.getElementById("im-page");
    if (!st) { st = document.createElement("style"); st.id = "im-page"; document.head.appendChild(st); }
    st.textContent = `@page { size: ${papier === "A4" ? "210mm 297mm" : "8.5in 11in"}; margin: 0; }`;
    const p = $("im-planches");
    p.replaceChildren();
    p.dataset.papier = papier;
    p.classList.toggle("sans-traits", !$("im-traits").checked);
    const aImprimer = cartes.filter((x) => !x.poss);
    const deja = cartes.filter((x) => x.poss);
    // À l'écran : d'abord les cartes à imprimer, puis les possédées (grisées, jamais imprimées).
    // Les possédées commencent une nouvelle feuille, qui n'est jamais imprimée.
    const lots = [];
    for (const g of [aImprimer, deja]) for (let i = 0; i < g.length; i += 9) lots.push(g.slice(i, i + 9));
    for (const lot of lots) {
      const f = document.createElement("section");
      f.className = "im-feuille";
      const g = document.createElement("div");
      g.className = "im-grille";
      if (lot.every((x) => x.poss)) f.classList.add("im-possedee");
      for (const x of lot) {
        const e = document.createElement("div");
        e.className = "im-carte" + (x.poss ? " im-possedee" : "");
        e.tabIndex = 0;
        e.setAttribute("role", "button");
        e.title = x.poss ? "Déjà possédée : cliquez pour l'imprimer quand même" : "Cliquez si vous avez déjà cette carte";
        e.dataset.cle = x.cle;
        e.dataset.max = x.max;
        e.dataset.poss = x.poss ? "1" : "";
        e.innerHTML = `<span>${esc(x.nom)}${x.couleur ? " (" + esc(x.couleur) + ")" : ""}</span>`;
        if (x.c && x.c.image) e.appendChild(Images.img(x.c.image, "large", null, null));
        g.appendChild(e);
      }
      f.appendChild(g);
      p.appendChild(f);
    }
    $("im-info").textContent = `${aImprimer.length} cartes à imprimer · ${Math.ceil(aImprimer.length / 9)} page(s)` + (deja.length ? ` · ${deja.length} déjà possédée(s)` : "");
    $("im-tout").hidden = !deja.length;
  }
  for (const id of ["im-deck", "im-papier", "im-heros", "im-arene", "im-traits", "im-espace"]) $(id).addEventListener("change", planches);
  $("im-imprimer").addEventListener("click", () => window.print());
  function basculer(e) {
    const k = e.dataset.cle, max = Number(e.dataset.max), n = possedees[k] || 0;
    possedees[k] = e.dataset.poss ? n - 1 : Math.min(max, n + 1);
    if (possedees[k] <= 0) delete possedees[k];
    ecrirePoss(possedees);
    planches();
  }
  $("im-planches").addEventListener("click", (ev) => { const e = ev.target.closest(".im-carte"); if (e) basculer(e); });
  $("im-planches").addEventListener("keydown", (ev) => { const e = ev.target.closest(".im-carte"); if (e && (ev.key === "Enter" || ev.key === " ")) { ev.preventDefault(); basculer(e); } });
  $("im-tout").addEventListener("click", () => { possedees = {}; ecrirePoss(possedees); planches(); });
  document.addEventListener("arsenal:vue", async (ev) => {
    if (ev.detail.vue !== "impression") return;
    try {
      complet = await outilPresent();
      sec.classList.toggle("im-complet", complet);
      $("im-sous-titre").textContent = complet ? "Proxies de Rathe" : "Proxies d'un deck, 63 × 88 mm, à découper";
      $("im-proxies").hidden = !complet;
      $("im-recharger").hidden = !complet;
      await remplirDecks(); await planches();
    } catch (err) { $("im-info").textContent = "Serveur injoignable."; }
  });
})();
