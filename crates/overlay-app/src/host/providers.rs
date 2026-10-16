//! Game providers: turn a running game into events for mods (`game.events`).
//!
//! For now there is one, for Hearthstone. It runs in a thread of the core while the game
//! runs: it makes sure the game writes the logs it needs, follows `Power.log`, sends every
//! event to the UI processes and keeps a summary of the game for mods that start later.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use overlay_games::RunningGame;
use overlay_hearthstone::locate::locate;
use overlay_hearthstone::{Provider, log_config};
use overlay_i18n::FluentArgs;
use overlay_ipc::CoreMessage;
use overlay_mods::Permission;
use tauri::{AppHandle, Manager};

use crate::host::children::UiProcesses;
use crate::host::state::AppState;

const POLL: Duration = Duration::from_millis(100);
/// The game may need a moment before its folders and process details are readable.
const LOCATE_ATTEMPTS: u32 = 30;

#[derive(Default)]
pub struct Providers {
    running: Mutex<HashMap<u32, Arc<AtomicBool>>>,
    /// Latest summary of the running game, as sent to mods.
    info: Mutex<Option<serde_json::Value>>,
}

impl Providers {
    pub fn info(&self) -> Option<serde_json::Value> {
        self.info.lock().expect("info lock").clone()
    }
}

pub fn start(app: &AppHandle, game: &RunningGame) {
    if game.game_id != "hearthstone" {
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let providers = app.state::<Providers>();
    {
        let mut running = providers.running.lock().expect("providers lock");
        if running.contains_key(&game.pid) {
            return;
        }
        running.insert(game.pid, Arc::clone(&stop));
    }
    let app = app.clone();
    let game = game.clone();
    let spawned = thread::Builder::new()
        .name("hearthstone-provider".into())
        .spawn(move || run_hearthstone(&app, &game, &stop));
    if let Err(err) = spawned {
        eprintln!("[overlay] cannot start the Hearthstone provider: {err}");
    }
}

pub fn stop(app: &AppHandle, game: &RunningGame) {
    let providers = app.state::<Providers>();
    if let Some(stop) = providers
        .running
        .lock()
        .expect("providers lock")
        .remove(&game.pid)
    {
        stop.store(true, Ordering::SeqCst);
    }
    *providers.info.lock().expect("info lock") = None;
}

fn run_hearthstone(app: &AppHandle, game: &RunningGame, stop: &AtomicBool) {
    let mut install = None;
    for _ in 0..LOCATE_ATTEMPTS {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        install = locate(game.pid, game.flavor);
        if install.is_some() {
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }
    let Some(install) = install else {
        eprintln!(
            "[overlay] cannot find the logs of Hearthstone (pid {})",
            game.pid
        );
        return;
    };
    eprintln!("[overlay] Hearthstone logs: {}", install.logs_dir.display());
    match log_config::ensure(&install.log_config) {
        Ok(true) => {
            eprintln!("[overlay] updated {}", install.log_config.display());
            ask_for_restart(app);
        }
        Ok(false) => {}
        Err(err) => eprintln!(
            "[overlay] cannot update {}: {err}",
            install.log_config.display()
        ),
    }

    let processes = app.state::<UiProcesses>();
    let providers = app.state::<Providers>();
    let mut provider = Provider::new(install.logs_dir);
    let mut last_info = String::new();
    while !stop.load(Ordering::SeqCst) {
        match provider.poll() {
            // Nothing new (e.g. in the menus): the summary cannot have changed either.
            Ok(events) if events.is_empty() => {}
            Ok(events) => {
                for event in events {
                    processes.broadcast(&CoreMessage::ModEvent {
                        mod_id: None,
                        name: "game-event".into(),
                        data: serde_json::to_value(event).unwrap_or_default(),
                        permission: Some(Permission::GameEvents),
                    });
                }
                let info = serde_json::to_value(provider.info()).unwrap_or_default();
                let text = info.to_string();
                if text != last_info {
                    last_info = text;
                    *providers.info.lock().expect("info lock") = Some(info.clone());
                    processes.broadcast(&CoreMessage::ModEvent {
                        mod_id: None,
                        name: "game-info".into(),
                        data: info,
                        permission: Some(Permission::GameEvents),
                    });
                }
            }
            Err(err) => eprintln!("[overlay] cannot read Power.log: {err}"),
        }
        thread::sleep(POLL);
    }
}

fn ask_for_restart(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut args = FluentArgs::new();
    args.set("game", "Hearthstone");
    let text = state.t_args("provider-restart-needed", &args);
    crate::host::toasts::push(app, overlay_ipc::Toast { text, source: None });
}
