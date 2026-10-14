// Bridge to the overLay core. UI pages run in their own process and reach the core
// through one Tauri command, `core_request`. In a plain browser (`npm run dev` for UI
// work) the bridge falls back to sample data, so pages can be designed without the host.

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
  hotkeys: { toggle_launcher: string; toggle_overlay: string };
  overlay: { enabled: boolean; stacking: 'above' | 'override'; demo_widget: boolean };
}

export interface Snapshot {
  version: string;
  settings: Settings;
  settings_path: string | null;
  games: GameView[];
}

export interface Position {
  x: number;
  y: number;
}

/** Requests understood by the core (mirrors `overlay_ipc::Request`). */
export type Request =
  | { type: 'snapshot' }
  | { type: 'set-language'; language: string | null }
  | { type: 'set-autostart'; enabled: boolean }
  | { type: 'set-start-minimized'; enabled: boolean }
  | { type: 'set-overlay-enabled'; enabled: boolean }
  | { type: 'set-demo-widget'; enabled: boolean }
  | { type: 'load-layout'; game_id: string; resolution: string }
  | {
      type: 'save-widget-position';
      game_id: string;
      resolution: string;
      widget: string;
      position: Position;
    };

export const inHost = '__TAURI_INTERNALS__' in window;

const sample: Snapshot = {
  version: __APP_VERSION__,
  settings: {
    language: null,
    start_minimized: false,
    autostart: false,
    hotkeys: { toggle_launcher: 'Ctrl+Shift+O', toggle_overlay: 'Ctrl+Shift+L' },
    overlay: { enabled: true, stacking: 'above', demo_widget: false },
  },
  settings_path: null,
  games: [{ id: 'hearthstone', name: 'Hearthstone', running: null }],
};

function sampleResponse(request: Request): unknown {
  const settings = sample.settings;
  switch (request.type) {
    case 'snapshot':
      return structuredClone(sample);
    case 'set-language':
      settings.language = request.language;
      break;
    case 'set-autostart':
      settings.autostart = request.enabled;
      break;
    case 'set-start-minimized':
      settings.start_minimized = request.enabled;
      break;
    case 'set-overlay-enabled':
      settings.overlay.enabled = request.enabled;
      break;
    case 'set-demo-widget':
      settings.overlay.demo_widget = request.enabled;
      break;
    case 'load-layout':
      return {};
    case 'save-widget-position':
      return null;
  }
  return structuredClone(settings);
}

export function coreRequest<T>(request: Request): Promise<T> {
  return inHost
    ? invoke<T>('core_request', { request })
    : Promise.resolve(sampleResponse(request) as T);
}

export async function onHostEvent<T>(event: string, callback: (payload: T) => void): Promise<void> {
  if (inHost) await listen<T>(event, (e) => callback(e.payload));
}

export function minimizeWindow(): void {
  if (inHost) void getCurrentWindow().minimize();
}

export function closeWindow(): void {
  if (inHost) void getCurrentWindow().close();
}
