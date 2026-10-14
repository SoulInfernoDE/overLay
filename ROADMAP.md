# overLay – Roadmap

> English original. German version: [ROADMAP.de.md](ROADMAP.de.md)
> Research basis: [docs/research.md](docs/research.md)
> As of: 2026-10-14 · Status: phase 0 done

## 1. Goal

overLay is an **open-source overlay manager** for games on **Linux (native, Wine, Proton)** and **Windows**:

- lean, fast, no bloat. The UI is **closely modelled on Overwolf** but uses its own branding and assets.
- It manages **overlay mods** from a public index repository and from any number of **custom repo URLs**.
- The first mod is **Flint**, a Hearthstone companion that recreates the look and feature set of **Firestone** (clean-room, see research §3.1). Flint's **primary target is the native Linux client** started through [hearthstone-linux-gui](https://github.com/DawnMagnet/hearthstone-linux-gui) (the official macOS game files on Unity's Linux player, no Wine, see research §5.1). Wine/Proton and Windows are supported as well.

## 2. Guiding principles (non-negotiable)

1. **Open source everywhere.** The host is under a free license. Every mod in the official index needs public source code and an OSI license, and its build artifact must be reproducible from that source.
2. **No telemetry, no ads.** Neither in the host nor in mods: no analytics, no ad SDKs, no trackers. Allowed network traffic:
   - fetching the signed mod index and updates (can be disabled)
   - data a mod has **declared** and the user has **approved** (e.g. card data from HearthstoneJSON)
   - optional community statistics, **opt-in only**, and only through an open, self-hostable server
3. **Verify first, then activate.** An update is downloaded, its signature and hash are checked, it is staged, and only then activated. If an update asks for new permissions or domains, the user must approve them.
4. **Resource budget as an acceptance criterion:**
   - With no game running, only the native core process runs, no WebView (target: < 30 MB RAM)
   - With an overlay active: < 150 MB RAM in total, < 2 % CPU when idle
   - Installer < 20 MB
5. **No code injection into games** by default. The overlay runs as its own window or surface, not as a DirectX or Vulkan hook. This keeps anti-cheat risk low and works under Proton.
6. **i18n from day one.** All text lives in locale catalogs, with English as the base and fallback and German as the second language. This applies to the host **and** to mods.
7. **Bilingual docs.** README, changelog and release notes are published in English with a German version.

## 3. Architecture (proposal)

```
┌──────────────────────────── overLay Host (Rust) ────────────────────────────┐
│ Core daemon: tray, game detection, hotkeys, mod manager, updater, settings  │
│   ├─ Game providers (Rust crates): emit events + state (like Overwolf GEP)  │
│   │     └─ hearthstone: log parser, Wine prefix finder, memory reader (opt.)│
│   ├─ Overlay backends: Windows · X11/XWayland · Wayland layer-shell · gamescope│
│   └─ Security layer: permissions, network allowlist, CSP, signatures        │
├──────────────────────────────────────────────────────────────────────────────┤
│ UI (WebView via Tauri 2): launcher/dock, library, store, settings           │
│ Mods = web bundles (HTML/JS/CSS) in sandboxed WebViews, API: `overlay.*`    │
└──────────────────────────────────────────────────────────────────────────────┘
          ▲ signed index (GitHub repo `overlay-mods-index`) + custom repo URLs
```

**Rationale for the core decisions:**

