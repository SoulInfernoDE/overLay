# Changelog

> English original. German version: [CHANGELOG.de.md](CHANGELOG.de.md)

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Roadmap and research: the native Linux Hearthstone client (hearthstone-linux-gui, macOS game files on Unity's Linux player) is now Flint's primary target, with its process, path and log details

### Added

- Phase 0 foundation: GPL-3.0-or-later license, roadmap, research notes, mod policy, contributing guide, code of conduct (Contributor Covenant 3.0) and security policy, each in English and German
- Cargo workspace (Rust 1.99, edition 2024) with `overlay-i18n`: Fluent catalogs with an English fallback chain and a completeness check across locales
- Launcher UI scaffold (Svelte 5, Vite 8, TypeScript 6): dock, library, store and settings sections, design tokens, language switch
- Shared Fluent catalogs in `locales/` (English, German)
- CI for Linux and Windows: formatting, lints, tests, type checks, build, license and advisory checks; Dependabot for Cargo, npm and GitHub Actions
