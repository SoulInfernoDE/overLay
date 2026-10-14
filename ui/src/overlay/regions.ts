// Reports the widgets' rectangles to the overlay process: only these take mouse input,
// everything else passes through to the game. Mark clickable elements with `.interactive`.

import { invoke } from '@tauri-apps/api/core';
import { inHost } from '../lib/host';

let queued = false;

export function syncRegions(): void {
  if (!inHost || queued) return;
  queued = true;
  requestAnimationFrame(() => {
    queued = false;
    const rects = [...document.querySelectorAll('.interactive')].map((element) => {
      const r = element.getBoundingClientRect();
      return { x: r.x, y: r.y, width: r.width, height: r.height };
    });
    void invoke('set_interactive_regions', { rects, devicePixelRatio: window.devicePixelRatio });
  });
}

new ResizeObserver(syncRegions).observe(document.documentElement);
