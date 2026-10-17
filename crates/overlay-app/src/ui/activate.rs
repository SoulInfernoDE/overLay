//! Brings this process's window to the front, also over a fullscreen game.
//!
//! Window managers prevent focus stealing: a window that asks for focus itself (as GTK
//! does) stays behind the active fullscreen game. On X11 the window is activated the way
//! a taskbar does it, with a `_NET_ACTIVE_WINDOW` request marked as coming from a pager,
//! which window managers carry out.

use std::thread;
use std::time::Duration;

/// Tries for a moment: a new window may not be mapped yet, and a request made while the
/// window manager still places it can be overruled by its focus-stealing rules.
const ATTEMPTS: u32 = 30;
const PAUSE: Duration = Duration::from_millis(100);

/// Activates the window in the background; failures are only logged.
pub fn raise_soon() {
    let spawned = thread::Builder::new()
        .name("activate-window".into())
        .spawn(|| {
            for _ in 0..ATTEMPTS {
                match activate() {
                    Ok(true) => return,
                    Ok(false) => thread::sleep(PAUSE),
                    Err(err) => {
                        eprintln!("[overlay] cannot bring the window to the front: {err}");
                        return;
                    }
                }
            }
            eprintln!("[overlay] the window manager kept the window in the background");
        });
    if let Err(err) = spawned {
        eprintln!("[overlay] cannot bring the window to the front: {err}");
    }
}

/// Whether a window of this process is the active window; requests that otherwise.
#[cfg(target_os = "linux")]
fn activate() -> Result<bool, Box<dyn std::error::Error>> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask};

    // Without an X server (e.g. a pure Wayland session) GTK's own request is all there is.
    let Ok((conn, screen)) = x11rb::connect(None) else {
        return Ok(true);
    };
    let root = conn.setup().roots[screen].root;
    let atom = |name: &[u8]| -> Result<u32, Box<dyn std::error::Error>> {
        Ok(conn.intern_atom(false, name)?.reply()?.atom)
    };
    let client_list = atom(b"_NET_CLIENT_LIST")?;
    let pid_atom = atom(b"_NET_WM_PID")?;
    let active = atom(b"_NET_ACTIVE_WINDOW")?;

    let clients = conn
        .get_property(false, root, client_list, AtomEnum::WINDOW, 0, u32::MAX)?
        .reply()?;
    let pid = std::process::id();
    let mut own = None;
    for window in clients.value32().into_iter().flatten() {
        let owner = conn
            .get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;
        if owner.value32().and_then(|mut v| v.next()) == Some(pid) {
            own = Some(window);
            break;
        }
    }
    let Some(window) = own else {
        return Ok(false);
    };
    let current = conn
        .get_property(false, root, active, AtomEnum::WINDOW, 0, 1)?
        .reply()?;
    if current.value32().and_then(|mut v| v.next()) == Some(window) {
        return Ok(true);
    }
    // Source 2 = pager; timestamp 0 = now.
    let request = ClientMessageEvent::new(32, window, active, [2, 0, 0, 0, 0]);
    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        request,
    )?;
    conn.flush()?;
    Ok(false)
}

#[cfg(not(target_os = "linux"))]
fn activate() -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}
