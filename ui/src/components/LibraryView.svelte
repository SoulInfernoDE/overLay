<script lang="ts">
  import Gamepad from '@lucide/svelte/icons/gamepad-2';
  import { app } from '../lib/app-state.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { SectionId } from '../lib/sections';

  let { navigate }: { navigate: (id: SectionId) => void } = $props();
</script>

<h1>{t('library-title')}</h1>

<h2>{t('library-games-title')}</h2>
<ul class="games">
  {#each app.snapshot?.games ?? [] as game (game.id)}
    <li class="game" class:running={game.running}>
      <Gamepad size={28} strokeWidth={1.5} aria-hidden="true" />
      <span class="name">{game.name}</span>
      <span class="status">
        {game.running ? t('game-status-running', { flavor: game.running }) : t('game-status-idle')}
      </span>
    </li>
  {/each}
</ul>

<h2>{t('library-mods-title')}</h2>
<div class="empty">
  <p>{t('library-empty')}</p>
  <button class="primary" onclick={() => navigate('store')}>{t('library-empty-action')}</button>
</div>

<style>
  h2 {
    margin: var(--ol-space-6) 0 var(--ol-space-3);
    font-size: var(--ol-font-size-sm);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ol-text-muted);
  }

  .games {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--ol-space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .game {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    column-gap: var(--ol-space-3);
    align-items: center;
    padding: var(--ol-space-4);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-md);
    background: var(--ol-surface-raised);
    color: var(--ol-text-secondary);
  }

  .game :global(svg) {
    grid-row: span 2;
  }

  .name {
    font-weight: 600;
    color: var(--ol-text-primary);
  }

  .status {
    font-size: var(--ol-font-size-sm);
  }

  .game.running {
    border-color: var(--ol-accent);
  }

  .game.running :global(svg),
  .game.running .status {
    color: var(--ol-accent);
  }

  .empty {
    display: grid;
    justify-items: center;
    gap: var(--ol-space-4);
    padding: var(--ol-space-8);
    border: 1px dashed var(--ol-border);
    border-radius: var(--ol-radius-lg);
    color: var(--ol-text-secondary);
  }

  .primary {
    padding: var(--ol-space-2) var(--ol-space-4);
    border: none;
    border-radius: var(--ol-radius-sm);
    background: var(--ol-accent);
    color: var(--ol-accent-contrast);
    font-weight: 600;
    cursor: pointer;
  }

  .primary:hover {
    background: var(--ol-accent-strong);
  }
</style>
