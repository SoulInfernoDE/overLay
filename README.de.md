# overLay

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/branding/overlay/logo-dark.svg">
    <img src="assets/branding/overlay/logo-light.svg" alt="overLay" width="360">
  </picture>
</p>

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

<img src="assets/branding/flint/icon.svg" width="64" align="right" alt="">

**Flint** ist ein Hearthstone-Companion mit Deck-Tracker, Gegner-Tracker sowie Helfern für Battlegrounds und Arena. Funktionsumfang und Optik orientieren sich an Firestone, Flint ist aber eine unabhängige Clean-Room-Implementierung. Hauptziel ist der **native Linux-Client**, gestartet über [hearthstone-linux-gui](https://github.com/DawnMagnet/hearthstone-linux-gui) (ohne Wine); Wine/Proton und Windows werden ebenfalls unterstützt. Flint bekommt ein eigenes Repository und wird über den Mod-Index installiert.

## Aufbau des Repositorys

| Pfad | Inhalt |
|---|---|
| `crates/` | Rust-Crates des Hosts: `overlay-app` (Tauri-App), `overlay-games`, `overlay-settings`, `overlay-i18n` |
| `ui/` | Launcher-Oberfläche (Svelte 5, TypeScript, Vite) |
| `locales/` | Fluent-Kataloge des Hosts, gemeinsam für Rust und UI (`en` als Basis, `de`) |
| `sdk/` | Mod-SDK und TypeScript-Typen (Phase 3) |
| `mods/` | Beispiel-Mods (Phase 3) |
| `assets/branding/` | Logos von overLay und Flint ([Details](assets/branding/README.de.md)) |
| `games/` | Spielekatalog: welche Prozesse zu welchem Spiel gehören |
| `tools/` | Skripte für das Repository (Logo-Generator, `measure.sh` für RAM/CPU) |
| `docs/` | Recherche und Design-Dokumente |
| `spikes/` | Wegwerf-Messprojekte, nicht Teil der CI |

## Entwicklung

Voraussetzungen: [rustup](https://rustup.rs) (installiert die gepinnte Toolchain automatisch) und Node.js 26 (siehe `.nvmrc`). Unter Linux braucht Tauri außerdem einige Systembibliotheken (Debian/Ubuntu/Mint):

```bash
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
```

JavaScript-Abhängigkeiten installieren:

```bash
npm ci
```

Die Host-App mit Hot Reload starten:

```bash
npm run app:dev
```

Installierbare Pakete bauen (`.deb`, AppImage, MSI):

```bash
npm run app:build
```

Nur an der Oberfläche im Browser arbeiten (Beispieldaten statt Host):

```bash
npm run dev
```

Rust-Tests ausführen:

```bash
cargo test
```

Alle Prüfungen, die auch die CI ausführt:

```bash
npm run format:check && npm run check && npm run build && npm run licenses && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo deny check
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
