<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import {
    coreRequest,
    inHost,
    onHostEvent,
    type Position,
    type Settings,
    type Snapshot,
  } from '../lib/host';
  import { applyLanguage, t } from '../lib/i18n.svelte';
  import DemoWidget from './DemoWidget.svelte';
  import { syncRegions } from './regions';
  import type { GameWindow, OverlayStatus, OverlayTarget } from './types';

  const TOAST_MS = 6000;

  let settings = $state<Settings | null>(null);
  let target = $state<OverlayTarget | null>(null);
  let gameWindow = $state<GameWindow | null>(null);
  let positions = $state<Record<string, Position>>({});
  let toasts = $state<{ id: number; text: string }[]>([]);
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

  function showToast(text: string) {
    const id = nextToast++;
    toasts.push({ id, text });
    setTimeout(() => (toasts = toasts.filter((toast) => toast.id !== id)), TOAST_MS);
  }

  function onVisible(visible: boolean) {
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
      onVisible(!!w && !w.hidden && w.active && !status.user_hidden);
    } else {
      // Browser preview: show the widget over a pretend game window.
      settings.overlay.demo_widget = true;
      target = { game_id: 'hearthstone', game_name: 'Hearthstone', pid: 0 };
      gameWindow = { x: 0, y: 0, width: 1920, height: 1080, hidden: false, active: true };
      onVisible(true);
    }
    syncRegions();
  });
</script>

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
