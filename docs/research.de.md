# Recherche: Overwolf, CurseForge, Firestone

> Deutsche Fassung. Englisches Original: [research.md](research.md)
> Stand der Recherche: 2026-10-14

Dieses Dokument hält fest, was über die Vorbilder bekannt ist, was davon öffentlich ist und was overLay daraus übernehmen darf und was nicht.

---

## 1. Overwolf

### 1.1 Was ist offen, was nicht?

| Teil | Status |
|---|---|
| Overwolf-Client (Desktop, Overlay-Injektion, GEP) | **Closed Source**, nur Windows |
| Overwolf-App-API (`overwolf.*`) | öffentlich **dokumentiert** (dev.overwolf.com), Typen in `overwolf/overwolf-api-ts` (ohne Lizenz) |
| `@overwolf/ow-electron` | Fork von Electron, npm-Paket MIT; die Overlay-/GEP-/Recorder-Pakete werden aber als Binärdateien nachgeladen und sind **nicht offen** |
| `@overwolf/ow-electron-packages-types` | MIT (nur TypeScript-Typen) |
| Sample-Apps (`overwolf/sample-app`) | öffentlich, **ohne Lizenz** |

**Folgerung:** Von Overwolf lässt sich nur das **Konzept** übernehmen (Manifest, Fenstermodell, Event-API, Berechtigungen). Code oder Grafiken dürfen nicht übernommen werden. Eine eigene Neuimplementierung einer API-Schnittstelle ist in der EU grundsätzlich zulässig, weil Schnittstellen und Funktionalität nicht urheberrechtlich geschützt sind (EuGH, SAS Institute, C-406/10).

### 1.2 Architektur von Overwolf Native

