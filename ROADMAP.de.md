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
- [x] Tauri-Shell (`crates/overlay-app`) mit **Tray-only-Modus**: Im Infobereich gestartet (`--minimized`, so startet der Autostart) braucht sie **27 MB PSS** und keine WebView; das Launcher-Fenster entsteht bei Bedarf und wird beim Schließen zerstört; nur eine Instanz (ein zweiter Start öffnet das Fenster)
- [x] Speicher freigeben, den WebKit nach dem Schließen des Launcher-Fensters behält (81 MB PSS statt 27 MB): in Phase 2 gelöst, der Launcher läuft in einem eigenen Prozess, der mit dem Fenster endet
- [x] Launcher-UI nach Overwolf-Vorbild: Dock links, **Bibliothek** (Spiele mit Live-Status), **Store** (Platzhalter bis Phase 4), **Einstellungen**, eigene Titelleiste mit Ziehbereich, Minimieren und Schließen
- [x] **Spieleerkennung** (`crates/overlay-games`): ToolHelp-Snapshot unter Windows, `/proc` unter Linux für den **nativen Client** (`Hearthstone.x86_64`) und Wine/Proton (`Hearthstone.exe`), alle 2 s, Start-/Ende-Ereignisse an Oberfläche und Tray
- [x] Spiele-Katalog (`games/*.toml`, beim Build eingebettet): Prozessnamen je Plattform und Variante. Fenstertitel und Launcher-Hinweise folgen mit Phase 2 und 5
- [x] Globales Tastenkürzel zum Ein- und Ausblenden des Launchers (Standard `Strg+Umschalt+O`) unter Windows und X11
- [ ] Globale Tastenkürzel unter Wayland über das XDG-GlobalShortcuts-Portal
- [x] Einstellungen persistent (`crates/overlay-settings`, TOML im Konfigurationsordner, atomares Schreiben, kaputte Dateien werden beiseitegelegt), Sprache, Start im Infobereich, optionaler Autostart
- [x] Tray-Menü und alle Launcher-Texte übersetzt (Englisch, Deutsch), folgen der Systemsprache, solange der Nutzer keine wählt
- **Abnahme:** Host startet, erkennt Hearthstone nativ und unter Proton, zeigt den Status im Launcher, Ressourcenbudget eingehalten. *Stand 2026-10-14: erfüllt für den nativen Client unter Linux (X11); Proton und Windows noch zu testen.*

### Phase 2: Overlay-Engine *(4–6 Wochen)*
- [x] **Prozessmodell:** Der Kern (Tray, Erkennung, Einstellungen, Tastenkürzel) lädt keine WebView; Launcher und Overlay sind Kindprozesse desselben Programms (`--ui launcher`, `--ui overlay`), die über stdin/stdout mit dem Kern sprechen (`crates/overlay-ipc`, JSON-Zeilen) und enden, sobald sie nicht mehr gebraucht werden. Nach dem Schließen des Launchers ist der Kern wieder bei 27 MB PSS (vorher 81 MB)
- [x] **Ein Overlay-Fenster** pro laufendem Spiel für alle Widgets: transparent, randlos, oben, nimmt nie den Fokus, **Click-through pro Region** (Widgets nehmen Eingaben an, alles andere geht ans Spiel; Bereiche mit dem Device-Pixel-Ratio umgerechnet)
- [x] Folgt dem Spielfenster: unter X11 über Ereignisse (`ConfigureNotify`, Eigenschaftsänderungen, keine Abfrage im Takt), unter Windows per Abfrage alle 50 ms; sichtbar, solange das Spiel angezeigt wird und kein anderes aktives Fenster es überdeckt (Arbeiten auf einem anderen Monitor lässt das Overlay also stehen)
- [ ] Backends:
  - [x] **X11/XWayland** (GTK-Input-Shape, Stapelung `above` oder Override-Redirect). Standard unter Proton und für den nativen Hearthstone-Client, die beide über X11/XWayland laufen; getestet unter Cinnamon
  - [ ] Windows (immer oben, Click-through über die Mausposition): umgesetzt und in der CI kompiliert, **unter Windows noch nicht getestet**
  - [ ] **Wayland layer-shell** (KDE, wlroots, Hyprland) für native Wayland-Spiele; für Hearthstone nicht nötig
  - [ ] **gamescope** (Steam Deck/Game Mode): External-Overlay-Atom, **später** (Phase 9)
