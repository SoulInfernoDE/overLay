# Spike results: overlay window

> English original. German version: [RESULTS.de.md](RESULTS.de.md)

Test system: Linux Mint 22.3 (Cinnamon, X11, compositing window manager), AMD Phoenix1 iGPU, four 1920×1080 monitors in a 2×2 grid (one X screen of 3840×2160) at 115 dpi, Tauri 2.12.3, WebKitGTK 2.52.6. All memory values are PSS summed over the spike and its WebKit helper processes (`measure.sh`), each after 8–9 s of warm-up and a 10 s idle sample.

## Demo mode (no game)

| Run | RAM (PSS) | CPU idle |
|---|---|---|
| A: window 3840×2160 (all four monitors) | 182 MB | 0.2 % |
| B: window 1920×1080 | 170 MB | 0.2 % |
| C: window 300×900 (widget-sized) | 167–170 MB | 0.1 % |
| D: 3840×2160, `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 184 MB | 0.2 % |
| E: 300×900, `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 140 MB | 0.0–0.2 % |
| 3 webview windows | 273 MB (3 web processes) | 0.1 % |
| 5 webview windows | 371 MB (5 web processes) | 0.2 % |

Transparency and rendering work in the default stacking mode (`override`), and the rendering with compositing mode off looks identical. The input shape is applied without errors; whether clicks really pass through is checked by hand with the game (see below).

## Findings

1. **Every webview window starts its own WebKit web process (about 50 MB each).** One window per widget, as Overwolf does, is too expensive with WebKitGTK. → overLay uses **one overlay webview** that covers the game and holds all in-game widgets of all mods (each mod in a sandboxed frame with its own origin and CSP), and clicks pass through outside the widgets via the input shape. The desktop main window is a second webview that only exists while it is open.
2. **Window size hardly matters** (about 12 MB between all four monitors and widget size). A full-size overlay window over the game is fine.
3. **Most of the memory is WebKit's fixed cost** (web process about 100 MB, UI process about 65 MB). Turning off accelerated compositing saves about 27 MB for a small window; it needs a closer look for the full-size window (run D shows no saving).
4. **Idle CPU is negligible** (≤ 0.2 % of one core, polling every 250 ms included).
5. **The 150 MB budget** from the roadmap is only reachable with compositing off. Realistic target for the host with one overlay: 140–170 MB, plus nothing when no game runs (tray-only mode without webview).

## With the game: native client (hearthstone-linux-gui)

Hearthstone 37.0 on Unity 6000.3, **fullscreen** on the primary monitor (1920×1080, top left of the 2×2 grid), stacking mode `override`, compositing mode off.

| Check | Result |
|---|---|
| Overlay appears when the game starts and follows its window | ✓ |
| Hides while the game is minimised, returns at the same place | ✓ |
| Clicks outside the widget reach the game (end-turn button, hover and click) | ✓ after fix 1 |
| Widget can be collapsed and dragged, the game does not get these clicks | ✓ |
| Fullscreen mode | ✓ in both stacking modes (`override`, `above`); Cinnamon keeps an `above` window over the active fullscreen game |
| Game moved to another monitor (windowed, then fullscreen on it) | ✓ overlay follows |
| Game restarted while the spike runs | ✓ new process found again |
| Stutter | none noticed |
| RAM / CPU with the game running | 146 MB PSS / 0.5 % of one core |
| `~/Library/Preferences/Blizzard/Hearthstone/log.config` is read, `Power.log` is written | ✓ (`<game_dir>/Logs/Hearthstone_<timestamp>/`) |

**Bugs found and fixed:**

1. **Clickable area offset.** The page reported widget rectangles in CSS pixels, but the GTK input shape expects GDK units. At 115 dpi WebKit scales CSS pixels by 1.2, so the clickable area sat about 20 % further up and to the left than the visible widget, right over the end-turn button: the game cursor turned into the system cursor and the button got neither hover nor clicks. Fix: the page sends `devicePixelRatio`, and the host converts with `devicePixelRatio / GDK scale`.
2. **Focus.** The overlay window now refuses keyboard focus (`accept_focus = false`, `focus_on_map = false`), so clicking a widget cannot take focus away from the game. This was a precaution; the reported problem was fully explained by bug 1.

**Note for phase 2:** the spike polls the window position every 250 ms, so the overlay trails slightly behind while the game window is being dragged. The real implementation should listen for X11 `ConfigureNotify` events instead.

## Still open

- [ ] Proton/Wine (Lutris installation)
- [ ] Native Wayland session (KDE, GNOME → XWayland), Sway
- [ ] Windows
