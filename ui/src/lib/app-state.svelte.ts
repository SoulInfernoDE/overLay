// Launcher state mirrored from the core.

import {
  coreRequest,
  modBaseUrl,
  onHostEvent,
  type GameView,
  type ModList,
  type Request,
  type Settings,
  type Snapshot,
} from './host';
import { applyLanguage } from './i18n.svelte';

export const app = $state<{
  snapshot: Snapshot | null;
  mods: ModList;
  modBase: string;
  error: string | null;
}>({
  snapshot: null,
  mods: { mods: [], errors: [] },
  modBase: 'mod://localhost',
  error: null,
});

export async function initApp(): Promise<void> {
  const snapshot = await coreRequest<Snapshot>({ type: 'snapshot' });
  applyLanguage(snapshot.settings.language);
  app.snapshot = snapshot;
  app.modBase = await modBaseUrl();
  app.mods = await coreRequest<ModList>({ type: 'list-mods' });
  await onHostEvent<GameView[]>('games-changed', (games) => {
    if (app.snapshot) app.snapshot.games = games;
  });
  // Settings can change in another UI process too (e.g. the language).
  await onHostEvent<Settings>('settings-changed', (settings) => {
    if (app.snapshot) app.snapshot.settings = settings;
    applyLanguage(settings.language);
  });
  await onHostEvent<ModList>('mods-changed', (list) => (app.mods = list));
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

/** Sends a mod request (they answer with the new mod list); errors end up in `app.error`. */
export async function updateMods(request: Request): Promise<void> {
  try {
    app.mods = await coreRequest<ModList>(request);
    app.error = null;
  } catch (error) {
    app.error = String(error);
  }
}
