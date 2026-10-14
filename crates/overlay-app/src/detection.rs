//! Background thread that watches the process list and reports games starting and exiting.

use std::thread;
use std::time::Duration;

use overlay_games::{Detector, Platform, list_processes};
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::games_view;
use crate::state::AppState;
use crate::tray;

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
                    let _ = app.emit("games-changed", games_view(&state));
                    tray::refresh(&app);
                }
                thread::sleep(INTERVAL);
            }
        });
    if let Err(err) = spawned {
        eprintln!("[overlay] cannot start game detection: {err}");
    }
}
