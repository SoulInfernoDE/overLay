# Research: Overwolf, CurseForge, Firestone

> English original. German version: [research.de.md](research.de.md)
> Research date: 2026-10-14

This document records what is known about the reference products, which parts are public, and what overLay may and may not take from them.

---

## 1. Overwolf

### 1.1 What is open, what is not?

| Part | Status |
|---|---|
| Overwolf client (desktop, overlay injection, GEP) | **closed source**, Windows only |
| Overwolf app API (`overwolf.*`) | publicly **documented** (dev.overwolf.com), types in `overwolf/overwolf-api-ts` (no license) |
| `@overwolf/ow-electron` | Electron fork, npm package MIT; the overlay/GEP/recorder packages are downloaded as binaries and are **not open** |
| `@overwolf/ow-electron-packages-types` | MIT (TypeScript types only) |
| Sample apps (`overwolf/sample-app`) | public, **no license** |

**Conclusion:** only the **concepts** can be taken from Overwolf (manifest, window model, event API, permissions). No code or graphics may be copied. Writing our own implementation of an API is generally allowed in the EU, because interfaces and functionality are not protected by copyright (CJEU, SAS Institute, C-406/10).

### 1.2 Overwolf Native architecture

- **Runtime:** Chromium Embedded Framework (CEF). Every app is a web app (HTML/JS/CSS) with several windows.
- **Background controller:** an invisible window (`is_background_page`) holds the state and controls the visible windows. They exchange data through `overwolf.windows.getMainWindow()` and messaging.
- **`manifest.json`**, the central concept:
  - `meta`: name, author (the app ID is derived from these), version, icons, `minimum-overwolf-version`
  - `permissions`: e.g. `GameInfo`, `GameControl`, `Hotkeys`, `FileSystem`, `Clipboard`, `Tray`, `Web`, `Profile`, `Media`, `Streaming`, `Extensions`
  - `data.windows.<name>`: `file`, `transparent`, `clickthrough`, `in_game_only`, `desktop_only`, `native_window`, `topmost`, `size`/`min_size`/`max_size`, `start_position`, `resizable`, `keep_window_location`, `restrict_to_game_bounds`, `grab_keyboard_focus`, `focus_game_takeover`, `is_alt_f4_blocked`, `show_in_taskbar`, `mute`, `block_top_window_navigation` …
  - `data.game_targeting` (`all`/`dedicated`/`none` and `game_ids`), `data.game_events` (game IDs for GEP)
  - `data.launch_events`: start the app automatically when a game starts (`GameLaunch`, `AllGamesLaunch`, `LaunchWithOverwolf`)
  - `data.hotkeys`: named shortcuts with `default`, `action-type` (`toggle`/`custom`), `passthrough`, `hold`, `game_ids`
  - `data.extra-objects`: **native DLL plugins** (C#/.NET), reached through `overwolf.extensions.current.getExtraObject()`. Firestone relies on these heavily (see 3.3).
  - `data.externally_connectable`, `content_scripts`, `url_protocol`, `process_name`, `auto_relaunch_on_crash`
- **Game Events Provider (GEP):** two data types
  - *Game events*: discrete moments (kill, death …) through `overwolf.games.events.onNewEvents`
  - *Game info*: persistent state through `onInfoUpdates2` and `getInfo()`
  - Apps declare the features they need with `setRequiredFeatures()`. "Event status endpoints" report outages.
  - Game studios can include an **events SDK DLL** that sends data one way to Overwolf.
- **Overlay:** Overwolf injects itself into the render pipeline of supported games (DirectX hook). The ow-electron flow: `overlay.registerGames({gameIds})`, then a `game-launched` event with `event.inject()` or `event.dismiss()`, then `game-exit`. Loading windows such as shader compilation or anti-cheat are now skipped.
- **Tooling:** package manager in developer mode ("Load unpacked"), Events Recorder/Player (ERP), Game Events Simulator.
- **Distribution:** OPK package (zip with manifest), store review by Overwolf, monetisation through an **ad SDK** and subscriptions (revenue share).
- **Linux/macOS:** according to the ow-electron FAQ, **only ad services** run there at the moment. Overlay, game events and recording are "in development". Overwolf currently offers nothing for Linux/Proton, and that is exactly the gap overLay targets.

### 1.3 UI (to recreate, without copying assets)

- Dark UI with a narrow **vertical app bar (dock)** on the left with app icons
- Sections: **Library** (installed apps), **App Store** (tiles, banners, categories, detail page with screenshots), **Settings** (nav on the left: General, Overlay & Hotkeys, Apps, Notifications, About)
- Tray icon with a context menu
- In game: app windows as an overlay, shown and hidden by hotkey, toast notifications at the screen edge
- Custom frameless windows with their own title bar

> Trademark note: layout and interaction concepts may be similar. Overwolf's logo, wolf motif, names, icons and colour identity (trade dress) are **not** copied.

---

## 2. CurseForge

### 2.1 What is open?

- The CurseForge app and backend are **closed source**. The app used to run as an Overwolf app and is now also available standalone.
- Only the **CurseForge Core API** (`https://api.curseforge.com`) is open, documented at docs.curseforge.com.

### 2.2 Core API (concepts useful to overLay)

- **Authentication:** `x-api-key` header. Keys come from the CurseForge for Studios console and are only granted on application. This applies to public mods too.
- **Endpoints:** Games (`/v1/games…`), Categories, Mods (`/v1/mods/search`, `/v1/mods/{id}`, batch `POST /v1/mods`, featured, description), Files (`/v1/mods/{id}/files…`, changelog, download URL), **Fingerprints** (exact and fuzzy matching), Minecraft-specific endpoints.
- **Pagination:** at most 50 per page, at most 10,000 results in total.
- **Mod model:** `id`, `gameId`, `slug`, `classId`, categories, authors, links (`websiteUrl`, `sourceUrl`, `issuesUrl`), `downloadCount`, `mainFileId`, `latestFiles`, **`allowModDistribution`** (authors can forbid third-party downloads, in which case `downloadUrl` = `null`).
- **File model:** `releaseType` (1 = release, 2 = beta, 3 = alpha), `fileStatus`, `hashes[]` (`algo` 1 = SHA-1, 2 = MD5), `gameVersions[]`, **`dependencies[]`** with `relationType` (1 embedded library, 2 optional, 3 required, 4 tool, 5 incompatible, 6 include), `downloadUrl`, `fileFingerprint` (MurmurHash2 with seed 1, whitespace bytes 9/10/13/32 removed first, as known from community tools), `modules[]`.
- **Weakness:** dependencies only point to a mod ID, not a file version, so they cannot be resolved unambiguously.

### 2.3 What overLay takes from it

overLay **does not use CurseForge** (proprietary, key required, no open-source requirement). It takes the **data-model ideas**:

- release channels (stable/beta/alpha)
- typed dependencies, but **with version ranges** (semver) rather than IDs only
- several hashes per file, but overLay uses **SHA-256 and a signature** instead of SHA-1/MD5
- game versions / compatibility per file
- fingerprint matching to detect manually installed mods (optional, later)

---

## 3. Firestone (Hearthstone companion)

### 3.1 License: critical

- Repo `Zero-to-Heroes/firestone` (TypeScript, about 170 stars, actively maintained): **there is no license file.** Legally that means "all rights reserved".
- `tos.md` explicitly forbids copying, adapting and creating derivative works.
- The Battlegrounds simulator `@firestone-hs/simulate-bgs-battle` is explicitly **"All rights reserved"**.

**Conclusion:** Firestone is rebuilt **clean-room** for overLay. Features and look serve as the model, but **no code, assets or names** are taken. Contributors to the overLay Hearthstone mod must not copy Firestone code. The wiki feature list and screenshots may be used as references.

Exceptions with a free license that can be used:
- `@firestone-hs/reference-data`: **MIT**
- `@firestone-hs/hs-replay-xml-parser`: **MIT**
- `@firestone-hs/deckstrings`: **ISC** (fork of HearthSim)

### 3.2 Code structure (as of 10/2026)

- **Nx monorepo, Angular + NgRx/RxJS**, i18n with `ngx-translate`, Node 18
- `apps/`
  - `legacy`: the Overwolf Native app (CEF)
  - `electron-app` and `electron-frontend`: **ongoing migration to ow-electron** (overlay service using `IOverwolfOverlayApi`, SQLite through `better-sqlite3`, `electron-updater`, process detection by `tasklist` every 2 s)
  - `web`, `mobile` (Capacitor), `twitch-extension`, `coliseum` (replay viewer)
- `libs/`: `game-state`, `memory`, `battlegrounds`, `arena`, `constructed`, `collection`, `achievements`, `mercenaries`, `replay`, `stats`, `profile`, `settings`, `mainwindow`, `twitch`, `discord`, `communities`, `lottery`, `mods`, `electron`, `electron-edge-libs`, `shared`
- **Game state sources:**
  1. **Log parsing**: `Power.log` (and other logs) are turned into game events and XML replays in real time by a C# library (`HearthstoneReplays.dll`)
  2. **Memory reading ("MindVision")**: `OverwolfUnitySpy.dll` and `UnitySpy.HearthstoneLib.dll` read Hearthstone's Mono heap through UnitySpy (collection, deck, arena, BG player info, quests, achievements, rewards track, account/region …)
  3. In Electron these .NET DLLs are bridged through **edge-js** (`electron-edge-libs/*-edge.js`)
- **Other native plugins:** `OwUtils.dll`, `DiscordRPCPlugin.dll`, `OwBgReconnector.dll` (BG reconnect trick), `HpCipher.dll`
- **Backend:** its own AWS Lambdas and databases, many `@firestone-hs/*` API packages (stats, meta, mulligan, arena picks, BG global stats, leaderboards …), static data on `static.zerotoheroes.com`
- **Telemetry/ads:** `plausible-tracker` for analytics, Overwolf ads, premium subscription. overLay must not have any of this.
- **Mods feature:** Firestone can install **BepInEx 5** into Hearthstone and distributes "trusted mods" through a `mods-config.json`. That means code injection into the game client (see risks).

### 3.3 Feature set (model for the Hearthstone mod)

Condensed from the wiki pages "Firestone features" and "Premium features":

**Deck tracker / overlay (ladder, arena, brawl)**
- Own deck, hand, board, graveyard in collapsible zones; dynamic card effects (Tess, Fizzle …)
- Opponent tracker: played cards, decklist import, hand with draw turn, buffs, forge, possible cards
- Secrets helper (possible secrets, filtered by mode and effects)
- Counters: total attack, fatigue damage, corpses, spell schools, excavate …; compact widget
- Max-resources widget, minion play order, synergy highlighting, related cards on hover
- Turn timer, recap of game-changing cards, discover stats, mulligan guide

**Constructed (main window)**
- Deck overview with versions, deck stats (mulligan, cards, matchups), meta decks, rank history, deck builder with deck code export, lobby overlay with deck preview

**Battlegrounds**
- Meta overview (heroes, tribes, quests, rewards), personal hero stats, MMR history
- **Combat simulator** (win odds, positioning optimiser, share codes)
- Overlay: battle odds, card list by tier/mechanic/tribe, last-opponent info, session recap, counters, shop highlighting, tribes widget, opponent MMR
- BG window: opponent boards, HP graph, battle history with re-simulation

**Arena**
- Run history, class tier list, card stats, 10+ win decks, pick overlay with stats and synergies

**Other:** Tavern Brawl (class win rates), Mercenaries (progression, teams, ability overlay), Duels (since removed from the game), **Replays** (list and play-by-play viewer), **Achievements** (native and custom, pinnable), **Collection** (packs, pity timer, set completion, card backs, skins), profile, Discord Rich Presence, "your turn" taskbar alert, Twitch extension

**Look:** dark purple-blue main window with icon navigation on the left (deck tracker, Battlegrounds, Arena, collection, achievements, replays …). In game, a decklist on the right made of card tiles (art crop, mana gem, count) and the opponent tracker on the left. Size, opacity and colours are configurable.

---

## 4. Open building blocks for Hearthstone (usable)

| Project | License | Use |
|---|---|---|
| `HearthSim/python-hslog` | MIT | reference for the Power.log parser |
| `HearthSim/HSTracker` (Swift, macOS) | MIT | complete open deck tracker as a reference |
| `HearthSim/HearthDb` (C#) | MIT | card DB, enums (GameTag, Zone …) |
| `HearthSim/python-hearthstone` | MIT | enums, card data tools |
| `Hackf5/unityspy` (C#) | MIT | Unity Mono memory reader **with Linux support** (`ProcessFacadeLinux*`), successor of the closed HearthMirror |
| `HearthSim/npm-deckstrings` / `deckstrings` | ISC | encode and decode deck codes |
| `hearthstonejson-client` / HearthstoneJSON | ISC (client) | card data (`api.hearthstonejson.com`), card images (`art.hearthstonejson.com`). The **images belong to Blizzard**: load them at runtime, do not ship them |
| `@firestone-hs/reference-data` | MIT | reference data and enums |
| `@firestone-hs/hs-replay-xml-parser` | MIT | replay XML |
| `HearthSim/Hearthstone-Deck-Tracker` | source open, **GitHub detects no license** | behaviour reference only, no code |
| Fireplace / RosettaStone | AGPL-3.0 | constructed simulators, not directly suited to Battlegrounds |

**Battlegrounds simulator:** there is **no current, freely licensed one**. Our own simulator is a large sub-project, because every card has to be implemented.

---

## 5. Technical facts about Hearthstone on Linux/Proton

- Hearthstone is a **Unity Mono** game with no official Linux build. It runs either **natively** through the community launcher hearthstone-linux-gui (see 5.1) or through Battle.net under Wine/Proton (Lutris, Steam Proton as a non-Steam game, Bottles, Heroic, Faugus).
- **Logs:** `log.config` lives at `<prefix>/drive_c/users/<user>/AppData/Local/Blizzard/Hearthstone/log.config`. Logs are under `<install dir>/Logs/Hearthstone_<date>/Power.log` etc. overLay must **find Wine prefixes automatically** and create or extend `log.config`.
- **Memory:** UnitySpy can read a Wine process from the Linux host through `/proc/<pid>/mem` or ptrace. It needs the same user, and depending on `ptrace_scope` it needs `CAP_SYS_PTRACE`.
- **Overlay without injection:** the HDT deck tracker runs on Windows as a transparent topmost window over Hearthstone. That works because HS uses borderless fullscreen. On Linux the same approach works under X11 and XWayland. Native Wayland needs `wlr-layer-shell` (KDE, wlroots, Hyprland; **not GNOME**). **gamescope** (Steam Deck) needs the external overlay atom (`GAMESCOPE_EXTERNAL_OVERLAY`, as used by MangoHud/mangoapp).
- **Blizzard ToS:** deck trackers that only read logs and show information players could see for themselves have been tolerated for years. Memory reading is a grey area (HDT and Firestone do it). **Code injection into the client** (BepInEx) is riskier, and overLay does **not** offer it by default.

### 5.1 Native Linux client (hearthstone-linux, hearthstone-linux-gui)

**Flint's primary target.** Hearthstone is a Unity game, and Unity's platform files are largely generic. The community project [`0xf4b1/hearthstone-linux`](https://github.com/0xf4b1/hearthstone-linux) (MIT) runs the **official macOS game files on Unity's Linux player** (OpenGLCore renderer), without Wine and without the Battle.net app. It downloads nothing proprietary from its own servers; the game files come from Blizzard's CDN. Known limits: the in-game shop is closed, and the authors point out that this unofficial method may carry a ban risk.

[`DawnMagnet/hearthstone-linux-gui`](https://github.com/DawnMagnet/hearthstone-linux-gui) (MIT, Rust, GTK4/libadwaita, actively maintained) turns this into a packaged launcher: a native NGDP downloader with cache and verification, login, update and launch from a window, optional `gamemoderun` and MangoHud, a game settings editor, UI in English, German and Chinese, packages as AppImage, DEB, RPM, Flatpak and Nix.

Facts that matter for Flint (from the launcher's source code, 2026-10-14):

| Topic | Value |
|---|---|
| Process | `<game_dir>/Bin/Hearthstone.x86_64`, a native ELF process with `game_dir` as its working directory |
| Default `game_dir` | `$XDG_DATA_HOME/hearthstone-linux-gui/game` (usually `~/.local/share/hearthstone-linux-gui/game`) |
| Launcher config | `$XDG_CONFIG_HOME/hearthstone-linux-gui/config.toml`, optional key `game_dir`, plus `installed_version`, `locale`, `region`, `use_mangohud` … |
| Mono runtime | `Bin/Hearthstone_Data/MonoBleedingEdge/x86_64/libmonobdwgc-2.0.so`, so a memory reader works on a normal Linux process, without Wine address translation |
| Game settings | the Linux Unity player uses the **macOS location**: `~/Library/Preferences/Blizzard/Hearthstone/options.txt`. `log.config` belongs in the same folder: **confirmed** on 2026-10-14, the client then writes `Power.log`, `Zone.log` and the other enabled logs |
| Game logs | `<game_dir>/Logs/Hearthstone_<YYYY_MM_DD_hh_mm_ss>/`, one folder per game start (**confirmed**; not `Bin/Logs/`). The launcher redirects Unity's stdout/stderr (`Player.log`) into its own log folder |
| Window | Unity Linux player, normally X11/XWayland, so the same overlay backend as for Proton applies |
| Log size limit | Once any log reaches 10 MB (`Truncating log, which has reached the size limit of 10000KB`), Hearthstone stops writing all logs. `client.config` in the game folder lifts the limit: `[Log] FileSizeLimit.Int=-1`, or flat `Log.FileSizeLimit.Int=-1` as the launcher writes its entries; the launcher keeps user entries in `~/.config/hearthstone-linux-gui/client.config.overrides` (**confirmed** 2026-10-16; on 2026-10-17 `Power.log` grew to 15.5 MB over two Battlegrounds games and was written to the end) |
| `Power.log` streams | Every change is written twice: `GameState.DebugPrintPower()` on arrival from the server, `PowerTaskList.DebugPrintPower()` when the client plays it (same records and counts). In Battlegrounds a combat takes under a second in the `GameState` stream and 11–16 s in the `PowerTaskList` stream, matching the animation. The `PowerTaskList` stream sometimes names players (e.g. the innkeeper) where `GameState` writes their entity id. The game header (`DebugPrintGame`) exists only in the `GameState` stream (**confirmed** 2026-10-17) |

**Other Linux trackers (references):**

- [`Ratpido-dev/cairn`](https://github.com/Ratpido-dev/cairn) (MIT, Python): native Linux tracker that only reads logs, with no injection, no memory reading and no telemetry. A useful finding from it: Hearthstone caps its logs at about 10 MB and then truncates them ("Truncating log"); under Wine this step can fail, and tracking goes blind. Cairn supports the native client only through a manually set log path and cannot write its `log.config`. Flint should handle both.
- [`borisbabic/hearthstone_hdt_linux`](https://github.com/borisbabic/hearthstone_hdt_linux) (Apache-2.0): instructions for HDT under Wine.
- [`yusufipk/deste-hearthstone-linux-tracker`](https://github.com/yusufipk/deste-hearthstone-linux-tracker) (no license): behaviour reference only.

---

## Sources

- [Overwolf framework overview](https://dev.overwolf.com/ow-native/getting-started/onboarding-resources/framework-overview)
- [Overwolf manifest.json](https://dev.overwolf.com/ow-native/reference/manifest/manifest-json)
- [Overwolf GEP intro](https://dev.overwolf.com/ow-native/reference/live-game-data-gep/live-game-data-gep-intro)
- [Overwolf events SDK for game developers](https://dev.overwolf.com/ow-native/live-game-data-gep/events-sdk-for-game-developers)
- [ow-electron overlay API](https://dev.overwolf.com/ow-electron/reference/Overwolf-electron-APIs/overlay/interfaces/IOverwolfOverlayApi/)
- [ow-electron FAQ (Linux/macOS status)](https://dev.overwolf.com/ow-electron/getting-started/onboarding-resources/ow-electron-faq)
- [ow-electron sample (community)](https://github.com/peka2/ow-electron-packages-sample)
- [CurseForge Core API](https://docs.curseforge.com/rest-api/)
- [Firestone repo](https://github.com/Zero-to-Heroes/firestone), [features](https://github.com/Zero-to-Heroes/firestone/wiki/Firestone-features), [premium](https://github.com/Zero-to-Heroes/firestone/wiki/Premium-features)
- [UnitySpy](https://github.com/Hackf5/unityspy), [python-hslog](https://github.com/HearthSim/python-hslog), [HSTracker](https://github.com/HearthSim/HSTracker), [HearthDb](https://github.com/HearthSim/HearthDb)
- BG simulators: [twanvl](https://github.com/twanvl/hearthstone-battlegrounds-simulator), [BGSimulator](https://github.com/yossielimelech/BGSimulator), [Ghastcoiler](https://github.com/pruderior/Ghastcoiler), [Fireplace](https://github.com/synap5e/fireplace)
