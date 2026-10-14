<script lang="ts">
  import Minus from '@lucide/svelte/icons/minus';
  import X from '@lucide/svelte/icons/x';
  import appIcon from '../../../assets/branding/overlay/icon.svg';
  import { closeWindow, inHost, minimizeWindow } from '../lib/host';
  import { t } from '../lib/i18n.svelte';
</script>

<header class="titlebar" data-tauri-drag-region>
  <img class="brand-icon" src={appIcon} alt="" width="20" height="20" />
  <span class="brand">overLay</span>
  {#if inHost}
    <div class="controls">
      <button
        title={t('shell-window-minimize')}
        aria-label={t('shell-window-minimize')}
        onclick={minimizeWindow}
      >
        <Minus size={16} aria-hidden="true" />
      </button>
      <button
        class="close"
        title={t('shell-window-close')}
        aria-label={t('shell-window-close')}
        onclick={closeWindow}
      >
        <X size={16} aria-hidden="true" />
      </button>
    </div>
  {/if}
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: var(--ol-space-2);
    height: var(--ol-titlebar-height);
    padding-left: var(--ol-space-4);
    background: var(--ol-surface-dock);
    border-bottom: 1px solid var(--ol-border);
  }

  /* Let mouse-downs reach the drag region behind the brand. */
  .brand-icon,
  .brand {
    pointer-events: none;
  }

  .brand {
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--ol-accent);
  }

  .controls {
    display: flex;
    margin-left: auto;
    height: 100%;
  }

  .controls button {
    display: grid;
    place-items: center;
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--ol-text-secondary);
    cursor: pointer;
  }

  .controls button:hover {
    background: var(--ol-surface-hover);
    color: var(--ol-text-primary);
  }

  .controls .close:hover {
    background: var(--ol-danger);
    color: #fff;
  }
</style>
