// Écran de partie de fab-web (volet « partie », adresse #partie).
// Lot 0 du plan de rapatriement : même code qu'avant, sorti d'index.html.
"use strict";
let etat = null;
let occupe = false;

const $ = (id) => document.getElementById(id);

function el(tag, classe, texte) {
  const e = document.createElement(tag);
  if (classe) e.className = classe;
  if (texte !== undefined && texte !== null) e.textContent = texte;
  return e;
}

async function api(methode, chemin) {
  const r = await fetch(chemin, { method: methode });
  const v = await r.json();
  if (!r.ok || v.erreur) throw new Error(v.erreur || ("erreur " + r.status));
  return v;
}

async function action(methode, chemin) {
  if (occupe) return;
  occupe = true;
  $("erreur").textContent = "";
  const minuterie = setTimeout(() => { $("attente").style.display = "block"; }, 250);
  document.querySelectorAll("button").forEach(b => b.disabled = true);
  let nouveau = null;
  try {
    nouveau = await api(methode, chemin);
  } catch (e) {
    $("erreur").textContent = e.message;
  } finally {
    clearTimeout(minuterie);
    $("attente").style.display = "none";
  }
  // Lot 2 : on rejoue d'abord le « film » des pas du moteur (votre choix,
  // puis les réponses du bot), puis on affiche l'état final.
  if (nouveau) {
    try { await Film.jouer(nouveau); } catch (e) { console.error(e); }
    etat = nouveau;
  }
  occupe = false;
  afficher();
}

const choisir = (i) => action("POST", "/api/choix?i=" + i);
const annuler = () => action("POST", "/api/annuler");

// ---------- cartes ----------

function optionsDeCarte(id) {
  const d = etat && etat.decision;
  if (!d) return [];
  return d.options.filter(o => o.carte === id);
}

function carte(c, petite) {
  if (!c || c.cache) {
    const e = el("div", "carte cachee" + (petite ? " petite" : ""), "?");
    e.title = "Carte cachée";
    return e;
  }
  const e = el("div", "carte " + (c.couleur || "") + (petite ? " petite" : ""));
  if (c.engage) e.classList.add("engage");
  if (c.face_cachee) e.classList.add("face-cachee");
  e.appendChild(el("div", "bande"));
  if (c.cout !== null && c.cout !== undefined) e.appendChild(el("div", "cout", c.cout + (c.x ? "+X" : "")));
  e.appendChild(el("div", "nom", c.nom));
  if (c.genre) e.appendChild(el("div", "genre", c.genre));
  if (c.face_cachee) e.appendChild(el("div", "fc", "face cachée"));
  for (const d of c.details || []) e.appendChild(el("div", "det", d));
  const s = el("div", "stats");
  s.appendChild(el("span", "pui", c.puissance !== null && c.puissance !== undefined ? c.puissance + " ⚔" : ""));
  s.appendChild(el("span", "pitch", c.pitch ? "◆" + c.pitch : ""));
  s.appendChild(el("span", "def", c.defense !== null && c.defense !== undefined ? c.defense + " ⛨" : ""));
  e.appendChild(s);
  imageDeCarte(e, c);
  if (c.carte_id !== undefined) {
    e.addEventListener("mouseenter", () => montrerPool(c.carte_id, c));
    return e;
  }
  e.dataset.k = c.id;
  const opts = optionsDeCarte(c.id);
  if (opts.length) {
    e.classList.add("jouable");
    e.title = opts.map(o => o.texte).join("\n");
  }
  if (sel === c.id) e.classList.add("sel");
  const n = brouillon.indexOf(c.id);
  if (n >= 0) {
    e.classList.add("on");
    if (genreDecision() === "ordre_pitch") e.appendChild(el("span", "rang", String(n + 1)));
  }
  e.addEventListener("mouseenter", () => montrer(c.id, c));
  e.addEventListener("click", (ev) => cliquer(c.id, ev));
  return e;
}

// Image officielle par-dessus le cadre dessiné (qui reste dessous si l'image
// ne vient pas). Les valeurs changées en jeu, les détails (compteurs…) et
// « face cachée » sont repris en pastilles sur l'image.
function imageDeCarte(e, c) {
  if (!c.image) return;
  e.classList.add("img");
  e.title = c.nom_fr ? c.nom_fr + " (" + c.nom + ")" : c.nom;
  e.prepend(Images.img(c.image, "normal", e, "img"));
  const p = [];
  if (c.puissance_imprimee !== undefined && c.puissance !== null && c.puissance !== c.puissance_imprimee) p.push(el("span", "pui", c.puissance + " ⚔"));
  if (c.defense_imprimee !== undefined && c.defense !== null && c.defense !== c.defense_imprimee) p.push(el("span", "def", c.defense + " ⛨"));
  for (const d of c.details || []) p.push(el("span", null, d));
  if (c.face_cachee) p.push(el("span", null, "face cachée"));
  if (p.length) { const b = el("div", "pastilles"); b.append(...p); e.appendChild(b); e.title += "\n" + p.map(x => x.textContent).join(" · "); }
}

// ---------- choix en cours dans le ruban (Arsenal JS : barHTML) ----------
// sel : carte de la main (ou de l'arsenal) sélectionnée pendant la priorité ;
// ses façons de la jouer s'affichent en boutons dans le ruban.
// brouillon : cartes touchées pour une défense ou un ordre, pas encore
// envoyées au moteur (un nouveau clic les reprend ; « Valider » les envoie
// d'un coup, annulées ensemble).
let sel = null;
let brouillon = [];
let cleDecision = null;
let passerTour = null; // « Passer jusqu'à la fin du tour » : numéro du tour

const genreDecision = () => (etat && etat.decision ? etat.decision.genre : null);
const dansMain = (id) => !!(etat && etat.vous && (etat.vous.main || []).some(c => c.id === id));
const dansArsenal = (id) => !!(etat && etat.vous && etat.vous.arsenal && etat.vous.arsenal.id === id);
function carteParId(id) {
  if (!etat) return null;
  for (const j of [etat.vous, etat.bot]) {
    if (!j) continue;
    for (const l of [j.main || [], j.pitch, j.equipement, j.permanents, j.cimetiere, j.bannies, j.arsenal ? [j.arsenal] : []]) {
      const c = (l || []).find(x => x && x.id === id);
      if (c) return c;
    }
  }
  for (const l of (etat.combat ? etat.combat.maillons : [])) {
    if (l.attaque && l.attaque.id === id) return l.attaque;
    const d = l.defenseurs.find(x => x.id === id); if (d) return d;
  }
  return null;
}
function basculer(id) {
  const k = brouillon.indexOf(id);
  if (k >= 0) brouillon.splice(k, 1);
  else {
    // Dominate : une seule carte de la main ; la nouvelle remplace l'autre.
    const l = etat.combat && etat.combat.maillons[etat.combat.maillons.length - 1];
    if (genreDecision() === "defense" && l && l.dominate && dansMain(id)) brouillon = brouillon.filter(x => !dansMain(x));
    brouillon.push(id);
  }
  afficher();
}
const envoyerBrouillon = (passer) => action("POST", `/api/choix-cartes?cartes=${brouillon.join(",")}&passer=${passer ? 1 : 0}`);
// Carte lâchée sur une zone (tirer une carte, finition.js).
function deposerOption(o) {
  if (o.action === "defendre" && genreDecision() === "defense") { if (!brouillon.includes(o.carte)) basculer(o.carte); return; }
  choisir(o.i);
}

function cliquer(id, ev) {
  montrer(id);
  const g = genreDecision();
  const opts = optionsDeCarte(id);
  if ((g === "defense" || g === "ordre_pitch") && (opts.length || brouillon.includes(id))) { basculer(id); return; }
  if (g === "priorite" && (dansMain(id) || dansArsenal(id))) {
    sel = sel === id ? null : id;
    afficher();
    return;
  }
  if (opts.length === 1) { choisir(opts[0].i); return; }
  if (opts.length > 1) ouvrirMenu(opts, ev);
}

function ouvrirMenu(opts, ev) {
  const m = $("menu");
  m.replaceChildren();
  for (const o of opts) {
    const b = el("button", null, o.texte);
    b.addEventListener("click", () => { fermerMenu(); choisir(o.i); });
    m.appendChild(b);
  }
  const f = el("button", null, "Fermer");
  f.addEventListener("click", fermerMenu);
  m.appendChild(f);
  m.style.display = "flex";
  m.style.left = Math.min(ev.clientX, window.innerWidth - 370) + "px";
  m.style.top = Math.min(ev.clientY, window.innerHeight - 40 * (opts.length + 1)) + "px";
  ev.stopPropagation();
}
function fermerMenu() { $("menu").style.display = "none"; }
document.addEventListener("click", (e) => { if (!$("menu").contains(e.target)) fermerMenu(); });

