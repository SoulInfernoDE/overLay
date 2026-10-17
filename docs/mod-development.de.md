# overLay-Mods schreiben

> Deutsche Fassung. Englisches Original: [mod-development.md](mod-development.md)
> Gilt für SDK 0.1 (Phase 3). Die API kann sich bis overLay 1.0 noch ändern.

Ein overLay-Mod ist eine kleine Web-App: HTML, JavaScript und CSS, dazu ein Manifest (`overlay.json`) und seine Übersetzungen. overLay zeigt seine Fenster im Spiel (Overlay-Fenster) oder im Launcher (Desktop-Fenster). Das Beispiel [`mods/hello-overlay`](../mods/hello-overlay) nutzt alles, was hier beschrieben ist.

Lies vor dem Veröffentlichen die [Mod-Richtlinie](../MOD_POLICY.de.md): Open Source, keine Telemetrie, keine Werbung, kein nachgeladener Code.

## Aufbau des Ordners

Der Ordner, den overLay lädt (beim Beispiel: `mods/hello-overlay/dist/` nach `npm run build`):

```
overlay.json          Manifest
index.html            Einstieg eines Fensters (beliebiger Name, siehe „windows“)
assets/…              eigene Skripte, Styles, Bilder
locales/en/*.ftl      Fluent-Kataloge, Englisch ist Pflicht
locales/de/*.ftl      weitere Sprachen
```

Alle URLs im Mod müssen relativ sein (mit Vite: `base: './'`).

## Manifest (`overlay.json`)

```json
{
  "id": "io.github.du.mein-mod",
  "version": "0.1.0",
  "name": "Mein Mod",
  "description": "mod-description",
  "author": "Du",
  "license": "GPL-3.0-or-later",
  "source": "https://github.com/du/mein-mod",
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

| Feld | Bedeutung |
|---|---|
| `id` | Reverse-DNS-ID in Kleinbuchstaben; ändert sich nach der Veröffentlichung nie |
| `version` | Semantische Version |
| `name` | Eigenname, wird unverändert angezeigt |
| `description` | Fluent-Nachrichten-ID der Beschreibung in deinen Katalogen (Standard `mod-description`) |
| `license`, `source` | SPDX-Lizenzkennung und https-URL des öffentlichen Quellcode-Repositorys |
| `games` | Spiele-IDs aus dem Spielekatalog des Hosts; Overlay-Fenster erscheinen nur in diesen Spielen |
| `permissions` | Siehe unten; unbekannte Werte werden abgelehnt |
| `network` | Einfache Host-Namen (ohne Schema, Port, Pfad oder Platzhalter), die der Mod per https erreichen darf |
| `windows` | `overlay`-Fenster brauchen `width` und `height`; `position` ist in CSS-Pixeln ab der linken oberen Ecke des Spielfensters, und overLay holt Fenster, die außerhalb lägen, zurück ins Bild; `entry` muss im Mod-Ordner liegen |
| `hotkeys` | Brauchen die Berechtigung `hotkeys`; `label` ist eine Fluent-Nachrichten-ID |
| `settings` | Typen `boolean`, `number` (`min`, `max`), `text`, `choice` (`options` sind Nachrichten-IDs und gespeicherte Werte); overLay erzeugt die Einstellungsseite |

overLay lehnt Manifeste mit unbekannten Feldern ab, damit Tippfehler nicht unbemerkt bleiben.

## Berechtigungen

| Berechtigung | Erlaubt |
|---|---|
| `game.state` | Welches Spiel läuft und wie groß sein Fenster ist (Overlay-Fenster) |
| `game.events` | Live-Ereignisse und Spielübersicht aus den Game Providern des Hosts |
| `storage` | Eigener Schlüssel-Wert-Speicher des Mods (JSON-Werte, bis 1 MB) |
| `notifications` | Hinweis-Einblendungen im Overlay |
| `hotkeys` | Die im Manifest deklarierten globalen Tastenkürzel |

Berechtigungen und Netzwerk-Hosts sehen Nutzer auf der Seite des Mods im Launcher.

## SDK

Installiere `@overlay/sdk` (in diesem Repository ist es der npm-Workspace `sdk/`) und warte vor allem anderen auf `ready()`:

```ts
import { overlay } from '@overlay/sdk';

