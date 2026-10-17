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
  developer: { enabled: boolean; mod_paths: string[] };
  mods: Record<string, { enabled: boolean; values: Record<string, SettingValue> }>;
}

export type SettingValue = boolean | number | string;

/** Mirrors `overlay_mods::Manifest`. */
export interface Manifest {
  id: string;
  version: string;
  name: string;
  description: string;
  author: string;
  license: string;
  source: string;
  games: string[];
  permissions: string[];
  network: string[];
  windows: Record<
    string,
    {
      kind: 'overlay' | 'desktop';
      entry: string;
      width?: number | null;
      height?: number | null;
      position?: { x: number; y: number } | null;
      click_through: boolean;
    }
  >;
  hotkeys: Record<string, { default: string; label: string }>;
  settings: SettingSpec[];
}

export type SettingSpec = { id: string; label: string } & (
  | { type: 'boolean'; default: boolean }
  | { type: 'number'; default: number; min?: number | null; max?: number | null }
  | { type: 'text'; default: string }
  | { type: 'choice'; default: string; options: string[] }
);

/** Mirrors `overlay_ipc::ModView`. */
export interface ModView {
  manifest: Manifest;
  origin: 'installed' | 'developer';
  enabled: boolean;
  path: string;
  description: string;
  values: Record<string, SettingValue>;
}

export interface ModList {
  mods: ModView[];
  errors: string[];
}

/** Mirrors `overlay_ipc::ModCall`. */
export type ModCall =
  | { method: 'storage-get'; key: string }
  | { method: 'storage-set'; key: string; value: unknown }
  | { method: 'storage-remove'; key: string }
  | { method: 'storage-keys' }
  | { method: 'notify'; text: string }
  | { method: 'bus-post'; message: unknown }
  | { method: 'game-info' }
  | { method: 'game-entities' }
  | { method: 'game-history' };

/** Mirrors `overlay_ipc::CoreMessage::ModEvent`. */
export interface ModEvent {
  mod_id: string | null;
  name: string;
  data: unknown;
  permission: string | null;
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
    }
  | { type: 'take-toasts' }
  | { type: 'list-mods' }
  | { type: 'reload-mods' }
  | { type: 'set-mod-enabled'; mod_id: string; enabled: boolean }
  | { type: 'set-mod-setting'; mod_id: string; setting: string; value: SettingValue }
  | { type: 'set-developer-mode'; enabled: boolean }
  | { type: 'add-developer-mod-path'; path: string }
  | { type: 'remove-developer-mod-path'; path: string }
  | { type: 'mod-call'; mod_id: string; call: ModCall };

export const inHost = '__TAURI_INTERNALS__' in window;

const sample: Snapshot = {
  version: __APP_VERSION__,
  settings: {
    language: null,
    start_minimized: false,
    autostart: false,
    hotkeys: { toggle_launcher: 'Ctrl+Shift+O', toggle_overlay: 'Ctrl+Shift+L' },
    overlay: { enabled: true, stacking: 'above', demo_widget: false },
    developer: { enabled: false, mod_paths: [] },
    mods: {},
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
    case 'mod-call':
      return null;
    case 'take-toasts':
      return [];
    case 'list-mods':
    case 'reload-mods':
    case 'set-mod-enabled':
    case 'set-mod-setting':
      return { mods: [], errors: [] } satisfies ModList;
    case 'set-developer-mode':
      settings.developer.enabled = request.enabled;
      return { mods: [], errors: [] } satisfies ModList;
    case 'add-developer-mod-path':
      settings.developer.mod_paths.push(request.path);
      return { mods: [], errors: [] } satisfies ModList;
    case 'remove-developer-mod-path':
      settings.developer.mod_paths = settings.developer.mod_paths.filter((p) => p !== request.path);
      return { mods: [], errors: [] } satisfies ModList;
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

/** Origin of mod frames (`mod://localhost`, or `http://mod.localhost` on Windows). */
export async function modBaseUrl(): Promise<string> {
  return inHost ? invoke<string>('mod_base_url') : 'mod://localhost';
}

export function minimizeWindow(): void {
  if (inHost) void getCurrentWindow().minimize();
}

export function closeWindow(): void {
  if (inHost) void getCurrentWindow().close();
}