let detailDemande = null;
// Fiche de la carte survolée : grande image (si elle existe), nom français,
// puis la fiche et le texte officiel donnés par le serveur.
function remplirDetail(v, c) {
  const d = $("detail");
  d.replaceChildren();
  if (c && c.image) {
    const g = el("div", "grande");
    g.appendChild(Images.img(c.image, "normal", g, "grande"));
    d.appendChild(g);
  }
  if (c && c.nom_fr) d.appendChild(el("div", "nom-fr", c.nom_fr));
  d.appendChild(el("div", "fiche", v.fiche));
  if (v.type) d.appendChild(el("div", "type", v.type));
  const t = el("div", "texte");
  t.innerHTML = Symboles.html(v.texte);
  d.appendChild(t);
}

async function montrer(id, c) {
  if (id === detailDemande) return;
  detailDemande = id;
  try {
    const v = await api("GET", "/api/carte?id=" + id);
    if (detailDemande !== id) return;
    remplirDetail(v, c);
  } catch (e) { /* carte cachée : rien à montrer */ }
}

async function montrerPool(id, c) {
  const cle = "pool-" + id;
  if (cle === detailDemande) return;
  detailDemande = cle;
  try {
    const v = await api("GET", "/api/carte-pool?id=" + id);
    if (detailDemande !== cle) return;
    remplirDetail(v, c);
  } catch (e) { /* rien à montrer */ }
}

// ---------- préparation d'une manche : fenêtre d'ajustement ----------
// D'après openAdjust / renderAdjust de l'Arsenal JS : en-tête (« Hala
// contre Ira », tiré au sort, Retirer au sort, Ce que joue le bot, départ),
// le deck en piles (clic : un exemplaire de plus, clic droit : un de moins),
// l'équipement une rangée par zone, et au pied le compteur, le rythme, le
// niveau du bot et « Lancer la partie ». Les étapes restent celles du
// moteur (CR 4.1.3 à 4.1.6) : qui commence, puis l'équipement (présenté),
// puis le deck ; le deck se règle dès l'ouverture et reste réglé d'une
// étape à l'autre. Les cartes s'ajustent pour que tout tienne sans défiler.

// Choix en cours de l'humain.
let choixArene = null;   // ensemble des positions choisies
let choixDeck = null;    // quantité par carte distincte
let etapeVue = null, mancheVue = null;
let sourceReglage = "proposition";
let infoBot = null;      // { nom, html } : « Ce que joue le bot »

// Dernier réglage de ce matchup (vos decks contre ce deck du bot).
const cleReglage = () => `arsenal-reglage:${etat.reglages.deck}>${etat.reglages.deck_bot}`;
const signature = (p) => p.arene.map(c => c.nom).join("|") + "#" + p.deck.map(c => c.nom + c.couleur).join("|");
function lireReglage(p) {
  try {
    const r = JSON.parse(localStorage.getItem(cleReglage()) || "null");
    return r && r.sig === signature(p) ? r : null;
  } catch (e) { return null; }
}
function garderReglage(p) {
  try { localStorage.setItem(cleReglage(), JSON.stringify({ sig: signature(p), arene: [...choixArene], deck: choixDeck, quand: Date.now() })); } catch (e) {}
}
function sources(p) {
  const d = lireReglage(p);
  return [
    ["dernier", "Dernier réglage", d ? { arene: d.arene, deck: d.deck } : null, d ? "Votre dernier réglage contre ce deck (" + new Date(d.quand).toLocaleDateString("fr-FR", { day: "numeric", month: "short" }) + ")" : "Pas encore de réglage gardé contre ce deck"],
    ["conseil", "Plan conseillé", p.conseil, "Choix de la table apprise des bots pour votre card-pool contre ce héros"],
    ["proposition", "Proposition", { arene: p.arene.filter(c => c.propose).map(c => c.i), deck: p.deck.map(c => c.propose) }, "Ordre de la liste, ou votre choix de la manche précédente"],
  ];
}
// L'équipement ne change plus une fois validé (étape « deck »).
function appliquer(p, nom) {
  const src = sources(p).find(x => x[0] === nom && x[2]);
  if (!src) return false;
  sourceReglage = nom;
  if (p.etape !== "deck") choixArene = new Set(src[2].arene);
  choixDeck = src[2].deck.slice();
  return true;
}
const libelleSource = (p) => {
  if (sourceReglage === "modifie") return "Réglage modifié";
  const s = sources(p).find(x => x[0] === sourceReglage);
  return s ? s[3] : "";
};
function barreSources(p) {
  const g = el("div", "seg seg-sources");
  g.setAttribute("role", "group");
  g.setAttribute("aria-label", "Point de départ");
  for (const [nom, lib, val, aide] of sources(p)) {
    const b = el("button", null, lib);
    b.type = "button";
    b.title = aide;
    b.disabled = !val;
    b.setAttribute("aria-pressed", String(sourceReglage === nom));
    b.addEventListener("click", () => { appliquer(p, nom); preparation(); });
    g.appendChild(b);
  }
  return g;
}

// Petit message dans la fenêtre (« Aucun exemplaire au deck. »…)
let minPrepToast = null;
function prepToast(m) {
  const s = $("preparation");
  for (const t of s.querySelectorAll(".toast")) t.remove();
  const t = el("div", "toast", m);
  t.setAttribute("role", "status");
  s.appendChild(t);
  clearTimeout(minPrepToast); minPrepToast = setTimeout(() => t.remove(), 3000);
}

// Une pile : les exemplaires d'une carte, ceux hors du deck grisés.
function pile(c, k, arrive) {
  const t = c.max, n = choixDeck[k];
  const e = el("div", "pile-cartes" + (n ? "" : " eteinte") + (arrive ? " arrive" : ""));
  e.tabIndex = 0;
  e.setAttribute("role", "button");
  e.style.setProperty("--t", t);
  const nom = `${c.nom_fr || c.nom}${c.couleur ? " (" + c.couleur + ")" : ""}`;
  e.setAttribute("aria-label", `${nom} : ${n} au deck sur ${t}`);
  e.title = `${nom} : ${n} au deck${t - n ? `, ${t - n} désactivé${t - n > 1 ? "s" : ""}` : ""}\nClic : un de plus · clic droit : un de moins`;
  for (let i = 0; i < t; i++) {
    const x = el("div", "ex" + (i < t - n ? " off" : ""));
    x.style.setProperty("--i", i);
    if (c.image) { x.classList.add("a-img"); x.appendChild(Images.img(c.image, "normal", x, "a-img")); }
    x.appendChild(el("span", "nm", c.nom_fr || c.nom));
    x.classList.add("c-" + (c.couleur || "n"));
    e.appendChild(x);
  }
  if (t > 1 || !n) e.appendChild(el("span", "q", `${n}/${t}`));
  const changer = (d) => {
    const v = choixDeck[k] + d;
    if (v < 0 || v > t) {
      e.classList.remove("secoue"); void e.offsetWidth; e.classList.add("secoue");
      prepToast(d > 0 ? "Tous les exemplaires sont déjà au deck. Clic droit pour en désactiver un." : "Aucun exemplaire au deck.");
      return;
    }
    choixDeck[k] = v; sourceReglage = "modifie"; preparation(k);
    const f = document.querySelector(`#preparation .pile-cartes[data-k="${k}"]`); if (f) f.focus({ preventScroll: true });
  };
  e.dataset.k = k;
  e.addEventListener("click", () => changer(+1));
  e.addEventListener("contextmenu", (ev) => { ev.preventDefault(); changer(-1); });
  e.addEventListener("keydown", (ev) => {
    if (ev.key === "Enter" || ev.key === " " || ev.key === "+") { ev.preventDefault(); changer(+1); }
    else if (ev.key === "Delete" || ev.key === "Backspace" || ev.key === "-") { ev.preventDefault(); changer(-1); }
  });
  e.addEventListener("mouseenter", () => montrerPool(c.carte_id, c));
  return e;
}

// Une carte d'équipement (choisie : cadre doré).
function carteEquipement(c, choisie, cliquable, quand) {
  const b = el("button", "eqc" + (c.image ? " a-img" : ""));
  b.type = "button";
  b.setAttribute("aria-pressed", String(choisie));
  b.title = c.nom_fr || c.nom;
  b.disabled = !cliquable;
  if (c.image) b.appendChild(Images.img(c.image, "normal", b, "a-img"));
  b.appendChild(el("span", "nmf", c.nom_fr || c.nom));
  if (cliquable) b.addEventListener("click", quand);
  if (c.carte_id !== undefined) b.addEventListener("mouseenter", () => montrerPool(c.carte_id, c));
  return b;
}

