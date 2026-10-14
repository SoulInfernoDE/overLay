//! Answers requests from UI processes.

use overlay_ipc::{CoreMessage, Request, UiRole};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::host::children::UiProcesses;
use crate::host::mods::{self, ModRegistry};
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
            let value = settings_changed(app, settings);
            // Mod descriptions are localized too.
            mods::mods_changed(app);
            value
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
        Request::ListMods => json(app.state::<ModRegistry>().list(&state.settings())),
        Request::ReloadMods => {
            let settings = state.settings().clone();
            app.state::<ModRegistry>().reload(&settings);
            json(mods::mods_changed(app))
        }
        Request::SetModEnabled { mod_id, enabled } => {
            state.update_settings(|s| s.mods.entry(mod_id).or_default().enabled = enabled)?;
            json(mods::mods_changed(app))
        }
        Request::SetModSetting {
            mod_id,
            setting,
            value,
        } => {
            let info = app
                .state::<ModRegistry>()
                .get(&mod_id)
                .ok_or_else(|| format!("unknown mod {mod_id}"))?;
            let spec = info
                .manifest
                .settings
                .iter()
                .find(|s| s.id == setting)
                .ok_or_else(|| format!("mod {mod_id} has no setting {setting}"))?;
            if !mods::fits(&spec.kind, &value) {
                return Err(format!("invalid value for setting {setting}"));
            }
            let settings = state.update_settings(|s| {
                s.mods
                    .entry(mod_id.clone())
                    .or_default()
                    .values
                    .insert(setting, value);
            })?;
            app.state::<UiProcesses>()
                .broadcast(&CoreMessage::ModEvent {
                    data: json(mods::effective_values(&info, &settings))?,
                    mod_id: Some(mod_id),
                    name: "settings".into(),
                    permission: None,
                });
            json(mods::mods_changed(app))
        }
        Request::SetDeveloperMode { enabled } => {
            let settings = state.update_settings(|s| s.developer.enabled = enabled)?;
            settings_changed(app, settings.clone())?;
            app.state::<ModRegistry>().reload(&settings);
            json(mods::mods_changed(app))
        }
        Request::AddDeveloperModPath { path } => {
            let path = std::path::PathBuf::from(path.trim());
            if !path.is_absolute() {
                return Err("give the full path to the mod folder".into());
            }
            let settings = state.update_settings(|s| {
                if !s.developer.mod_paths.contains(&path) {
                    s.developer.mod_paths.push(path);
                }
            })?;
            settings_changed(app, settings.clone())?;
            app.state::<ModRegistry>().reload(&settings);
            json(mods::mods_changed(app))
        }
        Request::RemoveDeveloperModPath { path } => {
            let path = std::path::PathBuf::from(path);
            let settings =
                state.update_settings(|s| s.developer.mod_paths.retain(|p| *p != path))?;
            settings_changed(app, settings.clone())?;
            app.state::<ModRegistry>().reload(&settings);
            json(mods::mods_changed(app))
        }
        Request::ModCall { mod_id, call } => mods::call(app, &mod_id, call),
        Request::ReportBlocked {
            mod_id,
            blocked_uri,
            directive,
        } => {
            eprintln!("[overlay] blocked a request of mod {mod_id}: {blocked_uri} ({directive})");
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
