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
   - With an overlay active: ≤ 150 MB RAM in total, < 2 % CPU when idle (spike over the running game: 146 MB, 0.5 %)
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
| **Overlay as its own transparent topmost window** | Works over borderless fullscreen games (HDT has done this for years), without injection and also under Proton. Exclusive fullscreen (Windows) is deliberately not supported. Confirmed by the [spike](spikes/overlay-window/RESULTS.md) over the native client. |
| **One shared overlay webview for all in-game widgets of all mods** | With WebKitGTK every webview window starts its own web process (about 50 MB each, measured). Mods run in sandboxed frames with their own origin and CSP inside that one webview; clicks outside the widgets pass through via the input shape. The desktop main window is a second webview that only exists while it is open. |
| **Overlay never takes focus; input shape in device pixels** | Lessons from the spike: the overlay must not take keyboard focus from the game, and the clickable regions must be converted from CSS pixels by the device pixel ratio (e.g. 1.2 at 115 dpi), otherwise they drift away from the visible widgets and block game buttons. |
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
- [x] Tauri shell (`crates/overlay-app`) with a **tray-only mode**: started in the tray (`--minimized`, used by autostart) it needs **27 MB PSS** and no WebView; the launcher window is created on demand and destroyed when closed; single instance (a second start opens the window)
- [x] Free the memory WebKit keeps after the launcher window was closed (81 MB PSS instead of 27 MB): solved in phase 2, the launcher runs in its own process that exits with the window
- [x] Launcher UI modelled on Overwolf: dock on the left, **Library** (games with live status), **Store** (placeholder until phase 4), **Settings**, custom title bar with drag region, minimize and close
- [x] **Game detection** (`crates/overlay-games`): ToolHelp snapshot on Windows, `/proc` on Linux for the **native client** (`Hearthstone.x86_64`) and Wine/Proton (`Hearthstone.exe`), every 2 s, start/exit events to UI and tray
- [x] Game catalog (`games/*.toml`, embedded at build time): process names per platform and flavor. Window titles and launcher hints follow with phase 2 and 5
- [x] Global hotkey to show or hide the launcher (default `Ctrl+Shift+O`) on Windows and X11
- [ ] Global hotkeys on Wayland through the XDG GlobalShortcuts portal
- [x] Persistent settings (`crates/overlay-settings`, TOML in the config folder, atomic writes, broken files moved aside), language, start in tray, optional autostart
- [x] Tray menu and all launcher text localized (English, German), following the system language unless the user picks one
- **Acceptance:** the host starts, detects Hearthstone natively and under Proton, shows its status in the launcher, and stays within the resource budget. *Status 2026-10-14: met for the native client on Linux (X11); Proton and Windows still to be tested.*

### Phase 2: Overlay engine *(4–6 weeks)*
- [x] **Process model:** the core (tray, detection, settings, hotkeys) loads no webview; the launcher and the overlay are child processes of the same executable (`--ui launcher`, `--ui overlay`) that talk to the core over stdin/stdout (`crates/overlay-ipc`, JSON lines) and exit when they are no longer needed. Closing the launcher brings the core back to 27 MB PSS (was 81 MB)
- [x] **One overlay window** per running game for all widgets: transparent, borderless, on top, never takes focus, **click-through per region** (widgets take input, everything else goes to the game; regions converted by the device pixel ratio)
- [x] Follows the game window: X11 through events (`ConfigureNotify`, property changes, no polling), Windows by polling every 50 ms; visible while the game is shown and no other active window covers it (so working on another monitor keeps the overlay)
- [ ] Backends:
  - [x] **X11/XWayland** (GTK input shape, `above` or override-redirect stacking). The default under Proton and for the native Hearthstone client, which both run on X11/XWayland; tested on Cinnamon
  - [ ] Windows (always on top, cursor-driven click-through): implemented and compiled in CI, **not yet tested on Windows**
  - [ ] **Wayland layer-shell** (KDE, wlroots, Hyprland) for native Wayland games; not needed for Hearthstone
  - [ ] **gamescope** (Steam Deck/Game Mode): external overlay atom, **later** (phase 9)
- [x] Toggle the overlay by hotkey (`Ctrl+Shift+L`), toast notifications (start of the overlay, overlay shown again)
- [x] Widget positions saved per game and resolution (`layout.toml`); a resolution without its own entry scales the closest stored one (e.g. fullscreen → windowed)
- [x] Built-in demo widget (setting) to test the overlay without mods
- **Acceptance:** a demo overlay over the native Linux client and over Hearthstone under Proton and on Windows, on KDE (Wayland + XWayland), GNOME (XWayland) and Sway. Clicks pass through to the game, and widgets can be dragged. *Status 2026-10-14: met for the native client on Cinnamon (X11), 174 MB PSS for core + overlay with the game running; Proton, Windows, KDE, GNOME and Sway still to be tested.*
- Known Hearthstone behaviour, not caused by overLay: resizing the game window in windowed mode makes Hearthstone zoom its view (same with the overlay switched off).

