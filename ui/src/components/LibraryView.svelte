<script lang="ts">
  import { t } from '../lib/i18n.svelte';
  import type { SectionId } from '../lib/sections';

  let { navigate }: { navigate: (id: SectionId) => void } = $props();
</script>

<h1>{t('library-title')}</h1>
<div class="empty">
  <p>{t('library-empty')}</p>
  <button class="primary" onclick={() => navigate('store')}>{t('library-empty-action')}</button>
</div>

<style>
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
