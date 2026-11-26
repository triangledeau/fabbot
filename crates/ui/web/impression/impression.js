// Volet « Imprimer » (adresse #impression ; lot 5 du plan de rapatriement,
// d'après Proxies de Rathe de l'Arsenal JS, en plus simple). Planches de
// proxies d'un deck (du projet ou maison) au format carte (63 × 88 mm),
// 9 par page A4 ou Letter, traits de coupe ; images officielles chargées en
// ligne (VF d'abord). Le navigateur imprime (ou enregistre en PDF).
"use strict";
(() => {
  const sec = document.getElementById("v-impression");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  sec.innerHTML = `<header class="dk-tete im-ecran">
      <a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a>
      <h1>Imprimer</h1>
      <span class="muted">Proxies d'un deck, 63 × 88 mm, à découper</span>
    </header>
    <div class="im-reglages im-ecran">
      <label>Deck <select id="im-deck"></select></label>
      <label>Papier <select id="im-papier"><option value="A4">A4</option><option value="Letter">Letter</option></select></label>
      <label><input type="checkbox" id="im-heros" checked> Héros</label>
      <label><input type="checkbox" id="im-arene" checked> Équipement et armes</label>
      <label><input type="checkbox" id="im-traits" checked> Traits de coupe</label>
      <label title="Espace entre les cartes, pour couper plus facilement">Espace <select id="im-espace"><option value="0">0 mm</option><option value="1">1 mm</option><option value="2">2 mm</option></select></label>
      <button class="btn quiet small" id="im-tout" hidden>Tout réimprimer</button>
      <button class="btn primary" id="im-imprimer">Imprimer…</button>
      <span id="im-info" class="muted"></span>
    </div>
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

  async function planches() {
    const nom = $("im-deck").value;
    if (!nom) return;
    const [v, im] = await Promise.all([fetch("/api/deck?nom=" + encodeURIComponent(nom)).then((r) => r.json()), tableImages()]);
    if (v.erreur) { $("im-info").textContent = v.erreur; return; }
    const cartes = [];
    // Le k-ième exemplaire d'une carte est « possédé » si k < possedees[cle].
    const ajouter = (n, nomC, couleur) => { const k = cle(nomC, couleur); for (let i = 0; i < n; i++) cartes.push({ nom: nomC, couleur, c: im.get(k), cle: k, poss: i < (possedees[k] || 0), max: n }); };
    if ($("im-heros").checked) ajouter(1, v.heros, "");
    if ($("im-arene").checked) for (const e of v.liste.arene) ajouter(e.n, e.nom, "");
    for (const e of v.liste.deck) ajouter(e.n, e.nom, e.couleur);
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
    try { await remplirDecks(); await planches(); } catch (err) { $("im-info").textContent = "Serveur injoignable."; }
  });
})();
