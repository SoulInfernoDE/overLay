import type { Component } from 'svelte';
import LayoutGrid from '@lucide/svelte/icons/layout-grid';
import Store from '@lucide/svelte/icons/store';
import Settings from '@lucide/svelte/icons/settings';

export type SectionId = 'library' | 'store' | 'settings';

export interface Section {
  id: SectionId;
  /** Fluent message ID of the dock label. */
  label: string;
  icon: Component;
  /** Pinned to the bottom of the dock. */
  bottom?: boolean;
}

export const sections: readonly Section[] = [
  { id: 'library', label: 'nav-library', icon: LayoutGrid },
  { id: 'store', label: 'nav-store', icon: Store },
  { id: 'settings', label: 'nav-settings', icon: Settings, bottom: true },
];
