<script lang="ts">
  import { onMount } from 'svelte';
  import Dock from './components/Dock.svelte';
  import LibraryView from './components/LibraryView.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import StoreView from './components/StoreView.svelte';
  import TitleBar from './components/TitleBar.svelte';
  import { app, initApp } from './lib/app-state.svelte';
  import { t } from './lib/i18n.svelte';
  import type { SectionId } from './lib/sections';

  let active: SectionId = $state('library');

  onMount(() => {
    void initApp();
  });
</script>

<div class="shell">
  <Dock bind:active />
  <div class="main">
    <TitleBar />
    <main class="content">
      {#if active === 'library'}
        <LibraryView navigate={(id) => (active = id)} />
      {:else if active === 'store'}
        <StoreView />
      {:else}
        <SettingsView />
      {/if}
    </main>
    <footer class="status">
      {t('status-version', { version: app.snapshot?.version ?? __APP_VERSION__ })}
    </footer>
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
