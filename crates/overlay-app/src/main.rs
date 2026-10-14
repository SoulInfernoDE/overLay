//! overLay host: tray icon, launcher window on demand, game detection.

mod commands;
mod detection;
mod l10n;
mod state;
mod tray;
mod window;

use overlay_games::Catalog;
use overlay_settings::SettingsStore;
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::state::AppState;

/// Passed by the autostart entry: start in the tray.
const MINIMIZED_ARG: &str = "--minimized";

fn main() {
    let store = SettingsStore::default_location();
    let (settings, error) = store.as_ref().map(SettingsStore::load).unwrap_or_default();
    if let Some(error) = error {
        eprintln!("[overlay] {error}");
    }
    let start_hidden = settings.start_minimized || std::env::args().any(|arg| arg == MINIMIZED_ARG);
    let state = AppState::new(store, settings, Catalog::builtin());

    let app = tauri::Builder::default()
        // Must be the first plugin: a second launch only opens the running instance's window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::open(app);
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        window::toggle(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![MINIMIZED_ARG]),
        ))
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::set_language,
            commands::set_start_minimized,
            commands::set_autostart,
        ])
        .setup(move |app| {
            let handle = app.handle();
            tray::create(handle)?;
            apply_startup_settings(handle);
            detection::spawn(handle.clone());
            if !start_hidden {
                window::open(handle);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build the overLay app");

    app.run(|_app, event| {
        // Closing the launcher window must not end the app; only "Quit" (exit code set) does.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}

fn apply_startup_settings(app: &AppHandle) {
    let settings = app.state::<AppState>().settings().clone();
    if let Err(err) = commands::apply_autostart(app, settings.autostart) {
        eprintln!("[overlay] cannot update the autostart entry: {err}");
    }
    let hotkey = settings.hotkeys.toggle_launcher.as_str();
    if let Err(err) = app.global_shortcut().register(hotkey) {
        // Expected on Wayland, where global shortcuts need the XDG portal (planned).
        eprintln!("[overlay] cannot register hotkey {hotkey:?}: {err}");
    }
}