- [x] Overlay per Tastenkürzel ein- und ausblenden (`Strg+Umschalt+L`), Hinweis-Einblendungen (Start des Overlays, Overlay wieder eingeblendet)
- [x] Widget-Positionen pro Spiel und Auflösung gespeichert (`layout.toml`); eine Auflösung ohne eigenen Eintrag rechnet den nächstgelegenen gespeicherten proportional um (z. B. Vollbild → Fenster)
- [x] Eingebautes Demo-Widget (Einstellung), um das Overlay ohne Mods zu testen
- **Abnahme:** Demo-Overlay über dem nativen Linux-Client sowie über Hearthstone unter Proton und unter Windows, auf KDE (Wayland + XWayland), GNOME (XWayland) und Sway. Klicks gehen ans Spiel durch, Widgets lassen sich ziehen. *Stand 2026-10-14: erfüllt für den nativen Client unter Cinnamon (X11), 174 MB PSS für Kern + Overlay bei laufendem Spiel; Proton, Windows, KDE, GNOME und Sway noch zu testen.*
- Bekanntes Verhalten von Hearthstone, nicht durch overLay verursacht: Wird das Spielfenster im Fenstermodus in der Größe verändert, zoomt Hearthstone seine Ansicht (genauso bei abgeschaltetem Overlay).

