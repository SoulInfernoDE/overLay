# Spike: Overlay-Fenster

> Deutsche Fassung. Englisches Original: [README.md](README.md)

Ein Wegwerf-Messprojekt für die riskanteste Annahme des Fahrplans: Ein Tauri-2-Fenster kann **transparent, im Vordergrund und durchklickbar** über Hearthstone liegen (nativer Linux-Client und Proton) und dabei im Ressourcenbudget bleiben. Es gehört weder zum Haupt-Workspace noch zur CI.

## Was es tut

- findet `Hearthstone.x86_64` (nativer Client) oder `Hearthstone.exe` (Wine/Proton) über `/proc` und dann das zugehörige X11-Fenster über `_NET_WM_PID`
- verschiebt das Overlay alle 250 ms auf den Fensterinhalt des Spiels, passt die Größe an und blendet es aus, solange das Spiel minimiert ist
- zeigt ein Deck-Tracker-ähnliches Widget mit Beispieldaten; nur die Widgets nehmen Mausklicks an (GTK-Input-Shape), alles andere geht ans Spiel
- zwei Stapel-Modi: `override` (Standard, am Fenstermanager vorbei) und `above` (bittet den Fenstermanager, das Fenster oben zu halten)

## Voraussetzungen (Debian/Ubuntu/Mint)

```bash
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev
```

## Starten

```bash
cargo run --release
```

Ohne laufendes Spiel deckt `OVERLAY_SPIKE_DEMO=1` den ganzen Bildschirm ab. `OVERLAY_SPIKE_STACKING=above` wechselt den Stapel-Modus. Für Messungen legt `OVERLAY_SPIKE_DEMO_BOUNDS=x,y,breite,höhe` die Größe des Demo-Fensters fest, und `OVERLAY_SPIKE_EXTRA_WINDOWS=n` öffnet n weitere WebView-Fenster. Ergebnisse: [RESULTS.de.md](RESULTS.de.md).

## Messen

Während der Spike läuft:

```bash
./measure.sh 30
```

## Test-Checkliste

1. Overlay erscheint über dem Spiel und folgt ihm, wenn das Fenster verschoben oder in der Größe geändert wird
2. Transparente Bereiche zeigen das Spiel; Klicks dort kommen beim Spiel an
3. Das Widget lässt sich ziehen und einklappen; Klicks darauf kommen nicht beim Spiel an
4. Funktioniert im Fenster- und im Vollbildmodus, in beiden Stapel-Modi
5. Overlay verschwindet, solange das Spiel minimiert ist, und kommt danach wieder
6. RAM (PSS gesamt) und CPU im Leerlauf aus `measure.sh`
7. Nativer Client: Liest Hearthstone `~/Library/Preferences/Blizzard/Hearthstone/log.config` und schreibt dann `Power.log`?
