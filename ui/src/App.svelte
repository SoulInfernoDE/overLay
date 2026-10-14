<script lang="ts">
  import Dock from './components/Dock.svelte';
  import LibraryView from './components/LibraryView.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import StoreView from './components/StoreView.svelte';
  import { t } from './lib/i18n.svelte';
  import type { SectionId } from './lib/sections';

  let active: SectionId = $state('library');
</script>

<div class="shell">
  <Dock bind:active />
  <div class="main">
    <header class="titlebar">
      <span class="brand">overLay</span>
    </header>
    <main class="content">
      {#if active === 'library'}
        <LibraryView navigate={(id) => (active = id)} />
      {:else if active === 'store'}
        <StoreView />
      {:else}
        <SettingsView />
      {/if}
    </main>
    <footer class="status">{t('status-version', { version: __APP_VERSION__ })}</footer>
  </div>
</div>

<style>
  .shell {
    display: flex;
    height: 100%;
  }

  .main {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .titlebar {
    display: flex;
    align-items: center;
    height: var(--ol-titlebar-height);
    padding: 0 var(--ol-space-4);
    background: var(--ol-surface-dock);
    border-bottom: 1px solid var(--ol-border);
  }

  .brand {
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--ol-accent);
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: var(--ol-space-6) var(--ol-space-8);
  }

  .content :global(h1) {
    margin: 0 0 var(--ol-space-6);
    font-size: var(--ol-font-size-xl);
    font-weight: 600;
  }

  .status {
    padding: var(--ol-space-1) var(--ol-space-4);
    border-top: 1px solid var(--ol-border);
    color: var(--ol-text-muted);
    font-size: var(--ol-font-size-sm);
  }
</style>
