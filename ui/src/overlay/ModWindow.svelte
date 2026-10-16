<script lang="ts">
  import type { Rect } from '@overlay/sdk/protocol';
  import type { Manifest, ModView } from '../lib/host';
  import type { ModFrameHost } from '../lib/mod-frames';
  import { fitInside } from './fit';
  import { syncRegions } from './regions';

  export interface WindowState {
    x: number;
    y: number;
    /** Clickable parts relative to the window; `null` means the whole window. */
    regions: Rect[] | null;
    clickThrough: boolean;
  }

  let {
    mod,
    windowId,
    spec,
    layout,
    src,
    host,
  }: {
    mod: ModView;
    windowId: string;
    spec: Manifest['windows'][string];
    layout: WindowState;
    src: string;
    host: ModFrameHost;
  } = $props();

  let frame: HTMLIFrameElement | undefined = $state();
  let viewportWidth = $state(window.innerWidth);
  let viewportHeight = $state(window.innerHeight);

  // Shown inside the overlay even if the stored or default position is not.
  const shown = $derived(
    fitInside(layout, spec.width ?? 0, spec.height ?? 0, viewportWidth, viewportHeight),
  );

  $effect(() => {
    if (!frame) return;
    const entry = { frame, mod, window: windowId, kind: 'overlay' as const };
    host.add(entry);
    return () => host.remove(entry);
  });

  $effect(() => {
    void shown.x;
    void shown.y;
    void layout.regions;
    void layout.clickThrough;
    syncRegions();
  });
</script>

<svelte:window bind:innerWidth={viewportWidth} bind:innerHeight={viewportHeight} />

<div
  class="mod-window"
  class:interactive={!layout.clickThrough && !layout.regions}
  style:left="{shown.x}px"
  style:top="{shown.y}px"
  style:width="{spec.width}px"
  style:height="{spec.height}px"
>
  <iframe bind:this={frame} {src} sandbox="allow-scripts" title={mod.manifest.name}></iframe>
  {#if layout.regions && !layout.clickThrough}
    {#each layout.regions as region, index (index)}
      <!-- Only marks the clickable area for the input shape; never takes events itself. -->
      <div
        class="interactive marker"
        style:left="{region.x}px"
        style:top="{region.y}px"
        style:width="{region.width}px"
        style:height="{region.height}px"
      ></div>
    {/each}
  {/if}
</div>

<style>
  .mod-window {
    position: absolute;
  }

  iframe {
    display: block;
    width: 100%;
    height: 100%;
    border: 0;
    background: transparent;
  }

  .marker {
    position: absolute;
    pointer-events: none;
  }
</style>
