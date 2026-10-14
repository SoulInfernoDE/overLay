# Ergebnisse des Spikes: Overlay-Fenster

> Deutsche Fassung. Englisches Original: [RESULTS.md](RESULTS.md)

Testsystem: Linux Mint 22.3 (Cinnamon, X11, Fenstermanager mit Compositing), AMD Phoenix1 (integrierte Grafik), 3840×2160 bei 115 dpi, Tauri 2.12.3, WebKitGTK 2.52.6. Alle Speicherwerte sind PSS, summiert über den Spike und seine WebKit-Hilfsprozesse (`measure.sh`), jeweils nach 8–9 s Anlaufzeit und 10 s Messung im Leerlauf.

## Demo-Modus (ohne Spiel)

| Lauf | RAM (PSS) | CPU im Leerlauf |
|---|---|---|
| A: Fenster 3840×2160 | 182 MB | 0,2 % |
| B: Fenster 1920×1080 | 170 MB | 0,2 % |
| C: Fenster 300×900 (Widget-Größe) | 167–170 MB | 0,1 % |
| D: 3840×2160, `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 184 MB | 0,2 % |
| E: 300×900, `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 140 MB | 0,0–0,2 % |
| 3 WebView-Fenster | 273 MB (3 Webprozesse) | 0,1 % |
| 5 WebView-Fenster | 371 MB (5 Webprozesse) | 0,2 % |

Transparenz und Darstellung funktionieren im Standard-Stapelmodus (`override`), und mit abgeschaltetem Compositing sieht die Darstellung identisch aus. Die Input-Shape wird ohne Fehler gesetzt; ob Klicks wirklich durchgehen, wird von Hand mit dem Spiel geprüft (siehe unten).

## Erkenntnisse

1. **Jedes WebView-Fenster startet einen eigenen WebKit-Webprozess (je etwa 50 MB).** Ein Fenster pro Widget, wie bei Overwolf, ist mit WebKitGTK zu teuer. → overLay nutzt **eine Overlay-WebView**, die das Spiel abdeckt und alle Widgets aller Mods im Spiel enthält (jeder Mod in einem Sandbox-Frame mit eigenem Ursprung und eigener CSP). Außerhalb der Widgets gehen Klicks über die Input-Shape ans Spiel. Das Hauptfenster auf dem Desktop ist eine zweite WebView, die nur existiert, solange es geöffnet ist.
2. **Die Fenstergröße spielt kaum eine Rolle** (etwa 12 MB zwischen 4K und Widget-Größe). Ein Overlay-Fenster in voller Spielgröße ist also in Ordnung.
3. **Der größte Teil des Speichers ist Grundlast von WebKit** (Webprozess etwa 100 MB, UI-Prozess etwa 65 MB). Abgeschaltetes beschleunigtes Compositing spart bei einem kleinen Fenster etwa 27 MB; beim Fenster in voller Größe muss das genauer untersucht werden (Lauf D zeigt keine Ersparnis).
4. **Die CPU-Last im Leerlauf ist vernachlässigbar** (≤ 0,2 % eines Kerns, inklusive Abfrage alle 250 ms).
5. **Das Budget von 150 MB** aus dem Fahrplan ist nur mit abgeschaltetem Compositing erreichbar. Realistisches Ziel für den Host mit einem Overlay: 140–170 MB, und nichts, solange kein Spiel läuft (Tray-Modus ohne WebView).

## Noch offen (braucht das Spiel)

- [ ] Overlay über dem nativen Client (hearthstone-linux-gui): folgt dem Fenster, im Fenster- und Vollbildmodus, in beiden Stapel-Modi
- [ ] Klicks gehen außerhalb des Widgets ans Spiel, auf dem Widget nicht
- [ ] Overlay verschwindet, solange das Spiel minimiert ist
- [ ] RAM/CPU bei laufendem Spiel
- [ ] Nativer Client liest `~/Library/Preferences/Blizzard/Hearthstone/log.config` und schreibt `Power.log`
- [ ] Dieselben Prüfungen unter Proton/Wine (Lutris-Installation)
- [ ] Windows
