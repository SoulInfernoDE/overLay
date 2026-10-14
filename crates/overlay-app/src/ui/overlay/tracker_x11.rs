//! Follows the game window on X11/XWayland through events instead of polling: the
//! thread blocks until the X server reports a move, resize, state or focus change.

use std::error::Error;
use std::thread;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, MapState, Window,
};
use x11rb::rust_connection::RustConnection;

use super::GameWindow;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

struct Atoms {
    client_list: u32,
    wm_pid: u32,
    wm_state: u32,
    wm_state_hidden: u32,
    active_window: u32,
}

impl Atoms {
    fn new(conn: &RustConnection) -> Result<Self> {
        let atom =
            |name: &[u8]| -> Result<u32> { Ok(conn.intern_atom(false, name)?.reply()?.atom) };
        Ok(Self {
            client_list: atom(b"_NET_CLIENT_LIST")?,
            wm_pid: atom(b"_NET_WM_PID")?,
            wm_state: atom(b"_NET_WM_STATE")?,
            wm_state_hidden: atom(b"_NET_WM_STATE_HIDDEN")?,
            active_window: atom(b"_NET_ACTIVE_WINDOW")?,
        })
    }
}

/// Calls `on_change` with the game window's state whenever it changes, and with `None`
/// while the game has no window.
pub fn spawn(pid: u32, on_change: impl Fn(Option<GameWindow>) + Send + 'static) {
    let spawned = thread::Builder::new()
        .name("game-window".into())
        .spawn(move || {
            if let Err(err) = track(pid, &on_change) {
                eprintln!("[overlay-ui] game window tracking stopped: {err}");
            }
        });
    if let Err(err) = spawned {
        eprintln!("[overlay-ui] cannot track the game window: {err}");
    }
}

fn track(pid: u32, on_change: &dyn Fn(Option<GameWindow>)) -> Result<()> {
    let (conn, screen) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen].root;
    let atoms = Atoms::new(&conn)?;
    // Root properties tell us about new windows (_NET_CLIENT_LIST) and focus changes.
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )?;
    conn.flush()?;

    let mut window: Option<Window> = None;
    let mut last: Option<Option<GameWindow>> = None;
    let mut watched_active: Option<Window> = None;
    loop {
        // Moving the active window over or away from the game changes the overlay's
        // visibility, so its moves are watched too.
        let active = active_window(&conn, root, &atoms);
        if active != watched_active {
            if let Some(a) = active.filter(|a| Some(*a) != window && *a != root) {
                let _ = top_level(&conn, root, a).and_then(|frame| {
                    conn.change_window_attributes(
                        frame,
                        &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
                    )?;
                    conn.flush()?;
                    Ok(())
                });
            }
            watched_active = active;
        }
        if window.is_none() {
            window = find_window(&conn, root, &atoms, pid)?;
            if let Some(w) = window {
                watch(&conn, root, w)?;
            }
        }
        let current = match window {
            Some(w) => match read_state(&conn, root, &atoms, w, pid) {
                Ok(state) => Some(state),
                Err(_) => {
                    // The window went away between events.
                    window = None;
                    None
                }
            },
            None => None,
        };
        if last != Some(current) {
            on_change(current);
            last = Some(current);
        }

        // Block until something happens, then drain the queue and re-read once.
        let mut event = Some(conn.wait_for_event()?);
        while let Some(e) = event {
            if let Event::DestroyNotify(destroyed) = &e
                && Some(destroyed.window) == window
            {
                window = None;
            }
            event = conn.poll_for_event()?;
        }
    }
}

/// The largest managed window that belongs to `pid` (skips splash and loading windows).
fn find_window(
    conn: &RustConnection,
    root: Window,
    atoms: &Atoms,
    pid: u32,
) -> Result<Option<Window>> {
    let clients = conn
        .get_property(
            false,
            root,
            atoms.client_list,
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        )?
        .reply()?;
    let mut best: Option<(Window, u64)> = None;
    for w in clients.value32().into_iter().flatten() {
        if window_pid(conn, atoms, w) != Some(pid) {
            continue;
        }
        let Ok(geometry) = conn.get_geometry(w)?.reply() else {
            continue;
        };
        let area = u64::from(geometry.width) * u64::from(geometry.height);
        if best.is_none_or(|(_, a)| area > a) {
            best = Some((w, area));
        }
    }
    Ok(best.map(|(w, _)| w))
}

