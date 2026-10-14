//! Overlay process: one transparent window over the game that holds all in-game widgets.
//!
//! Lessons from the overlay spike (`spikes/overlay-window/RESULTS.md`):
//! - one shared webview, because every webview window costs its own WebKit process;
//! - the window never takes focus, or the game swallows the next click;
//! - clickable regions arrive in CSS pixels and are converted by the device pixel ratio.
//!
//! The overlay is visible only while the game window is shown and active (or the overlay
//! itself was clicked) and the user has not hidden it with the hotkey.

mod input;
#[cfg(windows)]
mod tracker_windows;
#[cfg(target_os = "linux")]
mod tracker_x11;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use overlay_ipc::{CoreMessage, OverlayTarget};
use overlay_settings::{SettingsStore, Stacking};
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewUrl,
    WebviewWindowBuilder,
};

use super::{CoreClient, listen_to_core};

const WINDOW: &str = "overlay";

/// The game window's client area in physical screen pixels and its state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameWindow {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// Minimised or unmapped.
    pub hidden: bool,
    /// The game (or the overlay itself) has the focus.
    pub active: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct OverlayStatus {
    target: Option<OverlayTarget>,
    window: Option<GameWindow>,
    user_hidden: bool,
}

#[derive(Default)]
struct OverlayState {
    status: Mutex<OverlayStatus>,
    stacking: Mutex<Stacking>,
    window_ready: AtomicBool,
    tracking: AtomicBool,
}

pub fn run() {
    // The overlay only needs the stacking mode at start; everything else comes from the core.
    let stacking = SettingsStore::default_location()
        .map(|store| store.load().0.overlay.stacking)
        .unwrap_or_default();
    let client = Arc::new(CoreClient::default());
    let state = OverlayState::default();
    *state.stacking.lock().expect("stacking lock") = stacking;

    tauri::Builder::default()
        .manage(Arc::clone(&client))
        .manage(state)
        .manage(input::Regions::default())
        .invoke_handler(tauri::generate_handler![
            crate::ui::core_request,
            overlay_status,
            input::set_interactive_regions
        ])
        .setup(move |app| {
            listen_to_core(app.handle().clone(), Arc::clone(&client), on_message);
            let mut builder =
                WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("overlay.html".into()))
                    .title("overLay overlay")
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(stacking == Stacking::Above)
                    .skip_taskbar(true)
                    .focused(false)
                    .resizable(false)
                    .shadow(false)
                    .visible(false)
                    .inner_size(800.0, 600.0);
            // A separate WebKit data folder, so the launcher and the overlay never compete
            // for the same storage locks.
            if let Ok(dir) = app.path().app_local_data_dir() {
                builder = builder.data_directory(dir.join("overlay-webview"));
            }
            let window = builder.build()?;
            input::prepare_window(&window, stacking)?;
            let overlay = app.state::<OverlayState>();
            overlay.window_ready.store(true, Ordering::SeqCst);
            start_tracking(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the overLay overlay");
}

fn on_message(app: &AppHandle, message: CoreMessage) {
    let overlay = app.state::<OverlayState>();
    match message {
        CoreMessage::Target { target } => {
            overlay.status.lock().expect("status lock").target = Some(target.clone());
            let _ = app.emit("overlay-target", target);
            start_tracking(app);
        }
        CoreMessage::ToggleOverlay => {
            let hidden = {
                let mut status = overlay.status.lock().expect("status lock");
                status.user_hidden = !status.user_hidden;
                status.user_hidden
            };
            eprintln!(
                "[overlay-ui] overlay {}",
                if hidden { "hidden" } else { "shown" }
            );
            let _ = app.emit("overlay-toggled", hidden);
            apply(app);
        }
        CoreMessage::SettingsChanged { settings } => {
            let _ = app.emit("settings-changed", settings);
        }
        CoreMessage::GamesChanged { games } => {
            let _ = app.emit("games-changed", games);
        }
        CoreMessage::Close => app.exit(0),
        CoreMessage::Response { .. } | CoreMessage::Focus => {}
    }
}

/// Starts following the game window once both the window and the target exist.
fn start_tracking(app: &AppHandle) {
    let overlay = app.state::<OverlayState>();
    let pid = match &overlay.status.lock().expect("status lock").target {
        Some(target) => target.pid,
        None => return,
    };
    if !overlay.window_ready.load(Ordering::SeqCst) || overlay.tracking.swap(true, Ordering::SeqCst)
    {
        return;
    }
    let app = app.clone();
    // `OVERLAY_DEBUG=1` logs every change of the game window (for bug reports).
    let debug = std::env::var_os("OVERLAY_DEBUG").is_some();
    let on_change = move |window: Option<GameWindow>| {
        if debug {
            eprintln!("[overlay-ui] game window: {window:?}");
        }
        app.state::<OverlayState>()
            .status
            .lock()
            .expect("status lock")
            .window = window;
        let _ = app.emit("game-window", window);
        apply(&app);
    };
    #[cfg(target_os = "linux")]
    tracker_x11::spawn(pid, on_change);
    #[cfg(windows)]
    tracker_windows::spawn(pid, on_change);
    #[cfg(not(any(target_os = "linux", windows)))]
    let _ = (pid, on_change);
}

/// Shows the overlay over the game window or hides it.
fn apply(app: &AppHandle) {
    let overlay = app.state::<OverlayState>();
    let Some(window) = app.get_webview_window(WINDOW) else {
        return;
    };
    let status = overlay.status.lock().expect("status lock").clone();
    let game = status
        .window
        .filter(|w| !w.hidden && w.active && !status.user_hidden);
    let _ = app.emit("overlay-visible", game.is_some());
    match game {
        Some(game) => {
            let _ = window.set_position(PhysicalPosition::new(game.x, game.y));
            let _ = window.set_size(PhysicalSize::new(game.width, game.height));
            let _ = window.show();
            if *overlay.stacking.lock().expect("stacking lock") == Stacking::Override {
                input::raise(&window);
            }
        }
        None => {
            let _ = window.hide();
        }
    }
}

/// Current target, game window and visibility, for a page that loads after the events.
#[tauri::command]
fn overlay_status(state: State<'_, OverlayState>) -> OverlayStatus {
    state.status.lock().expect("status lock").clone()
}
