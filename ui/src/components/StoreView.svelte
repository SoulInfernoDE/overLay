<script lang="ts">
  import { t } from '../lib/i18n.svelte';
</script>

<h1>{t('store-title')}</h1>
<p class="muted">{t('store-placeholder')}</p>

<style>
  .muted {
    color: var(--ol-text-secondary);
  }
</style>