await overlay.ready();
const t = overlay.i18n.t;
document.body.textContent = t('hello', { game: overlay.game.getState()?.gameName ?? '' });
```

| API | Zweck |
|---|---|
| `overlay.ready()` | Verbindet mit overLay, lädt deine Kataloge; liefert den Kontext (Mod-ID, Fenster, Sprache, Berechtigungen, Einstellungen) |
| `overlay.i18n.t(id, args)`, `onChange(cb)` | Texte aus deinen Katalogen; `onChange` meldet sich nach einem Sprachwechsel |
| `overlay.storage.get/set/remove/keys` | Dein Speicher (`storage`) |
| `overlay.notifications.show(text)` | Hinweis im Overlay (`notifications`) |
| `overlay.bus.post(msg)`, `onMessage(cb)` | Nachrichten zwischen den Fenstern deines Mods, z. B. Launcher-Seite ↔ Overlay |
| `overlay.hotkeys.on(id, cb)` | Deine deklarierten Tastenkürzel (`hotkeys`) |
| `overlay.settings.get()`, `onChange(cb)` | Werte deiner Einstellungen |
| `overlay.game.getState()`, `onChange(cb)` | Spiel und Fenstergröße (`game.state`) |
| `overlay.events.on(cb)`, `getInfo()`, `onInfo(cb)` | Live-Ereignisse und Spielübersicht (`game.events`), siehe unten |
| `overlay.windows.makeDraggable(el)` | Nutzer können das Overlay-Fenster an `el` verschieben; overLay speichert die Position pro Spiel und Auflösung |
| `overlay.windows.setInteractiveRegions(rects)` | Nur diese Bereiche nehmen Klicks an, der Rest geht ans Spiel; `null` = ganzes Fenster |
| `overlay.windows.setClickThrough(true)` | Jeder Klick geht ans Spiel (rein optische Overlays) |

**Baue das DOM einmal auf und aktualisiere nur die Texte.** Werden Elemente bei jedem Ereignis ersetzt, kann ein Knopf zwischen Maustaste-runter und Maustaste-hoch ausgetauscht werden, und der Klick geht verloren.

Übergib Auflösungen und andere Zahlen, die nicht gruppiert werden dürfen, als Text (`String(width)`), denn WebKit ignoriert die Fluent-Option `useGrouping`.

## Spielereignisse (Hearthstone)

Mit `game.events` bekommt ein Mod, was der Host aus Hearthstones `Power.log` liest, und zwar im Takt dessen, was der Spieler sieht: Änderungen kommen, wenn der Client sie abspielt, sodass sich ein Schlachtfeld-Kampf über seine Animation entfaltet, statt vorher vollständig anzukommen. Typen: `import type { Hearthstone } from '@overlay/sdk'`.

| Ereignis | Daten |
|---|---|
| `game-start` | `game_type` (z. B. `GT_RANKED`, `GT_BATTLEGROUNDS`, `GT_ARENA`), `format`, `scenario_id`, `players` (ID, Name, Seite) |
| `mulligan` | – (im Schlachtfeld: die Heldenwahl) |
| `turn-start` | `turn`, `side` (`friendly`, `opposing`, `unknown`) |
| `zone-change` | `entity_id`, `card_id` (null, solange verdeckt), `side`, `from` (null bei neuen Karten), `to` (`deck`, `hand`, `play`, `graveyard`, `secret`, `setaside` …) |
| `card-revealed` | `entity_id`, `card_id`, `side`, `zone` |
| `game-end` | `result` (`win`, `loss`, `tie`, `unknown`) |
| `game-created` | – eine neue Partie beginnt |

`overlay.events.getInfo()` liefert eine Übersicht der laufenden Partie (Modus, Zug und pro Seite: Held, Hand, Brett, Geheimnisse, Friedhof, Deckgröße), `onInfo(cb)` meldet Änderungen. Nutze das, wenn dein Fenster mitten in einer Partie startet.

Für alles andere hält das SDK einen **Spiegel aller Entitäten der Partie mit allen Tags**: `overlay.events.entities()` (eine schreibgeschützte `Map` nach Entitäts-ID) und `onEntitiesChanged(cb)` (die im letzten Paket geänderten IDs). Er wird beim Start deines Fensters vollständig geladen und alle 100 ms paketweise aktualisiert. Dort stecken die Schlachtfeld-Daten, z. B. `PLAYER_TECH_LEVEL` (Tavernenstufe) und `RESOURCES` an der Spieler-Entität, `PLAYER_LEADERBOARD_PLACE` an den Helden, `BACON_CURRENT_COMBAT_PLAYER_ID` für den aktuellen Gegner. Karten-IDs sind Hearthstones eigene IDs; Namen und Bilder lädst du aus einer Kartendatenbank, die du in `network` deklarierst (z. B. `api.hearthstonejson.com`, `art.hearthstonejson.com`).

## Sandbox

Jedes Fenster läuft als Sandbox-Frame mit eigenem, undurchsichtigem Ursprung:

- kein Zugriff auf die Seiten von overLay, kein `localStorage`/keine Cookies (nutze `overlay.storage`), kein `eval`, keine Pop-ups, keine Formulare, keine verschachtelten Frames;
- Skripte, Styles und Dateien nur aus dem eigenen Mod-Ordner;
- `fetch`, Bilder und Medien nur vom eigenen Mod und von den https-Hosts aus `network`. Alles andere blockiert die WebView, und overLay protokolliert es.

## Entwicklermodus

1. Mod bauen (`npm run build` in seinem Ordner).
2. In overLay: Einstellungen → Entwicklermodus → einschalten, vollständigen Pfad zum gebauten Ordner hinzufügen.
3. Der Mod erscheint in der Bibliothek mit dem Abzeichen *Entwicklung*; ein Entwickler-Ordner hat Vorrang vor einem installierten Mod mit derselben ID.
4. Nach einem neuen Build: *Mods neu laden* in den Entwickler-Einstellungen.

Starte overLay mit `OVERLAY_DEBUG=1`, um jede Dateianfrage deines Mods, jede Änderung des Spielfensters und die klickbaren Bereiche des Overlays zu protokollieren (hilfreich, wenn Klicks dein Fenster nicht erreichen oder es dort erreichen, wo sie ans Spiel gehen sollten).
