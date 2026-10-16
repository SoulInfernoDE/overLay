# Branding

> English original. German version: [README.de.md](README.de.md)

| | overLay (host app) | Flint (Hearthstone mod) |
|---|---|---|
| Icon | <img src="overlay/icon.svg" width="96" alt="overLay icon"> | <img src="flint/icon.svg" width="96" alt="Flint icon"> |
| Logo for dark backgrounds | [`overlay/logo-dark.svg`](overlay/logo-dark.svg) | [`flint/logo-dark.svg`](flint/logo-dark.svg) |
| Logo for light backgrounds | [`overlay/logo-light.svg`](overlay/logo-light.svg) | [`flint/logo-light.svg`](flint/logo-light.svg) |
| PNG icons | 16, 32, 48, 64, 128, 256, 512 px | 16, 32, 48, 64, 128, 256, 512 px |

## Meaning

- **overLay:** two stacked panes, the game screen behind and the overlay in front. The front pane is see-through where it covers the back one, and the two short bars hint at the widgets an overlay shows.
- **Flint:** a knapped flint stone striking a spark. In the wordmark the spark replaces the dot of the "i".

## Colours

| Token | Value | Use |
|---|---|---|
| overLay teal | `#2ed3b7` (`#12a58d` on light backgrounds) | accent, "Lay" in the wordmark |
| Flint spark | `#ffb347` (`#e8701f` on light backgrounds) | spark |
| Text on dark | `#e8ebf0` | wordmarks |
| Text on light | `#15181e` | wordmarks |

## Rebuilding

The files are generated; change `tools/branding/build_logos.py` instead of editing the SVGs by hand. The wordmarks use [Inter](https://github.com/rsms/inter) (SIL Open Font License 1.1), converted to outlines, so the font is not needed to display them. PNGs are rendered with Inkscape.

```bash
python3 tools/branding/build_logos.py --font-dir <folder with Inter-Bold.ttf and Inter-ExtraBold.ttf> --out assets/branding
```

## License and use

The logos are licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). You may use them to refer to overLay and Flint, for example in articles, videos or package listings. Do not use them in a way that suggests your project is overLay or Flint, or is endorsed by them. The [Flint repository](https://github.com/SoulInfernoDE/flint) carries a copy of Flint's logos; they are generated here.
