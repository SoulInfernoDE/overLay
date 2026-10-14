//! Global hotkeys of the core: show/hide the launcher and the in-game overlay.
//!
//! Works on Windows and X11. Wayland needs the XDG GlobalShortcuts portal (planned).

use std::str::FromStr;

use overlay_settings::Hotkeys;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::host::{launcher, overlay};

/// The registered shortcuts, to tell them apart in the handler.
#[derive(Default)]
pub struct Registered {
    launcher: Option<Shortcut>,
    overlay: Option<Shortcut>,
}

pub fn register(app: &AppHandle, hotkeys: &Hotkeys) {
    let registered = Registered {
        launcher: register_one(app, &hotkeys.toggle_launcher),
        overlay: register_one(app, &hotkeys.toggle_overlay),
    };
    app.manage(registered);
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
        // Expected on Wayland until the portal is supported.
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
    if registered.launcher.as_ref() == Some(shortcut) {
        launcher::toggle(app);
    } else if registered.overlay.as_ref() == Some(shortcut) {
        overlay::toggle_visibility(app);
    }
}
