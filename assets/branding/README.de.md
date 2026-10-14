# Markenauftritt

> Deutsche Fassung. Englisches Original: [README.md](README.md)

| | overLay (Host-App) | Flint (Hearthstone-Mod) |
|---|---|---|
| Icon | <img src="overlay/icon.svg" width="96" alt="overLay-Icon"> | <img src="flint/icon.svg" width="96" alt="Flint-Icon"> |
| Logo für dunkle Hintergründe | [`overlay/logo-dark.svg`](overlay/logo-dark.svg) | [`flint/logo-dark.svg`](flint/logo-dark.svg) |
| Logo für helle Hintergründe | [`overlay/logo-light.svg`](overlay/logo-light.svg) | [`flint/logo-light.svg`](flint/logo-light.svg) |
| PNG-Icons | 16, 32, 48, 64, 128, 256, 512 px | 16, 32, 48, 64, 128, 256, 512 px |

## Bedeutung

- **overLay:** zwei gestapelte Ebenen, hinten der Spielbildschirm, vorne das Overlay. Wo die vordere Ebene die hintere überdeckt, scheint diese durch, und die zwei kurzen Balken deuten die Widgets an, die ein Overlay zeigt.
- **Flint:** ein behauener Feuerstein, der einen Funken schlägt. In der Wortmarke ersetzt der Funke den Punkt auf dem „i“.

## Farben

| Token | Wert | Verwendung |
|---|---|---|
| overLay-Türkis | `#2ed3b7` (`#12a58d` auf hellem Hintergrund) | Akzent, „Lay“ in der Wortmarke |
| Flint-Funke | `#ffb347` (`#e8701f` auf hellem Hintergrund) | Funke |
| Text auf dunkel | `#e8ebf0` | Wortmarken |
| Text auf hell | `#15181e` | Wortmarken |

## Neu erzeugen

Die Dateien werden generiert. Änderungen gehören in `tools/branding/build_logos.py`, nicht von Hand in die SVGs. Die Wortmarken nutzen [Inter](https://github.com/rsms/inter) (SIL Open Font License 1.1), in Pfade umgewandelt, sodass die Schrift zum Anzeigen nicht nötig ist. Die PNGs rendert Inkscape.

```bash
python3 tools/branding/build_logos.py --font-dir <Ordner mit Inter-Bold.ttf und Inter-ExtraBold.ttf> --out assets/branding
```

## Lizenz und Nutzung

Die Logos stehen unter [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/deed.de). Du darfst sie verwenden, um auf overLay und Flint zu verweisen, zum Beispiel in Artikeln, Videos oder Paketlisten. Verwende sie nicht so, dass der Eindruck entsteht, dein Projekt sei overLay oder Flint oder werde von ihnen unterstützt. Flints Markenauftritt zieht in das Flint-Repository um, sobald es existiert.
