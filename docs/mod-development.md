# Writing overLay mods

> English original. German version: [mod-development.de.md](mod-development.de.md)
> Applies to SDK 0.1 (phase 3). The API may still change before overLay 1.0.

An overLay mod is a small web app: HTML, JavaScript and CSS, plus a manifest (`overlay.json`) and its translations. overLay shows its windows in the game (overlay windows) or in the launcher (desktop windows). The example [`mods/hello-overlay`](../mods/hello-overlay) uses every feature described here.

Before publishing, read the [mod policy](../MOD_POLICY.md): open source, no telemetry, no ads, no remote code.

## Folder layout

The folder overLay loads (for the example: `mods/hello-overlay/dist/` after `npm run build`):

```
overlay.json          manifest
index.html            entry of a window (any name, see "windows")
assets/…              your scripts, styles, images
locales/en/*.ftl      Fluent catalogs, English is required
locales/de/*.ftl      further languages
```

All URLs inside the mod must be relative (with Vite: `base: './'`).

## Manifest (`overlay.json`)

```json
{
  "id": "io.github.you.my-mod",
  "version": "0.1.0",
  "name": "My Mod",
  "description": "mod-description",
  "author": "You",
  "license": "GPL-3.0-or-later",
  "source": "https://github.com/you/my-mod",
  "games": ["hearthstone"],
  "permissions": ["game.state", "storage", "notifications", "hotkeys"],
  "network": ["api.hearthstonejson.com"],
  "windows": {
    "panel": { "kind": "overlay", "entry": "index.html", "width": 300, "height": 210,
               "position": { "x": 40, "y": 320 } },
    "page": { "kind": "desktop", "entry": "page.html" }
  },
  "hotkeys": { "compact": { "default": "Ctrl+Shift+K", "label": "hotkey-compact" } },
  "settings": [
    { "id": "show-size", "label": "setting-show-size", "type": "boolean", "default": true }
  ]
}
```

| Field | Meaning |
|---|---|
| `id` | Lowercase reverse-DNS id; never changes once published |
| `version` | Semantic version |
| `name` | Proper name, shown as is |
| `description` | Fluent message id of the description in your catalogs (default `mod-description`) |
| `license`, `source` | SPDX license id and the https URL of the public source repository |
| `games` | Game ids from the host's game catalog; overlay windows only appear in these games |
| `permissions` | See below; unknown values are rejected |
| `network` | Plain host names (no scheme, port, path or wildcard) the mod may contact over https |
| `windows` | `overlay` windows need `width` and `height`; `entry` must stay inside the mod folder |
| `hotkeys` | Need the `hotkeys` permission; `label` is a Fluent message id |
| `settings` | Types `boolean`, `number` (`min`, `max`), `text`, `choice` (`options` are message ids and stored values); overLay generates the settings page |

overLay rejects manifests with unknown fields, so typos do not go unnoticed.

## Permissions

| Permission | Allows |
|---|---|
| `game.state` | Which game runs and the size of its window (overlay windows) |
| `game.events` | Live game events from the host's game providers (from phase 5) |
| `storage` | The mod's own key-value storage (JSON values, up to 1 MB) |
| `notifications` | Toast notifications in the overlay |
| `hotkeys` | The global hotkeys declared in the manifest |

Users see the permissions and network hosts on the mod's page in the launcher.

## SDK

Install `@overlay/sdk` (in this repository it is the npm workspace `sdk/`) and wait for `ready()` before anything else:

```ts
import { overlay } from '@overlay/sdk';

await overlay.ready();
const t = overlay.i18n.t;
document.body.textContent = t('hello', { game: overlay.game.getState()?.gameName ?? '' });
```

| API | Purpose |
|---|---|
| `overlay.ready()` | Connects to overLay, loads your catalogs; resolves with the context (mod id, window, locale, permissions, settings) |
| `overlay.i18n.t(id, args)`, `onChange(cb)` | Text from your catalogs; `onChange` fires after a language switch |
| `overlay.storage.get/set/remove/keys` | Your storage (`storage`) |
| `overlay.notifications.show(text)` | Toast in the overlay (`notifications`) |
| `overlay.bus.post(msg)`, `onMessage(cb)` | Messages between your mod's windows, e.g. launcher page ↔ overlay |
| `overlay.hotkeys.on(id, cb)` | Your declared hotkeys (`hotkeys`) |
| `overlay.settings.get()`, `onChange(cb)` | Values of your settings |
| `overlay.game.getState()`, `onChange(cb)` | Game and window size (`game.state`) |
| `overlay.events.on(cb)` | Live game events (`game.events`, from phase 5) |
| `overlay.windows.makeDraggable(el)` | Lets users move the overlay window by `el`; overLay saves the position per game and resolution |
| `overlay.windows.setInteractiveRegions(rects)` | Only these parts take clicks, the rest goes to the game; `null` = whole window |
| `overlay.windows.setClickThrough(true)` | Every click goes to the game (purely visual overlays) |

**Build your DOM once and update texts in place.** Replacing elements on every event can swap a button between mouse down and mouse up, and the click is lost.

Pass resolutions and other numbers that must not be grouped as strings (`String(width)`), because WebKit ignores Fluent's `useGrouping` option.

## Sandbox

Each window runs as a sandboxed frame with its own opaque origin:

- no access to overLay's pages, no `localStorage`/cookies (use `overlay.storage`), no `eval`, no popups, no forms, no nested frames;
- scripts, styles and files only from your own mod folder;
- `fetch`, images and media only from your mod and the https hosts in `network`. Everything else is blocked by the webview and logged by overLay.

## Developer mode

1. Build your mod (`npm run build` in its folder).
2. In overLay: Settings → Developer mode → enable, add the full path to the built folder.
3. The mod appears in the library with the *Developer* badge; a developer folder wins over an installed mod with the same id.
4. After a rebuild: *Reload mods* in the developer settings.

Start overLay with `OVERLAY_DEBUG=1` to log every file request of your mod and every change of the game window.