/// Subscribes to changes of the game window and of its window-manager frame, because a
/// frame move does not always reach the client window as a ConfigureNotify.
fn watch(conn: &RustConnection, root: Window, window: Window) -> Result<()> {
    let client_events = EventMask::STRUCTURE_NOTIFY | EventMask::PROPERTY_CHANGE;
    conn.change_window_attributes(
        window,
        &ChangeWindowAttributesAux::new().event_mask(client_events),
    )?;
    let frame = top_level(conn, root, window)?;
    if frame != window {
        conn.change_window_attributes(
            frame,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
        )?;
    }
    conn.flush()?;
    Ok(())
}

fn top_level(conn: &RustConnection, root: Window, mut window: Window) -> Result<Window> {
    loop {
        let tree = conn.query_tree(window)?.reply()?;
        if tree.parent == root || tree.parent == x11rb::NONE {
            return Ok(window);
        }
        window = tree.parent;
    }
}

fn read_state(
    conn: &RustConnection,
    root: Window,
    atoms: &Atoms,
    window: Window,
    pid: u32,
) -> Result<GameWindow> {
    let geometry = conn.get_geometry(window)?.reply()?;
    let origin = conn.translate_coordinates(window, root, 0, 0)?.reply()?;
    let mapped = conn.get_window_attributes(window)?.reply()?.map_state == MapState::VIEWABLE;
    let state = conn
        .get_property(false, window, atoms.wm_state, AtomEnum::ATOM, 0, 64)?
        .reply()?;
    let minimized = state
        .value32()
        .is_some_and(|mut atoms_set| atoms_set.any(|a| a == atoms.wm_state_hidden));
    let active_window = conn
        .get_property(false, root, atoms.active_window, AtomEnum::WINDOW, 0, 1)?
        .reply()?
        .value32()
        .and_then(|mut v| v.next());
    // Clicking the overlay must not count as leaving the game, so our own windows count too.
    let active = active_window.is_some_and(|a| {
        a == window
            || matches!(window_pid(conn, atoms, a), Some(p) if p == pid || p == std::process::id())
    });
    let game = Area {
        x: origin.dst_x.into(),
        y: origin.dst_y.into(),
        width: geometry.width.into(),
        height: geometry.height.into(),
    };
    // With several monitors, another window can be active without hiding the game.
    let covered = !active
        && active_window
            .filter(|a| *a != root && *a != x11rb::NONE)
            .and_then(|a| area_of(conn, root, a).ok())
            .is_some_and(|other| other.overlaps(&game));
    Ok(GameWindow {
        x: game.x,
        y: game.y,
        width: game.width,
        height: game.height,
        hidden: !mapped || minimized,
        active,
        covered,
    })
}

/// A rectangle in root (screen) coordinates.
struct Area {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Area {
    fn overlaps(&self, other: &Area) -> bool {
        let right = |a: &Area| a.x + a.width as i32;
        let bottom = |a: &Area| a.y + a.height as i32;
        self.x < right(other)
            && other.x < right(self)
            && self.y < bottom(other)
            && other.y < bottom(self)
    }
}

fn area_of(conn: &RustConnection, root: Window, window: Window) -> Result<Area> {
    let geometry = conn.get_geometry(window)?.reply()?;
    let origin = conn.translate_coordinates(window, root, 0, 0)?.reply()?;
    Ok(Area {
        x: origin.dst_x.into(),
        y: origin.dst_y.into(),
        width: geometry.width.into(),
        height: geometry.height.into(),
    })
}

fn active_window(conn: &RustConnection, root: Window, atoms: &Atoms) -> Option<Window> {
    conn.get_property(false, root, atoms.active_window, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?
        .value32()
        .and_then(|mut v| v.next())
}

fn window_pid(conn: &RustConnection, atoms: &Atoms, window: Window) -> Option<u32> {
    conn.get_property(false, window, atoms.wm_pid, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?
        .value32()
        .and_then(|mut v| v.next())
}
