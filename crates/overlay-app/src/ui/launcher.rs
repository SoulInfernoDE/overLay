//! Launcher process: one window with the library, store and settings.

use std::sync::Arc;

use overlay_ipc::CoreMessage;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use super::mods::{self, ModMap};
use super::{CoreClient, listen_to_core};

const WINDOW: &str = "main";

pub fn run() {
    let client = Arc::new(CoreClient::default());
    mods::register(tauri::Builder::default())
        .manage(Arc::clone(&client))
        .manage(ModMap::default())
        .invoke_handler(tauri::generate_handler![
            crate::ui::core_request,
            crate::ui::mods::mod_base_url
        ])
        .setup(move |app| {
            listen_to_core(app.handle().clone(), Arc::clone(&client), on_message);
            WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("index.html".into()))
                .title("overLay")
                .inner_size(1100.0, 720.0)
                .min_inner_size(820.0, 560.0)
                .decorations(false)
                .center()
                .build()?;
            Ok(())
        })
        // Closing the window ends the process (Tauri's default when the last window closes).
        .run(tauri::generate_context!())
        .expect("failed to run the overLay launcher");
}

fn on_message(app: &AppHandle, message: CoreMessage) {
    match message.clone() {
        CoreMessage::GamesChanged { games } => {
            let _ = app.emit("games-changed", games);
        }
        CoreMessage::SettingsChanged { settings } => {
            let _ = app.emit("settings-changed", settings);
        }
        CoreMessage::Focus => {
            if let Some(window) = app.get_webview_window(WINDOW) {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }
        CoreMessage::Close => app.exit(0),
        CoreMessage::ModsChanged { list } => {
            app.state::<ModMap>().update(&list);
            let _ = app.emit("mods-changed", list);
        }
        CoreMessage::ModEvent { .. } => {
            let _ = app.emit("mod-event", message);
        }
        CoreMessage::Response { .. }
        | CoreMessage::Target { .. }
        | CoreMessage::ToggleOverlay
        | CoreMessage::ToastsPending => {}
    }
}
