// Didacticiel « Apprendre » (volet #apprendre ; phase 2 du plan de
// rapatriement, docs/didacticiel.md). Le contenu est dans lecons.js ; les
// scènes tournent sur le vrai moteur (crates/ui/src/api_apprendre.rs) :
// la page n'applique aucune règle elle-même.
"use strict";
(() => {
  const sec = document.getElementById("v-apprendre");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const CLE = "arsenal.apprendre.v1";
  const lire = () => { try { return JSON.parse(localStorage.getItem(CLE)) || {}; } catch (e) { return {}; } };
  const ecrire = (p) => { try { localStorage.setItem(CLE, JSON.stringify(p)); } catch (e) { /* stockage indisponible */ } };
  const crs = (l) => (l && l.length) ? `<span class="ap-cr">${l.map((x) => "CR " + esc(x)).join(" · ")}</span>` : "";
  const api = async (methode, chemin) => {
    const r = await fetch(chemin, { method: methode });
    const v = await r.json();
    if (!r.ok || v.erreur) throw new Error(v.erreur || r.status);
    return v;
  };

  let lecon = null, etape = 0, scene = null;

  function sommaire() {
    const p = lire();
    lecon = null; scene = null;
    const chap = LECONS.map((c) => {
      const items = c.lecons.length
        ? c.lecons.map((l) => `<li><button class="ap-lien${p[l.id] ? " fait" : ""}" data-l="${esc(l.id)}">${p[l.id] ? "✓ " : ""}${esc(l.titre)}</button></li>`).join("")
        : `<li class="muted">À venir</li>`;
      const faites = c.lecons.filter((l) => p[l.id]).length;
      return `<section class="ap-chap${c.lecons.length ? "" : " vide"}"><h2><span class="num">${c.no}</span> ${esc(c.titre)}</h2>
        ${c.lecons.length ? `<div class="muted petit">${faites} / ${c.lecons.length} leçon${c.lecons.length > 1 ? "s" : ""}</div>` : ""}<ol>${items}</ol></section>`;
    }).join("");
    sec.innerHTML = `<div class="ap">
      <header class="ap-tete"><a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a><h1>Apprendre les règles</h1></header>
      <p class="ap-intro">Les règles de Flesh and Blood, pas à pas, pour commencer de zéro. Chaque leçon explique une notion, te la fait jouer sur le vrai moteur d'Arsenal, puis te pose une question. Les numéros « CR » renvoient aux règles officielles (<i>Comprehensive Rules</i>).</p>
      <div class="ap-chaps">${chap}</div>${couverture()}</div>`;
    sec.querySelectorAll("[data-l]").forEach((b) => b.addEventListener("click", () => ouvrir(b.dataset.l)));
    document.title = "Arsenal — Apprendre";
  }

  // Couverture des règles : pour chaque section du CR, les leçons qui la citent.
  function couverture() {
    const cite = {};
    for (const c of LECONS) for (const l of c.lecons) for (const et of l.etapes) for (const r of (et.cr || [])) {
      const sect = r.split(".").slice(0, 2).join(".").replace(/[a-z]$/, "");
      (cite[sect] = cite[sect] || new Set()).add(l.id);
    }
    const n = CR_SECTIONS.filter(([s]) => cite[s]).length;
    const lignes = CR_SECTIONS.map(([s, t]) => `<tr class="${cite[s] ? "ok" : ""}"><td class="num">${esc(s)}</td><td>${esc(t)}</td><td>${cite[s] ? [...cite[s]].map(esc).join(", ") : '<span class="muted">pas encore</span>'}</td></tr>`).join("");
    return `<details class="ap-couv"><summary><b>Couverture des règles officielles</b> : <span class="num">${n} / ${CR_SECTIONS.length}</span> sections expliquées</summary>
      <table><thead><tr><th>CR</th><th>Section</th><th>Leçons</th></tr></thead><tbody>${lignes}</tbody></table></details>`;
  }

  function ouvrir(id) {
    for (const c of LECONS) for (const l of c.lecons) if (l.id === id) { lecon = { ...l, chap: c }; }
    etape = 0;
    afficher();
  }

  function suivante() {
    const tout = LECONS.flatMap((c) => c.lecons);
    const i = tout.findIndex((l) => l.id === lecon.id);
    return tout[i + 1];
  }

  function afficher() {
    const l = lecon, et = l.etapes[etape];
    const fin = etape >= l.etapes.length;
    const prog = l.etapes.map((_, i) => `<span class="${i < etape ? "ok" : i === etape ? "ici" : ""}"></span>`).join("");
    sec.innerHTML = `<div class="ap">
      <header class="ap-tete"><button class="btn small" id="ap-somm">← Sommaire</button>
        <h1><span class="muted">${l.chap.no}. ${esc(l.chap.titre)} —</span> ${esc(l.titre)}</h1><div class="ap-prog">${prog}</div></header>
      <div class="ap-corps" id="ap-corps"></div></div>`;
    sec.querySelector("#ap-somm").addEventListener("click", sommaire);
    const corps = sec.querySelector("#ap-corps");
    if (fin) {
      const p = lire(); p[l.id] = true; ecrire(p);
      const s = suivante();
      corps.innerHTML = `<div class="ap-carte ap-fin"><h2>Leçon terminée</h2><p>Tu peux la refaire quand tu veux depuis le sommaire.</p>
        <div class="ap-actions"><button class="btn" id="ap-refaire">Refaire</button>${s ? `<button class="btn primary" id="ap-suite">Leçon suivante : ${esc(s.titre)}</button>` : ""}<button class="btn" id="ap-s2">Sommaire</button></div></div>`;
      corps.querySelector("#ap-refaire").onclick = () => ouvrir(l.id);
      corps.querySelector("#ap-s2").onclick = sommaire;
      if (s) corps.querySelector("#ap-suite").onclick = () => ouvrir(s.id);
      return;
    }
    if (et.cartes !== undefined) return montrerCartes(corps, et);
    if (et.texte !== undefined) return texte(corps, et);
    if (et.quiz !== undefined) return quiz(corps, et);
    if (et.scene !== undefined) return jouer(corps, et);
  }

  const avancer = () => { etape++; afficher(); };
  const reculer = () => { if (etape > 0) { etape--; afficher(); } };
  const nav = (suivantOk) => `<div class="ap-actions">${etape > 0 ? `<button class="btn quiet" id="ap-prec">← Précédent</button>` : ""}<button class="btn primary" id="ap-suiv"${suivantOk ? "" : " disabled"}>Continuer →</button></div>`;
  function brancher(corps) {
    const p = corps.querySelector("#ap-prec"); if (p) p.onclick = reculer;
    const s = corps.querySelector("#ap-suiv"); if (s) s.onclick = avancer;
    if (s && !s.disabled) s.focus();
  }

  function texte(corps, et) {
    corps.innerHTML = `<div class="ap-carte"><p class="ap-texte">${et.texte}</p>${crs(et.cr)}${nav(true)}</div>`;
    brancher(corps);
  }

  // Des cartes montrées hors partie (/api/apprendre/fiche), avec le texte de l'étape.
  async function montrerCartes(corps, et) {
    corps.innerHTML = `<div class="ap-carte ap-avec-cartes"><p class="ap-texte">${et.texte}</p>${crs(et.cr)}<div class="ap-rang" id="ap-cartes"></div>${nav(true)}</div>`;
    brancher(corps);
    const zone = corps.querySelector("#ap-cartes");
    for (const [nom, couleur] of et.cartes) {
      try {
        const c = await api("GET", `/api/apprendre/fiche?nom=${encodeURIComponent(nom)}&couleur=${encodeURIComponent(couleur || "")}`);
        const vals = [["Coût", c.cout], ["Pitch", c.pitch], ["Puissance", c.puissance], ["Défense", c.defense], ["PV", c.vie], ["Intellect", c.intellect]]
          .filter(([, v]) => v != null).map(([k, v]) => `${k} <b class="num">${v}</b>`).join(" · ");
        zone.insertAdjacentHTML("beforeend", `<figure class="ap-fig">${carteHtml({ ...c, id: "", genre: "" })}
          <figcaption><b>${esc(c.nom)}</b>${c.couleur ? ` (${esc(c.couleur)})` : ""}<br><span class="petit">${vals}</span>${c.type ? `<br><span class="petit muted">${esc(c.type)}</span>` : ""}</figcaption></figure>`);
      } catch (err) { zone.insertAdjacentHTML("beforeend", `<p class="erreur">${esc(nom)} : ${esc(err.message)}</p>`); }
    }
  }

  function quiz(corps, et) {
    // Les réponses sont mélangées à chaque affichage (la bonne n'est pas toujours la première).
    const ordre = et.choix.map((_, i) => i).sort(() => Math.random() - 0.5);
    corps.innerHTML = `<div class="ap-carte"><div class="ap-etiq">Question</div><p class="ap-texte">${esc(et.quiz)}</p>
      <div class="ap-choix">${ordre.map((i) => `<button class="btn" data-i="${i}">${esc(et.choix[i])}</button>`).join("")}</div>
      <div id="ap-retour" class="ap-retour"></div>${nav(false)}</div>`;
    corps.querySelectorAll("[data-i]").forEach((b) => b.addEventListener("click", () => {
      const bon = Number(b.dataset.i) === et.bonne;
      corps.querySelectorAll("[data-i]").forEach((x) => { x.disabled = true; if (Number(x.dataset.i) === et.bonne) x.classList.add("bon"); });
      if (!bon) b.classList.add("faux");
      corps.querySelector("#ap-retour").innerHTML = `<b>${bon ? "Exact." : "Pas tout à fait."}</b> ${esc(et.pourquoi)} ${crs(et.cr)}${bon ? "" : ` <button class="btn small" id="ap-reessayer">Réessayer</button>`}`;
      const r = corps.querySelector("#ap-reessayer"); if (r) r.onclick = () => quiz(corps, et);
      corps.querySelector("#ap-suiv").disabled = !bon;
      brancher(corps);
    }));
    brancher(corps);
  }

  // ——— Scènes : une petite partie sur le moteur ———
  function carteHtml(c, opt) {
    if (!c || c.cache) return `<div class="ap-c cachee"></div>`;
    const coul = c.couleur ? ` coul-${esc(c.couleur)}` : "";
    const v = [c.cout != null ? `<span class="cout" title="Coût">${c.cout}</span>` : "",
      c.pitch != null ? `<span class="pitch" title="Pitch">${c.pitch}</span>` : "",
      c.puissance != null ? `<span class="pui" title="Puissance">${c.puissance}</span>` : "",
      c.defense != null ? `<span class="def" title="Défense">${c.defense}</span>` : ""].join("");
    // Image officielle de la carte si elle se charge (Images, commun/cartes.js), sinon le cadre dessiné.
    const img = c.image ? `<img alt="" loading="lazy" referrerpolicy="no-referrer" src="${esc(Images.url(c.image))}" onerror="this.parentNode.classList.remove('a-img');this.remove()">` : "";
    return `<button class="ap-c${coul}${img ? " a-img" : ""}${opt != null ? " jouable" : ""}" data-carte="${c.id}"${opt != null ? ` data-opt="${opt}"` : ""} title="${esc(c.nom)}">${img}
      <span class="nom">${esc(c.nom_fr || c.nom)}</span>${c.nom_fr && c.nom_fr !== c.nom ? `<span class="nom-en">${esc(c.nom)}</span>` : ""}
      <span class="genre">${esc(c.genre || "")}</span><span class="vals">${v}</span></button>`;
  }

  function plateau(e) {
    const optDe = {};
    (e.decision?.options || []).forEach((o) => { if (o.carte != null && optDe[o.carte] == null) optDe[o.carte] = o.i; });
    const c = (x) => carteHtml(x, x && optDe[x.id]);
    const heros = (j, qui) => `<div class="ap-heros"><div class="ap-hnom">${esc(qui)} — ${esc(j.heros)}</div>
      <div class="ap-pv"><span class="num">${j.vie}</span> PV</div>
      <div class="muted petit">${j.main_nombre} en main · deck ${j.deck}${qui === "Toi" ? ` · ${j.points_action} point(s) d'action · ${j.ressources} ressource(s)` : ""}</div></div>`;
    const chaine = (e.combat?.maillons || []).map((m) => `<div class="ap-maillon"><div class="muted petit">${m.par_vous ? "Ton attaque" : "Attaque adverse"}${m.puissance != null ? ` — puissance ${m.puissance}` : ""}</div>${c(m.attaque)}
      ${(m.defenseurs || []).length ? `<div class="muted petit">Défendue par (total ${m.defense_totale})</div><div class="ap-rang">${m.defenseurs.map(c).join("")}</div>` : ""}</div>`).join("");
    return `<div class="ap-plateau">
      ${heros(e.bot, "Adversaire")}
      <div class="ap-zone"><div class="ap-etiq">Chaîne de combat <span class="muted">(étape : ${esc(e.combat?.etape || "—")})</span></div><div class="ap-rang">${chaine || `<span class="muted petit">vide</span>`}</div></div>
      <div class="ap-zones">
        <div class="ap-zone"><div class="ap-etiq">Ton pitch</div><div class="ap-rang">${(e.vous.pitch || []).map(c).join("") || `<span class="muted petit">vide</span>`}</div></div>
        <div class="ap-zone"><div class="ap-etiq">Ton arsenal</div><div class="ap-rang">${e.vous.arsenal ? c(e.vous.arsenal) : `<span class="muted petit">vide</span>`}</div></div>
        <div class="ap-zone"><div class="ap-etiq">Ton cimetière</div><div class="ap-rang">${(e.vous.cimetiere || []).map(c).join("") || `<span class="muted petit">vide</span>`}</div></div>
      </div>
      ${heros(e.vous, "Toi")}
      <div class="ap-zone"><div class="ap-etiq">Ta main</div><div class="ap-rang">${(e.vous.main || []).map(c).join("") || `<span class="muted petit">vide</span>`}</div></div>
    </div>`;
  }

  async function jouer(corps, et) {
    corps.innerHTML = `<div class="ap-scene"><div class="ap-gauche"><div class="ap-carte ap-consigne"><div class="ap-etiq">À toi</div><p class="ap-texte">${et.consigne}</p>
        <div id="ap-bravo" class="ap-retour"></div>
        <div class="ap-actions"><button class="btn quiet" id="ap-prec">← Précédent</button><button class="btn small" id="ap-annuler">↶ Annuler</button><button class="btn small" id="ap-recommencer">Recommencer</button><button class="btn primary" id="ap-suiv" disabled>Continuer →</button></div></div>
        <div class="ap-carte"><div class="ap-etiq">Ton choix</div><div id="ap-question"></div><div id="ap-options" class="ap-choix"></div></div>
        <div class="ap-carte" id="ap-detail"><div class="ap-etiq">Carte</div><div class="muted petit">Survole une carte pour lire son texte officiel.</div></div>
        <details class="ap-carte" open><summary class="ap-etiq">Journal</summary><div id="ap-journal" class="petit"></div></details></div>
      <div id="ap-plateau"></div></div>`;
    brancher(corps);
    corps.querySelector("#ap-recommencer").onclick = () => jouer(corps, et);
    corps.querySelector("#ap-annuler").onclick = () => envoyer(corps, et, "POST", "/api/apprendre/annuler");
    try {
      const e = await api("POST", "/api/apprendre/scene?id=" + encodeURIComponent(et.scene));
      scene = { debut: e };
      rendre(corps, et, e);
    } catch (err) { corps.querySelector("#ap-plateau").innerHTML = `<p class="erreur">Scène indisponible : ${esc(err.message)}</p>`; }
  }

  async function envoyer(corps, et, m, chemin) {
    try { rendre(corps, et, await api(m, chemin)); }
    catch (err) { corps.querySelector("#ap-question").innerHTML = `<p class="erreur">${esc(err.message)}</p>`; }
  }

  function rendre(corps, et, e) {
    e.debut = scene.debut;
    corps.querySelector("#ap-plateau").innerHTML = plateau(e);
    const d = e.decision;
    corps.querySelector("#ap-question").innerHTML = d ? `<p>${esc(d.titre)}</p>` : `<p class="muted">${esc(e.resultat || "")}</p>`;
    corps.querySelector("#ap-options").innerHTML = d ? d.options.map((o) => `<button class="btn${o.passer ? " quiet" : ""}" data-o="${o.i}">${esc(o.texte)}</button>`).join("") : "";
    corps.querySelector("#ap-journal").innerHTML = (e.journal || []).slice(-30).map((l) => `<div${l.debut ? ' class="debut"' : ""}>${esc(l.texte)}</div>`).join("");
    const choisir = (i) => envoyer(corps, et, "POST", "/api/apprendre/choix?i=" + i);
    corps.querySelectorAll("[data-o]").forEach((b) => b.addEventListener("click", () => choisir(b.dataset.o)));
    corps.querySelectorAll("[data-opt]").forEach((b) => b.addEventListener("click", () => choisir(b.dataset.opt)));
    corps.querySelectorAll("[data-carte]").forEach((b) => b.addEventListener("mouseenter", () => detail(corps, b.dataset.carte)));
    const ok = !!et.reussi(e);
    const bravo = corps.querySelector("#ap-bravo");
    bravo.innerHTML = ok ? `<b>Réussi.</b> ${esc(typeof et.bravo === "function" ? et.bravo(e) : et.bravo || "")}` : "";
    corps.querySelector("#ap-suiv").disabled = !ok;
    if (ok) corps.querySelector("#ap-suiv").focus();
  }

  async function detail(corps, id) {
    try {
      const v = await api("GET", "/api/apprendre/carte?id=" + id);
      corps.querySelector("#ap-detail").innerHTML = `<div class="ap-etiq">Carte</div><b>${esc(v.carte?.nom || "")}</b> <span class="muted petit">${esc(v.type || "")}</span><p class="petit ap-officiel">${esc(v.texte || "")}</p><div class="muted petit">${esc(v.fiche || "")}</div>`;
    } catch (err) { /* carte cachée */ }
  }

  document.addEventListener("arsenal:vue", (ev) => { if (ev.detail.vue === "apprendre" && !lecon) sommaire(); });
})();
