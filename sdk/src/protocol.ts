// Messages between a mod frame and the overLay page that hosts it (window.postMessage).
// Mod frames run sandboxed with an opaque origin; this is their only channel to the host.

export const MARK = '__overlay';

export type SettingValue = boolean | number | string;

/** The game the overlay runs over. */
export interface GameState {
  gameId: string;
  gameName: string;
  /** Size of the game window in physical pixels, or null while it has none. */
  window: { width: number; height: number } | null;
}

/** What a mod window learns when it connects. */
export interface ModContext {
  modId: string;
  /** Window id from the manifest. */
  window: string;
  kind: 'overlay' | 'desktop';
  /** Current UI language, e.g. `de`. */
  locale: string;
  permissions: string[];
  settings: Record<string, SettingValue>;
  /** Origin of mod files, e.g. `mod://localhost`. */
  modBase: string;
  /** Only with the `game.state` permission, and only in overlay windows. */
  game: GameState | null;
}

export type FrameMessage =
  | { [MARK]: 1; kind: 'hello' }
  | { [MARK]: 1; kind: 'call'; id: number; method: string; params?: unknown };

export type HostMessage =
  | { [MARK]: 1; kind: 'init'; context: ModContext }
  | { [MARK]: 1; kind: 'result'; id: number; ok: true; value: unknown }
  | { [MARK]: 1; kind: 'result'; id: number; ok: false; error: string }
  | { [MARK]: 1; kind: 'event'; name: string; data: unknown };

export function isOverlayMessage(data: unknown): data is FrameMessage | HostMessage {
  return typeof data === 'object' && data !== null && (data as Record<string, unknown>)[MARK] === 1;
}

/** A rectangle in CSS pixels, relative to the mod window. */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}
