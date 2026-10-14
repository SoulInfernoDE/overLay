//! Commands the launcher UI can invoke.

use overlay_games::Flavor;
use overlay_settings::Settings;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::state::AppState;
use crate::tray;

#[derive(Clone, Debug, Serialize)]
pub struct GameView {
    pub id: String,
    pub name: String,
    /// How the game currently runs, or `None` when it is not running.
    pub running: Option<Flavor>,
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub version: &'static str,
    pub settings: Settings,
    pub settings_path: Option<String>,
    pub games: Vec<GameView>,
}

pub fn games_view(state: &AppState) -> Vec<GameView> {
    let running = state.running.lock().expect("running lock");
    state
        .catalog
        .games()
        .iter()
        .map(|game| GameView {
            id: game.id.clone(),
            name: game.name.clone(),
            running: running
                .iter()
                .find(|r| r.game_id == game.id)
                .map(|r| r.flavor),
        })
        .collect()
}

#[tauri::command]
pub fn snapshot(state: State<'_, AppState>) -> Snapshot {
    Snapshot {
        version: env!("CARGO_PKG_VERSION"),
        settings: state.settings().clone(),
        settings_path: state.settings_path(),
        games: games_view(&state),
    }
}

#[tauri::command]
pub fn set_language(
    app: AppHandle,
    state: State<'_, AppState>,
    language: Option<String>,
) -> Result<Settings, String> {
    let settings = state.update_settings(|s| s.language = language)?;
    tray::refresh(&app);
    Ok(settings)
}

#[tauri::command]
pub fn set_start_minimized(state: State<'_, AppState>, enabled: bool) -> Result<Settings, String> {
    state.update_settings(|s| s.start_minimized = enabled)
}

#[tauri::command]
pub fn set_autostart(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<Settings, String> {
    apply_autostart(&app, enabled)?;
    state.update_settings(|s| s.autostart = enabled)
}

/// Brings the system's autostart entry in line with `enabled`.
pub fn apply_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let current = launcher.is_enabled().unwrap_or(false);
    match (enabled, current) {
        (true, false) => launcher.enable().map_err(|e| e.to_string()),
        (false, true) => launcher.disable().map_err(|e| e.to_string()),
        _ => Ok(()),
    }
}
