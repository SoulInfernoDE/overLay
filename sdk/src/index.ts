// overLay mod SDK. Import it in a mod window and wait for `overlay.ready()`:
//
//   import { overlay } from '@overlay/sdk';
//   const context = await overlay.ready();
//   document.body.textContent = overlay.i18n.t('hello', { name: 'World' });
//
// Everything goes through the host page via postMessage; the host checks the mod's
// permissions (overlay.json) for every call.

import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';
import { negotiateLanguages } from '@fluent/langneg';
import {
  MARK,
  isOverlayMessage,
  type FrameMessage,
  type GameState,
  type HostMessage,
  type ModContext,
  type Rect,
  type SettingValue,
} from './protocol';

export type { GameState, ModContext, Rect, SettingValue } from './protocol';
export type * as Hearthstone from './hearthstone';
import type {
  Entity,
  GameEvent as HearthstoneEvent,
  GameInfo,
  ProviderMessage,
} from './hearthstone';

type Listener = (data: unknown) => void;

// Mod pages sit on overLay's dark surfaces; without this, WebKit paints frames on a
// light canvas with light form controls.
document.documentElement.style.colorScheme = 'dark';

const pending = new Map<number, { resolve: (v: unknown) => void; reject: (e: Error) => void }>();
const listeners = new Map<string, Set<Listener>>();
let nextId = 1;
let context: ModContext | null = null;
let bundles: FluentBundle[] = [];
let readyPromise: Promise<ModContext> | null = null;

// Mirror of all game entities, kept up to date from the provider's batches.
const entities = new Map<number, Entity>();
// Batches that arrive while the initial snapshot is loading.
let seeding = false;
let buffered: ProviderMessage[][] = [];

function applyBatch(batch: ProviderMessage[]): void {
  const changed = new Set<number>();
  for (const message of batch) {
    if (message.name === 'entity-update') {
      const { entity_id, card_id, tags } = message.data;
      const entity = entities.get(entity_id) ?? { entity_id, card_id: null, tags: {} };
      if (card_id) entity.card_id = card_id;
      Object.assign(entity.tags, tags);
      entities.set(entity_id, entity);
      changed.add(entity_id);
      continue;
    }
    if (message.name === 'game-created') entities.clear();
    // High-level events see the mirror as it was at that point of the game.
    emit('game-event', message);
  }
  if (changed.size > 0) emit('entities', [...changed]);
}

async function seedEntities(): Promise<void> {
  seeding = true;
  try {
    const snapshot = await call<Entity[] | null>('events.entities');
    entities.clear();
    for (const entity of snapshot ?? []) entities.set(entity.entity_id, entity);
  } catch (error) {
    console.error('[overlay-sdk] cannot load the game entities:', error);
  } finally {
    seeding = false;
    for (const batch of buffered.splice(0)) applyBatch(batch);
  }
}

function post(message: FrameMessage): void {
  try {
    window.parent.postMessage(message, '*');
  } catch (error) {
    // Reactive state of UI frameworks (e.g. Svelte or Vue proxies) cannot be cloned
    // for postMessage; a plain JSON copy carries the same data.
    if (!(error instanceof DOMException && error.name === 'DataCloneError')) throw error;
    window.parent.postMessage(JSON.parse(JSON.stringify(message)) as FrameMessage, '*');
  }
}

function call<T>(method: string, params?: unknown): Promise<T> {
  const id = nextId++;
  return new Promise<T>((resolve, reject) => {
    pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
    post({ [MARK]: 1, kind: 'call', id, method, params });
  });
}

function on(name: string, listener: Listener): () => void {
  const set = listeners.get(name) ?? new Set<Listener>();
  set.add(listener);
  listeners.set(name, set);
  return () => set.delete(listener);
}

function emit(name: string, data: unknown): void {
  for (const listener of listeners.get(name) ?? []) listener(data);
}

