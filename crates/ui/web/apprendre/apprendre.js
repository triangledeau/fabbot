// Didacticiel « Apprendre » (volet #apprendre ; docs/didacticiel.md).
// Les leçons (lecons.js) se jouent sur un TAPIS animé : les vraies cartes
// volent de zone en zone, les PV défilent, le guide parle dans un bandeau.
// Les scènes « sur le moteur » (plan.scene) tournent sur le vrai moteur
// fabbot (crates/ui/src/api_apprendre.rs) : la page n'applique alors
// aucune règle, elle montre l'état que le moteur renvoie.
"use strict";
(() => {
  const sec = document.getElementById("v-apprendre");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const attendre = (ms) => new Promise((r) => setTimeout(r, ms));
  const CLE = "arsenal.apprendre.v2";
  const lire = () => { try { return JSON.parse(localStorage.getItem(CLE)) || {}; } catch (e) { return {}; } };
  const ecrire = (p) => { try { localStorage.setItem(CLE, JSON.stringify(p)); } catch (e) { /* stockage indisponible */ } };
  const api = async (methode, chemin) => {
    const r = await fetch(chemin, { method: methode });
    const v = await r.json();
    if (!r.ok || v.erreur) throw new Error(v.erreur || r.status);
    return v;
  };

  // ——— Icônes officielles (site des règles de LSS, chargées en ligne ; symbole dessiné sinon) ———
  const NOMS_ICONES = { p: "puissance", d: "défense", r: "ressource", h: "vie", i: "intellect" };
  const icone = (k, cls = "") => k ? `<img class="ico ${cls}" src="https://rules.fabtcg.com/en/assets/images/icon_${k}.png" alt="${NOMS_ICONES[k] || ""}" referrerpolicy="no-referrer" onerror="this.outerHTML=Symboles.html('{${k}}')">` : "";
  // {{notion}} ou {{notion|forme}} → mot souligné, avec son icône et sa définition au survol.
  // Typographie française : espace insécable avant « : ; ? ! » et après « « ».
  const fine = (t) => String(t ?? "").replace(/ ([:;?!»])/g, "\u00a0$1").replace(/« /g, "«\u00a0");
  const rendre = (t) => fine(t).replace(/\{\{(\w+)(?:\|([^}]+))?\}\}/g, (m, n, forme) => {
    const d = NOTIONS[n];
    return d ? `<span class="ap-mot" tabindex="0" data-def="${esc(d.def)}">${icone(d.icone)}${esc(forme || d.mot)}</span>` : m;
  });
  const crs = (l) => (l && l.length) ? `<span class="ap-cr" title="Comprehensive Rules officielles">${l.map((x) => "CR " + esc(x)).join(" · ")}</span>` : "";

  // ——— Le tapis : 1280 × 720 unités, mis à l'échelle de la fenêtre ———
  const W = 1280, H = 720, CW = 140, CH = 196;
  const ZONES = {
    "adv.heros": [570, 22], "moi.heros": [570, 488],
    "chaine": [470, 262], "def.0": [640, 262], "def.1": [790, 262], "def.2": [940, 262],
    "moi.pitch": [400, 488], "adv.pitch": [400, 22],
    "moi.arsenal": [1110, 262], "pile": [300, 262], "maillon.0": [320, 262], "maillon.1": [470, 262], "moi.cimetiere": [230, 488], "adv.cimetiere": [230, 22],
    "moi.deck": [60, 488], "adv.deck": [60, 22],
    "centre": [570, 120], "trio.0": [330, 200], "trio.1": [570, 200], "trio.2": [810, 200],
  };
  // Le plan complet des zones (chapitre 4) : cases de 105 × 147 (cartes à 75 %),
  // x, y, largeur, hauteur. Les zones communes sont en haut.
  const PLAN = {
    "plan.pile": [40, 24, 105, 147, "Pile"], "plan.chaine": [290, 24, 450, 147, "Chaîne de combat"], "plan.permanents": [800, 24, 440, 147, "Permanents"],
    "plan.tete": [40, 250, 105, 147, "Tête"], "plan.torse": [165, 250, 105, 147, "Torse"], "plan.bras": [290, 250, 105, 147, "Bras"], "plan.jambes": [415, 250, 105, 147, "Jambes"],
    "plan.arme1": [560, 250, 105, 147, "Arme"], "plan.arme2": [690, 250, 105, 147, "Arme"],
    "plan.deck": [40, 470, 105, 147, "Deck"], "plan.cimetiere": [165, 470, 105, 147, "Cimetière"], "plan.bannies": [290, 470, 105, 147, "Bannies"], "plan.pitch": [415, 470, 105, 147, "Pitch"],
    "plan.heros": [625, 470, 105, 147, "Héros"], "plan.arsenal": [790, 470, 105, 147, "Arsenal"], "plan.main": [930, 470, 310, 147, "Main"],
  };
  const PETIT = 0.75, DX = CW * (1 - PETIT) / 2, DY = CH * (1 - PETIT) / 2;
  const zone = (z) => {
    if (ZONES[z]) return ZONES[z];
    if (PLAN[z]) return [PLAN[z][0] - DX, PLAN[z][1] - DY];
    let p = /^plan\.(main|chaine|permanents)\.(\d+)$/.exec(z);
    if (p) { const [x, y] = PLAN["plan." + p[1]]; return [x - DX + (p[1] === "main" ? 102 : 115) * +p[2] + (p[1] === "main" ? 0 : 10), y - DY]; }
    p = /^plan\.cimetiere\.(\d+)$/.exec(z);
    if (p) { const [x, y] = PLAN["plan.cimetiere"]; return [x - DX + 8 * +p[1], y - DY - 8 * +p[1]]; }
    if (z === "plan.armes") return [(PLAN["plan.arme1"][0] + PLAN["plan.arme2"][0] + 105) / 2 - 52.5 - DX, PLAN["plan.arme1"][1] - DY];
    let m = /^moi\.main\.(\d+)$/.exec(z); if (m) return [760 + 120 * +m[1], 493];
    m = /^adv\.main\.(\d+)$/.exec(z); if (m) return [920 + 70 * +m[1], 18];
    m = /^moi\.pitch\.(\d+)$/.exec(z); if (m) return [400, 488 - 26 * +m[1]];
    // Cimetière : chaque carte un peu décalée, pour qu'on voie qu'il y en a plusieurs.
    m = /^(moi|adv)\.cimetiere\.(\d+)$/.exec(z);
    if (m) { const [x, y] = ZONES[m[1] + ".cimetiere"], k = +m[2]; return [x + 10 * k, y + (m[1] === "moi" ? -10 : 10) * k]; }
    return [W / 2 - CW / 2, H / 2 - CH / 2];
  };
  const GRANDES = { centre: 2.1, "trio.0": 1.45, "trio.1": 1.45, "trio.2": 1.45 };
  // Coins d'une carte (en % de sa largeur / hauteur) pour les zooms.
  const COINS = { pitch: [11, 6], cout: [89, 6], puissance: [10, 93], defense: [90, 93], vie: [90, 93], intellect: [10, 93], type: [50, 92], texte: [50, 72], nom: [50, 5] };
  // Zones de texte d'une carte (x, y, largeur, hauteur en %) : surlignées par un cadre.
  // Les sous-zones (classe, sorte, règles, ambiance) sont mesurées sur la carte
  // du chapitre 2, Décharge de la lame d'acier (scan français).
  const RECTS = { type: [21, 88.8, 58, 5.6], texte: [7, 62, 86, 23], nom: [20, 2.5, 60, 6],
    classe: [26.5, 88.8, 17, 5.6], sorte: [41, 88.8, 34, 5.6], regles: [7, 63, 86, 11], ambiance: [7, 74.5, 86, 11] };
  const ICONE_COIN = { pitch: "r", cout: "r", puissance: "p", defense: "d", vie: "h", intellect: "i" };

  const fiches = {};
  const fiche = (spec) => {
    const k = spec.nom + "|" + (spec.couleur || "");
    return fiches[k] || (fiches[k] = api("GET", `/api/apprendre/fiche?nom=${encodeURIComponent(spec.nom)}&couleur=${encodeURIComponent(spec.couleur || "")}`).catch(() => ({ nom: spec.nom })));
  };

  // Le didacticiel nomme les cartes comme elles sont imprimées. Brutal Assault
  // rouge et jaune n'existent qu'en anglais : la bleue est montrée elle aussi
  // en version anglaise, pour que les trois couleurs portent le même nom.
  const VO = { "Brutal Assault|bleu": "IRA010" };
  function carteHtml(c) {
    const vo = VO[c.nom + "|" + c.couleur];
    if (vo) c = { ...c, image: vo, nom_fr: null };
    const coul = c.couleur ? ` coul-${esc(c.couleur)}` : "";
    const v = (k, x) => x != null ? `<span class="v v-${k}">${icone(k)}<b>${x}</b></span>` : "";
    const cadre = `<div class="cadre${coul}"><div class="nom">${esc(c.nom_fr || c.nom)}</div><div class="vals">${v("r", c.cout)}${v("p", c.puissance)}${v("d", c.defense)}${v("h", c.vie)}${v("i", c.intellect)}</div></div>`;
    const img = c.image ? `<img class="img" alt="${esc(c.nom)}" referrerpolicy="no-referrer" src="${esc(Images.url(c.image, "normal"))}" onerror="this.remove()">` : "";
    return cadre + img;
  }

  const RESERVE_GUIDE = 230;
  // Une « table » : les éléments posés sur le tapis, repérés par un id.
  function Tapis(racine) {
    const el = {};
    let vite = false;
    const tapis = racine.querySelector(".ap-tapis");
    // Le tapis tient dans la fenêtre avec le guide (texte + boutons) en dessous :
    // on réserve la place du guide, pour ne jamais avoir à faire défiler la page.
    const ajuster = () => {
      const parent = racine.parentElement; if (!parent) return;
      const haut = racine.getBoundingClientRect().top + window.scrollY;
      const g = parent.querySelector(".ap-guide");
      const dispo = window.innerHeight - haut - Math.max(RESERVE_GUIDE, (g ? g.offsetHeight : 0) + 24);
      const s = Math.max(0.4, Math.min(parent.clientWidth / W, dispo / H));
      tapis.style.setProperty("--s", s);
      racine.style.width = (W * s) + "px"; racine.style.height = (H * s) + "px";
    };
    const ro = new ResizeObserver(ajuster); ro.observe(racine.parentElement || racine);
    const gd = racine.parentElement?.querySelector(".ap-guide"); if (gd) ro.observe(gd);
    window.addEventListener("resize", ajuster); ajuster();
    const duree = (d) => vite ? 0 : d;
    function placer(id, z, d = 650, echelle) {
      const e = el[id]; if (!e) return attendre(0);
      const [x, y] = zone(z);
      e.z = z;
      const s = echelle ?? GRANDES[z] ?? (z.startsWith("plan.") ? PETIT : 1);
      e.n.style.transition = `transform ${duree(d)}ms cubic-bezier(.2,.75,.2,1), opacity ${duree(d)}ms`;
      e.n.style.transform = `translate(${x}px, ${y}px) scale(${s})`;
      e.n.style.zIndex = z.includes(".cimetiere.") ? 10 + (+z.split(".")[2]) : z.startsWith("moi.main") ? 20 + (+z.split(".")[2]) : (GRANDES[z] ? 30 : (z === "chaine" || z.startsWith("def") ? 25 : 10));
      return attendre(duree(d));
    }
    async function creer(id, spec, z, depuis, opts = {}) {
      retirer(id);
      const n = document.createElement("div");
      n.className = "ct" + (spec ? "" : " dos");
      n.dataset.id = id;
      n.innerHTML = spec ? carteHtml(await fiche(spec)) : "";
      tapis.appendChild(n);
      el[id] = { n, spec };
      const [x0, y0] = depuis || (z.startsWith("adv") ? [zone(z)[0], -260] : [zone(z)[0], H + 60]);
      n.style.transform = `translate(${x0}px, ${y0}px) scale(1)`;
      n.style.opacity = "0";
      n.getBoundingClientRect();
      n.style.opacity = "1";
      return placer(id, z, opts.vite ? 250 : 750);
    }
    function retirer(id) { if (el[id]) { el[id].n.remove(); delete el[id]; } }
    function vider() { for (const id of Object.keys(el)) retirer(id); tapis.querySelectorAll(".ap-zoom, .ap-flot, .ap-bulle, .ap-slot, .ap-recap, .ap-fin, .ap-jauge, .ap-points, .ap-frise, .ap-note").forEach((x) => x.remove()); }
    // Une zone du tapis, dessinée en pointillés avec son nom. Elle apparaît la
    // première fois qu'on en parle et reste ensuite ; `eclairer` la fait briller.
    async function slot(z, nom, eclairer = true) {
      let n = tapis.querySelector(`.ap-slot[data-z="${z}"]`);
      if (!n) {
        const r = PLAN[z], [x, y] = r || zone(z);
        n = document.createElement("div"); n.className = "ap-slot"; n.dataset.z = z;
        n.style.left = x + "px"; n.style.top = y + "px";
        if (r) { n.style.width = r[2] + "px"; n.style.height = r[3] + "px"; }
        n.innerHTML = `<span>${esc(nom || (r && r[4]) || "")}</span>`;
        tapis.prepend(n);
      }
      if (eclairer && !vite) { n.classList.remove("eclaire"); n.getBoundingClientRect(); n.classList.add("eclaire"); await attendre(900); }
    }
    // PV d'un héros : pastille sur la carte, chiffres qui défilent.
    async function vie(id, v, lent) {
      const e = el[id]; if (!e) return;
      let p = e.n.querySelector(".pv");
      if (!p) { p = document.createElement("div"); p.className = "pv"; p.innerHTML = `${icone("h")}<b>${v}</b>`; e.n.appendChild(p); e.vie = v; return; }
      const de = e.vie ?? v; e.vie = v;
      const b = p.querySelector("b");
      if (vite || de === v) { b.textContent = v; return; }
      const pas = de > v ? -1 : 1, t = lent ? 90 : 55;
      p.classList.add(de > v ? "baisse" : "hausse");
      for (let k = de; k !== v; k += pas) { b.textContent = k + pas; await attendre(t); }
      p.classList.remove("baisse", "hausse");
    }
    function flotter(id, texte, cls) {
      if (vite) return;
      const e = el[id]; if (!e) return;
      const f = document.createElement("div"); f.className = "ap-flot " + (cls || ""); f.innerHTML = texte;
      const [x, y] = zone(e.z);
      f.style.left = (x + CW / 2) + "px"; f.style.top = (y + CH * 0.35) + "px";
      tapis.appendChild(f); setTimeout(() => f.remove(), 1600);
    }
    async function secouer(id, cls = "choc") {
      const e = el[id]; if (!e || vite) return;
      e.n.classList.add(cls); await attendre(450); e.n.classList.remove(cls);
    }
    // Zoom : la carte en grand, un anneau sur le coin, une étiquette avec l'icône.
    async function zoom(id, coin, label) {
      const e = el[id]; if (!e) return;
      tapis.querySelectorAll(".ap-zoom").forEach((x) => x.remove());
      const z = document.createElement("div"); z.className = "ap-zoom";
      // Une zone de texte (nom, ligne de type, texte) : un cadre ajusté à la
      // zone, à droite de la carte. Une icône (coin) : un cercle.
      const rect = RECTS[coin];
      if (rect) {
        const [x0, y0, w0, h0] = rect, ZW = 458, ZH = 641, L = 230;
        const bout = L + (x0 + w0) / 100 * ZW, ry = 40 + (y0 + h0 / 2) / 100 * ZH, fin = 740;
        z.innerHTML = `<div class="zc ct" style="left:${L}px">${e.n.innerHTML}<div class="surligne" style="left:${x0}%;top:${y0}%;width:${w0}%;height:${h0}%"></div></div>
          <div class="trait" style="left:${bout}px;top:${ry - 1}px;width:${fin - bout}px"></div>
          <div class="etiq" style="left:${fin}px;top:${ry - 25}px"><span>${esc(label || "")}</span></div>`;
        z.querySelector(".pv")?.remove();
        tapis.appendChild(z);
        if (!vite) { z.getBoundingClientRect(); z.classList.add("ouvert"); await attendre(1400); } else z.classList.add("ouvert");
        return;
      }
      const [cx, cy] = COINS[coin] || [50, 50];
      // Carte agrandie (140×196 × 3.27) : à droite si le coin est à gauche, pour
      // que l'étiquette et son trait pointent vers le bon coin.
      const ZW = 458, ZH = 641, R = 49, gauche = cx < 50;
      const L = gauche ? 700 : 230, rx = L + cx / 100 * ZW, ry = 40 + cy / 100 * ZH;
      const bout = gauche ? rx - R : rx + R, fin = gauche ? bout - 80 : bout + 80;
      const posEtiq = gauche ? `right:${1280 - fin}px` : `left:${fin}px`;
      z.innerHTML = `<div class="zc ct" style="left:${L}px">${e.n.innerHTML}<div class="anneau" style="left:${cx}%;top:${cy}%"></div></div>
        <div class="trait" style="left:${Math.min(bout, fin)}px;top:${ry - 1}px;width:80px"></div>
        <div class="etiq${gauche ? " g" : ""}" style="${posEtiq};top:${ry - 25}px">${icone(ICONE_COIN[coin], "grande")}<span>${esc(label || "")}</span></div>`;
      z.querySelector(".pv")?.remove();
      tapis.appendChild(z);
      if (!vite) { z.getBoundingClientRect(); z.classList.add("ouvert"); await attendre(1400); } else z.classList.add("ouvert");
    }
    function fermerZoom() { tapis.querySelectorAll(".ap-zoom").forEach((x) => { x.classList.remove("ouvert"); setTimeout(() => x.remove(), 300); }); }
    function bulle(id, coin, texte) {
      const e = el[id]; if (!e) return;
      const b = document.createElement("div"); b.className = "ap-bulle"; b.innerHTML = `${icone(ICONE_COIN[coin])}<b>${esc(texte)}</b>`;
      const [x, y] = zone(e.z), s = GRANDES[e.z] || 1, [cx, cy] = COINS[coin];
      b.style.left = (x + CW / 2 + (cx / 100 - .5) * CW * s) + "px"; b.style.top = (y + CH / 2 + (cy / 100 - .5) * CH * s) + "px";
      tapis.appendChild(b);
    }
    return { el, placer, creer, retirer, vider, slot, vie, flotter, secouer, zoom, fermerZoom, bulle, tapis, set vite(v) { vite = v; }, get vite() { return vite; } };
  }

  // ——— Les actions animées des plans (lecons.js, champ `fait`) ———
  let main = 0, pitchs = 0, ressources = 0;
  const ACTIONS = {
    vider: (t) => t.vider(),
    zone: (t, z, nom, calme) => t.slot(z, nom, !calme),
    // Plusieurs zones du plan d'un coup (sans pulsation) : ["plan", ["plan.tete", …]].
    plan: async (t, zs) => { for (const z of zs) await t.slot(z, null, false); },
    // Met en avant des zones (les autres pâlissent) ; ["voir"] remet tout à égalité.
    voir: async (t, zs = []) => {
      t.tapis.querySelectorAll(".ap-slot").forEach((n) => {
        const dedans = zs.includes(n.dataset.z);
        n.classList.toggle("vu", dedans); n.classList.toggle("terne", zs.length > 0 && !dedans);
        if (dedans && !t.vite) { n.classList.remove("eclaire"); n.getBoundingClientRect(); n.classList.add("eclaire"); }
      });
      if (!t.vite && zs.length) await attendre(900);
    },
    // Petite étiquette sur des zones (« Publique », « Privée ») ; ["etiquette"] les retire.
    etiquette: (t, zs = [], texte, cls = "") => {
      if (!zs.length) { t.tapis.querySelectorAll(".ap-slot .tag").forEach((x) => x.remove()); return; }
      for (const z of zs) {
        const n = t.tapis.querySelector(`.ap-slot[data-z="${z}"]`); if (!n) continue;
        n.querySelector(".tag")?.remove();
        n.insertAdjacentHTML("beforeend", `<b class="tag ${esc(cls)}">${esc(texte)}</b>`);
      }
    },
    // Une carte mise en valeur (sans être cliquable) ; ["lueur", id, false] l'éteint.
    lueur: (t, id, oui = true) => { t.el[id]?.n.classList.toggle("lueur", oui); },
    retirer: (t, ...ids) => { for (const id of ids) t.retirer(id); },
    // La carte est retournée face cachée.
    cacher: (t, id) => { const e = t.el[id]; if (e) { e.n.classList.add("dos"); e.n.innerHTML = ""; } },
    // Une carte arrive depuis le deck.
    tirer: (t, id, spec, z) => t.creer(id, spec, z, zone(z.startsWith("plan.") ? "plan.deck" : z.startsWith("adv") ? "adv.deck" : "moi.deck"), {}),
    // La carte retourne sous le deck, face cachée.
    sous_deck: async (t, id, deck = "moi.deck") => {
      ACTIONS.cacher(t, id); await t.placer(id, deck, 550); t.retirer(id);
    },
    // Frise des phases ou des étapes, à droite du tapis ; i = étape en cours (-1 : aucune).
    frise: async (t, noms, i = -1) => {
      let f = t.tapis.querySelector(".ap-frise");
      if (!f) { f = document.createElement("ol"); f.className = "ap-frise"; t.tapis.appendChild(f); }
      f.innerHTML = noms.map((n, k) => `<li class="${k < i ? "passe" : k === i ? "ici" : ""}">${esc(n)}</li>`).join("");
      if (!t.vite) await attendre(500);
    },
    // Une note posée sur le tapis (fond sombre) ; ["note", id] sans texte la retire.
    note: (t, id, texte, x, y) => {
      t.tapis.querySelector(`.ap-note[data-id="${id}"]`)?.remove();
      if (!texte) return;
      const n = document.createElement("div"); n.className = "ap-note"; n.dataset.id = id;
      n.style.left = x + "px"; n.style.top = y + "px"; n.innerHTML = fine(esc(texte));
      t.tapis.appendChild(n);
    },
    // Un texte qui s'élève au-dessus d'une carte (« +1 », « Bloqué »…).
    flot: (t, id, texte, cls) => t.flotter(id, texte, cls),
    poser: (t, id, spec, z, o = {}) => t.creer(id, spec, z, null, o),
    dos: (t, id, z) => t.creer(id, null, z, null, { vite: true }),
    deplacer: (t, id, z) => t.placer(id, z),
    retour: (t, id, z) => t.placer(id, z, 400),
    vie: (t, id, v, o = {}) => t.vie(id, v, o.lent),
    zoom: (t, id, coin, label) => t.zoom(id, coin, label),
    bulle: async (t, id, coin, txt) => { t.bulle(id, coin, txt); if (!t.vite) await attendre(350); },
    perd: async (t, id) => { t.el[id]?.n.classList.add("vaincu"); if (!t.vite) await attendre(700); },
    releve: async (t, id) => { t.el[id]?.n.classList.remove("vaincu"); },
    cimetiere: (t, id) => {
      const qui = id.startsWith("x") ? "adv" : "moi";
      const k = Object.values(t.el).filter((e) => e.z?.startsWith(qui + ".cimetiere")).length;
      return t.placer(id, qui + ".cimetiere." + k, 600);
    },
    jouer: async (t, id, spec, z) => {
      const e = t.el[id];
      if (spec && e) { e.n.classList.remove("dos"); e.n.innerHTML = carteHtml(await fiche(spec)); e.spec = spec; }
      await t.placer(id, z, 700);
    },
    // L'attaque bondit vers le héros visé ; les PV baissent du nombre de dégâts.
    attaquer: async (t, id, cible, degats) => {
      if (t.el[id]?.z !== "chaine") await t.placer(id, "chaine", 650);
      const e = t.el[id], h = t.el[cible];
      if (e && h && !t.vite) {
        const [x, y] = zone("chaine"), [hx, hy] = zone(h.z);
        e.n.style.transition = "transform 180ms ease-in";
        e.n.style.transform = `translate(${x + (hx - x) * 0.55}px, ${y + (hy - y) * 0.55}px) scale(1.08) rotate(-4deg)`;
        await attendre(190);
        t.secouer(cible); t.flotter(cible, `−${degats}`, "degats");
        await t.placer(id, "chaine", 300);
      }
      const v = (t.el[cible]?.vie ?? 20) - degats;
      await t.vie(cible, Math.max(0, v));
    },
    defendre: (t, id, k) => t.placer(id, "def." + k, 550),
    impact: async (t, cible, degats) => {
      if (!t.vite) { t.secouer(cible); t.flotter(cible, `−${degats}`, "degats"); }
      await t.vie(cible, (t.el[cible]?.vie ?? 20) - degats);
    },
    bouclier: async (t) => { if (!t.vite) { t.flotter("moi", "Bloqué !", "bloque"); await attendre(600); } },
    deck: async (t, qui) => { for (let k = 0; k < 3; k++) await t.creer(qui + "-deck-" + k, null, qui + ".deck", [zone(qui + ".deck")[0] + k * 2, H + 60], { vite: true }); },
    piocher: async (t, n) => {
      for (let k = 0; k < n; k++) {
        await t.creer("p" + k, { nom: ["Raging Onslaught", "Brutal Assault", "Brutal Assault", "Brutal Assault"][k], couleur: ["rouge", "rouge", "jaune", "bleu"][k] }, "moi.main." + k, zone("moi.deck"), { vite: false });
      }
    },
    pitcher: async (t, id, n) => {
      await t.placer(id, "moi.pitch." + pitchs++, 600);
      ressources += n; jauge(t);
      t.flotter(id, `+${n} ${icone("r")}`, "gain");
    },
    payer: async (t, n) => { if (!t.vite) await attendre(500); ressources -= n; jauge(t); },
    ressources: (t, n) => { ressources = n; jauge(t); },
    points: async (t, n) => { pointsAction(t, n); if (!t.vite) await attendre(500); },
    // Fin du tour : les ressources non dépensées sont perdues.
    finTour: async (t) => {
      if (!t.vite && ressources) { t.flotter("moi", `−${ressources} ${icone("r")}`, "degats"); await attendre(700); }
      ressources = 0; jauge(t);
    },
  };
  function compteur(t, cls, html) {
    let j = t.tapis.querySelector("." + cls);
    if (!j) { j = document.createElement("div"); j.className = cls; t.tapis.appendChild(j); }
    j.innerHTML = html;
    if (!t.vite) { j.classList.remove("bouge"); j.getBoundingClientRect(); j.classList.add("bouge"); }
  }
  const jauge = (t) => compteur(t, "ap-jauge", `${icone("r")}<b>${ressources}</b><span>ressource${ressources > 1 ? "s" : ""}</span>`);
  const pointsAction = (t, n) => compteur(t, "ap-points", `<i class="pa${n ? "" : " vide"}"></i><b>${n}</b><span>point${n > 1 ? "s" : ""} d'action</span>`);
  async function faire(t, actions) {
    for (const [nom, ...args] of actions || []) {
      if (!ACTIONS[nom]) { console.warn("action inconnue", nom); continue; }
      await ACTIONS[nom](t, ...args);
    }
  }

  // ——— Sommaire ———
  let lecon = null, k = 0, tapis = null, jeton = 0;
  function sommaire() {
    const p = lire();
    lecon = null;
    const chap = LECONS.map((c) => {
      const faites = c.lecons.filter((l) => p[l.id]).length;
      const items = c.lecons.map((l, i) => `<button class="ap-pas${p[l.id] ? " fait" : ""}" data-l="${esc(l.id)}"><span class="n">${p[l.id] ? "✓" : c.no + "." + (i + 1)}</span>${esc(l.titre)}</button>`).join("");
      return `<section class="ap-chap${c.lecons.length ? "" : " vide"}" style="--p:${c.lecons.length ? faites / c.lecons.length : 0}">
        <div class="ap-chap-t"><span class="num">${c.no}</span><h2>${esc(c.titre)}</h2></div>
        ${c.lecons.length ? `<div class="ap-barre"><i></i></div><div class="ap-pas-l">${items}</div>` : `<div class="muted petit">Bientôt</div>`}</section>`;
    }).join("");
    const suite = LECONS.flatMap((c) => c.lecons).find((l) => !p[l.id]);
    sec.innerHTML = `<div class="ap">
      <header class="ap-hero"><a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a>
        <div class="ap-hero-t"><small>FLESH AND BLOOD · SILVER AGE</small><h1>Apprendre à jouer</h1>
        <p>Les règles pas à pas, sur le tapis, avec les vraies cartes. Une notion à la fois, jamais un mot avant de l'avoir vu.</p>
        ${suite ? `<button class="btn primary grand" data-l="${esc(suite.id)}">${Object.keys(p).length ? "Reprendre" : "Commencer"} : ${esc(suite.titre)} ▸</button>` : ""}</div></header>
      <div class="ap-chaps">${chap}</div>${couverture()}</div>`;
    sec.querySelectorAll("[data-l]").forEach((b) => b.addEventListener("click", () => ouvrir(b.dataset.l)));
    document.title = "Arsenal — Apprendre";
  }

  function couverture() {
    const cite = {};
    for (const c of LECONS) for (const l of c.lecons) for (const pl of l.plans) for (const r of (pl.cr || [])) {
      const s = r.split(".").slice(0, 2).join(".").replace(/[a-z]$/, "");
      (cite[s] = cite[s] || new Set()).add(l.id);
    }
    const n = CR_SECTIONS.filter(([s]) => cite[s]).length;
    const lignes = CR_SECTIONS.map(([s, t]) => `<tr class="${cite[s] ? "ok" : ""}"><td class="num">${esc(s)}</td><td>${esc(t)}</td><td>${cite[s] ? [...cite[s]].map(esc).join(", ") : '<span class="muted">pas encore</span>'}</td></tr>`).join("");
    return `<details class="ap-couv"><summary><b>Couverture des règles officielles</b> : <span class="num">${n} / ${CR_SECTIONS.length}</span> sections expliquées</summary>
      <table><thead><tr><th>CR</th><th>Section</th><th>Leçons</th></tr></thead><tbody>${lignes}</tbody></table></details>`;
  }

  // ——— Une leçon ———
  function ouvrir(id) {
    for (const c of LECONS) for (const l of c.lecons) if (l.id === id) lecon = { ...l, chap: c };
    sec.innerHTML = `<div class="ap ap-lecon">
      <header class="ap-tete"><button class="btn small" id="ap-somm">← Sommaire</button>
        <h1><span class="muted">${lecon.chap.no}. ${esc(lecon.chap.titre)}</span> · ${esc(lecon.titre)}</h1><div class="ap-prog" id="ap-prog"></div></header>
      <div class="ap-scene-cadre"><div class="ap-tapis"><div class="ap-lignes"></div></div></div>
      <div class="ap-guide" id="ap-guide"></div></div>`;
    sec.querySelector("#ap-somm").onclick = sommaire;
    tapis = Tapis(sec.querySelector(".ap-scene-cadre"));
    aller(0);
  }

  // Aller au plan n : on rejoue les plans précédents sans animation (le tapis
  // retrouve exactement son état), puis le plan n est joué normalement.
  async function aller(n) {
    const mon = ++jeton;
    k = n; main = 0; pitchs = 0; ressources = 0;
    tapis.vider(); tapis.tapis.querySelector(".ap-jauge")?.remove();
    tapis.vite = true;
    for (let i = 0; i < n; i++) { const pl = lecon.plans[i]; await faire(tapis, pl.fait); if (pl.clic) await faire(tapis, pl.apres); }
    tapis.fermerZoom();
    tapis.vite = false;
    if (mon !== jeton) return;
    jouerPlan(mon);
  }

  const prog = () => { sec.querySelector("#ap-prog").innerHTML = lecon.plans.map((_, i) => `<span class="${i < k ? "ok" : i === k ? "ici" : ""}"></span>`).join(""); };

  // Le texte du guide arrive mot à mot.
  const anime = (html) => {
    const d = document.createElement("div"); d.innerHTML = html;
    let i = 0;
    const marcher = (n) => {
      for (const c of [...n.childNodes]) {
        if (c.nodeType === 3) {
          const f = document.createDocumentFragment();
          for (const m of c.textContent.split(/([ \t\n]+)/)) {
            if (!m) continue;
            if (/^[ \t\n]+$/.test(m)) { f.appendChild(document.createTextNode(m)); continue; }
            const s = document.createElement("span"); s.className = "mt"; s.style.animationDelay = (i++ * 28) + "ms"; s.textContent = m; f.appendChild(s);
          }
          c.replaceWith(f);
        } else if (c.nodeType === 1 && !c.classList.contains("ap-mot")) marcher(c);
        else if (c.nodeType === 1) { c.classList.add("mt"); c.style.animationDelay = (i++ * 28) + "ms"; }
      }
    };
    marcher(d);
    return d.innerHTML;
  };

  function guide(html, boutons, cr) {
    const g = sec.querySelector("#ap-guide");
    g.innerHTML = `<div class="ap-dit">${anime(html)}</div><div class="ap-bas">${crs(cr)}<span class="ap-etape">Étape ${k + 1} / ${lecon.plans.length}</span><div class="ap-actions">${boutons || ""}</div></div>`;
    return g;
  }
  const INDICE_CLIC = `<span class="ap-indice">👆 Clique sur la carte qui brille</span>`;
  const navBoutons = (suivant = true) => `${k > 0 ? `<button class="btn quiet" data-nav="prec">← Revoir</button>` : ""}${suivant ? `<button class="btn primary" data-nav="suiv">Continuer ▸</button>` : ""}`;
  function brancherNav(g) {
    g.querySelector('[data-nav="prec"]')?.addEventListener("click", () => aller(k - 1));
    const s = g.querySelector('[data-nav="suiv"]');
    if (s) { s.addEventListener("click", suivant); setTimeout(() => s.focus({ preventScroll: true }), 50); }
  }
  function suivant() {
    if (k + 1 < lecon.plans.length) { tapis.fermerZoom(); aller(k + 1); }
    else fin();
  }

  async function jouerPlan(mon) {
    const pl = lecon.plans[k];
    prog();
    if (pl.scene) return scene(pl, mon);
    if (pl.quiz) { tapis.fermerZoom(); return quiz(pl); }
    if (pl.recap) {
      // Récapitulatif de fin de chapitre : une fiche sur le tapis.
      tapis.vider(); tapis.tapis.querySelector(".ap-jauge")?.remove();
      tapis.tapis.insertAdjacentHTML("beforeend", `<div class="ap-recap"><h2>À retenir</h2><ol>${pl.recap.map((r, i) => `<li style="animation-delay:${200 + i * 350}ms">${fine(esc(r))}</li>`).join("")}</ol></div>`);
      const g = guide("Voici l'essentiel du chapitre. Relis-le avant de passer à la suite.", navBoutons(), null);
      brancherNav(g);
      return;
    }
    // Les boutons sont là tout de suite : « Continuer » pendant l'animation la
    // passe ; quand l'élève doit cliquer une carte, on le lui dit en clair.
    const g = guide(rendre(pl.dit || ""), pl.clic ? navBoutons(false) + INDICE_CLIC : navBoutons(), pl.cr);
    brancherNav(g);
    await faire(tapis, pl.fait);
    if (mon !== jeton) return;
    if (!pl.clic) return;
    // L'élève doit cliquer une carte : elle s'illumine ; une autre carte tremble.
    tapis.fermerZoom();
    const cible = tapis.el[pl.clic];
    cible?.n.classList.add("appel");
    await new Promise((ok) => {
      tapis.tapis.onclick = (ev) => {
        const c = ev.target.closest(".ct"); if (!c) return;
        if (c.dataset.id === pl.clic) { tapis.tapis.onclick = null; ok(); }
        else tapis.secouer(c.dataset.id, "non");
      };
    });
    if (mon !== jeton) return;
    cible?.n.classList.remove("appel");
    await faire(tapis, pl.apres);
    if (mon !== jeton) return;
    const g2 = guide(rendre(pl.ensuite || ""), navBoutons(), pl.cr);
    brancherNav(g2);
  }

  function quiz(pl) {
    const q = pl.quiz;
    const ordre = q.choix.map((_, i) => i).sort(() => Math.random() - 0.5);
    const g = guide(`<span class="ap-etiq">Question</span> ${rendre(q.q)}`, navBoutons(false), null);
    const c = document.createElement("div"); c.className = "ap-choix ligne";
    c.innerHTML = ordre.map((i) => `<button class="btn" data-i="${i}">${rendre(esc(q.choix[i]))}</button>`).join("");
    g.querySelector(".ap-dit").after(c);
    brancherNav(g);
    c.querySelectorAll("[data-i]").forEach((b) => b.addEventListener("click", () => {
      const bon = +b.dataset.i === q.bonne;
      g.querySelector(".ap-retour")?.remove();
      const r = document.createElement("div"); r.className = "ap-retour " + (bon ? "ok" : "ko");
      if (bon) {
        // Bonne réponse : on explique pourquoi, et on peut continuer.
        c.querySelectorAll("[data-i]").forEach((x) => { x.disabled = true; });
        b.classList.add("bon");
        r.innerHTML = `<b>Exact !</b> ${rendre(esc(q.pourquoi))} ${crs(pl.cr)}`;
        g.querySelector(".ap-actions").innerHTML = navBoutons();
        brancherNav(g);
      } else {
        // Mauvaise réponse : on ne donne pas la solution, l'élève réessaie.
        b.classList.add("faux"); b.disabled = true;
        r.innerHTML = `<b>Pas tout à fait.</b> Réessaie avec une autre réponse.`;
      }
      c.after(r);
    }));
  }

  function fin() {
    const p = lire(); p[lecon.id] = true; ecrire(p);
    const tout = LECONS.flatMap((c) => c.lecons);
    const s = tout[tout.findIndex((l) => l.id === lecon.id) + 1];
    tapis.vider();
    tapis.tapis.insertAdjacentHTML("beforeend", `<div class="ap-fin"><div class="sceau">✦</div><h2>Leçon terminée</h2><p>${esc(lecon.titre)}</p></div>`);
    const g = guide("Bravo ! Tu peux passer à la suite ou revoir la leçon.", `<button class="btn quiet" id="ap-refaire">Refaire</button><button class="btn" id="ap-s2">Sommaire</button>${s ? `<button class="btn primary" id="ap-suite">Leçon suivante : ${esc(s.titre)} ▸</button>` : ""}`);
    g.querySelector("#ap-refaire").onclick = () => ouvrir(lecon.id);
    g.querySelector("#ap-s2").onclick = sommaire;
    if (s) { g.querySelector("#ap-suite").onclick = () => ouvrir(s.id); g.querySelector("#ap-suite").focus(); }
  }

  // ——— Scène sur le vrai moteur ———
  // L'état du moteur est posé sur le même tapis ; chaque carte garde son id
  // d'objet, donc elle vole d'une zone à l'autre quand l'état change. Les
  // images du « film » (pas du moteur après le choix, réponses du bot
  // comprises) sont jouées l'une après l'autre.
  // La carte que le joueur est en train de jouer (montrée pendant qu'il paie).
  let enJeu = null;
  async function scene(pl, mon) {
    const sc = pl.scene;
    tapis.vider(); enJeu = null;
    for (const [z, nom] of [["moi.cimetiere", "Cimetière"], ["moi.pitch", "Pitch"], ["adv.cimetiere", "Cimetière"]]) tapis.slot(z, nom, false);
    let dernier = null;
    const g = guide(`${rendre(pl.dit || "")}<div class="ap-consigne">${rendre(sc.consigne)}</div>`, navBoutons(false), pl.cr);
    brancherNav(g);
    let debut = null;
    const opts = document.createElement("div"); opts.className = "ap-choix ligne"; g.querySelector(".ap-dit").after(opts);
    const afficher = async (e, film) => {
      if (mon !== jeton) return;
      for (const f of (film || [])) { await poserEtat(f.plateau); await attendre(650); if (mon !== jeton) return; }
      await poserEtat(e);
      e.debut = debut; dernier = e;
      const ok = sc.reussi(e);
      const d = e.decision;
      const optDe = {};
      (d?.options || []).forEach((o) => { if (o.carte != null && optDe[o.carte] == null) optDe[o.carte] = o.i; });
      tapis.tapis.querySelectorAll(".ct").forEach((n) => n.classList.toggle("jouable", !ok && optDe[n.dataset.id.slice(1)] != null));
      tapis.tapis.onclick = (ev) => {
        const c = ev.target.closest(".ct.jouable"); if (!c) return;
        choisir(optDe[c.dataset.id.slice(1)]);
      };
      opts.innerHTML = ok ? "" : (d ? `<span class="ap-q">${fine(esc(tutoyer(d.titre)))}</span>` + d.options.filter((o) => o.carte == null || o.passer).map((o) => `<button class="btn small${o.passer ? " quiet" : ""}" data-o="${o.i}">${esc(propre(o.texte))}</button>`).join("") : "");
      opts.querySelectorAll("[data-o]").forEach((b) => b.addEventListener("click", () => choisir(b.dataset.o)));
      g.querySelector(".ap-actions").innerHTML = navBoutons(ok) + (ok ? "" : `<button class="btn quiet" id="ap-recommencer">Recommencer</button>`);
      brancherNav(g);
      g.querySelector("#ap-recommencer")?.addEventListener("click", () => aller(k));
      if (ok) {
        const r = document.createElement("div"); r.className = "ap-retour ok"; r.innerHTML = `<b>Réussi !</b> ${rendre(esc(typeof sc.bravo === "function" ? sc.bravo(e) : sc.bravo))}`;
        opts.replaceWith(r);
      }
    };
    const choisir = async (i) => {
      tapis.tapis.onclick = null;
      const o = dernier?.decision?.options?.find((x) => String(x.i) === String(i));
      if (o?.action === "jouer") enJeu = (dernier.vous.main || []).find((c) => c.id === o.carte) || null;
      try { const e = await api("POST", "/api/apprendre/choix?i=" + i); await afficher(e, e.film); }
      catch (err) { opts.innerHTML = `<p class="erreur">${esc(err.message)}</p>`; }
    };
    try {
      const e = await api("POST", "/api/apprendre/scene?id=" + encodeURIComponent(sc.id));
      debut = e;
      await afficher(e, null);
    } catch (err) { g.querySelector(".ap-dit").insertAdjacentHTML("beforeend", `<p class="erreur">Scène indisponible : ${esc(err.message)}</p>`); }
  }
  // « Jouer Raging Onslaught (rouge) [40] — coût 3… » → « Jouer Raging Onslaught (rouge) ».
  const propre = (t) => tutoyer(String(t).replace(/\s*\[\d+\]/g, "").replace(/\s+—.*$/, "").replace(/\s*\(\+\d+ ressources?\)/, ""));
  // Les questions du moteur sont écrites pour l'application (« vous ») ; le
  // didacticiel tutoie, et n'emploie pas de mot qu'il n'a pas encore expliqué.
  const TUTOIEMENT = [
    [/^Vous avez la priorité\.$/, "À toi de jouer : clique sur une carte qui brille."],
    [/^Passer$/, "Ne rien jouer"],
    [/^Payez le coût : choisissez une carte à pitcher — il faut (\d+), vous avez (\d+)$/, (m, a, b) => `Paie le coût : clique sur une carte de ta main pour la pitcher. Il te faut ${a}, tu en as ${b}.`],
    [/^Payez le coût : choisissez une carte à pitcher/, "Paie le coût : clique sur une carte de ta main pour la pitcher."],
    [/^Déclarez vos cartes qui défendent \(une par une\), puis terminez\.$/, "Choisis tes cartes qui défendent, une par une, puis termine."],
    [/^Déclarez vos cartes qui défendent\. Déjà déclarées : /, "Choisis tes cartes qui défendent. Déjà choisies : "],
    [/^Terminer la déclaration des défenseurs$/, "J'ai fini de défendre"],
    [/^Fin de tour : choisissez/, "Fin de tour : choisis"],
    [/^Choisissez/, "Choisis"],
    [/^Des dégâts vous sont infligés/, "Tu subis des dégâts"],
  ];
  function tutoyer(t) {
    for (const [re, par] of TUTOIEMENT) if (re.test(t)) return t.replace(re, par);
    return t;
  }

  // Pose un état du moteur : héros, mains, pitch, arsenal, cimetière, chaîne.
  async function poserEtat(e) {
    const voulu = new Map();
    const mettre = (c, z) => { if (c && c.id != null) voulu.set("o" + c.id, { c, z }); };
    mettre({ id: "h0", ...e.vous, image: e.vous.heros_image, nom: e.vous.heros }, "moi.heros");
    mettre({ id: "h1", ...e.bot, image: e.bot.heros_image, nom: e.bot.heros }, "adv.heros");
    (e.vous.main || []).forEach((c, i) => mettre(c, "moi.main." + i));
    (e.vous.pitch || []).forEach((c, i) => mettre(c, "moi.pitch." + i));
    (e.bot.pitch || []).forEach((c) => mettre(c, "adv.pitch"));
    if (e.vous.arsenal && !e.vous.arsenal.cache) mettre(e.vous.arsenal, "moi.arsenal");
    (e.vous.cimetiere || []).forEach((c, i) => mettre(c, "moi.cimetiere." + i));
    (e.bot.cimetiere || []).forEach((c, i) => mettre(c, "adv.cimetiere." + i));
    (e.combat?.maillons || []).forEach((m) => { mettre(m.attaque, "chaine"); (m.defenseurs || []).forEach((d, i) => mettre(d, "def." + i)); });
    if (enJeu && (e.pile || []).length && !voulu.has("o" + enJeu.id)) mettre(enJeu, "chaine");
    // Les cartes de l'adversaire en main : des dos.
    for (let i = 0; i < (e.bot.main_nombre || 0); i++) voulu.set("ob" + i, { dos: true, z: "adv.main." + i });
    for (const id of Object.keys(tapis.el)) if (!voulu.has(id)) tapis.retirer(id);
    const anims = [];
    for (const [id, { c, z, dos }] of voulu) {
      if (!tapis.el[id]) {
        const n = document.createElement("div"); n.className = "ct" + (dos ? " dos" : ""); n.dataset.id = id;
        if (!dos) n.innerHTML = carteHtml(c);
        tapis.tapis.appendChild(n); tapis.el[id] = { n };
        const [x, y] = zone(z);
        n.style.transform = `translate(${x}px, ${y + (z.startsWith("adv") ? -40 : 40)}px)`; n.getBoundingClientRect();
      }
      if (tapis.el[id].z !== z) anims.push(tapis.placer(id, z, 600));
    }
    await Promise.all(anims);
    await Promise.all([tapis.vie("oh0", e.vous.vie), tapis.vie("oh1", e.bot.vie)]);
    const j = tapis.tapis.querySelector(".ap-jauge") || Object.assign(document.createElement("div"), { className: "ap-jauge" });
    j.innerHTML = `${icone("r")}<b>${e.vous.ressources}</b><span>ressource${e.vous.ressources > 1 ? "s" : ""}</span>`;
    tapis.tapis.appendChild(j);
  }

  // Bulle de définition au survol d'un mot du jeu.
  sec.addEventListener("mouseover", (ev) => {
    const m = ev.target.closest(".ap-mot"); if (!m || m.querySelector(".ap-def")) return;
    m.insertAdjacentHTML("beforeend", `<span class="ap-def">${esc(m.dataset.def)}</span>`);
  });
  document.addEventListener("keydown", (ev) => {
    if (Arsenal.vue() !== "apprendre" || !lecon) return;
    if (ev.key === "ArrowRight") sec.querySelector('[data-nav="suiv"]')?.click();
    if (ev.key === "ArrowLeft") sec.querySelector('[data-nav="prec"]')?.click();
  });
  document.addEventListener("arsenal:vue", (ev) => { if (ev.detail.vue === "apprendre") sommaire(); });
})();
