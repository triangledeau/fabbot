// Arsenal sur Android : adaptation de la page PC au téléphone.
//
// Servi par l'appli à /android/mobile.js et ajouté à la fin du <head> de
// la page (crates/android/src/serveur.rs). Règle : toute adaptation passe
// par ce fichier et mobile.css, jamais par crates/ui/web/.
//
// Les sélecteurs de la page PC utilisés ici (querySelector, closest,
// getElementById…) sont vérifiés par crates/android/tests/selecteurs.rs.
// Les classes créées ici commencent par « android ».
//
// Jalon 3 (docs/android.md, maquette docs/android/maquette-jalon-3.html) :
// - partie : barre du haut (menu, score, journal), colonne de droite du PC
//   rangée dans un tiroir, appui long sur une carte = fiche de la carte,
//   permanents identiques regroupés (×N) ;
// - Jouer : visage du héros centré sur les yeux, symbole « réglé pour le bot ».
"use strict";

(function () {
  // Marque la page : mobile.css ne s'applique qu'à html.android, la page
  // PC reste donc inchangée hors de l'appli.
  const html = document.documentElement;
  html.classList.add("android");

  const SVG = {
    menu: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h16"/></svg>',
    annuler: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 14 4 9l5-5"/><path d="M4 9h11a5 5 0 0 1 0 10h-3"/></svg>',
    journal: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 3h11a2 2 0 0 1 2 2v16H8a2 2 0 0 1-2-2z"/><path d="M6 19a2 2 0 0 1 2-2h11M10 8h6M10 12h6"/></svg>',
  };
  // Tête de robot verte, cochée : « réglé pour le bot ».
  const ROBOT = '<svg class="android-robot" viewBox="0 0 32 32" role="img" aria-label="Réglé pour le bot"><title>Réglé pour le bot</title>'
    + '<path d="M16 3v4" stroke="#63c98b" stroke-width="2" stroke-linecap="round"/><circle cx="16" cy="3" r="1.8" fill="#63c98b"/>'
    + '<rect x="5" y="8" width="20" height="16" rx="5" fill="none" stroke="#63c98b" stroke-width="2"/>'
    + '<circle cx="11.5" cy="15" r="2.2" fill="#63c98b"/><circle cx="18.5" cy="15" r="2.2" fill="#63c98b"/>'
    + '<path d="M11 20h8M2.5 13v5" stroke="#63c98b" stroke-width="2" stroke-linecap="round"/>'
    + '<circle cx="25" cy="24" r="6" fill="#63c98b"/><path d="M22.2 24.2l2 2 3.8-4" fill="none" stroke="#0a0d13" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>';

  // Point entre les yeux de chaque héros, en fraction de la carte (image
  // « large » de LSS, 546 × 763), noté à la main le 2026-11-28. Un héros
  // absent de la table (deck maison) prend le point par défaut.
  const YEUX = {
    FR_IRA001: [0.50, 0.205], FR_MPW004: [0.437, 0.225], FR_SDO001: [0.55, 0.212],
    FR_SBR001: [0.436, 0.295], FR_SIY001: [0.543, 0.199], FR_ROS002: [0.436, 0.234],
    FR_IAR002: [0.50, 0.225], UPR002: [0.50, 0.245], FR_SAR001: [0.38, 0.238],
    FR_ROS008: [0.565, 0.244],
  };
  const YEUX_DEFAUT = [0.5, 0.22];

  // ---------- Jouer : tuiles de decks ----------
  // Carte affichée à 420 points de large ; les yeux à 60 points du bord
  // gauche, au milieu de la hauteur de la tuile (50 points).
  function tuilesDeDecks(racine) {
    for (const t of racine.querySelectorAll(".dk")) {
      if (t.dataset.android) continue;
      t.dataset.android = "1";
      const art = t.querySelector(".dart");
      if (art) {
        const [x, y] = YEUX[art.dataset.art] || YEUX_DEFAUT;
        const L = 420, H = L * 763 / 546;
        art.style.setProperty("--android-x", (60 - x * L).toFixed(1) + "px");
        art.style.setProperty("--android-y", (25 - y * H).toFixed(1) + "px");
      }
      // Sous-titre « Héros · Source » : le nom du héros est déjà sur la
      // tuile, on garde la source (elle tient alors sur la ligne).
      const sous = t.querySelector(".sub");
      if (sous && !sous.children.length && sous.textContent.includes(" · ")) {
        sous.title = sous.textContent;
        sous.textContent = sous.textContent.slice(sous.textContent.indexOf(" · ") + 3);
      }
      const pastilles = [...t.querySelectorAll(".pills .pill")];
      if (pastilles.some((p) => p.classList.contains("ok") && /réglé pour le bot/.test(p.textContent))) {
        t.classList.add("android-bot");
        const plus = t.querySelector(".more");
        if (plus) plus.insertAdjacentHTML("beforebegin", ROBOT);
      }
      // Card-pool complet : rien à signaler ; incomplet : le compte reste.
      for (const p of pastilles) if (/^\s*(\d+)\s*\/\s*\1\s*$/.test(p.textContent)) p.classList.add("android-plein");
    }
  }

  // ---------- Partie ----------
  let barre = null;
  function creerBarre() {
    const jeu = document.querySelector("#v-partie main.jeu");
    if (!jeu || barre) return;
    barre = document.createElement("div");
    barre.className = "android-barre";
    barre.innerHTML = `<button type="button" class="android-b-menu" aria-label="Menu">${SVG.menu}</button>`
      + '<div class="android-titre"><b class="android-score"></b><span class="android-manche"></span></div>'
      + `<button type="button" class="android-b-annuler" aria-label="Annuler mon dernier choix">${SVG.annuler}</button>`
      + `<button type="button" class="android-b-journal" aria-label="Journal">${SVG.journal}</button>`;
    jeu.prepend(barre);
    barre.querySelector(".android-b-menu").addEventListener("click", () => ouvrir("menu"));
    barre.querySelector(".android-b-journal").addEventListener("click", () => ouvrir("journal"));
    // Annuler : le bouton « Annuler mon dernier choix » de la page PC.
    barre.querySelector(".android-b-annuler").addEventListener("click", () => {
      const a = document.getElementById("b-annuler");
      if (a && !a.disabled) a.click();
    });

    const voile = document.createElement("div");
    voile.className = "android-voile";
    voile.addEventListener("click", fermerTout);
    document.getElementById("v-partie").appendChild(voile);

    // L'aperçu de carte (#detail) quitte la colonne de droite : il devient
    // la fiche qui monte du bas. partie.js le retrouve par son identifiant.
    const detail = document.getElementById("detail");
    if (detail) document.getElementById("v-partie").appendChild(detail);

    // Le menu se referme quand on quitte la partie (lien Accueil…).
    const cote = document.querySelector("#v-partie .cote");
    if (cote) cote.addEventListener("click", (e) => { if (e.target.closest("a[href]")) fermerTout(); });
    document.addEventListener("arsenal:vue", fermerTout);
  }

  // Titre de la barre : « Hala 1 – 0 Dromai » (score du match, s'il a
  // plusieurs manches) et « Manche 2 » dessous.
  function majTitre() {
    if (!barre) return;
    // etat : l'état de la partie, variable globale de partie/partie.js.
    const e = typeof etat !== "undefined" && etat && !etat.aucune_partie ? etat : null;
    const vous = e && e.vous ? e.vous.heros : "", bot = e && e.bot ? e.bot.heros : "";
    const score = (document.getElementById("score") || {}).textContent || "";
    const m = score.match(/vous\s+(\d+)\s*–\s*(\d+)\s+bot/);
    const b = barre.querySelector(".android-score"), s = barre.querySelector(".android-manche");
    const gauche = vous || "Vous", droite = bot || "le bot";
    const cle = [gauche, droite, m ? m[1] + "-" + m[2] : ""].join("|");
    // Rien ne change dans la barre si le titre est le même (sinon
    // l'observateur de la page serait relancé sans fin).
    if (b.dataset.cle !== cle) {
      b.dataset.cle = cle;
      b.textContent = gauche + " ";
      if (m) { const n = document.createElement("span"); n.textContent = `${m[1]} – ${m[2]}`; b.append(n, " "); }
      else b.append("contre ");
      b.append(droite);
    }
    const a = document.getElementById("b-annuler"), ba = barre.querySelector(".android-b-annuler");
    if (a && ba.disabled !== a.disabled) ba.disabled = a.disabled;
    const info = (document.getElementById("info") || {}).textContent || "";
    if (s.textContent !== info) s.textContent = info;
  }

  // Tiroirs (menu à gauche, journal à droite) et fiche de carte. Le bouton
  // Retour d'Android les referme (une entrée d'historique sans changer
  // l'adresse, donc sans changer de volet).
  function ouvert() {
    return html.classList.contains("android-tiroir-menu") || html.classList.contains("android-tiroir-journal") || html.classList.contains("android-fiche");
  }
  function ouvrir(quoi) {
    const deja = ouvert();
    html.classList.remove("android-tiroir-menu", "android-tiroir-journal", "android-fiche");
    html.classList.add(quoi === "fiche" ? "android-fiche" : "android-tiroir-" + quoi);
    if (quoi === "journal") { const j = document.getElementById("journal"); if (j) j.scrollTop = j.scrollHeight; }
    if (!deja) history.pushState({ android: quoi }, "");
  }
  function fermerTout() {
    if (!ouvert()) return;
    html.classList.remove("android-tiroir-menu", "android-tiroir-journal", "android-fiche");
    if (history.state && history.state.android) history.back();
  }
  window.addEventListener("popstate", () => {
    html.classList.remove("android-tiroir-menu", "android-tiroir-journal", "android-fiche");
  });

  // Appui long sur une carte (la vôtre, celle du bot, sur la chaîne, dans
  // la pile…) : la fiche avec la carte en grand et son texte officiel.
  // partie.js remplit #detail au survol ; on lui envoie ce survol.
  const APPUI_LONG = 450;
  let appui = null, ignorerClic = false;
  function fiche(carte) {
    carte.dispatchEvent(new MouseEvent("mouseenter"));
    const d = document.getElementById("detail");
    if (!d) return;
    // Le bouton Fermer est hors de #detail : partie.js en remplace le
    // contenu quand le texte officiel arrive.
    if (!document.querySelector(".android-fermer")) {
      const f = document.createElement("button");
      f.type = "button";
      f.className = "android-fermer";
      f.textContent = "Fermer";
      f.addEventListener("click", fermerTout);
      document.getElementById("v-partie").appendChild(f);
    }
    d.scrollTop = 0;
    ouvrir("fiche");
  }
  // Une fiche s'ouvre aussi au toucher simple d'une carte qui ne propose
  // aucune action (carte du bot, cimetière ouvert…) : rien d'autre à faire.
  function annulerAppui() { if (appui) { clearTimeout(appui.t); appui = null; } }
  document.addEventListener("pointerdown", (e) => {
    if (e.pointerType === "mouse") return;
    const c = e.target.closest && e.target.closest("#v-partie .carte, #d-zone .carte");
    if (!c || c.classList.contains("cachee")) return;
    annulerAppui();
    appui = { x: e.clientX, y: e.clientY, t: setTimeout(() => { appui = null; ignorerClic = true; fiche(c); }, APPUI_LONG) };
  }, true);
  document.addEventListener("pointermove", (e) => {
    if (appui && Math.hypot(e.clientX - appui.x, e.clientY - appui.y) > 10) annulerAppui();
  }, true);
  document.addEventListener("pointerup", annulerAppui, true);
  document.addEventListener("pointercancel", annulerAppui, true);
  // Le clic qui suit un appui long n'agit pas sur la partie.
  document.addEventListener("click", (e) => {
    if (!ignorerClic) return;
    ignorerClic = false;
    e.preventDefault();
    e.stopPropagation();
  }, true);
  document.addEventListener("contextmenu", (e) => {
    if (e.target.closest && e.target.closest("#v-partie")) e.preventDefault();
  });
  // Toucher simple d'une carte sans action : la fiche directement.
  document.addEventListener("click", (e) => {
    const c = e.target.closest && e.target.closest("#plateau .carte");
    if (!c || c.classList.contains("cachee") || c.classList.contains("jouable")) return;
    if (c.closest(".hand") || c.classList.contains("on") || c.classList.contains("sel")) return;
    if (c.closest("button.z")) return; // cimetière, banni : la liste s'ouvre
    fiche(c);
  });

  // Permanents : les exemplaires identiques (même nom, mêmes compteurs) qui
  // ne proposent aucune action sont regroupés en une carte « ×N ». La page
  // PC ne regroupe qu'à partir de 4 (carte .pile-perm) : on garde son
  // nombre et on regroupe aussi par 2 et 3.
  function regrouperPermanents() {
    for (const p of document.querySelectorAll("#plateau .perms")) {
      if (p.dataset.android) continue;
      p.dataset.android = "1";
      const groupes = new Map();
      for (const c of p.children) {
        if (!c.classList.contains("carte") || c.classList.contains("jouable") || c.classList.contains("on") || c.classList.contains("sel")) continue;
        const cle = (c.getAttribute("title") || "").replace(/\n?\d+ exemplaires$/, "") + "|" + c.className.replace(/\b(pile-perm|engage)\b/g, "");
        const nb = c.querySelector(".nb-perm");
        const n = nb ? parseInt(nb.textContent.replace(/\D/g, ""), 10) || 1 : 1;
        const g = groupes.get(cle);
        if (!g) groupes.set(cle, { c, n });
        else { g.n += n; c.classList.add("android-cache"); }
      }
      for (const { c, n } of groupes.values()) {
        if (n < 2) continue;
        const nb = c.querySelector(".nb-perm");
        if (nb) nb.textContent = "×" + n;
        else { const s = document.createElement("span"); s.className = "android-nb"; s.textContent = "×" + n; c.appendChild(s); }
      }
    }
  }

  // ---------- Mise en route ----------
  function apresRendu() {
    regrouperPermanents();
    majTitre();
    const dk = document.getElementById("dk-vue");
    if (dk) tuilesDeDecks(dk);
  }
  let prevu = false;
  const observateur = new MutationObserver(() => {
    if (prevu) return;
    prevu = true;
    requestAnimationFrame(() => { prevu = false; apresRendu(); });
  });
  function demarrer() {
    creerBarre();
    observateur.observe(document.body, { childList: true, subtree: true, characterData: true });
    apresRendu();
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", demarrer);
  else demarrer();
})();
