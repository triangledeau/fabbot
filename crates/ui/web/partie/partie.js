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

function cliquer(id, ev) {
  montrer(id);
  const opts = optionsDeCarte(id);
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
  d.replaceChildren(el("h2", null, "Carte"));
  if (c && c.image) {
    const g = el("div", "grande");
    g.appendChild(Images.img(c.image, "normal", g, "grande"));
    d.appendChild(g);
  }
  if (c && c.nom_fr) d.appendChild(el("div", "nom-fr", c.nom_fr));
  d.appendChild(el("div", "fiche", v.fiche));
  if (v.type) d.appendChild(el("div", "type", v.type));
  d.appendChild(el("div", "texte", v.texte));
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

// ---------- préparation d'une manche (phase de side) ----------

// Choix en cours de l'humain (gardé tant que l'étape ne change pas).
let choixArene = null;   // ensemble des positions choisies
let choixDeck = null;    // quantité par carte distincte
let etapeVue = null;

function preparation() {
  const p = etat.preparation;
  const cle = etat.match.jouees + "-" + p.etape;
  if (cle !== etapeVue) {
    etapeVue = cle;
    choixArene = new Set(p.arene.filter(c => c.propose).map(c => c.i));
    choixDeck = p.deck.map(c => c.propose);
  }
  const s = $("preparation");
  s.replaceChildren();
  const manche = etat.match.jouees + 1;
  const tete = el("div", "bloc");
  tete.appendChild(el("h2", null, `Manche ${manche} — vous (${p.heros}) contre le bot (${p.heros_bot})`));
  const prem = p.premier ? (p.premier === "vous" ? "Vous commencez." : "Le bot commence.") : "";
  tete.appendChild(el("div", null, (p.choisit_premier === "vous" ? "Vous choisissez qui commence. " : "Le bot choisit qui commence. ") + prem));
  s.appendChild(tete);

  if (p.etape === "premier") {
    const b = el("div", "bloc");
    b.appendChild(el("div", null, "Qui commence la manche ? (CR 4.1.3 : choisi avant l'équipement et le deck)"));
    const l = el("div", "barre");
    const b1 = el("button", "principal", "Je commence");
    b1.addEventListener("click", () => action("POST", "/api/premier?qui=vous"));
    const b2 = el("button", null, "Le bot commence");
    b2.addEventListener("click", () => action("POST", "/api/premier?qui=bot"));
    l.append(b1, b2);
    b.appendChild(l);
    s.appendChild(b);
  } else if (p.etape === "arene") {
    const b = el("div", "bloc");
    const barre = el("div", "barre");
    const v = el("button", "principal", "Valider l'équipement");
    v.addEventListener("click", () => action("POST", "/api/arene?cartes=" + [...choixArene].join(",")));
    barre.appendChild(v);
    barre.appendChild(el("span", "aide", "Une carte par zone (deux mains pour les armes). Le bot choisit le sien sans voir le vôtre ; vous le verrez ensuite."));
    b.appendChild(barre);
    const zones = [];
    for (const c of p.arene) if (!zones.includes(c.zone)) zones.push(c.zone);
    for (const z of zones) {
      const cartes = p.arene.filter(c => c.zone === z);
      const zz = el("div", "zone");
      zz.appendChild(el("div", "titre", z));
      const r = el("div", "cartes");
      for (const c of cartes) {
        const e = carte(c, true);
        e.classList.add(choixArene.has(c.i) ? "choisie" : "non-choisie");
        e.style.cursor = "pointer";
        e.addEventListener("click", () => {
          if (choixArene.has(c.i)) choixArene.delete(c.i);
          else {
            // Hors armes : une seule carte par zone.
            if (z !== "Armes") for (const o of cartes) choixArene.delete(o.i);
            choixArene.add(c.i);
          }
          preparation();
        });
        r.appendChild(e);
      }
      zz.appendChild(r);
      b.appendChild(zz);
    }
    s.appendChild(b);
  } else if (p.etape === "deck") {
    const ab = el("div", "bloc");
    ab.appendChild(zone("Équipement du bot (présenté, CR 4.1.5)", p.arene_bot, true));
    s.appendChild(ab);
    const b = el("div", "bloc");
    const total = choixDeck.reduce((a, n) => a + n, 0);
    const barre = el("div", "barre");
    const t = el("span", total === p.taille_deck ? "ok" : "ko", `${total} / ${p.taille_deck} cartes`);
    t.id = "total";
    const v = el("button", "principal", "Valider le deck");
    v.disabled = total !== p.taille_deck;
    v.addEventListener("click", () => action("POST", "/api/deck?n=" + choixDeck.join(",")));
    const r0 = el("button", null, "Revenir à la proposition");
    r0.addEventListener("click", () => { choixDeck = p.deck.map(c => c.propose); preparation(); });
    barre.append(t, v, r0, el("span", "aide", "Choisissez exactement 40 cartes parmi celles de votre card-pool."));
    b.appendChild(barre);
    const r = el("div", "cartes");
    p.deck.forEach((c, k) => {
      const box = el("div", "carte-deck");
      const e = carte(c, true);
      if (choixDeck[k] === 0) e.classList.add("non-choisie");
      box.appendChild(e);
      const q = el("div", "qte");
      const moins = el("button", null, "−");
      moins.disabled = choixDeck[k] === 0;
      moins.addEventListener("click", () => { choixDeck[k]--; preparation(); });
      const plus = el("button", null, "+");
      plus.disabled = choixDeck[k] >= c.max;
      plus.addEventListener("click", () => { choixDeck[k]++; preparation(); });
      q.append(moins, el("b", null, `${choixDeck[k]}/${c.max}`), plus);
      box.appendChild(q);
      r.appendChild(box);
    });
    b.appendChild(r);
    s.appendChild(b);
  }
}

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

function joueur(j, estVous, actif) {
  const qui = estVous ? "vous" : "bot";
  const b = el("div", "aire " + (estVous ? "m" : "o"));
  b.dataset.o = qui;
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
  if (j.marque) heros.push(el("span", "marque", "marqué"));

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
      z("Héros", "big", heros, { ancre: "heros-" + qui, classe: "heros" }),
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
  b.appendChild(tapis);

  // À côté du tapis : ressources, points d'action, main du bot.
  const et = el("div", "etat-joueur");
  const c = (lib, v, a) => { const s = el("span", "compteur"); s.append(el("b", null, String(v)), " " + lib); if (a) s.dataset.a = a + "-" + qui; return s; };
  et.appendChild(el("span", "qui", (estVous ? "Vous — " : "Bot — ") + j.heros));
  et.appendChild(c("ressource" + (j.ressources > 1 ? "s" : ""), j.ressources, "res"));
  et.appendChild(c("point" + (j.points_action > 1 ? "s" : "") + " d'action", j.points_action));
  if (!estVous) {
    const m = el("div", "main-bot");
    m.dataset.a = "main-bot";
    m.appendChild(el("span", "lib", `Main (${j.main_nombre})`));
    const f = el("div", "eventail dos-main");
    for (let i = 0; i < j.main_nombre; i++) f.appendChild(dos());
    m.appendChild(f);
    et.appendChild(m);
  }
  b.appendChild(et);
  if (estVous) {
    const m = el("div", "main-vous");
    m.dataset.a = "main-vous";
    m.appendChild(el("span", "lib", `Votre main (${j.main.length})`));
    const cs = el("div", "cartes");
    if (!j.main.length) cs.appendChild(el("span", "vide", "Main vide"));
    for (const x of j.main) cs.appendChild(carte(x, false));
    m.appendChild(cs);
    b.appendChild(m);
  }

  const conteneur = $(estVous ? "j-vous" : "j-bot");
  conteneur.classList.toggle("actif", actif);
  conteneur.replaceChildren(b);
}

function milieu() {
  const m = $("milieu");
  m.replaceChildren();
  m.dataset.depot = "chaine";
  const cb = etat.combat;
  if (cb) {
    m.appendChild(el("div", "titre", "Chaîne de combat — étape : " + cb.etape));
    cb.maillons.forEach((l, i) => {
      const r = el("div", "maillon");
      const res = el("div", "resume");
      res.appendChild(el("b", null, "Maillon " + (i + 1)));
      res.appendChild(el("div", null, (l.par_vous ? "Vous attaquez " : "Le bot attaque ") + l.cible));
      res.appendChild(el("div", null, l.puissance !== null ? l.puissance + " puissance" : "terminée"));
      if (l.defenseurs.length) res.appendChild(el("div", null, "défense totale " + l.defense_totale));
      if (l.touche) res.appendChild(el("div", null, "a touché"));
      r.appendChild(res);
      r.dataset.o = l.par_vous ? "vous" : "bot";
      r.appendChild(carte(l.attaque, true));
      for (const d of l.defenseurs) r.appendChild(carte(d, true));
      m.appendChild(r);
    });
  }
  if (etat.pile.length) {
    m.appendChild(el("div", "titre", "Pile (du dessus vers le dessous)"));
    const ul = el("ul", "pile");
    for (const p of etat.pile) ul.appendChild(el("li", null, p));
    m.appendChild(ul);
  }
  if (!cb && !etat.pile.length) m.appendChild(el("span", "vide", "Pas de chaîne de combat ni de pile."));
}

function panneau() {
  $("resultat").textContent = etat.resultat || "";
  const d = etat.decision;
  const o = $("options");
  o.replaceChildren();
  if (!d) {
    $("question").textContent = etat.fini ? "" : "En attente…";
    return;
  }
  $("question").textContent = d.titre;
  d.options.forEach((x, n) => {
    const b = el("button", x.passer ? "passer" : null);
    b.appendChild(el("span", "no", String(n + 1)));
    b.appendChild(document.createTextNode(" " + x.texte));
    b.addEventListener("click", () => choisir(x.i));
    if (x.carte !== null) b.addEventListener("mouseenter", () => montrer(x.carte));
    o.appendChild(b);
  });
}

function journal() {
  const j = $("journal");
  const enBas = j.scrollHeight - j.scrollTop - j.clientHeight < 30;
  j.replaceChildren();
  for (const l of etat.journal) {
    j.appendChild(el("div", l.debut ? "debut" : null, l.debut ? `Tour ${l.tour} — ${l.texte}` : "· " + l.texte));
  }
  if (enBas) j.scrollTop = j.scrollHeight;
}

function afficher() {
  document.querySelectorAll("button").forEach(b => b.disabled = false);
  if (!etat || etat.aucune_partie) {
    $("info").textContent = "Aucun match : cliquez « Nouveau match » (ou « Importer » une sauvegarde).";
    $("b-annuler").disabled = true;
    $("b-exporter").disabled = true;
    return;
  }
  const m = etat.match;
  $("score").textContent = m ? `Match : vous ${m.vous} – ${m.bot} bot (${m.manches} manche${m.manches > 1 ? "s" : ""} à gagner)` : "";
  const prep = etat.phase === "preparation";
  $("preparation").style.display = prep ? "flex" : "none";
  $("plateau").style.display = prep ? "none" : "flex";
  if (prep) {
    const r = etat.reglages;
    $("info").textContent = `Préparation de la manche ${m.jouees + 1} · vous : ${r.deck} · bot ${r.niveau} : ${r.deck_bot}`;
    $("b-annuler").disabled = true;
    $("resultat").textContent = "";
    $("suite").replaceChildren();
    $("question").textContent = "Choisissez votre équipement et votre deck (à gauche).";
    $("options").replaceChildren();
    preparation();
    return;
  }
  const r = etat.reglages;
  $("info").textContent = `Manche ${m ? m.jouees + (etat.fini ? 0 : 1) : 1} · tour ${etat.tour} (${etat.votre_tour ? "votre tour" : "tour du bot"}) · vous : ${r.deck} · bot ${r.nom_bot} : ${r.deck_bot} · graine ${r.graine}`;
  $("b-annuler").disabled = etat.annulables === 0;
  $("b-annuler").textContent = "↶ Annuler" + (etat.annulables ? ` (${etat.annulables})` : "");
  FX.avant();
  joueur(etat.bot, false, !etat.votre_tour);
  milieu();
  joueur(etat.vous, true, etat.votre_tour);
  panneau();
  suite();
  journal();
  FX.apres(etat);
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

// ---------- nouvelle partie ----------

async function preparerDialogue() {
  const v = await api("GET", "/api/decks");
  const f = $("f-nouvelle");
  const options = (liste) => liste.map(n => { const o = el("option", null, n); o.value = n; return o; });
  // Decks du projet, puis decks maison jouables (lot 4a, volet « Mes decks »).
  const remplir = (sel) => {
    const garde = sel.value;
    sel.replaceChildren(...options(v.decks));
    if ((v.decks_maison || []).length) {
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
  } catch (e) { $("erreur").textContent = "Import impossible : " + e.message; }
  occupe = false;
  afficher();
});

document.addEventListener("keydown", (e) => {
  if (Arsenal.vue() !== "partie") return;
  if ($("d-nouvelle").open || e.target.tagName === "INPUT") return;
  if (etat && etat.phase === "preparation") return;
  if ((e.key === "z" && (e.ctrlKey || e.metaKey)) || e.key === "a" || e.key === "A") { e.preventDefault(); annuler(); return; }
  const d = etat && etat.decision;
  if (!d) return;
  if (e.key >= "1" && e.key <= "9") {
    const n = Number(e.key) - 1;
    if (n < d.options.length) choisir(d.options[n].i);
  } else if (e.key === "p" || e.key === "P") {
    const p = d.options.find(o => o.passer);
    if (p) choisir(p.i);
  } else if (e.key === "Escape") fermerMenu();
});

(async () => {
  try { await preparerDialogue(); } catch (e) { $("erreur").textContent = e.message; }
  try { etat = await api("GET", "/api/etat"); } catch (e) { $("erreur").textContent = e.message; }
  Film.vu(etat);
  afficher();
})();