// Le deck du bot : héros, armes et équipements de son card-pool (pas son side).
async function chargerInfoBot() {
  const nom = etat.reglages.deck_bot;
  if (infoBot && infoBot.nom === nom) return;
  infoBot = { nom, html: "…" };
  try {
    const v = await api("GET", "/api/deck?nom=" + encodeURIComponent(nom));
    const par = {};
    for (const a of v.liste.arene) (par[a.emplacement || "?"] = par[a.emplacement || "?"] || []).push(a.nom);
    const NOMS = { Weapon: "Arme", Head: "Tête", Chest: "Torse", Arms: "Bras", Legs: "Jambes", "Off-Hand": "Main gauche" };
    const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
    infoBot.html = `<div><b>${esc(v.heros)}</b>${v.classe ? " · " + esc(v.classe) : ""}${v.source ? `<div class="muted">${esc(v.source)}</div>` : ""}</div>`
      + Object.entries(par).map(([z, l]) => `<div><span class="muted">${esc(NOMS[z] || z)} :</span> ${l.map(esc).join(", ")}</div>`).join("")
      + `<div class="muted">${v.arene} cartes d'arène, ${v.deck} cartes de deck. Le bot choisit son équipement et ses 40 cartes sans voir les vôtres ; son choix n'est pas montré avant la présentation de l'équipement.</div>`;
  } catch (e) { infoBot.html = "Deck du bot illisible."; }
  const p = document.querySelector("#preparation .abp");
  if (p) p.innerHTML = infoBot.html;
}

// Refaire le match (manche 1, rien de joué) avec d'autres réglages.
async function refaire(change) {
  const r = etat.reglages;
  const q = Object.assign({ deck: r.deck, bot: r.deck_bot, niveau: r.niveau, manches: String(etat.match.manches), graine: r.graine }, change);
  await action("POST", "/api/nouvelle?" + new URLSearchParams(q).toString());
}
async function retirerAuSort() {
  try {
    const p = await api("GET", "/api/decks-projet");
    let arch = [];
    try { arch = JSON.parse(localStorage.getItem("arsenal-bot-archives") || "[]"); } catch (e) {}
    const L = p.decks.map(d => d.nom).filter(n => n !== etat.reglages.deck_bot && !arch.includes(n));
    if (!L.length) return;
    await refaire({ bot: L[Math.floor(Math.random() * L.length)] });
    botAuHasard.add(etat.reglages.graine + ">" + etat.reglages.deck_bot);
    preparation();
  } catch (e) { $("erreur").textContent = e.message; }
}
const botAuHasard = new Set(); // matchs lancés avec « Deck du bot : Au hasard »

const NIVEAUX_BOT = [
  ["b0", "Niveau 0 (hasard)"], ["b1", "Niveau 1 (règles fixes)"], ["b2", "Niveau 2 (réfléchit)"],
  ["b3", "Niveau 3 (évaluation apprise)"], ["b2-500ms", "Niveau 2, ½ s par décision"], ["b2-1000", "Niveau 2, 1 000 essais"],
];

function preparation(bouge) {
  const p = etat.preparation;
  const m = etat.match;
  if (mancheVue !== m.jouees + "|" + etat.reglages.graine + "|" + etat.reglages.deck_bot) {
    mancheVue = m.jouees + "|" + etat.reglages.graine + "|" + etat.reglages.deck_bot;
    etapeVue = null;
    choixArene = new Set(p.arene.filter(c => c.propose).map(c => c.i));
    choixDeck = p.deck.map(c => c.propose);
    sourceReglage = "proposition";
    appliquer(p, "dernier");
  }
  etapeVue = p.etape;
  chargerInfoBot();
  const s = $("preparation");
  s.replaceChildren();
  const premiere = m.jouees === 0;
  const hasard = botAuHasard.has(etat.reglages.graine + ">" + etat.reglages.deck_bot);

  // En-tête
  const ah = el("header", "ah");
  const att = el("div", "att");
  const h = el("h2");
  h.append(document.createTextNode(p.heros + " "), el("span", "vs muted", "contre"), document.createTextNode(" " + p.heros_bot));
  if (hasard) h.appendChild(el("span", "pill", "tiré au sort"));
  if (hasard && premiere) {
    const b = el("button", "btn small quiet", "Retirer au sort");
    b.type = "button";
    b.title = "Tirer un autre deck du bot";
    b.addEventListener("click", retirerAuSort);
    h.appendChild(b);
  }
  att.appendChild(h);
  const src = el("div", "asrc");
  src.append(document.createTextNode(`Manche ${m.jouees + 1} · Départ : `), el("b", null, libelleSource(p)));
  const prem = p.premier ? (p.premier === "vous" ? " · Vous commencez." : " · Le bot commence.") : (p.choisit_premier === "vous" ? "" : " · Le bot choisit qui commence.");
  src.appendChild(document.createTextNode(prem));
  att.appendChild(src);
  ah.append(att, el("span", "sp"));
  const det = el("details", "abot");
  det.appendChild(el("summary", null, "Ce que joue le bot"));
  const abp = el("div", "abp");
  abp.innerHTML = infoBot ? infoBot.html : "…";
  det.appendChild(abp);
  ah.append(det, barreSources(p));
  s.appendChild(ah);

  // Corps
  const body = el("div", "abody");
  if (p.etape === "premier") {
    const q = el("div", "aquestion");
    q.appendChild(el("span", null, "Qui commence la manche ? (CR 4.1.3 : choisi avant l'équipement et le deck)"));
    const b1 = el("button", "btn primary", "Je commence");
    b1.addEventListener("click", () => action("POST", "/api/premier?qui=vous"));
    const b2 = el("button", "btn", "Le bot commence");
    b2.addEventListener("click", () => action("POST", "/api/premier?qui=bot"));
    q.append(b1, b2);
    body.appendChild(q);
  }
  const total = choixDeck.reduce((a, n) => a + n, 0);
  const hors = p.deck.reduce((a, c, k) => a + c.max - choixDeck[k], 0);
  const T = p.taille_deck;
  const zd = el("section", "zone deck");
  const h3 = el("h3");
  h3.append(document.createTextNode("Deck "), el("b", null, `${total} / ${T}`));
  h3.appendChild(el("span", "m " + (total === T ? "ok" : total < T ? "manque" : "trop"), total === T ? "prêt" : total < T ? `manque ${T - total}` : `${total - T} de trop`));
  if (hors) h3.appendChild(el("span", "m muted", `${hors} exemplaire${hors > 1 ? "s" : ""} désactivé${hors > 1 ? "s" : ""} (side)`));
  zd.appendChild(h3);
  const r = el("div", "stks");
  const ordre = p.deck.map((c, k) => k).sort((x, y) => {
    const a = p.deck[x], c = p.deck[y];
    return ((a.cout ?? 99) - (c.cout ?? 99)) || ((a.pitch ?? 9) - (c.pitch ?? 9)) || a.nom.localeCompare(c.nom);
  });
  for (const k of ordre) r.appendChild(pile(p.deck[k], k, bouge === k));
  zd.appendChild(r);
  body.appendChild(zd);

  const ze = el("section", "zone eqz");
  ze.appendChild(el("h3", null, p.etape === "deck" ? "Votre équipement (validé)" : "Équipement et armes"));
  const rang = el("div", "eqrow");
  const zones = [];
  for (const c of p.arene) if (!zones.includes(c.zone)) zones.push(c.zone);
  for (const z of zones) {
    const cartes = p.arene.filter(c => c.zone === z);
    const slot = el("div", "eqslot");
    slot.appendChild(el("span", null, z));
    const o = el("div", "opts");
    for (const c of cartes) {
      o.appendChild(carteEquipement(c, choixArene.has(c.i), p.etape !== "deck", () => {
        if (choixArene.has(c.i)) choixArene.delete(c.i);
        else {
          if (z !== "Armes") for (const x of cartes) choixArene.delete(x.i); // une carte par zone, deux mains pour les armes
          choixArene.add(c.i);
        }
        sourceReglage = "modifie";
        preparation();
      }));
    }
    slot.appendChild(o);
    rang.appendChild(slot);
  }
  ze.appendChild(rang);
  if (p.etape === "deck" && p.arene_bot) {
    const sb = el("div", "eqslot bot");
    sb.appendChild(el("span", null, "Équipement du bot (présenté, CR 4.1.5)"));
    const o = el("div", "opts");
    for (const c of p.arene_bot) o.appendChild(carteEquipement(c, true, false));
    sb.appendChild(o);
    rang.appendChild(sb);
  } else if (p.etape !== "deck") ze.appendChild(el("p", "aide", "Une carte par zone (deux mains pour les armes). Le bot choisit le sien sans voir le vôtre ; vous le verrez avant de valider le deck."));
  body.appendChild(ze);
  s.appendChild(body);

  // Pied
  const af = el("footer", "af");
  const cpt = el("div", "acount");
  cpt.append(document.createTextNode(`${total} / ${T}`), el("small", total === T ? "ok" : "muted", total === T ? "prêt à jouer" : total < T ? `active encore ${T - total} exemplaire${T - total > 1 ? "s" : ""}` : `désactive ${total - T} exemplaire${total - T > 1 ? "s" : ""}`));
  const aide = el("span", "muted ahelp");
  aide.innerHTML = "<kbd>clic gauche</kbd> active un exemplaire · <kbd>clic droit</kbd> le désactive";
  af.append(cpt, aide, el("span", "sp"));
  const rythme = el("div", "aopt");
  rythme.appendChild(el("span", null, "Rythme"));
  const seg = el("div", "seg");
  seg.setAttribute("role", "group"); seg.setAttribute("aria-label", "Rythme des animations");
  for (const [v, lib, t] of [[false, "Normal", "animations au rythme normal"], [true, "Rapide", "animations accélérées"]]) {
    const b = el("button", null, lib);
    b.type = "button"; b.title = t;
    b.setAttribute("aria-pressed", String(ReglagesFx.rapide === v));
    b.addEventListener("click", () => { ReglagesFx.regler("rapide", v); preparation(); });
    seg.appendChild(b);
  }
  rythme.appendChild(seg);
  const niv = el("label", "aopt");
  niv.appendChild(el("span", null, "Bot"));
  const sn = el("select");
  for (const [v, l] of NIVEAUX_BOT) { const o = el("option", null, l); o.value = v; sn.appendChild(o); }
  if (!NIVEAUX_BOT.some(x => x[0] === etat.reglages.niveau)) { const o = el("option", null, etat.reglages.niveau); o.value = etat.reglages.niveau; sn.appendChild(o); }
  sn.value = etat.reglages.niveau;
  sn.disabled = !premiere;
  sn.title = premiere ? "Niveau du bot pour tout le match" : "Le niveau se choisit avant la première manche";
  sn.addEventListener("change", () => refaire({ niveau: sn.value }));
  niv.appendChild(sn);
  const retour = el("button", "btn", "Retour");
  retour.type = "button";
  retour.title = "Revenir à l'écran Jouer (le match est gardé : Reprendre depuis l'accueil)";
  retour.addEventListener("click", () => Arsenal.aller("decks"));
  let go;
  if (p.etape === "deck") {
    go = el("button", "btn primary big", "Lancer la partie");
    go.disabled = total !== T;
    go.title = total === T ? "" : `Le deck doit compter ${T} cartes`;
    go.addEventListener("click", () => { garderReglage(p); action("POST", "/api/deck?n=" + choixDeck.join(",")); });
  } else {
    go = el("button", "btn primary big", "Valider l'équipement");
    go.disabled = p.etape !== "arene";
    go.title = p.etape === "arene" ? "L'équipement est présenté au bot ; le deck reste réglable ensuite" : "Choisissez d'abord qui commence";
    go.addEventListener("click", () => { garderReglage(p); action("POST", "/api/arene?cartes=" + [...choixArene].join(",")); });
  }
  go.id = "prep-go";
  af.append(rythme, niv, retour, go);
  s.appendChild(af);
  if (bouge === undefined) requestAnimationFrame(ajusterTaille);
}