- **Laufzeit:** Chromium Embedded Framework (CEF). Jede App ist eine Web-App (HTML/JS/CSS) mit mehreren Fenstern.
- **Background Controller:** Ein unsichtbares Fenster (`is_background_page`) hält den Zustand und steuert die sichtbaren Fenster. Diese reichen Daten über `overwolf.windows.getMainWindow()` und Messaging weiter.
- **`manifest.json`**, das zentrale Konzept:
  - `meta`: Name, Autor (daraus wird die App-ID abgeleitet), Version, Icons, `minimum-overwolf-version`
  - `permissions`: z. B. `GameInfo`, `GameControl`, `Hotkeys`, `FileSystem`, `Clipboard`, `Tray`, `Web`, `Profile`, `Media`, `Streaming`, `Extensions`
  - `data.windows.<name>`: `file`, `transparent`, `clickthrough`, `in_game_only`, `desktop_only`, `native_window`, `topmost`, `size`/`min_size`/`max_size`, `start_position`, `resizable`, `keep_window_location`, `restrict_to_game_bounds`, `grab_keyboard_focus`, `focus_game_takeover`, `is_alt_f4_blocked`, `show_in_taskbar`, `mute`, `block_top_window_navigation` …
  - `data.game_targeting` (`all`/`dedicated`/`none` und `game_ids`), `data.game_events` (Game-IDs für GEP)
  - `data.launch_events`: App automatisch starten, wenn ein Spiel startet (`GameLaunch`, `AllGamesLaunch`, `LaunchWithOverwolf`)
  - `data.hotkeys`: benannte Tastenkürzel mit `default`, `action-type` (`toggle`/`custom`), `passthrough`, `hold`, `game_ids`
  - `data.extra-objects`: **native DLL-Plugins** (C#/.NET), erreichbar über `overwolf.extensions.current.getExtraObject()`. Firestone nutzt das intensiv (siehe 3.3).
  - `data.externally_connectable`, `content_scripts`, `url_protocol`, `process_name`, `auto_relaunch_on_crash`
- **Game Events Provider (GEP):** zwei Datentypen
  - *Game Events*: einzelne Momente (Kill, Tod …) über `overwolf.games.events.onNewEvents`
  - *Game Info*: dauerhafter Zustand über `onInfoUpdates2` und `getInfo()`
  - Apps melden benötigte Features mit `setRequiredFeatures()` an. Für Ausfälle gibt es „Event Status Endpoints“.
  - Spielestudios können eine **Events-SDK-DLL** einbinden und Daten einseitig an Overwolf senden.
- **Overlay:** Overwolf injiziert sich in die Render-Pipeline unterstützter Spiele (DirectX-Hook). Ablauf in ow-electron: `overlay.registerGames({gameIds})`, dann Event `game-launched` mit `event.inject()` oder `event.dismiss()`, danach `game-exit`. Ladefenster wie Shader-Kompilierung oder Anti-Cheat werden inzwischen ausgelassen.
- **Werkzeuge:** Package Manager im Developer-Modus („Load unpacked“), Events Recorder/Player (ERP), Game Events Simulator.
- **Verteilung:** OPK-Paket (Zip mit Manifest), Store-Review durch Overwolf, Monetarisierung über **Werbe-SDK** und Abos (Umsatzbeteiligung).
- **Linux/macOS:** Laut ow-electron-FAQ laufen dort aktuell **nur Werbedienste**. Overlay, Game Events und Recording sind „in Entwicklung“. Für Linux/Proton gibt es also derzeit keine Lösung von Overwolf, genau diese Lücke soll overLay füllen.

### 1.3 Oberfläche (zum Nachempfinden, ohne Assets zu kopieren)

- Dunkles UI mit schmaler **vertikaler App-Leiste (Dock)** links und App-Icons
- Bereiche: **Bibliothek** (installierte Apps), **App-Store** (Kacheln, Banner, Kategorien, Detailseite mit Screenshots), **Einstellungen** (Navigation links: Allgemein, Overlay & Hotkeys, Apps, Benachrichtigungen, Über)
- Tray-Icon mit Kontextmenü
- Im Spiel: App-Fenster als Overlay, Ein- und Ausblenden per Hotkey, Toast-Benachrichtigungen am Bildschirmrand
- Eigene Fensterrahmen (frameless) mit eigener Titelleiste

> Hinweis zum Markenrecht: Layout und Bedienkonzept dürfen ähnlich sein. Logo, Wolf-Motiv, Namen, Icons und Farbschema-Identität (Trade Dress) von Overwolf werden **nicht** übernommen.

---

## 2. CurseForge

### 2.1 Was ist offen?

- CurseForge-App und Backend sind **Closed Source**. Die App lief früher als Overwolf-App und gibt es heute auch eigenständig.
- Offen ist nur die **CurseForge Core API** (`https://api.curseforge.com`), dokumentiert auf docs.curseforge.com.

### 2.2 Core API (Konzepte, die für overLay nützlich sind)

- **Authentifizierung:** Header `x-api-key`. Einen Key bekommt man über die CurseForge-for-Studios-Konsole, Zugriff gibt es nur auf Antrag. Gilt auch für öffentliche Mods.
- **Endpunkte:** Games (`/v1/games…`), Categories, Mods (`/v1/mods/search`, `/v1/mods/{id}`, Batch `POST /v1/mods`, featured, description), Files (`/v1/mods/{id}/files…`, Changelog, Download-URL), **Fingerprints** (exakter und unscharfer Abgleich), Minecraft-spezifische Endpunkte.
- **Paginierung:** höchstens 50 pro Seite, höchstens 10.000 Ergebnisse insgesamt.
- **Mod-Modell:** `id`, `gameId`, `slug`, `classId`, Kategorien, Autoren, Links (`websiteUrl`, `sourceUrl`, `issuesUrl`), `downloadCount`, `mainFileId`, `latestFiles`, **`allowModDistribution`** (Autoren können Drittanbieter-Downloads verbieten, dann ist `downloadUrl` = `null`).
- **File-Modell:** `releaseType` (1 = Release, 2 = Beta, 3 = Alpha), `fileStatus`, `hashes[]` (`algo` 1 = SHA-1, 2 = MD5), `gameVersions[]`, **`dependencies[]`** mit `relationType` (1 eingebettete Bibliothek, 2 optional, 3 erforderlich, 4 Tool, 5 inkompatibel, 6 Include), `downloadUrl`, `fileFingerprint` (MurmurHash2 mit Seed 1, Whitespace-Bytes 9/10/13/32 vorher entfernt, so ist es aus Community-Tools bekannt), `modules[]`.
- **Schwäche:** Abhängigkeiten verweisen nur auf eine Mod-ID, nicht auf eine Dateiversion. Deshalb lassen sie sich nicht eindeutig auflösen.

### 2.3 Was overLay davon übernimmt

overLay **nutzt CurseForge nicht** (proprietär, Key-Pflicht, keine Pflicht zum offenen Quellcode). Übernommen werden die **Datenmodell-Ideen**:

- Release-Kanäle (stable/beta/alpha)
- Abhängigkeiten mit Typ, aber **mit Versionsbereichen** (semver) statt nur IDs
- Mehrere Hashes pro Datei, overLay nimmt aber **SHA-256 und eine Signatur** statt SHA-1/MD5
- Spielversionen bzw. Kompatibilität pro Datei
- Fingerprint-Abgleich, um manuell installierte Mods zu erkennen (optional, später)

---

## 3. Firestone (Hearthstone-Companion)

### 3.1 Lizenz: kritisch

- Repo `Zero-to-Heroes/firestone` (TypeScript, rund 170 Sterne, aktiv gepflegt): **Es gibt keine Lizenzdatei.** Rechtlich heißt das „alle Rechte vorbehalten“.
- `tos.md` verbietet ausdrücklich das Kopieren, Anpassen und Erstellen abgeleiteter Werke.
- Der Battlegrounds-Simulator `@firestone-hs/simulate-bgs-battle` ist ausdrücklich **„All rights reserved“**.

**Folgerung:** Firestone wird für overLay **clean-room** nachgebaut. Funktionen und Aussehen dienen als Vorbild, aber es wird **kein Code, kein Asset und kein Name** übernommen. Wer am overLay-Hearthstone-Mod mitarbeitet, darf Firestone-Code nicht abschreiben. Erlaubt sind die Feature-Liste aus dem Wiki und Screenshots als Referenz.

Ausnahmen mit freier Lizenz, die nutzbar sind:
- `@firestone-hs/reference-data`: **MIT**
- `@firestone-hs/hs-replay-xml-parser`: **MIT**
- `@firestone-hs/deckstrings`: **ISC** (Fork von HearthSim)

### 3.2 Aufbau des Codes (Stand 10/2026)

- **Nx-Monorepo, Angular + NgRx/RxJS**, i18n mit `ngx-translate`, Node 18
- `apps/`
  - `legacy`: die Overwolf-Native-App (CEF)
  - `electron-app` und `electron-frontend`: **laufende Migration auf ow-electron** (Overlay-Service mit `IOverwolfOverlayApi`, SQLite über `better-sqlite3`, `electron-updater`, Prozesserkennung per `tasklist` alle 2 s)
  - `web`, `mobile` (Capacitor), `twitch-extension`, `coliseum` (Replay-Viewer)
- `libs/`: `game-state`, `memory`, `battlegrounds`, `arena`, `constructed`, `collection`, `achievements`, `mercenaries`, `replay`, `stats`, `profile`, `settings`, `mainwindow`, `twitch`, `discord`, `communities`, `lottery`, `mods`, `electron`, `electron-edge-libs`, `shared`
- **Datenquellen des Spielzustands:**
  1. **Log-Parsing**: `Power.log` (und weitere Logs) werden über eine C#-Bibliothek (`HearthstoneReplays.dll`) in Echtzeit in Spielereignisse und XML-Replays umgewandelt
  2. **Speicherlesen („MindVision“)**: `OverwolfUnitySpy.dll` und `UnitySpy.HearthstoneLib.dll` lesen per UnitySpy den Mono-Heap von Hearthstone (Sammlung, Deck, Arena, BG-Spielerinfos, Quests, Achievements, Belohnungspfad, Account/Region …)
  3. In Electron werden diese .NET-DLLs über **edge-js** angebunden (`electron-edge-libs/*-edge.js`)
- **Weitere native Plugins:** `OwUtils.dll`, `DiscordRPCPlugin.dll`, `OwBgReconnector.dll` (BG-Reconnect-Trick), `HpCipher.dll`
- **Backend:** eigene AWS-Lambdas und Datenbanken, viele `@firestone-hs/*`-API-Pakete (Stats, Meta, Mulligan, Arena-Picks, BG-Global-Stats, Leaderboards …), statische Daten auf `static.zerotoheroes.com`
- **Telemetrie/Werbung:** `plausible-tracker` als Analytics, Overwolf-Werbung, Premium-Abo. Genau das soll es in overLay nicht geben.
- **Mods-Feature:** Firestone installiert optional **BepInEx 5** in Hearthstone und verteilt „trusted mods“ über eine `mods-config.json`. Das bedeutet Code-Injektion in den Spielclient (siehe Risiken).

### 3.3 Funktionsumfang (Vorbild für den Hearthstone-Mod)

Gekürzt nach dem Wiki „Firestone features“ und „Premium features“:

**Deck-Tracker / Overlay (Ladder, Arena, Brawl)**
- Eigenes Deck, Hand, Board, Friedhof in einklappbaren Zonen; dynamische Karteneffekte (Tess, Fizzle …)
- Gegner-Tracker: gespielte Karten, Decklisten-Import, Hand mit Ziehzeitpunkt, Buffs, Forge, mögliche Karten
- Secrets-Helfer (mögliche Geheimnisse, gefiltert nach Modus und Effekten)
- Zähler: Gesamtangriff, Erschöpfungsschaden, Leichen, Zauberschulen, Ausgraben …; Kompakt-Widget
- Max-Ressourcen-Widget, Ausspielreihenfolge der Diener, Synergie-Hervorhebung, verwandte Karten beim Hover
- Rundentimer, Recap spielentscheidender Karten, Statistik beim Entdecken (Discover), Mulligan-Guide

**Constructed (Hauptfenster)**
- Deckübersicht mit Versionen, Deck-Stats (Mulligan, Karten, Matchups), Meta-Decks, Rang-Verlauf, Deckbuilder mit Deckcode-Export, Lobby-Overlay mit Deckvorschau

**Battlegrounds**
- Meta-Übersicht (Helden, Stämme, Quests, Belohnungen), persönliche Hero-Stats, MMR-Verlauf
- **Kampfsimulator** (Gewinnchancen, Positionierungs-Optimierer, Share-Codes)
- Overlay: Kampfchancen, Kartenliste nach Stufe/Mechanik/Stamm, Infos zum letzten Gegner, Session-Recap, Zähler, Shop-Hervorhebung, Stämme-Widget, Gegner-MMR
- BG-Fenster: Gegner-Boards, HP-Graph, Kampfhistorie mit Re-Simulation

**Arena**
- Run-Historie, Klassen-Tierliste, Kartenstatistiken, 10+-Siege-Decks, Pick-Overlay mit Stats und Synergien

**Weitere:** Tavern Brawl (Klassen-Winrate), Mercenaries (Fortschritt, Teams, Ability-Overlay), Duels (inzwischen aus dem Spiel entfernt), **Replays** (Liste und Play-by-Play-Viewer), **Achievements** (native und eigene, anpinnbar), **Sammlung** (Packs, Pity-Timer, Set-Vervollständigung, Kartenrücken, Skins), Profil, Discord Rich Presence, Taskleisten-Hinweis „Du bist dran“, Twitch-Extension

**Optik:** dunkles, lila-blaues Hauptfenster mit Icon-Navigation links (Decktracker, Battlegrounds, Arena, Sammlung, Achievements, Replays …). Im Spiel eine Deckliste rechts aus Kartenkacheln (Kartenbild-Ausschnitt, Manakristall, Anzahl) und der Gegner-Tracker links. Größe, Deckkraft und Farben lassen sich einstellen.

---

## 4. Offene Bausteine für Hearthstone (nutzbar)

| Projekt | Lizenz | Nutzen |
|---|---|---|
| `HearthSim/python-hslog` | MIT | Referenz für den Power.log-Parser |
| `HearthSim/HSTracker` (Swift, macOS) | MIT | vollständiger offener Deck-Tracker als Referenz |
| `HearthSim/HearthDb` (C#) | MIT | Karten-DB, Enums (GameTag, Zone …) |
| `HearthSim/python-hearthstone` | MIT | Enums, Kartendaten-Tools |
| `Hackf5/unityspy` (C#) | MIT | Unity-Mono-Speicherleser **mit Linux-Unterstützung** (`ProcessFacadeLinux*`), Nachfolger des geschlossenen HearthMirror |
| `HearthSim/npm-deckstrings` / `deckstrings` | ISC | Deckcodes kodieren und dekodieren |
| `hearthstonejson-client` / HearthstoneJSON | ISC (Client) | Kartendaten (`api.hearthstonejson.com`), Kartenbilder (`art.hearthstonejson.com`). Die **Bilder gehören Blizzard**: zur Laufzeit laden, nicht mitliefern |
| `@firestone-hs/reference-data` | MIT | Referenzdaten und Enums |
| `@firestone-hs/hs-replay-xml-parser` | MIT | Replay-XML |
| `HearthSim/Hearthstone-Deck-Tracker` | Quellen offen, **GitHub erkennt keine Lizenz** | nur als Verhaltensreferenz, kein Code |
| Fireplace / RosettaStone | AGPL-3.0 | Constructed-Simulatoren, für Battlegrounds nicht direkt geeignet |

**Battlegrounds-Simulator:** Es gibt **keinen aktuellen, frei lizenzierten**. Ein eigener Simulator ist ein großes Teilprojekt, weil jede Karte implementiert werden muss.

---

## 5. Technische Fakten zu Hearthstone unter Linux/Proton

- Hearthstone ist ein **Unity-Mono**-Spiel ohne offiziellen Linux-Build. Es läuft entweder **nativ** über den Community-Launcher hearthstone-linux-gui (siehe 5.1) oder über Battle.net in Wine/Proton (Lutris, Steam-Proton als Nicht-Steam-Spiel, Bottles, Heroic, Faugus).
- **Logs:** `log.config` unter `<prefix>/drive_c/users/<user>/AppData/Local/Blizzard/Hearthstone/log.config`. Logs liegen unter `<Installationsordner>/Logs/Hearthstone_<Datum>/Power.log` usw. overLay muss **Wine-Präfixe automatisch finden** und die `log.config` anlegen bzw. ergänzen.
- **Speicher:** UnitySpy kann einen Wine-Prozess vom Linux-Host aus über `/proc/<pid>/mem` bzw. ptrace lesen. Nötig ist derselbe Benutzer, und je nach `ptrace_scope` braucht es `CAP_SYS_PTRACE`.
- **Overlay ohne Injektion:** Der Deck-Tracker HDT läuft unter Windows als transparentes Topmost-Fenster über Hearthstone. Das geht, weil HS randlos im Vollbild läuft. Unter Linux funktioniert dasselbe Prinzip unter X11 und XWayland. Unter nativem Wayland braucht man `wlr-layer-shell` (KDE, wlroots, Hyprland; **nicht GNOME**). Unter **gamescope** (Steam Deck) braucht man das External-Overlay-Atom (`GAMESCOPE_EXTERNAL_OVERLAY`, wie bei MangoHud/mangoapp).
- **Blizzard-ToS:** Deck-Tracker, die nur Logs auswerten und Informationen zeigen, die der Spieler ohnehin selbst sehen könnte, werden seit Jahren geduldet. Speicherlesen ist eine Grauzone (HDT und Firestone tun es). **Code-Injektion in den Client** (BepInEx) ist riskanter und wird in overLay **nicht** standardmäßig angeboten.

### 5.1 Nativer Linux-Client (hearthstone-linux, hearthstone-linux-gui)

**Hauptziel von Flint.** Hearthstone ist ein Unity-Spiel, und Unitys Plattformdateien sind weitgehend generisch. Das Community-Projekt [`0xf4b1/hearthstone-linux`](https://github.com/0xf4b1/hearthstone-linux) (MIT) startet die **offiziellen macOS-Spieldateien mit Unitys Linux-Player** (Renderer OpenGLCore), ohne Wine und ohne Battle.net-App. Es verteilt selbst keine proprietären Dateien; die Spieldateien kommen vom CDN von Blizzard. Bekannte Grenzen: Der Shop im Spiel ist geschlossen, und die Autoren weisen darauf hin, dass diese inoffizielle Methode ein Bann-Risiko haben kann.

[`DawnMagnet/hearthstone-linux-gui`](https://github.com/DawnMagnet/hearthstone-linux-gui) (MIT, Rust, GTK4/libadwaita, aktiv gepflegt) macht daraus einen fertigen Launcher: nativer NGDP-Downloader mit Cache und Prüfung, Login, Update und Start aus einem Fenster, optional `gamemoderun` und MangoHud, Editor für Spieleinstellungen, Oberfläche auf Englisch, Deutsch und Chinesisch, Pakete als AppImage, DEB, RPM, Flatpak und Nix.

Fakten, die für Flint wichtig sind (aus dem Quellcode des Launchers, Stand 2026-10-14):

| Thema | Wert |
|---|---|
| Prozess | `<game_dir>/Bin/Hearthstone.x86_64`, ein nativer ELF-Prozess mit `game_dir` als Arbeitsverzeichnis |
| Standard-`game_dir` | `$XDG_DATA_HOME/hearthstone-linux-gui/game` (meist `~/.local/share/hearthstone-linux-gui/game`) |
| Launcher-Konfiguration | `$XDG_CONFIG_HOME/hearthstone-linux-gui/config.toml`, optionaler Schlüssel `game_dir`, außerdem `installed_version`, `locale`, `region`, `use_mangohud` … |
| Mono-Laufzeit | `Bin/Hearthstone_Data/MonoBleedingEdge/x86_64/libmonobdwgc-2.0.so`, ein Speicherleser arbeitet also mit einem normalen Linux-Prozess, ohne Wine-Adressumrechnung |
| Spieleinstellungen | Der Linux-Unity-Player nutzt den **macOS-Ort**: `~/Library/Preferences/Blizzard/Hearthstone/options.txt`. `log.config` wird daher im selben Ordner erwartet (**im Spike zu bestätigen**) |
| Spiel-Logs | `Logs/` unterhalb des Spielordners (das ursprüngliche Projekt verweist zur Fehlersuche auf `Bin/Logs/`; **zu bestätigen**). Der Launcher leitet Unitys stdout/stderr (`Player.log`) in seinen eigenen Log-Ordner um |
| Fenster | Unity-Linux-Player, normalerweise X11/XWayland, also dasselbe Overlay-Backend wie bei Proton |

**Weitere Linux-Tracker (als Referenz):**

- [`Ratpido-dev/cairn`](https://github.com/Ratpido-dev/cairn) (MIT, Python): nativer Linux-Tracker, der nur Logs liest, ohne Injektion, ohne Speicherlesen und ohne Telemetrie. Nützliche Erkenntnis daraus: Hearthstone begrenzt seine Logs auf etwa 10 MB und schneidet sie dann ab („Truncating log“); unter Wine kann das scheitern, und das Tracking wird blind. Cairn unterstützt den nativen Client nur über einen manuell gesetzten Log-Pfad und kann dessen `log.config` nicht schreiben. Flint sollte beides können.
- [`borisbabic/hearthstone_hdt_linux`](https://github.com/borisbabic/hearthstone_hdt_linux) (Apache-2.0): Anleitung für HDT unter Wine.
- [`yusufipk/deste-hearthstone-linux-tracker`](https://github.com/yusufipk/deste-hearthstone-linux-tracker) (keine Lizenz): nur als Verhaltensreferenz.

---

## Quellen

- [Overwolf Framework Overview](https://dev.overwolf.com/ow-native/getting-started/onboarding-resources/framework-overview)
- [Overwolf manifest.json](https://dev.overwolf.com/ow-native/reference/manifest/manifest-json)
- [Overwolf GEP Intro](https://dev.overwolf.com/ow-native/reference/live-game-data-gep/live-game-data-gep-intro)
- [Overwolf Events SDK für Spielentwickler](https://dev.overwolf.com/ow-native/live-game-data-gep/events-sdk-for-game-developers)
- [ow-electron Overlay API](https://dev.overwolf.com/ow-electron/reference/Overwolf-electron-APIs/overlay/interfaces/IOverwolfOverlayApi/)
- [ow-electron FAQ (Linux/macOS-Status)](https://dev.overwolf.com/ow-electron/getting-started/onboarding-resources/ow-electron-faq)
- [ow-electron Sample (Community)](https://github.com/peka2/ow-electron-packages-sample)
- [CurseForge Core API](https://docs.curseforge.com/rest-api/)
- [Firestone Repo](https://github.com/Zero-to-Heroes/firestone), [Features](https://github.com/Zero-to-Heroes/firestone/wiki/Firestone-features), [Premium](https://github.com/Zero-to-Heroes/firestone/wiki/Premium-features)
- [UnitySpy](https://github.com/Hackf5/unityspy), [python-hslog](https://github.com/HearthSim/python-hslog), [HSTracker](https://github.com/HearthSim/HSTracker), [HearthDb](https://github.com/HearthSim/HearthDb)
- BG-Simulatoren: [twanvl](https://github.com/twanvl/hearthstone-battlegrounds-simulator), [BGSimulator](https://github.com/yossielimelech/BGSimulator), [Ghastcoiler](https://github.com/pruderior/Ghastcoiler), [Fireplace](https://github.com/synap5e/fireplace)
