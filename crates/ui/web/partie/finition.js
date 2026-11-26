// Plateau animé (lot 2 du plan de rapatriement, docs/rapatriement-arsenal.md),
// d'après finition.js de l'Arsenal JS, réécrit autour de l'API de fab-web.
// Tout est de l'habillage : aucune règle ici, la page ne fait qu'afficher ce
// que le serveur envoie.
//
// - FX : vols de cartes (chaque carte du plateau porte sa clé data-k = son
//   numéro d'objet ; avant chaque affichage on relève les places, après on
//   fait voler un double de l'ancienne place à la nouvelle), impacts, PV qui
//   défilent, bandeaux (tour, étape du combat), tirer une carte avec une
//   chaîne de laiton et la lâcher sur la chaîne, le pitch ou l'arsenal.
// - Film : après un choix, le serveur renvoie le plateau après chaque pas du
//   moteur (votre choix, puis les réponses du bot) ; on les rejoue un par un,
//   avec une pause « Continuer » avant les dégâts si elle est demandée.
"use strict";

const ReglagesFx = (() => {
  const lire = (k, d) => { try { const v = localStorage.getItem(k); return v === null ? d : v === "1"; } catch (e) { return d; } };
  const ecrire = (k, v) => { try { localStorage.setItem(k, v ? "1" : "0"); } catch (e) { /* stockage indisponible */ } };
  // rapide : « Rythme » de la fenêtre d'ajustement (le film des coups va plus vite)
  const CASES = [["o-anim", "anim", "arsenal-anim"], ["o-pause", "pause", "arsenal-pause"], ["o-rapide", "rapide", "arsenal-rapide"]];
  const r = { anim: lire("arsenal-anim", true), pause: lire("arsenal-pause", false), rapide: lire("arsenal-rapide", false) };
  r.regler = (k, v) => {
    r[k] = v;
    const x = CASES.find((c) => c[1] === k);
    if (!x) return;
    ecrire(x[2], v);
    const c = document.getElementById(x[0]);
    if (c) c.checked = v;
  };
  document.addEventListener("DOMContentLoaded", () => {
    for (const [id, k] of CASES) {
      const c = document.getElementById(id);
      if (!c) continue;
      c.checked = r[k];
      c.addEventListener("change", () => r.regler(k, c.checked));
    }
  });
  return r;
})();

