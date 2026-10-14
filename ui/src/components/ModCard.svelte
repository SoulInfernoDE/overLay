<script lang="ts">
  import Puzzle from '@lucide/svelte/icons/puzzle';
  import { updateMods } from '../lib/app-state.svelte';
  import type { ModView } from '../lib/host';
  import { t } from '../lib/i18n.svelte';

  let { mod, onopen }: { mod: ModView; onopen: () => void } = $props();
</script>

<li class="mod" class:disabled={!mod.enabled}>
  <Puzzle size={28} strokeWidth={1.5} aria-hidden="true" />
  <div class="info">
    <div class="name">
      {mod.manifest.name}
      {#if mod.origin === 'developer'}
        <span class="badge">{t('library-mod-developer')}</span>
      {/if}
    </div>
    <div class="meta">
      {t('library-mod-by', { version: mod.manifest.version, author: mod.manifest.author })}
    </div>
    {#if mod.description}
      <p class="description">{mod.description}</p>
    {/if}
  </div>
  <div class="actions">
    <label class="toggle">
      <input
        type="checkbox"
        checked={mod.enabled}
        onchange={(event) =>
          updateMods({
            type: 'set-mod-enabled',
            mod_id: mod.manifest.id,
            enabled: event.currentTarget.checked,
          })}
      />
      <span>{t('library-mod-enabled')}</span>
    </label>
    <button onclick={onopen}>{t('library-mod-open')}</button>
  </div>
</li>

<style>
  .mod {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--ol-space-3);
    align-items: start;
    padding: var(--ol-space-4);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-md);
    background: var(--ol-surface-raised);
    color: var(--ol-text-secondary);
  }

  .mod.disabled {
    opacity: 0.6;
  }

  .name {
    display: flex;
    gap: var(--ol-space-2);
    align-items: center;
    font-weight: 600;
    color: var(--ol-text-primary);
  }

  .badge {
    padding: 0 var(--ol-space-2);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-warning);
    color: #241a05;
    font-size: var(--ol-font-size-sm);
    font-weight: 600;
  }

  .meta {
    font-size: var(--ol-font-size-sm);
  }

  .description {
    margin: var(--ol-space-2) 0 0;
  }

  .actions {
    display: grid;
    gap: var(--ol-space-2);
    justify-items: end;
  }

  .toggle {
    display: flex;
    gap: var(--ol-space-2);
    align-items: center;
    cursor: pointer;
  }

  input[type='checkbox'] {
    accent-color: var(--ol-accent);
  }

  button {
    padding: var(--ol-space-1) var(--ol-space-3);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-hover);
    cursor: pointer;
  }
</style>
