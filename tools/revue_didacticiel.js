// Revue : capture de chaque étape des leçons données, à la taille d'écran d'Antoine.
const { chromium } = require('playwright');
const [, , dossier, w, h, ...lecons] = process.argv;
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: +w, height: +h } });
  const errs = []; p.on('pageerror', e => errs.push(e.message));
  await p.route('https://rules.fabtcg.com/**', r => r.abort());
  const cliquer = (sel) => p.evaluate((s) => { const n = document.querySelector(s); if (n) n.click(); return !!n; }, sel);
  for (const id of lecons) {
    await p.goto('http://127.0.0.1:8091/#apprendre'); await p.reload(); await p.waitForTimeout(900);
    await cliquer(`.ap-pas[data-l="${id}"]`);
    let n = 0;
    const cap = async () => {
      const nom = `${id}-${String(n++).padStart(2, '0')}`;
      const plein = await p.$$eval('.ap-tapis .ct, .ap-tapis .ap-slot, .ap-tapis .ap-frise, .ap-tapis .ap-note, .ap-tapis .ap-recap, .ap-tapis .ap-fin, .ap-tapis .ap-zoom', (ns) => ns.length);
      if (!plein) console.log('VIDE', nom);
      return p.screenshot({ path: `${dossier}/${nom}.png` });
    };
    for (let s = 0; s < 40; s++) {
      await p.waitForTimeout(3300);
      await cap();
      if (await p.$('.ap-fin')) break;
      if (await cliquer('.ct.appel')) continue;
      if (await p.$('.ap-choix [data-i]')) {
        for (let t = 0; t < 4 && !(await p.$('.ap-choix .bon')); t++) {
          await cliquer('.ap-choix [data-i]:not([disabled])'); await p.waitForTimeout(700);
        }
        await cap();
      }
      const jouables = await p.$$eval('.ct.jouable', ns => ns.map(n => n.dataset.id));
      if (jouables.length && !(await p.$('[data-nav="suiv"]'))) { await cliquer(`.ct.jouable[data-id="${jouables[0]}"]`); continue; }
      if (!(await cliquer('[data-nav="suiv"]'))) await cliquer('[data-o]');
    }
  }
  console.log('ERR', JSON.stringify(errs)); await b.close();
})();