const FX = (() => {
  const RM = !!(window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches);
  const on = () => !RM && ReglagesFx.anim && typeof Arsenal !== "undefined" && Arsenal.vue() === "partie";
  const T = () => Date.now();
  const plateau = () => document.getElementById("plateau");
  const couche = () => document.getElementById("fx");
  const q = (s) => plateau().querySelector(s);
  const ancre = (n) => q(`[data-a="${n}"]`);
  const cle = (k) => q(`[data-k="${k}"]`);
  const proprio = (el) => { const x = el.closest("[data-o]"); return x ? x.dataset.o : null; };
  function rect(el) {
    const r = el.getBoundingClientRect();
    return { x: r.left, y: r.top, w: r.width, h: r.height, cx: r.left + r.width / 2, cy: r.top + r.height / 2 };
  }
  const loin = (a, b) => Math.abs(a.cx - b.cx) > 8 || Math.abs(a.cy - b.cy) > 8 || Math.abs(a.w - b.w) > b.w * 0.12;

  /* ----- Particules (calque <canvas>) ----- */
  let cv = null, g2 = null, parts = [], tourne = false;
  function toile() {
    if (cv) return;
    cv = document.createElement("canvas"); cv.id = "fxc"; couche().appendChild(cv);
    g2 = cv.getContext ? cv.getContext("2d") : null;
    const taille = () => { cv.width = window.innerWidth; cv.height = window.innerHeight; };
    taille(); window.addEventListener("resize", taille);
  }
  const FEU = ["#ff5a44", "#ffb03a", "#fff", "#ffd98a"], OR = ["#ffd24a", "#ff9a3a", "#fff6c8"], ACIER = ["#c7d6e6", "#fff"], POUSSIERE = ["#ffd98a", "#b89a6a", "#fff"];
  function gerbe(x, y, n, col, vit, grav, sz) {
    toile();
    for (let i = 0; i < n; i++) { const a = Math.random() * 6.283, v = (Math.random() ** 0.6) * vit; parts.push({ x, y, vx: Math.cos(a) * v, vy: Math.sin(a) * v, l: 0, m: 0.5 + Math.random() * 0.7, s: (sz || 3) * (0.5 + Math.random()), c: col[i % col.length], g: grav || 0 }); }
    reveil();
  }
  function flux(x0, y0, x1, y1, n, col) {
    toile();
    for (let i = 0; i < n; i++) parts.push({ x: x0 + (Math.random() - 0.5) * 40, y: y0 + (Math.random() - 0.5) * 40, tx: x1, ty: y1, vx: (Math.random() - 0.5) * 9, vy: -Math.random() * 9, l: -i * 0.012, m: 0.75, s: 2.5 + Math.random() * 2.5, c: col, g: 0 });
    reveil();
  }
  function tic() {
    if (!g2) { tourne = false; return; }
    g2.clearRect(0, 0, cv.width, cv.height);
    if (!parts.length) { tourne = false; return; }
    g2.globalCompositeOperation = "lighter";
    parts = parts.filter((p) => p.l < p.m);
    for (const p of parts) {
      p.l += 1 / 60; if (p.l < 0) continue; const k = p.l / p.m;
      if (p.tx !== undefined) { p.vx += (p.tx - p.x) * 0.012 * (1 + k * 6); p.vy += (p.ty - p.y) * 0.012 * (1 + k * 6); p.vx *= 0.9; p.vy *= 0.9; } else { p.vy += p.g; p.vx *= 0.965; p.vy *= 0.965; }
      p.x += p.vx; p.y += p.vy; g2.globalAlpha = Math.max(0, 1 - k * k); g2.fillStyle = p.c; g2.beginPath(); g2.arc(p.x, p.y, p.s * (1 - k * 0.6), 0, 6.3); g2.fill();
    }
    g2.globalAlpha = 1; requestAnimationFrame(tic);
  }
  function reveil() { if (!tourne && g2) { tourne = true; requestAnimationFrame(tic); } }

  function flotter(txt, x, y, cls) {
    const f = document.createElement("div"); f.className = "fxfloat " + (cls || ""); f.textContent = txt;
    f.style.left = x + "px"; f.style.top = y + "px"; couche().appendChild(f); setTimeout(() => f.remove(), 1600);
  }
  function secousse() { const p = plateau(); p.classList.remove("secoue"); void p.offsetWidth; p.classList.add("secoue"); setTimeout(() => p.classList.remove("secoue"), 420); }
  function classe(el, c, ms) { if (!el) return; el.classList.remove(c); void el.offsetWidth; el.classList.add(c); setTimeout(() => el.classList.remove(c), ms); }
  function banniere(txt, couleur, petite) {
    const b = document.createElement("div"); b.className = "fxbanner" + (petite ? " petite" : ""); b.textContent = txt;
    b.style.setProperty("--c", couleur || "var(--brass)");
    const m = plateau().getBoundingClientRect();
    b.style.left = m.left + "px"; b.style.width = m.width + "px";
    const mil = document.getElementById("milieu").getBoundingClientRect();
    b.style.top = (petite ? mil.top + 6 : mil.top + mil.height / 2 - 30) + "px";
    couche().appendChild(b); setTimeout(() => b.remove(), petite ? 1200 : 1400);
  }

  /* ----- Vols de cartes ----- */
  let avantVols = new Map(), premier = true;
  function releve() {
    const m = new Map();
    for (const el of plateau().querySelectorAll("[data-k]")) {
      const k = el.dataset.k; if (m.has(k)) continue;
      const r = rect(el); if (!r.w) continue;
      m.set(k, { r, o: proprio(el), node: el });
    }
    return m;
  }
  function double(el, a) {
    const c = el.cloneNode(true);
    c.removeAttribute("data-k"); c.classList.remove("jouable"); c.classList.add("vol");
    c.style.cssText = `position:fixed;left:${a.x}px;top:${a.y}px;width:${a.w}px;height:${a.h}px;margin:0;visibility:visible;`;
    couche().appendChild(c); return c;
  }
  const decal = (a, b) => `translate(${a.cx - b.cx}px,${a.cy - b.cy}px) scale(${a.w / b.w})`;
  function voler(el, de, vers, o = {}) {
    const c = double(o.node || el, vers), duree = o.duree || 460;
    const an = c.animate(o.fondu
      ? [{ transform: decal(de, vers), opacity: 1 }, { transform: "none", opacity: 1, offset: 0.75 }, { transform: "none", opacity: 0 }]
      : [{ transform: decal(de, vers) }, { transform: "none" }],
    { duration: duree, delay: o.delai || 0, easing: "cubic-bezier(.2,.85,.25,1.08)", fill: "both" });
    if (el) el.style.visibility = "hidden";
    const fin = () => { c.remove(); if (el) el.style.visibility = ""; };
    an.onfinish = fin; setTimeout(fin, duree + (o.delai || 0) + 200);
  }
  // D'où vient une carte qu'on n'avait pas encore vue sur le plateau
  function origine(el) {
    const o = proprio(el);
    if (o === "vous" && el.closest('[data-a="main-vous"]')) return ancre("deck-vous");
    if (o === "bot") return ancre("main-bot") || ancre("heros-bot");
    return ancre("heros-" + (o || "vous"));
  }
  function vols() {
    const cur = new Map();
    for (const el of plateau().querySelectorAll("[data-k]")) if (!cur.has(el.dataset.k) && el.getClientRects().length) cur.set(el.dataset.k, el);
    let n = 0;
    for (const [k, el] of cur) {
      const r = rect(el); if (!r.w) continue;
      const p = avantVols.get(k);
      let de = p ? p.r : null;
      if (!de) { const a = origine(el); if (a) de = rect(a); }
      if (!de || !loin(de, r)) continue;
      voler(el, de, r, { delai: p ? 0 : 60 * n++ });
    }
    // Cartes parties (au cimetière, bannies, sous le deck…) : elles filent vers le cimetière en s'effaçant
    for (const [k, p] of avantVols) {
      if (cur.has(k)) continue;
      const a = ancre("cimetiere-" + (p.o || "vous")); if (!a) continue;
      voler(null, p.r, rect(a), { node: p.node, fondu: true, duree: 420 });
    }
  }

  /* ----- PV qui défilent ----- */
  const vies = { vous: null, bot: null }, minut = { vous: 0, bot: 0 };
  function vieAffichee(qui, v) {
    if (!on() || vies[qui] === null) { vies[qui] = v; return v; }
    if (vies[qui] !== v) setTimeout(() => compter(qui), 0);
    return vies[qui];
  }
  function compter(qui) {
    if (minut[qui]) return;
    const pas = () => {
      const e = typeof etat !== "undefined" && etat && etat[qui];
      const cible = e ? e.vie : vies[qui];
      if (vies[qui] === cible) { minut[qui] = 0; return; }
      vies[qui] += Math.sign(cible - vies[qui]) * Math.max(1, Math.ceil(Math.abs(cible - vies[qui]) / 8));
      const x = q(`[data-vie="${qui}"]`);
      if (x) { x.textContent = String(vies[qui]); classe(x, "bump", 180); }
      minut[qui] = setTimeout(pas, 85);
    };
    minut[qui] = 1; pas();
  }

  /* ----- Bandeau des étapes du combat ----- */
  let etape = null;
  function bandeauEtape(e) {
    const s = e.combat ? e.combat.etape : null;
    if (s && s !== etape && s !== "—") banniere("Étape : " + s, e.votre_tour ? "var(--mc)" : "var(--oc)", true);
    etape = s;
  }

  /* ----- Événements d'un pas du film ----- */
  function attaqueEnCours(par) {
    const ms = [...plateau().querySelectorAll(`#milieu .maillon[data-o="${par}"]`)];
    const m = ms[ms.length - 1];
    return m ? m.querySelector("[data-k]") : null;
  }
  function frappe(e) {
    const a = attaqueEnCours(e.par), h = ancre("heros-" + e.cible);
    if (!a || !h) return;
    const ra = rect(a), rh = rect(h), dir = e.cible === "vous" ? 1 : -1;
    const c = double(a, ra); a.style.visibility = "hidden";
    const dx = rh.cx - ra.cx, dy = rh.cy - ra.cy;
    const coup = `translate(${dx}px,${dy}px) rotate(${12 * dir}deg) scale(.8)`;
    const an = c.animate([{ transform: "none" }, { transform: `translate(${-dx * 0.07}px,${-dy * 0.1}px) rotate(${-9 * dir}deg) scale(1.07)`, offset: 0.4 }, { transform: coup, offset: 0.64 }, { transform: coup, offset: 0.7 }, { transform: "none" }], { duration: 700, fill: "both" });
    const fin = () => { c.remove(); a.style.visibility = ""; };
    an.onfinish = fin; setTimeout(fin, 820);
    setTimeout(() => { gerbe(rh.cx, rh.cy, 80, FEU, 16, 0.3, 4); secousse(); }, 450);
  }
  function evenements(liste) {
    if (!on() || !liste) return 0;
    let duree = 0;
    const frappee = new Set();
    for (const e of liste) {
      if (e.t === "tour") { banniere(e.par === "vous" ? "Votre tour" : "Tour du bot", e.par === "vous" ? "var(--mc)" : "var(--oc)"); duree = Math.max(duree, 900); }
      else if (e.t === "attaque") {
        const el = e.carte != null ? cle(e.carte) : attaqueEnCours(e.par);
        if (el) { const r = rect(el); setTimeout(() => { gerbe(r.cx, r.y + r.h * 0.95, 26, POUSSIERE, 8, 0.25, 3); classe(el, "slam", 450); }, 380); }
        duree = Math.max(duree, 700);
      } else if (e.t === "touche" && !e.allie) {
        frappe(e); frappee.add(e.cible); duree = Math.max(duree, 1000);
      } else if (e.t === "degats" && !e.allie) {
        const h = ancre("heros-" + e.cible), d = frappee.has(e.cible) ? 450 : 0;
        if (h) setTimeout(() => { const r = rect(h); flotter("−" + e.n, r.cx, r.cy, e.n >= 4 ? "big" : ""); if (!d) gerbe(r.cx, r.cy, 34, FEU, 10, 0.2, 3); classe(h, "touche", 520); }, d);
        duree = Math.max(duree, d + 700);
      } else if (e.t === "bloque") {
        const h = ancre("heros-" + e.cible);
        if (h) setTimeout(() => { const r = rect(h); gerbe(r.cx, r.cy, 30, ACIER, 9, 0, 3); flotter("Bloqué", r.cx, r.y, "plus"); }, 250);
        duree = Math.max(duree, 800);
      } else if (e.t === "pitch") {
        const c = e.carte != null ? cle(e.carte) : null, b = ancre("res-" + e.par);
        if (c && b) { const p = rect(c), g = rect(b); setTimeout(() => { gerbe(p.cx, p.cy, 14, ["#ff7a66", "#fff"], 5, 0, 2.4); flux(p.cx, p.cy, g.cx, g.cy, 18, "#ffb03a"); }, 200); }
        duree = Math.max(duree, 500);
      }
    }
    return duree;
  }

  /* ----- Tirer une carte : chaîne de laiton, lâcher pour jouer ----- */
  let tir = null, garde = 0;
  const svg = () => document.getElementById("visee");
  function chaine(de, x, y) {
    let s = svg();
    if (!s) {
      const d = document.createElement("div");
      d.innerHTML = '<svg id="visee" aria-hidden="true"><path id="v1"/><path id="v2"/><path id="v3"/><polygon id="vbout" points="0,-7 10,0 0,7 -10,0"/></svg>';
      couche().appendChild(d.firstChild); s = svg();
    }
    const sag = Math.min(22, Math.hypot(x - de.cx, y - de.cy) * 0.045);
    const d = `M${x},${y} Q${(de.cx + x) / 2},${(de.cy + y) / 2 + sag} ${de.cx},${de.cy}`;
    for (const n of ["v1", "v2", "v3"]) document.getElementById(n).setAttribute("d", d);
    document.getElementById("vbout").setAttribute("transform", `translate(${x},${y}) rotate(${Math.atan2(y - de.cy, x - de.cx) * 57.3})`);
    s.classList.add("on");
  }
  // Les dépôts possibles d'une carte : zone → option
  function depots(id) {
    const opts = optionsDeCarte(Number(id));
    const m = new Map();
    for (const o of opts) {
      const z = o.action === "pitch" ? "pitch" : o.action === "arsenal" ? "arsenal" : ["jouer", "defendre", "activer"].includes(o.action) ? "chaine" : null;
      if (z && !m.has(z)) m.set(z, o);
    }
    return m;
  }
  function eclairer(t, oui) {
    for (const z of plateau().querySelectorAll("[data-depot]")) z.classList.toggle("depot", oui && t.depots.has(z.dataset.depot));
  }
  function sous(x, y) {
    for (const el of document.elementsFromPoint(x, y)) { const z = el.closest && el.closest("[data-depot]"); if (z) return z.dataset.depot; }
    return null;
  }
  document.addEventListener("pointerdown", (e) => {
    if (tir || e.button > 0 || occupe || !e.target.closest) return;
    const el = e.target.closest("#j-vous .carte.jouable[data-k]");
    if (!el) return;
    const d = depots(el.dataset.k); if (!d.size) return;
    tir = { el, id: e.pointerId, sx: e.clientX, sy: e.clientY, bouge: false, depots: d, r: rect(el) };
  });
  window.addEventListener("pointermove", (e) => {
    const t = tir; if (!t || e.pointerId !== t.id) return;
    if (!t.bouge) {
      if (Math.hypot(e.clientX - t.sx, e.clientY - t.sy) < 8) return;
      t.bouge = true; t.el.classList.add("tiree"); eclairer(t, true);
      try { t.el.setPointerCapture(t.id); } catch (err) { /* rien */ }
    }
    t.el.style.transform = `translate(${e.clientX - t.sx}px,${e.clientY - t.sy}px) scale(1.08) rotate(${Math.max(-10, Math.min(10, (e.clientX - t.sx) * 0.03))}deg)`;
    chaine(t.r, e.clientX, e.clientY);
    const z = sous(e.clientX, e.clientY);
    for (const x of plateau().querySelectorAll("[data-depot]")) x.classList.toggle("vise", x.dataset.depot === z && t.depots.has(z));
  });
  function lacher(e) {
    const t = tir; if (!t || e.pointerId !== t.id) return;
    tir = null; if (!t.bouge) return;
    garde = T() + 350;
    eclairer(t, false);
    for (const x of plateau().querySelectorAll(".vise")) x.classList.remove("vise");
    const s = svg(); if (s) s.classList.remove("on");
    t.el.style.pointerEvents = "none";
    const z = sous(e.clientX, e.clientY);
    t.el.style.pointerEvents = "";
    const o = z ? t.depots.get(z) : null;
    t.el.classList.remove("tiree");
    if (o) { t.el.style.transform = ""; deposerOption(o); }
    else { t.el.style.transition = "transform .25s ease-out"; t.el.style.transform = ""; setTimeout(() => { t.el.style.transition = ""; }, 260); }
  }
  window.addEventListener("pointerup", lacher); window.addEventListener("pointercancel", lacher);
  document.addEventListener("click", (e) => { if (T() < garde) { e.stopPropagation(); e.preventDefault(); } }, true);
  document.addEventListener("dragstart", (e) => { if (e.target.closest && e.target.closest("#plateau")) e.preventDefault(); });

  return {
    on,
    avant() { avantVols = on() ? releve() : new Map(); },
    apres(e) {
      if (!on()) { avantVols = new Map(); etape = e && e.combat ? e.combat.etape : null; return; }
      try {
        if (premier) { premier = false; } else vols();
        bandeauEtape(e);
      } catch (err) { console.error(err); }
    },
    evenements,
    vieAffichee,
    // Nouveau match ou nouvelle manche : rien ne vole, les PV repartent sans défiler.
    oublier() { premier = true; vies.vous = vies.bot = null; etape = null; },
  };
})();

