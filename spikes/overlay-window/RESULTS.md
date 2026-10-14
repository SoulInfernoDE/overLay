# Spike results: overlay window

> English original. German version: [RESULTS.de.md](RESULTS.de.md)

Test system: Linux Mint 22.3 (Cinnamon, X11, compositing window manager), AMD Phoenix1 iGPU, 3840×2160 at 115 dpi, Tauri 2.12.3, WebKitGTK 2.52.6. All memory values are PSS summed over the spike and its WebKit helper processes (`measure.sh`), each after 8–9 s of warm-up and a 10 s idle sample.

## Demo mode (no game)

| Run | RAM (PSS) | CPU idle |
|---|---|---|
| A: window 3840×2160 | 182 MB | 0.2 % |
| B: window 1920×1080 | 170 MB | 0.2 % |
| C: window 300×900 (widget-sized) | 167–170 MB | 0.1 % |
| D: 3840×2160, `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 184 MB | 0.2 % |
| E: 300×900, `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 140 MB | 0.0–0.2 % |
| 3 webview windows | 273 MB (3 web processes) | 0.1 % |
| 5 webview windows | 371 MB (5 web processes) | 0.2 % |

Transparency and rendering work in the default stacking mode (`override`), and the rendering with compositing mode off looks identical. The input shape is applied without errors; whether clicks really pass through is checked by hand with the game (see below).

## Findings

1. **Every webview window starts its own WebKit web process (about 50 MB each).** One window per widget, as Overwolf does, is too expensive with WebKitGTK. → overLay uses **one overlay webview** that covers the game and holds all in-game widgets of all mods (each mod in a sandboxed frame with its own origin and CSP), and clicks pass through outside the widgets via the input shape. The desktop main window is a second webview that only exists while it is open.
2. **Window size hardly matters** (about 12 MB between 4K and widget size). A full-size overlay window over the game is fine.
3. **Most of the memory is WebKit's fixed cost** (web process about 100 MB, UI process about 65 MB). Turning off accelerated compositing saves about 27 MB for a small window; it needs a closer look for the full-size window (run D shows no saving).
4. **Idle CPU is negligible** (≤ 0.2 % of one core, polling every 250 ms included).
5. **The 150 MB budget** from the roadmap is only reachable with compositing off. Realistic target for the host with one overlay: 140–170 MB, plus nothing when no game runs (tray-only mode without webview).

## Still open (needs the game)

- [ ] Overlay over the native client (hearthstone-linux-gui): follows the window, windowed and fullscreen, both stacking modes
- [ ] Click-through to the game outside the widget, no clicks through the widget
- [ ] Overlay hides while the game is minimised
- [ ] RAM/CPU while the game runs
- [ ] Native client reads `~/Library/Preferences/Blizzard/Hearthstone/log.config` and writes `Power.log`
- [ ] Same checks under Proton/Wine (Lutris installation)
- [ ] Windows
