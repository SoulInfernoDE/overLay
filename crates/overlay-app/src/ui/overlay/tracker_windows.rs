//! Follows the game window on Windows by polling every 50 ms (a WinEvent hook would
//! avoid the polling; planned once this can be tested on Windows).

use std::thread;
use std::time::Duration;

use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClientRect, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible,
};

use super::GameWindow;

const INTERVAL: Duration = Duration::from_millis(50);

pub fn spawn(pid: u32, on_change: impl Fn(Option<GameWindow>) + Send + 'static) {
    let spawned = thread::Builder::new()
        .name("game-window".into())
        .spawn(move || {
            let mut last: Option<Option<GameWindow>> = None;
            loop {
                let current = find_window(pid).map(|hwnd| read_state(hwnd, pid));
                if last != Some(current) {
                    on_change(current);
                    last = Some(current);
                }
                thread::sleep(INTERVAL);
            }
        });
    if let Err(err) = spawned {
        eprintln!("[overlay-ui] cannot track the game window: {err}");
    }
}

struct Search {
    pid: u32,
    best: Option<(HWND, i64)>,
}

#[allow(unsafe_code)]
fn find_window(pid: u32) -> Option<HWND> {
    unsafe extern "system" fn visit(hwnd: HWND, lparam: LPARAM) -> i32 {
        // SAFETY: `lparam` is the `Search` passed to EnumWindows below, alive for the call.
        let search = unsafe { &mut *(lparam as *mut Search) };
        let mut owner = 0u32;
        // SAFETY: plain Win32 queries on a window handle EnumWindows just gave us.
        unsafe {
            if IsWindowVisible(hwnd) == 0 {
                return 1;
            }
            GetWindowThreadProcessId(hwnd, &mut owner);
            if owner != search.pid {
                return 1;
            }
            let mut rect: RECT = std::mem::zeroed();
            if GetClientRect(hwnd, &mut rect) == 0 {
                return 1;
            }
            let area = i64::from(rect.right - rect.left) * i64::from(rect.bottom - rect.top);
            if search.best.is_none_or(|(_, best)| area > best) {
                search.best = Some((hwnd, area));
            }
        }
        1
    }
    let mut search = Search { pid, best: None };
    // SAFETY: the callback only runs during this call, while `search` is alive.
    unsafe { EnumWindows(Some(visit), &mut search as *mut Search as LPARAM) };
    search.best.map(|(hwnd, _)| hwnd)
}

#[allow(unsafe_code)]
fn read_state(hwnd: HWND, pid: u32) -> GameWindow {
    // SAFETY: Win32 queries on a valid window handle with properly initialised out-params.
    unsafe {
        let mut rect: RECT = std::mem::zeroed();
        GetClientRect(hwnd, &mut rect);
        let mut origin = POINT { x: 0, y: 0 };
        ClientToScreen(hwnd, &mut origin);
        let foreground = GetForegroundWindow();
        let mut foreground_pid = 0u32;
        GetWindowThreadProcessId(foreground, &mut foreground_pid);
        let width = (rect.right - rect.left).max(0);
        let height = (rect.bottom - rect.top).max(0);
        let active =
            foreground == hwnd || foreground_pid == pid || foreground_pid == std::process::id();
        // With several monitors, another window can be active without hiding the game.
        let mut other: RECT = std::mem::zeroed();
        let covered = !active
            && !foreground.is_null()
            && GetWindowRect(foreground, &mut other) != 0
            && other.left < origin.x + width
            && origin.x < other.right
            && other.top < origin.y + height
            && origin.y < other.bottom;
        GameWindow {
            x: origin.x,
            y: origin.y,
            width: width as u32,
            height: height as u32,
            hidden: IsIconic(hwnd) != 0,
            active,
            covered,
        }
    }
}
