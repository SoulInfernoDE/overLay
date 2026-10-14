// The launcher page of Hello Overlay: shares storage with the panel and resets it over
// the message bus.

import { overlay } from '@overlay/sdk';
import { el } from './dom';
import './desktop.css';

async function main() {
  await overlay.ready();
  const t = overlay.i18n.t;
  const root = document.getElementById('app')!;
  let count = (await overlay.storage.get<number>('count')) ?? 0;

  async function reset() {
    count = 0;
    await overlay.storage.set('count', 0);
    await overlay.bus.post({ type: 'reset' });
    render();
  }

  function render() {
    root.replaceChildren(
      el('h1', { text: t('desktop-title') }),
      el('p', { text: t('desktop-hint') }),
      el('p', { className: 'count', text: t('panel-count', { count }) }),
      el('button', { text: t('desktop-reset'), onclick: () => void reset() }),
    );
  }

  overlay.i18n.onChange(render);
  overlay.bus.onMessage((message) => {
    const m = message as { type?: string; count?: number };
    if (m.type === 'count' && typeof m.count === 'number') {
      count = m.count;
      render();
    }
  });
  render();
}

void main();
