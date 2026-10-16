// Hosts mod frames: answers their SDK calls and delivers events to them.
//
// Each frame is a sandboxed iframe with an opaque origin. Messages are matched to the
// frame that sent them through `event.source`, so a mod cannot speak for another one.
// Calls that touch data go to the core, which checks the mod's permissions; events are
// delivered only to mods that hold the permission they need.

import {
  MARK,
  isOverlayMessage,
  type FrameMessage,
  type HostMessage,
  type ModContext,
} from '@overlay/sdk/protocol';
import { coreRequest, type ModCall, type ModView } from './host';

export interface FrameEntry {
  frame: HTMLIFrameElement;
  mod: ModView;
  window: string;
  kind: 'overlay' | 'desktop';
}

/** Handles a method locally (e.g. window dragging) or returns `undefined`. */
export type LocalHandler = (
  entry: FrameEntry,
  method: string,
  params: Record<string, unknown>,
) => Promise<unknown> | undefined;

export class ModFrameHost {
  private entries = new Set<FrameEntry>();

  constructor(
    private readonly options: {
      context: (entry: FrameEntry) => ModContext;
      local?: LocalHandler;
    },
  ) {
    window.addEventListener('message', this.onMessage);
  }

  /** Stops listening; call when the page that owns the frames goes away. */
  dispose(): void {
    window.removeEventListener('message', this.onMessage);
    this.entries.clear();
  }

  add(entry: FrameEntry): void {
    this.entries.add(entry);
  }

  remove(entry: FrameEntry): void {
    this.entries.delete(entry);
  }

  /** Sends an event to the frames of `modId` (all mods if omitted) that hold `permission`. */
  emit(
    name: string,
    data: unknown,
    filter: { modId?: string | null; permission?: string | null } = {},
  ) {
    for (const entry of this.entries) {
      if (filter.modId && entry.mod.manifest.id !== filter.modId) continue;
      if (filter.permission && !entry.mod.manifest.permissions.includes(filter.permission))
        continue;
      this.send(entry, { [MARK]: 1, kind: 'event', name, data });
    }
  }

  /** Sends an event computed per frame (e.g. a context that depends on the mod). */
  emitEach(name: string, data: (entry: FrameEntry) => unknown | undefined) {
    for (const entry of this.entries) {
      const value = data(entry);
      if (value !== undefined) this.send(entry, { [MARK]: 1, kind: 'event', name, data: value });
    }
  }

  private send(entry: FrameEntry, message: HostMessage) {
    // Messages often contain Svelte state proxies, which postMessage cannot clone; all
    // messages are JSON by design, so a JSON round trip gives plain objects.
    const plain = JSON.parse(JSON.stringify(message)) as HostMessage;
    try {
      entry.frame.contentWindow?.postMessage(plain, '*');
    } catch (error) {
      console.error('[overlay] cannot reach mod frame:', error);
    }
  }

  private onMessage = (event: MessageEvent) => {
    if (!isOverlayMessage(event.data)) return;
    const entry = [...this.entries].find((e) => e.frame.contentWindow === event.source);
    if (!entry) return;
    const message = event.data as FrameMessage;
    if (message.kind === 'hello') {
      this.send(entry, { [MARK]: 1, kind: 'init', context: this.options.context(entry) });
      return;
    }
    if (message.kind !== 'call') return;
    const params = (message.params ?? {}) as Record<string, unknown>;
    this.handle(entry, message.method, params).then(
      (value) => this.send(entry, { [MARK]: 1, kind: 'result', id: message.id, ok: true, value }),
      (error: unknown) =>
        this.send(entry, {
          [MARK]: 1,
          kind: 'result',
          id: message.id,
          ok: false,
          error: String(error),
        }),
    );
  };

  private handle(entry: FrameEntry, method: string, params: Record<string, unknown>) {
    const local = this.options.local?.(entry, method, params);
    if (local) return local;
    const core = (call: ModCall) =>
      coreRequest({ type: 'mod-call', mod_id: entry.mod.manifest.id, call });
    switch (method) {
      case 'storage.get':
        return core({ method: 'storage-get', key: String(params.key) });
      case 'storage.set':
        return core({ method: 'storage-set', key: String(params.key), value: params.value });
      case 'storage.remove':
        return core({ method: 'storage-remove', key: String(params.key) });
      case 'storage.keys':
        return core({ method: 'storage-keys' });
      case 'notify':
        return core({ method: 'notify', text: String(params.text) });
      case 'bus.post':
        return core({ method: 'bus-post', message: params.message });
      case 'events.info':
        return core({ method: 'game-info' });
      case 'events.entities':
        return core({ method: 'game-entities' });
      default:
        return Promise.reject(new Error(`${method} is not available in this window`));
    }
  }
}
