// Éditeur de deck (adresse #editeur), repris de l'éditeur de l'Arsenal JS
// (arsenal.html : tplEditeur, buildFilters, cardMenu, renderDeck,
// renderEqui, issues, showPrev ; docs/rapatriement-interface.md, E1 à E15),
// réécrit autour de l'API de fab-web.
// À gauche, la collection Silver Age (GET /api/collection) : recherche,
// filtres, taille des cartes. À droite, le deck en quatre catégories :
//  - Deck : les 40 cartes proposées au départ (les premières de la liste) ;
//  - Side : le reste du card-pool de 55 ;
//  - Équipement et armes ;
//  - Liste d'envies : hors du card-pool, gardée par ce navigateur.
// Clic : +1 au deck ; ⇧ clic : +1 au side ; Alt clic : +1 aux envies ;
// clic droit : menu ; glisser vers une catégorie (⇧ : tous les exemplaires) ;
// Ctrl+Z : annuler ; Ctrl+S : enregistrer ; Échap : fermer menu et aperçu.
// Aucune règle ici : le serveur vérifie (POST /api/deck-maison/verifier).
"use strict";
const Editeur = (() => {
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const norm = (s) => String(s ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  const slug = (s) => norm(s).replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40);
  const lire = (k, d) => { try { const v = localStorage.getItem(k); return v === null ? d : JSON.parse(v); } catch (e) { return d; } };
  const ecrire = (k, v) => { try { localStorage.setItem(k, JSON.stringify(v)); } catch (e) { /* navigation privée */ } };
  const COUL_EN = { rouge: "red", jaune: "yellow", bleu: "blue" };
  const COUL_FR = { red: "rouge", yellow: "jaune", blue: "bleu", rouge: "rouge", jaune: "jaune", bleu: "bleu" };
  const PCOL = { rouge: "var(--red)", jaune: "var(--yellow)", bleu: "var(--blue)", "": "#5a5f6a" };
  const cle = (nom, couleur) => nom + "|" + (couleur || "");
  const EMPL = { Weapon: "arme", Head: "tête", Chest: "torse", Arms: "bras", Legs: "jambes", "Off-Hand": "main gauche", Quiver: "carquois", Equipment: "équipement" };
  const SLOTS = ["arme", "tête", "torse", "bras", "jambes"];
  const CATS = [["d", "Deck", "Les 40 cartes proposées au départ de la manche"], ["s", "Side", "Le reste du card-pool de 55"], ["e", "Équipement et armes", "Comptent dans les 55"], ["w", "Liste d'envies", "Hors du card-pool : cartes à garder sous la main (gardée par ce navigateur)"]];
  const NOMS_CAT = { d: "Deck", s: "Side", e: "Équipement", w: "Envies" };
  const KWS = ["Go again", "Dominate", "Crush", "Intimidate", "Arcane Barrier", "Blade Break", "Battleworn", "Temper", "Phantasm", "Spellvoid", "Opt", "Ward", "Boost", "Combo", "Reprise", "Blood Debt", "Clash", "Stealth", "Mark"];
  const CLSCOL = { Warrior: "#c9483c", Guardian: "#d08a2a", Brute: "#b0653a", Ninja: "#e0563f", Wizard: "#4f8fe0", Ranger: "#3fb39c", Runeblade: "#9a6bd1", Mechanologist: "#c7b36b", Illusionist: "#e0782f", Assassin: "#8a5bc8", Pirate: "#3f9fc4", Necromancer: "#7b8b6f", Merchant: "#c9a24a", Thief: "#8f8f8f", Bard: "#d47fb0" };

  let COL = null;          // collection : { source, cartes }
  let PAR_CLE = new Map(); // clé → carte de la collection
  let ED = null;           // { nom, nomAffiche, heros, cartes: { clé: {d,s,e,w} }, pile, propre, source }
  const FL = { q: "", pitch: new Set(), cout: new Set(), type: new Set(), kw: new Set(), cls: new Set(), heros: true, bot: false, dedans: false, botDabord: true, tri: "cout", zoom: lire("arsenal-editeur-zoom", 150) };
  let onglet = "liste";
  let liste = [], montrees = 0, minuterie = null, observateur = null;
  let glisse = null, glisseDe = null; // glisser-déposer
  const sec = document.getElementById("v-decks");
  const $ = (id) => document.getElementById(id);
  const couche = () => window.DecksCouche; // menu, toast, boîte de dialogue (decks.js)

  async function collection() {
    if (COL) return COL;
    COL = await fetch("/api/collection").then((r) => r.json());
    PAR_CLE = new Map(COL.cartes.map((c) => [cle(c.nom, c.couleur), c]));
    return COL;
  }
  const carte = (k) => PAR_CLE.get(k);
  const herosC = () => COL && ED && COL.cartes.find((c) => c.heros && c.nom === ED.heros);
  const estArene = (c) => !!(c && c.emplacement);
  const maxCopies = (c) => ((c && c.mots_cles) || []).includes("Legendary") ? 1 : 2;
  const a = (c, t) => (c.types || []).includes(t);
  const TYPES = [
    ["atk", "Attaque", (c) => a(c, "Action") && a(c, "Attack"), "Action – attaque"],
    ["naa", "Action", (c) => a(c, "Action") && !a(c, "Attack"), "Action sans attaque"],
    ["ar", "R. attaque", (c) => a(c, "Attack Reaction"), "Réaction d'attaque"],
    ["dr", "R. défense", (c) => a(c, "Defense Reaction"), "Réaction de défense"],
    ["inst", "Instant", (c) => a(c, "Instant"), "Instant"],
    ["eq", "Équip.", (c) => estArene(c) && !a(c, "Weapon"), "Équipement"],
    ["w", "Arme", (c) => a(c, "Weapon"), "Arme"],
  ];
  const typeDe = (id) => TYPES.find((x) => x[0] === id)[2];
  const emplacement = (c) => EMPL[c.emplacement] || String(c.emplacement || "").toLowerCase();

  // Classes, talents et « Essence of X » du héros
  function soutiens() {
    const h = herosC();
    if (!h) return null;
    // « Essence of X » : talents lus dans le fichier officiel (champ essences).
    return new Set([...(h.classes || []), ...(h.talents || []), ...(h.essences || [])]);
  }
  // Filtre « pour ce héros » : même règle que la vérification du serveur
  // (Collection::hors_classe dans api_decks.rs) : la carte n'a que des
  // classes / talents du héros ; pour une carte hybride (« Brute / Warrior »),
  // un seul côté suffit ; « Essence of X » du héros ajoute le talent X.
  // CR 8.3.7 : « [HERO] Specialization » = seulement pour le héros dont le
  // surnom (le nom avant la virgule, CR 2.7.3) est HERO.
  function pourLeHeros(c) {
    const h = herosC();
    if (h && (c.specialisation || []).length && !c.specialisation.includes(String(h.nom).split(",")[0].trim())) return false;
    if (!h || !(h.classes || []).length || !c.type_texte) return true;
    const sup = new Set([...soutiens(), "Generic"]);
    const connus = new Set([...(c.classes || []), ...(c.talents || [])]);
    const cotes = c.type_texte.split(" - ")[0].split(" / ").map((s) => s.split(/\s+/).filter((m) => connus.has(m)));
    return cotes.some((cote) => cote.every((s) => sup.has(s)));
  }

  // ---------- quantités ----------
  const q = (k) => (ED.cartes[k] || { d: 0, s: 0, e: 0, w: 0 });
  const qMod = (k) => ED.cartes[k] || (ED.cartes[k] = { d: 0, s: 0, e: 0, w: 0 });
  const dansPool = (k) => { const x = q(k); return x.d + x.s + x.e; };
  const somme = (cat) => Object.values(ED.cartes).reduce((s, x) => s + x[cat], 0);
  const cartesDe = (cat) => { const L = []; for (const [k, x] of Object.entries(ED.cartes)) { const c = carte(k); if (c) for (let i = 0; i < x[cat]; i++) L.push(c); } return L; };
  const nettoyer = () => { for (const [k, x] of Object.entries(ED.cartes)) if (!x.d && !x.s && !x.e && !x.w) delete ED.cartes[k]; };

  function filtre(c) {
    if (c.heros) return false;
    if (FL.heros && !pourLeHeros(c)) return false;
    if (FL.bot && !c.codee) return false;
    if (FL.dedans) { const x = q(cle(c.nom, c.couleur)); if (!x.d && !x.s && !x.e && !x.w) return false; }
    if (FL.pitch.size && !FL.pitch.has(c.couleur || "")) return false;
    if (FL.cout.size) { if (c.cout === null || c.cout === undefined) return false; if (!FL.cout.has(c.cout >= 3 ? "3+" : String(c.cout))) return false; }
    if (FL.type.size && ![...FL.type].some((t) => typeDe(t)(c))) return false;
    if (FL.cls.size && ![...(c.classes || []), ...(c.talents || [])].some((s) => FL.cls.has(s)) && !(FL.cls.has("Generic") && !(c.classes || []).length && !(c.talents || []).length)) return false;
    if (FL.kw.size) { const t = norm(c.texte + " " + (c.mots_cles || []).join(" ")); if (![...FL.kw].every((k) => t.includes(norm(k)))) return false; }
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
    puissance: (x, y) => (y.puissance ?? -1) - (x.puissance ?? -1) || x.nom.localeCompare(y.nom),
    defense: (x, y) => (y.defense ?? -1) - (x.defense ?? -1) || x.nom.localeCompare(y.nom),
  };

  function gabarit() {
    return `<header class="top">
      <a class="btn small" href="#decks">← Jouer</a>
      <input id="ed-nom" class="dname" aria-label="Nom du deck" autocomplete="off" spellcheck="false">
      <span id="ed-etat" class="saved"></span><span class="sp"></span>
      <button class="btn small" id="ed-annuler" title="Ctrl+Z">↶ Annuler</button>
      <button class="btn small" id="ed-importer">Importer</button>
      <button class="btn small" id="ed-exporter">Exporter</button>
      <button class="btn small" id="ed-imprimer">Imprimer</button>
      <button class="btn small primary" id="ed-enregistrer" title="Ctrl+S">Enregistrer</button>
    </header>
    <div class="ed">
      <div class="coll">
        <div class="fbar" id="ed-fbar">
          <div class="frow">
            <label class="search"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg><input id="ed-q" placeholder="Nom, texte, mot-clé…  ( / pour chercher )" autocomplete="off" spellcheck="false" aria-label="Chercher une carte"></label>
            <div class="fgrp" id="ed-f-pitch"></div>
            <div class="fgrp" id="ed-f-cout"></div>
          </div>
          <div class="frow"><div class="fgrp" id="ed-f-type"></div><div class="fgrp" id="ed-f-kw"></div></div>
          <div class="frow">
            <div class="fgrp" id="ed-f-cls"></div>
            <div class="fgrp" id="ed-f-aff"></div>
            <select class="sel" id="ed-tri" aria-label="Tri"><option value="cout">Tri : coût</option><option value="pitch">Tri : pitch</option><option value="nom">Tri : nom</option><option value="puissance">Tri : puissance</option><option value="defense">Tri : défense</option></select>
          </div>
          <div class="fsum"><span id="ed-resume"></span><label class="zoom muted" title="Taille des cartes">Taille des cartes <input type="range" id="ed-zoom" min="110" max="230"></label></div>
        </div>
        <div class="eres" id="ed-res"><div class="cgrid" id="ed-grille"></div><div class="more-sentinel" id="ed-sent"></div></div>
      </div>
      <aside class="deckp" aria-label="Le deck">
        <div class="dhead" id="ed-dhead"></div>
        <div id="ed-banniere"></div>
        <div class="mini" id="ed-mini"></div>
        <div class="tabs" role="tablist">
          <button role="tab" data-onglet="liste">Liste</button>
          <button role="tab" data-onglet="equi">Équilibrage</button>
          <button role="tab" data-onglet="verif">Vérifications <span class="nb" id="ed-nb-pb"></span></button>
        </div>
        <div class="dbody" id="ed-dbody"></div>
      </aside>
    </div>
    <div class="prev" id="ed-prev" aria-hidden="true"><img alt=""><div class="tx"></div></div>`;
  }

  function monter() {
    let v = $("ed-vue");
    if (v) return v;
    v = document.createElement("div");
    v.id = "ed-vue";
    v.className = "dkz";
    v.innerHTML = gabarit();
    sec.appendChild(v);
    $("ed-q").addEventListener("input", (e) => { clearTimeout(minuterie); minuterie = setTimeout(() => { FL.q = e.target.value.trim(); refiltrer(); }, 80); });
    $("ed-tri").addEventListener("change", (e) => { FL.tri = e.target.value; refiltrer(); });
    $("ed-zoom").addEventListener("input", (e) => { FL.zoom = +e.target.value; $("ed-grille").style.setProperty("--cw", FL.zoom + "px"); ecrire("arsenal-editeur-zoom", FL.zoom); });
    $("ed-fbar").addEventListener("click", (e) => {
      const b = e.target.closest(".tg"); if (!b) return;
      const bascule = (s, x) => (s.has(x) ? s.delete(x) : s.add(x));
      if (b.dataset.fp !== undefined) bascule(FL.pitch, b.dataset.fp);
      else if (b.dataset.fc) bascule(FL.cout, b.dataset.fc);
      else if (b.dataset.ft) bascule(FL.type, b.dataset.ft);
      else if (b.dataset.fk) FL.kw.delete(b.dataset.fk);
      else if (b.dataset.fl) bascule(FL.cls, b.dataset.fl);
      else if (b.dataset.ff) FL[b.dataset.ff] = !FL[b.dataset.ff];
      filtres(); refiltrer();
    });
    $("ed-grille").addEventListener("click", (e) => {
      const el = e.target.closest(".cc"); if (!el) return;
      ajouter(el.dataset.k, e.altKey || e.ctrlKey ? "w" : e.shiftKey ? "s" : "d", el);
    });
    v.addEventListener("contextmenu", (e) => {
      const el = e.target.closest("#ed-vue .cc, #ed-vue .rw"); if (!el) return;
      e.preventDefault(); menuCarte(el.dataset.k, el.dataset.cat || null, e.clientX, e.clientY, el);
    });
    $("ed-dbody").addEventListener("click", (e) => {
      const h = e.target.closest(".cat > header"); if (h) { h.parentElement.classList.toggle("closed"); return; }
      const b = e.target.closest("[data-op]");
      if (b) { const r = b.closest(".rw"); if (b.dataset.op === "+") ajouter(r.dataset.k, r.dataset.cat); else retirer(r.dataset.k, r.dataset.cat); return; }
      const vc = e.target.closest("[data-carte]");
      if (vc) {
        choisirOnglet("liste");
        const r = vc.dataset.carte && [...document.querySelectorAll("#ed-dbody .rw")].find((x) => x.dataset.k === vc.dataset.carte);
        if (r) { r.closest(".cat").classList.remove("closed"); r.scrollIntoView({ block: "center" }); r.classList.remove("flash"); void r.offsetWidth; r.classList.add("flash"); }
        return;
      }
      if (e.target.closest("[data-montrer]")) { FL.q = ""; $("ed-q").value = ""; FL.dedans = true; FL.heros = false; FL.bot = false; filtres(); refiltrer(); }
    });
    $("ed-dbody").addEventListener("dragover", (e) => {
      const c = e.target.closest(".cat"); if (!c || !glisse) return;
      e.preventDefault();
      document.querySelectorAll("#ed-vue .cat.drop").forEach((x) => x !== c && x.classList.remove("drop"));
      c.classList.add("drop");
    });
    $("ed-dbody").addEventListener("drop", (e) => {
      const c = e.target.closest(".cat"); if (!c || !glisse) return;
      e.preventDefault();
      const vers = c.dataset.cat;
      if (glisseDe === null) ajouter(glisse, vers);
      else if (glisseDe !== vers) deplacer(glisse, glisseDe, vers, e.shiftKey ? 99 : 1);
    });
    v.addEventListener("dragstart", (e) => {
      const r = e.target.closest && e.target.closest("#ed-vue .rw"), c = e.target.closest && e.target.closest("#ed-vue .cc");
      if (r) { glisse = r.dataset.k; glisseDe = r.dataset.cat; r.classList.add("dragging"); }
      else if (c) { glisse = c.dataset.k; glisseDe = null; }
      else return;
      cacherApercu();
      e.dataTransfer.effectAllowed = "move"; e.dataTransfer.setData("text/plain", glisse);
    });
    v.addEventListener("dragend", () => {
      document.querySelectorAll("#ed-vue .dragging").forEach((x) => x.classList.remove("dragging"));
      document.querySelectorAll("#ed-vue .cat.drop").forEach((x) => x.classList.remove("drop"));
      glisse = null;
    });
    document.querySelector("#ed-vue .tabs").addEventListener("click", (e) => { const b = e.target.closest("[data-onglet]"); if (b) choisirOnglet(b.dataset.onglet); });
    $("ed-nom").addEventListener("input", (e) => { ED.nomAffiche = e.target.value; ED.propre = false; etat(); });
    $("ed-annuler").addEventListener("click", annuler);
    $("ed-enregistrer").addEventListener("click", enregistrer);
    $("ed-importer").addEventListener("click", importer);
    $("ed-exporter").addEventListener("click", exporter);
    $("ed-imprimer").addEventListener("click", imprimer);
    // aperçu au survol
    let minApercu = null, kApercu = null;
    v.addEventListener("mouseover", (e) => {
      const el = e.target.closest("#ed-vue .cc, #ed-vue .rw");
      if (!el) { clearTimeout(minApercu); cacherApercu(); kApercu = null; return; }
      if (el.dataset.k === kApercu) return;
      kApercu = el.dataset.k; clearTimeout(minApercu); cacherApercu();
      minApercu = setTimeout(() => apercu(el), el.classList.contains("rw") ? 120 : 550);
    });
    v.addEventListener("mouseleave", () => { clearTimeout(minApercu); cacherApercu(); kApercu = null; });
    document.addEventListener("keydown", (e) => {
      if (Arsenal.adresse() !== "editeur" || !ED) return;
      if (e.key === "Escape") { cacherApercu(); couche().fermerMenu(); return; }
      const dlg = document.getElementById("dk-dlg");
      if (dlg && dlg.open) return;
      const ae = document.activeElement || {}, saisie = ["INPUT", "TEXTAREA", "SELECT"].includes(ae.tagName);
      const m = document.querySelector("#dk-couche .ctx");
      if (m && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
        e.preventDefault();
        const bs = [...m.querySelectorAll("button")], i = bs.indexOf(ae);
        bs[(i + (e.key === "ArrowDown" ? 1 : -1) + bs.length) % bs.length].focus();
        return;
      }
      if (e.key === "/" && !saisie) { e.preventDefault(); $("ed-q").focus(); }
      if ((e.ctrlKey || e.metaKey) && e.key === "z" && !saisie) { e.preventDefault(); annuler(); }
      if ((e.ctrlKey || e.metaKey) && e.key === "s") { e.preventDefault(); enregistrer(); }
      if ((e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) && ae.closest && ae.closest(".cc, .rw")) { e.preventDefault(); const r = ae.getBoundingClientRect(); menuCarte(ae.dataset.k, ae.dataset.cat || null, r.left + 20, r.top + 20, ae); }
      if (e.key === "Enter" && ae.classList && ae.classList.contains("cc")) ajouter(ae.dataset.k, e.shiftKey ? "s" : "d", ae);
    });
    document.addEventListener("scroll", cacherApercu, true);
    window.addEventListener("beforeunload", (e) => { if (ED && !ED.propre) { e.preventDefault(); e.returnValue = ""; } });
    if (typeof IntersectionObserver === "function") {
      observateur = new IntersectionObserver((es) => { if (es[0].isIntersecting && Arsenal.adresse() === "editeur" && montrees < liste.length) plus(); }, { root: $("ed-res"), rootMargin: "600px" });
      observateur.observe($("ed-sent"));
    }
    return v;
  }

  // ---------- filtres ----------
  function filtres() {
    const tg = (attr, v, lib, on, extra = "") => `<button class="tg" data-${attr}="${esc(v)}" aria-pressed="${on}" ${extra}>${lib}</button>`;
    $("ed-f-pitch").innerHTML = "<span>Pitch</span>" + [["rouge", "Rouge (1)"], ["jaune", "Jaune (2)"], ["bleu", "Bleu (3)"], ["", "Sans pitch"]].map(([p, t]) => tg("fp", p, `<i class="pip p-${p || "n"}"></i>`, FL.pitch.has(p), `title="${t}"`)).join("");
    $("ed-f-cout").innerHTML = "<span>Coût</span>" + ["0", "1", "2", "3+"].map((v) => tg("fc", v, `<span class="num">${v}</span>`, FL.cout.has(v))).join("");
    $("ed-f-type").innerHTML = "<span>Type</span>" + TYPES.map(([v, l, , t]) => tg("ft", v, l, FL.type.has(v), `title="${t}"`)).join("");
    $("ed-f-kw").innerHTML = "<span>Mots-clés</span>" + `<select class="sel" id="ed-kw" aria-label="Ajouter un mot-clé"><option value="">＋ ajouter…</option>${KWS.filter((k) => !FL.kw.has(k)).map((k) => `<option>${k}</option>`).join("")}</select>` + [...FL.kw].map((k) => tg("fk", k, esc(k) + " ✕", true)).join("");
    const s = soutiens();
    const sup = s ? [...s, "Generic"] : [];
    $("ed-f-cls").innerHTML = "<span>Classe / talent</span>" + sup.map((x) => tg("fl", x, `${CLSCOL[x] ? `<span class="dot" style="background:${CLSCOL[x]}"></span>` : ""}${esc(x)}`, FL.cls.has(x))).join("");
    $("ed-f-aff").innerHTML = "<span>Afficher</span>" + tg("ff", "heros", "légales pour le héros", FL.heros, 'title="Classe ou talent du héros, ou Generic ; décoché : toutes les cartes Silver Age"')
      + tg("ff", "bot", "jouables par le bot", FL.bot) + tg("ff", "dedans", "déjà dans le deck", FL.dedans) + tg("ff", "botDabord", "jouables d'abord", FL.botDabord, 'title="Les cartes que le bot sait jouer en tête de liste"');
    $("ed-tri").value = FL.tri;
    $("ed-kw").addEventListener("change", (e) => { if (e.target.value) { FL.kw.add(e.target.value); filtres(); refiltrer(); } });
  }
  function refiltrer() {
    liste = COL.cartes.filter(filtre).sort((x, y) => (estArene(x) - estArene(y)) || (FL.botDabord ? y.codee - x.codee : 0) || TRIS[FL.tri](x, y));
    montrees = 0;
    $("ed-grille").replaceChildren();
    $("ed-res").scrollTop = 0;
    plus();
    const nb = liste.filter((c) => !c.codee).length;
    const actifs = FL.pitch.size + FL.cout.size + FL.type.size + FL.kw.size + FL.cls.size + (FL.q ? 1 : 0) + (FL.bot ? 1 : 0) + (FL.dedans ? 1 : 0) + (FL.heros ? 0 : 1);
    $("ed-resume").innerHTML = `<span><b class="num">${liste.length}</b> cartes${nb ? ` · <span class="num">${nb}</span> pas encore jouables par le bot (grisées)` : ""}</span>${actifs ? ' <button class="lien" id="ed-effacer">Effacer les filtres</button>' : ""} <span class="kbdhelp"><span><kbd>clic</kbd> +1 deck</span><span><kbd>⇧ clic</kbd> +1 side</span><span><kbd>Alt clic</kbd> envies</span><span><kbd>clic droit</kbd> menu</span><span><kbd>Ctrl Z</kbd> annuler</span></span>`;
    const r = $("ed-effacer");
    if (r) r.addEventListener("click", () => { Object.assign(FL, { q: "", heros: true, bot: false, dedans: false }); ["pitch", "cout", "type", "kw", "cls"].forEach((k) => FL[k].clear()); $("ed-q").value = ""; filtres(); refiltrer(); });
  }
  function carteEl(c) {
    const k = cle(c.nom, c.couleur);
    const e = document.createElement("div");
    e.className = "cc" + (c.codee ? "" : " nobot") + (pourLeHeros(c) ? "" : " illegal");
    e.dataset.k = k; e.tabIndex = 0; e.draggable = true;
    e.innerHTML = `<div class="cph"><b>${esc(c.nom_fr || c.nom)}</b><span><i class="pip p-${c.couleur || "n"}"></i> ${esc(c.type_texte)}</span></div><div class="cnts"></div>`;
    if (c.image) { const i = Images.img(c.image, "normal", null, null); i.draggable = false; i.addEventListener("load", () => e.classList.add("loaded")); e.prepend(i); }
    compte(e);
    return e;
  }
  function compte(e) {
    const x = q(e.dataset.k), c = carte(e.dataset.k);
    e.querySelector(".cnts").innerHTML = (x.d ? `<i class="d" title="au deck">${x.d}</i>` : "") + (x.e ? `<i class="d" title="équipement">${x.e}</i>` : "") + (x.s ? `<i class="s" title="au side">S${x.s}</i>` : "") + (x.w ? `<i class="e" title="envies">★${x.w}</i>` : "");
    e.classList.toggle("max", !!c && x.d + x.s + x.e >= maxCopies(c));
  }
  function plus() {
    const g = $("ed-grille");
    const f = document.createDocumentFragment();
    for (const c of liste.slice(montrees, montrees + 90)) f.appendChild(carteEl(c));
    montrees = Math.min(liste.length, montrees + 90);
    g.appendChild(f);
  }
  const repeindre = (k) => document.querySelectorAll(`#ed-grille .cc[data-k="${CSS.escape(k)}"]`).forEach(compte);

  // ---------- opérations ----------
  function peutAjouter(c, cat) {
    if (cat === "w") return null;
    if (dansPool(cle(c.nom, c.couleur)) >= maxCopies(c)) return `${maxCopies(c) === 1 ? "Legendary : 1 exemplaire" : "2 exemplaires"} au plus en Silver Age.`;
    if (cat === "d" && estArene(c)) return "Les équipements et armes ne vont pas dans le deck.";
    if (cat === "e" && !estArene(c)) return "Seuls les équipements et armes vont dans cette catégorie.";
    return null;
  }
  function memoriser() { ED.pile.push(JSON.stringify(ED.cartes)); if (ED.pile.length > 60) ED.pile.shift(); }
  function secouer(el) { if (!el || !el.classList.contains("cc")) return; el.classList.remove("shake"); void el.offsetWidth; el.classList.add("shake"); }
  function ajouter(k, cat, el) {
    const c = carte(k); if (!c) return;
    if (estArene(c) && cat === "d") cat = "e";
    if (!estArene(c) && cat === "e") cat = "d";
    const err = peutAjouter(c, cat);
    if (err) { couche().toast(esc(err)); secouer(el); return; }
    memoriser(); qMod(k)[cat]++; change(k, cat, el);
  }
  function retirer(k, cat, tout) {
    const x = ED.cartes[k]; if (!x || !x[cat]) return;
    memoriser(); x[cat] = tout ? 0 : x[cat] - 1; nettoyer(); change(k, cat);
  }
  function deplacer(k, de, vers, n) {
    const c = carte(k), x = qMod(k);
    if (vers === "d" && estArene(c)) { couche().toast("Les équipements et armes ne vont pas dans le deck."); return; }
    if (vers === "e" && !estArene(c)) { couche().toast("Seuls les équipements et armes vont dans cette catégorie."); return; }
    n = Math.min(n, x[de]); if (!n) return;
    if (de === "w" && vers !== "w" && dansPool(k) + n > maxCopies(c)) { couche().toast(`${maxCopies(c)} exemplaires au plus dans le card-pool.`); return; }
    memoriser(); x[de] -= n; x[vers] += n; nettoyer(); change(k, vers);
  }
  function annuler() {
    const s = ED.pile.pop(); if (!s) { couche().toast("Rien à annuler."); return; }
    ED.cartes = JSON.parse(s); ED.propre = false;
    document.querySelectorAll("#ed-grille .cc").forEach(compte);
    deck(); verifierPlusTard();
  }
  function change(k, cat, el) {
    ED.propre = false; repeindre(k); deck(k); verifierPlusTard();
    if (el && el.classList && el.classList.contains("cc")) voler(el, cat);
    const m = document.querySelector(`#ed-vue .mt[data-cat="${cat}"]`); if (m) { m.classList.remove("bump"); void m.offsetWidth; m.classList.add("bump"); }
  }
  // La carte vole vers sa catégorie
  function voler(el, cat) {
    const cible = document.querySelector(`#ed-vue .cat[data-cat="${cat}"] > header`) || document.querySelector(`#ed-vue .mt[data-cat="${cat}"]`);
    if (!cible || !el.animate || (window.matchMedia && matchMedia("(prefers-reduced-motion: reduce)").matches)) return;
    const a1 = el.getBoundingClientRect(), b = cible.getBoundingClientRect(), img = el.querySelector("img");
    const g = document.createElement("div");
    g.className = "ghost";
    Object.assign(g.style, { left: a1.left + "px", top: a1.top + "px", width: a1.width + "px", height: a1.height + "px", backgroundImage: img && img.currentSrc ? `url('${img.currentSrc}')` : "" });
    $("ed-vue").appendChild(g);
    const dx = b.left + 40 - a1.left, dy = b.top + b.height / 2 - a1.top - a1.height / 2;
    g.animate([{ transform: "translate(0,0) scale(1) rotate(0)", opacity: 1 }, { transform: `translate(${dx * 0.55}px,${dy * 0.45 - 60}px) scale(.55) rotate(6deg)`, opacity: 1, offset: 0.55 }, { transform: `translate(${dx}px,${dy}px) scale(.18) rotate(10deg)`, opacity: 0.2 }], { duration: 480, easing: "cubic-bezier(.3,.7,.3,1)" }).onfinish = () => g.remove();
  }
  function message(t, sale) { const e = $("ed-etat"); e.textContent = t; e.classList.toggle("dirty", !!sale); }
  function etat() {
    if (!ED.propre) message("● modifications non enregistrées", true);
    else if (ED.enregistre) message(`enregistré · data/decks/maison/${ED.nom}.txt`);
    else if (ED.source && !ED.source.maison) message(`copie du deck du bot « ${ED.source.nom} »`);
    else message(ED.nom ? "enregistré" : "");
  }

  // ---------- menu clic droit ----------
  function menuCarte(k, deCat, x, y, el) {
    const c = carte(k); if (!c) return;
    cacherApercu();
    const qq = q(k), eq = estArene(c), principal = eq ? "e" : "d";
    const items = [{ grp: "Ajouter un exemplaire" }];
    for (const cat of eq ? ["e", "w"] : ["d", "s", "w"]) {
      const err = peutAjouter(c, cat);
      items.push(["＋", cat === "w" ? "aux envies" : cat === "e" ? "à l'équipement" : `au ${NOMS_CAT[cat].toLowerCase()}`, () => ajouter(k, cat, el && el.classList.contains("cc") ? el : null), cat === principal ? "clic" : cat === "s" ? "⇧ clic" : "Alt clic", !!err]);
    }
    const ici = Object.keys(NOMS_CAT).filter((cat) => qq[cat] > 0);
    if (ici.length) {
      items.push(null, { grp: "Déplacer" });
      for (const de of deCat ? [deCat] : ici) for (const vers of Object.keys(NOMS_CAT)) {
        if (vers === de || (vers === "d" && eq) || (vers === "e" && !eq) || (vers === "s" && eq)) continue;
        items.push(["⇄", `${qq[de] > 1 ? "1 exemplaire : " : ""}${NOMS_CAT[de]} → ${NOMS_CAT[vers]}`, () => deplacer(k, de, vers, 1)]);
        if (qq[de] > 1) items.push(["⇉", `Les ${qq[de]} : ${NOMS_CAT[de]} → ${NOMS_CAT[vers]}`, () => deplacer(k, de, vers, qq[de])]);
      }
      items.push(null, { grp: "Retirer" });
      for (const de of deCat ? [deCat] : ici) items.push(["−", de === "w" ? "1 exemplaire des envies" : de === "e" ? "1 exemplaire de l'équipement" : `1 exemplaire du ${NOMS_CAT[de].toLowerCase()}`, () => retirer(k, de)]);
      items.push(["✕", "Retirer toutes les copies", () => { memoriser(); delete ED.cartes[k]; change(k, principal); }, "", false, "danger"]);
    }
    items.push(null, ["🔍", "Voir en grand", () => enGrand(c)], ["◐", "Les autres couleurs de cette carte", () => { FL.q = c.nom; $("ed-q").value = c.nom; FL.heros = false; filtres(); refiltrer(); }]);
    const details = [qq.d ? "deck " + qq.d : "", qq.s ? "side " + qq.s : "", qq.e ? "équip. " + qq.e : "", qq.w ? "envies " + qq.w : "", c.codee ? "" : "pas encore jouable par le bot"].filter(Boolean).join(" · ");
    couche().menu(x, y, `<div class="h"><i class="pip p-${c.couleur || "n"}"></i>${esc(c.nom_fr || c.nom)}<small>${esc(details)}</small></div>`, items);
  }
  function enGrand(c) {
    couche().dialogue(esc(c.nom_fr || c.nom), `<div class="grand">${c.image ? `<img alt="" referrerpolicy="no-referrer" src="${esc(Images.url(c.image, "large"))}">` : ""}<div><div class="muted">${esc(c.nom)} · ${esc(c.type_texte)}</div><p>${Symboles.html(c.texte)}</p></div></div>`, [["Fermer", null]]);
  }

  // ---------- aperçu au survol ----------
  function cacherApercu() { const p = $("ed-prev"); if (p) p.classList.remove("on"); }
  function apercu(el) {
    if (document.querySelector("#dk-couche .ctx") || !el.isConnected || glisse) return;
    const c = carte(el.dataset.k), p = $("ed-prev"); if (!c) return;
    const i = p.querySelector("img");
    i.referrerPolicy = "no-referrer";
    if (c.image) { i.hidden = false; i.src = Images.url(c.image, "normal"); } else i.hidden = true;
    const st = [c.cout !== null && c.cout !== undefined ? `coût ${c.cout}` : "", c.puissance !== null && c.puissance !== undefined ? `puissance ${c.puissance}` : "", c.defense !== null && c.defense !== undefined ? `défense ${c.defense}` : "", c.pitch ? `pitch ${c.pitch}` : ""].filter(Boolean).join(" · ");
    p.querySelector(".tx").innerHTML = `<b>${esc(c.nom_fr || c.nom)}</b><span class="muted">${esc(c.nom)}${c.couleur ? " (" + c.couleur + ")" : ""} · ${esc(c.type_texte)}</span><span class="st">${st}</span><p>${Symboles.html(c.texte)}</p>${c.codee ? '<span class="pill ok">jouable par le bot</span>' : '<span class="pill">pas encore jouable par le bot</span>'}${pourLeHeros(c) ? "" : '<span class="pill ko">hors classe / talent du héros</span>'}`;
    // à côté de la carte, jamais sur le panneau du deck
    const r = el.getBoundingClientRect(), W = 560, H = 380, panneau = document.querySelector("#ed-vue .deckp").getBoundingClientRect();
    let x = r.right + 12;
    if (x + W > panneau.left - 8) x = r.left - W - 12;
    if (x < 8) x = 8;
    const y = Math.min(innerHeight - H - 8, Math.max(8, r.top - 40));
    p.style.left = x + "px"; p.style.top = y + "px"; p.classList.add("on");
  }

  // ---------- panneau du deck ----------
  function deck(eclair) {
    const h = herosC();
    const nd = somme("d"), ns = somme("s"), ne = somme("e"), nw = somme("w"), pool = nd + ns + ne;
    const eqL = cartesDe("e"), ont = new Set(eqL.map(emplacement));
    const manquent = SLOTS.filter((s) => !ont.has(s));
    const jauge = (cat, l, v, tot, msg, cls) => `<div class="mt" data-cat="${cat}" title="Aller à la catégorie"><div class="l">${l}</div><div class="v">${v}${tot ? `<small> / ${tot}</small>` : ""}</div><div class="m ${cls}">${msg}</div>${tot ? `<div class="mbar"><i style="width:${Math.min(100, (v / tot) * 100)}%;${v > tot ? "background:var(--dmg)" : ""}"></i></div>` : ""}</div>`;
    const dm = nd < 40 ? [`manque ${40 - nd}`, "ko"] : nd > 40 ? [`${nd - 40} de trop`, "trop"] : ["complet", "ok"];
    const dh = $("ed-dhead");
    dh.style.setProperty("--art", h && h.image ? `url('${Images.url(h.image, "large")}')` : "none");
    dh.innerHTML = `<div class="hn"><span class="ava"></span><div><b>${esc(ED.heros)}</b><small>${esc(h ? String(h.type_texte || "").replace(" - Young", "") : "")}${h && h.pv != null ? ` · <span class="num">${h.pv}</span> PV` : ""}${h && h.intellect != null ? ` · intellect <span class="num">${h.intellect}</span>` : ""}${h && !h.codee ? ' · <span style="color:var(--dmg)">héros pas encore codé</span>' : ""}</small></div></div>
      <div class="meters">${jauge("d", "Deck", nd, 40, ...dm)}${jauge("s", "Side", ns, 0, `${pool}/55 en tout`, pool > 55 ? "trop" : "")}${jauge("e", "Équipement", ne, 0, manquent.length ? `sans ${manquent.join(", ")}` : "complet", manquent.length ? "ko" : "ok")}${jauge("w", "Envies", nw, 0, "hors card-pool", "")}</div>`;
    if (h && h.image) dh.querySelector(".ava").style.setProperty("--art", Images.fond(h.image));
    dh.querySelectorAll(".mt").forEach((m) => m.addEventListener("click", () => { choisirOnglet("liste"); const c = document.querySelector(`#ed-vue .cat[data-cat="${m.dataset.cat}"]`); if (c) { c.classList.remove("closed"); c.scrollIntoView({ behavior: "smooth", block: "start" }); } }));
    // bandeau : copie d'un deck du bot
    $("ed-banniere").innerHTML = ED.source && !ED.source.maison && !ED.enregistre ? `<div class="banner"><span>⚠</span><span><b>Copie du deck du bot « ${esc(ED.source.nom)} ».</b> Enregistrer en fait un de vos decks ; la liste du bot et ses réglages ne changent pas.</span></div>` : "";
    // pitch, ressources par main de 4, courbe de coût (cartes du deck)
    const D = cartesDe("d");
    const pc = { rouge: 0, jaune: 0, bleu: 0, "": 0 };
    D.forEach((c) => pc[c.couleur || ""]++);
    const cv = [0, 1, 2, 3, 4].map(() => ({ rouge: 0, jaune: 0, bleu: 0, "": 0 }));
    D.forEach((c) => { if (c.cout !== null && c.cout !== undefined) cv[Math.min(4, c.cout)][c.couleur || ""]++; });
    const cmax = Math.max(1, ...cv.map((x) => x.rouge + x.jaune + x.bleu + x[""]));
    const res = D.length ? ((D.reduce((s, c) => s + (c.pitch || 0), 0) / D.length) * 4).toFixed(1) : "–";
    $("ed-mini").innerHTML = `<div><div class="l"><span>Pitch</span><span class="num">${pc.rouge}·${pc.jaune}·${pc.bleu}</span></div><div class="pstack">${["rouge", "jaune", "bleu", ""].map((p) => `<i style="width:${D.length ? (pc[p] / D.length) * 100 : 0}%;background:${PCOL[p]}"></i>`).join("")}</div><div class="l" style="margin-top:6px"><span>Ressources / main de 4</span><span class="num">${res}</span></div></div>
      <div><div class="l"><span>Courbe de coût</span><span class="muted">0 → 4+</span></div><div class="curve">${cv.map((x, i) => `<div class="ecol">${["rouge", "jaune", "bleu", ""].map((p) => x[p] ? `<i style="height:${(x[p] / cmax) * 100}%;background:${PCOL[p]}"></i>` : "").join("")}<span>${i === 4 ? "4+" : i}</span></div>`).join("")}</div></div>`;
    const pb = problemes();
    const nko = pb.filter((i) => i[0] !== "ok").length;
    $("ed-nb-pb").textContent = nko || "✓";
    $("ed-nb-pb").classList.toggle("ko", pb.some((i) => i[0] === "ko"));
    document.querySelectorAll("#ed-vue .tabs button").forEach((b) => b.setAttribute("aria-selected", String(b.dataset.onglet === onglet)));
    if (onglet === "liste") listeDeck(eclair, ont, pool);
    else if (onglet === "equi") equilibrage();
    else verifications(pb);
    etat();
  }
  function listeDeck(eclair, ont, pool) {
    const db = $("ed-dbody"), haut = db.scrollTop;
    const fermees = new Set([...document.querySelectorAll("#ed-vue .cat.closed")].map((c) => c.dataset.cat));
    db.innerHTML = CATS.map(([cat, l, sous]) => {
      const L = Object.entries(ED.cartes).filter(([, x]) => x[cat] > 0).map(([k, x]) => [carte(k) || horsCollection(k), x[cat], k])
        .sort((a1, b) => cat === "e" ? SLOTS.indexOf(emplacement(a1[0])) - SLOTS.indexOf(emplacement(b[0])) || a1[0].nom.localeCompare(b[0].nom) : TRIS.cout(a1[0], b[0]));
      const n = L.reduce((s, r) => s + r[1], 0);
      const reste = cat === "d" ? (n < 40 ? `<span class="miss" style="color:var(--yellow)">manque ${40 - n}</span>` : n > 40 ? `<span class="miss" style="color:var(--dmg)">${n - 40} de trop</span>` : '<span class="miss" style="color:var(--good)">40 ✓</span>') : cat === "s" ? `<span class="miss muted">card-pool : ${pool} / 55</span>` : "";
      return `<section class="cat ${fermees.has(cat) ? "closed" : ""}" data-cat="${cat}"><header><svg class="chev" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg><h3>${l}</h3><span class="ct num">${n}</span>${reste}</header>
        ${cat === "e" ? `<div class="slots">${SLOTS.map((s) => `<span class="slot ${ont.has(s) ? "ok" : ""}">${s}</span>`).join("")}</div>` : ""}
        <div class="rows">${L.length ? L.map(([c, n2, k]) => ligne(c, n2, cat, k, k === eclair)).join("") : `<div class="tip">${sous}. Glisse une carte ici ou utilise le clic droit.</div>`}</div></section>`;
    }).join("");
    db.scrollTop = haut;
    db.querySelectorAll(".rw").forEach((e) => { const c = carte(e.dataset.k); if (c && c.image) e.style.setProperty("--art", `url('${Images.url(c.image, "normal")}')`); });
  }
  // Carte de la liste absente de la collection (bannie depuis, ou hors format) : gardée et signalée
  const horsCollection = (k) => ({ nom: String(k).split("|")[0], couleur: String(k).split("|")[1] || "", hors: true, codee: false, type_texte: "" });
  function ligne(c, n, cat, k, eclair) {
    const mauvais = c.hors || !pourLeHeros(c) || dansPool(k) > maxCopies(c);
    return `<div class="rw ${c.codee ? "" : "nobot"} ${mauvais ? "bad" : ""} ${eclair ? "flash" : ""}" draggable="true" data-k="${esc(k)}" data-cat="${cat}" tabindex="0" style="--pc:${PCOL[c.couleur || ""]}" ${c.hors ? 'title="Absente de la collection Silver Age (bannie ou hors format)"' : ""}>
      <span class="co">${c.cout ?? "·"}</span><span class="nm">${esc(c.nom_fr || c.nom)}${estArene(c) ? `<small>${esc(emplacement(c))}</small>` : ""}${c.hors ? '<small style="color:var(--dmg)">hors format</small>' : ""}</span>
      <span class="q"><button data-op="-" aria-label="Retirer un exemplaire">−</button><b>×${n}</b><button data-op="+" aria-label="Ajouter un exemplaire">+</button></span></div>`;
  }
  function choisirOnglet(o) { onglet = o; deck(); if (o === "equi") chargerRepere(); }

  // ---------- équilibrage ----------
  function roles(L) {
    const r = { rouge: 0, jaune: 0, bleu: 0, atk: 0, naa: 0, ar: 0, dr: 0, inst: 0, blk: 0, pui: 0, def: 0 };
    let np = 0, nd = 0;
    for (const c of L) {
      if (c.couleur) r[c.couleur]++;
      if (typeDe("atk")(c)) { r.atk++; if (c.puissance !== null && c.puissance !== undefined) { r.pui += c.puissance; np++; } }
      else if (typeDe("naa")(c)) r.naa++;
      else if (typeDe("ar")(c)) r.ar++;
      else if (typeDe("dr")(c)) r.dr++;
      else if (typeDe("inst")(c)) r.inst++;
      if (c.defense !== null && c.defense !== undefined) { r.def += c.defense; nd++; if (c.defense >= 3) r.blk++; }
    }
    r.pui = np ? r.pui / np : 0; r.def = nd ? r.def / nd : 0;
    return r;
  }
  let REPERE = null, repereEnCours = false;
  // Repère : moyenne des decks du bot (leurs 40 premières cartes de deck)
  async function chargerRepere() {
    if (REPERE || repereEnCours) return;
    repereEnCours = true;
    try {
      const p = await fetch("/api/decks-projet").then((r) => r.json());
      const acc = { rouge: 0, jaune: 0, bleu: 0, atk: 0, naa: 0, ar: 0, dr: 0, inst: 0, blk: 0, pui: 0, def: 0, n: 0 };
      for (const d of p.decks) {
        const v = await fetch("/api/deck?nom=" + encodeURIComponent(d.nom)).then((r) => r.json());
        const L = [];
        for (const e of v.liste.deck) { const c = carte(cle(e.nom, e.couleur)); if (c) for (let i = 0; i < e.n && L.length < 40; i++) L.push(c); }
        const r = roles(L);
        for (const x in r) acc[x] += r[x];
        acc.n++;
      }
      for (const x in acc) if (x !== "n") acc[x] /= acc.n || 1;
      REPERE = acc;
    } catch (e) { REPERE = null; }
    repereEnCours = false;
    if (onglet === "equi" && ED) deck();
  }
  function equilibrage() {
    const R = REPERE, D = cartesDe("d"), r = roles(D), n = D.length || 1;
    const hb = (l, v, ref, col, tot) => `<div class="hbar"><span>${l}</span><span class="t"><i style="width:${Math.min(100, (v / tot) * 100)}%;background:${col}"></i>${ref !== null && ref !== undefined ? `<u style="left:${(ref / tot) * 100}%" title="repère des decks du bot : ${ref.toFixed(1)}"></u>` : ""}</span><span class="num">${v}</span></div>`;
    const ref = (x) => (R ? R[x] : null);
    const cv = [0, 1, 2, 3, 4].map(() => ({ rouge: 0, jaune: 0, bleu: 0, "": 0 }));
    D.forEach((c) => { if (c.cout !== null && c.cout !== undefined) cv[Math.min(4, c.cout)][c.couleur || ""]++; });
    const cmax = Math.max(1, ...cv.map((x) => x.rouge + x.jaune + x.bleu + x[""]));
    const pM = D.reduce((s, c) => s + (c.pitch || 0), 0) / n, cM = D.reduce((s, c) => s + (c.cout || 0), 0) / n;
    const v4 = (D.reduce((s, c) => s + Math.max(c.puissance || 0, c.defense || 0), 0) / n) * 4;
    const dedans = Object.keys(ED.cartes).filter((k) => dansPool(k) > 0);
    const nob = dedans.filter((k) => !(carte(k) || {}).codee);
    const h = herosC();
    $("ed-dbody").innerHTML = `
    <div class="stat"><h4>Pitch <span>${D.length} cartes au deck</span></h4>${hb("Rouge (1)", r.rouge, ref("rouge"), "var(--red)", 40)}${hb("Jaune (2)", r.jaune, ref("jaune"), "var(--yellow)", 40)}${hb("Bleu (3)", r.bleu, ref("bleu"), "var(--blue)", 40)}
      <div class="kpis" style="margin-top:8px"><div class="kpi"><div class="v">${pM.toFixed(2)}</div><div class="l">pitch moyen</div></div><div class="kpi"><div class="v">${(pM * 4).toFixed(1)}</div><div class="l">ressources / main de 4</div></div><div class="kpi"><div class="v">${cM.toFixed(2)}</div><div class="l">coût moyen</div></div></div>
      <div class="lgd"><span><u></u>${R ? `repère : moyenne des ${R.n} decks du bot` : "repère des decks du bot : chargement…"}</span></div></div>
    <div class="stat"><h4>Courbe de coût <span>par couleur de pitch</span></h4><div class="bigcurve">${cv.map((x, i) => { const t = x.rouge + x.jaune + x.bleu + x[""]; return `<div class="ecol">${["rouge", "jaune", "bleu", ""].map((p) => x[p] ? `<i style="height:${(x[p] / cmax) * 100}%;background:${PCOL[p]}" title="${x[p]} ${p || "sans pitch"}"></i>` : "").join("")}<em style="bottom:calc(${(t / cmax) * 100}% + 2px)">${t || ""}</em><span>${i === 4 ? "4+" : i}</span></div>`; }).join("")}</div></div>
    <div class="stat"><h4>Rôles des cartes</h4>${hb("Attaques", r.atk, ref("atk"), "#e0574a", 40)}${hb("Actions (non-attaques)", r.naa, ref("naa"), "#c9a24a", 40)}${hb("Réactions d'attaque", r.ar, ref("ar"), "#e0782f", 40)}${hb("Réactions de défense", r.dr, ref("dr"), "#4f8fe0", 40)}${hb("Instants", r.inst, ref("inst"), "#9a6bd1", 40)}${hb("Bloqueurs (défense 3+)", r.blk, ref("blk"), "#63c98b", 40)}</div>
    <div class="stat"><h4>Puissance et défense</h4><div class="kpis"><div class="kpi"><div class="v">${r.pui.toFixed(2)}</div><div class="l">puissance moyenne d'une attaque</div>${R ? `<div class="r">repère ${R.pui.toFixed(2)}</div>` : ""}</div><div class="kpi"><div class="v">${r.def.toFixed(2)}</div><div class="l">défense moyenne</div>${R ? `<div class="r">repère ${R.def.toFixed(2)}</div>` : ""}</div><div class="kpi"><div class="v">${v4.toFixed(1)}</div><div class="l">valeur brute d'une main de 4</div><div class="r">chaque carte compte sa puissance ou sa défense, la plus haute</div></div></div></div>
    <div class="stat"><h4>Jouabilité par le bot <span>${dedans.length - nob.length} / ${dedans.length} cartes différentes</span></h4>${h && !h.codee ? '<div class="iss ko" style="margin-bottom:6px"><span class="ic">!</span><span>Le héros n\'est pas encore codé dans le moteur.</span></div>' : ""}
      ${nob.length ? `<div class="muted" style="font-size:.84rem;margin-bottom:6px">Ces cartes empêchent de jouer ce deck contre le bot :</div>${nob.map((k) => { const x = q(k); return ligne(carte(k) || horsCollection(k), x.d + x.s + x.e, x.d ? "d" : x.s ? "s" : "e", k); }).join("")}` : '<div class="iss ok"><span class="ic">✓</span><span>Toutes les cartes sont jouables par le moteur.</span></div>'}</div>`;
    $("ed-dbody").querySelectorAll(".rw").forEach((e) => { const c = carte(e.dataset.k); if (c && c.image) e.style.setProperty("--art", `url('${Images.url(c.image, "normal")}')`); });
  }

  // ---------- vérifications ----------
  // Celles de la page (comptes, exemplaires, classes) puis celles du serveur.
  function problemes() {
    const L = [];
    const nd = somme("d"), pool = nd + somme("s") + somme("e");
    if (nd !== 40) L.push(["ko", `Le deck doit compter 40 cartes au départ (${nd} aujourd'hui) : le reste va au side. <span class="lien" data-carte="">Voir la liste</span>`]);
    if (pool > 55) L.push(["ko", `Le card-pool dépasse 55 cartes (${pool}).`]);
    for (const k of Object.keys(ED.cartes)) {
      const t = dansPool(k); if (!t) continue;
      const c = carte(k);
      const voir = ` <span class="lien" data-carte="${esc(k)}">Montrer</span>`;
      if (!c) { L.push(["ko", `${esc(k.split("|")[0])} : absente de la collection Silver Age (bannie ou hors format).${voir}`]); continue; }
      if (t > maxCopies(c)) L.push(["ko", `${esc(c.nom)}${c.couleur ? " (" + c.couleur + ")" : ""} : ${t} exemplaires (${maxCopies(c)} au plus).${voir}`]);
      if (!pourLeHeros(c)) L.push(["ko", `${esc(c.nom)} : hors de la classe ou du talent de ${esc(ED.heros.split(",")[0])}.${voir}`]);
    }
    if (!cartesDe("e").some((c) => a(c, "Weapon"))) L.push(["warn", "Aucune arme."]);
    const h = herosC();
    if (h && !h.codee) L.push(["warn", "Héros pas encore codé : pas de partie contre le bot avec ce deck."]);
    const nob = Object.keys(ED.cartes).filter((k) => dansPool(k) && carte(k) && !carte(k).codee).length;
    if (nob) L.push(["warn", `${nob} carte${nob > 1 ? "s" : ""} pas encore jouable${nob > 1 ? "s" : ""} par le bot : le deck s'enregistre et s'imprime, mais ne peut pas être joué contre le bot. <span class="lien" data-montrer>Les afficher</span>`]);
    if (ED.serveur && pool > 0) {
      // Les problèmes déjà signalés ici (nombre de cartes, exemplaires) ne sont pas répétés.
      const deja = (p) => (nd !== 40 && /cartes de deck/.test(p)) || (L.some((x) => /exemplaires \(/.test(x[1])) && /copies de/.test(p));
      for (const p of ED.serveur.problemes || []) if (!deja(p) && !L.some((x) => x[1].includes(esc(p)))) L.push(["ko", "Serveur : " + esc(p)]);
    }
    if (!L.some((i) => i[0] === "ko")) L.unshift(["ok", "Deck conforme aux règles de construction Silver Age vérifiées ici (40 cartes au départ, card-pool de 55, 2 exemplaires au plus, classes du héros)."]);
    if (ED.serveur && ED.serveur.jouable && !L.some((i) => i[0] === "ko")) L.push(["ok", "Jouable contre le bot (vérifié par le serveur)."]);
    return L;
  }
  function verifications(L) {
    $("ed-dbody").innerHTML = `<div class="issues" style="margin-top:10px">${L.map(([t, m]) => `<div class="iss ${t}"><span class="ic">${t === "ok" ? "✓" : "!"}</span><span>${m}</span></div>`).join("")}</div>`;
  }

  // ---------- liste, enregistrement ----------
  // Format de l'export FaBrary (lu par le serveur, fab_cards::import) : le
  // deck d'abord (les 40 du départ), puis le side ; les envies restent ici.
  function texte() {
    const t = [`Name: ${ED.nomAffiche || ED.nom}`, `Hero: ${ED.heros}`, "Format: Silver Age", "", "Arena cards"];
    const L = Object.entries(ED.cartes).map(([k, x]) => ({ k, x, c: carte(k), nom: k.split("|")[0], coul: k.split("|")[1] }));
    const tri = (A) => A.sort((u, v) => (u.c && v.c ? TRIS.cout(u.c, v.c) : u.nom.localeCompare(v.nom)));
    for (const l of L) if (l.x.e) t.push(`${l.x.e}x ${l.nom}`);
    t.push("", "Deck cards");
    for (const l of tri(L.filter((l) => l.x.d))) t.push(`${l.x.d}x ${l.nom}${l.coul ? ` (${COUL_EN[l.coul]})` : ""}`);
    const side = tri(L.filter((l) => l.x.s));
    if (side.length) t.push("// side : après les 40 cartes du départ");
    for (const l of side) t.push(`${l.x.s}x ${l.nom}${l.coul ? ` (${COUL_EN[l.coul]})` : ""}`);
    return t.join("\n") + "\n";
  }
  const cleEnvies = (nom) => "arsenal-envies:" + nom;
  function garderEnvies(nom) {
    const w = {};
    for (const [k, x] of Object.entries(ED.cartes)) if (x.w) w[k] = x.w;
    ecrire(cleEnvies(nom), w);
  }
  function verifierPlusTard() { clearTimeout(ED.verif); ED.verif = setTimeout(verifier, 400); }
  async function verifier() {
    try {
      const r = await fetch("/api/deck-maison/verifier", { method: "POST", body: texte() });
      const v = await r.json();
      ED.serveur = r.ok && !v.erreur ? v : { problemes: [v.erreur || "erreur " + r.status] };
    } catch (e) { ED.serveur = { problemes: ["serveur injoignable"] }; }
    if (onglet === "verif") deck();
    else { const pb = problemes(); $("ed-nb-pb").textContent = pb.filter((i) => i[0] !== "ok").length || "✓"; $("ed-nb-pb").classList.toggle("ko", pb.some((i) => i[0] === "ko")); }
  }
  async function enregistrer() {
    const nom = slug(ED.nomAffiche || ED.nom);
    if (!nom) { couche().toast("Donne un nom au deck."); $("ed-nom").focus(); return; }
    if (ED.source && ED.source.nom === nom && !ED.source.maison) { couche().toast("C'est le nom d'un deck du bot : choisis-en un autre."); $("ed-nom").focus(); return; }
    if (ED.propre && ED.enregistre) { couche().toast("Rien à enregistrer."); return; }
    try {
      const r = await fetch("/api/deck-maison/importer?nom=" + encodeURIComponent(nom), { method: "POST", body: texte() });
      const v = await r.json();
      if (!r.ok || v.erreur) { couche().toast("Enregistrement impossible : " + esc(v.erreur || r.status)); return; }
      garderEnvies(nom);
      ED.nom = nom; ED.nomAffiche = nom; ED.propre = true; ED.enregistre = true;
      ED.source = { nom, maison: true };
      $("ed-nom").value = nom;
      deck();
      couche().toast(`Enregistré dans data/decks/maison/${esc(nom)}.txt.${v.jouable ? " Jouable contre le bot." : ""}`);
    } catch (e) { couche().toast("Serveur injoignable."); }
  }

  // Lit le texte d'un deck (format des fichiers de data/decks/) : arène,
  // puis deck dans l'ordre ; les 40 premières cartes vont au deck, le reste au side.
  function lireTexte(t) {
    const cartes = {};
    let section = "", nd = 0;
    for (const brut of String(t).split("\n")) {
      const l = brut.trim();
      if (l === "Arena:") { section = "a"; continue; }
      if (l === "Deck:") { section = "d"; continue; }
      const m = /^(\d+)\s+(.+?)\s*(?:\[(.+)\]|\((red|yellow|blue|rouge|jaune|bleu)\))?$/.exec(l);
      if (!m || !section) continue;
      const n = +m[1], nom = m[2].trim();
      if (section === "a") { const x = cartes[cle(nom, "")] || (cartes[cle(nom, "")] = { d: 0, s: 0, e: 0, w: 0 }); x.e += n; continue; }
      const k = cle(nom, COUL_FR[m[4]] || "");
      const x = cartes[k] || (cartes[k] = { d: 0, s: 0, e: 0, w: 0 });
      const versDeck = Math.max(0, Math.min(n, 40 - nd));
      x.d += versDeck; x.s += n - versDeck; nd += versDeck;
    }
    return cartes;
  }
  function importer() {
    couche().dialogue("Remplacer la liste", `<p class="muted">Colle une liste FaBrary (export texte) ou le format des fichiers du projet : elle <b>remplace</b> la liste en cours (Ctrl+Z pour revenir). Le héros du deck ne change pas. Les 40 premières cartes de deck vont au deck, le reste au side.</p><textarea id="ed-imp" spellcheck="false"></textarea><div id="ed-imp-r" class="muted petit" aria-live="polite"></div>`, [
      ["Annuler", null],
      ["Remplacer", async () => {
        try {
          const r = await fetch("/api/deck-maison/verifier", { method: "POST", body: $("ed-imp").value });
          const v = await r.json();
          if (!r.ok || v.erreur) { $("ed-imp-r").textContent = "✗ " + (v.erreur || r.status); return false; }
          const nouvelles = lireTexte(v.texte);
          if (!Object.keys(nouvelles).length) { $("ed-imp-r").textContent = "Aucune carte reconnue."; return false; }
          memoriser();
          for (const [k, x] of Object.entries(ED.cartes)) if (x.w) (nouvelles[k] || (nouvelles[k] = { d: 0, s: 0, e: 0, w: 0 })).w = x.w;
          ED.cartes = nouvelles; ED.propre = false;
          document.querySelectorAll("#ed-grille .cc").forEach(compte);
          deck(); verifierPlusTard();
          const n = Object.values(nouvelles).reduce((s, x) => s + x.d + x.s + x.e, 0);
          const pbs = problemes().filter((i) => i[0] === "ko").length;
          couche().toast(`${n} cartes importées${(v.remarques || []).length ? ` · ${v.remarques.length} remarque(s)` : ""}${pbs ? ` · ${pbs} problème(s), voir « Vérifications »` : ""}${v.heros && v.heros !== ED.heros ? " · héros de la liste ignoré" : ""}.`);
        } catch (e) { $("ed-imp-r").textContent = "Serveur injoignable."; return false; }
      }, "btn primary"],
    ]);
  }
  function exporter() {
    const t = texte();
    couche().dialogue("Exporter la liste", `<p class="muted">Format FaBrary (à coller dans « Import » sur fabrary.net, ou dans « Importer une liste »). Les envies ne sont pas exportées.</p><textarea readonly id="ed-exp">${esc(t)}</textarea>`, [
      ["Fermer", null],
      ["Fichier .txt", () => { const l = document.createElement("a"); l.href = URL.createObjectURL(new Blob([t], { type: "text/plain" })); l.download = (slug(ED.nomAffiche || ED.nom) || "deck") + ".txt"; document.body.appendChild(l); l.click(); l.remove(); return false; }],
      ["Copier", async () => { try { await navigator.clipboard.writeText(t); couche().toast("Liste copiée."); } catch (e) { $("ed-exp").select(); couche().toast("Copie refusée par le navigateur : la liste est sélectionnée, Ctrl+C."); return false; } }, "btn primary"],
    ]);
  }
  function imprimer() {
    if (!ED.enregistre && !(ED.source && ED.propre)) { couche().toast("Enregistre d'abord le deck pour l'imprimer."); return; }
    const nom = ED.enregistre ? ED.nom : ED.source.nom;
    couche().imprimer(nom);
  }

  async function demarrer(e) {
    monter();
    await collection();
    ED = Object.assign({ pile: [], propre: true, enregistre: false, serveur: null }, e);
    $("ed-nom").value = ED.nomAffiche || ED.nom;
    $("ed-zoom").value = FL.zoom;
    $("ed-grille").style.setProperty("--cw", FL.zoom + "px");
    FL.dedans = false; FL.cls.clear(); FL.q = ""; $("ed-q").value = "";
    filtres(); refiltrer(); deck(); verifier();
  }

  // Un deck existant (du projet ou maison) : copie pour un deck du projet.
  async function ouvrir(nom) {
    const v = await fetch("/api/deck?nom=" + encodeURIComponent(nom)).then((r) => r.json());
    if (v.erreur) throw new Error(v.erreur);
    const cartes = lireTexte(v.texte);
    const copie = !v.maison;
    if (!copie) for (const [k, n] of Object.entries(lire(cleEnvies(nom), {}))) (cartes[k] || (cartes[k] = { d: 0, s: 0, e: 0, w: 0 })).w = n;
    await demarrer({ nom: copie ? "" : nom, nomAffiche: copie ? nom + "-copie" : nom, heros: v.heros, cartes, source: { nom, maison: v.maison }, enregistre: !copie });
    if (copie) { ED.propre = false; etat(); }
  }
  async function nouveau(h) { await demarrer({ nom: "", nomAffiche: slug(h) + "-nouveau", heros: h, cartes: {} }); ED.propre = false; etat(); }
  async function herosJeunes() {
    await collection();
    return COL.cartes.filter((c) => c.heros && ((c.types || []).includes("Young") || !(c.types || []).length)).sort((x, y) => x.nom.localeCompare(y.nom));
  }
  // « Dupliquer » un deck maison : même liste, enregistrée sous un autre nom.
  function commeCopie() {
    if (!ED) return;
    ED.nomAffiche = (ED.nomAffiche || ED.nom) + "-copie"; ED.nom = ""; ED.propre = false; ED.enregistre = false;
    $("ed-nom").value = ED.nomAffiche;
    deck();
  }
  const ouvert = () => !!ED && !ED.propre;
  return { ouvrir, nouveau, herosJeunes, ouvert, commeCopie, lireTexte };
})();
