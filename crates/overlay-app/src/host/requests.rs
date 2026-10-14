//! Answers requests from UI processes.

use overlay_ipc::{CoreMessage, Request, UiRole};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::host::children::UiProcesses;
use crate::host::state::AppState;
use crate::host::{overlay, tray};

pub fn handle(
    app: &AppHandle,
    _from: UiRole,
    request: Request,
) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    match request {
        Request::Snapshot => json(state.snapshot()),
        Request::SetLanguage { language } => {
            let settings = state.update_settings(|s| s.language = language)?;
            tray::refresh(app);
            settings_changed(app, settings)
        }
        Request::SetAutostart { enabled } => {
            apply_autostart(app, enabled)?;
            let settings = state.update_settings(|s| s.autostart = enabled)?;
            settings_changed(app, settings)
        }
        Request::SetStartMinimized { enabled } => {
            let settings = state.update_settings(|s| s.start_minimized = enabled)?;
            settings_changed(app, settings)
        }
        Request::SetOverlayEnabled { enabled } => {
            let settings = state.update_settings(|s| s.overlay.enabled = enabled)?;
            let value = settings_changed(app, settings);
            overlay::sync(app);
            value
        }
        Request::SetDemoWidget { enabled } => {
            let settings = state.update_settings(|s| s.overlay.demo_widget = enabled)?;
            settings_changed(app, settings)
        }
        Request::LoadLayout {
            game_id,
            resolution,
        } => json(state.widget_positions(&game_id, &resolution)),
        Request::SaveWidgetPosition {
            game_id,
            resolution,
            widget,
            position,
        } => {
            state.save_widget_position(&game_id, &resolution, &widget, position)?;
            Ok(serde_json::Value::Null)
        }
    }
}

/// Tells every UI process about new settings and returns them to the requester.
fn settings_changed(
    app: &AppHandle,
    settings: overlay_settings::Settings,
) -> Result<serde_json::Value, String> {
    app.state::<UiProcesses>()
        .broadcast(&CoreMessage::SettingsChanged {
            settings: settings.clone(),
        });
    json(settings)
}

fn json(value: impl Serialize) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
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