### Phase 3: Mod-SDK & Laufzeit *(4–6 Wochen)*
- [x] **`overlay.json`-Manifest** (`crates/overlay-mods`), streng geprüft: Reverse-DNS-ID, Semver, SPDX-Lizenz, https-Quelle, `games`, `permissions` (`game.state`, `game.events`, `storage`, `notifications`, `hotkeys`), `network` (einfache Host-Namen), Overlay- und Desktop-`windows`, `hotkeys`, `settings`-Schema. Referenz: [docs/mod-development.de.md](docs/mod-development.de.md)
- [x] **Sandbox** (geprüft im [Spike mod-sandbox](spikes/mod-sandbox/RESULTS.de.md)): Mod-Fenster sind Sandbox-Frames in der einen Overlay-WebView (und im Launcher), ausgeliefert vom Host über `mod://`; CSP pro Mod aus dem Manifest, sodass **die WebView die Netzwerk-Allowlist durchsetzt** und blockierte Anfragen an den Host meldet, der sie protokolliert; kein `eval`, kein Zugriff auf die Seiten von overLay, Dateien nur aus dem eigenen Mod-Ordner (auch nicht über symbolische Links)
- [x] SDK-Paket `@overlay/sdk` (`sdk/`, TypeScript): `ready`, `i18n` (eigene Fluent-Kataloge des Mods), `storage` (JSON-Schlüssel-Wert, 1 MB), `notifications`, `bus` (zwischen den Fenstern eines Mods, prozessübergreifend), `hotkeys` (im Manifest deklariert, vom Kern registriert), `settings` (erzeugte Einstellungsseite im Launcher), `game` (Zustand und Fenstergröße), `events` (vorbereitet für Phase 5), `windows` (Ziehen, Klickbereiche, Click-through). Jeder Aufruf wird im Kern auf Berechtigungen geprüft
- [x] **Entwicklermodus**: Mods aus lokalen Ordnern laden, neu laden, Ladefehler in den Einstellungen, `OVERLAY_DEBUG=1` protokolliert Mod-Anfragen und Änderungen des Spielfensters
- [x] Beispiel-Mod [`mods/hello-overlay`](mods/hello-overlay) (EN/DE)
- [x] Launcher: Mods in der Bibliothek (an/aus), Mod-Seite mit Beschreibung, Berechtigungen, Netzwerk-Hosts, erzeugten Einstellungen und dem Desktop-Fenster des Mods
- [ ] Event-Recorder/-Player (Spielsitzungen aufzeichnen und abspielen) – braucht Spielereignisse, wandert in Phase 5
- [ ] Vorlage `create-overlay-mod`
- [ ] SQLite-Speicher pro Mod (der Schlüssel-Wert-Speicher reicht vorerst)
- [ ] Automatisches Neuladen, wenn sich ein Entwickler-Ordner ändert (jetzt: Knopf *Mods neu laden*)
- [ ] *(optional)* Kompatibilitätsschicht für Teile der Overwolf-API, damit sich offene Overwolf-Apps leichter portieren lassen
- **Abnahme:** Beispiel-Mod „Hello Overlay“ mit Locale-Katalog läuft. Ein Request an eine nicht deklarierte Domain wird nachweislich blockiert. *Stand 2026-10-15: erfüllt unter Linux (X11) über dem nativen Hearthstone-Client; die blockierte Anfrage an example.com wurde gemeldet und protokolliert.*

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
`crates/overlay-hearthstone`, ausgeführt vom Kern, solange das Spiel läuft.
- [x] **Installations-Finder** für das laufende Spiel: nativer Linux-Client (Arbeitsverzeichnis oder Elternordner von `Bin/`), Wine/Proton (`WINEPREFIX` bzw. `STEAM_COMPAT_DATA_PATH`, Arbeitsverzeichnis oder Windows-Pfad der Programmdatei), Windows (Pfad der Programmdatei). Im Betrieb geprüft für den nativen Client; Wine/Proton und Windows sind umgesetzt, aber noch zu testen
- [ ] Installations-Finder ohne laufendes Spiel (Launcher-Konfiguration, Steam-`compatdata`, Lutris, Bottles, Heroic, Faugus, frei wählbarer Pfad), z. B. um `log.config` vor dem ersten Start vorzubereiten
- [x] `log.config` wird automatisch angelegt bzw. ergänzt (`Power` mit `Verbose`, `LoadingScreen`, `Decks`, `Arena`), Einstellungen des Nutzers bleiben erhalten; hat sie sich geändert, bittet ein Hinweis um einen einmaligen Neustart des Spiels (Hinweise warten jetzt im Kern, bis das Overlay sie anzeigt)
- [x] Größenbegrenzung und Rotation der Logs: abgeschnittene Logs werden von vorne gelesen, einem neuen Ordner `Logs/Hearthstone_<Zeitstempel>/` folgt der Provider automatisch
- [x] **Log-Größenbegrenzung aufgehoben** (`client.config`: `[Log] FileSizeLimit.Int=-1`, beim nativen Client in der flachen Form `Log.FileSizeLimit.Int=-1`, zusätzlich in `client.config.overrides` von hearthstone-linux-gui, damit Updates des Launchers sie behalten). Gefunden am 2026-10-16: Sobald irgendein Log (hier `Zone.log`) 10 MB erreicht, schreibt Hearthstone *gar keine* Logs mehr; `Power.log` endete so sieben Minuten nach Beginn einer Schlachtfeld-Partie, und das Spielende kam nie an. overLay schaltet keine Logs mehr ein, die es nicht liest (`Zone`, `Asset`)
- [x] **Power.log-Parser in Rust** (eigene Umsetzung auf dem `GameState`-Strom): Entitätenmodell mit allen Tags, Zuordnung der Spielernamen, die in Partien mit mehr als zwei Spielern nie rät, Ereignisse `game-start`, `mulligan`, `turn-start`, `zone-change`, `card-revealed`, `game-end` und eine Spielübersicht (`game-info`) für Mods, die mitten in der Partie starten. SDK: `overlay.events.on`, `getInfo`, `onInfo` mit TypeScript-Typen
- [x] Spielmodus aus dem Spielkopf (`GameType`, `FormatType`, `ScenarioID`): Rangliste/Gelegenheit, Arena, Schlachtfeld, Kartenchaos, Solo …
- [x] Kosten: 0,5 % CPU im Leerlauf, Ereignisse alle 100 ms, solange das Spiel schreibt; das erste Einlesen einer langen Schlachtfeld-Partie dauert einen Moment
- [ ] Deckliste zum Spielstart (`Decks.log`) für den Deck-Tracker
- Kartendatenbank: **nicht im Host**. Mods laden Kartendaten selbst von den Hosts, die sie deklarieren (z. B. HearthstoneJSON), damit der Nutzer diesen Netzwerkzugriff sieht und bestätigt; Kartenbilder nur zur Laufzeit (Blizzard-IP)
- [ ] *(optional, per Opt-in)* **Speicherleser** auf Basis der Ideen von UnitySpy (MIT, eigene Rust-Portierung mit Namensnennung): Sammlung, aktives Deck, Arena-Draft, BG-Lobby, Quests, Achievements. Zuerst für den nativen Client: ein normaler ELF-Prozess mit `libmonobdwgc-2.0.so`, also ohne Wine-Adressumrechnung. Wine/Proton über `/proc/<pid>/mem` als zweiter Schritt.
- [ ] Event-Recorder/-Player für die Mod-Entwicklung ohne laufendes Spiel
- **Abnahme:** Der Provider liefert für 50 aufgezeichnete Partien (alle Modi, überwiegend auf dem nativen Linux-Client aufgezeichnet) einen Spielzustand, der mit dem Endstand im Log übereinstimmt. *Stand 2026-10-16: Unit-Tests mit einer von Hand geschriebenen Partie; echte Partien lokal geprüft mit `cargo run -p overlay-hearthstone --example replay -- <Power.log>` (eine Partie gegen die KI und eine Schlachtfeld-Partie, Seiten, Helden und Zonen stimmen) und live in einer Schlachtfeld-Partie. Echte Logs bleiben lokal, weil sie Kontodaten enthalten; die Sammlung wächst mit jeder gespielten Partie.*

