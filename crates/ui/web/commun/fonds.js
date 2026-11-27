// Fonds d'Arsenal (2026-11-27) : une illustration de lieu, sans personnage,
// fondue dans le fond sombre, tirée au hasard à l'ouverture et à chaque
// changement de page (adresse #…).
//
// Illustrations de cartes Flesh and Blood en haute définition, publiées par
// leurs artistes sur ArtStation : les scans officiels (546 px de large) sont
// trop flous en plein écran. Comme les images des cartes, elles sont chargées
// en ligne et ne sont pas dans le dépôt (dépôt public). Sans Internet, le fond
// reste uni.
//
// Noms et artistes vérifiés dans le fichier officiel LSS.
// Une entrée : n = nom de la carte, a = artiste, u = image (≤ 1920 px),
// p = point de l'image gardé au centre quand l'écran la recadre
// (object-position ; sert aussi à écarter un logo incrusté dans un coin).
"use strict";
const Fonds = (() => {
  const A = "https://cdna.artstation.com/p/assets/images/images/";
  const B = "https://cdnb.artstation.com/p/assets/images/images/";
  const FONDS = [
    { n: "Great Library of Solana", a: "Federico Musetti", u: A + "102/634/734/large/federico-musetti-sow628-fmusetti-def3.webp?1790104894", p: "50% 40%" },
    { n: "Mistcloak Gully", a: "Anton Bezrukov", u: B + "080/058/273/large/anton-bezrukov-grafit-studio-pic.jpg?1726567119", p: "50% 45%" },
    { n: "Ley Line of the Old Ones", a: "Simon Dominic", u: A + "090/709/356/large/simon-dominic-brewer-ley-line-of-the-old-ones-1700.webp?1754648872", p: "50% 35%" },
    { n: "Channel the Bleak Expanse", a: "Mihail Spil-Haufter", u: A + "089/871/480/large/mihail-spil-haufter-winters-graspfeedback06.jpg?1752173151", p: "50% 50%" },
    { n: "Channel Mount Heroic", a: "Mihail Spil-Haufter", u: B + "043/360/703/large/mihail-spil-haufter-channel-mount-heroic-logo.jpg?1637061459", p: "30% 80%" },
    { n: "Channel Galcia's Cradle", a: "Flora Silve", u: A + "096/838/124/large/flora-silve-channel-galcia-cradle-publish.webp?1772462217", p: "50% 45%" },
    { n: "Dust from the Fertile Fields", a: "Flora Silve", u: B + "079/842/599/large/flora-silve-dust-fertile-fields.jpg?1725974409", p: "50% 40%" },
    { n: "Colors of Aria", a: "Flora Silve", u: A + "096/838/348/large/flora-silve-203.webp?1772462616", p: "50% 40%" },
    { n: "Distant Rumbling", a: "Flora Silve", u: B + "096/837/505/large/flora-silve-013.webp?1772461007", p: "50% 40%" },
    { n: "Parched Terrain", a: "Flora Silve", u: B + "092/016/947/large/flora-silve-sup-art-93.webp?1758482973", p: "50% 45%" },
    { n: "Big Blue Sky", a: "Reza Afshar", u: B + "076/519/879/large/reza-afshar-mst-art-062.jpg?1717159066", p: "50% 45%" },
    { n: "Current Funnel", a: "Reza Afshar", u: A + "079/946/718/large/reza-afshar-ros-art-052.jpg?1726232039", p: "50% 40%" },
    // Seat of the Emperor : illustration officielle de Flesh and Blood, pas une carte.
    { n: "Seat of the Emperor", a: "Alena Kuzminykh", u: A + "075/403/670/large/alena-kuzminykh-grafit-studio-seat-of-the-emperor.jpg?1714487179", p: "50% 45%" },
    { n: "Visit the Boneyard", a: "Anna Kovalevskaya (Grafit Studio)", u: A + "099/034/054/large/grafit-studio-visit-the-boneyard.webp?1778751760", p: "50% 45%" },
    { n: "Battalion Barque", a: "Arkoii", u: A + "096/537/680/large/arkoii-thumbnail-sea-art-114.webp?1771605068", p: "50% 35%" },
  ];
  // Tirage hors Math.random (même principe que l'Arsenal JS : ne touche à
  // aucun autre aléa de la page).
  const hasard = (n) => { try { return crypto.getRandomValues(new Uint32Array(1))[0] % n; } catch (e) { return Date.now() % n; } };
  let courant = -1, attendu = -1, derniere = null, fond = null, legende = null;

  function installer() {
    fond = document.createElement("div");
    fond.id = "fond";
    fond.setAttribute("aria-hidden", "true");
    for (let i = 0; i < 2; i++) {
      const img = document.createElement("img");
      img.alt = "";
      img.decoding = "async";
      img.referrerPolicy = "no-referrer";
      fond.append(img);
    }
    legende = document.createElement("div");
    legende.id = "fond-legende";
    document.body.prepend(fond);
    document.body.append(legende);
  }

  // Tire une autre illustration que celle affichée, la charge, puis la fond
  // par-dessus l'ancienne (fondu enchaîné) une fois prête.
  function changer() {
    if (!FONDS.length) return;
    if (!fond) installer();
    let i = hasard(FONDS.length);
    if (FONDS.length > 1 && i === courant) i = (i + 1 + hasard(FONDS.length - 1)) % FONDS.length;
    attendu = i;
    const f = FONDS[i];
    const [x, y] = fond.querySelectorAll("img");
    const suivante = x.classList.contains("vu") ? y : x, ancienne = suivante === x ? y : x;
    suivante.onload = () => {
      if (attendu !== i) return; // une page plus récente a déjà tiré la sienne
      courant = i;
      suivante.classList.add("vu");
      ancienne.classList.remove("vu");
      legende.textContent = `Illustration : ${f.n} · ${f.a}`;
    };
    suivante.onerror = () => { if (attendu === i) attendu = -1; };
    suivante.style.objectPosition = f.p || "50% 50%";
    suivante.src = f.u;
  }

  document.addEventListener("arsenal:vue", (e) => {
    const a = e.detail && e.detail.adresse;
    document.documentElement.dataset.adresse = a || "";
    if (a === derniere) return;
    derniere = a;
    changer();
  });
  return { changer, FONDS };
})();
