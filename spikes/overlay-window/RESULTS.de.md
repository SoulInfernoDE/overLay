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

## Mit dem Spiel: nativer Client (hearthstone-linux-gui)

Hearthstone 37.0 auf Unity 6000.3, im Fenster 1920×1080 oben links auf dem 4K-Bildschirm, Stapel-Modus `override`, Compositing aus.

| Prüfung | Ergebnis |
|---|---|
| Overlay erscheint beim Spielstart und folgt dem Spielfenster | ✓ |
| Verschwindet, solange das Spiel minimiert ist, und kommt an derselben Stelle wieder | ✓ |
| Klicks außerhalb des Widgets kommen im Spiel an („Fertig“-Knopf, Hover und Klick) | ✓ nach Korrektur 1 |
| Widget lässt sich einklappen und ziehen, das Spiel bekommt diese Klicks nicht | ✓ |
| Ruckler | keine bemerkt |
| RAM / CPU bei laufendem Spiel | 146 MB PSS / 0,5 % eines Kerns |
| `~/Library/Preferences/Blizzard/Hearthstone/log.config` wird gelesen, `Power.log` wird geschrieben | ✓ (`<game_dir>/Logs/Hearthstone_<Zeitstempel>/`) |

**Gefundene und behobene Fehler:**

1. **Klickbereich verschoben.** Die Seite meldete die Widget-Rechtecke in CSS-Pixeln, die GTK-Input-Shape erwartet aber GDK-Einheiten. Bei 115 dpi skaliert WebKit CSS-Pixel um den Faktor 1,2. Der klickbare Bereich lag deshalb etwa 20 % weiter links oben als das sichtbare Widget, genau über dem „Fertig“-Knopf: Der Spiel-Mauszeiger wurde zum System-Mauszeiger, und der Knopf bekam weder Hover noch Klicks. Korrektur: Die Seite sendet `devicePixelRatio`, und der Host rechnet mit `devicePixelRatio / GDK-Skalierung` um.
2. **Fokus.** Das Overlay-Fenster nimmt keinen Tastaturfokus mehr an (`accept_focus = false`, `focus_on_map = false`), damit ein Klick aufs Widget dem Spiel nicht den Fokus nehmen kann. Das war eine Vorsichtsmaßnahme; das gemeldete Problem erklärte sich vollständig durch Fehler 1.

## Noch offen

- [ ] Vollbildmodus des Spiels und Stapel-Modus `above`
- [ ] Proton/Wine (Lutris-Installation)
- [ ] Native Wayland-Sitzung (KDE, GNOME → XWayland), Sway
- [ ] Windows
