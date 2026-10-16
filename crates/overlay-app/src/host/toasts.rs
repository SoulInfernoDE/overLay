//! Toasts for the overlay wait here until the overlay page takes them, so none is lost
//! while the overlay process starts or its page is still loading.

use std::sync::Mutex;

use overlay_ipc::{CoreMessage, Toast, UiRole};
use tauri::{AppHandle, Manager};

use crate::host::children::UiProcesses;

/// Older toasts are dropped beyond this many.
const LIMIT: usize = 20;

#[derive(Default)]
pub struct Toasts(Mutex<Vec<Toast>>);

impl Toasts {
    pub fn take(&self) -> Vec<Toast> {
        std::mem::take(&mut *self.0.lock().expect("toasts lock"))
    }
}

pub fn push(app: &AppHandle, toast: Toast) {
    {
        let toasts = app.state::<Toasts>();
        let mut queue = toasts.0.lock().expect("toasts lock");
        queue.push(toast);
        let excess = queue.len().saturating_sub(LIMIT);
        queue.drain(..excess);
    }
    app.state::<UiProcesses>()
        .send(UiRole::Overlay, &CoreMessage::ToastsPending);
}
