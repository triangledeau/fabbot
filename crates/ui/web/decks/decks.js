// Écran « Jouer » (adresse #decks), repris de l'écran Jouer de l'Arsenal JS
// (arsenal.html : tplJouer, deckTile, deckMenu, renderDuel, archives,
// nouveau deck ; voir docs/rapatriement-interface.md, J1 à J10).
//  - tuiles « Mes decks » (data/decks/maison/) et « Decks du bot » (data/decks/) ;
//  - clic : choisir son deck ; double-clic : l'ouvrir dans l'éditeur ;
//    clic droit ou ⋯ : menu d'actions ; Entrée / Espace : choisir ;
//  - panneau « Partie contre le bot » : deck du bot, niveau, manches,
//    avertissements, « Ajuster et lancer », « Charger une partie » ;
//  - #archives : decks archivés ; #nouveau-deck : grille des héros Young.
// Aucune règle ici : la vérification des decks vient du serveur (api_decks.rs).
"use strict";
(() => {
  const sec = document.getElementById("v-decks");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const $ = (id) => document.getElementById(id);
  const lire = (k, d) => { try { const v = localStorage.getItem(k); return v === null ? d : JSON.parse(v); } catch (e) { return d; } };
  const ecrire = (k, v) => { try { localStorage.setItem(k, JSON.stringify(v)); } catch (e) { /* navigation privée */ } };

  // Couleurs des héros et des classes (celles de l'Arsenal JS : HCOL, CLSCOL)
  const HCOL = { dorinthea: "#c9483c", ira: "#e0563f", dromai: "#e0782f", iyslander: "#6fd0e6" };
  const CLSCOL = { Warrior: "#c9483c", Guardian: "#d08a2a", Brute: "#b0653a", Ninja: "#e0563f", Wizard: "#4f8fe0", Ranger: "#3fb39c", Runeblade: "#9a6bd1", Mechanologist: "#c7b36b", Illusionist: "#e0782f", Assassin: "#8a5bc8", Pirate: "#3f9fc4", Necromancer: "#7b8b6f", Merchant: "#c9a24a", Thief: "#8f8f8f", Bard: "#d47fb0", Elemental: "#6fd0e6" };
  const couleurClasse = (classe) => { for (const m of String(classe || "").split(/\s+/)) if (CLSCOL[m]) return CLSCOL[m]; return "#c9a24a"; };
  // Niveaux du bot : libellés de joueur (docs/bots.md)
  const NIVEAUX = [
    ["b0", "Niveau 0 : joue au hasard (B0)"],
    ["b1", "Niveau 1 : règles fixes, joue vite (B1)"],
    ["b2", "Niveau 2 : réfléchit plusieurs coups (B2)"],
    ["b3", "Niveau 3 : B2 avec une évaluation apprise (B3)"],
    ["b2-500ms", "Niveau 2, une demi-seconde par décision"],
    ["b2-1000", "Niveau 2, 1 000 essais par décision"],
  ];

  sec.innerHTML = `<div id="dk-vue" class="dkz">
    <header class="top">
      <a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a>
      <h1>Jouer</h1><span class="crumb">Choisis ton deck, celui du bot, ou crée le tien</span><span class="sp"></span>
      <span class="dossier" id="dk-dossier" title="">decks enregistrés sur ton ordinateur</span>
      <a class="btn small" href="#nouveau-deck">＋ Nouveau deck</a>
      <button class="btn small" id="dk-b-importer">Importer une liste</button>
      <button class="btn small" id="dk-b-imprimer">Imprimer un deck</button>
      <a class="btn small" href="#archives">Decks archivés <span class="num" id="dk-nb-arch"></span></a>
    </header>
    <div class="pbody"><div class="lib">
      <div>
        <div class="libsec"><h2>Mes decks <span class="muted aide">clic : choisir · double-clic : modifier · clic droit ou ⋯ : actions</span></h2><div class="dgrid" id="dk-g-mes"></div></div>
        <div class="libsec"><h2>Decks du bot <span class="muted aide">réglés pour le bot ; « Modifier » en fait une copie, l'original ne change pas</span></h2><div class="dgrid" id="dk-g-bot"></div></div>
        <p class="muted petit" id="dk-collection"></p>
      </div>
      <aside class="panel vsp" aria-labelledby="dk-t-duel">
        <h2 class="ph" id="dk-t-duel">Partie contre le bot</h2>
        <div class="vsrow"><div class="hc" id="dk-hc-moi"></div><span class="vsx">VS</span><div class="hc" id="dk-hc-bot"></div></div>
        <label class="champ">Deck du bot <select id="dk-s-bot"></select></label>
        <label class="champ">Niveau du bot <select id="dk-s-niveau">${NIVEAUX.map(([v, l]) => `<option value="${v}">${esc(l)}</option>`).join("")}</select></label>
        <label class="champ">Manches à gagner <select id="dk-s-manches"><option value="1">1 (une seule manche)</option><option value="2">2 (au meilleur des 3)</option><option value="3">3 (au meilleur des 5)</option></select></label>
        <details class="avance"><summary>Options avancées</summary>
          <label class="champ">Graine <input id="dk-graine" placeholder="au hasard" autocomplete="off" spellcheck="false"></label>
          <p class="muted petit">Même graine + mêmes decks = mêmes mélanges (pour rejouer une donne).</p>
        </details>
        <div id="dk-avert"></div>
        <button class="btn primary big" id="dk-lancer">Ajuster et lancer</button>
        <p class="muted petit">Ouvre la préparation de la manche : équipement et 40 cartes parmi les 55.</p>
        <button class="btn small quiet" id="dk-charger" title="Reprendre une partie exportée (.json)">Charger une partie (.json)</button>
        <p class="dk-erreur" id="dk-erreur" role="alert"></p>
      </aside>
    </div></div>
  </div>
  <div id="dk-archives" class="dkz" hidden>
    <header class="top"><a class="btn small" href="#decks">← Jouer</a><h1>Decks archivés</h1><span class="crumb">Hors du choix de deck ; rien n'est perdu</span><span class="sp"></span></header>
    <div class="pbody"><div class="archl" id="dk-l-arch"></div></div>
  </div>
  <div id="dk-nouveau" class="dkz" hidden>
    <header class="top"><a class="btn small" href="#decks">← Jouer</a><h1>Nouveau deck</h1><span class="crumb">Choisis ton héros Young</span><span class="sp"></span></header>
    <div class="pbody"><div class="filt" id="dk-h-filtres"></div><div class="hgrid" id="dk-h-grille"></div></div>
  </div>
  <div id="dk-couche" class="dkz"><dialog id="dk-dlg"><div class="dh" id="dk-dlg-h"></div><div class="db" id="dk-dlg-b"></div><div class="df" id="dk-dlg-f"></div></dialog></div>`;

  // ---------- état ----------
  let MES = [], BOT = [], ARCH_MES = [], HEROS = [];
  const SEL = Object.assign({ moi: null, bot: "hasard", niveau: "b1", manches: "2" }, lire("arsenal-jouer", {}));
  const sauverSel = () => ecrire("arsenal-jouer", SEL);
  let ARCH_BOT = new Set(lire("arsenal-bot-archives", []));
  const tous = () => [...MES, ...BOT];
  const trouver = (id) => tous().find((d) => d.id === id);

  // Un deck de l'API (vérification du serveur) → une tuile
  function deDeck(v, bot) {
    const court = String(v.heros || v.nom).split(",")[0];
    return {
      id: (bot ? "b:" : "m:") + v.nom, nom: v.nom, bot, v,
      titre: bot ? court : v.nom,
      couleur: (bot && HCOL[v.nom]) || couleurClasse(v.classe),
      sous: esc(v.heros || "?") + (v.source ? " · " + esc(String(v.source).split("(")[0].trim()) : ""),
    };
  }

  // ---------- tuiles ----------
  function pastilles(d) {
    const v = d.v, p = [];
    const nonCodees = v.non_codees || [], pb = v.problemes || [];
    if (d.bot) p.push('<span class="pill ok">réglé pour le bot</span>');
    else if (v.jouable) p.push('<span class="pill ok">jouable contre le bot</span>');
    else if (nonCodees.includes(v.heros)) p.push('<span class="pill ko">héros pas encore codé</span>');
    else if (nonCodees.length) p.push(`<span class="pill ko">${nonCodees.length} carte${nonCodees.length > 1 ? "s" : ""} pas encore jouable${nonCodees.length > 1 ? "s" : ""}</span>`);
    else if (pb.length) p.push(`<span class="pill ko">${pb.length} hors règles</span>`);
    const pool = (v.arene || 0) + (v.deck || 0);
    p.push(`<span class="pill" title="card-pool : équipement + cartes de deck">${pool} / 55</span>`);
    return p.join("");
  }
  function tuile(d) {
    const choisi = SEL.moi === d.id;
    return `<div class="dk${choisi ? " sel" : ""}" tabindex="0" role="button" aria-pressed="${choisi}" style="--c:${d.couleur}" data-deck="${esc(d.id)}">
      <button class="more" aria-label="Actions sur ${esc(d.titre)}" data-more="${esc(d.id)}">⋯</button>
      <div class="dart" data-art="${esc(d.v.heros_image || "")}"></div>
      <div class="bd"><b>${esc(d.titre)}</b><span class="sub">${d.sous}</span><div class="pills">${pastilles(d)}</div></div></div>`;
  }
  function peindreArt(racine) {
    for (const e of racine.querySelectorAll("[data-art]")) if (e.dataset.art) e.style.backgroundImage = Images.fond(e.dataset.art);
  }

  function afficherJouer() {
    const mes = MES, bot = BOT.filter((d) => !ARCH_BOT.has(d.nom));
    if (!tous().some((d) => d.id === SEL.moi && !(d.bot && ARCH_BOT.has(d.nom)))) SEL.moi = (bot[0] || mes[0] || {}).id || null;
    $("dk-g-mes").innerHTML = `<a class="dk newd${mes.length ? "" : " compacte"}" href="#nouveau-deck"><span class="plus">＋</span><span class="tx"><b>Nouveau deck</b><span class="sub">choisir un héros Young</span></span></a>` + mes.map(tuile).join("");
    $("dk-g-bot").innerHTML = bot.map(tuile).join("");
    peindreArt($("dk-vue"));
    const na = ARCH_MES.length + BOT.filter((d) => ARCH_BOT.has(d.nom)).length;
    $("dk-nb-arch").textContent = na ? `(${na})` : "";
    // Deck du bot : au hasard, un deck du bot, ou un de mes decks jouables
    const s = $("dk-s-bot");
    const jouablesMes = mes.filter((d) => d.v.jouable);
    s.innerHTML = '<option value="hasard">Au hasard</option>' + bot.map((d) => `<option value="${esc(d.id)}">${esc(d.titre)}</option>`).join("")
      + (jouablesMes.length ? `<optgroup label="Mes decks">${jouablesMes.map((d) => `<option value="${esc(d.id)}">${esc(d.titre)}</option>`).join("")}</optgroup>` : "");
    if (SEL.bot !== "hasard" && !bot.concat(jouablesMes).some((d) => d.id === SEL.bot)) SEL.bot = "hasard";
    s.value = SEL.bot;
    $("dk-s-niveau").value = SEL.niveau;
    if (!$("dk-s-niveau").value) { SEL.niveau = "b1"; $("dk-s-niveau").value = "b1"; }
    $("dk-s-manches").value = SEL.manches;
    sauverSel();
    duel();
  }
  function carteHeros(e, d) {
    e.style.setProperty("--c", d ? d.couleur : "#555");
    e.textContent = d ? d.titre : "?";
    e.style.backgroundImage = d && d.v.heros_image ? `url('${Images.url(d.v.heros_image, "normal")}')` : "";
  }
  function duel() {
    const moi = trouver(SEL.moi), bot = SEL.bot === "hasard" ? null : trouver(SEL.bot);
    carteHeros($("dk-hc-moi"), moi);
    carteHeros($("dk-hc-bot"), bot);
    const w = [];
    if (!moi) w.push("Choisis un deck.");
    else if (!moi.v.jouable) {
      const nc = moi.v.non_codees || [];
      if (nc.includes(moi.v.heros)) w.push("Le moteur ne sait pas encore jouer ce héros.");
      const autres = nc.filter((n) => n !== moi.v.heros);
      if (autres.length) w.push(autres.length > 1 ? `${autres.length} cartes de ce deck ne sont pas encore jouables par le bot : ${autres.map(esc).join(", ")}.` : `1 carte de ce deck n'est pas encore jouable par le bot : ${esc(autres[0])}.`);
      for (const p of moi.v.problemes || []) w.push(esc(p));
    }
    $("dk-avert").innerHTML = w.length ? `<div class="issues">${w.map((x) => `<div class="iss ko"><span class="ic">!</span><span>${x}</span></div>`).join("")}</div>` : "";
    $("dk-lancer").disabled = w.length > 0;
  }
  function choisir(id) {
    SEL.moi = id; sauverSel();
    for (const t of document.querySelectorAll("#dk-vue .dk.sel")) { t.classList.remove("sel"); t.setAttribute("aria-pressed", "false"); }
    const t = document.querySelector(`#dk-vue .dk[data-deck="${CSS.escape(id)}"]`);
    if (t) { t.classList.add("sel"); t.setAttribute("aria-pressed", "true"); }
    duel();
  }

  // ---------- chargement ----------
  async function charger() {
    try {
      const [m, p, c] = await Promise.all([
        fetch("/api/decks-maison").then((r) => r.json()),
        fetch("/api/decks-projet").then((r) => r.json()),
        fetch("/api/collection").then((r) => r.json()),
      ]);
      MES = m.decks.map((v) => deDeck(v, false));
      BOT = p.decks.map((v) => deDeck(v, true));
      ARCH_MES = m.archives || [];
      $("dk-dossier").title = String(m.dossier || "");
      $("dk-collection").textContent = c.source === "fabcube"
        ? `Collection Silver Age : ${c.nombre} cartes, dont ${c.codees} jouables par le bot.`
        : `Collection : ${c.codees} cartes jouables par le bot (liste complète Silver Age absente).`;
      $("dk-erreur").textContent = "";
    } catch (e) { $("dk-erreur").textContent = "Serveur injoignable : relance Jouer.bat."; }
  }

  // ---------- menu (clic droit ou ⋯) ----------
  function fermerMenu() { for (const m of document.querySelectorAll("#dk-couche .ctx")) m.remove(); }
  function ouvrirMenu(x, y, tete, items) {
    fermerMenu();
    const m = document.createElement("div");
    m.className = "ctx"; m.setAttribute("role", "menu");
    m.innerHTML = tete + items.map((it, i) => it === null ? '<div class="sep"></div>'
      : `<button role="menuitem" data-i="${i}" class="${it[3] || ""}"><span class="ico">${it[0]}</span>${esc(it[1])}</button>`).join("");
    $("dk-couche").appendChild(m);
    const r = m.getBoundingClientRect();
    m.style.left = Math.max(8, Math.min(x, innerWidth - r.width - 8)) + "px";
    m.style.top = Math.max(8, Math.min(y, innerHeight - r.height - 8)) + "px";
    m.addEventListener("click", (e) => { const b = e.target.closest("[data-i]"); if (!b) return; fermerMenu(); items[b.dataset.i][2](); });
    const f = m.querySelector("button"); if (f) f.focus();
  }
  function menuDeck(id, x, y) {
    const d = trouver(id); if (!d) return;
    const items = [
      ["▶", "Jouer avec ce deck", () => choisir(id)],
    ];
    if (d.bot || d.v.jouable) items.push(["⚔", "Le bot joue ce deck", () => { SEL.bot = id; sauverSel(); $("dk-s-bot").value = id; duel(); }]);
    items.push(
      ["✎", d.bot ? "Modifier (une copie)" : "Modifier", () => editer(d)],
      ["⧉", "Dupliquer", () => editer(d, true)],
      ["⎙", "Imprimer (proxies)", () => imprimer(d)],
      ["⇩", "Exporter la liste", () => exporter(d)],
      null,
      ["🗄", "Archiver", () => archiver(d, true)],
    );
    ouvrirMenu(x, y, `<div class="h">${esc(d.titre)}<small>${d.bot ? "deck du bot" : "mon deck"}</small></div>`, items);
  }

  // ---------- toasts et boîtes de dialogue ----------
  let minToast = null;
  function toast(msg, act) {
    for (const t of document.querySelectorAll("#dk-couche .toast")) t.remove();
    const t = document.createElement("div");
    t.className = "toast"; t.setAttribute("role", "status");
    t.innerHTML = `<span>${msg}</span>` + (act ? `<button class="btn">${esc(act[0])}</button>` : "");
    if (act) t.querySelector("button").onclick = () => { t.remove(); act[1](); };
    $("dk-couche").appendChild(t);
    clearTimeout(minToast); minToast = setTimeout(() => t.remove(), act ? 6000 : 3600);
  }
  // boutons : [libellé, action (renvoie false pour garder la fenêtre ouverte), classe]
  function dialogue(titre, corps, boutons) {
    const d = $("dk-dlg");
    $("dk-dlg-h").innerHTML = titre; $("dk-dlg-b").innerHTML = corps;
    $("dk-dlg-f").innerHTML = boutons.map(([l, , c], i) => `<button class="${c || "btn"}" data-b="${i}">${esc(l)}</button>`).join("");
    $("dk-dlg-f").onclick = async (e) => {
      const b = e.target.closest("[data-b]"); if (!b) return;
      const f = boutons[b.dataset.b][1];
      if (f && (await f()) === false) return;
      d.close();
    };
    if (!d.open) d.showModal();
  }

  // ---------- actions ----------
  async function post(url, corps) {
    const r = await fetch(url, corps === undefined ? { method: "POST" } : { method: "POST", body: corps, headers: { "Content-Type": "text/plain; charset=utf-8" } });
    const v = await r.json();
    if (!r.ok || v.erreur) throw new Error(v.erreur || "erreur " + r.status);
    return v;
  }
  async function editer(d, copie) {
    try {
      await Editeur.ouvrir(d.nom);
      Arsenal.aller("editeur");
      if (copie && !d.bot) Editeur.commeCopie();
    } catch (e) { toast("Ouverture impossible : " + esc(e.message)); }
  }
  async function texteDe(d) {
    const v = await fetch("/api/deck?nom=" + encodeURIComponent(d.nom)).then((r) => r.json());
    if (v.erreur) throw new Error(v.erreur);
    return v.texte;
  }
  async function exporter(d) {
    let t;
    try { t = await texteDe(d); } catch (e) { toast("Lecture impossible : " + esc(e.message)); return; }
    dialogue("Exporter la liste", `<p class="muted">Format des decks de fabbot (lu aussi par « Importer une liste »).</p><textarea readonly id="dk-export">${esc(t)}</textarea>`, [
      ["Fermer", null],
      ["Fichier .txt", () => { telecharger(d.nom + ".txt", t); return false; }],
      ["Copier", async () => { try { await navigator.clipboard.writeText(t); toast("Liste copiée."); } catch (e) { $("dk-export").select(); toast("Copie refusée par le navigateur : la liste est sélectionnée, Ctrl+C."); return false; } }, "btn primary"],
    ]);
  }
  function telecharger(nom, texte) {
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([texte], { type: "text/plain" }));
    a.download = nom; document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 2000);
  }
  // Ouvre Imprimer avec ce deck choisi (la liste du volet se remplit après coup)
  function imprimer(d) {
    Arsenal.aller("impression");
    let n = 0;
    const essai = () => {
      const s = $("im-deck");
      if (s && [...s.options].some((o) => o.value === d.nom)) { s.value = d.nom; s.dispatchEvent(new Event("change")); return; }
      if (++n < 40) setTimeout(essai, 50);
    };
    essai();
  }
  async function archiver(d, oui) {
    try {
      if (d.bot) { oui ? ARCH_BOT.add(d.nom) : ARCH_BOT.delete(d.nom); ecrire("arsenal-bot-archives", [...ARCH_BOT]); }
      else await post(`/api/deck-maison/${oui ? "archiver" : "restaurer"}?nom=` + encodeURIComponent(d.nom));
    } catch (e) { toast(esc(e.message)); return; }
    await rafraichir();
    toast(oui ? `« ${esc(d.titre)} » archivé.` : `« ${esc(d.titre)} » est de retour dans tes decks.`, oui ? ["Annuler", () => archiver(d, false)] : null);
  }
  function importer() {
    dialogue("Importer une liste", `<p class="muted">Colle l'export texte de FaBrary (« Copy » → « Copy deck list ») ou le format des fichiers du projet (<span class="num">Hero:</span>, <span class="num">Arena:</span>, <span class="num">Deck:</span>). Le héros est lu sur la ligne « Hero: ».</p>
      <label class="champ">Nom du deck <input id="dk-i-nom" placeholder="ex. hala-agressif" autocomplete="off" spellcheck="false"></label>
      <textarea id="dk-i-texte" spellcheck="false" placeholder="Hero: Hala&#10;&#10;Arena cards&#10;1x Zenith Blade&#10;…&#10;&#10;Deck cards&#10;2x Edict of Steel (red)&#10;…"></textarea>
      <div id="dk-i-res" class="muted petit" aria-live="polite"></div>`, [
      ["Annuler", null],
      ["Vérifier", async () => { await envoyerImport(false); return false; }],
      ["Créer le deck", async () => (await envoyerImport(true)) ? undefined : false, "btn primary"],
    ]);
    $("dk-i-texte").addEventListener("input", () => {
      const m = /^\s*name\s*:\s*(.+)$/im.exec($("dk-i-texte").value);
      if (m && !$("dk-i-nom").value) $("dk-i-nom").value = slug(m[1]);
    });
  }
  const slug = (s) => String(s).normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40);
  function bilan(v) {
    const l = [];
    l.push(v.jouable ? '<b class="ok">✓ Jouable contre le bot</b>' : '<b class="ko">✗ Pas encore jouable</b>');
    l.push(`${esc(v.heros)} · ${v.arene ?? "?"} carte(s) d'arène + ${v.deck ?? "?"} de deck`);
    for (const p of v.problemes || []) l.push("• " + esc(p));
    if ((v.non_codees || []).length) l.push(`• ${v.non_codees.length} carte(s) pas encore codée(s) pour le bot : ${v.non_codees.map(esc).join(", ")}`);
    for (const r of v.remarques || []) l.push("• " + esc(r));
    return l.join("<br>");
  }
  async function envoyerImport(creer) {
    const res = $("dk-i-res"), texte = $("dk-i-texte").value, nom = slug($("dk-i-nom").value);
    if (creer && !nom) { res.innerHTML = '<b class="ko">Donne un nom au deck.</b>'; $("dk-i-nom").focus(); return false; }
    res.textContent = "…";
    try {
      const v = await post(creer ? "/api/deck-maison/importer?nom=" + encodeURIComponent(nom) : "/api/deck-maison/verifier", texte);
      if (!creer) { res.innerHTML = bilan(v); return false; }
      await rafraichir();
      SEL.moi = "m:" + v.nom; sauverSel(); afficherJouer();
      toast(`« ${esc(v.nom)} » créé.` + (v.jouable ? " Jouable contre le bot." : ""));
      return true;
    } catch (e) { res.innerHTML = `<b class="ko">✗ ${esc(e.message)}</b>`; return false; }
  }

  // ---------- lancer ----------
  async function lancer() {
    const moi = trouver(SEL.moi); if (!moi || !moi.v.jouable) return;
    let bot = SEL.bot === "hasard" ? null : trouver(SEL.bot);
    if (!bot) { const L = BOT.filter((d) => !ARCH_BOT.has(d.nom)); bot = L[Math.floor(Math.random() * L.length)]; }
    const q = { deck: moi.nom, bot: bot.nom, niveau: SEL.niveau, manches: SEL.manches, graine: $("dk-graine").value.trim() };
    $("dk-erreur").textContent = "";
    $("dk-lancer").disabled = true;
    try { await PartieArsenal.lancer(q); } catch (e) { $("dk-erreur").textContent = e.message; }
    $("dk-lancer").disabled = false;
  }

  // ---------- archives ----------
  function afficherArchives() {
    const L = [...ARCH_MES.map((n) => ({ nom: n, titre: n, bot: false, couleur: "#c9a24a", v: {} })), ...BOT.filter((d) => ARCH_BOT.has(d.nom))];
    $("dk-l-arch").innerHTML = L.length ? L.map((d) => `<div class="arow" style="--c:${d.couleur}"><div class="a" data-art="${esc(d.v.heros_image || "")}"></div>
      <div><b>${esc(d.titre)}</b><div class="muted petit">${d.bot ? esc(d.v.heros) + " · deck du bot (ses réglages sont gardés)" : "mon deck"}</div></div>
      <div class="boutons"><button class="btn" data-desarch="${esc(d.nom)}" data-bot="${d.bot ? 1 : 0}">Désarchiver</button>${d.bot ? "" : `<button class="btn danger" data-suppr="${esc(d.nom)}">Supprimer…</button>`}</div></div>`).join("")
      : '<div class="vide">Aucun deck archivé. Archive un deck depuis Jouer (clic droit ou ⋯ sur sa tuile).</div>';
    peindreArt($("dk-l-arch"));
  }
  function supprimer(nom) {
    dialogue("Supprimer définitivement ?", `<p>« <b>${esc(nom)}</b> » sera supprimé de ton ordinateur (dossier des decks archivés). Impossible de revenir en arrière.</p>`, [
      ["Garder", null],
      ["Supprimer", async () => { try { await post("/api/deck-maison/supprimer?nom=" + encodeURIComponent(nom)); } catch (e) { toast(esc(e.message)); } await rafraichir(); toast(`« ${esc(nom)} » supprimé.`); }, "btn danger fill"],
    ]);
  }

  // ---------- nouveau deck ----------
  let filtreHeros = "tous";
  function afficherNouveau() {
    const classes = [...new Set(HEROS.flatMap((h) => h.classes || []))].sort();
    $("dk-h-filtres").innerHTML = [["tous", "Tous"], ["bot", "Jouables par le bot"], ...classes.map((c) => [c, c])].map(([v, l]) =>
      `<button class="tg" aria-pressed="${filtreHeros === v}" data-hf="${esc(v)}">${v !== "tous" && v !== "bot" ? `<span class="dot" style="background:${CLSCOL[v] || "#888"}"></span>` : ""}${esc(l)}</button>`).join("")
      + `<span class="muted compte">${HEROS.length} héros Young</span>`;
    const L = HEROS.filter((h) => filtreHeros === "tous" || (filtreHeros === "bot" ? h.codee : (h.classes || []).includes(filtreHeros)))
      .sort((a, b) => (b.codee - a.codee) || a.nom.localeCompare(b.nom));
    $("dk-h-grille").innerHTML = L.map((h) => `<button class="hpick" data-heros="${esc(h.nom)}" style="--c:${couleurClasse((h.classes || []).join(" "))}">${h.image ? `<img alt="" loading="lazy" referrerpolicy="no-referrer" src="${esc(Images.url(h.image, "normal"))}">` : ""}<span class="pill tagb ${h.codee ? "ok" : ""}">${h.codee ? "bot ✓" : "bot ✗"}</span><span class="nm">${esc(h.nom)}</span></button>`).join("");
    for (const i of $("dk-h-grille").querySelectorAll("img")) i.addEventListener("error", () => i.remove());
  }
  async function creer(heros) {
    try { await Editeur.nouveau(heros); Arsenal.aller("editeur"); } catch (e) { toast("Création impossible : " + esc(e.message)); }
  }

  async function rafraichir() {
    await charger();
    afficherJouer();
    if (!$("dk-archives").hidden) afficherArchives();
  }

  // ---------- écouteurs ----------
  sec.addEventListener("click", (e) => {
    const m = e.target.closest("[data-more]");
    if (m) { e.stopPropagation(); const r = m.getBoundingClientRect(); menuDeck(m.dataset.more, r.left, r.bottom + 4); return; }
    const t = e.target.closest("#dk-vue .dk[data-deck]"); if (t) { choisir(t.dataset.deck); return; }
    const u = e.target.closest("[data-desarch]");
    if (u) { const bot = u.dataset.bot === "1"; archiver(bot ? trouver("b:" + u.dataset.desarch) : { nom: u.dataset.desarch, titre: u.dataset.desarch, bot: false }, false); return; }
    const x = e.target.closest("[data-suppr]"); if (x) { supprimer(x.dataset.suppr); return; }
    const f = e.target.closest("[data-hf]"); if (f) { filtreHeros = f.dataset.hf; afficherNouveau(); return; }
    const h = e.target.closest("[data-heros]"); if (h) creer(h.dataset.heros);
  });
  sec.addEventListener("dblclick", (e) => { const t = e.target.closest("#dk-vue .dk[data-deck]"); if (t && !e.target.closest("[data-more]")) { const d = trouver(t.dataset.deck); if (d) editer(d); } });
  sec.addEventListener("contextmenu", (e) => { const t = e.target.closest("#dk-vue .dk[data-deck]"); if (t) { e.preventDefault(); menuDeck(t.dataset.deck, e.clientX, e.clientY); } });
  document.addEventListener("mousedown", (e) => { if (!e.target.closest("#dk-couche .ctx")) fermerMenu(); });
  document.addEventListener("scroll", fermerMenu, true);
  document.addEventListener("keydown", (e) => {
    if (Arsenal.vue() !== "decks") return;
    const ae = document.activeElement;
    const m = document.querySelector("#dk-couche .ctx");
    if (e.key === "Escape" && m) { e.preventDefault(); fermerMenu(); return; }
    if (m && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
      e.preventDefault();
      const bs = [...m.querySelectorAll("button")], i = bs.indexOf(ae);
      bs[(i + (e.key === "ArrowDown" ? 1 : -1) + bs.length) % bs.length].focus();
      return;
    }
    if (ae && ae.matches && ae.matches("#dk-vue .dk[data-deck]")) {
      if (e.key === "Enter" || e.key === " ") { e.preventDefault(); choisir(ae.dataset.deck); }
      else if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) { e.preventDefault(); const r = ae.getBoundingClientRect(); menuDeck(ae.dataset.deck, r.left + 20, r.top + 20); }
    }
  });
  $("dk-s-bot").addEventListener("change", (e) => { SEL.bot = e.target.value; sauverSel(); duel(); });
  $("dk-s-niveau").addEventListener("change", (e) => { SEL.niveau = e.target.value; sauverSel(); });
  $("dk-s-manches").addEventListener("change", (e) => { SEL.manches = e.target.value; sauverSel(); });
  $("dk-lancer").addEventListener("click", lancer);
  $("dk-charger").addEventListener("click", () => PartieArsenal.charger());
  $("dk-b-importer").addEventListener("click", importer);
  $("dk-b-imprimer").addEventListener("click", (e) => {
    const r = e.currentTarget.getBoundingClientRect();
    ouvrirMenu(r.left, r.bottom + 4, '<div class="h">Imprimer quel deck ?</div>', tous().filter((d) => !(d.bot && ARCH_BOT.has(d.nom))).map((d) => ["⎙", d.titre, () => imprimer(d)]));
  });

  // Quel écran du volet : Jouer, archives, nouveau deck ou éditeur
  document.addEventListener("arsenal:vue", async (ev) => {
    if (ev.detail.vue !== "decks") { fermerMenu(); return; }
    const a = ev.detail.adresse;
    const vueEd = $("ed-vue");
    if (a === "editeur" && !vueEd) { Arsenal.aller("decks"); return; } // rechargement de la page : rien d'ouvert
    $("dk-vue").hidden = a !== "decks";
    $("dk-archives").hidden = a !== "archives";
    $("dk-nouveau").hidden = a !== "nouveau-deck";
    if (vueEd) vueEd.hidden = a !== "editeur";
    if (a === "decks") { await charger(); afficherJouer(); }
    else if (a === "archives") { await charger(); afficherArchives(); }
    else if (a === "nouveau-deck") {
      try { HEROS = await Editeur.herosJeunes(); } catch (e) { HEROS = []; }
      afficherNouveau();
    }
  });
})();