// Pas de défilement et pas de place perdue : la plus grande largeur de carte
// (46 à 150 px) qui laisse tout tenir dans la fenêtre (dichotomie).
let essaisTaille = 0;
function ajusterTaille() {
  const b = document.querySelector("#preparation .abody");
  if (!b) return;
  if (!b.clientHeight) { if (++essaisTaille < 40) setTimeout(ajusterTaille, 50); return; } // volet pas encore affiché
  essaisTaille = 0;
  const essai = (cw) => { b.style.setProperty("--cwf", cw + "px"); b.style.setProperty("--ewf", Math.round(cw * 0.8) + "px"); return b.scrollHeight <= b.clientHeight + 1; };
  let lo = 46, hi = 150;
  if (!essai(lo)) return;
  while (hi - lo > 1) { const mid = (lo + hi) >> 1; if (essai(mid)) lo = mid; else hi = mid; }
  essai(lo);
}
window.addEventListener("resize", () => { if (etat && etat.phase === "preparation") ajusterTaille(); });

// ---------- affichage ----------

function zone(titre, liste, petite, ancre, depot) {
  const z = el("div", "zone");
  if (ancre) z.dataset.a = ancre;
  if (depot) z.dataset.depot = depot;
  z.appendChild(el("div", "titre", titre));
  const c = el("div", "cartes");
  if (!liste || liste.length === 0) c.appendChild(el("span", "vide", "—"));
  else for (const x of liste) c.appendChild(carte(x, petite));
  z.appendChild(c);
  return z;
}

// ---------- tapis (disposition de l'Arsenal JS, maquette-tapis-zones) ----------
// Zones placées comme sur le tapis officiel. Vous, de gauche à droite :
// équipement (tête, torse, jambes ; bras collé au torse), arme, héros
// (arsenal dessous), seconde arme ou off-hand (permanents dessous), pitch
// collé au deck, puis cimetière / deck / banni. Le bot : rotation à 180°
// (même tapis, vu d'en face). Affichage seulement : aucune règle ici.

// Une zone imprimée sur le tapis : contour fin, nom gravé sur le bord
// (dans la zone quand elle est vide).
function zoneTapis(lib, taille, contenu, o = {}) {
  const z = el(o.bouton ? "button" : "div", "z " + taille);
  if (o.bouton) z.type = "button";
  z.dataset.l = lib;
  if (o.ancre) z.dataset.a = o.ancre;
  if (o.depot) z.dataset.depot = o.depot;
  if (o.classe) z.classList.add(...o.classe.split(" "));
  const vide = !contenu || (Array.isArray(contenu) && !contenu.length);
  if (vide) z.classList.add("vide");
  else for (const c of [].concat(contenu)) z.appendChild(c);
  if (o.n !== undefined) z.appendChild(el("span", "n", String(o.n)));
  if (o.titre) z.title = o.titre;
  if (o.clic) z.addEventListener("click", o.clic);
  return z;
}

function colonne(classe, ...zones) {
  const c = el("div", "col " + classe);
  c.append(...zones);
  return c;
}

// Liste complète d'une zone (cimetière, bannies) dans une fenêtre.
function voirZone(titre, liste) {
  const d = $("d-zone");
  $("d-zone-titre").textContent = titre + " (" + liste.length + ")";
  const c = $("d-zone-cartes");
  c.replaceChildren();
  if (!liste.length) c.appendChild(el("span", "vide", "Aucune carte."));
  for (const x of liste) c.appendChild(carte(x, false));
  if (!d.open) d.showModal();
}

// Ressources : symboles de pitch (trois emplacements dans un cercle), un
// symbole par tranche de 3 (Arsenal JS, resHTML). Pendant un paiement :
// contour doré = ce qu'il reste à trouver.
const RDOTS = [[20, 12.6], [13.4, 24.2], [26.6, 24.2]];
function ressources(j, estVous) {
  const d = etat.decision;
  const besoin = estVous && d && d.genre === "pitch" && d.a_payer != null ? d.a_payer : 0;
  const tot = Math.max(j.ressources, besoin), nsym = Math.max(1, Math.ceil(tot / 3));
  const r = el("div", "res" + (nsym > 2 ? " many" : "") + (besoin ? " paie" : ""));
  r.dataset.a = "res-" + (estVous ? "vous" : "bot");
  r.title = "Ressources : " + j.ressources;
  r.setAttribute("aria-label", r.title);
  if (besoin) r.appendChild(el("small", null, `Paie ${besoin} · reste ${Math.max(0, besoin - j.ressources)}`));
  const g = el("div", "gems");
  let h = "";
  for (let k = 0; k < nsym; k++) {
    h += '<svg class="psym" viewBox="0 0 40 40" aria-hidden="true"><circle class="rim" cx="20" cy="20" r="18.4"/><circle class="rim2" cx="20" cy="20" r="15.6"/>';
    RDOTS.forEach(([x, y], n) => { const i = k * 3 + n; h += `<circle class="dot${i < j.ressources ? " on" : i < besoin ? " need" : ""}" cx="${x}" cy="${y}" r="4.6"/>`; });
    h += "</svg>";
  }
  g.innerHTML = h;
  r.appendChild(g);
  return r;
}
// Points d'action : losanges dorés.
function pointsAction(n) {
  const s = el("span", "aps");
  s.title = "Points d'action : " + n;
  s.setAttribute("aria-label", s.title);
  let h = "";
  for (let i = 0; i < Math.max(1, n); i++) h += `<svg class="apsym${i < n ? " on" : ""}" viewBox="0 0 40 40" aria-hidden="true"><path class="g" d="M20 2 L37 20 L20 38 L3 20 Z"/><path class="f" d="M20 2 L28 20 L20 38 L12 20 Z M3 20 H37"/></svg>`;
  s.innerHTML = h;
  return s;
}
// Bloc « état » : ressources et points d'action, effets en cours dessous.
function blocEtat(j, estVous) {
  const b = el("div", "etat " + (estVous ? "m" : "o"));
  const g = el("div", "gauges");
  g.append(ressources(j, estVous), el("span", "sep"), pointsAction(j.points_action));
  b.appendChild(g);
  const fx = el("div", "effects");
  fx.appendChild(el("span", "lab", "Effets en cours"));
  const eff = j.effets || [];
  if (!eff.length) fx.appendChild(el("span", "none", "aucun"));
  else {
    const p = el("div", "pills");
    for (const t of eff) p.appendChild(el("span", "pill " + (/perdu|Marqué|pas de gain/.test(t) ? "warn" : "buff"), t.replace(/\{p\}/g, "⚔").replace(/\{d\}/g, "⛨")));
    fx.appendChild(p);
  }
  b.appendChild(fx);
  return b;
}

