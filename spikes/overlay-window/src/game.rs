//! Finds the Hearthstone process and its X11 window, and reads the window bounds.
//!
//! Works for the native Linux client (`Hearthstone.x86_64`) and for Wine/Proton
//! (`Hearthstone.exe`): both are X11 (or XWayland) clients that set `_NET_WM_PID`.

use std::error::Error;
use std::fs;

use serde::Serialize;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

/// `/proc/<pid>/comm` is cut to 15 bytes, so `Hearthstone.x86_64` shows up as `Hearthstone.x86`.
const PROCESS_NAMES: [(&str, &str); 2] =
    [("Hearthstone.x86", "native"), ("Hearthstone.exe", "wine")];

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameProcess {
    pub pid: u32,
    pub flavor: &'static str,
}

pub struct X11Tracker {
    conn: RustConnection,
    root: Window,
    net_client_list: u32,
    net_wm_pid: u32,
    net_wm_state: u32,
    net_wm_state_hidden: u32,
}

impl X11Tracker {
    pub fn connect() -> Result<Self> {
        let (conn, screen) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen].root;
        let atom =
            |name: &[u8]| -> Result<u32> { Ok(conn.intern_atom(false, name)?.reply()?.atom) };
        Ok(Self {
            net_client_list: atom(b"_NET_CLIENT_LIST")?,
            net_wm_pid: atom(b"_NET_WM_PID")?,
            net_wm_state: atom(b"_NET_WM_STATE")?,
            net_wm_state_hidden: atom(b"_NET_WM_STATE_HIDDEN")?,
            root,
            conn,
        })
    }

    pub fn root_bounds(&self) -> Result<Bounds> {
        self.bounds(self.root)
    }

    /// The largest managed window owned by `pid` (skips splash and loading windows).
    pub fn window_for_pid(&self, pid: u32) -> Result<Option<Window>> {
        let clients = self
            .conn
            .get_property(
                false,
                self.root,
                self.net_client_list,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?;
        let mut best: Option<(Window, u64)> = None;
        for window in clients.value32().into_iter().flatten() {
            let owner = self
                .conn
                .get_property(false, window, self.net_wm_pid, AtomEnum::CARDINAL, 0, 1)?
                .reply()?;
            if owner.value32().and_then(|mut v| v.next()) != Some(pid) {
                continue;
            }
            let b = self.bounds(window)?;
            let area = u64::from(b.width) * u64::from(b.height);
            if best.is_none_or(|(_, a)| area > a) {
                best = Some((window, area));
            }
        }
        Ok(best.map(|(w, _)| w))
    }

    /// Client-area bounds in root (screen) coordinates.
    pub fn bounds(&self, window: Window) -> Result<Bounds> {
        let geometry = self.conn.get_geometry(window)?.reply()?;
        let origin = self
            .conn
            .translate_coordinates(window, self.root, 0, 0)?
            .reply()?;
        Ok(Bounds {
            x: origin.dst_x.into(),
            y: origin.dst_y.into(),
            width: geometry.width.into(),
            height: geometry.height.into(),
        })
    }

    pub fn is_hidden(&self, window: Window) -> Result<bool> {
        let state = self
            .conn
            .get_property(false, window, self.net_wm_state, AtomEnum::ATOM, 0, 64)?
            .reply()?;
        Ok(state
            .value32()
            .is_some_and(|mut atoms| atoms.any(|a| a == self.net_wm_state_hidden)))
    }
}

/// Scans `/proc` for a running Hearthstone process.
pub fn find_process() -> Option<GameProcess> {
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(comm) = fs::read_to_string(entry.path().join("comm")) else {
            continue;
        };
        if let Some((_, flavor)) = PROCESS_NAMES
            .iter()
            .find(|(name, _)| comm.trim_end() == *name)
        {
            return Some(GameProcess { pid, flavor });
        }
    }
    None
}

pub fn is_alive(pid: u32) -> bool {
    fs::metadata(format!("/proc/{pid}")).is_ok()
}
