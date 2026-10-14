// Launcher state mirrored from the core.

import {
  coreRequest,
  onHostEvent,
  type GameView,
  type Request,
  type Settings,
  type Snapshot,
} from './host';
import { applyLanguage } from './i18n.svelte';

export const app = $state<{ snapshot: Snapshot | null; error: string | null }>({
  snapshot: null,
  error: null,
});

export async function initApp(): Promise<void> {
  const snapshot = await coreRequest<Snapshot>({ type: 'snapshot' });
  applyLanguage(snapshot.settings.language);
  app.snapshot = snapshot;
  await onHostEvent<GameView[]>('games-changed', (games) => {
    if (app.snapshot) app.snapshot.games = games;
  });
  // Settings can change in another UI process too (e.g. the language).
  await onHostEvent<Settings>('settings-changed', (settings) => {
    if (app.snapshot) app.snapshot.settings = settings;
    applyLanguage(settings.language);
  });
}

/** Sends a settings request and keeps the local copy in sync; errors end up in `app.error`. */
export async function updateSettings(request: Request): Promise<void> {
  try {
    const settings = await coreRequest<Settings>(request);
    if (app.snapshot) app.snapshot.settings = settings;
    app.error = null;
  } catch (error) {
    app.error = String(error);
  }
}