const Film = (() => {
  let dernier = null, sauter = false, reprendre = null;
  const cle = (e) => `${e.reglages && e.reglages.graine}|${e.match ? e.match.jouees : 0}|${e.phase}|${e.film_no}`;
  const attendre = (ms) => new Promise((r) => { const t = setTimeout(r, ms); reprendre = () => { clearTimeout(t); r(); }; });
  function bouton(oui) {
    let b = document.getElementById("film-passer");
    if (!b) {
      b = document.createElement("button"); b.id = "film-passer"; b.type = "button"; b.textContent = "Passer l'animation ▸▸";
      b.addEventListener("click", () => { sauter = true; if (reprendre) reprendre(); });
      document.getElementById("v-partie").appendChild(b);
    }
    b.hidden = !oui;
  }
  // Pause avant les dégâts : bouton « Continuer » (ou Espace / Entrée)
  function continuer() {
    return new Promise((r) => {
      $("question").textContent = "Les blocages sont posés : les dégâts vont être infligés.";
      const o = $("options"); o.replaceChildren();
      const b = el("button", "principal", "Continuer ▸");
      const fin = () => { document.removeEventListener("keydown", clavier, true); r(); };
      const clavier = (e) => { if (e.key === " " || e.key === "Enter") { e.preventDefault(); e.stopPropagation(); fin(); } };
      b.addEventListener("click", fin);
      b.disabled = false;
      o.appendChild(b); b.focus({ preventScroll: true });
      document.addEventListener("keydown", clavier, true);
      reprendre = fin;
    });
  }
  // Pendant le film, on ne peut rien jouer : les cartes ne sont plus surlignées.
  const plateauFilm = (oui) => { const p = document.getElementById("plateau"); if (p) p.classList.toggle("film", oui); };
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && reprendre) { sauter = true; reprendre(); } });
  return {
    vu(e) { if (e) dernier = cle(e); },
    async jouer(nouveau) {
      if (!nouveau || nouveau.film_no === undefined) return;
      const k = cle(nouveau);
      if (k === dernier) return;
      const changement = !etat || !etat.reglages || etat.reglages.graine !== nouveau.reglages.graine || (etat.match && nouveau.match && etat.match.jouees !== nouveau.match.jouees) || etat.phase !== nouveau.phase;
      dernier = k;
      if (changement) FX.oublier();
      const film = nouveau.film || [];
      if (!FX.on() || !film.length || nouveau.phase !== "partie") return;
      sauter = false; bouton(true);
      plateauFilm(true);
      try {
        for (const image of film) {
          const ev = image.evenements || [];
          const degats = ev.some((x) => (x.t === "touche" || x.t === "bloque") && !x.allie);
          if (degats && ReglagesFx.pause && !sauter) await continuer();
          etat = Object.assign({}, nouveau, image.plateau, { decision: null, fini: false, resultat: null, manche_suivante: false });
          afficher();
          document.querySelectorAll("#v-partie aside button, #options button").forEach((b) => { b.disabled = true; });
          const d = FX.evenements(ev);
          if (!sauter) await attendre(ReglagesFx.rapide ? Math.max(200, d * 0.5) : Math.max(420, d));
        }
      } finally { reprendre = null; bouton(false); plateauFilm(false); }
    },
  };
})();
