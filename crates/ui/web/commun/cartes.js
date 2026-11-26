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
