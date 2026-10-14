//! Starts the overlay process while a game runs (and the overlay is enabled) and stops it
//! when the game exits.

use std::sync::Mutex;

use overlay_ipc::{CoreMessage, OverlayTarget, UiRole};
use tauri::{AppHandle, Manager};

use crate::host::children::UiProcesses;
use crate::host::state::AppState;

/// PID of the game the running overlay process follows.
#[derive(Default)]
pub struct CurrentTarget(Mutex<Option<u32>>);

/// Brings the overlay process in line with the running games and the settings. Also
/// called when an overlay process ends, so a restarted game gets a fresh overlay.
pub fn sync(app: &AppHandle) {
    let state = app.state::<AppState>();
    let processes = app.state::<UiProcesses>();
    let target_state = app.state::<CurrentTarget>();
    let mut current = target_state.0.lock().expect("target lock");
    let enabled = state.settings().overlay.enabled;
    let game = state.running.lock().expect("running lock").first().cloned();

    if processes.is_running(UiRole::Overlay) {
        let wanted = game.as_ref().filter(|_| enabled).map(|g| g.pid);
        if wanted != *current {
            // Restart (or stop) via the process ending; it calls sync again.
            processes.send(UiRole::Overlay, &CoreMessage::Close);
        }
        return;
    }
    *current = None;
    let (true, Some(game)) = (enabled, game) else {
        return;
    };
    let target = OverlayTarget {
        game_name: state.game_name(&game.game_id).unwrap_or_default(),
        game_id: game.game_id,
        pid: game.pid,
    };
    match processes.start(app, UiRole::Overlay, &[CoreMessage::Target { target }]) {
        Ok(()) => *current = Some(game.pid),
        Err(err) => eprintln!("[overlay] cannot start the overlay: {err}"),
    }
}

pub fn toggle_visibility(app: &AppHandle) {
    app.state::<UiProcesses>()
        .send(UiRole::Overlay, &CoreMessage::ToggleOverlay);
}
