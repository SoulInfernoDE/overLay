# Spike: overlay window

> English original. German version: [README.de.md](README.de.md)

A throwaway measurement project for the riskiest assumption of the roadmap: a Tauri 2 window can sit **transparent, on top and click-through** over Hearthstone (native Linux client and Proton), and stay within the resource budget. It is not part of the main workspace or CI.

## What it does

- finds `Hearthstone.x86_64` (native client) or `Hearthstone.exe` (Wine/Proton) through `/proc`, then its X11 window through `_NET_WM_PID`
- moves and resizes the overlay to the game's client area every 250 ms and hides it while the game is minimised
- shows a deck-tracker-like widget with sample data; only the widgets take mouse input (GTK input shape), everything else passes through to the game
- two stacking modes: `override` (default, bypasses the window manager) and `above` (asks the window manager to keep the window on top)

## Requirements (Debian/Ubuntu/Mint)

```bash
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev
```

## Run

```bash
cargo run --release
```

Without a running game, `OVERLAY_SPIKE_DEMO=1` covers the whole screen. `OVERLAY_SPIKE_STACKING=above` switches the stacking mode. For measurements, `OVERLAY_SPIKE_DEMO_BOUNDS=x,y,width,height` fixes the demo window size and `OVERLAY_SPIKE_EXTRA_WINDOWS=n` opens n more webview windows. Results: [RESULTS.md](RESULTS.md).

## Measure

While the spike runs:

```bash
./measure.sh 30
```

## Test checklist

1. Overlay appears over the game and follows it when the window moves or changes size
2. Transparent areas show the game; clicks there reach the game
3. The widget can be dragged and collapsed; clicks on it do not reach the game
4. Works in windowed and in fullscreen mode, in both stacking modes
5. Overlay hides while the game is minimised and comes back afterwards
6. RAM (PSS total) and idle CPU from `measure.sh`
7. Native client: does Hearthstone read `~/Library/Preferences/Blizzard/Hearthstone/log.config` and write `Power.log`?