| Decision | Why |
|---|---|
| **Tauri 2 (Rust + system WebView)** instead of Electron/CEF | No bundled Chromium, so much less RAM and disk use. WebView2 on Windows, WebKitGTK on Linux. The UI can still use web tech, which keeps mods easy to write (as with Overwolf). |
| **Mods as web bundles + manifest**, game data from the host | Mods run no native code. Memory reading, file access and process access live in reviewed **game providers** inside the host. This keeps the sandbox tight, and mods need no DLLs (unlike Overwolf's `extra-objects`). |
| **Later: providers as WASM components** (wasmtime, capability-based) | Third parties can ship providers for new games without getting full system access. |
| **Overlay as its own transparent topmost window** | Works over borderless fullscreen games (HDT has done this for years), without injection and also under Proton. Exclusive fullscreen (Windows) is deliberately not supported. |
| **UI framework for host and Flint: Svelte 5** (TypeScript, Vite) | Compiles to small bundles without a heavy runtime (Firestone uses Angular). Mods can still use any framework, because the mod API is plain web-standard JavaScript. |
| **Local data: SQLite** (from Rust, `rusqlite`) | Games, decks, replays and stats stay local. No account required. |

## 4. Phases

Each phase ends with a working, releasable state. Time estimates are rough and assume a small part-time team.

### Phase 0: Foundation *(1–2 weeks)*
- [x] Git repo, license **GPL-3.0-or-later**, `CONTRIBUTING`, `CODE_OF_CONDUCT` (Contributor Covenant 3.0), `SECURITY.md`, each in EN + DE
- [x] **Mod policy** ([MOD_POLICY.md](MOD_POLICY.md)): source code required, OSI license, no telemetry/ads, network declaration, no remote code, fair play, clean-room rule, review process
- [x] Monorepo layout: `crates/` (Rust), `ui/` (host UI), `locales/`, `sdk/` (mod SDK + types), `mods/` (example mods), `tools/`, `docs/`
- [x] CI: build and tests on Linux and Windows, lints, type checks, license and advisory scan (`cargo-deny`, `tools/check-npm-licenses.mjs`, `npm audit`), actions pinned to commit hashes, Dependabot
- [x] i18n scaffolding with **Fluent**: catalogs shared by Rust (`overlay-i18n`) and the UI, English fallback, test for complete translations
- [x] Design tokens (`ui/src/styles/tokens.css`) and icons based on Lucide (ISC) in the dark dock look
- Toolchain: Rust 1.99 (edition 2024), Node 26, Svelte 5, Vite 8, TypeScript 6 (svelte-check does not support TypeScript 7 yet)

### Phase 1: Host core & launcher UI *(4–6 weeks)*
- [ ] Tauri shell with a **tray-only mode** (no WebView while the window is closed)
- [ ] Launcher UI modelled on Overwolf: dock on the left, **Library**, **Store** (local only for now), **Settings**, custom title bar
- [ ] **Game detection**: processes on Windows (WinAPI, no `tasklist` polling), on Linux via `/proc` for the **native client** (`Hearthstone.x86_64`) and for Wine/Proton processes (`Hearthstone.exe` under `wine64-preloader`/`wineserver`)
- [ ] Game catalog (`games/*.toml`): process names, window titles, launcher hints
- [ ] Global hotkeys (Windows: RegisterHotKey; Linux: XDG GlobalShortcuts portal, X11 fallback)
- [ ] Persistent settings (TOML), optional autostart
- **Acceptance:** the host starts, detects Hearthstone natively and under Proton, shows its status in the launcher, and stays within the resource budget.

### Phase 2: Overlay engine *(4–6 weeks)*
- [ ] Overlay windows: transparent, borderless, topmost, **click-through per region** (interactive areas take input, everything else goes to the game)
- [ ] Attach windows to the game window: follow its position and size, hide when it is minimised or loses focus
- [ ] Backends:
  - Windows (`WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE`)
  - **X11/XWayland** (Shape extension for click-through, `_NET_WM_STATE_ABOVE`). This is the default path under Proton, because Proton games run through XWayland.
  - **Wayland layer-shell** (KDE, wlroots, Hyprland). GNOME falls back to XWayland.
  - **gamescope** (Steam Deck/Game Mode): external overlay atom, **later** (phase 9)
- [ ] Toggle the overlay by hotkey (like Overwolf), toast notifications
- [ ] Save widget positions (per game, per resolution)
- **Acceptance:** a demo overlay over the native Linux client and over Hearthstone under Proton and on Windows, on KDE (Wayland + XWayland), GNOME (XWayland) and Sway. Clicks pass through to the game, and widgets can be dragged.

### Phase 3: Mod SDK & runtime *(4–6 weeks)*
- [ ] **`overlay.json` manifest** (modelled on Overwolf, but leaner):
  ```jsonc
  {
    "id": "io.github.<author>.<mod>", "version": "1.2.0",
    "name": "@i18n:name", "license": "GPL-3.0-or-later",
    "source": "https://github.com/<author>/<mod>",
    "games": [{ "id": "hearthstone", "providers": ["hearthstone@^1"] }],
    "permissions": ["game.events", "game.state", "storage.local", "hotkeys", "notifications"],
    "network": ["api.hearthstonejson.com", "art.hearthstonejson.com"],
    "windows": { "main": { "kind": "desktop" }, "tracker": { "kind": "overlay", "clickthrough": "auto" } },
    "hotkeys": { "toggle-tracker": { "default": "Ctrl+Shift+D" } },
    "locales": "locales/"
  }
  ```
- [ ] JS API `overlay.*`: `games` (launch/exit), `events` (`onEvent`, `onInfo`, `getInfo`, `requireFeatures`), `windows` (open, move, click-through, opacity), `storage` (key-value + per-mod SQLite), `hotkeys`, `notifications`, `i18n`, `settings` (schema → generated settings page)
- [ ] **Sandbox**: one WebView per mod window, strict CSP, **network allowlist enforced in the host** (requests to undeclared domains are blocked and logged), no Node, no file system access
- [ ] Message bus between a mod's windows (replaces Overwolf's background controller)
- [ ] **Developer mode**: load a mod from a local folder, hot reload, DevTools, **event recorder/player** (record and replay game sessions, which matters for testing without a running game)
- [ ] SDK package (`@overlay/sdk`) with TypeScript types, `create-overlay-mod` template
- [ ] *(optional)* compatibility layer for parts of the Overwolf API, so open Overwolf apps are easier to port
- **Acceptance:** the "Hello Overlay" example mod runs with a locale catalog. A request to an undeclared domain is shown to be blocked.

