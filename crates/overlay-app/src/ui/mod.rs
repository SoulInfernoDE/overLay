//! UI processes: the launcher and the in-game overlay. Each is a small Tauri app with a
//! single window that talks to the core over stdin/stdout (see `overlay-ipc`).
//! Nothing in a UI process may print to stdout; logs go to stderr.

mod activate;
mod launcher;
mod mods;
mod overlay;

use std::collections::HashMap;
use std::io::{self, BufReader};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use overlay_ipc::{CoreMessage, Request, UiMessage, UiRole, read_messages, write_message};
use tauri::{AppHandle, State};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub fn run(role: UiRole) {
    match role {
        UiRole::Launcher => launcher::run(),
        UiRole::Overlay => overlay::run(),
    }
}

/// Sends requests to the core and matches up its responses.
#[derive(Default)]
pub struct CoreClient {
    next_id: AtomicU64,
    pending: Mutex<HashMap<u64, Sender<Result<serde_json::Value, String>>>>,
    stdout: Mutex<()>,
}

impl CoreClient {
    pub fn request(&self, request: Request) -> Result<serde_json::Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel();
        self.pending
            .lock()
            .expect("pending lock")
            .insert(id, sender);
        let sent = {
            let _guard = self.stdout.lock().expect("stdout lock");
            write_message(
                &mut io::stdout().lock(),
                &UiMessage::Request { id, request },
            )
        };
        if let Err(err) = sent {
            self.pending.lock().expect("pending lock").remove(&id);
            return Err(format!("cannot reach the overLay core: {err}"));
        }
        receiver
            .recv_timeout(REQUEST_TIMEOUT)
            .map_err(|_| "the overLay core did not answer".to_owned())?
    }

    fn deliver(&self, id: u64, result: Result<serde_json::Value, String>) {
        if let Some(sender) = self.pending.lock().expect("pending lock").remove(&id) {
            let _ = sender.send(result);
        }
    }
}

/// Reads core messages from stdin: responses go to their request, everything else to
/// `on_message`. When the core goes away (stdin closes), the UI process exits.
pub fn listen_to_core(
    app: AppHandle,
    client: Arc<CoreClient>,
    on_message: impl Fn(&AppHandle, CoreMessage) + Send + 'static,
) {
    let spawned = thread::Builder::new()
        .name("core-messages".into())
        .spawn(move || {
            for message in read_messages::<_, CoreMessage>(BufReader::new(io::stdin())) {
                match message {
                    Ok(CoreMessage::Response { id, result }) => client.deliver(id, result),
                    Ok(message) => on_message(&app, message),
                    Err(err) => eprintln!("[overlay-ui] {err}"),
                }
            }
            app.exit(0);
        });
    if let Err(err) = spawned {
        eprintln!("[overlay-ui] cannot read core messages: {err}");
    }
}

/// The page's single entry point to the core.
#[tauri::command]
pub(crate) async fn core_request(
    client: State<'_, Arc<CoreClient>>,
    request: Request,
) -> Result<serde_json::Value, String> {
    let client = Arc::clone(&client);
    tauri::async_runtime::spawn_blocking(move || client.request(request))
        .await
        .map_err(|e| e.to_string())?
}
