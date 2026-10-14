// The in-game panel of Hello Overlay.
//
// The DOM is built once and only its texts are updated afterwards. Rebuilding elements
// on every event would replace a button between mouse down and mouse up, and the click
// would be lost.

import { overlay } from '@overlay/sdk';
import { el } from './dom';
import './panel.css';

const HOTKEY = 'Ctrl+Shift+K';

async function main() {
  await overlay.ready();
  const t = overlay.i18n.t;
  let count = (await overlay.storage.get<number>('count')) ?? 0;
  let compact = false;
  let network: string[] = [];

  const title = el('span');
  const hint = el('span', { className: 'hint' });
  const header = el('header', { className: 'handle' }, title, hint);
  const greeting = el('p');
  const size = el('p', { className: 'muted' });
  const counter = el('p', { className: 'muted' });
  const countButton = el('button', { onclick: () => void countClick() });
  const notifyButton = el('button', {
    onclick: () => void overlay.notifications.show(t('notification')),
  });
  const networkButton = el('button', { onclick: () => void testNetwork() });
  const networkLines = el('div');
  const body = el(
    'div',
    { className: 'body' },
    greeting,
    size,
    counter,
    el('div', { className: 'buttons' }, countButton, notifyButton, networkButton),
    networkLines,
  );
  const panel = el('section', { className: 'panel' }, header, body);
  document.getElementById('app')!.replaceChildren(panel);
  overlay.windows.makeDraggable(header);

  async function testNetwork() {
    network = [];
    try {
      const response = await fetch('https://api.hearthstonejson.com/v1/');
      network.push(t('network-allowed', { status: response.status }));
    } catch (error) {
      network.push(t('network-failed', { error: String(error) }));
    }
    try {
      await fetch('https://example.com/');
      network.push(t('network-unexpected'));
    } catch {
      network.push(t('network-blocked'));
    }
    update();
  }

  async function countClick() {
    count += 1;
    update();
    await overlay.storage.set('count', count);
    await overlay.bus.post({ type: 'count', count });
  }

  function update() {
    const settings = overlay.settings.get();
    const game = overlay.game.getState();
    title.textContent = t('panel-title');
    hint.textContent = t('panel-compact-hint', { hotkey: HOTKEY });
    greeting.textContent = game
      ? t(String(settings.greeting ?? 'greeting-friendly'), { game: game.gameName })
      : t('waiting');
    size.hidden = !(settings['show-size'] && game?.window);
    if (game?.window) {
      // Plain text: resolutions are never written with digit grouping.
      size.textContent = t('panel-size', {
        width: String(game.window.width),
        height: String(game.window.height),
      });
    }
    counter.textContent = t('panel-count', { count });
    countButton.textContent = t('panel-click');
    notifyButton.textContent = t('panel-notify');
    networkButton.textContent = t('panel-network');
    networkLines.replaceChildren(
      ...network.map((line) => el('p', { className: 'muted', text: line })),
    );
    body.hidden = compact;
    panel.classList.toggle('compact', compact);
    // In compact mode only the title bar takes clicks; the rest of the window passes
    // them through to the game.
    void overlay.windows.setInteractiveRegions(compact ? [header.getBoundingClientRect()] : null);
  }

  overlay.i18n.onChange(update);
  overlay.settings.onChange(update);
  overlay.game.onChange(update);
  overlay.hotkeys.on('compact', () => {
    compact = !compact;
    update();
  });
  overlay.bus.onMessage((message) => {
    if ((message as { type?: string }).type === 'reset') {
      count = 0;
      update();
    }
  });
  update();
}

void main();