function joueur(j, estVous, actif) {
  const qui = estVous ? "vous" : "bot";
  const z = zoneTapis;

  // Armes et équipement à leur emplacement (seconde arme ou off-hand à droite du héros).
  const eq = { tete: null, torse: null, bras: null, jambes: null };
  const armes = [];
  for (const c of j.equipement) {
    if (c.emplacement && c.emplacement !== "arme" && !eq[c.emplacement]) eq[c.emplacement] = c;
    else armes.push(c);
  }
  const k = (c) => (c ? carte(c, true) : null);
  const seconde = armes[1];
  const libSeconde = seconde && seconde.genre !== "arme" ? "Off-hand" : "Arme";

  // Héros : la carte du héros, avec les PV gravés dessus.
  const hc = carte({ id: j.heros_id, nom: j.heros, image: j.heros_image, couleur: "", details: [] }, false);
  const vie = el("span", "pv");
  const nv = el("b", null, String(FX.vieAffichee(qui, j.vie)));
  nv.dataset.vie = qui;
  vie.appendChild(nv);
  vie.title = "Points de vie";
  vie.dataset.a = "vie-" + qui;
  const heros = [hc, vie];
  const prio = etat.priorite === qui && !etat.fini;

  const ars = j.arsenal ? [carte(j.arsenal, true)] : null;
  const pitch = j.pitch.length ? (() => {
    const f = el("div", "eventail");
    f.style.setProperty("--k", Math.max(1, j.pitch.length - 1));
    for (const c of j.pitch) f.appendChild(carte(c, true));
    return [f];
  })() : null;
  const perms = j.permanents.length ? (() => {
    const f = el("div", "perms");
    for (const c of j.permanents) f.appendChild(carte(c, true));
    return [f];
  })() : null;
  const dos = () => { const d = el("div", "dos"); d.setAttribute("aria-hidden", "true"); return d; };
  const haut = (liste) => (liste.length ? [carte(liste[liste.length - 1], true)] : null);

  const tapis = el("section", "arena " + (estVous ? "m" : "o") + (actif ? " actif" : ""));
  tapis.setAttribute("aria-label", estVous ? "Votre tapis" : "Tapis du bot");
  tapis.append(
    colonne("etroite", z("Tête", "sm", k(eq.tete)), z("Torse", "sm", k(eq.torse)), z("Jambes", "sm", k(eq.jambes))),
    colonne("bras", z("Bras", "sm", k(eq.bras))),
    colonne("large", z("Arme", "big", k(armes[0]))),
    colonne("large",
      z("Héros", "big", heros, { ancre: "heros-" + qui, classe: "heros" + (prio ? " prio" : "") }),
      z("Arsenal", "big", ars, { ancre: "arsenal-" + qui, depot: estVous ? "arsenal" : null })),
    colonne("large",
      z(libSeconde, "big", seconde ? [carte(seconde, true)] : null),
      z(j.permanents.length ? `Permanents (${j.permanents.length})` : "Permanents", "big", perms)),
    colonne("pitchcol", el("span", "espace"),
      z("Pitch", "sm", pitch, { ancre: "pitch-" + qui, depot: estVous ? "pitch" : null, classe: "pitch" }),
      el("span", "espace")),
    colonne("etroite",
      z("Cimetière", "sm", haut(j.cimetiere), { ancre: "cimetiere-" + qui, bouton: true, n: j.cimetiere.length, titre: "Voir le cimetière", clic: () => voirZone((estVous ? "Votre cimetière" : "Cimetière du bot"), j.cimetiere) }),
      z("Deck", "sm", j.deck ? [dos()] : null, { ancre: "deck-" + qui, n: j.deck }),
      z("Banni", "sm", haut(j.bannies), { bouton: true, n: j.bannies.length, titre: "Voir les cartes bannies", clic: () => voirZone((estVous ? "Vos cartes bannies" : "Cartes bannies du bot"), j.bannies) })),
  );
  const morceaux = [tapis, blocEtat(j, estVous)];

  if (!estVous) {
    // Main du bot : dos en éventail.
    const m = el("div", "ohand");
    m.dataset.a = "main-bot";
    m.appendChild(el("span", "handlab", `Main du bot (${j.main_nombre})`));
    const f = el("div", "eventail dos-main");
    for (let i = 0; i < j.main_nombre; i++) f.appendChild(dos());
    m.appendChild(f);
    morceaux.push(m);
  } else {
    // Votre main en carré sous la pile : 2 colonnes jusqu'à 4 cartes, 3 au-delà ;
    // les cartes rapetissent s'il faut plus de 2 rangées (Arsenal JS).
    // Les bloqueurs posés sur la chaîne quittent la main tant qu'on ne les reprend pas.
    const main = genreDecision() === "defense" ? j.main.filter(c => !brouillon.includes(c.id)) : j.main;
    const n = main.length, cols = n > 4 ? 3 : 2, rangs = Math.max(2, Math.ceil(n / cols));
    const m = el("div", "hand");
    m.dataset.a = "main-vous";
    m.style.setProperty("--cols", cols);
    m.style.setProperty("--rows", rangs);
    m.setAttribute("aria-label", "Votre main");
    m.appendChild(el("span", "handlab", `Votre main (${n})`));
    if (!n) m.appendChild(el("div", "vide", "Main vide"));
    for (const x of main) { const h = el("div", "hc"); h.appendChild(carte(x, false)); m.appendChild(h); }
    morceaux.push(m);
  }

  const conteneur = $(estVous ? "j-vous" : "j-bot");
  conteneur.classList.toggle("actif", actif);
  conteneur.replaceChildren(...morceaux);
  // Illustration du héros en fond de sa moitié du tapis.
  const art = $(estVous ? "art-vous" : "art-bot");
  if (art) art.style.backgroundImage = Images.fond(j.heros_image);
}

const ETAPES = ["Layer", "Attaque", "Défense", "Réactions", "Dégâts", "Résolution"];

// La chaîne de combat (Arsenal JS, midHTML) : un maillon par attaque, le
// maillon en cours avec « Attaque / Défense » en gros et les bloqueurs à côté.
function milieu() {
  const m = $("milieu");
  m.replaceChildren();
  m.dataset.depot = "chaine";
  const cb = etat.combat;
  const tete = el("div", "mhead");
  const tm = el("span", "turnmark", "Tour"); tm.appendChild(el("b", null, String(etat.tour))); tete.appendChild(tm);
  const attaqueSurPile = (etat.couches || []).some(c => c.genre === "attaque" || (c.genre === "carte" && c.carte && /attaque/i.test(c.texte) && !cb));
  const rang = etat.fini ? -1 : cb && cb.etape_rang != null ? cb.etape_rang : attaqueSurPile ? 0 : -1;
  if (rang >= 0) {
    const st = el("div", "steps");
    st.setAttribute("aria-label", ETAPES[rang]);
    ETAPES.forEach((t, k) => st.appendChild(el("span", k === rang ? "on" : k < rang ? "done" : null, t)));
    tete.appendChild(st);
  } else tete.appendChild(el("span", "step", etat.fini ? "Partie terminée" : etat.votre_tour ? "Votre tour" : "Tour du bot"));
  tete.appendChild(el("span", "step prio", etat.priorite && !etat.fini ? "priorité : " + (etat.priorite === "vous" ? "vous" : "bot") : ""));
  m.appendChild(tete);

  if (!cb || !cb.maillons.length) {
    m.appendChild(el("div", "chainempty", cb ? "Chaîne de combat" : "Chaîne fermée"));
    return;
  }
  const liens = el("div", "links");
  liens.id = "liens";
  const defense = genreDecision() === "defense";
  cb.maillons.forEach((l, i) => {
    const cur = i === cb.maillons.length - 1;
    if (i) liens.appendChild(el("span", "arrow"));
    const lk = el("div", "lk " + (cur ? "cur" : "old") + (l.par_vous ? " mine" : ""));
    lk.dataset.o = l.par_vous ? "vous" : "bot";
    const lkc = el("div", "lkc");
    lkc.appendChild(carte(l.attaque, true));
    // Bloqueurs en cours de choix posés sur le maillon (un clic les reprend).
    const tente = cur && defense ? brouillon.map(carteParId).filter(Boolean) : [];
    const d = l.defense_totale + tente.reduce((s, c) => s + Math.max(0, c.defense || 0), 0);
    if (cur && l.puissance !== null) {
      const cl = el("div", "clash");
      cl.append(el("small", null, "Attaque"), el("b", "cp", String(l.puissance)), el("span", "sep"), el("b", "cdf", String(d)), el("small", null, "Défense"));
      lkc.appendChild(cl);
    }
    if (l.defenseurs.length || tente.length) {
      const ds = el("div", "defs");
      for (const x of l.defenseurs) ds.appendChild(carte(x, true));
      for (const x of tente) { const c = carte(x, true); c.classList.add("tente"); c.title = "Bloqueur posé : touchez-le pour le reprendre."; ds.appendChild(c); }
      lkc.appendChild(ds);
    }
    lk.appendChild(lkc);
    const info = el("div", "info");
    info.appendChild(el("span", "muted", `Maillon ${i + 1} · ${l.par_vous ? etat.vous.heros : etat.bot.heros}` + (l.cible && !/^(vous|le bot)$/i.test(l.cible) && l.cible !== etat.vous.heros && l.cible !== etat.bot.heros ? " → " + l.cible : "")));
    const mots = [l.dominate ? "Dominate" : "", l.go_again ? "Go again" : ""].filter(Boolean).join(" · ");
    if (mots) info.appendChild(el("span", "muted", mots));
    if (l.touche) info.appendChild(el("span", "res-hit", "Touche"));
    else if (!cur || l.puissance === null) info.appendChild(el("span", "res-blk", "Bloquée"));
    lk.appendChild(info);
    liens.appendChild(lk);
  });
  m.appendChild(liens);
  liens.scrollLeft = liens.scrollWidth;
}

