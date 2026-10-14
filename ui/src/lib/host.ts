// Bridge to the overLay host (Tauri). In a plain browser (`npm run dev` for UI work) it
// falls back to sample data, so the launcher can be designed without the host.

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type Flavor = 'native' | 'wine';

export interface GameView {
  id: string;
  name: string;
  running: Flavor | null;
}

export interface Settings {
  language: string | null;
  start_minimized: boolean;
  autostart: boolean;
  hotkeys: { toggle_launcher: string };
}

export interface Snapshot {
  version: string;
  settings: Settings;
  settings_path: string | null;
  games: GameView[];
}

export const inHost = '__TAURI_INTERNALS__' in window;

const sample: Snapshot = {
  version: __APP_VERSION__,
  settings: {
    language: null,
    start_minimized: false,
    autostart: false,
    hotkeys: { toggle_launcher: 'Ctrl+Shift+O' },
  },
  settings_path: null,
  games: [{ id: 'hearthstone', name: 'Hearthstone', running: null }],
};

export function getSnapshot(): Promise<Snapshot> {
  return inHost ? invoke<Snapshot>('snapshot') : Promise.resolve(structuredClone(sample));
}

function updateSample(change: Partial<Settings>): Promise<Settings> {
  sample.settings = { ...sample.settings, ...change };
  return Promise.resolve(structuredClone(sample.settings));
}

export function setLanguage(language: string | null): Promise<Settings> {
  return inHost ? invoke<Settings>('set_language', { language }) : updateSample({ language });
}

export function setAutostart(enabled: boolean): Promise<Settings> {
  return inHost
    ? invoke<Settings>('set_autostart', { enabled })
    : updateSample({ autostart: enabled });
}

export function setStartMinimized(enabled: boolean): Promise<Settings> {
  return inHost
    ? invoke<Settings>('set_start_minimized', { enabled })
    : updateSample({ start_minimized: enabled });
}

export async function onGamesChanged(callback: (games: GameView[]) => void): Promise<void> {
  if (inHost) await listen<GameView[]>('games-changed', (event) => callback(event.payload));
}

export function minimizeWindow(): void {
  if (inHost) void getCurrentWindow().minimize();
}

export function closeWindow(): void {
  if (inHost) void getCurrentWindow().close();
}
