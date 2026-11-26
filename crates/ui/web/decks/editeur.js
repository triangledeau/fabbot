// Éditeur de deck (adresse #editeur ; lot 4b du plan de rapatriement,
// d'après editeur.js de l'Arsenal JS, réécrit autour de l'API).
// À gauche, la collection Silver Age (GET /api/collection) avec ses filtres ;
// à droite, le card-pool du deck (équipement + cartes de deck, 55 au plus) :
// compteurs, courbe de coût, répartition du pitch, et la vérification du
// serveur (POST /api/deck-maison/verifier). Enregistrer = POST
// /api/deck-maison/importer. Aucune règle ici : le serveur vérifie.
//
// Ouvrir l'éditeur : Editeur.ouvrir({ nom, liste }) (deck existant ou copie)
// ou Editeur.nouveau(heros). Clic sur une carte : +1 ; clic droit : −1 ;
// Ctrl+Z : annuler.
"use strict";
const Editeur = (() => {
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const norm = (s) => String(s ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  const slug = (s) => norm(s).replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40);
  const COUL_EN = { rouge: "red", jaune: "yellow", bleu: "blue" };
  const COUL_FR = { red: "rouge", yellow: "jaune", blue: "bleu" };
  const cle = (nom, couleur) => nom + "|" + (couleur || "");
  const EMPL = { Weapon: "arme", Head: "tête", Chest: "torse", Arms: "bras", Legs: "jambes", "Off-Hand": "off-hand", Quiver: "carquois", Equipment: "équipement" };

  let COL = null;          // collection : { source, cartes }
  let PAR_CLE = new Map(); // clé → carte de la collection
  let ED = null;           // { nom, heros, cartes: { clé: n }, pile: [], propre }
  const FL = { q: "", pitch: new Set(), type: new Set(), heros: true, bot: false, dedans: false, tri: "cout" };
  let liste = [], montrees = 0, minuterie = null;
  const sec = document.getElementById("v-decks");

  async function collection() {
    if (COL) return COL;
    COL = await fetch("/api/collection").then((r) => r.json());
    PAR_CLE = new Map(COL.cartes.map((c) => [cle(c.nom, c.couleur), c]));
    return COL;
  }
  const carte = (k) => PAR_CLE.get(k);
  const heros = () => COL && COL.cartes.find((c) => c.heros && c.nom === ED.heros);
  const estArene = (c) => !!c.emplacement;
  const maxCopies = (c) => (c.mots_cles || []).includes("Legendary") ? 1 : 2;

  // Types pour les filtres (d'après les types de la carte, pas de règle)
  const a = (c, t) => (c.types || []).includes(t);
  const TYPES = [
    ["atk", "Attaque", (c) => a(c, "Action") && a(c, "Attack")],
    ["naa", "Action", (c) => a(c, "Action") && !a(c, "Attack")],
    ["ar", "R. attaque", (c) => a(c, "Attack Reaction")],
    ["dr", "R. défense", (c) => a(c, "Defense Reaction")],
    ["inst", "Instant", (c) => a(c, "Instant")],
    ["eq", "Équipement", (c) => estArene(c) && !a(c, "Weapon")],
    ["w", "Arme", (c) => a(c, "Weapon")],
  ];
  // Filtre « pour ce héros » : la carte n'a que des classes / talents du héros
  // (ou aucun). Aide au tri seulement : TODO(règles) vérification de
  // construction côté serveur (voir docs/suivi.md, questions ouvertes).
  function pourLeHeros(c) {
    const h = heros();
    if (!h || !(h.classes || []).length) return true;
    const sup = new Set([...(h.classes || []), ...(h.talents || []), "Generic"]);
    return [...(c.classes || []), ...(c.talents || [])].every((s) => sup.has(s));
  }

  function filtre(c) {
    if (c.heros) return false;
    if (FL.heros && !pourLeHeros(c)) return false;
    if (FL.bot && !c.codee) return false;
    if (FL.dedans && !ED.cartes[cle(c.nom, c.couleur)]) return false;
    if (FL.pitch.size && !FL.pitch.has(c.couleur || "")) return false;
    if (FL.type.size && ![...FL.type].some((t) => TYPES.find((x) => x[0] === t)[2](c))) return false;
    if (FL.q) {
      const t = norm(c.nom + " " + (c.nom_fr || "") + " " + c.type_texte + " " + c.texte);
      if (!norm(FL.q).split(/\s+/).every((w) => t.includes(w))) return false;
    }
    return true;
  }
  const n0 = (v) => (v === null || v === undefined ? 99 : v);
  const TRIS = {
    cout: (x, y) => n0(x.cout) - n0(y.cout) || n0(x.pitch) - n0(y.pitch) || x.nom.localeCompare(y.nom),
    pitch: (x, y) => n0(x.pitch) - n0(y.pitch) || n0(x.cout) - n0(y.cout) || x.nom.localeCompare(y.nom),
    nom: (x, y) => x.nom.localeCompare(y.nom) || n0(x.pitch) - n0(y.pitch),
    puissance: (x, y) => n0(y.puissance === null ? -1 : y.puissance) - n0(x.puissance === null ? -1 : x.puissance) || x.nom.localeCompare(y.nom),
  };

  function gabarit() {
    return `<header class="dk-tete">
      <a class="btn small" href="#decks">← Mes decks</a>
      <input id="ed-nom" class="ed-nom" aria-label="Nom du deck" spellcheck="false">
      <span id="ed-etat" class="muted"></span>
      <span class="ed-actions">
        <button class="btn small" id="ed-annuler" title="Ctrl+Z">↶ Annuler</button>
        <button class="btn small" id="ed-texte">Copier la liste</button>
        <button class="btn small primary" id="ed-enregistrer">Enregistrer</button>
      </span>
    </header>
    <div class="ed-corps">
      <section class="ed-col" aria-label="Collection">
        <div class="ed-filtres">
          <input id="ed-q" type="search" placeholder="Chercher (nom, type, texte)…" spellcheck="false">
          <span class="grp"><span>Pitch</span>${[["rouge", "R"], ["jaune", "J"], ["bleu", "B"], ["", "–"]].map(([v, l]) => `<button class="tg pip-${v || "n"}" data-fp="${v}" aria-pressed="false">${l}</button>`).join("")}</span>
          <span class="grp"><span>Type</span>${TYPES.map(([v, l]) => `<button class="tg" data-ft="${v}" aria-pressed="false">${l}</button>`).join("")}</span>
          <span class="grp"><span>Afficher</span>
            <button class="tg" data-ff="heros" aria-pressed="true" title="Classe ou talent du héros, ou Generic (aide au tri, pas une vérification)">pour ce héros</button>
            <button class="tg" data-ff="bot" aria-pressed="false">jouables par le bot</button>
            <button class="tg" data-ff="dedans" aria-pressed="false">dans le deck</button></span>
          <label class="grp"><span>Tri</span><select id="ed-tri"><option value="cout">coût</option><option value="pitch">pitch</option><option value="nom">nom</option><option value="puissance">puissance</option></select></label>
        </div>
        <div id="ed-resume" class="muted petit"></div>
        <div id="ed-grille" class="ed-grille"></div>
        <button class="btn small" id="ed-plus" hidden>Afficher plus</button>
      </section>
      <section class="ed-deck" aria-label="Le deck">
        <div class="ed-heros" id="ed-heros"></div>
        <div class="ed-compteurs" id="ed-compteurs"></div>
        <div class="ed-courbe" id="ed-courbe" aria-label="Courbe de coût"></div>
        <div id="ed-verif" class="dk-resultat"></div>
        <div id="ed-liste"></div>
      </section>
    </div>`;
  }

  function monter() {
    let v = document.getElementById("ed-vue");
    if (!v) {
      v = document.createElement("div");
      v.id = "ed-vue";
      v.innerHTML = gabarit();
      sec.appendChild(v);
      const $ = (id) => document.getElementById(id);
      $("ed-q").addEventListener("input", (e) => { clearTimeout(minuterie); minuterie = setTimeout(() => { FL.q = e.target.value.trim(); refiltrer(); }, 80); });
      $("ed-tri").addEventListener("change", (e) => { FL.tri = e.target.value; refiltrer(); });
      v.querySelector(".ed-filtres").addEventListener("click", (e) => {
        const b = e.target.closest(".tg"); if (!b) return;
        const bascule = (s, x) => (s.has(x) ? s.delete(x) : s.add(x));
        if (b.dataset.fp !== undefined) bascule(FL.pitch, b.dataset.fp);
        else if (b.dataset.ft) bascule(FL.type, b.dataset.ft);
        else if (b.dataset.ff) FL[b.dataset.ff] = !FL[b.dataset.ff];
        b.setAttribute("aria-pressed", String(b.dataset.fp !== undefined ? FL.pitch.has(b.dataset.fp) : b.dataset.ft ? FL.type.has(b.dataset.ft) : FL[b.dataset.ff]));
        refiltrer();
      });
      $("ed-grille").addEventListener("click", (e) => { const el = e.target.closest(".ec"); if (el) ajouter(el.dataset.k, el); });
      $("ed-grille").addEventListener("contextmenu", (e) => { const el = e.target.closest(".ec"); if (el) { e.preventDefault(); retirer(el.dataset.k); } });
      $("ed-plus").addEventListener("click", plus);
      $("ed-liste").addEventListener("click", (e) => {
        const b = e.target.closest("[data-op]"); if (!b) return;
        const k = b.closest("[data-k]").dataset.k;
        if (b.dataset.op === "+") ajouter(k); else retirer(k);
      });
      $("ed-nom").addEventListener("input", (e) => { ED.nomAffiche = e.target.value; ED.propre = false; etat(); });
      $("ed-annuler").addEventListener("click", annuler);
      $("ed-enregistrer").addEventListener("click", enregistrer);
      $("ed-texte").addEventListener("click", async () => {
        try { await navigator.clipboard.writeText(texte()); $("ed-etat").textContent = "Liste copiée (format FaBrary)."; }
        catch (err) { $("ed-etat").textContent = "Copie impossible dans ce navigateur."; }
      });
      document.addEventListener("keydown", (e) => {
        if (Arsenal.adresse() !== "editeur" || !ED) return;
        if ((e.ctrlKey || e.metaKey) && e.key === "z" && e.target.tagName !== "INPUT") { e.preventDefault(); annuler(); }
      });
      window.addEventListener("beforeunload", (e) => { if (ED && !ED.propre) { e.preventDefault(); e.returnValue = ""; } });
    }
    return v;
  }

  // ---------- la collection ----------
  function refiltrer() {
    liste = COL.cartes.filter(filtre).sort((x, y) => (estArene(y) - estArene(x)) || (y.codee - x.codee) || TRIS[FL.tri](x, y));
    montrees = 0;
    document.getElementById("ed-grille").replaceChildren();
    plus();
    const nb = liste.filter((c) => !c.codee).length;
    document.getElementById("ed-resume").innerHTML = `<b class="num">${liste.length}</b> cartes${nb ? ` · <span class="num">${nb}</span> pas encore jouables par le bot (grisées)` : ""} · <kbd>clic</kbd> +1 · <kbd>clic droit</kbd> −1 · <kbd>Ctrl Z</kbd> annuler`;
  }
  function carteEl(c) {
    const k = cle(c.nom, c.couleur);
    const e = document.createElement("div");
    e.className = "ec" + (c.codee ? "" : " nobot") + (c.couleur ? " " + c.couleur : "");
    e.dataset.k = k;
    e.title = (c.nom_fr ? c.nom_fr + " — " : "") + c.nom + (c.couleur ? " (" + c.couleur + ")" : "") + (c.codee ? "" : "\nPas encore jouable par le bot") + (c.texte ? "\n\n" + c.texte : "");
    e.innerHTML = `<div class="cadre"><b>${esc(c.nom)}</b><span>${esc(c.type_texte)}</span></div><span class="n"></span>`;
    if (c.image) e.prepend(Images.img(c.image, "normal", e, "a-img"));
    if (c.image) e.classList.add("a-img");
    compte(e);
    return e;
  }
  function compte(e) {
    const n = ED.cartes[e.dataset.k] || 0;
    const s = e.querySelector(".n");
    s.textContent = n ? "×" + n : "";
    const c = carte(e.dataset.k);
    e.classList.toggle("max", !!c && n >= maxCopies(c));
  }
  function plus() {
    const g = document.getElementById("ed-grille");
    const f = document.createDocumentFragment();
    for (const c of liste.slice(montrees, montrees + 120)) f.appendChild(carteEl(c));
    montrees = Math.min(liste.length, montrees + 120);
    g.appendChild(f);
    document.getElementById("ed-plus").hidden = montrees >= liste.length;
  }
  const repeindre = (k) => document.querySelectorAll(`#ed-grille .ec[data-k="${CSS.escape(k)}"]`).forEach(compte);

  // ---------- le deck ----------
  function memoriser() { ED.pile.push(JSON.stringify(ED.cartes)); if (ED.pile.length > 80) ED.pile.shift(); }
  function ajouter(k, el) {
    const c = carte(k); if (!c) return;
    const n = ED.cartes[k] || 0;
    if (n >= maxCopies(c)) { secouer(el); message(`${maxCopies(c) === 1 ? "1 exemplaire (Legendary)" : "2 exemplaires"} au plus.`); return; }
    memoriser(); ED.cartes[k] = n + 1; change(k);
  }
  function retirer(k) {
    if (!ED.cartes[k]) return;
    memoriser(); ED.cartes[k]--; if (!ED.cartes[k]) delete ED.cartes[k]; change(k);
  }
  function annuler() {
    const s = ED.pile.pop(); if (!s) { message("Rien à annuler."); return; }
    ED.cartes = JSON.parse(s);
    ED.propre = false;
    document.querySelectorAll("#ed-grille .ec").forEach(compte);
    deck(); verifierPlusTard();
  }
  function change(k) { ED.propre = false; repeindre(k); deck(k); verifierPlusTard(); }
  function secouer(el) { if (!el) return; el.classList.remove("secoue"); void el.offsetWidth; el.classList.add("secoue"); }
  function message(t) { document.getElementById("ed-etat").textContent = t; }
  function etat() { message(ED.propre ? (ED.enregistre ? "enregistré" : "") : "● modifications non enregistrées"); }

  function deck(eclair) {
    const h = heros();
    const H = document.getElementById("ed-heros");
    H.innerHTML = `<span class="ava"></span><div><b>${esc(ED.heros)}</b><div class="muted petit">${esc(h ? [...(h.classes || []), ...(h.talents || [])].join(" · ") : "")}</div></div>`;
    if (h && h.image) H.querySelector(".ava").style.setProperty("--art", Images.fond(h.image));
    const lignes = Object.entries(ED.cartes).map(([k, n]) => ({ k, n, c: carte(k) }));
    const arene = lignes.filter((l) => l.c && estArene(l.c));
    const pioche = lignes.filter((l) => !l.c || !estArene(l.c));
    const somme = (L) => L.reduce((s, l) => s + l.n, 0);
    const na = somme(arene), nd = somme(pioche), pool = na + nd;
    const pc = { rouge: 0, jaune: 0, bleu: 0 };
    for (const l of pioche) if (l.c && pc[l.c.couleur] !== undefined) pc[l.c.couleur] += l.n;
    document.getElementById("ed-compteurs").innerHTML = `
      <span class="${pool > 55 ? "ko" : ""}"><b class="num">${pool}</b>/55 card-pool</span>
      <span class="${nd < 40 ? "ko" : ""}"><b class="num">${nd}</b> de deck (40 au moins)</span>
      <span><b class="num">${na}</b> d'arène</span>
      <span class="pitchs">${["rouge", "jaune", "bleu"].map((c) => `<i class="pip-${c}" style="flex:${pc[c] || 0.0001}" title="${pc[c]} ${c}">${pc[c] || ""}</i>`).join("")}</span>`;
    // courbe de coût des cartes de deck
    const cc = [0, 0, 0, 0, 0];
    for (const l of pioche) if (l.c && l.c.cout !== null && l.c.cout !== undefined) cc[Math.min(4, l.c.cout)] += l.n;
    const m = Math.max(1, ...cc);
    document.getElementById("ed-courbe").innerHTML = cc.map((v, i) => `<div><i style="height:${Math.round((v / m) * 100)}%"></i><span class="num">${i === 4 ? "4+" : i}</span><b class="num">${v || ""}</b></div>`).join("");
    const ligne = (l) => `<div class="rw${l.k === eclair ? " eclair" : ""}${l.c && !l.c.codee ? " nobot" : ""}" data-k="${esc(l.k)}">
        <i class="pip-${(l.c && l.c.couleur) || "n"}"></i><span class="nm">${esc(l.c ? (l.c.nom_fr || l.c.nom) : l.k)}</span>
        <span class="muted petit">${l.c && l.c.emplacement ? esc(EMPL[l.c.emplacement] || l.c.emplacement) : l.c && l.c.cout !== null ? "coût " + l.c.cout : ""}</span>
        <button class="op" data-op="-" aria-label="Retirer un">−</button><b class="num">${l.n}</b><button class="op" data-op="+" aria-label="Ajouter un">+</button></div>`;
    const tri = (L) => L.sort((x, y) => (x.c && y.c ? TRIS.pitch(x.c, y.c) : 0));
    document.getElementById("ed-liste").innerHTML =
      `<h3>Équipement et armes <span class="num">${na}</span></h3>${tri(arene).map(ligne).join("") || '<p class="muted petit">Aucun.</p>'}` +
      `<h3>Deck <span class="num">${nd}</span></h3>${tri(pioche).map(ligne).join("") || '<p class="muted petit">Cliquez des cartes de la collection.</p>'}`;
    etat();
  }

  // Liste au format de l'export FaBrary (lu par le serveur, fab_cards::import)
  function texte() {
    const t = [`Name: ${ED.nomAffiche || ED.nom}`, `Hero: ${ED.heros}`, "Format: Silver Age", "", "Arena cards"];
    const L = Object.entries(ED.cartes).map(([k, n]) => ({ k, n, c: carte(k), nom: k.split("|")[0], coul: k.split("|")[1] }));
    for (const l of L) if (!l.coul) t.push(`${l.n}x ${l.nom}`);
    t.push("", "Deck cards");
    for (const l of L) if (l.coul) t.push(`${l.n}x ${l.nom} (${COUL_EN[l.coul]})`);
    return t.join("\n") + "\n";
  }
  function verifierPlusTard() { clearTimeout(ED.verif); ED.verif = setTimeout(verifier, 400); }
  async function verifier() {
    const r = await fetch("/api/deck-maison/verifier", { method: "POST", body: texte() });
    const v = await r.json();
    const z = document.getElementById("ed-verif");
    if (!r.ok || v.erreur) { z.innerHTML = `<span class="dk-ko">✗ ${esc(v.erreur || r.status)}</span>`; return; }
    const l = [...(v.problemes || [])];
    if ((v.non_codees || []).length) l.push(`${v.non_codees.length} carte(s) pas encore codée(s) pour le bot`);
    z.innerHTML = (v.jouable ? `<span class="dk-ok">✓ Jouable contre le bot</span>` : `<span class="dk-ko">✗ Pas encore jouable</span>`) + (l.length ? `<ul>${l.map((x) => `<li>${esc(x)}</li>`).join("")}</ul>` : "");
  }
  async function enregistrer() {
    const nom = slug(ED.nomAffiche || ED.nom);
    if (!nom) { message("Donnez un nom au deck."); return; }
    if (ED.source && ED.source.nom === nom && !ED.source.maison) { message("C'est le nom d'un deck du projet : choisissez-en un autre."); return; }
    const r = await fetch("/api/deck-maison/importer?nom=" + encodeURIComponent(nom), { method: "POST", body: texte() });
    const v = await r.json();
    if (!r.ok || v.erreur) { message("✗ " + (v.erreur || r.status)); return; }
    ED.nom = nom; ED.nomAffiche = nom; ED.propre = true; ED.enregistre = true;
    ED.source = { nom, maison: true };
    document.getElementById("ed-nom").value = nom;
    message(`enregistré dans data/decks/maison/${nom}.txt` + (v.jouable ? " · jouable" : ""));
  }

  async function demarrer(e) {
    monter();
    await collection();
    ED = Object.assign({ pile: [], propre: true, enregistre: false }, e);
    document.getElementById("ed-nom").value = ED.nomAffiche || ED.nom;
    FL.dedans = false;
    document.querySelector('[data-ff="dedans"]').setAttribute("aria-pressed", "false");
    refiltrer(); deck(); verifier();
  }

  // Un deck existant (du projet ou maison) : copie pour un deck du projet.
  async function ouvrir(nom) {
    const v = await fetch("/api/deck?nom=" + encodeURIComponent(nom)).then((r) => r.json());
    if (v.erreur) throw new Error(v.erreur);
    const cartes = {};
    for (const e of v.liste.arene) cartes[cle(e.nom, "")] = (cartes[cle(e.nom, "")] || 0) + e.n;
    for (const e of v.liste.deck) cartes[cle(e.nom, e.couleur)] = (cartes[cle(e.nom, e.couleur)] || 0) + e.n;
    const copie = !v.maison;
    await demarrer({ nom: copie ? "" : nom, nomAffiche: copie ? nom + "-copie" : nom, heros: v.heros, cartes, source: { nom, maison: v.maison } });
    if (copie) { ED.propre = false; etat(); }
  }
  async function nouveau(h) { await demarrer({ nom: "", nomAffiche: slug(h) + "-nouveau", heros: h, cartes: {} }); ED.propre = false; etat(); }
  async function herosJeunes() {
    await collection();
    return COL.cartes.filter((c) => c.heros && ((c.types || []).includes("Young") || !(c.types || []).length)).sort((x, y) => x.nom.localeCompare(y.nom));
  }
  const ouvert = () => !!ED && !ED.propre;
  return { ouvrir, nouveau, herosJeunes, ouvert };
})();