// La pile (Arsenal JS) : la plus ancienne couche en haut, la plus récente
// devant ; celle de devant se résout en premier.
function pileCouches() {
  const p = $("pile");
  p.replaceChildren();
  const cs = (etat.couches || []).slice().reverse();
  p.classList.toggle("live", cs.length > 0);
  p.appendChild(el("span", "lab", "Pile"));
  const ls = el("div", "layers");
  for (const c of cs) {
    const ly = el("div", "layer");
    ly.title = c.texte;
    if (c.carte) ly.appendChild(carte(c.carte, true));
    else ly.appendChild(el("div", "fantome", c.texte));
    const t = el("span", "tag", c.genre);
    t.appendChild(el("small", null, c.par_vous ? "vous" : "bot"));
    ly.appendChild(t);
    ls.appendChild(ly);
  }
  p.appendChild(ls);
  p.appendChild(el("em", null, "la carte de devant se résout en premier"));
}

// ---------- le ruban d'invite (Arsenal JS, barHTML) ----------
let boutonsRuban = [];
function bouton(texte, fn, o = {}) {
  const b = el("button", (o.principal ? "principal" : "") + (o.discret ? " discret" : ""));
  b.textContent = texte;
  if (o.i !== undefined) b.dataset.i = o.i;
  if (o.carte !== undefined && o.carte !== null) b.addEventListener("mouseenter", () => montrer(o.carte));
  b.disabled = !!o.inactif;
  b.addEventListener("click", fn);
  boutonsRuban.push(b);
  return b;
}
const html = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
function maillonActif() { const cb = etat.combat; return cb && cb.maillons.length ? cb.maillons[cb.maillons.length - 1] : null; }

function phrasePriorite() {
  const v = etat.vous, l = maillonActif(), cb = etat.combat;
  const pw = l && l.puissance !== null ? `<b>${l.puissance}</b> contre <b>${l.defense_totale}</b>` : "";
  const haut = (etat.couches || [])[0];
  if (haut) return `Sur la pile : <b>${html(haut.texte)}</b>. Vous pouvez répondre ou passer pour la laisser se résoudre.`;
  if (!cb && etat.votre_tour && etat.phase_action) return `Votre tour : <b>${v.ressources}</b> ressource${v.ressources > 1 ? "s" : ""}, <b>${v.points_action}</b> point${v.points_action > 1 ? "s" : ""} d'action. Choisissez une carte, votre arme, une capacité, ou terminez votre phase d'action.`;
  const r = cb ? cb.etape_rang : null;
  if (r === 5) return etat.votre_tour ? `Maillon résolu. Attaquez à nouveau (<b>${v.points_action}</b> action${v.points_action > 1 ? "s" : ""}) ou passez : la chaîne se fermera.` : "Fin du maillon du bot. Vous pouvez jouer un instant ou passer.";
  if (r === 3 && l) return l.par_vous ? `Réactions sur votre attaque : ${pw}. Jouez une réaction d'attaque ou passez.` : `Le bot attaque : ${pw}. Jouez une réaction de défense ou passez.`;
  if (cb && r !== null && l) return `${ETAPES[r]}${pw ? " (" + pw + ")" : ""}. Vous pouvez jouer un instant ou passer.`;
  return html(etat.decision.titre);
}

function ruban() {
  const q = $("question"), o = $("options");
  o.replaceChildren();
  boutonsRuban = [];
  const d = etat.decision;
  if (etat.fini) { q.innerHTML = html(etat.resultat || "Partie terminée."); return; }
  if (!d) { q.innerHTML = '<span class="thinking">Le bot joue…</span>'; return; }
  const sp = () => o.appendChild(el("span", "sp"));
  const passe = d.options.find(x => x.passer);
  const autres = (filtre) => d.options.filter(x => !x.passer && filtre(x));
  const cartesDe = (x) => x.carte !== null && x.carte !== undefined;

  if (d.genre === "priorite") {
    q.innerHTML = phrasePriorite();
    if (sel !== null) for (const x of optionsDeCarte(sel)) o.appendChild(bouton(x.texte, () => choisir(x.i), { principal: true, i: x.i, carte: x.carte }));
    // Capacités (arme, héros, équipement, permanents) et options sans carte.
    for (const x of autres(x => !cartesDe(x) || (!dansMain(x.carte) && !dansArsenal(x.carte)))) o.appendChild(bouton(x.texte, () => choisir(x.i), { i: x.i, carte: x.carte }));
    // Sans carte sélectionnée : les réactions de la main en boutons (comme l'Arsenal JS).
    if (sel === null) for (const x of autres(x => cartesDe(x) && dansMain(x.carte) && /réaction/.test((carteParId(x.carte) || {}).genre || ""))) o.appendChild(bouton(x.texte, () => choisir(x.i), { i: x.i, carte: x.carte }));
    sp();
    if (passe) {
      const propre = etat.votre_tour && !(etat.couches || []).length;
      const r = etat.combat ? etat.combat.etape_rang : null;
      const lib = propre && !etat.combat && etat.phase_action ? "Fin de ma phase d'action" : propre && r === 5 ? "Passer (fermer la chaîne)" : "Passer";
      o.appendChild(bouton(lib, () => choisir(passe.i), { discret: !propre, i: passe.i }));
      if (!etat.votre_tour) o.appendChild(bouton("Passer jusqu'à la fin du tour", () => { passerTour = etat.tour; choisir(passe.i); }, { discret: true }));
    }
    return;
  }
  if (d.genre === "defense") {
    const l = maillonActif();
    const tente = brouillon.map(carteParId).filter(Boolean);
    const dd = (l ? l.defense_totale : 0) + tente.reduce((s, c) => s + Math.max(0, c.defense || 0), 0);
    const p = l && l.puissance !== null ? l.puissance : 0;
    const notes = [];
    if (l && l.dominate) notes.push("Dominate : une carte de la main au plus.");
    if (d.options.some(x => x.carte !== null && dansArsenal(x.carte))) notes.push("Votre arsenal peut bloquer.");
    q.innerHTML = l
      ? `${l.par_vous ? "Vous" : "Le bot"} · ${html(l.attaque.nom_fr || l.attaque.nom || "attaque")} <b>${p}</b> — défense <b>${dd}</b>, vous subissez <b>${Math.max(0, p - dd)}</b>.${notes.length ? ` <span class="muted">${notes.join(" ")}</span>` : ""}`
      : html(d.titre);
    q.title = "Touchez les cartes qui défendent (main, équipement), puis validez. Le total ne compte pas les préventions éventuelles.";
    const legal = d.options.filter(x => !x.passer && cartesDe(x)).map(x => x.carte);
    brouillon = brouillon.filter(id => legal.includes(id));
    for (const x of autres(x => !cartesDe(x))) o.appendChild(bouton(x.texte, () => choisir(x.i), { i: x.i }));
    o.appendChild(bouton(brouillon.length ? "Valider la défense" : "Ne pas bloquer", () => envoyerBrouillon(true), { principal: true }));
    if (brouillon.length) o.appendChild(bouton("Tout reprendre", () => { brouillon = []; afficher(); }, { discret: true }));
    return;
  }
  if (d.genre === "pitch") {
    const reste = d.a_payer != null ? Math.max(0, d.a_payer - etat.vous.ressources) : null;
    q.innerHTML = d.a_payer != null ? `Payez <b>${d.a_payer}</b>${reste !== d.a_payer ? ` (reste <b>${reste}</b>)` : ""} : touchez les cartes à pitcher.` : html(d.titre);
    for (const x of autres(x => !cartesDe(x) || !dansMain(x.carte))) o.appendChild(bouton(x.texte, () => choisir(x.i), { i: x.i, carte: x.carte }));
    if (passe) o.appendChild(bouton(passe.texte, () => choisir(passe.i), { discret: true, i: passe.i }));
    return;
  }
  if (d.genre === "arsenal") {
    q.innerHTML = "Fin de tour : touchez une carte de votre main pour la mettre face cachée en arsenal.";
    for (const x of autres(x => !cartesDe(x) || !dansMain(x.carte))) o.appendChild(bouton(x.texte, () => choisir(x.i), { i: x.i, carte: x.carte }));
    if (passe) o.appendChild(bouton("Ne rien mettre en arsenal", () => choisir(passe.i), { discret: true, i: passe.i }));
    return;
  }
  if (d.genre === "ordre_pitch") {
    const ids = d.options.filter(cartesDe).map(x => x.carte);
    brouillon = brouillon.filter(id => ids.includes(id));
    q.innerHTML = "Vos cartes pitchées retournent sous votre deck. Touchez-les dans l'ordre où vous voulez les repiocher (la n° 1 sera piochée en premier).";
    const ordre = el("div", "order");
    for (const id of ids) { const c = carteParId(id); if (c) ordre.appendChild(carte(c, true)); }
    o.appendChild(ordre);
    sp();
    // Le moteur demande la prochaine carte à mettre sous le deck : la
    // première envoyée est la plus haute des cartes remises sous le deck,
    // donc la première repiochée.
    o.appendChild(bouton("Valider cet ordre", () => envoyerBrouillon(false), { principal: true, inactif: brouillon.length < ids.length - 1 }));
    o.appendChild(bouton("Garder l'ordre de pitch", () => { brouillon = ids.slice(); envoyerBrouillon(false); }, { discret: true }));
    if (brouillon.length) o.appendChild(bouton("Recommencer", () => { brouillon = []; afficher(); }, { discret: true }));
    return;
  }
  // Cibles, modes, choix des capacités… : un bouton par option, le premier en évidence.
  q.innerHTML = html(d.titre);
  const horsPlateau = d.options.filter(x => cartesDe(x) && !document.querySelector(`#plateau [data-k="${x.carte}"]`));
  if (horsPlateau.length) {
    const ordre = el("div", "order");
    for (const x of horsPlateau) { const c = carteParId(x.carte); if (c) { const e = carte(c, true); e.classList.add("jouable"); ordre.appendChild(e); } }
    if (ordre.children.length) o.appendChild(ordre);
  }
  d.options.forEach((x, n) => o.appendChild(bouton(x.texte, () => choisir(x.i), { principal: n === 0 && !x.passer, discret: x.passer, i: x.i, carte: x.carte })));
}