### Phase 6: Flint MVP: zuerst Schlachtfeld, dann der Deck-Tracker *(6–8 Wochen)*
Flint bekommt ein eigenes Repository und wird über den Index installiert. So durchläuft schon der erste Mod die komplette Pipeline. Aussehen orientiert an Firestone, mit eigenen Assets. **Der Schlachtfeld-Modus hat Vorrang** (entschieden am 2026-10-16) und muss fehlerfrei funktionieren. *Stand 2026-10-16: Repository [SoulInfernoDE/flint](https://github.com/SoulInfernoDE/flint) mit CI; das Schlachtfeld-Panel (Lobby, zuletzt gesehene Bretter, eigenes Brett, Tavernenstufe und Gold) läuft im Entwicklermodus und wurde live im Spiel geprüft; am 2026-10-17 bestätigten zwei komplette Schlachtfeld-Partien in einer frischen Linux-Mint-22.3-VM (nativer Client) Lobby, Kampfgegner, zuletzt gesehene Bretter, eigenes Brett, Gold und Spielende im Takt des Spiels; eine dritte Partie bestätigte die Völker der Lobby (abgeleitet aus den gesehenen Dienern, weil das Log sie nicht nennt) und die Dienerliste nach Stufe; eine vierte Partie bestätigte Sitzungsrückblick und Spielverlauf auf Flints Launcher-Seite, eine fünfte den Filter nach Schlüsselwörtern in der Dienerliste. Das Schlachtfeld-Overlay ist vollständig; als Nächstes: Overlay fürs eigene Deck.*
- [x] **Schlachtfeld-Overlay:** Tavernenstufe und Gold, eigenes Brett, Helden der Lobby mit Leben und Platzierung (Rangliste), Brett des letzten Gegners, verfügbare und gebannte Völker, Dienerliste nach Stufe, Volk und Mechanik (Kartendaten von HearthstoneJSON), Sitzungsrückblick (Partien, Platzierungen)
- [x] Overlay **eigenes Deck**: Kartenkacheln (Bildausschnitt, Manakosten, Anzahl, Seltenheit), gezogene Karten ausgegraut, Restkarten, Ziehwahrscheinlichkeiten *(2026-10-17: Die Deckliste kommt bei eigenen Decks aus `Decks.log`; Leihdecks zeigen die bisher gesehenen Karten; in Übungspartien geprüft)*
- [ ] Overlay **Gegner**: gespielte Karten, Hand mit Ziehrunde, Buffs und bekannten Karten *(2026-10-17: gespielte Karten, Hand mit Ziehrunde, Starthand, Münze und bekannte Karten sind fertig und in Übungspartien geprüft; Buffs auf Handkarten fehlen noch)*
- [ ] **Secrets-Helfer**, Zähler (Gesamtangriff, Fatigue, Leichen, Zauberschulen …), Rundentimer
- [ ] Hauptfenster (Firestone-artig): Navigation links, **Schlachtfeld** (Partien, Helden, Platzierungen), Bereich **Decks** (Deckcode-Import, lokale Stats pro Deck, Matchups), **Replays** (lokale Liste)
- [ ] Einstellungen: Größe, Deckkraft, Position, welche Zonen sichtbar sind
- [ ] Erstes öffentliches **Alpha-Release** von overLay plus Mod
- **Abnahme:** Komplette Schlachtfeld-Partien und eine komplette Ranked-Partie auf dem nativen Linux-Client (hearthstone-linux-gui), unter Proton und unter Windows werden korrekt verfolgt, und das Ressourcenbudget wird eingehalten.

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
