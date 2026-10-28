// Optional DOM check. Requires jsdom; does not establish browser layout or visual quality.
const { JSDOM, VirtualConsole } = require('jsdom');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
let pages = 0;
for (const task of ['authored-html', 'artifact-style']) {
  const dir = path.resolve('harness/task-artifacts', task);
  const ledger = JSON.parse(fs.readFileSync(path.join(dir, 'authored.json')));
  for (const [i, revision] of ledger.revisions.entries()) {
    // Old revisions predating the foundation are deliberately preserved.
    if (!revision.files.some(s => s.file.path === 'foundation.css')) continue;
    for (const base of [path.join(dir, 'authored-revisions', String(i + 1)), ...(i === ledger.revisions.length - 1 ? [dir] : [])]) {
      for (const snapshot of revision.files.filter(s => s.file.path.endsWith('.html'))) {
        const file = path.join(base, snapshot.file.path);
        const errors = [];
        const virtualConsole = new VirtualConsole();
        virtualConsole.on('jsdomError', e => errors.push(e.message));
        const dom = new JSDOM(fs.readFileSync(file, 'utf8'), { runScripts: 'dangerously', virtualConsole, url: 'https://example.test/' + snapshot.file.path });
        const d = dom.window.document;
        const link = d.head.querySelector('link[data-artifact-foundation="1"]');
        assert.ok(link, file + ' must load the foundation in its head');
        assert.equal(link.rel, 'stylesheet');
        const cssPath = path.resolve(path.dirname(file), link.getAttribute('href'));
        assert.equal(cssPath, path.join(base, 'foundation.css'));
        const style = d.createElement('style');
        style.textContent = fs.readFileSync(cssPath, 'utf8');
        d.head.appendChild(style);
        assert.ok(style.sheet.cssRules.length > 30);
        assert.equal(dom.window.getComputedStyle(d.documentElement).getPropertyValue('--accent').trim(), '#126d5c');
        if (d.querySelector('#export')) {
          d.querySelector('#response').value = 'Review this claim';
          d.querySelector('#export').click();
          assert.equal(JSON.parse(d.querySelector('#payload').value).observations.response, 'Review this claim');
        }
        if (d.querySelector('#approach')) {
          d.querySelector('#approach').value = 'presence';
          d.querySelector('#approach').dispatchEvent(new dom.window.Event('change'));
          assert.match(d.querySelector('#effect').textContent, /without detection/);
        }
        if (d.querySelector('#history')) {
          d.querySelector('#history').checked = false;
          d.querySelector('#history').dispatchEvent(new dom.window.Event('change'));
          assert.match(d.querySelector('#consequence').textContent, /past reviewed content is lost/);
        }
        if (d.querySelector('#prepare')) {
          d.querySelector('#focus').value = 'Keyboard controls';
          d.querySelector('#prepare').click();
          assert.equal(d.querySelector('#preview').textContent, 'Review Keyboard controls');
          d.querySelector('#clear').click();
          assert.equal(d.querySelector('#preview').textContent, 'No response prepared.');
        }
        assert.deepEqual(errors, [], file);
        dom.window.close();
        pages++;
      }
    }
  }
}
console.log(JSON.stringify({ ok: true, pages, scope: 'Local revision CSS resolution, CSS parsing and DOM interactions; not browser visual verification' }, null, 2));