// Journal (Arsenal JS) : une ligne de titre par tour, collé en bas tant
// qu'on ne remonte pas.
function journal() {
  const j = $("journal");
  const enBas = j.scrollHeight - j.scrollTop - j.clientHeight < 30;
  j.replaceChildren();
  for (const l of etat.journal) {
    if (l.debut) j.appendChild(el("p", "turn", l.texte.replace(/═/g, "").trim()));
    else j.appendChild(el("p", /dégât|touche|perd/i.test(l.texte) ? "dmg" : null, l.texte));
  }
  if (enBas) j.scrollTop = j.scrollHeight;
}

function afficher() {
  document.querySelectorAll("#v-partie button").forEach(b => b.disabled = false);
  if (!etat || etat.aucune_partie) {
    $("info").textContent = "Aucun match : « Nouveau match » ou « Importer une partie » (menu Options).";
    $("b-annuler").disabled = true;
    $("b-exporter").disabled = true;
    document.dispatchEvent(new CustomEvent("arsenal:etat"));
    return;
  }
  const m = etat.match;
  $("score").textContent = m ? `Match : vous ${m.vous} – ${m.bot} bot (${m.manches} manche${m.manches > 1 ? "s" : ""} à gagner)` : "";
  const prep = etat.phase === "preparation";
  $("preparation").style.display = prep ? "flex" : "none";
  $("plateau").style.display = prep ? "none" : "grid";
  if (prep) {
    const r = etat.reglages;
    $("info").textContent = `Préparation de la manche ${m.jouees + 1}`;
    $("journal").replaceChildren(); // le journal d'une autre partie n'a rien à faire ici
    $("b-annuler").disabled = true;
    $("resultat").textContent = "";
    $("suite").replaceChildren();
    $("question").textContent = "";
    $("options").replaceChildren();
    preparation();
    document.dispatchEvent(new CustomEvent("arsenal:etat"));
    return;
  }
  // Nouvelle décision : la sélection et le brouillon repartent de zéro.
  const d = etat.decision;
  const cle = d ? `${etat.film_no}|${d.genre}|${d.options.length}` : null;
  if (cle !== cleDecision) { cleDecision = cle; sel = null; brouillon = []; }
  if (sel !== null && !optionsDeCarte(sel).length) sel = null;
  const r = etat.reglages;
  $("info").textContent = `Manche ${m ? m.jouees + (etat.fini ? 0 : 1) : 1} · vous : ${r.deck} · bot ${r.nom_bot} : ${r.deck_bot} · graine ${r.graine}`;
  $("b-annuler").disabled = etat.annulables === 0;
  $("b-exporter").disabled = false;
  FX.avant();
  joueur(etat.bot, false, !etat.votre_tour);
  milieu();
  pileCouches();
  joueur(etat.vous, true, etat.votre_tour);
  ruban();
  $("resultat").textContent = etat.resultat || "";
  suite();
  journal();
  finDePartie();
  FX.apres(etat);
  // Lot 9 (partie/extras.js) : conseil du bot, barre d'évaluation.
  document.dispatchEvent(new CustomEvent("arsenal:etat"));
  // « Passer jusqu'à la fin du tour » : on passe tant que c'est le même tour
  // et qu'on n'a que la priorité à rendre.
  if (passerTour !== null) {
    const p = d && d.genre === "priorite" && etat.tour === passerTour && !etat.votre_tour ? d.options.find(x => x.passer) : null;
    if (p) setTimeout(() => { if (!occupe) choisir(p.i); }, 120);
    else if (!d || etat.tour !== passerTour || etat.votre_tour) passerTour = null;
  }
}

function suite() {
  const s = $("suite");
  s.replaceChildren();
  const m = etat.match;
  if (!etat.fini || !m) return;
  if (m.gagnant) {
    s.appendChild(el("div", null, m.gagnant === "vous" ? `Vous gagnez le match ${m.vous}–${m.bot} !` : `Le bot gagne le match ${m.bot}–${m.vous}.`));
  } else if (etat.manche_suivante) {
    const b = el("button", "principal", "Manche suivante");
    b.addEventListener("click", () => action("POST", "/api/manche-suivante"));
    s.appendChild(b);
  }
}

// Fin de partie (Arsenal JS, showEnd) : fenêtre une fois par manche finie.
let finVue = null;
function finDePartie() {
  if (!etat.fini) { finVue = null; return; }
  const cle = `${etat.reglages.graine}|${etat.match ? etat.match.jouees : 0}`;
  if (finVue === cle) return;
  finVue = cle;
  const d = $("d-fin");
  const v = etat.vous, b = etat.bot, st = etat.stats || {};
  const gagne = v.vie > 0 && b.vie <= 0, perdu = v.vie <= 0 && b.vie > 0;
  $("d-fin-titre").textContent = gagne ? "Victoire !" : perdu ? "Victoire du bot" : (etat.resultat || "Partie terminée");
  $("d-fin-pv").textContent = `${etat.tour} tours. ${v.heros} finit à ${Math.max(0, v.vie)} PV, ${b.heros} à ${Math.max(0, b.vie)} PV.`;
  $("d-fin-stats").innerHTML = `Vous avez infligé <b class="num">${st.infliges || 0}</b> dégâts et subi <b class="num">${st.subis || 0}</b>. Vous avez défendu avec <b class="num">${st.blocs_main || 0}</b> carte${(st.blocs_main || 0) > 1 ? "s" : ""} de votre main et <b class="num">${st.blocs_autres || 0}</b> autre${(st.blocs_autres || 0) > 1 ? "s" : ""} (équipement, arsenal).`;
  const m = etat.match;
  $("d-fin-match").textContent = m ? (m.gagnant ? (m.gagnant === "vous" ? `Vous gagnez le match ${m.vous}–${m.bot} !` : `Le bot gagne le match ${m.bot}–${m.vous}.`) : `Match : vous ${m.vous} – ${m.bot} bot.`) : "";
  $("d-fin-suivante").hidden = !etat.manche_suivante;
  $("d-fin-annuler").hidden = !etat.annulables;
  if (!d.open) d.showModal();
}

