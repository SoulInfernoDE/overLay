//! overLay: one executable, three roles. Without arguments it is the core (tray, game
//! detection, settings); with `--ui launcher` or `--ui overlay` it is a UI process that
//! the core starts and supervises.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod host;
mod l10n;
mod ui;

use overlay_ipc::UiRole;

fn main() {
    match UiRole::from_args(std::env::args()) {
        Some(role) => ui::run(role),
        None => host::run(),
    }
}
