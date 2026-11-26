// Images des cartes (lot 1 du plan de rapatriement). Le serveur donne pour
// chaque carte un code d'image (`FR_SBR016` : version française d'abord, voir
// crates/ui/src/images.rs et tools/images_cartes.py) ; l'image est chargée en
// ligne sur le serveur d'images de Legend Story Studios. Sans Internet, ou si
// une image manque, la carte garde son cadre dessiné.
"use strict";
const Images = (() => {
  const BASE = "https://legendstory-production-s3-public.s3.amazonaws.com/media/cards/";
  // taille : "normal" (≈ 450 px de large) ou "large"
  const url = (code, taille) => code ? BASE + (taille || "normal") + "/" + encodeURIComponent(code) + ".webp" : "";
  // Une <img> qui se retire d'elle-même (et retire `classe` du parent) si l'image ne vient pas.
  function img(code, taille, parent, classe) {
    const i = document.createElement("img");
    i.alt = ""; i.decoding = "async"; i.referrerPolicy = "no-referrer";
    i.src = url(code, taille);
    i.addEventListener("error", () => { if (parent && classe) parent.classList.remove(classe); i.remove(); });
    return i;
  }
  // Fond d'illustration (cadrage sur l'art de la carte, comme dans l'Arsenal JS)
  const fond = (code) => code ? `url('${url(code, "large")}'), url('${url(code, "normal")}')` : "";
  return { url, img, fond };
})();

// Symboles du fichier officiel des cartes (colonne *_true_textbox de LSS) :
// {r} ressource, {p} puissance, {d} défense, {h} et {g} vie (deux notations
// du même symbole selon l'édition), {i} intellect, {t} engager, {u} dégager,
// {c} chi. Symboles.html(texte) échappe le texte puis remplace chaque symbole
// connu par une petite icône (avec son nom pour les lecteurs d'écran et au
// survol). Les symboles dont le sens n'est pas vérifié ({x}, {v}, {^}…)
// restent tels quels. TODO(règles) : vérifier {x} et {v}/{^} sur le fichier LSS.
const Symboles = (() => {
  const svg = (d, vb) => `<svg viewBox="${vb || "0 0 16 16"}" aria-hidden="true" fill="currentColor"><path d="${d}"/></svg>`;
  const EPEE = "M13.5 1.5 6 9l1 1 7.5-7.5V1.5h-1zM3 10l3 3-1 1-1-1-1.5 1.5-1-1L3 12l-1-1 1-1z";
  const BOUCLIER = "M8 1 2.5 3v4.2c0 3.3 2.3 6.3 5.5 7.3 3.2-1 5.5-4 5.5-7.3V3L8 1z";
  const COEUR = "M8 14.5 2.3 8.9A3.6 3.6 0 0 1 8 4.2a3.6 3.6 0 0 1 5.7 4.7z";
  const TABLE = {
    r: ["sym-r", "ressource", svg("M8 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2z")],
    p: ["sym-p", "puissance", svg(EPEE)],
    d: ["sym-d", "défense", svg(BOUCLIER)],
    h: ["sym-h", "vie", svg(COEUR)],
    g: ["sym-h", "vie", svg(COEUR)],
    i: ["sym-i", "intellect", "i"],
    t: ["sym-t", "engager", "↷"],
    u: ["sym-t", "dégager", "↶"],
    c: ["sym-c", "chi", "chi"],
  };
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  // Texte brut lisible (infobulles, title="…") : « +3{p} » → « +3 puissance ».
  const texte = (t) => String(t ?? "").replace(/\{([a-z])\}/g, (m, k) => TABLE[k] ? " " + TABLE[k][1] : m).replace(/ {2,}/g, " ");
  // HTML avec icônes (panneau Carte, aperçus).
  const html = (t) => esc(t).replace(/\{([a-z])\}/g, (m, k) => {
    const s = TABLE[k];
    return s ? `<span class="sym ${s[0]}" role="img" aria-label="${s[1]}" title="${s[1]}">${s[2]}</span>` : m;
  });
  return { html, texte };
})();

// Vrai dos de carte (image locale dans data/outils/, voir arsenal.css).
fetch("/api/outils").then((r) => r.json()).then((o) => {
  if (o && o.dos) document.documentElement.classList.add("vrai-dos");
}).catch(() => { /* serveur sans cette route : dos dessiné */ });
