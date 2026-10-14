// Spike front end: renders a deck-tracker-like widget, keeps the host's input shape in
// sync with the interactive elements, and shows what the tracker reports.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let messages = {};
let fallback = {};

async function loadLocale(locale) {
  const response = await fetch(`locales/${locale}.json`);
  return response.ok ? response.json() : {};
}

function t(id, args = {}) {
  const template = messages[id] ?? fallback[id] ?? id;
  return template.replace(/\{(\w+)\}/g, (_, key) => String(args[key] ?? ''));
}

// --- Input shape ----------------------------------------------------------------------

let shapeQueued = false;

function syncInteractiveRegions() {
  if (shapeQueued) return;
  shapeQueued = true;
  requestAnimationFrame(() => {
    shapeQueued = false;
    const rects = [...document.querySelectorAll('.interactive')].map((el) => {
      const r = el.getBoundingClientRect();
      return { x: r.x, y: r.y, width: r.width, height: r.height };
    });
    invoke('set_interactive_regions', { rects, devicePixelRatio: window.devicePixelRatio }).catch(
      (err) => console.error(err),
    );
  });
}

new ResizeObserver(syncInteractiveRegions).observe(document.body);

// --- Deck widget ----------------------------------------------------------------------

async function renderDeck() {
  const deck = await (await fetch('sample-deck.json')).json();
  const list = document.getElementById('cards');
  let remaining = 0;
  for (const card of deck) {
    const left = card.count - card.drawn;
    remaining += left;
    const row = document.createElement('li');
    row.className = `card ${card.rarity}${left === 0 ? ' drawn' : ''}`;
    row.innerHTML =
      '<span class="mana"></span><span class="name"></span><span class="count"></span>';
    row.querySelector('.mana').textContent = card.cost;
    row.querySelector('.name').textContent = card.name;
    row.querySelector('.count').textContent = left > 1 ? left : '';
    list.append(row);
  }
  document.getElementById('tracker-remaining').textContent = t('tracker.remaining', {
    n: remaining,
  });
}

function setupCollapse() {
  const tracker = document.getElementById('tracker');
  const button = document.getElementById('collapse');
  const label = () => {
    button.textContent = t(
      tracker.classList.contains('collapsed') ? 'tracker.expand' : 'tracker.collapse',
    );
  };
  button.addEventListener('click', () => {
    tracker.classList.toggle('collapsed');
    label();
    syncInteractiveRegions();
  });
  label();
}

function setupDrag() {
  const tracker = document.getElementById('tracker');
  const handle = document.getElementById('tracker-handle');
  handle.addEventListener('pointerdown', (event) => {
    if (event.target.closest('button')) return;
    const start = tracker.getBoundingClientRect();
    const dx = event.clientX - start.left;
    const dy = event.clientY - start.top;
    handle.setPointerCapture(event.pointerId);
    const move = (e) => {
      tracker.style.left = `${e.clientX - dx}px`;
      tracker.style.top = `${e.clientY - dy}px`;
      tracker.style.right = 'auto';
      syncInteractiveRegions();
    };
    const up = () => {
      handle.removeEventListener('pointermove', move);
      handle.removeEventListener('pointerup', up);
    };
    handle.addEventListener('pointermove', move);
    handle.addEventListener('pointerup', up);
  });
}

// --- Status and FPS -------------------------------------------------------------------

let config = { demo: false, stacking: 'override' };

function showState(state) {
  const b = state.bounds ?? {};
  const size = { w: b.width, h: b.height, x: b.x, y: b.y };
  let text;
  if (state.kind === 'demo') text = t('status.demo', size);
  else if (state.kind === 'tracking' && state.hidden) text = t('status.hidden');
  else if (state.kind === 'tracking')
    text = t('status.tracking', { ...size, pid: state.pid, flavor: state.flavor });
  else text = t('status.searching');
  document.getElementById('status-text').textContent =
    `${text} · ${t('status.stacking', { mode: config.stacking })}`;
  syncInteractiveRegions();
}

function setupFps() {
  // Off by default: a permanent animation frame loop would distort the idle CPU measurement.
  const button = document.getElementById('fps-toggle');
  const output = document.getElementById('fps');
  let running = false;
  let frames = 0;
  let since = performance.now();
  const tick = (now) => {
    if (!running) return;
    frames += 1;
    if (now - since >= 1000) {
      output.textContent = t('status.fps', { n: Math.round((frames * 1000) / (now - since)) });
      frames = 0;
      since = now;
    }
    requestAnimationFrame(tick);
  };
  const label = () => (button.textContent = t(running ? 'status.fps-on' : 'status.fps-off'));
  button.addEventListener('click', () => {
    running = !running;
    output.textContent = '';
    label();
    if (running) requestAnimationFrame(tick);
    syncInteractiveRegions();
  });
  label();
}

async function main() {
  const locale = navigator.language.split('-')[0];
  fallback = await loadLocale('en');
  messages = locale === 'en' ? fallback : await loadLocale(locale).catch(() => ({}));
  document.documentElement.lang = locale;
  document.getElementById('tracker-title').textContent = t('tracker.title');

  config = await invoke('spike_config');
  await renderDeck();
  setupCollapse();
  setupDrag();
  setupFps();
  await listen('track-state', (event) => showState(event.payload));
  showState(await invoke('track_state'));
  syncInteractiveRegions();
}

main();
