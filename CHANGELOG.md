# Changelog

> English original. German version: [CHANGELOG.de.md](CHANGELOG.de.md)

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Roadmap after the overlay spike: one shared overlay webview for all mods, the overlay never takes focus, clickable regions are converted by the device pixel ratio; the native client's `log.config` and log locations are confirmed
- Roadmap and research: the native Linux Hearthstone client (hearthstone-linux-gui, macOS game files on Unity's Linux player) is now Flint's primary target, with its process, path and log details

### Added

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
