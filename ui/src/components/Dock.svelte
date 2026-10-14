<script lang="ts">
  import { t } from '../lib/i18n.svelte';
  import { sections, type SectionId } from '../lib/sections';

  let { active = $bindable() }: { active: SectionId } = $props();
</script>

<nav class="dock" aria-label={t('shell-dock-label')}>
  {#each sections as section (section.id)}
    <button
      class="dock-item"
      class:bottom={section.bottom}
      aria-current={active === section.id ? 'page' : undefined}
      title={t(section.label)}
      onclick={() => (active = section.id)}
    >
      <section.icon size={22} strokeWidth={1.75} aria-hidden="true" />
      <span class="visually-hidden">{t(section.label)}</span>
    </button>
  {/each}
</nav>

<style>
  .dock {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--ol-space-2);
    width: var(--ol-dock-width);
    padding: var(--ol-space-3) 0;
    background: var(--ol-surface-dock);
    border-right: 1px solid var(--ol-border);
  }

  .dock-item {
    position: relative;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: none;
    border-radius: var(--ol-radius-md);
    background: transparent;
    color: var(--ol-text-secondary);
    cursor: pointer;
    transition:
      background var(--ol-duration-fast) var(--ol-easing),
      color var(--ol-duration-fast) var(--ol-easing);
  }

  .dock-item:hover {
    background: var(--ol-surface-hover);
    color: var(--ol-text-primary);
  }

  .dock-item[aria-current='page'] {
    color: var(--ol-accent);
  }

  .dock-item[aria-current='page']::before {
    content: '';
    position: absolute;
    left: -10px;
    width: 3px;
    height: 24px;
    border-radius: 0 3px 3px 0;
    background: var(--ol-accent);
  }

  .bottom {
    margin-top: auto;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