### Phase 3: Mod SDK & runtime *(4–6 weeks)*
- [x] **`overlay.json` manifest** (`crates/overlay-mods`), strictly validated: reverse-DNS id, semver, SPDX license, https source, `games`, `permissions` (`game.state`, `game.events`, `storage`, `notifications`, `hotkeys`), `network` (plain host names), overlay and desktop `windows`, `hotkeys`, `settings` schema. Reference: [docs/mod-development.md](docs/mod-development.md)
- [x] **Sandbox** (verified in [spikes/mod-sandbox](spikes/mod-sandbox/RESULTS.md)): mod windows are sandboxed frames inside the one overlay webview (and the launcher), served by the host over `mod://`; per-mod CSP from the manifest, so the **webview enforces the network allowlist** and reports blocked requests to the host, which logs them; no `eval`, no access to overLay's pages, files only from the mod's own folder (also through symlinks)
- [x] SDK package `@overlay/sdk` (`sdk/`, TypeScript): `ready`, `i18n` (the mod's own Fluent catalogs), `storage` (JSON key-value, 1 MB), `notifications`, `bus` (between a mod's windows, across processes), `hotkeys` (declared in the manifest, registered by the core), `settings` (generated settings page in the launcher), `game` (state and window size), `events` (prepared for phase 5), `windows` (drag, clickable regions, click-through). Every call is permission-checked in the core
- [x] **Developer mode**: load mods from local folders, reload, load errors in the settings, `OVERLAY_DEBUG=1` logs mod requests and game window changes
- [x] Example mod [`mods/hello-overlay`](mods/hello-overlay) (EN/DE)
- [x] Launcher: mods in the library (on/off), mod page with description, permissions, network hosts, generated settings and the mod's desktop window
- [ ] Event recorder/player (record and replay game sessions) – needs game events, moves to phase 5
- [ ] `create-overlay-mod` template
- [ ] Per-mod SQLite storage (key-value storage is enough for now)
- [ ] Automatic reload when a developer folder changes (now: *Reload mods* button)
- [ ] *(optional)* compatibility layer for parts of the Overwolf API, so open Overwolf apps are easier to port
- **Acceptance:** the "Hello Overlay" example mod runs with a locale catalog. A request to an undeclared domain is shown to be blocked. *Status 2026-10-15: met on Linux (X11) over the native Hearthstone client; the blocked request to example.com was reported and logged.*

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
`crates/overlay-hearthstone`, run by the core while the game runs.
- [x] **Install finder** for the running game: native Linux client (working directory or `Bin/` parent), Wine/Proton (`WINEPREFIX` or `STEAM_COMPAT_DATA_PATH`, working directory or the Windows path of the executable), Windows (executable path). Verified at runtime for the native client; Wine/Proton and Windows are implemented, still to be tested
- [ ] Install finder without a running game (launcher config, Steam `compatdata`, Lutris, Bottles, Heroic, Faugus, user-chosen path), e.g. to prepare `log.config` before the first start
- [x] `log.config` is created or extended automatically (`Power` with `Verbose`, `LoadingScreen`, `Decks`, `Arena`), user settings are kept; when it changed, a toast asks for one restart of the game (toasts now wait in the core until the overlay shows them)
- [x] Log size cap and rotation: truncated logs are read again from the start, a new `Logs/Hearthstone_<timestamp>/` folder is followed automatically
- [x] **Log size limit lifted** (`client.config`: `[Log] FileSizeLimit.Int=-1`, flat `Log.FileSizeLimit.Int=-1` for the native client, also in hearthstone-linux-gui's `client.config.overrides` so launcher updates keep it). Found on 2026-10-16: once any log (here `Zone.log`) reaches 10 MB, Hearthstone stops writing *all* logs, so `Power.log` ended seven minutes into a Battlegrounds game and the end of the game was never seen. overLay no longer enables logs it does not read (`Zone`, `Asset`)
- [x] **Power.log parser in Rust** (own implementation on the `GameState` stream): entity model with all tags, player-name resolution that never guesses in games with more than two players, events `game-start`, `mulligan`, `turn-start`, `zone-change`, `card-revealed`, `game-end`, and a game summary (`game-info`) for mods that start mid-game. SDK: `overlay.events.on`, `getInfo`, `onInfo` with TypeScript types
- [x] Game mode from the game header (`GameType`, `FormatType`, `ScenarioID`): Ranked/Casual, Arena, Battlegrounds, Brawl, Solo …
- [x] Cost: 0.5 % CPU idle, events every 100 ms while the game writes; the first read of a long Battlegrounds game takes a moment
- [ ] Deck list at game start (`Decks.log`), for the constructed deck tracker
- Card database: **not in the host**. Mods load card data themselves from the hosts they declare (e.g. HearthstoneJSON), so the user sees and approves that network access; card images only at runtime (Blizzard IP)
- [ ] *(optional, opt-in)* **memory reader** based on the ideas of UnitySpy (MIT, our own Rust port with attribution): collection, active deck, arena draft, BG lobby, quests, achievements. Native client first: a plain ELF process with `libmonobdwgc-2.0.so`, so no Wine address translation is needed. Wine/Proton via `/proc/<pid>/mem` as the second step.
- [ ] Event recorder/player for mod development without a running game
- **Acceptance:** for 50 recorded games (all modes, recorded mainly on the native Linux client), the provider produces a game state that matches the final state in the log. *Status 2026-10-16: unit tests with a hand-written game; real games checked locally with `cargo run -p overlay-hearthstone --example replay -- <Power.log>` (a game against the AI and a Battlegrounds game, sides, heroes and zones correct) and live in a Battlegrounds game. Real logs stay local because they contain account data; the set grows with every game played.*

### Phase 6: Flint MVP: Battlegrounds first, then the deck tracker *(6–8 weeks)*
Flint gets its own repository and is installed through the index, so the very first mod goes through the full pipeline. Its look follows Firestone, with original assets. **Battlegrounds has priority** (decided 2026-10-16) and must work without errors. *Status 2026-10-16: repository [SoulInfernoDE/flint](https://github.com/SoulInfernoDE/flint) with CI; the Battlegrounds panel (lobby, last seen boards, own board, tavern tier and gold) runs in developer mode and was checked live in the game; on 2026-10-17 two full Battlegrounds games in a clean Linux Mint 22.3 VM (native client) confirmed lobby, combat opponent, last seen boards, own board, gold and the game end in step with the game; a third game confirmed the lobby's tribes (inferred from the minions seen, as the log does not name them) and the minion list by tier; a fourth game confirmed the session recap and the game history on Flint's launcher page, a fifth the keyword filter of the minion list. The Battlegrounds overlay is complete; next: own deck overlay.*
- [x] **Battlegrounds overlay:** tavern tier and gold, own board, heroes of the lobby with health and placement (leaderboard), the last opponent's board, available and banned tribes, minion list by tier, tribe and mechanic (card data from HearthstoneJSON), session recap (games, placements)
- [x] **Own deck** overlay: card tiles (art crop, mana cost, count, rarity), drawn cards greyed out, remaining cards, draw probabilities *(2026-10-17: the deck list comes from `Decks.log` for the user's own decks; loaner decks show the cards seen so far; checked in practice games)*
- [x] **Opponent** overlay: played cards, hand with draw turn, buffs and known cards *(2026-10-17: checked in practice games; buffs show which card changed a card in hand, as the new stats of hidden cards are not logged)*
- [ ] **Secrets helper**, counters (total attack, fatigue, corpses, spell schools …), turn timer *(2026-10-17: counters for board attack, fatigue, corpses and spells played are done; secrets helper, spell schools and turn timer are open)*
- [ ] Main window (Firestone-like): navigation on the left, **Battlegrounds** (games, heroes, placements), **Decks** section (deck code import, local stats per deck, matchups), **Replays** (local list)
- [ ] Settings: size, opacity, position, which zones are visible
- [ ] First public **alpha release** of overLay plus the mod
- **Acceptance:** full Battlegrounds games and a full Ranked game on the native Linux client (hearthstone-linux-gui), under Proton and under Windows are tracked correctly, within the resource budget.

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
3. ~~Create the GitHub repository and check the first CI run~~ ✓
4. **Technical spike** (largely done: native client on X11 works, see [results](spikes/overlay-window/RESULTS.md); still open: Proton, Wayland/KDE/GNOME, Windows): a transparent click-through Tauri window over the **native Linux client (hearthstone-linux-gui)** and over Hearthstone under Proton, on KDE and GNOME, and on Windows, with RAM measurements. Also confirm the native client's `log.config` and log locations. This is the riskiest assumption in the whole project and should be tested first.