// ---------- nouvelle partie ----------

async function preparerDialogue() {
  const v = await api("GET", "/api/decks");
  const f = $("f-nouvelle");
  const options = (liste) => liste.map(n => { const o = el("option", null, n); o.value = n; return o; });
  // Decks du projet, puis decks maison jouables (lot 4a, volet « Mes decks »).
  const remplir = (sel, liste) => {
    const garde = sel.value;
    sel.replaceChildren(...options(liste || v.decks));
    if (!liste && (v.decks_maison || []).length) {
      const g = el("optgroup");
      g.label = "Decks maison";
      g.append(...options(v.decks_maison));
      sel.appendChild(g);
    }
    if (garde) sel.value = garde;
  };
  remplir(f.deck);
  remplir(f.bot);
  remplir(f.niveau, [...v.niveaux, "b2-500ms", "b2-1000"]);
}

async function ouvrirNouveau() {
  try { await preparerDialogue(); } catch (e) { /* on garde la liste déjà là */ }
  const f = $("f-nouvelle");
  if (etat && etat.reglages) {
    f.deck.value = etat.reglages.deck;
    f.bot.value = etat.reglages.deck_bot;
    f.niveau.value = etat.reglages.niveau;
    if (etat.reglages.manches) f.manches.value = String(etat.reglages.manches);
  }
  f.graine.value = "";
  if (!$("d-nouvelle").open) $("d-nouvelle").showModal();
}
$("b-nouvelle").addEventListener("click", ouvrirNouveau);
// Le volet Jouer de l'accueil mène à #nouveau-match : on ouvre la fenêtre.
document.addEventListener("arsenal:vue", (e) => { if (e.detail.adresse === "nouveau-match") ouvrirNouveau(); });
$("d-nouvelle").addEventListener("close", () => {
  if ($("d-nouvelle").returnValue !== "ok") return;
  const f = $("f-nouvelle");
  const q = new URLSearchParams({ deck: f.deck.value, bot: f.bot.value, niveau: f.niveau.value, manches: f.manches.value, graine: f.graine.value.trim() });
  action("POST", "/api/nouvelle?" + q.toString());
});
$("b-annuler").addEventListener("click", annuler);

// ---------- sauvegarde (lot 3) ----------
// Le serveur sauvegarde déjà tout seul après chaque choix ; ces boutons
// servent à garder un match dans un fichier, ou à le reprendre plus tard.
$("b-exporter").addEventListener("click", async () => {
  try {
    const v = await api("GET", "/api/sauvegarde");
    const lien = el("a");
    lien.href = URL.createObjectURL(new Blob([JSON.stringify(v, null, 1)], { type: "application/json" }));
    const d = new Date();
    const p2 = (n) => String(n).padStart(2, "0");
    lien.download = `arsenal-${d.getFullYear()}-${p2(d.getMonth() + 1)}-${p2(d.getDate())}-${p2(d.getHours())}h${p2(d.getMinutes())}.json`;
    document.body.appendChild(lien);
    lien.click();
    lien.remove();
    setTimeout(() => URL.revokeObjectURL(lien.href), 2000);
  } catch (e) { $("erreur").textContent = e.message; }
});
$("b-importer").addEventListener("click", () => $("f-importer").click());
$("f-importer").addEventListener("change", async () => {
  const f = $("f-importer").files[0];
  $("f-importer").value = "";
  if (!f || occupe) return;
  occupe = true;
  $("erreur").textContent = "";
  try {
    const r = await fetch("/api/charger", { method: "POST", body: await f.text(), headers: { "Content-Type": "application/json" } });
    const v = await r.json();
    if (!r.ok || v.erreur) throw new Error(v.erreur || ("erreur " + r.status));
    etat = v;
    Film.vu(etat);
    if (Arsenal.vue() !== "partie") Arsenal.aller("partie");
  } catch (e) {
    $("erreur").textContent = "Import impossible : " + e.message;
    const ailleurs = document.getElementById("dk-erreur"); // écran Jouer
    if (ailleurs && Arsenal.vue() !== "partie") ailleurs.textContent = "Import impossible : " + e.message;
  }
  occupe = false;
  afficher();
});

// Pour l'écran Jouer (decks/decks.js) : lancer un match, charger une partie.
window.PartieArsenal = {
  async lancer(reglages, auHasard) {
    const v = await api("POST", "/api/nouvelle?" + new URLSearchParams(reglages).toString());
    if (auHasard) botAuHasard.add(v.reglages.graine + ">" + v.reglages.deck_bot);
    etat = v;
    Film.vu(etat);
    Arsenal.aller("partie");
    afficher();
  },
  charger: () => $("f-importer").click(),
};

document.addEventListener("keydown", (e) => {
  if (Arsenal.vue() !== "partie") return;
  if ($("d-nouvelle").open || $("d-bug").open || e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;
  if (e.key === "Escape" && !$("gopts").hidden) { $("gopts").hidden = true; $("b-options").setAttribute("aria-expanded", "false"); $("b-options").focus(); return; }
  if (etat && etat.phase === "preparation") return;
  if ((e.key === "z" && (e.ctrlKey || e.metaKey)) || e.key === "a" || e.key === "A") { e.preventDefault(); annuler(); return; }
  const d = etat && etat.decision;
  if (!d) return;
  if ($("d-fin").open) return;
  if (e.key >= "1" && e.key <= "9") {
    // Les boutons du ruban, dans l'ordre affiché.
    const b = boutonsRuban.filter(x => !x.disabled)[Number(e.key) - 1];
    if (b) b.click();
  } else if (e.key === "p" || e.key === "P") {
    if (d.genre === "defense") { envoyerBrouillon(true); return; }
    const p = d.options.find(o => o.passer);
    if (p) choisir(p.i);
  } else if (e.key === "Escape") { fermerMenu(); if (sel !== null || brouillon.length) { sel = null; brouillon = []; afficher(); } }
});

// ---------- colonne de droite : Options, abandon, fin de partie ----------
$("b-options").addEventListener("click", (e) => {
  const g = $("gopts"), ouvert = g.hidden;
  g.hidden = !ouvert;
  $("b-options").setAttribute("aria-expanded", String(ouvert));
  e.stopPropagation();
});
document.addEventListener("click", (e) => {
  const g = $("gopts");
  if (!g.hidden && !g.contains(e.target) && e.target !== $("b-options")) { g.hidden = true; $("b-options").setAttribute("aria-expanded", "false"); }
});
$("b-abandon").addEventListener("click", async () => {
  $("gopts").hidden = true;
  if (!confirm("Abandonner le match en cours ? Il sera oublié (sauvegarde comprise).")) return;
  try { etat = await api("POST", "/api/abandon"); } catch (e) { $("erreur").textContent = e.message; return; }
  afficher();
  Arsenal.aller("decks");
});
$("d-fin").addEventListener("close", () => {
  const v = $("d-fin").returnValue;
  if (v === "suivante") action("POST", "/api/manche-suivante");
  else if (v === "annuler") annuler();
  else if (v === "exporter") $("b-exporter").click();
  else if (v === "nouveau") ouvrirNouveau();
  else if (v === "decks") Arsenal.aller("decks");
});

(async () => {
  try { await preparerDialogue(); } catch (e) { $("erreur").textContent = e.message; }
  try { etat = await api("GET", "/api/etat"); } catch (e) { $("erreur").textContent = e.message; }
  Film.vu(etat);
  afficher();
})();

// Survol d'une carte de la main : elle s'incline vers le curseur (Arsenal JS).
document.addEventListener("pointermove", (e) => {
  const h = e.target && e.target.closest ? e.target.closest("#j-vous .hand .hc") : null;
  for (const x of document.querySelectorAll("#j-vous .hand .carte.incline")) if (!h || !h.contains(x)) { x.classList.remove("incline"); x.style.removeProperty("--rx"); x.style.removeProperty("--ry"); }
  if (!h || !FX.on()) return;
  const c = h.firstElementChild; if (!c || c.classList.contains("tiree")) return;
  const r = h.getBoundingClientRect(), u = (e.clientX - r.left) / r.width - 0.5, v = (e.clientY - r.top) / r.height - 0.5;
  c.classList.add("incline");
  c.style.setProperty("--rx", (v * -12).toFixed(1) + "deg");
  c.style.setProperty("--ry", (u * 14).toFixed(1) + "deg");
});
