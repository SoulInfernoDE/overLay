//! The core process: tray icon, game detection, settings, hotkeys. It loads no webview
//! itself; the launcher and the overlay run as child processes (see `children`).

mod children;
mod detection;
mod hotkeys;
mod launcher;
mod mods;
mod overlay;
mod providers;
mod requests;
mod state;
mod toasts;
mod tray;

use overlay_games::Catalog;
use overlay_settings::{LayoutStore, SettingsStore};
use tauri::{Manager, RunEvent};
use tauri_plugin_autostart::MacosLauncher;

use children::UiProcesses;
use state::AppState;

/// Passed by the autostart entry: start in the tray.
const MINIMIZED_ARG: &str = "--minimized";

pub fn run() {
    let store = SettingsStore::default_location();
    let (settings, error) = store.as_ref().map(SettingsStore::load).unwrap_or_default();
    if let Some(error) = error {
        eprintln!("[overlay] {error}");
    }
    let layout_store = LayoutStore::default_location();
    let (layout, error) = layout_store
        .as_ref()
        .map(LayoutStore::load)
        .unwrap_or_default();
    if let Some(error) = error {
        eprintln!("[overlay] {error}");
    }
    let start_hidden = settings.start_minimized || std::env::args().any(|arg| arg == MINIMIZED_ARG);
    let state = AppState::new(store, settings, layout_store, layout, Catalog::builtin());

    let app = tauri::Builder::default()
        // Must be the first plugin: a second launch only opens the running launcher.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            launcher::open(app);
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(hotkeys::handle)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![MINIMIZED_ARG]),
        ))
        .manage(state)
        .manage(UiProcesses::default())
        .manage(overlay::CurrentTarget::default())
        .manage(mods::ModRegistry::new())
        .manage(providers::Providers::default())
        .manage(toasts::Toasts::default())
        .manage(hotkeys::Registered::default())
        .setup(move |app| {
            let handle = app.handle();
            tray::create(handle)?;
            let settings = handle.state::<AppState>().settings().clone();
            if let Err(err) = requests::apply_autostart(handle, settings.autostart) {
                eprintln!("[overlay] cannot update the autostart entry: {err}");
            }
            handle.state::<mods::ModRegistry>().reload(&settings);
            hotkeys::register_all(handle);
            detection::spawn(handle.clone());
            if !start_hidden {
                launcher::open(handle);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build the overLay core");

    app.run(|_app, event| {
        // The core has no windows of its own; only "Quit" (exit code set) ends it.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