### Phase 4: Mod repository & secure updates *(3–4 weeks)*
- [ ] Separate repo **`overlay-mods-index`**: one file per mod (`mods/<id>.toml`) with source repo, releases (git tag + commit hash), artifact URL, **SHA-256**, permissions, domains, license
- [ ] **CI in the index repo** checks every PR:
  1. source repo is public and has an OSI license
  2. **reproducible build**: CI builds from the tag, and the result must match the artifact hash
  3. scan for tracker and ad domains and known SDKs, and compare the manifest domains with the code
  4. diff permissions against the previous version → manual review if they changed
  5. after merge, a maintainer or CI signs the index snapshot (**minisign/Ed25519**, ideas from TUF: expiry, version number against rollback)
- [ ] Client: fetch the index → verify the signature → list mods (store page with screenshots, changelog, permissions, source link)
- [ ] **Update pipeline**: download → verify hash → unpack into `staging/` → validate the manifest → ask the user to approve new permissions → activate atomically (rollback to the previous version possible)
- [ ] **Custom repos**: add a URL plus public key in the settings. On first use the user must explicitly trust the key (warning). Mods from custom repos are marked in the UI.
- [ ] Dependencies with semver ranges (lesson from CurseForge), stable/beta release channels
- **Acceptance:** a mod can be installed, updated and rolled back through the index. A tampered artifact is rejected, and so is an index with a wrong signature.

### Phase 5: Hearthstone game provider *(6–8 weeks)*
- [ ] **Install finder**, in this order:
  1. **native Linux client**: hearthstone-linux-gui (`game_dir` from `$XDG_CONFIG_HOME/hearthstone-linux-gui/config.toml`, otherwise `$XDG_DATA_HOME/hearthstone-linux-gui/game`), the original hearthstone-linux (user-chosen folder), or a running `Hearthstone.x86_64` (its working directory)
  2. Wine/Proton prefixes: Steam `compatdata`, Lutris, Bottles, Heroic, Faugus
  3. Windows: registry/Battle.net config
  4. a user-chosen path
- [ ] Create or extend `log.config` automatically (Power, Zone, LoadingScreen, Arena, Decks, Achievements …). Native client: macOS-style location `~/Library/Preferences/Blizzard/Hearthstone/` (to be confirmed in the spike); Wine: inside the prefix
- [ ] Handle Hearthstone's log size cap (logs are truncated at about 10 MB) and log rotation without losing the game state
- [ ] **Power.log parser in Rust** (references: python-hslog, HSTracker, both MIT): entities, tags, blocks, zone changes → game state and events (`game-start`, `mulligan`, `card-drawn`, `card-played`, `turn-start`, `game-end` …)
- [ ] Game mode detection (Ranked/Casual, Arena, BG, Brawl, Mercs, Solo)
- [ ] Card database: download HearthstoneJSON, cache it, refresh on a new patch. Card images are loaded **at runtime only** (Blizzard IP).
- [ ] *(optional, opt-in)* **memory reader** based on the ideas of UnitySpy (MIT, our own Rust port with attribution): collection, active deck, arena draft, BG lobby, quests, achievements. Native client first: a plain ELF process with `libmonobdwgc-2.0.so`, so no Wine address translation is needed. Wine/Proton via `/proc/<pid>/mem` as the second step.
- [ ] Event recordings of real games as test fixtures (recorder from phase 3)
- **Acceptance:** for 50 recorded games (all modes, recorded mainly on the native Linux client), the provider produces a game state that matches the final state in the log.

