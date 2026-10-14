//! The launcher window runs in its own process, started on demand.

use overlay_ipc::{CoreMessage, UiRole};
use tauri::{AppHandle, Manager};

use crate::host::children::UiProcesses;

pub fn open(app: &AppHandle) {
    let processes = app.state::<UiProcesses>();
    if processes.is_running(UiRole::Launcher) {
        processes.send(UiRole::Launcher, &CoreMessage::Focus);
    } else if let Err(err) = processes.start(app, UiRole::Launcher, &[]) {
        eprintln!("[overlay] cannot start the launcher: {err}");
    }
}

pub fn toggle(app: &AppHandle) {
    let processes = app.state::<UiProcesses>();
    if !processes.send(UiRole::Launcher, &CoreMessage::Close) {
        open(app);
    }
}
