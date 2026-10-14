# Änderungsprotokoll

> Deutsche Fassung. Englisches Original: [CHANGELOG.md](CHANGELOG.md)

Alle nennenswerten Änderungen werden hier festgehalten. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), die Versionen folgen [Semantic Versioning](https://semver.org/lang/de/).

## [Unveröffentlicht]

### Geändert

- Fahrplan nach dem Overlay-Spike: eine gemeinsame Overlay-WebView für alle Mods, das Overlay nimmt nie den Fokus, klickbare Bereiche werden mit dem Device-Pixel-Ratio umgerechnet; Orte von `log.config` und Logs beim nativen Client bestätigt
- Fahrplan und Recherche: Der native Linux-Client von Hearthstone (hearthstone-linux-gui, macOS-Spieldateien auf Unitys Linux-Player) ist jetzt Hauptziel von Flint, mit Details zu Prozess, Pfaden und Logs

### Hinzugefügt

- Host-App (`overlay-app`, Tauri 2): Tray-Symbol mit übersetztem Menü, Launcher-Fenster bei Bedarf, nur eine Instanz, globales Tastenkürzel `Strg+Umschalt+O`, optionaler Autostart; 27 MB PSS beim Start im Infobereich
- Spieleerkennung (`overlay-games`) für den nativen Linux-Client, Wine/Proton und Windows, gesteuert über den Spielekatalog in `games/`
- Persistente Einstellungen (`overlay-settings`): Sprache, Start im Infobereich, Autostart, Tastenkürzel
- Launcher-Oberfläche: Spieleliste mit Live-Status, Einstellungsseite, eigene Titelleiste; läuft im normalen Browser mit Beispieldaten
- Spike für das Overlay-Fenster (`spikes/overlay-window`): transparentes, durchklickbares Tauri-2-Fenster, das dem Hearthstone-Fenster folgt, mit Messskript und ersten Ergebnissen
- Logos für overLay und Flint (SVG für dunkle und helle Hintergründe, PNG-Icons von 16 bis 512 px, CC BY-SA 4.0) mit reproduzierbarem Generator-Skript; das overLay-Icon dient als Favicon und erscheint in der Titelleiste
- Fundament aus Phase 0: Lizenz GPL-3.0-or-later, Fahrplan, Recherche, Mod-Richtlinie, Leitfaden zum Mitwirken, Verhaltenskodex (Contributor Covenant 3.0) und Sicherheitsrichtlinie, jeweils auf Englisch und Deutsch
- Cargo-Workspace (Rust 1.99, Edition 2024) mit `overlay-i18n`: Fluent-Kataloge mit englischer Fallback-Kette und Vollständigkeitsprüfung über alle Sprachen
- Grundgerüst der Launcher-Oberfläche (Svelte 5, Vite 8, TypeScript 6): Dock, Bereiche Bibliothek, Store und Einstellungen, Design-Tokens, Sprachumschaltung
- Gemeinsame Fluent-Kataloge in `locales/` (Englisch, Deutsch)
- CI für Linux und Windows: Formatierung, Lints, Tests, Typprüfung, Build, Lizenz- und Advisory-Prüfung; Dependabot für Cargo, npm und GitHub Actions
