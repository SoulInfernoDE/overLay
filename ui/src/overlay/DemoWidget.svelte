<script lang="ts">
  import type { Position } from '../lib/host';
  import { t } from '../lib/i18n.svelte';
  import { syncRegions } from './regions';
  import type { GameWindow } from './types';

  let {
    position,
    gameWindow,
    onmove,
  }: {
    position: Position | undefined;
    gameWindow: GameWindow | null;
    onmove: (position: Position) => void;
  } = $props();

  const DEFAULT: Position = { x: 24, y: 96 };

  let collapsed = $state(false);
  let dragging = $state<Position | null>(null);
  const current = $derived(dragging ?? position ?? DEFAULT);

  // Keep at least the title bar reachable inside the overlay.
  function clamp(p: Position): Position {
    return {
      x: Math.min(Math.max(p.x, 0), window.innerWidth - 80),
      y: Math.min(Math.max(p.y, 0), window.innerHeight - 32),
    };
  }

  function startDrag(event: PointerEvent) {
    if ((event.target as Element).closest('button')) return;
    const handle = event.currentTarget as HTMLElement;
    const offset = { x: event.clientX - current.x, y: event.clientY - current.y };
    handle.setPointerCapture(event.pointerId);
    const move = (e: PointerEvent) => {
      dragging = clamp({ x: e.clientX - offset.x, y: e.clientY - offset.y });
    };
    const stop = () => {
      handle.removeEventListener('pointermove', move);
      handle.removeEventListener('pointerup', stop);
      if (dragging) onmove(dragging);
      dragging = null;
    };
    handle.addEventListener('pointermove', move);
    handle.addEventListener('pointerup', stop);
  }

  $effect(() => {
    // Re-report the clickable area whenever the widget moves or changes size.
    void current.x;
    void current.y;
    void collapsed;
    syncRegions();
  });
</script>

<section class="widget interactive" style:left="{current.x}px" style:top="{current.y}px">
  <header
    role="toolbar"
    tabindex="-1"
    aria-label={t('overlay-demo-title')}
    onpointerdown={startDrag}
  >
    <span class="title">{t('overlay-demo-title')}</span>
    <button onclick={() => (collapsed = !collapsed)}>
      {collapsed ? t('overlay-demo-expand') : t('overlay-demo-collapse')}
    </button>
  </header>
  {#if !collapsed}
    <div class="body">
      <p>{t('overlay-demo-body')}</p>
      {#if gameWindow}
        <p class="muted">
          {t('overlay-demo-window', {
            // Plain text: resolutions are never written with digit grouping.
            width: String(gameWindow.width),
            height: String(gameWindow.height),
          })}
        </p>
      {/if}
    </div>
  {/if}
</section>

<style>
  .widget {
    position: absolute;
    width: 280px;
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-md);
    background: var(--ol-surface-overlay);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
  }

  header {
    display: flex;
    align-items: center;
    gap: var(--ol-space-2);
    padding: var(--ol-space-2) var(--ol-space-3);
    border-bottom: 1px solid var(--ol-border);
    cursor: grab;
  }

  .title {
    flex: 1;
    font-weight: 600;
    color: var(--ol-accent);
  }

  button {
    padding: 2px var(--ol-space-2);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-hover);
    font-size: var(--ol-font-size-sm);
    cursor: pointer;
  }

  .body {
    padding: var(--ol-space-3);
  }

  .body p {
    margin: 0 0 var(--ol-space-2);
  }

  .muted {
    color: var(--ol-text-muted);
    font-size: var(--ol-font-size-sm);
  }
</style>
