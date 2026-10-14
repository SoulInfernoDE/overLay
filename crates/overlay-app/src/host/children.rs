//! UI processes (launcher, overlay) started and supervised by the core.
//!
//! Each runs this same executable with `--ui <role>`. The core talks to it over the
//! child's stdin and stdout (JSON lines) and answers its requests. When the window
//! closes, the process exits and its memory, including WebKit's, goes back to the system.

use std::collections::BTreeMap;
use std::io::{self, BufReader};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use overlay_ipc::{CoreMessage, UI_FLAG, UiMessage, UiRole, read_messages, write_message};
use tauri::AppHandle;

use crate::host::{overlay, requests};

struct Running {
    generation: u64,
    stdin: Arc<Mutex<ChildStdin>>,
}

#[derive(Default)]
pub struct UiProcesses {
    running: Mutex<BTreeMap<UiRoleKey, Running>>,
    generation: AtomicU64,
}

/// `UiRole` as a map key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UiRoleKey(u8);

impl From<UiRole> for UiRoleKey {
    fn from(role: UiRole) -> Self {
        Self(role as u8)
    }
}

impl UiProcesses {
    pub fn is_running(&self, role: UiRole) -> bool {
        self.running
            .lock()
            .expect("children lock")
            .contains_key(&role.into())
    }

    /// Starts the UI process for `role` unless it already runs, then sends `initial`.
    pub fn start(&self, app: &AppHandle, role: UiRole, initial: &[CoreMessage]) -> io::Result<()> {
        let mut running = self.running.lock().expect("children lock");
        if running.contains_key(&role.into()) {
            return Ok(());
        }
        let mut command = Command::new(std::env::current_exe()?);
        command
            .args([UI_FLAG, role.as_arg()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if role == UiRole::Overlay && cfg!(target_os = "linux") {
            // Games under Proton and the native Hearthstone client are X11/XWayland
            // clients; the overlay must be one too to stack and position itself over them.
            command.env("GDK_BACKEND", "x11");
            // Measured in the spike: about 25 MB less, with identical rendering of the
            // overlay's flat, transparent widgets.
            command.env("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
        let mut child = command.spawn()?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("piped stdin")));
        let stdout = child.stdout.take().expect("piped stdout");
        for message in initial {
            write_message(&mut *stdin.lock().expect("stdin lock"), message)?;
        }
        let generation = self.generation.fetch_add(1, Ordering::Relaxed);
        running.insert(
            role.into(),
            Running {
                generation,
                stdin: Arc::clone(&stdin),
            },
        );
        drop(running);

        let app = app.clone();
        thread::Builder::new()
            .name(format!("ui-{}", role.as_arg()))
            .spawn(move || {
                for message in read_messages::<_, UiMessage>(BufReader::new(stdout)) {
                    match message {
                        Ok(UiMessage::Request { id, request }) => {
                            let result = requests::handle(&app, role, request);
                            let response = CoreMessage::Response { id, result };
                            let mut stdin = stdin.lock().expect("stdin lock");
                            if write_message(&mut *stdin, &response).is_err() {
                                break;
                            }
                        }
                        Err(err) => eprintln!("[overlay] {} process: {err}", role.as_arg()),
                    }
                }
                let status = child.wait();
                eprintln!("[overlay] {} process ended: {status:?}", role.as_arg());
                let processes = tauri::Manager::state::<UiProcesses>(&app);
                let mut running = processes.running.lock().expect("children lock");
                if running
                    .get(&role.into())
                    .is_some_and(|r| r.generation == generation)
                {
                    running.remove(&role.into());
                }
                drop(running);
                if role == UiRole::Overlay {
                    overlay::sync(&app);
                }
            })?;
        Ok(())
    }

    /// Sends `message` to the process for `role`; returns whether it runs.
    pub fn send(&self, role: UiRole, message: &CoreMessage) -> bool {
        let stdin = match self
            .running
            .lock()
            .expect("children lock")
            .get(&role.into())
        {
            Some(running) => Arc::clone(&running.stdin),
            None => return false,
        };
        let mut stdin = stdin.lock().expect("stdin lock");
        write_message(&mut *stdin, message).is_ok()
    }

    pub fn broadcast(&self, message: &CoreMessage) {
        for role in [UiRole::Launcher, UiRole::Overlay] {
            self.send(role, message);
        }
    }
}
