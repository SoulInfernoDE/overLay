//! Spike: a transparent, click-through Tauri overlay that follows the Hearthstone window.
//!
//! Environment switches:
//! - `OVERLAY_SPIKE_DEMO=1`: cover the whole screen instead of waiting for the game.
//! - `OVERLAY_SPIKE_DEMO_BOUNDS=x,y,width,height`: demo mode with a fixed window size.
//! - `OVERLAY_SPIKE_EXTRA_WINDOWS=n`: open n more small webview windows, to measure the
//!   cost of one window per widget.
//! - `OVERLAY_SPIKE_STACKING=above|override`: `above` asks the window manager to keep the
//!   window on top; `override` (default) bypasses the window manager (override-redirect),
//!   which is what keeps it above fullscreen windows on most X11 window managers.

mod game;

use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use game::{Bounds, GameProcess, X11Tracker};

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const PROCESS_RESCAN: Duration = Duration::from_secs(2);
const RAISE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Stacking {
    Above,
    Override,
}

impl Stacking {
    fn from_env() -> Self {
        match std::env::var("OVERLAY_SPIKE_STACKING").as_deref() {
            Ok("above") => Self::Above,
            _ => Self::Override,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct SpikeConfig {
    demo: bool,
    stacking: Stacking,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum TrackState {
    Searching,
    Demo {
        bounds: Bounds,
    },
    Tracking {
        pid: u32,
        flavor: &'static str,
        bounds: Bounds,
        hidden: bool,
        active: bool,
    },
}

impl TrackState {
    fn visible_bounds(&self) -> Option<Bounds> {
        match self {
            Self::Demo { bounds } => Some(*bounds),
            Self::Tracking {
                bounds,
                hidden: false,
                ..
            } => Some(*bounds),
            _ => None,
        }
    }
}

/// A rectangle in CSS pixels of the overlay page.
#[derive(Debug, Deserialize)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Latest tracker state, for a page that starts listening after the first event.
struct CurrentState(Mutex<TrackState>);

#[tauri::command]
fn spike_config(config: tauri::State<'_, SpikeConfig>) -> SpikeConfig {
    *config
}

#[tauri::command]
fn track_state(current: tauri::State<'_, CurrentState>) -> TrackState {
    current.0.lock().expect("state lock").clone()
}

/// Restricts mouse input to `rects`; everything else passes through to the game.
///
/// `device_pixel_ratio` is the page's `window.devicePixelRatio`. CSS pixels are scaled by
/// it (e.g. 1.2 at 115 dpi), while the input shape is set in GDK units, so the rectangles
/// must be converted or the clickable area drifts away from the visible widgets.
#[tauri::command]
fn set_interactive_regions(
    window: WebviewWindow,
    rects: Vec<Rect>,
    device_pixel_ratio: f64,
) -> Result<(), String> {
    let target = window.clone();
    window
        .run_on_main_thread(move || apply_input_shape(&target, &rects, device_pixel_ratio))
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn apply_input_shape(window: &WebviewWindow, rects: &[Rect], device_pixel_ratio: f64) {
    use gtk::cairo::{RectangleInt, Region};
    use gtk::prelude::*;

    let Ok(gtk_window) = window.gtk_window() else {
        return;
    };
    // CSS pixels -> device pixels -> GDK units (GDK scale is an integer, usually 1).
    let factor = device_pixel_ratio / f64::from(gtk_window.scale_factor().max(1));
    let rects: Vec<RectangleInt> = rects
        .iter()
        .map(|r| {
            let left = (r.x * factor).floor();
            let top = (r.y * factor).floor();
            let right = ((r.x + r.width) * factor).ceil();
            let bottom = ((r.y + r.height) * factor).ceil();
            RectangleInt::new(
                left as i32,
                top as i32,
                (right - left) as i32,
                (bottom - top) as i32,
            )
        })
        .collect();
    eprintln!("[spike] input shape (factor {factor:.3}): {rects:?}");
    gtk_window.input_shape_combine_region(Some(&Region::create_rectangles(&rects)));
}

#[cfg(not(target_os = "linux"))]
fn apply_input_shape(_window: &WebviewWindow, _rects: &[Rect], _device_pixel_ratio: f64) {}

/// Click-through from the start, and override-redirect if requested. Must run before
/// the window is mapped for the first time.
#[cfg(target_os = "linux")]
fn prepare_window(window: &WebviewWindow, stacking: Stacking) -> tauri::Result<()> {
    use gtk::prelude::*;

    let gtk_window = window.gtk_window()?;
    // An overlay must never take focus from the game, or the game swallows the next click.
    gtk_window.set_accept_focus(false);
    gtk_window.set_focus_on_map(false);
    gtk_window.realize();
    if stacking == Stacking::Override
        && let Some(gdk_window) = gtk_window.window()
    {
        gdk_window.set_override_redirect(true);
    }
    apply_input_shape(window, &[], 1.0);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn prepare_window(_window: &WebviewWindow, _stacking: Stacking) -> tauri::Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn raise(window: &WebviewWindow) {
    use gtk::prelude::*;

    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Some(gdk_window) = target.gtk_window().ok().and_then(|w| w.window()) {
            gdk_window.raise();
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn raise(_window: &WebviewWindow) {}

fn apply_state(window: &WebviewWindow, state: &TrackState) {
    match state.visible_bounds() {
        Some(b) => {
            let _ = window.set_position(PhysicalPosition::new(b.x, b.y));
            let _ = window.set_size(PhysicalSize::new(b.width, b.height));
            let _ = window.show();
        }
        None => {
            let _ = window.hide();
        }
    }
}

fn poll(
    tracker: &X11Tracker,
    demo: bool,
    game: &mut Option<GameProcess>,
    last_scan: &mut Instant,
) -> TrackState {
    if demo {
        if let Some(bounds) = demo_bounds() {
            return TrackState::Demo { bounds };
        }
        return match tracker.root_bounds() {
            Ok(bounds) => TrackState::Demo { bounds },
            Err(_) => TrackState::Searching,
        };
    }
    if game.as_ref().is_some_and(|g| !game::is_alive(g.pid)) {
        *game = None;
    }
    if game.is_none() && last_scan.elapsed() >= PROCESS_RESCAN {
        *game = game::find_process();
        *last_scan = Instant::now();
    }
    let Some(process) = game else {
        return TrackState::Searching;
    };
    let window = match tracker.window_for_pid(process.pid) {
        Ok(Some(window)) => window,
        _ => return TrackState::Searching,
    };
    match (
        tracker.bounds(window),
        tracker.is_hidden(window),
        tracker.is_active(window),
    ) {
        (Ok(bounds), Ok(hidden), Ok(active)) => TrackState::Tracking {
            pid: process.pid,
            flavor: process.flavor,
            bounds,
            hidden,
            active,
        },
        _ => TrackState::Searching,
    }
}

fn demo_bounds() -> Option<Bounds> {
    let value = std::env::var("OVERLAY_SPIKE_DEMO_BOUNDS").ok()?;
    let parts: Vec<i64> = value
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();
    let [x, y, width, height] = parts[..] else {
        return None;
    };
    Some(Bounds {
        x: i32::try_from(x).ok()?,
        y: i32::try_from(y).ok()?,
        width: u32::try_from(width).ok()?,
        height: u32::try_from(height).ok()?,
    })
}

fn spawn_tracker(app: AppHandle, config: SpikeConfig) {
    thread::spawn(move || {
        let tracker = match X11Tracker::connect() {
            Ok(tracker) => tracker,
            Err(err) => {
                eprintln!("[spike] cannot connect to X11: {err}");
                return;
            }
        };
        let Some(window) = app.get_webview_window("overlay") else {
            return;
        };
        let mut game = None;
        let mut last_scan = Instant::now() - PROCESS_RESCAN;
        let mut last_raise = Instant::now();
        let mut last_state = None;
        loop {
            let state = poll(&tracker, config.demo, &mut game, &mut last_scan);
            if last_state.as_ref() != Some(&state) {
                eprintln!("[spike] {state:?}");
                apply_state(&window, &state);
                *app.state::<CurrentState>().0.lock().expect("state lock") = state.clone();
                let _ = app.emit("track-state", &state);
                last_state = Some(state);
            }
            if config.stacking == Stacking::Override
                && last_state
                    .as_ref()
                    .is_some_and(|s| s.visible_bounds().is_some())
                && last_raise.elapsed() >= RAISE_INTERVAL
            {
                raise(&window);
                last_raise = Instant::now();
            }
            thread::sleep(POLL_INTERVAL);
        }
    });
}

fn open_extra_windows(app: &tauri::App) -> tauri::Result<()> {
    let count: u32 = std::env::var("OVERLAY_SPIKE_EXTRA_WINDOWS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    for i in 0..count {
        tauri::WebviewWindowBuilder::new(
            app,
            format!("extra-{i}"),
            tauri::WebviewUrl::App("index.html".into()),
        )
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .inner_size(300.0, 300.0)
        .position(f64::from(40 + i * 320), 400.0)
        .build()?;
    }
    Ok(())
}

fn main() {
    // Both the native client and Proton games run on X11/XWayland, so the overlay does too.
    if std::env::var_os("GDK_BACKEND").is_none() {
        // SAFETY: called first thing in main, before any other thread exists.
        unsafe { std::env::set_var("GDK_BACKEND", "x11") };
    }
    let config = SpikeConfig {
        demo: std::env::var("OVERLAY_SPIKE_DEMO").as_deref() == Ok("1"),
        stacking: Stacking::from_env(),
    };
    eprintln!("[spike] {config:?}");

    tauri::Builder::default()
        .manage(config)
        .manage(CurrentState(Mutex::new(TrackState::Searching)))
        .invoke_handler(tauri::generate_handler![
            spike_config,
            track_state,
            set_interactive_regions
        ])
        .setup(move |app| {
            let window = app
                .get_webview_window("overlay")
                .expect("overlay window is declared in tauri.conf.json");
            prepare_window(&window, config.stacking)?;
            spawn_tracker(app.handle().clone(), config);
            open_extra_windows(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the overlay spike");
}
