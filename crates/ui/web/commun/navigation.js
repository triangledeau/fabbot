// Navigation d'Arsenal par le # de l'adresse (lot 0 du plan de rapatriement).
// Chaque adresse mène à un volet (une <section class="vue" id="v-…"> de
// index.html) ; le bouton Précédent du navigateur marche donc. Un volet
// apprend qu'il devient visible par l'événement « arsenal:vue ».
"use strict";
const Arsenal = (() => {
  // adresse (#…) → volet affiché. Les lots suivants ajoutent leurs lignes ici.
  const ADRESSES = {
    "accueil": "accueil",
    "partie": "partie",
    "nouveau-match": "partie",
    "decks": "decks",
    "editeur": "decks",
    "impression": "impression",
  };
  const TITRES = { accueil: "Arsenal — FaB Silver Age", partie: "Arsenal — Partie", decks: "Arsenal — Mes decks", impression: "Arsenal — Imprimer" };
  const adresse = () => {
    const a = decodeURIComponent(location.hash.slice(1));
    return ADRESSES[a] ? a : "accueil";
  };
  const vue = () => document.documentElement.dataset.vue || "accueil";
  function montrer() {
    const a = adresse(), v = ADRESSES[a];
    const change = v !== document.documentElement.dataset.vue;
    document.documentElement.dataset.vue = v;
    document.title = TITRES[v] || TITRES.accueil;
    if (change) {
      window.scrollTo(0, 0);
      const t = document.querySelector("#v-" + v + " h1");
      if (t && v !== "accueil") { t.tabIndex = -1; t.focus({ preventScroll: true }); }
    }
    document.dispatchEvent(new CustomEvent("arsenal:vue", { detail: { vue: v, adresse: a } }));
  }
  function aller(a) {
    if (location.hash === "#" + a) montrer(); else location.hash = a;
  }
  window.addEventListener("hashchange", montrer);
  const HOME_SVG = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 11l9-7 9 7"/><path d="M5 10v10h14V10"/></svg>';
  return { vue, adresse, montrer, aller, HOME_SVG };
})();
