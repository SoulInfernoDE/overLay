<script lang="ts">
  import { availableLocales, currentLocale, localeName, setLocale, t } from '../lib/i18n.svelte';
</script>

<h1>{t('settings-title')}</h1>
<label class="field">
  <span>{t('settings-language')}</span>
  <select value={currentLocale()} onchange={(event) => setLocale(event.currentTarget.value)}>
    {#each availableLocales as locale (locale)}
      <option value={locale}>{localeName(locale)}</option>
    {/each}
  </select>
</label>
<p class="muted">{t('settings-placeholder')}</p>

<style>
  .field {
    display: flex;
    align-items: center;
    gap: var(--ol-space-4);
  }

  select {
    padding: var(--ol-space-1) var(--ol-space-2);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-raised);
  }

  .muted {
    color: var(--ol-text-secondary);
  }
</style>