### Phase 6: Flint MVP: deck tracker *(6–8 weeks)*
Flint gets its own repository and is installed through the index, so the very first mod goes through the full pipeline. Its look follows Firestone, with original assets.
- [ ] **Own deck** overlay: card tiles (art crop, mana cost, count, rarity), drawn cards greyed out, remaining cards, draw probabilities
- [ ] **Opponent** overlay: played cards, hand with draw turn, buffs and known cards
- [ ] **Secrets helper**, counters (total attack, fatigue, corpses, spell schools …), turn timer
- [ ] Main window (Firestone-like): navigation on the left, **Decks** section (deck code import, local stats per deck, matchups), **Replays** (local list)
- [ ] Settings: size, opacity, position, which zones are visible
- [ ] First public **alpha release** of overLay plus the mod
- **Acceptance:** a full Ranked game on the native Linux client (hearthstone-linux-gui), under Proton and under Windows is tracked correctly, within the resource budget.

### Phase 7: Firestone parity in waves *(ongoing)*
Ordered by value and effort:
1. **Constructed+**: synergy highlighting, related cards on hover, mulligan view (local), deck builder, rank history, play order, recap of game-changing cards
2. **Battlegrounds basics**: card list by tier, tribe and mechanic, active/banned tribes, last-opponent info, session recap, HP graph, opponent boards, counters
3. **Arena**: run history, draft overlay (synergies, own stats), run recap
4. **Replays**: play-by-play viewer (our own implementation, the MIT XML parser can help)
5. **Collection & achievements** (needs the memory reader): packs, pity timer, set completion, pinned achievements
6. **Battlegrounds combat simulator**: a **separate sub-project**, because no freely licensed one exists. Engine in Rust → WASM, card mechanics data-driven and covered by tests
7. Tavern Brawl, Mercenaries (if still relevant in the game), Discord Rich Presence (local), "your turn" taskbar alert

### Phase 8: Community statistics *(optional, needs a policy decision)*
Meta decks, global mulligan data, BG hero tier lists and arena card values need **aggregated data from many players**. Without telemetry, the only way is:
- [ ] an open, **self-hostable** stats server (separate repo, AGPL), uploads **only by explicit opt-in**, anonymised, with a documented data format
- [ ] the client can also just **download** data without sending any
- [ ] alternatively or additionally: import public datasets where their license allows it

### Phase 9: Release 1.0 & distribution *(3–4 weeks)*
- [ ] Packages: **AppImage, Flatpak (Flathub), .deb, AUR**, Windows **MSI + winget**
- [ ] gamescope/Steam Deck support
- [ ] Host self-update using the same signature chain as mods
- [ ] Docs site (EN/DE), mod developer handbook, screenshots EN/DE
- [ ] Security review of the sandbox and the update chain

## 5. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Firestone has no license / "All rights reserved" | Copyright infringement if code is copied | strictly clean-room, only MIT/ISC building blocks, check where contributions come from |
| Trademark/trade dress (Overwolf, Firestone, Blizzard) | Legal notices | own names, logos and icons, no bundled Blizzard assets |
| Wayland does not allow global positioning | Overlay does not work everywhere | XWayland by default, layer-shell where available, gamescope backend |
| Memory reading (ToS grey area, ptrace rights on Linux) | Ban risk (low), setup effort | opt-in only, read-only, never write or inject; clear guide on `ptrace_scope` |
| Hearthstone patches change the log format or memory layout | Tracker breaks | recorder fixtures, fast provider updates through the signed pipeline |
| The native Linux client is unofficial (its authors mention a ban risk; the in-game shop is closed) and its paths can change | Flint's main target breaks or users are put at risk | Flint only reads, never patches the client; read the launcher's own `config.toml` instead of hard-coding paths; follow its releases and agree on a stable interface with its maintainers; Wine/Proton stays a full fallback |
| WebKitGTK performance and transparency vary by distro | Choppy overlay | early benchmarks in phase 2, fallback with compositing flags |
| BG simulator effort (every card) | Delay | separate sub-project, designed for community contributions |
| Global stats without telemetry | Feature gap compared with Firestone | phase 8 (opt-in, self-hostable) |

## 6. Open decisions

Decided on 2026-10-14: license **GPL-3.0-or-later**, Hearthstone mod name **Flint** (the stone that strikes sparks), UI framework **Svelte 5**.

Still open:

4. **Memory reader** already in phase 5, or only with phase 7.5?
5. **Community stats** (phase 8): build them at all, and if so, who runs the server?
6. **Hosting**: GitHub (as proposed), or also or instead Forgejo/Codeberg?

## 7. Next steps

1. ~~Settle open decisions 1–3~~ ✓
2. ~~Do phase 0~~ ✓
3. Create the GitHub repository and check the first CI run
4. **Technical spike** (before phase 1, about 1 week): a transparent click-through Tauri window over the **native Linux client (hearthstone-linux-gui)** and over Hearthstone under Proton, on KDE and GNOME, and on Windows, with RAM measurements. Also confirm the native client's `log.config` and log locations. This is the riskiest assumption in the whole project and should be tested first.
