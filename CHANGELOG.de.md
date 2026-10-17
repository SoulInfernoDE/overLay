# Änderungsprotokoll

> Deutsche Fassung. Englisches Original: [CHANGELOG.md](CHANGELOG.md)

Alle nennenswerten Änderungen werden hier festgehalten. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), die Versionen folgen [Semantic Versioning](https://semver.org/lang/de/).

## [Unveröffentlicht]

### Geändert

- Der Hearthstone-Provider erkennt den Nutzer an den Startkarten, die das Spiel beim Ziehen aufdeckt, und meldet keinen Gegner, solange der Nutzer unbekannt ist; vorher konnte während der Starthand-Auswahl der eigene Spieler als Gegner gemeldet werden
- Der Launcher kommt nach vorn, wenn er per Tastenkürzel, Infobereich oder zweitem Programmstart geöffnet wird, während ein Vollbild-Spiel aktiv ist (X11); der Fenstermanager hielt ihn vorher hinter dem Spiel
- Das Mod-SDK schickt eine einfache Kopie, wenn sich die Daten eines Aufrufs nicht für `postMessage` klonen lassen; so kann reaktiver Zustand von UI-Frameworks (Svelte, Vue) an `overlay.storage.set` und andere Aufrufe gehen, solche Aufrufe schlugen vorher still fehl
- Der Hearthstone-Provider verfolgt die Partie so, wie der Client sie abspielt (der `PowerTaskList`-Strom von `Power.log`), statt so, wie sie vom Server kommt: Schlachtfeld-Kämpfe kamen vollständig an, bevor ihre Animation begann, sodass Mods Ergebnisse zu früh zeigten und den Kampf selbst nie sahen. Änderungen an Spielern, die genannt werden, bevor ihr Name zugeordnet werden kann, werden nachgereicht, sobald das geht
- Overlay-Fenster von Mods bleiben innerhalb des Overlays: Eine Position aus dem Manifest oder dem Speicher außerhalb des Spielfensters (z. B. nach dem Wechsel in den Fenstermodus) versteckt das Fenster nicht mehr, und Verschieben beginnt dort, wo es angezeigt wird
- Der Hearthstone-Provider hebt die Log-Größenbegrenzung des Spiels in der `client.config` auf (und in den Overrides von hearthstone-linux-gui): Hearthstone hörte auf, Logs zu schreiben, sobald einer 10 MB erreichte, sodass das Ende längerer Partien nie ankam
- Das Overlay bleibt sichtbar, solange das Spiel angezeigt wird und kein anderes aktives Fenster es überdeckt; Arbeiten auf einem anderen Monitor lässt es also stehen (vorher verschwand es, sobald das Spiel den Fokus verlor)
- Fahrplan nach dem Overlay-Spike: eine gemeinsame Overlay-WebView für alle Mods, das Overlay nimmt nie den Fokus, klickbare Bereiche werden mit dem Device-Pixel-Ratio umgerechnet; Orte von `log.config` und Logs beim nativen Client bestätigt
- Fahrplan und Recherche: Der native Linux-Client von Hearthstone (hearthstone-linux-gui, macOS-Spieldateien auf Unitys Linux-Player) ist jetzt Hauptziel von Flint, mit Details zu Prozess, Pfaden und Logs

### Hinzugefügt

- `REPLAY_ALL_EVENTS=1` lässt das Replay-Beispiel alle Ereignisse samt Entitäts-Updates schreiben, damit Mods eine aufgezeichnete Partie Schritt für Schritt nachbauen können
- Spielereignis `attack` (Angreifer, Verteidiger und Seite des Angreifers), z. B. für Geheimnis-Helfer
- `overlay.events.history()`: die bisherigen Ereignisse der laufenden Partie, für Mods, die mitten in einer Partie starten; der Kern merkt sie sich pro Partie
- Das Replay-Beispiel kann die Spielereignisse als JSON schreiben, um Mods an aufgezeichneten Partien zu testen
- Der Hearthstone-Provider liest `Decks.log` und teilt Mods mit, welches seiner eigenen Decks der Nutzer spielt (`GameInfo.deck` mit Name und Deck-Code)
- Hearthstone-Provider (Phase 5, `overlay-hearthstone`): findet die Logs des nativen Clients, von Wine/Proton und Windows, ergänzt `log.config`, folgt `Power.log` auch über Abschneiden und neue Log-Ordner hinweg und macht daraus Spielereignisse und eine Spielübersicht für Mods mit `game.events` (`overlay.events.on`, `getInfo`, `onInfo` mit TypeScript-Typen); Beispiele `replay` und `locate`, um echte Logs lokal zu prüfen
- Hinweise warten im Kern, bis die Overlay-Seite sie abholt, und werden zurückgehalten, solange das Overlay ausgeblendet ist; so geht keiner verloren
- Mod-SDK und Laufzeit (Phase 3): `overlay.json`-Manifeste mit strenger Prüfung (`overlay-mods`); Mods laufen als Sandbox-Frames, ausgeliefert über `mod://` mit einer CSP pro Mod, sodass die WebView nicht deklarierte Netzwerk-Hosts blockiert und dem Kern meldet; SDK `@overlay/sdk` mit Übersetzungen, Speicher, Hinweisen, Nachrichtenbus, Tastenkürzeln, Einstellungen, Spielzustand und verschiebbaren, teilweise durchklickbaren Overlay-Fenstern; Berechtigungsprüfung im Kern; Entwicklermodus für lokale Mod-Ordner; Mods in der Bibliothek des Launchers mit Mod-Seite (Berechtigungen, Netzwerk-Hosts, erzeugte Einstellungen, Desktop-Fenster); Beispiel-Mod `hello-overlay`; Anleitung für Mod-Autoren (`docs/mod-development.de.md`)
- Overlay im Spiel (Phase 2): ein transparentes, durchklickbares Fenster pro laufendem Spiel, das dem Spielfenster über X11-Ereignisse folgt, nur bei aktivem Spiel sichtbar ist, nie den Fokus nimmt und sich mit `Strg+Umschalt+L` ein- und ausblenden lässt; Hinweis-Einblendungen; ein Demo-Widget, dessen Position pro Spiel und Auflösung gespeichert wird
- Prozessmodell: Der Kern läuft ohne WebView, Launcher und Overlay sind Kindprozesse, die über stdin/stdout mit ihm sprechen (`overlay-ipc`); nach dem Schließen des Launchers braucht der Kern wieder 27 MB PSS
- `OVERLAY_DEBUG=1` protokolliert jede Änderung des Spielfensters und die klickbaren Bereiche des Overlays
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
