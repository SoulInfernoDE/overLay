//! Global hotkeys of the core: show/hide the launcher and the overlay, plus the hotkeys
//! that enabled mods declare in their manifest (with the `hotkeys` permission).
//!
//! Works on Windows and X11. Wayland needs the XDG GlobalShortcuts portal (planned).

use std::str::FromStr;
use std::sync::Mutex;

use overlay_ipc::CoreMessage;
use overlay_mods::Permission;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::host::children::UiProcesses;
use crate::host::mods::ModRegistry;
use crate::host::state::AppState;
use crate::host::{launcher, overlay};

#[derive(Default)]
struct Bindings {
    launcher: Option<Shortcut>,
    overlay: Option<Shortcut>,
    /// Shortcut, mod id, hotkey id.
    mods: Vec<(Shortcut, String, String)>,
}

/// The registered shortcuts, to tell them apart in the handler.
#[derive(Default)]
pub struct Registered(Mutex<Bindings>);

/// (Re-)registers all hotkeys; called at start and whenever mods or settings change.
pub fn register_all(app: &AppHandle) {
    if let Err(err) = app.global_shortcut().unregister_all() {
        eprintln!("[overlay] cannot reset hotkeys: {err}");
    }
    let settings = app.state::<AppState>().settings().clone();
    let mut bindings = Bindings {
        launcher: register_one(app, &settings.hotkeys.toggle_launcher),
        overlay: register_one(app, &settings.hotkeys.toggle_overlay),
        mods: Vec::new(),
    };
    for info in app.state::<ModRegistry>().enabled_mods(&settings) {
        if !info.manifest.has(Permission::Hotkeys) {
            continue;
        }
        for (hotkey, spec) in &info.manifest.hotkeys {
            if let Some(shortcut) = register_one(app, &spec.default) {
                bindings
                    .mods
                    .push((shortcut, info.manifest.id.clone(), hotkey.clone()));
            }
        }
    }
    *app.state::<Registered>().0.lock().expect("hotkeys lock") = bindings;
}

fn register_one(app: &AppHandle, accelerator: &str) -> Option<Shortcut> {
    let shortcut = match Shortcut::from_str(accelerator) {
        Ok(shortcut) => shortcut,
        Err(err) => {
            eprintln!("[overlay] invalid hotkey {accelerator:?}: {err}");
            return None;
        }
    };
    if let Err(err) = app.global_shortcut().register(shortcut) {
        // Expected on Wayland until the portal is supported, or if another program or
        // mod already uses the combination.
        eprintln!("[overlay] cannot register hotkey {accelerator:?}: {err}");
        return None;
    }
    Some(shortcut)
}

pub fn handle(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let Some(registered) = app.try_state::<Registered>() else {
        return;
    };
    let bindings = registered.0.lock().expect("hotkeys lock");
    if bindings.launcher.as_ref() == Some(shortcut) {
        drop(bindings);
        launcher::toggle(app);
    } else if bindings.overlay.as_ref() == Some(shortcut) {
        drop(bindings);
        overlay::toggle_visibility(app);
    } else if let Some((_, mod_id, hotkey)) = bindings.mods.iter().find(|(s, _, _)| s == shortcut) {
        let message = CoreMessage::ModEvent {
            mod_id: Some(mod_id.clone()),
            name: "hotkey".into(),
            data: serde_json::json!({ "id": hotkey }),
            permission: Some(Permission::Hotkeys),
        };
        drop(bindings);
        app.state::<UiProcesses>().broadcast(&message);
    }
}
