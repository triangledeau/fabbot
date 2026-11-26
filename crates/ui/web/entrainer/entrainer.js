// Volet « Entraîner » (adresse #entrainer ; lot 8 du plan de rapatriement).
// Tournoi des bots lancé dans la page (matrice qui se remplit, score, Elo,
// classement des decks), banc SPRT, historique et export ; lancement des
// apprentissages de fab-sim (table de side, poids de B3) avec leur sortie.
// Tout le calcul est fait par le serveur (api_tournoi.rs) : la page affiche.
"use strict";
(() => {
  const sec = document.getElementById("v-entrainer");
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const $ = (id) => document.getElementById(id);
  const pct = (s) => (s == null ? "–" : Math.round(100 * s) + " %");
  // Au-delà de ±800, un score de 0 ou 100 % : l'écart n'est plus mesurable.
  const elo = (e) => (e == null ? "–" : e >= 800 ? "≥ +800" : e <= -800 ? "≤ −800" : (e >= 0 ? "+" : "") + Math.round(e));
  const duree = (s) => { s = Math.round(s); return s < 60 ? s + " s" : s < 3600 ? Math.floor(s / 60) + " min " + (s % 60) + " s" : Math.floor(s / 3600) + " h " + Math.floor((s % 3600) / 60) + " min"; };
  const date = (t) => new Date(t * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
  // Niveaux des bots en mots de joueur (docs/bots.md, « Les bots »).
  const NIVEAUX = { b0: "B0 · joue au hasard", b1: "B1 · règles fixes", b2: "B2 · cherche le meilleur coup", b3: "B3 · cherche, avec évaluation apprise" };
  const opts = (liste, choisi) => liste.map((b) => `<option value="${esc(b)}"${b === choisi ? " selected" : ""}>${esc(NIVEAUX[b] || b.toUpperCase())}</option>`).join("");
  const SIDES = [["appris", "plan appris (conseillé)"], ["liste", "liste telle quelle"], ["general", "plan général"], ["hasard", "au hasard"]];
  // Parties jouées en même temps : par défaut, tous les cœurs de l'ordinateur sauf un.
  const FILS = Math.max(1, (navigator.hardwareConcurrency || 4) - 1);
  let noms = {}; // nom court du deck → nom du héros (/api/decks)
  let portraits = {}; // nom court d'un deck du bot → image du héros (/api/decks-projet)
  const ava = (n) => portraits[n] ? `<i class="en-ava" style="--art:${esc(Images.fond(portraits[n]))}"></i>` : "";
  const nomDeck = (n) => noms[n] || n;
  const court = (n) => nomDeck(n).split(",")[0];
  const sides = (choisi) => SIDES.map(([v, l]) => `<option value="${v}"${v === choisi ? " selected" : ""}>${l}</option>`).join("");

  sec.innerHTML = `<header class="dk-tete">
      <a class="btn small home-btn" href="#accueil">${Arsenal.HOME_SVG}Accueil</a>
      <a class="btn small" id="en-retour" href="#entrainer" hidden>← Entraîner</a>
      <h1 id="en-titre">Entraîner</h1>
      <span class="muted" id="en-sous-titre">Faire jouer les bots entre eux et mesurer leur force</span>
    </header>
    <nav class="tiles3 en-hub" id="en-hub" aria-label="Entraîner">
      <a class="ht" style="--c:#ffd84a" href="#tournoi">
        <div class="pic"><div class="art" id="en-art-1"></div><div class="art" id="en-art-2" hidden></div><span class="vs" id="en-vs" hidden>VS</span><span class="badge">TOURNOI</span></div>
        <div class="txt"><h2>Tournoi des bots</h2><p>Chaque deck affronte tous les autres. Matrice des victoires, Elo et classement en direct ; historique des tournois.</p></div>
        <div class="foot"><div class="row"><span class="muted">Dernier tournoi</span><span id="en-hub-tournoi">…</span></div>
          <div class="pbar"><i id="en-hub-barre"></i></div></div>
      </a>
      <a class="ht" style="--c:#63c98b" href="#apprentissage-bots">
        <div class="pic"><div class="art" id="en-art-3"></div><span class="badge">APPRENTISSAGE</span></div>
        <div class="txt"><h2>Faire apprendre les bots</h2><p>Parties en série entre bots pour apprendre les plans de side et l'évaluation de B3.</p></div>
        <div class="foot"><div class="row"><span class="muted">Calcul</span><span id="en-hub-tache">sur cet ordinateur</span></div></div>
      </a>
      <div class="ht" style="--c:#9a6bd1" aria-disabled="true">
        <div class="pic"><div class="art"></div><span class="badge">BIENTÔT</span></div>
        <div class="txt"><h2>Entraîner le bot sur un deck</h2><p>Régler le bot sur un deck créé ou modifié dans l'éditeur, en arrière-plan.</p></div>
        <div class="foot"><div class="row"><span class="muted">État</span><span>pas encore disponible</span></div></div>
      </div>
    </nav>
    <div id="en-v-tournoi">
    <div class="en-grille">
      <section class="en-bloc" aria-labelledby="en-t1">
        <h2 id="en-t1">Tournoi</h2>
        <div class="en-form">
          <fieldset><legend>Bot A</legend>
            <label>Niveau <select id="en-bot-a"></select></label>
            <label>Side <select id="en-side-a">${sides("appris")}</select></label></fieldset>
          <fieldset><legend>Bot B</legend>
            <label>Niveau <select id="en-bot-b"></select></label>
            <label>Side <select id="en-side-b">${sides("appris")}</select></label></fieldset>
          <fieldset><legend>Parties</legend>
            <label title="Parties par paire de decks (A avec le deck de la ligne, B avec celui de la colonne)">Par paire de decks <input type="number" id="en-parties" min="1" max="100000" value="10"></label></fieldset>
        </div>
        <details class="en-avance"><summary>Réglages avancés</summary>
          <div class="en-form">
            <fieldset><legend>Calcul</legend>
              <label title="Nombre de parties jouées en même temps. Par défaut : les cœurs de l'ordinateur moins un.">Parties en même temps <input type="number" id="en-threads" min="1" max="64" value="${FILS}"></label>
              <label title="Même graine et mêmes réglages = exactement les mêmes parties (pour comparer deux versions)">Graine <input type="number" id="en-graine" min="1" value="1"></label></fieldset>
            <fieldset><legend>Arrêt anticipé (test SPRT)</legend>
              <label title="Le tournoi s'arrête dès que le test tranche entre « A n'a pas d'avance » et « A a au moins cette avance »"><input type="checkbox" id="en-sprt"> s'arrêter dès que l'écart est prouvé</label>
              <label>Écart recherché <input type="number" id="en-sprt-elo" min="1" max="400" value="30"> points Elo</label></fieldset>
          </div>
        </details>
        <div class="en-decks"><div class="en-decks-tete"><b>Decks</b>
          <button class="btn small quiet" id="en-tous">Tous</button><button class="btn small quiet" id="en-aucun">Aucun</button>
          <span class="muted" id="en-nb"></span></div><div id="en-liste-decks" class="en-coches"></div></div>
        <div class="en-actions">
          <button class="btn primary" id="en-lancer">Lancer le tournoi</button>
          <button class="btn" id="en-arreter" hidden>Arrêter</button>
          <span id="en-msg" class="muted"></span>
        </div>
      </section>
      <section class="en-bloc" aria-labelledby="en-t2">
        <h2 id="en-t2">Historique</h2>
        <div id="en-historique" class="en-hist"><p class="muted">…</p></div>
      </section>
    </div>
    <section class="en-bloc en-resultat" id="en-resultat" hidden aria-live="polite"></section>
    </div>
    <section class="en-bloc" id="en-v-appr" aria-labelledby="en-t3">
      <h2 id="en-t3">Apprentissage</h2>
      <p class="muted petit">Les bots rejouent beaucoup de parties entre eux pour apprendre : soit leurs plans de side (quelles cartes mettre dans les 40 contre chaque héros), soit l'évaluation des positions de B3. Compter plusieurs heures. Le résultat ne sert qu'au prochain lancement du jeu après une mise à jour.</p>
      <p class="muted petit" title="fab-sim --apprendre-side écrit data/side/b1.txt ; fab-sim --apprendre-b3 écrit data/b3/evaluation.txt. Un seul agent à la fois doit refaire data/side/b1.txt (docs/taches.md).">Fichiers modifiés : survolez ici pour le détail technique.</p>
      <div class="en-actions">
        <label>Parties par paire de decks <input type="number" id="en-t-parties" min="1" value="200"></label>
        <label title="Nombre de parties jouées en même temps">Parties en même temps <input type="number" id="en-t-threads" min="1" max="64" value="${FILS}"></label>
        <button class="btn" id="en-t-side">Apprendre les plans de side</button>
        <button class="btn" id="en-t-b3">Apprendre l'évaluation de B3</button>
        <button class="btn" id="en-t-arreter" hidden>Arrêter</button>
        <span id="en-t-msg" class="muted"></span>
      </div>
      <pre id="en-t-journal" class="en-journal" hidden></pre>
    </section>`;

  let dernier = null; // dernier état affiché (pour l'export)
  let minuterie = null;

  async function remplir() {
    const d = await fetch("/api/decks").then((r) => r.json());
    noms = d.noms || {};
    const ba = $("en-bot-a"), bb = $("en-bot-b");
    if (!ba.options.length) {
      ba.innerHTML = opts(d.niveaux, "b1");
      bb.innerHTML = opts(d.niveaux, "b0");
    }
    const coches = new Set([...document.querySelectorAll("#en-liste-decks input:checked")].map((i) => i.value));
    const premier = !document.querySelector("#en-liste-decks input");
    const ligne = (n, maison) => `<label class="en-puce${maison ? " maison" : ""}"><input type="checkbox" value="${esc(n)}"${(premier ? !maison : coches.has(n)) ? " checked" : ""}><span>${ava(n)}${esc(nomDeck(n))}${maison ? " <small>maison</small>" : ""}</span></label>`;
    $("en-liste-decks").innerHTML = d.decks.map((n) => ligne(n, false)).join("") + (d.decks_maison || []).map((n) => ligne(n, true)).join("");
    compter();
  }
  const choisis = () => [...document.querySelectorAll("#en-liste-decks input:checked")].map((i) => i.value);
  // Durée arrondie, pour une estimation : « environ 18 min », « environ 1 h 20 », « moins d'une minute ».
  const environ = (sec) => {
    const m = Math.round(sec / 60);
    if (m < 1) return "moins d'une minute";
    if (m < 60) return "environ " + m + " min";
    const h = Math.floor(m / 60), r = Math.round((m % 60) / 5) * 5;
    return "environ " + h + " h" + (r ? " " + String(r).padStart(2, "0") : "");
  };
  function compter() {
    const n = choisis().length, p = +$("en-parties").value || 0;
    let txt = n ? `${n} deck${n > 1 ? "s" : ""} : ${n * n} paires, ${n * n * p} parties au plus` : "aucun deck";
    const v = vitesse[[$("en-bot-a").value, $("en-bot-b").value].join("/")];
    if (n && v) txt += `, ${environ((n * n * p) / v)}`;
    else if (n) txt += " (durée estimée après un premier tournoi de ces bots)";
    $("en-nb").textContent = txt;
  }
  // Parties par seconde du dernier tournoi fini pour chaque paire de bots (historique).
  let vitesse = {};
  $("en-liste-decks").addEventListener("change", compter);
  $("en-parties").addEventListener("input", compter);
  for (const id of ["en-bot-a", "en-bot-b", "en-threads"]) $(id).addEventListener("change", compter);
  $("en-tous").onclick = () => { document.querySelectorAll("#en-liste-decks input").forEach((i) => (i.checked = true)); compter(); };
  $("en-aucun").onclick = () => { document.querySelectorAll("#en-liste-decks input").forEach((i) => (i.checked = false)); compter(); };

  $("en-lancer").onclick = async () => {
    const q = new URLSearchParams({
      decks: choisis().join(","),
      bot_a: $("en-bot-a").value, bot_b: $("en-bot-b").value,
      side_a: $("en-side-a").value, side_b: $("en-side-b").value,
      parties: $("en-parties").value, threads: $("en-threads").value, graine: $("en-graine").value,
    });
    if ($("en-sprt").checked) q.set("sprt", $("en-sprt-elo").value);
    const r = await fetch("/api/tournoi/lancer?" + q, { method: "POST" });
    const v = await r.json();
    $("en-msg").textContent = r.ok ? "" : v.erreur;
    if (r.ok) { afficher(v); suivre(); }
  };
  $("en-arreter").onclick = () => fetch("/api/tournoi/arreter", { method: "POST" });

  function suivre() {
    clearInterval(minuterie);
    minuterie = setInterval(async () => {
      if (document.documentElement.dataset.vue !== "entrainer") { clearInterval(minuterie); return; }
      const v = await fetch("/api/tournoi").then((r) => r.json());
      afficher(v);
      if (!v.en_cours) { clearInterval(minuterie); historique(); }
    }, 700);
  }

  // Couleur d'une case : rouge (A perd) → neutre → vert (A gagne).
  const teinte = (s) => (s == null ? "" : `style="--s:${s.toFixed(3)}"`);
  function cellule(c, titre) {
    if (!c || c.score == null) return `<td class="vide" title="${esc(titre)}">·</td>`;
    const n = c.v + c.d + c.n;
    return `<td class="sc" ${teinte(c.score)} title="${esc(titre)} : ${c.v} V, ${c.d} D, ${c.n} N${c.e ? ", " + c.e + " erreurs" : ""} (${n} parties)">${Math.round(100 * c.score)}</td>`;
  }

  function afficher(v, depuisHistorique) {
    if (!v || v.aucun) return;
    dernier = v;
    const r = v.reglage, t = v.total_a, decks = r.decks;
    const [A, B] = r.bots.map((b) => b.toUpperCase());
    $("en-lancer").hidden = !!v.en_cours;
    $("en-arreter").hidden = !v.en_cours;
    const part = v.total ? v.fait / v.total : 0;
    const reste = v.en_cours && v.fait ? (v.secondes / v.fait) * (v.total - v.fait) : 0;
    let sprt = "";
    if (v.sprt) {
      const x = Math.max(-1, Math.min(1, v.sprt.llr / v.sprt.borne));
      const verdict = v.sprt.conclusion === "h1" ? `${A} a au moins +${v.sprt.elo1} Elo d'avance` : v.sprt.conclusion === "h0" ? `${A} n'a pas +${v.sprt.elo1} Elo d'avance` : "pas encore tranché";
      sprt = `<div class="en-sprt"><span>SPRT (${elo(v.sprt.elo0)} contre ${elo(v.sprt.elo1)} Elo, 5 % d'erreur) : <b>${verdict}</b></span>
        <div class="en-llr" title="LLR ${v.sprt.llr.toFixed(2)} (bornes ±${v.sprt.borne.toFixed(2)})"><i style="--x:${x}"></i></div>
        <span class="num muted">LLR ${v.sprt.llr.toFixed(2)}</span></div>`;
    }
    const tete = decks.map((d, j) => `<th title="${esc(r.heros[j])}">${esc(court(d).slice(0, 9))}</th>`).join("");
    const lignes = decks.map((d, i) => `<tr><th title="${esc(r.heros[i])}">${esc(court(d))}</th>${v.matrice[i].map((c, j) => cellule(c, `${A} avec ${d} contre ${B} avec ${decks[j]}`)).join("")}
      ${cellule(v.lignes[i], `${A} avec ${d}, moyenne`)}<td class="num muted">${elo(v.elo_ligne[i])}</td></tr>`).join("");
    const bas = `<tr class="moy"><th>moy.</th>${v.colonnes.map((c, j) => cellule(c, `contre ${B} avec ${decks[j]}, moyenne`)).join("")}<td></td><td></td></tr>`;
    const memeBot = r.bots[0] === r.bots[1] && r.sides[0] === r.sides[1];
    const classement = v.classement.map((c, k) => `<tr><td class="num">${k + 1}</td><td>${esc(c.deck)}</td><td class="muted">${esc(c.heros)}</td><td class="num">${elo(c.elo)}</td></tr>`).join("");
    $("en-resultat").hidden = false;
    $("en-resultat").innerHTML = `<div class="en-res-tete"><h2>${depuisHistorique ? "Tournoi du " + date(v.date) : v.en_cours ? "Tournoi en cours" : "Dernier tournoi"}</h2>
        <span class="muted">${A} (side ${esc(r.sides[0])}) contre ${B} (side ${esc(r.sides[1])}), ${decks.length} decks, ${r.parties} parties par paire, graine ${r.graine}</span>
        <span class="en-export"><button class="btn small quiet" id="en-json">Exporter (JSON)</button><button class="btn small quiet" id="en-csv">Matrice (CSV)</button></span></div>
      <div class="en-prog"><div class="barre"><i style="width:${(100 * part).toFixed(1)}%"></i></div>
        <span class="num">${v.fait} / ${v.total}</span><span class="muted">${duree(v.secondes)}, ${v.parties_par_s.toFixed(1)} parties/s${reste ? ", encore ≈ " + duree(reste) : ""}${v.arret ? " · " + esc(v.arret) : ""}</span></div>
      <div class="en-chiffres">
        <div><small>Score de ${A}</small><b class="num">${pct(t.score)}</b></div>
        <div><small>Victoires · défaites · nulles</small><b class="num">${t.v} · ${t.d} · ${t.n}</b></div>
        <div><small>Écart d'Elo (95 %)</small><b class="num">${elo(t.elo)}</b><span class="muted num">${t.elo == null ? "" : elo(t.elo_bas) + " à " + elo(t.elo_haut)}</span></div>
        <div><small>Tours par partie</small><b class="num">${t.tours_moyens.toFixed(1)}</b></div>
        ${t.e ? `<div class="mauvais"><small>Erreurs</small><b class="num">${t.e}</b></div>` : ""}
      </div>
      ${sprt}
      <div class="en-deux">
        <div class="en-matrice-cadre"><p class="muted petit">Score de ${A} (%) : ligne = son deck, colonne = deck de ${B}.</p>
          <table class="en-matrice"><thead><tr><th></th>${tete}<th>moy.</th><th>Elo</th></tr></thead><tbody>${lignes}${bas}</tbody></table></div>
        <div><p class="muted petit">Classement des decks (Elo, moyenne = 0)${memeBot ? "" : " : A et B n'étant pas le même bot, il mêle force du deck et du bot"}.</p>
          <table class="en-classement"><tbody>${classement}</tbody></table></div>
      </div>
      ${v.messages.length ? `<ul class="en-messages">${v.messages.map((m) => `<li>${esc(m)}</li>`).join("")}</ul>` : ""}`;
    $("en-json").onclick = () => telecharger(`tournoi-${v.date}.json`, JSON.stringify(v, null, 2), "application/json");
    $("en-csv").onclick = () => telecharger(`tournoi-${v.date}.csv`, csv(v), "text/csv");
  }

  function csv(v) {
    const d = v.reglage.decks;
    const l = [["A \\ B", ...d, "moyenne"].join(";")];
    d.forEach((x, i) => l.push([x, ...v.matrice[i].map((c) => (c.score == null ? "" : (100 * c.score).toFixed(1))), v.lignes[i].score == null ? "" : (100 * v.lignes[i].score).toFixed(1)].join(";")));
    return l.join("\n") + "\n";
  }
  function telecharger(nom, texte, type) {
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([texte], { type }));
    a.download = nom;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  }

  async function historique() {
    const h = await fetch("/api/tournois").then((r) => r.json());
    vitesse = {};
    for (const t of h.tournois) { // du plus récent au plus ancien : on garde le plus récent
      const k = (t.reglage.bots || []).join("/");
      if (!(k in vitesse) && t.parties_par_s > 0) vitesse[k] = t.parties_par_s;
    }
    compter();
    const div = $("en-historique");
    if (!h.tournois.length) { div.innerHTML = `<p class="muted" title="Dossier : ${esc(h.dossier)}">Aucun tournoi fini pour l'instant. Chaque tournoi fini s'ajoute ici.</p>`; return; }
    div.innerHTML = `<table><tbody>${h.tournois.map((t) => `<tr><td class="num">${date(t.date)}</td>
        <td>${esc(t.reglage.bots.join(" / ").toUpperCase())}</td><td class="muted">${t.reglage.decks.length} decks, ${t.fait} parties</td>
        <td class="num">${pct(t.total_a.score)}</td><td class="num">${elo(t.total_a.elo)}</td>
        <td><button class="btn small" data-voir="${esc(t.nom)}">Voir</button><button class="btn small quiet" data-oublier="${esc(t.nom)}" title="Retirer de l'historique">✕</button></td></tr>`).join("")}</tbody></table>`;
  }
  $("en-historique").addEventListener("click", async (ev) => {
    const b = ev.target.closest("button");
    if (!b) return;
    if (b.dataset.voir) {
      const v = await fetch("/api/tournoi/historique?nom=" + encodeURIComponent(b.dataset.voir)).then((r) => r.json());
      afficher(v, true);
      $("en-lancer").hidden = false;
      $("en-arreter").hidden = true;
      $("en-resultat").scrollIntoView({ behavior: "smooth" });
    } else if (b.dataset.oublier) {
      await fetch("/api/tournoi/oublier?nom=" + encodeURIComponent(b.dataset.oublier), { method: "POST" });
      historique();
    }
  });

  // ---- tâches d'apprentissage
  let minuterieTache = null;
  async function lancerTache(quoi) {
    const fichier = quoi === "side" ? "data/side/b1.txt" : "data/b3/evaluation.txt";
    if (!confirm(`Cette tâche réécrit ${fichier} à la fin. Continuer ?`)) return;
    const q = new URLSearchParams({ quoi, parties: $("en-t-parties").value, threads: $("en-t-threads").value });
    const r = await fetch("/api/tache/lancer?" + q, { method: "POST" });
    const v = await r.json();
    if (!r.ok) { $("en-t-msg").textContent = v.erreur; return; }
    afficherTache(v);
    suivreTache();
  }
  $("en-t-side").onclick = () => lancerTache("side");
  $("en-t-b3").onclick = () => lancerTache("b3");
  $("en-t-arreter").onclick = () => fetch("/api/tache/arreter", { method: "POST" });
  function afficherTache(v) {
    if (v.aucune) {
      $("en-t-msg").textContent = v.fab_sim ? "" : "Pas encore disponible sur cet ordinateur : l'outil d'apprentissage n'est pas préparé. Demandez à Claude de le compiler (commande « cargo build --release -p fab-sim »).";
      return;
    }
    $("en-t-arreter").hidden = !v.en_cours;
    $("en-t-side").disabled = $("en-t-b3").disabled = v.en_cours;
    $("en-t-msg").textContent = `${v.commande} : ${v.en_cours ? "en cours depuis " + duree(v.secondes) : v.code === 0 ? "terminé" : "arrêté (code " + (v.code ?? "–") + ")"}`;
    const j = $("en-t-journal");
    j.hidden = false;
    j.textContent = v.journal.join("\n") || "…";
    j.scrollTop = j.scrollHeight;
  }
  function suivreTache() {
    clearInterval(minuterieTache);
    minuterieTache = setInterval(async () => {
      if (document.documentElement.dataset.vue !== "entrainer") { clearInterval(minuterieTache); return; }
      const v = await fetch("/api/tache").then((r) => r.json());
      afficherTache(v);
      if (!v.en_cours) clearInterval(minuterieTache);
    }, 1000);
  }

  // Sous-pages, comme dans l'Arsenal JS : #entrainer (trois volets),
  // #tournoi, #apprentissage-bots.
  const SOUS = {
    "entrainer": ["Entraîner", "Faire jouer les bots entre eux et mesurer leur force"],
    "tournoi": ["Tournoi des bots", "Chaque deck affronte tous les autres"],
    "apprentissage-bots": ["Faire apprendre les bots", "Plans de side et évaluation de B3"],
  };
  function sousPage(a) {
    if (!SOUS[a]) a = "entrainer";
    $("en-hub").hidden = a !== "entrainer";
    $("en-v-tournoi").hidden = a !== "tournoi";
    $("en-v-appr").hidden = a !== "apprentissage-bots";
    $("en-retour").hidden = a === "entrainer";
    [$("en-titre").textContent, $("en-sous-titre").textContent] = SOUS[a];
  }
  async function chargerPortraits() {
    if (Object.keys(portraits).length) return;
    try {
      const p = await fetch("/api/decks-projet").then((r) => r.json());
      for (const d of p.decks || []) if (d.heros_image) portraits[d.nom] = d.heros_image;
    } catch (e) { /* sans portraits */ }
    const poser = (id, n) => { const el = $(id); if (!el || !portraits[n]) return; el.style.setProperty("--art", Images.fond(portraits[n])); el.classList.add("illu"); el.hidden = false; };
    const pris = (...l) => l.find((n) => portraits[n]) || Object.keys(portraits)[0];
    const g = pris("grindstone", "bravo"), d = pris("iyslander", "kano");
    poser("en-art-1", g);
    if (d && d !== g) { poser("en-art-2", d); $("en-art-1").classList.add("split1"); $("en-art-2").classList.add("split2"); $("en-vs").hidden = false; }
    poser("en-art-3", pris("dromai", "ira"));
  }
  function resumeHub(v, h) {
    const t = $("en-hub-tournoi"), barre = $("en-hub-barre");
    if (v && v.en_cours) { t.innerHTML = `en cours, <span class="num">${v.fait} / ${v.total}</span>`; barre.style.width = (v.total ? (100 * v.fait) / v.total : 0).toFixed(1) + "%"; return; }
    const d = h && (h.tournois || [])[0];
    if (!d) { t.textContent = "aucun"; barre.style.width = "0"; return; }
    t.innerHTML = `${esc(new Date(d.date * 1000).toLocaleDateString("fr-FR", { day: "numeric", month: "short" }))}, <span class="num">${d.fait}</span> parties${d.arret ? " (arrêté)" : ""}`;
    barre.style.width = "100%";
  }

  document.addEventListener("arsenal:vue", async (ev) => {
    if (ev.detail.vue !== "entrainer") return;
    sousPage(ev.detail.adresse);
    try {
      await chargerPortraits();
      if (ev.detail.adresse === "entrainer") {
        const [v, h, t] = await Promise.all(["/api/tournoi", "/api/tournois", "/api/tache"].map((u) => fetch(u).then((r) => r.json())));
        resumeHub(v, h);
        if (t && t.en_cours) $("en-hub-tache").textContent = "apprentissage en cours";
      }
    } catch (e) { /* le volet reste affiché sans résumé */ }
    try {
      await remplir();
      historique();
      const v = await fetch("/api/tournoi").then((r) => r.json());
      if (!v.aucun) { afficher(v); if (v.en_cours) suivre(); }
      const t = await fetch("/api/tache").then((r) => r.json());
      afficherTache(t);
      if (t.en_cours) suivreTache();
    } catch (e) { $("en-msg").textContent = "Serveur injoignable."; }
  });
})();