window.addEventListener('message', (event) => {
  // Only the host page may talk to us.
  if (event.source !== window.parent || !isOverlayMessage(event.data)) return;
  const message = event.data as HostMessage;
  switch (message.kind) {
    case 'result': {
      const waiting = pending.get(message.id);
      pending.delete(message.id);
      if (!waiting) return;
      if (message.ok) waiting.resolve(message.value);
      else waiting.reject(new Error(message.error));
      break;
    }
    case 'event':
      if (message.name === 'locale' && context) {
        context.locale = String(message.data);
        void loadCatalogs(context).then(() => emit('locale', message.data));
        return;
      }
      if (message.name === 'settings' && context) {
        context.settings = message.data as Record<string, SettingValue>;
      }
      if (message.name === 'game-state' && context) {
        context.game = message.data as GameState | null;
      }
      if (message.name === 'game-events') {
        const batch = message.data as ProviderMessage[];
        if (seeding) buffered.push(batch);
        else applyBatch(batch);
        return;
      }
      emit(message.name, message.data);
      break;
    case 'init':
      break;
  }
});

/** Loads the mod's own Fluent catalogs (`locales/<locale>/*.ftl`), English last. */
async function loadCatalogs(ctx: ModContext): Promise<void> {
  const root = `${ctx.modBase}/${ctx.modId}`;
  try {
    const listing = (await (await fetch(`${root}/.overlay/locales.json`)).json()) as Record<
      string,
      string[]
    >;
    const available = Object.keys(listing);
    const order = negotiateLanguages([ctx.locale], available, {
      defaultLocale: 'en',
      strategy: 'filtering',
    }).filter((l) => l !== 'en');
    if (available.includes('en')) order.push('en');
    bundles = await Promise.all(
      order.map(async (locale) => {
        const bundle = new FluentBundle(locale);
        for (const file of listing[locale] ?? []) {
          const source = await (await fetch(`${root}/locales/${locale}/${file}`)).text();
          for (const error of bundle.addResource(new FluentResource(source))) {
            console.error(`[overlay-sdk] ${locale}/${file}:`, error);
          }
        }
        return bundle;
      }),
    );
  } catch (error) {
    console.error('[overlay-sdk] cannot load catalogs:', error);
    bundles = [];
  }
}

