<script lang="ts">
  import { app, updateSettings } from '../lib/app-state.svelte';
  import { applyLanguage, availableLocales, localeName, t } from '../lib/i18n.svelte';

  const settings = $derived(app.snapshot?.settings);

  function changeLanguage(value: string) {
    const language = value === '' ? null : value;
    applyLanguage(language);
    void updateSettings({ type: 'set-language', language });
  }
</script>

<h1>{t('settings-title')}</h1>

{#if app.error}
  <p class="error" role="alert">{t('settings-save-failed', { reason: app.error })}</p>
{/if}

{#if settings}
  <section>
    <h2>{t('settings-general')}</h2>
    <label class="field">
      <span>{t('settings-language')}</span>
      <select
        value={settings.language ?? ''}
        onchange={(event) => changeLanguage(event.currentTarget.value)}
      >
        <option value="">{t('settings-language-system')}</option>
        {#each availableLocales as locale (locale)}
          <option value={locale}>{localeName(locale)}</option>
        {/each}
      </select>
    </label>
    <label class="check">
      <input
        type="checkbox"
        checked={settings.autostart}
        onchange={(event) =>
          updateSettings({ type: 'set-autostart', enabled: event.currentTarget.checked })}
      />
      <span>{t('settings-autostart')}</span>
    </label>
    <label class="check">
      <input
        type="checkbox"
        checked={settings.start_minimized}
        onchange={(event) =>
          updateSettings({ type: 'set-start-minimized', enabled: event.currentTarget.checked })}
      />
      <span>{t('settings-start-minimized')}</span>
    </label>
  </section>

  <section>
    <h2>{t('settings-overlay')}</h2>
    <label class="check">
      <input
        type="checkbox"
        checked={settings.overlay.enabled}
        onchange={(event) =>
          updateSettings({ type: 'set-overlay-enabled', enabled: event.currentTarget.checked })}
      />
      <span>{t('settings-overlay-enabled')}</span>
    </label>
    <label class="check">
      <input
        type="checkbox"
        checked={settings.overlay.demo_widget}
        onchange={(event) =>
          updateSettings({ type: 'set-demo-widget', enabled: event.currentTarget.checked })}
      />
      <span>{t('settings-overlay-demo')}</span>
    </label>
  </section>

  <section>
    <h2>{t('settings-hotkeys')}</h2>
    <div class="field">
      <span>{t('settings-hotkey-launcher')}</span>
      <kbd>{settings.hotkeys.toggle_launcher}</kbd>
    </div>
    <div class="field">
      <span>{t('settings-hotkey-overlay')}</span>
      <kbd>{settings.hotkeys.toggle_overlay}</kbd>
    </div>
    {#if app.snapshot?.settings_path}
      <p class="hint">{t('settings-hotkey-hint', { path: app.snapshot.settings_path })}</p>
    {/if}
  </section>
{/if}

<style>
  section {
    display: grid;
    gap: var(--ol-space-3);
    max-width: 640px;
    margin-bottom: var(--ol-space-6);
  }

  h2 {
    margin: 0 0 var(--ol-space-1);
    font-size: var(--ol-font-size-sm);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ol-text-muted);
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--ol-space-4);
  }

  .check {
    display: flex;
    align-items: center;
    gap: var(--ol-space-2);
    cursor: pointer;
  }

  input[type='checkbox'] {
    accent-color: var(--ol-accent);
    width: 16px;
    height: 16px;
  }

  select {
    padding: var(--ol-space-1) var(--ol-space-2);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-raised);
  }

  kbd {
    padding: 2px var(--ol-space-2);
    border: 1px solid var(--ol-border);
    border-bottom-width: 2px;
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-raised);
    font-family: inherit;
    font-size: var(--ol-font-size-sm);
  }

  .hint {
    margin: 0;
    font-size: var(--ol-font-size-sm);
    color: var(--ol-text-muted);
    word-break: break-all;
  }

  .error {
    padding: var(--ol-space-2) var(--ol-space-3);
    border-left: 3px solid var(--ol-danger);
    background: var(--ol-surface-raised);
  }
</style>
