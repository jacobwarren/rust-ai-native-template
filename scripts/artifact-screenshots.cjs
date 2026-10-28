#!/usr/bin/env node
// Render HTML pages to PNG tiles with a local Chrome over the DevTools pipe.
// No npm packages. Usage: node scripts/artifact-screenshots.cjs OUT_DIR PAGE.html...
// Set CHROME to a Chrome/Chromium binary when it is not in a standard location.
'use strict';
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

const VIEWPORTS = [
  { name: 'desktop', width: 1280, height: 900, mobile: false },
  { name: 'phone', width: 390, height: 844, mobile: true },
];
const MAX_TILES = 12;
const SETTLE_MS = 400;
const TIMEOUT_MS = 120000;

function chrome() {
  const candidates = [
    process.env.CHROME,
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium',
    '/usr/bin/google-chrome',
    '/usr/bin/google-chrome-stable',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
  ];
  return candidates.find((p) => p && fs.existsSync(p));
}

// DevTools protocol over --remote-debugging-pipe: NUL-terminated JSON on fds 3 (in) and 4 (out).
function connect(child) {
  let next = 1;
  let buffer = Buffer.alloc(0);
  const pending = new Map();
  const listeners = new Set();
  child.stdio[4].on('data', (data) => {
    buffer = Buffer.concat([buffer, data]);
    for (let end; (end = buffer.indexOf(0)) >= 0; ) {
      const message = JSON.parse(buffer.subarray(0, end).toString());
      buffer = buffer.subarray(end + 1);
      const waiter = message.id && pending.get(message.id);
      if (waiter) {
        pending.delete(message.id);
        if (message.error) waiter.reject(new Error(message.error.message));
        else waiter.resolve(message.result);
      } else {
        listeners.forEach((listener) => listener(message));
      }
    }
  });
  return {
    send(method, params = {}, sessionId) {
      const id = next++;
      child.stdio[3].write(JSON.stringify({ id, method, params, sessionId }) + '\0');
      return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
    },
    listen(fn) {
      listeners.add(fn);
      return () => listeners.delete(fn);
    },
  };
}

async function capture(cdp, page, viewport, outDir) {
  const { targetId } = await cdp.send('Target.createTarget', { url: 'about:blank' });
  const { sessionId } = await cdp.send('Target.attachToTarget', { targetId, flatten: true });
  const send = (method, params) => cdp.send(method, params, sessionId);
  const result = { console_errors: [], external_requests: [] };
  let loaded;
  const onLoad = new Promise((resolve) => (loaded = resolve));
  const stop = cdp.listen((m) => {
    if (m.sessionId !== sessionId) return;
    if (m.method === 'Page.loadEventFired') loaded();
    if (m.method === 'Runtime.exceptionThrown') {
      const d = m.params.exceptionDetails;
      result.console_errors.push((d.exception && d.exception.description) || d.text);
    }
    if (m.method === 'Runtime.consoleAPICalled' && m.params.type === 'error') {
      result.console_errors.push(m.params.args.map((a) => a.value ?? a.description).join(' '));
    }
    if (m.method === 'Network.requestWillBeSent') {
      const url = m.params.request.url;
      if (!/^(file|data|blob|about):/.test(url)) result.external_requests.push(url);
    }
  });
  await Promise.all(['Page.enable', 'Runtime.enable', 'Network.enable'].map((m) => send(m)));
  await send('Emulation.setDeviceMetricsOverride', {
    width: viewport.width,
    height: viewport.height,
    deviceScaleFactor: 1,
    mobile: viewport.mobile,
  });
  await send('Page.navigate', { url: pathToFileURL(path.resolve(page)).href });
  await onLoad;
  await new Promise((r) => setTimeout(r, SETTLE_MS));
  const { result: metrics } = await send('Runtime.evaluate', {
    returnByValue: true,
    // Outermost elements whose right edge passes the viewport, named like CSS selectors.
    expression: `(() => {
      const root = document.documentElement, cw = root.clientWidth;
      const name = (e) => e.tagName.toLowerCase() + (e.id ? '#' + e.id : '') +
        (typeof e.className === 'string' && e.className.trim() ? '.' + e.className.trim().split(/\\s+/).join('.') : '');
      const wide = [...document.body.querySelectorAll('*')]
        .filter((e) => e.getBoundingClientRect().right > cw + 1);
      const outer = wide.filter((e) => !wide.includes(e.parentElement)).slice(0, 5).map(name);
      return { h: root.scrollHeight, w: root.scrollWidth, cw, title: document.title, outer };
    })()`,
  });
  const { h, w, cw, title, outer } = metrics.value;
  result.title = title;
  result.height = h;
  result.horizontal_overflow = w > cw;
  if (result.horizontal_overflow) result.overflowing = outer;
  result.tiles = [];
  const base = path.basename(page, '.html');
  for (let y = 0; y < h && result.tiles.length < MAX_TILES; y += viewport.height) {
    const { data } = await send('Page.captureScreenshot', {
      format: 'png',
      captureBeyondViewport: true,
      clip: { x: 0, y, width: viewport.width, height: Math.min(viewport.height, h - y), scale: 1 },
    });
    const file = path.join(outDir, `${base}-${viewport.name}-${String(result.tiles.length + 1).padStart(2, '0')}.png`);
    fs.writeFileSync(file, Buffer.from(data, 'base64'));
    result.tiles.push(file);
  }
  result.truncated = result.tiles.length * viewport.height < h;
  stop();
  await cdp.send('Target.closeTarget', { targetId });
  return result;
}

async function main() {
  const [outDir, ...pages] = process.argv.slice(2);
  if (!outDir || pages.length === 0) throw new Error('usage: artifact-screenshots.cjs OUT_DIR PAGE.html...');
  const binary = chrome();
  if (!binary) throw new Error('no Chrome/Chromium found; set CHROME to its binary');
  fs.mkdirSync(outDir, { recursive: true });
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'artifact-shots-'));
  const child = spawn(
    binary,
    ['--headless=new', '--remote-debugging-pipe', '--no-first-run', '--no-default-browser-check',
      '--disable-gpu', '--hide-scrollbars', '--mute-audio', `--user-data-dir=${profile}`],
    { stdio: ['ignore', 'ignore', 'ignore', 'pipe', 'pipe'] },
  );
  const timer = setTimeout(() => {
    child.kill('SIGKILL');
    console.log(JSON.stringify({ ok: false, error: `timed out after ${TIMEOUT_MS / 1000}s` }));
    process.exit(1);
  }, TIMEOUT_MS);
  try {
    const cdp = connect(child);
    const report = [];
    for (const page of pages) {
      const views = {};
      for (const viewport of VIEWPORTS) views[viewport.name] = await capture(cdp, page, viewport, outDir);
      report.push({ page, ...views });
    }
    const problems = report.some((r) =>
      VIEWPORTS.some((v) => r[v.name].console_errors.length || r[v.name].external_requests.length) ||
      r.phone.horizontal_overflow);
    console.log(JSON.stringify({ ok: true, chrome: binary, problems, pages: report,
      scope: 'Rendered appearance for a person or agent to inspect; not a quality judgment' }));
  } finally {
    clearTimeout(timer);
    const exited = new Promise((resolve) => child.once('exit', resolve));
    child.kill();
    await exited;
    fs.rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  }
}

main().catch((error) => {
  console.log(JSON.stringify({ ok: false, error: error.message }));
  process.exit(1);
});
