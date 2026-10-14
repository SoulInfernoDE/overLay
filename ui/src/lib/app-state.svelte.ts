// Launcher state mirrored from the host.

import { getSnapshot, onGamesChanged, type Settings, type Snapshot } from './host';
import { applyLanguage } from './i18n.svelte';

export const app = $state<{ snapshot: Snapshot | null; error: string | null }>({
  snapshot: null,
  error: null,
});

export async function initApp(): Promise<void> {
  const snapshot = await getSnapshot();
  applyLanguage(snapshot.settings.language);
  app.snapshot = snapshot;
  await onGamesChanged((games) => {
    if (app.snapshot) app.snapshot.games = games;
  });
}

/** Runs a settings command and keeps the local copy in sync; errors end up in `app.error`. */
export async function updateSettings(command: () => Promise<Settings>): Promise<void> {
  try {
    const settings = await command();
    if (app.snapshot) app.snapshot.settings = settings;
    app.error = null;
  } catch (error) {
    app.error = String(error);
  }
}
