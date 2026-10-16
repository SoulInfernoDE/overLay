<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { GameState, Rect } from '@overlay/sdk/protocol';
  import {
    coreRequest,
    inHost,
    modBaseUrl,
    onHostEvent,
    type ModEvent,
    type ModList,
    type Position,
    type Settings,
    type Snapshot,
  } from '../lib/host';
  import { applyLanguage, currentLocale, t } from '../lib/i18n.svelte';
  import { ModFrameHost } from '../lib/mod-frames';
  import DemoWidget from './DemoWidget.svelte';
  import ModWindow, { type WindowState } from './ModWindow.svelte';
  import { fitInside } from './fit';
  import { syncRegions } from './regions';
  import type { GameWindow, OverlayStatus, OverlayTarget } from './types';

  const TOAST_MS = 6000;

  let settings = $state<Settings | null>(null);
  let target = $state<OverlayTarget | null>(null);
  let gameWindow = $state<GameWindow | null>(null);
  let positions = $state<Record<string, Position>>({});
  let toasts = $state<{ id: number; text: string }[]>([]);
  let mods = $state<ModList>({ mods: [], errors: [] });
  let modBase = $state('mod://localhost');
  let windowStates = $state<Record<string, WindowState>>({});
  let greeted = false;
  let nextToast = 0;

  const resolution = $derived(gameWindow ? `${gameWindow.width}x${gameWindow.height}` : null);

  // Widget positions are stored per game and resolution.
  $effect(() => {
    const game = target;
    const size = resolution;
    if (!game || !size) return;
    void coreRequest<Record<string, Position>>({
      type: 'load-layout',
      game_id: game.game_id,
      resolution: size,
    }).then((loaded) => (positions = loaded));
  });

  // Overlay windows of enabled mods that support the current game.
  const modWindows = $derived(
    target
      ? mods.mods
          .filter((m) => m.enabled && target && m.manifest.games.includes(target.game_id))
          .flatMap((m) =>
            Object.entries(m.manifest.windows)
              .filter(([, spec]) => spec.kind === 'overlay')
              .map(([id, spec]) => ({ mod: m, id, spec, key: `${m.manifest.id}/${id}` })),
          )
      : [],
  );

  function gameState(): GameState | null {
    if (!target) return null;
    return {
      gameId: target.game_id,
      gameName: target.game_name,
      window: gameWindow ? { width: gameWindow.width, height: gameWindow.height } : null,
    };
  }

  // Every mod window gets its layout state once, outside of rendering.
  $effect(() => {
    for (const w of modWindows) {
      windowStates[w.key] ??= {
        x: positions[w.key]?.x ?? w.spec.position?.x ?? 24,
        y: positions[w.key]?.y ?? w.spec.position?.y ?? 96,
        regions: null,
        clickThrough: w.spec.click_through,
      };
    }
  });

  // Stored positions win once they are loaded for this resolution.
  $effect(() => {
    for (const [key, position] of Object.entries(positions)) {
      const existing = windowStates[key];
      if (existing) {
        existing.x = position.x;
        existing.y = position.y;
      }
    }
  });

  const frameHost = new ModFrameHost({
    context: (entry) => ({
      modId: entry.mod.manifest.id,
      window: entry.window,
      kind: 'overlay',
      locale: currentLocale(),
      permissions: entry.mod.manifest.permissions,
      settings: entry.mod.values,
      modBase,
      game: entry.mod.manifest.permissions.includes('game.state') ? gameState() : null,
    }),
    local: (entry, method, params) => {
      const key = `${entry.mod.manifest.id}/${entry.window}`;
      const layout = windowStates[key];
      if (!layout) return undefined;
      switch (method) {
        case 'windows.dragBy': {
          // Starts from where the window is shown, which may differ from its stored spot.
          const spec = entry.mod.manifest.windows[entry.window];
          const fit = (point: Position) =>
            fitInside(
              point,
              spec?.width ?? 0,
              spec?.height ?? 0,
              window.innerWidth,
              window.innerHeight,
            );
          const from = fit(layout);
          const to = fit({ x: from.x + Number(params.dx), y: from.y + Number(params.dy) });
          layout.x = to.x;
          layout.y = to.y;
          return Promise.resolve(null);
        }
        case 'windows.dragEnd':
          savePosition(key, { x: layout.x, y: layout.y });
          return Promise.resolve(null);
        case 'windows.setRegions':
          layout.regions = Array.isArray(params.rects) ? (params.rects as Rect[]) : null;
          return Promise.resolve(null);
        case 'windows.setClickThrough':
          layout.clickThrough = Boolean(params.enabled);
          return Promise.resolve(null);
      }
      return undefined;
    },
  });

  // Mods with `game.state` follow the game window. Only real changes are sent: focus
  // changes of the game window happen on every click into the overlay, and a mod that
  // re-renders on them would lose the click.
  let lastGameState = '';
  $effect(() => {
    const current = gameState();
    const key = JSON.stringify(current);
    if (key === lastGameState) return;
    lastGameState = key;
    frameHost.emitEach('game-state', (entry) =>
      entry.mod.manifest.permissions.includes('game.state') ? current : undefined,
    );
  });

  // Mods reload their catalogs when the language changes.
  $effect(() => {
    frameHost.emit('locale', currentLocale());
  });

  // Toasts that arrive while the overlay is hidden wait until it is visible again.
  let visible = false;
  const waiting: string[] = [];

  function showToast(text: string) {
    if (!visible) {
      waiting.push(text);
      return;
    }
    const id = nextToast++;
    toasts.push({ id, text });
    setTimeout(() => (toasts = toasts.filter((toast) => toast.id !== id)), TOAST_MS);
  }

  /** Fetches the toasts the core keeps for the overlay. */
  async function takeToasts() {
    const taken = await coreRequest<{ text: string; source: string | null }[]>({
      type: 'take-toasts',
    });
    for (const toast of taken) {
      showToast(toast.source ? `${toast.source}: ${toast.text}` : toast.text);
    }
  }

  function onVisible(isVisible: boolean) {
    visible = isVisible;
    if (visible) {
      for (const text of waiting.splice(0)) showToast(text);
    }
    if (!visible || greeted || !target || !settings) return;
    greeted = true;
    showToast(
      t('overlay-toast-active', {
        game: target.game_name,
        hotkey: settings.hotkeys.toggle_overlay,
      }),
    );
  }

  function savePosition(widget: string, position: Position) {
    positions[widget] = position;
    if (!target || !resolution) return;
    void coreRequest({
      type: 'save-widget-position',
      game_id: target.game_id,
      resolution,
      widget,
      position,
    });
  }

  onMount(async () => {
    const snapshot = await coreRequest<Snapshot>({ type: 'snapshot' });
    settings = snapshot.settings;
    applyLanguage(settings.language);
    await onHostEvent<Settings>('settings-changed', (changed) => {
      settings = changed;
      applyLanguage(changed.language);
    });
    await onHostEvent<OverlayTarget>('overlay-target', (changed) => (target = changed));
    await onHostEvent<GameWindow | null>('game-window', (changed) => (gameWindow = changed));
    await onHostEvent<boolean>('overlay-visible', onVisible);
    modBase = await modBaseUrl();
    mods = await coreRequest<ModList>({ type: 'list-mods' });
    await onHostEvent<ModList>('mods-changed', (list) => (mods = list));
    await onHostEvent<ModEvent>('mod-event', (event) =>
      frameHost.emit(event.name, event.data, { modId: event.mod_id, permission: event.permission }),
    );
    await onHostEvent<null>('toasts-pending', () => void takeToasts());
    // The hotkey must show an effect even while no widget is on screen.
    await onHostEvent<boolean>('overlay-toggled', (hidden) => {
      if (!hidden && settings) {
        showToast(t('overlay-toast-shown', { hotkey: settings.hotkeys.toggle_overlay }));
      }
    });

    if (inHost) {
      const status = await invoke<OverlayStatus>('overlay_status');
      target = status.target;
      gameWindow = status.window;
      const w = status.window;
      onVisible(!!w && !w.hidden && (w.active || !w.covered) && !status.user_hidden);
    } else {
      // Browser preview: show the widget over a pretend game window.
      settings.overlay.demo_widget = true;
      target = { game_id: 'hearthstone', game_name: 'Hearthstone', pid: 0 };
      gameWindow = {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        hidden: false,
        active: true,
        covered: false,
      };
      onVisible(true);
    }
    await takeToasts();
    syncRegions();
  });
</script>

{#each modWindows as w (w.key)}
  {@const layout = windowStates[w.key]}
  {#if layout}
    <ModWindow
      mod={w.mod}
      windowId={w.id}
      spec={w.spec}
      {layout}
      src={`${modBase}/${w.mod.manifest.id}/${w.spec.entry}`}
      host={frameHost}
    />
  {/if}
{/each}

{#if settings?.overlay.demo_widget}
  <DemoWidget
    position={positions.demo}
    {gameWindow}
    onmove={(position) => savePosition('demo', position)}
  />
{/if}

<div class="toasts" aria-live="polite">
  {#each toasts as toast (toast.id)}
    <div class="toast">{toast.text}</div>
  {/each}
</div>

<style>
  /* Toasts never take clicks, so they cannot block the game. */
  .toasts {
    position: absolute;
    right: var(--ol-space-6);
    bottom: var(--ol-space-6);
    display: grid;
    gap: var(--ol-space-2);
    max-width: 420px;
    pointer-events: none;
  }

  .toast {
    padding: var(--ol-space-3) var(--ol-space-4);
    border-left: 3px solid var(--ol-accent);
    border-radius: var(--ol-radius-md);
    background: var(--ol-surface-overlay);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
  }
</style>
