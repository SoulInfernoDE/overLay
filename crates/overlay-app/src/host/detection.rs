//! Background thread that watches the process list and reports games starting and exiting.

use std::thread;
use std::time::Duration;

use overlay_games::{Detector, Platform, list_processes};
use overlay_ipc::CoreMessage;
use tauri::{AppHandle, Manager};

use crate::host::children::UiProcesses;
use crate::host::state::AppState;
use crate::host::{overlay, tray};

/// A process scan reads about one small file per process; every 2 s is plenty.
const INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn(app: AppHandle) {
    let spawned = thread::Builder::new()
        .name("game-detection".into())
        .spawn(move || {
            let state = app.state::<AppState>();
            let mut detector = Detector::new(state.catalog.clone(), Platform::CURRENT);
            loop {
                let events = detector.update(&list_processes());
                if !events.is_empty() {
                    for event in &events {
                        eprintln!("[overlay] {event:?}");
                    }
                    *state.running.lock().expect("running lock") =
                        detector.running().cloned().collect();
                    app.state::<UiProcesses>()
                        .broadcast(&CoreMessage::GamesChanged {
                            games: state.games_view(),
                        });
                    tray::refresh(&app);
                    overlay::sync(&app);
                }
                thread::sleep(INTERVAL);
            }
        });
    if let Err(err) = spawned {
        eprintln!("[overlay] cannot start game detection: {err}");
    }
}
