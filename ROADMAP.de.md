# overLay – Fahrplan

> Deutsche Fassung. Englisches Original: [ROADMAP.md](ROADMAP.md)
> Recherche-Grundlage: [docs/research.de.md](docs/research.de.md)
> Stand: 2026-10-14 · Status: Phase 0 abgeschlossen

## 1. Ziel

overLay ist ein **quelloffener Overlay-Manager** für Spiele unter **Linux (nativ, Wine, Proton)** und **Windows**:

- schlank, schnell, ohne Ballast. Die Oberfläche ist **stark an Overwolf angelehnt**, nutzt aber eigene Marke und eigene Assets.
- Er verwaltet **Overlay-Mods** aus einem öffentlichen Index-Repository und aus beliebig vielen **eigenen Repo-URLs**.
- Der erste Mod ist **Flint**, ein Hearthstone-Companion, der Aussehen und Funktionsumfang von **Firestone** nachbaut (clean-room, siehe Recherche §3.1). **Hauptziel von Flint ist der native Linux-Client**, gestartet über [hearthstone-linux-gui](https://github.com/DawnMagnet/hearthstone-linux-gui) (die offiziellen macOS-Spieldateien auf Unitys Linux-Player, ohne Wine, siehe Recherche §5.1). Wine/Proton und Windows werden ebenfalls unterstützt.

## 2. Leitprinzipien (nicht verhandelbar)

1. **Open Source überall.** Der Host steht unter freier Lizenz. Jeder Mod im offiziellen Index braucht öffentlichen Quellcode und eine OSI-Lizenz, und das Build-Artefakt muss aus dem Quellcode reproduzierbar sein.
2. **Keine Telemetrie, keine Werbung.** Weder im Host noch in Mods: keine Analytics, keine Ad-SDKs, keine Tracker. Erlaubter Netzverkehr:
   - signierten Mod-Index und Updates abrufen (abschaltbar)
   - Daten, die ein Mod **deklariert** und der Nutzer **bestätigt** hat (z. B. Kartendaten von HearthstoneJSON)
   - optionale Community-Statistiken **nur als Opt-in** und nur über einen offenen, selbst hostbaren Server
3. **Updates erst prüfen, dann aktivieren.** Ein Update wird heruntergeladen, Signatur und Hash werden geprüft, es wird bereitgestellt, und erst dann wird es aktiviert. Fordert ein Update neue Berechtigungen oder Domains an, muss der Nutzer zustimmen.
4. **Ressourcenbudget als Abnahmekriterium:**
   - Ohne laufendes Spiel läuft nur der native Kernprozess, keine WebView (Ziel: < 30 MB RAM)
   - Mit Overlay aktiv: ≤ 150 MB RAM gesamt, < 2 % CPU im Leerlauf (Spike über dem laufenden Spiel: 146 MB, 0,5 %)
   - Installer < 20 MB
5. **Keine Code-Injektion in Spiele** als Standard. Das Overlay läuft als eigenes Fenster bzw. eigene Oberfläche, nicht als DirectX- oder Vulkan-Hook. Das hält das Anti-Cheat-Risiko klein und funktioniert unter Proton.
6. **i18n von Anfang an.** Alle Texte stehen in Locale-Katalogen, Englisch ist Basis und Fallback, Deutsch ist die zweite Sprache. Das gilt für Host **und** Mods.
7. **Zweisprachige Doku.** README, Changelog und Release-Notes erscheinen auf Englisch mit deutscher Fassung.

## 3. Architektur (Vorschlag)

```
┌──────────────────────────── overLay Host (Rust) ────────────────────────────┐
│ Core-Daemon: Tray, Spieleerkennung, Hotkeys, Mod-Manager, Updater, Settings │
│   ├─ Game Providers (Rust-Crates): liefern Events + Zustand (wie Overwolf GEP)│
│   │     └─ hearthstone: Log-Parser, Wine-Präfix-Finder, Speicherleser (opt.)│
│   ├─ Overlay-Backends: Windows · X11/XWayland · Wayland layer-shell · gamescope│
│   └─ Sicherheits-Layer: Berechtigungen, Netz-Allowlist, CSP, Signaturen    │
├──────────────────────────────────────────────────────────────────────────────┤
│ UI (WebView via Tauri 2): Launcher/Dock, Bibliothek, Store, Einstellungen   │
│ Mods = Web-Bundles (HTML/JS/CSS) in sandboxed WebViews, API: `overlay.*`    │
└──────────────────────────────────────────────────────────────────────────────┘
          ▲ signierter Index (GitHub-Repo `overlay-mods-index`) + eigene Repo-URLs
```

**Begründung der Kernentscheidungen:**

| Entscheidung | Warum |
|---|---|
| **Tauri 2 (Rust + System-WebView)** statt Electron/CEF | Kein mitgeliefertes Chromium, deutlich weniger RAM und Speicherplatz. WebView2 unter Windows, WebKitGTK unter Linux. Die UI kann trotzdem Web-Technik nutzen, damit Mods einfach zu schreiben sind (wie bei Overwolf). |
| **Mods als Web-Bundles + Manifest**, Spiel-Daten vom Host | Mods führen keinen nativen Code aus. Speicherlesen, Dateizugriff und Prozesszugriff stecken in geprüften **Game Providers** im Host. So bleibt die Sandbox dicht, und Mods brauchen keine DLLs (anders als Overwolfs `extra-objects`). |
| **Später: Provider als WASM-Komponenten** (wasmtime, Capability-basiert) | Drittanbieter können Provider für neue Spiele liefern, ohne vollen Systemzugriff zu bekommen. |
| **Overlay als eigenes, transparentes Topmost-Fenster** | Funktioniert über randlosen Vollbild-Spielen (so macht es HDT seit Jahren), ohne Injektion und auch unter Proton. Exklusiver Vollbildmodus (Windows) wird bewusst nicht unterstützt. Durch den [Spike](spikes/overlay-window/RESULTS.de.md) über dem nativen Client bestätigt. |
| **Eine gemeinsame Overlay-WebView für alle Widgets aller Mods im Spiel** | Mit WebKitGTK startet jedes WebView-Fenster einen eigenen Webprozess (je etwa 50 MB, gemessen). Mods laufen in Sandbox-Frames mit eigenem Ursprung und eigener CSP innerhalb dieser einen WebView; außerhalb der Widgets gehen Klicks über die Input-Shape ans Spiel. Das Hauptfenster auf dem Desktop ist eine zweite WebView, die nur existiert, solange es geöffnet ist. |
| **Overlay nimmt nie den Fokus; Input-Shape in Gerätepixeln** | Lehren aus dem Spike: Das Overlay darf dem Spiel nicht den Tastaturfokus nehmen, und die klickbaren Bereiche müssen von CSS-Pixeln mit dem Device-Pixel-Ratio umgerechnet werden (z. B. 1,2 bei 115 dpi), sonst wandern sie von den sichtbaren Widgets weg und blockieren Knöpfe im Spiel. |
| **UI-Framework für Host und Flint: Svelte 5** (TypeScript, Vite) | Kompiliert zu kleinen Bundles ohne schwere Laufzeit (Firestone nutzt Angular). Mods sind trotzdem frei in der Wahl ihres Frameworks, weil die Mod-API reines Web-Standard-JavaScript ist. |
| **Lokale Daten: SQLite** (über Rust, `rusqlite`) | Spiele, Decks, Replays und Stats bleiben lokal. Es gibt keinen Account-Zwang. |

## 4. Phasen

Jede Phase endet mit einem lauffähigen, veröffentlichbaren Stand. Die Zeitangaben sind grobe Schätzungen für ein kleines Team in Teilzeit.

### Phase 0: Fundament *(1–2 Wochen)*
- [x] Git-Repo, Lizenz **GPL-3.0-or-later**, `CONTRIBUTING`, `CODE_OF_CONDUCT` (Contributor Covenant 3.0), `SECURITY.md`, jeweils EN + DE
- [x] **Mod-Richtlinie** ([MOD_POLICY.de.md](MOD_POLICY.de.md)): Quellcode-Pflicht, OSI-Lizenz, keine Telemetrie/Werbung, Netzwerk-Deklaration, kein nachgeladener Code, Fair Play, Clean-Room-Regel, Review-Prozess
- [x] Monorepo-Struktur: `crates/` (Rust), `ui/` (Host-UI), `locales/`, `sdk/` (Mod-SDK + Typen), `mods/` (Beispiel-Mods), `tools/`, `docs/`
- [x] CI: Build und Tests für Linux und Windows, Lints, Typprüfung, Lizenz- und Advisory-Scan (`cargo-deny`, `tools/check-npm-licenses.mjs`, `npm audit`), Actions auf Commit-Hashes gepinnt, Dependabot
- [x] i18n-Grundgerüst mit **Fluent**: gemeinsame Kataloge für Rust (`overlay-i18n`) und UI, Englisch als Fallback, Test auf vollständige Übersetzungen
- [x] Design-Tokens (`ui/src/styles/tokens.css`) und Icons auf Basis von Lucide (ISC) im dunklen Dock-Look
- Toolchain: Rust 1.99 (Edition 2024), Node 26, Svelte 5, Vite 8, TypeScript 6 (TypeScript 7 wird von svelte-check noch nicht unterstützt)

### Phase 1: Host-Kern & Launcher-Oberfläche *(4–6 Wochen)*
- [ ] Tauri-Shell mit **Tray-only-Modus** (keine WebView, solange das Fenster zu ist)
- [ ] Launcher-UI nach Overwolf-Vorbild: Dock links, **Bibliothek**, **Store** (vorerst nur lokal), **Einstellungen**, eigene Titelleiste
- [ ] **Spieleerkennung**: Prozesse unter Windows (WinAPI, kein `tasklist`-Polling), unter Linux über `/proc` für den **nativen Client** (`Hearthstone.x86_64`) und für Wine/Proton-Prozesse (`Hearthstone.exe` unter `wine64-preloader`/`wineserver`)
- [ ] Spiele-Katalog (`games/*.toml`): Prozessnamen, Fenstertitel, Launcher-Hinweise
- [ ] Globale Hotkeys (Windows: RegisterHotKey; Linux: XDG GlobalShortcuts-Portal, Fallback X11)
- [ ] Einstellungen persistent (TOML), Autostart optional
- **Abnahme:** Host startet, erkennt Hearthstone nativ und unter Proton, zeigt den Status im Launcher, Ressourcenbudget eingehalten.

### Phase 2: Overlay-Engine *(4–6 Wochen)*
- [ ] Overlay-Fenster: transparent, randlos, topmost, **click-through pro Region** (interaktive Bereiche fangen Eingaben ab, der Rest geht ans Spiel)
- [ ] Fenster an das Spielfenster koppeln: Position und Größe folgen, ausblenden bei Minimieren oder Fokusverlust
- [ ] Backends:
  - Windows (`WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE`)
  - **X11/XWayland** (Shape-Extension für Click-through, `_NET_WM_STATE_ABOVE`). Das ist der Standardpfad unter Proton, weil Proton-Spiele über XWayland laufen.
  - **Wayland layer-shell** (KDE, wlroots, Hyprland). GNOME fällt auf XWayland zurück.
  - **gamescope** (Steam Deck/Game Mode): External-Overlay-Atom, **später** (Phase 9)
- [ ] Overlay-Ein-/Ausblenden per Hotkey (wie Overwolf), Toast-Benachrichtigungen
- [ ] Widget-Positionen speichern (pro Spiel, pro Auflösung)
- **Abnahme:** Demo-Overlay über dem nativen Linux-Client sowie über Hearthstone unter Proton und unter Windows, auf KDE (Wayland + XWayland), GNOME (XWayland) und Sway. Klicks gehen ans Spiel durch, Widgets lassen sich ziehen.

### Phase 3: Mod-SDK & Laufzeit *(4–6 Wochen)*
- [ ] **`overlay.json`-Manifest** (angelehnt an Overwolf, aber schlanker):
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
- [ ] JS-API `overlay.*`: `games` (Start/Ende), `events` (`onEvent`, `onInfo`, `getInfo`, `requireFeatures`), `windows` (öffnen, verschieben, Click-through, Deckkraft), `storage` (Key-Value + SQLite pro Mod), `hotkeys`, `notifications`, `i18n`, `settings` (Schema → automatisch erzeugte Einstellungsseite)
- [ ] **Sandbox**: eigene WebView pro Mod-Fenster, strikte CSP, **Netz-Allowlist erzwungen im Host** (Requests an nicht deklarierte Domains werden blockiert und geloggt), kein Node, kein Dateisystemzugriff
- [ ] Nachrichtenbus zwischen den Fenstern eines Mods (Ersatz für Overwolfs Background Controller)
- [ ] **Developer-Modus**: Mod aus lokalem Ordner laden, Hot-Reload, DevTools, **Event-Recorder/-Player** (Spielsitzungen aufzeichnen und abspielen, wichtig für Tests ohne laufendes Spiel)
- [ ] SDK-Paket (`@overlay/sdk`) mit TypeScript-Typen, Vorlage `create-overlay-mod`
- [ ] *(optional)* Kompatibilitätsschicht für Teile der Overwolf-API, damit sich offene Overwolf-Apps leichter portieren lassen
- **Abnahme:** Beispiel-Mod „Hello Overlay“ mit Locale-Katalog läuft. Ein Request an eine nicht deklarierte Domain wird nachweislich blockiert.

### Phase 4: Mod-Repository & sichere Updates *(3–4 Wochen)*
- [ ] Separates Repo **`overlay-mods-index`**: eine Datei pro Mod (`mods/<id>.toml`) mit Quell-Repo, Releases (Git-Tag + Commit-Hash), Artefakt-URL, **SHA-256**, Berechtigungen, Domains, Lizenz
- [ ] **CI im Index-Repo** prüft jeden PR:
  1. Quell-Repo öffentlich, OSI-Lizenz vorhanden
  2. **Reproduzierbarer Build**: CI baut aus dem Tag, das Ergebnis muss dem Artefakt-Hash entsprechen
  3. Scan nach Tracker- und Ad-Domains und bekannten SDKs, Abgleich der Manifest-Domains mit dem Code
  4. Diff der Berechtigungen gegenüber der Vorversion → bei Änderungen manuelles Review
  5. Nach dem Merge signiert ein Maintainer bzw. CI das Index-Snapshot (**minisign/Ed25519**, Ideen aus TUF: Ablaufdatum, Versionsnummer gegen Rollback)
- [ ] Client: Index abrufen → Signatur prüfen → Mods anzeigen (Store-Seite mit Screenshots, Changelog, Berechtigungen, Quellcode-Link)
- [ ] **Update-Pipeline**: herunterladen → Hash prüfen → in `staging/` entpacken → Manifest validieren → neue Berechtigungen bestätigen lassen → atomar aktivieren (Rollback auf die Vorversion möglich)
- [ ] **Eigene Repos**: URL plus öffentlicher Schlüssel in den Einstellungen hinzufügen. Beim ersten Mal muss der Nutzer dem Schlüssel ausdrücklich vertrauen (Warnung). Mods aus eigenen Repos sind in der UI markiert.
- [ ] Abhängigkeiten mit semver-Bereichen (Lehre aus CurseForge), Release-Kanäle stable/beta
- **Abnahme:** Mod über den Index installieren, aktualisieren und zurückrollen. Ein manipuliertes Artefakt wird abgelehnt, ebenso ein Index mit falscher Signatur.

### Phase 5: Hearthstone Game Provider *(6–8 Wochen)*
- [ ] **Installations-Finder**, in dieser Reihenfolge:
  1. **nativer Linux-Client**: hearthstone-linux-gui (`game_dir` aus `$XDG_CONFIG_HOME/hearthstone-linux-gui/config.toml`, sonst `$XDG_DATA_HOME/hearthstone-linux-gui/game`), das ursprüngliche hearthstone-linux (frei gewählter Ordner) oder ein laufendes `Hearthstone.x86_64` (dessen Arbeitsverzeichnis)
  2. Wine/Proton-Präfixe: Steam-`compatdata`, Lutris, Bottles, Heroic, Faugus
  3. Windows: Registry/Battle.net-Config
  4. frei wählbarer Pfad
- [ ] `log.config` automatisch anlegen bzw. ergänzen (Power, Zone, LoadingScreen, Arena, Decks, Achievements …). Nativer Client: `~/Library/Preferences/Blizzard/Hearthstone/log.config` (im Spike bestätigt); die Logs erscheinen dann in `<game_dir>/Logs/Hearthstone_<Zeitstempel>/`; Wine: im Präfix
- [ ] Größenbegrenzung der Hearthstone-Logs (sie werden bei etwa 10 MB abgeschnitten) und Log-Rotation abfangen, ohne den Spielzustand zu verlieren
- [ ] **Power.log-Parser in Rust** (Referenz: python-hslog, HSTracker, beide MIT): Entities, Tags, Blocks, Zonenwechsel → Spielzustand und Events (`game-start`, `mulligan`, `card-drawn`, `card-played`, `turn-start`, `game-end` …)
- [ ] Spielmodus-Erkennung (Ranked/Casual, Arena, BG, Brawl, Mercs, Solo)
- [ ] Kartendatenbank: HearthstoneJSON herunterladen, Cache, Update bei neuem Patch. Kartenbilder werden **nur zur Laufzeit** geladen (Blizzard-IP).
- [ ] *(optional, per Opt-in)* **Speicherleser** auf Basis der Ideen von UnitySpy (MIT, eigene Rust-Portierung mit Namensnennung): Sammlung, aktives Deck, Arena-Draft, BG-Lobby, Quests, Achievements. Zuerst für den nativen Client: ein normaler ELF-Prozess mit `libmonobdwgc-2.0.so`, also ohne Wine-Adressumrechnung. Wine/Proton über `/proc/<pid>/mem` als zweiter Schritt.
- [ ] Event-Mitschnitte echter Partien als Testfixtures (Recorder aus Phase 3)
- **Abnahme:** Der Provider liefert für 50 aufgezeichnete Partien (alle Modi, überwiegend auf dem nativen Linux-Client aufgezeichnet) einen Spielzustand, der mit dem Endstand im Log übereinstimmt.

### Phase 6: Flint MVP: Deck-Tracker *(6–8 Wochen)*
Flint bekommt ein eigenes Repository und wird über den Index installiert. So durchläuft schon der erste Mod die komplette Pipeline. Aussehen orientiert an Firestone, mit eigenen Assets.
- [ ] Overlay **eigenes Deck**: Kartenkacheln (Bildausschnitt, Manakosten, Anzahl, Seltenheit), gezogene Karten ausgegraut, Restkarten, Ziehwahrscheinlichkeiten
- [ ] Overlay **Gegner**: gespielte Karten, Hand mit Ziehrunde, Buffs und bekannten Karten
- [ ] **Secrets-Helfer**, Zähler (Gesamtangriff, Fatigue, Leichen, Zauberschulen …), Rundentimer
- [ ] Hauptfenster (Firestone-artig): Navigation links, Bereich **Decks** (Deckcode-Import, lokale Stats pro Deck, Matchups), **Replays** (lokale Liste)
- [ ] Einstellungen: Größe, Deckkraft, Position, welche Zonen sichtbar sind
- [ ] Erstes öffentliches **Alpha-Release** von overLay plus Mod
- **Abnahme:** Eine komplette Ranked-Partie auf dem nativen Linux-Client (hearthstone-linux-gui), unter Proton und unter Windows wird korrekt verfolgt, und das Ressourcenbudget wird eingehalten.

### Phase 7: Firestone-Parität in Wellen *(fortlaufend)*
Reihenfolge nach Nutzen und Aufwand:
1. **Constructed+**: Synergie-Hervorhebung, verwandte Karten beim Hover, Mulligan-Ansicht (lokal), Deckbuilder, Rang-Verlauf, Ausspielreihenfolge, Recap spielentscheidender Karten
2. **Battlegrounds Basis**: Kartenliste nach Stufe, Stamm und Mechanik, aktive/gebannte Stämme, Infos zum letzten Gegner, Session-Recap, HP-Graph, Gegner-Boards, Zähler
3. **Arena**: Run-Historie, Draft-Overlay (Synergien, eigene Stats), Run-Recap
4. **Replays**: Play-by-Play-Viewer (eigene Implementierung, MIT-XML-Parser als Hilfe)
5. **Sammlung & Achievements** (benötigt den Speicherleser): Packs, Pity-Timer, Set-Vervollständigung, angepinnte Achievements
6. **Battlegrounds-Kampfsimulator**: **eigenes Teilprojekt**, weil es keinen frei lizenzierten gibt. Engine in Rust → WASM, Kartenmechaniken datengetrieben und mit Tests
7. Tavern Brawl, Mercenaries (sofern im Spiel noch relevant), Discord Rich Presence (lokal), Taskleisten-Hinweis „Du bist dran“

### Phase 8: Community-Statistiken *(optional, nach Grundsatzentscheidung)*
Meta-Decks, globale Mulligan-Werte, BG-Hero-Tierlisten und Arena-Kartenwerte brauchen **aggregierte Daten vieler Spieler**. Ohne Telemetrie geht das nur so:
- [ ] Offener, **selbst hostbarer** Stats-Server (eigenes Repo, AGPL), Upload **nur per ausdrücklichem Opt-in**, anonymisiert, das Datenformat ist dokumentiert
- [ ] Der Client kann auch nur Daten **beziehen**, ohne selbst Daten zu senden
- [ ] Alternativ oder zusätzlich: Import öffentlicher Datensätze, sofern deren Lizenz das erlaubt

### Phase 9: Release 1.0 & Verteilung *(3–4 Wochen)*
- [ ] Pakete: **AppImage, Flatpak (Flathub), .deb, AUR**, Windows **MSI + winget**
- [ ] gamescope/Steam-Deck-Unterstützung
- [ ] Self-Update des Hosts mit derselben Signaturkette wie bei Mods
- [ ] Doku-Seite (EN/DE), Mod-Entwickler-Handbuch, Screenshots EN/DE
- [ ] Security-Review der Sandbox und der Update-Kette

## 5. Risiken

| Risiko | Auswirkung | Gegenmaßnahme |
|---|---|---|
| Firestone hat keine Lizenz bzw. „All rights reserved“ | Urheberrechtsverletzung bei Code-Übernahme | strikt clean-room, nur MIT/ISC-Bausteine, Herkunft der Beiträge prüfen |
| Markenrecht/Trade Dress (Overwolf, Firestone, Blizzard) | Abmahnung | eigene Namen, Logos und Icons, keine mitgelieferten Blizzard-Assets |
| Wayland erlaubt kein globales Positionieren | Overlay funktioniert nicht überall | XWayland als Standard, layer-shell wo verfügbar, gamescope-Backend |
| Speicherlesen (ToS-Grauzone, ptrace-Rechte unter Linux) | Bann-Risiko (gering), Einrichtungsaufwand | nur Opt-in, nur lesend, nie schreiben oder injizieren; verständliche Anleitung zu `ptrace_scope` |
| Hearthstone-Patches ändern das Logformat bzw. Speicherstrukturen | Tracker fällt aus | Recorder-Fixtures, schnelle Provider-Updates über die signierte Pipeline |
| Der native Linux-Client ist inoffiziell (seine Autoren nennen ein Bann-Risiko, der Shop im Spiel ist geschlossen), und seine Pfade können sich ändern | Flints Hauptziel fällt aus oder Nutzer gehen ein Risiko ein | Flint liest nur und verändert den Client nie; `config.toml` des Launchers auslesen statt Pfade fest zu verdrahten; Releases verfolgen und mit den Maintainern eine stabile Schnittstelle abstimmen; Wine/Proton bleibt vollwertiger Ausweichweg |
| WebKitGTK-Performance und -Transparenz je nach Distribution | ruckelndes Overlay | frühe Benchmarks in Phase 2, Fallback mit Compositing-Flags |
| BG-Simulator-Aufwand (jede Karte) | Verzögerung | als eigenes Teilprojekt, auf Community-Beiträge ausgelegt |
| Globale Stats ohne Telemetrie | Feature-Lücke gegenüber Firestone | Phase 8 (Opt-in, selbst hostbar) |

## 6. Offene Entscheidungen

Entschieden am 2026-10-14: Lizenz **GPL-3.0-or-later**, Name des Hearthstone-Mods **Flint** (Feuerstein: der Stein, der Funken schlägt), UI-Framework **Svelte 5**.

Noch offen:

4. **Speicherleser** in Phase 5 schon dabei oder erst mit Phase 7.5?
5. **Community-Stats** (Phase 8): überhaupt umsetzen und, falls ja, wer betreibt den Server?
6. **Hosting**: GitHub (wie vorgeschlagen) oder zusätzlich bzw. stattdessen Forgejo/Codeberg?

## 7. Nächste Schritte

1. ~~Offene Entscheidungen 1–3 klären~~ ✓
2. ~~Phase 0 umsetzen~~ ✓
3. ~~GitHub-Repository anlegen und den ersten CI-Lauf prüfen~~ ✓
4. **Technischer Spike** (weitgehend erledigt: nativer Client unter X11 funktioniert, siehe [Ergebnisse](spikes/overlay-window/RESULTS.de.md); noch offen: Proton, Wayland/KDE/GNOME, Windows): transparentes Click-through-Tauri-Fenster über dem **nativen Linux-Client (hearthstone-linux-gui)** und über Hearthstone unter Proton, auf KDE und GNOME sowie unter Windows, mit RAM-Messung. Dazu die Orte von `log.config` und Logs beim nativen Client bestätigen. Das ist die riskanteste Annahme des ganzen Projekts und sollte zuerst geprüft werden.
