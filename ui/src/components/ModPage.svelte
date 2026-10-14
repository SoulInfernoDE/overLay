<script lang="ts">
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import { onDestroy } from 'svelte';
  import { app, updateMods } from '../lib/app-state.svelte';
  import { onHostEvent, type ModEvent, type ModView, type SettingValue } from '../lib/host';
  import { currentLocale, t } from '../lib/i18n.svelte';
  import { ModFrameHost } from '../lib/mod-frames';
  import { loadModTranslator, type ModTranslator } from '../lib/mod-i18n';

  let { mod, onback }: { mod: ModView; onback: () => void } = $props();

  const permissionLabels: Record<string, string> = {
    'game.state': 'permission-game-state',
    'game.events': 'permission-game-events',
    storage: 'permission-storage',
    notifications: 'permission-notifications',
    hotkeys: 'permission-hotkeys',
  };

  let modT: ModTranslator = $state((id: string) => id);
  let frame: HTMLIFrameElement | undefined = $state();

  const desktop = $derived(
    Object.entries(mod.manifest.windows).find(([, spec]) => spec.kind === 'desktop'),
  );

  // Setting labels come from the mod's own catalogs.
  $effect(() => {
    const locale = currentLocale();
    void loadModTranslator(app.modBase, mod.manifest.id, locale).then((translate) => {
      modT = translate;
    });
  });

  const frameHost = new ModFrameHost({
    context: (entry) => ({
      modId: entry.mod.manifest.id,
      window: entry.window,
      kind: 'desktop',
      locale: currentLocale(),
      permissions: entry.mod.manifest.permissions,
      settings: entry.mod.values,
      modBase: app.modBase,
      game: null,
    }),
  });
  void onHostEvent<ModEvent>('mod-event', (event) =>
    frameHost.emit(event.name, event.data, { modId: event.mod_id, permission: event.permission }),
  );
  onDestroy(() => frameHost.dispose());

  $effect(() => {
    if (!frame || !desktop) return;
    const entry = { frame, mod, window: desktop[0], kind: 'desktop' as const };
    frameHost.add(entry);
    return () => frameHost.remove(entry);
  });

  $effect(() => {
    frameHost.emit('locale', currentLocale());
  });

  function setValue(setting: string, value: SettingValue) {
    void updateMods({ type: 'set-mod-setting', mod_id: mod.manifest.id, setting, value });
  }
</script>

<button class="back" onclick={onback}>
  <ArrowLeft size={16} aria-hidden="true" />
  {t('mod-back')}
</button>

<h1>{mod.manifest.name}</h1>
<p class="meta">
  {t('library-mod-by', { version: mod.manifest.version, author: mod.manifest.author })} ·
  {t('mod-license', { license: mod.manifest.license })}
</p>
{#if mod.description}
  <p>{mod.description}</p>
{/if}

{#if desktop}
  <iframe
    bind:this={frame}
    class="desktop"
    sandbox="allow-scripts"
    title={mod.manifest.name}
    src={`${app.modBase}/${mod.manifest.id}/${desktop[1].entry}`}
  ></iframe>
{/if}

{#if mod.manifest.settings.length > 0}
  <section>
    <h2>{t('mod-settings')}</h2>
    {#each mod.manifest.settings as spec (spec.id)}
      {@const value = mod.values[spec.id]}
      <label class="field">
        <span>{modT(spec.label)}</span>
        {#if spec.type === 'boolean'}
          <input
            type="checkbox"
            checked={value === true}
            onchange={(e) => setValue(spec.id, e.currentTarget.checked)}
          />
        {:else if spec.type === 'number'}
          <input
            type="number"
            value={Number(value)}
            min={spec.min ?? undefined}
            max={spec.max ?? undefined}
            onchange={(e) => setValue(spec.id, Number(e.currentTarget.value))}
          />
        {:else if spec.type === 'choice'}
          <select value={String(value)} onchange={(e) => setValue(spec.id, e.currentTarget.value)}>
            {#each spec.options as option (option)}
              <option value={option}>{modT(option)}</option>
            {/each}
          </select>
        {:else}
          <input
            type="text"
            value={String(value)}
            onchange={(e) => setValue(spec.id, e.currentTarget.value)}
          />
        {/if}
      </label>
    {/each}
  </section>
{/if}

<section>
  <h2>{t('mod-permissions')}</h2>
  {#if mod.manifest.permissions.length === 0}
    <p class="muted">{t('mod-permissions-none')}</p>
  {:else}
    <ul>
      {#each mod.manifest.permissions as permission (permission)}
        <li>{t(permissionLabels[permission] ?? permission)}</li>
      {/each}
    </ul>
  {/if}
  <h2>{t('mod-network')}</h2>
  {#if mod.manifest.network.length === 0}
    <p class="muted">{t('mod-network-none')}</p>
  {:else}
    <ul>
      {#each mod.manifest.network as host (host)}
        <li><code>https://{host}</code></li>
      {/each}
    </ul>
  {/if}
  <h2>{t('mod-source')}</h2>
  <p><code class="select">{mod.manifest.source}</code></p>
  <p class="muted">{t('mod-folder', { path: mod.path })}</p>
</section>

<style>
  .back {
    display: inline-flex;
    gap: var(--ol-space-2);
    align-items: center;
    margin-bottom: var(--ol-space-4);
    padding: var(--ol-space-1) var(--ol-space-3);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-raised);
    cursor: pointer;
  }

  .meta,
  .muted {
    color: var(--ol-text-muted);
    font-size: var(--ol-font-size-sm);
  }

  .desktop {
    display: block;
    width: 100%;
    height: 320px;
    margin: var(--ol-space-4) 0;
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-md);
    background: var(--ol-surface-raised);
  }

  section {
    max-width: 720px;
    margin-bottom: var(--ol-space-6);
  }

  h2 {
    margin: var(--ol-space-4) 0 var(--ol-space-2);
    font-size: var(--ol-font-size-sm);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ol-text-muted);
  }

  ul {
    margin: 0;
    padding-left: var(--ol-space-6);
  }

  .field {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--ol-space-4);
    margin-bottom: var(--ol-space-2);
  }

  input[type='number'],
  input[type='text'],
  select {
    padding: var(--ol-space-1) var(--ol-space-2);
    border: 1px solid var(--ol-border);
    border-radius: var(--ol-radius-sm);
    background: var(--ol-surface-raised);
    color: inherit;
    font: inherit;
  }

  input[type='checkbox'] {
    accent-color: var(--ol-accent);
  }

  .select {
    user-select: text;
  }
</style>
