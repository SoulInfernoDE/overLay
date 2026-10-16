# Changelog

> English original. German version: [CHANGELOG.de.md](CHANGELOG.de.md)

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- The overlay stays visible while the game is shown and no other active window covers it, so working on another monitor keeps it (it used to hide whenever the game lost focus)
- Roadmap after the overlay spike: one shared overlay webview for all mods, the overlay never takes focus, clickable regions are converted by the device pixel ratio; the native client's `log.config` and log locations are confirmed
- Roadmap and research: the native Linux Hearthstone client (hearthstone-linux-gui, macOS game files on Unity's Linux player) is now Flint's primary target, with its process, path and log details

### Added

- Hearthstone game provider (phase 5, `overlay-hearthstone`): finds the logs of the native client, Wine/Proton and Windows, extends `log.config`, follows `Power.log` across truncation and new log folders, and turns it into game events and a game summary for mods with `game.events` (`overlay.events.on`, `getInfo`, `onInfo` with TypeScript types); `replay` and `locate` examples for checking real logs locally
- Toasts wait in the core until the overlay page takes them and stay back while the overlay is hidden, so none is lost
- Mod SDK and runtime (phase 3): `overlay.json` manifests with strict validation (`overlay-mods`); mods run as sandboxed frames served over `mod://` with a per-mod CSP, so undeclared network hosts are blocked by the webview and reported to the core; SDK `@overlay/sdk` with translations, storage, notifications, message bus, hotkeys, settings, game state and draggable, partly click-through overlay windows; permission checks in the core; developer mode for local mod folders; mods in the launcher library with a mod page (permissions, network hosts, generated settings, desktop window); example mod `hello-overlay`; guide for mod authors (`docs/mod-development.md`)
- In-game overlay (phase 2): one transparent, click-through window per running game that follows the game window through X11 events, shows only while the game is active, never takes focus and can be toggled with `Ctrl+Shift+L`; toast notifications; a demo widget whose position is saved per game and resolution
- Process model: the core runs without a webview, the launcher and the overlay are child processes that talk to it over stdin/stdout (`overlay-ipc`); closing the launcher brings the core back to 27 MB PSS
- `OVERLAY_DEBUG=1` logs every change of the game window
- Host app (`overlay-app`, Tauri 2): tray icon with localized menu, launcher window on demand, single instance, global hotkey `Ctrl+Shift+O`, optional autostart; 27 MB PSS when started in the tray
- Game detection (`overlay-games`) for the native Linux client, Wine/Proton and Windows, driven by the game catalog in `games/`
- Persistent settings (`overlay-settings`): language, start in tray, autostart, hotkey
- Launcher UI: game list with live status, settings page, custom title bar; runs with sample data in a plain browser
- Overlay window spike (`spikes/overlay-window`): transparent click-through Tauri 2 window that follows the Hearthstone window, with a measurement script and first results
- Logos for overLay and Flint (SVG for dark and light backgrounds, PNG icons from 16 to 512 px, CC BY-SA 4.0) with a reproducible generator script; the overLay icon is used as favicon and in the title bar
- Phase 0 foundation: GPL-3.0-or-later license, roadmap, research notes, mod policy, contributing guide, code of conduct (Contributor Covenant 3.0) and security policy, each in English and German
- Cargo workspace (Rust 1.99, edition 2024) with `overlay-i18n`: Fluent catalogs with an English fallback chain and a completeness check across locales
- Launcher UI scaffold (Svelte 5, Vite 8, TypeScript 6): dock, library, store and settings sections, design tokens, language switch
- Shared Fluent catalogs in `locales/` (English, German)
- CI for Linux and Windows: formatting, lints, tests, type checks, build, license and advisory checks; Dependabot for Cargo, npm and GitHub Actions