export const overlay = {
  /** Connects to the host; resolves once context and catalogs are loaded. */
  ready(): Promise<ModContext> {
    readyPromise ??= new Promise<ModContext>((resolve) => {
      const onInit = async (event: MessageEvent) => {
        if (event.source !== window.parent || !isOverlayMessage(event.data)) return;
        const message = event.data as HostMessage;
        if (message.kind !== 'init') return;
        window.removeEventListener('message', onInit);
        context = message.context;
        await loadCatalogs(context);
        if (context.permissions.includes('game.events')) await seedEntities();
        resolve(context);
      };
      window.addEventListener('message', onInit);
      post({ [MARK]: 1, kind: 'hello' });
    });
    return readyPromise;
  },

  i18n: {
    /** Formats a message from the mod's catalogs; returns the id if no catalog has it. */
    t(id: string, args?: Record<string, FluentVariable>): string {
      for (const bundle of bundles) {
        const pattern = bundle.getMessage(id)?.value;
        if (pattern) return bundle.formatPattern(pattern, args);
      }
      return id;
    },
    locale(): string {
      return context?.locale ?? 'en';
    },
    /** Called after the user changed the language and the catalogs were reloaded. */
    onChange(listener: (locale: string) => void): () => void {
      return on('locale', (l) => listener(String(l)));
    },
  },

  /** Needs the `storage` permission. Values must be JSON-serializable. */
  storage: {
    async get<T = unknown>(key: string): Promise<T | null> {
      return (await call<T | null>('storage.get', { key })) ?? null;
    },
    set(key: string, value: unknown): Promise<void> {
      return call('storage.set', { key, value });
    },
    remove(key: string): Promise<void> {
      return call('storage.remove', { key });
    },
    keys(): Promise<string[]> {
      return call('storage.keys');
    },
  },

  /** Needs the `notifications` permission; shown as a toast in the overlay. */
  notifications: {
    show(text: string): Promise<void> {
      return call('notify', { text });
    },
  },

  /** Messages between the windows of this mod (e.g. its launcher page and its overlay). */
  bus: {
    post(message: unknown): Promise<void> {
      return call('bus.post', { message });
    },
    onMessage(listener: (message: unknown) => void): () => void {
      return on('bus', listener);
    },
  },

  /** Needs the `hotkeys` permission and the hotkey declared in overlay.json. */
  hotkeys: {
    on(id: string, listener: () => void): () => void {
      return on('hotkey', (data) => {
        if ((data as { id?: string }).id === id) listener();
      });
    },
  },

  /** The mod's own settings, declared in overlay.json and edited in the launcher. */
  settings: {
    get(): Record<string, SettingValue> {
      return { ...(context?.settings ?? {}) };
    },
    onChange(listener: (settings: Record<string, SettingValue>) => void): () => void {
      return on('settings', (data) => listener(data as Record<string, SettingValue>));
    },
  },

  /** Needs the `game.state` permission. */
  game: {
    getState(): GameState | null {
      return context?.game ?? null;
    },
    onChange(listener: (state: GameState | null) => void): () => void {
      return on('game-state', (data) => listener(data as GameState | null));
    },
  },

  /**
   * Live game events from the host's game providers; needs `game.events`. For
   * Hearthstone, events and summary follow the types in `Hearthstone`.
   */
  events: {
    on(listener: (event: HearthstoneEvent) => void): () => void {
      return on('game-event', (data) => listener(data as HearthstoneEvent));
    },
    /** All entities of the running game with all their tags (read-only view). */
    entities(): ReadonlyMap<number, Readonly<Entity>> {
      return entities;
    },
    /** Called after each batch with the ids of the entities that changed. */
    onEntitiesChanged(listener: (entityIds: number[]) => void): () => void {
      return on('entities', (data) => listener(data as number[]));
    },
    /** The current game summary, or null outside a game. */
    getInfo(): Promise<GameInfo | null> {
      return call<GameInfo | null>('events.info');
    },
    /** Called whenever the summary changes (at most ten times a second). */
    onInfo(listener: (info: GameInfo) => void): () => void {
      return on('game-info', (data) => listener(data as GameInfo));
    },
  },

  /** Overlay windows only. */
  windows: {
    /**
     * Lets the user move the window by dragging `handle`. The host saves the position
     * per game and resolution.
     */
    makeDraggable(handle: HTMLElement): void {
      handle.addEventListener('pointerdown', (event) => {
        if ((event.target as Element).closest('button, input, select, textarea, a')) return;
        let last = { x: event.screenX, y: event.screenY };
        const start = { client: event.clientX, screen: event.screenX };
        // Screen coordinates can be device pixels (WebKit with a zoomed page, e.g. 1.2 at
        // 115 dpi) while the window moves in CSS pixels. The first move, before the window
        // has moved, tells the ratio.
        let scale: number | null = null;
        handle.setPointerCapture(event.pointerId);
        const move = (e: PointerEvent) => {
          if (scale === null) {
            const client = e.clientX - start.client;
            if (Math.abs(client) < 3) return;
            const ratio = (e.screenX - start.screen) / client;
            scale =
              Math.abs(ratio - window.devicePixelRatio) < Math.abs(ratio - 1)
                ? window.devicePixelRatio
                : 1;
          }
          const dx = (e.screenX - last.x) / scale;
          const dy = (e.screenY - last.y) / scale;
          last = { x: e.screenX, y: e.screenY };
          if (dx || dy) void call('windows.dragBy', { dx, dy });
        };
        const stop = () => {
          handle.removeEventListener('pointermove', move);
          handle.removeEventListener('pointerup', stop);
          void call('windows.dragEnd');
        };
        handle.addEventListener('pointermove', move);
        handle.addEventListener('pointerup', stop);
      });
    },
    /**
     * Only these parts of the window take mouse input; the rest lets clicks through to
     * the game. `null` makes the whole window clickable again.
     */
    setInteractiveRegions(rects: Rect[] | null): Promise<void> {
      return call('windows.setRegions', { rects });
    },
    /** Lets every click through to the game (for purely visual overlays). */
    setClickThrough(enabled: boolean): Promise<void> {
      return call('windows.setClickThrough', { enabled });
    },
  },
};
