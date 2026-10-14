# overLay

> Deutsche Fassung. Englisches Original: [README.md](README.md)

**overLay** ist ein schlanker, quelloffener Overlay-Manager für Spiele unter **Linux (nativ, Wine, Proton)** und **Windows**. Er installiert und startet **Overlay-Mods**: kleine Web-Apps, die Live-Informationen über dem Spiel anzeigen, zum Beispiel einen Deck-Tracker für Hearthstone.

Der Launcher orientiert sich am Dock-Look bekannter Overlay-Plattformen, overLay ist aber klein und transparent gebaut:

- **Keine Telemetrie, keine Werbung.** Der Host und alle Mods im offiziellen Index kommen ohne Analytics, Tracker und Werbe-SDKs aus.
- **Nur Open Source.** Jeder Mod im offiziellen Index hat öffentlichen Quellcode, eine OSI-anerkannte Lizenz und einen reproduzierbaren Build.
- **Geprüfte Updates.** Updates werden heruntergeladen, gegen signierte Hashes geprüft und erst danach aktiviert. Neue Berechtigungen musst du bestätigen.
- **Auch eigene Repos.** Neben dem offiziellen Index kannst du eigene Mod-Repositories per URL und öffentlichem Schlüssel hinzufügen.
- **Leichtgewichtig.** Nativer Rust-Kern mit der System-WebView, keine mitgelieferte Browser-Engine und keine Code-Injektion in Spiele.

> **Status: Pre-Alpha (Phase 0).** Noch nichts ist benutzbar. Siehe [Fahrplan](ROADMAP.de.md).

## Erster Mod: Flint

**Flint** ist ein Hearthstone-Companion mit Deck-Tracker, Gegner-Tracker sowie Helfern für Battlegrounds und Arena. Funktionsumfang und Optik orientieren sich an Firestone, Flint ist aber eine unabhängige Clean-Room-Implementierung. Hauptziel ist der **native Linux-Client**, gestartet über [hearthstone-linux-gui](https://github.com/DawnMagnet/hearthstone-linux-gui) (ohne Wine); Wine/Proton und Windows werden ebenfalls unterstützt. Flint bekommt ein eigenes Repository und wird über den Mod-Index installiert.

## Aufbau des Repositorys

| Pfad | Inhalt |
|---|---|
| `crates/` | Rust-Crates des Hosts (`overlay-i18n`, weitere folgen) |
| `ui/` | Launcher-Oberfläche (Svelte 5, TypeScript, Vite) |
| `locales/` | Fluent-Kataloge des Hosts, gemeinsam für Rust und UI (`en` als Basis, `de`) |
| `sdk/` | Mod-SDK und TypeScript-Typen (Phase 3) |
| `mods/` | Beispiel-Mods (Phase 3) |
| `tools/` | Skripte für das Repository |
| `docs/` | Recherche und Design-Dokumente |

## Entwicklung

Voraussetzungen: [rustup](https://rustup.rs) (installiert die gepinnte Toolchain automatisch) und Node.js 26 (siehe `.nvmrc`).

```bash
npm ci
```

```bash
npm run dev
```

```bash
cargo test
```

Alle Prüfungen, die auch die CI ausführt:

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo deny check && npm run format:check && npm run check && npm run build && npm run licenses
```

## Dokumentation

- [Fahrplan](ROADMAP.de.md)
- [Recherche: Overwolf, CurseForge, Firestone](docs/research.de.md)
- [Mod-Richtlinie](MOD_POLICY.de.md)
- [Mitwirken](CONTRIBUTING.de.md) · [Verhaltenskodex](CODE_OF_CONDUCT.de.md) · [Sicherheit](SECURITY.de.md)
- [Änderungsprotokoll](CHANGELOG.de.md)

## Lizenz

overLay steht unter der [GNU General Public License v3.0 oder neuer](LICENSE).

overLay steht in keiner Verbindung zu Overwolf, CurseForge, Blizzard Entertainment oder Zero to Heroes (Firestone) und wird von ihnen nicht unterstützt. Alle Marken gehören ihren jeweiligen Inhabern.
