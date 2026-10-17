//! Clickable regions and window behaviour of the overlay. Everything outside the widgets
//! passes through to the game; the window never takes focus.
//!
//! Linux (X11): a GTK input shape. Windows: no input shapes exist, so a thread watches
//! the cursor and turns click-through off only while it is over a widget.

use std::sync::{Arc, Mutex};

use overlay_settings::Stacking;
use serde::Deserialize;
use tauri::{State, WebviewWindow};

/// A rectangle in CSS pixels of the overlay page.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// A rectangle in device pixels, relative to the overlay window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl DeviceRect {
    /// CSS pixels → device pixels, rounded outwards so no widget edge falls through.
    fn from_css(rect: &Rect, factor: f64) -> Self {
        Self {
            left: (rect.x * factor).floor() as i32,
            top: (rect.y * factor).floor() as i32,
            right: ((rect.x + rect.width) * factor).ceil() as i32,
            bottom: ((rect.y + rect.height) * factor).ceil() as i32,
        }
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

/// The current clickable regions in device pixels.
#[derive(Default)]
pub struct Regions(Arc<Mutex<Vec<DeviceRect>>>);

/// Called by the page whenever widgets move, appear or disappear. `device_pixel_ratio`
/// is the page's `window.devicePixelRatio` (e.g. 1.2 at 115 dpi).
#[tauri::command]
pub fn set_interactive_regions(
    window: WebviewWindow,
    regions: State<'_, Regions>,
    rects: Vec<Rect>,
    device_pixel_ratio: f64,
) -> Result<(), String> {
    let device: Vec<DeviceRect> = rects
        .iter()
        .map(|r| DeviceRect::from_css(r, device_pixel_ratio))
        .collect();
    // `OVERLAY_DEBUG=1` logs the clickable areas (for bug reports about lost clicks).
    if std::env::var_os("OVERLAY_DEBUG").is_some() {
        eprintln!("[overlay-ui] clickable areas: {device:?}");
    }
    *regions.0.lock().expect("regions lock") = device.clone();
    apply_input_shape(&window, device)
}

#[cfg(target_os = "linux")]
fn apply_input_shape(window: &WebviewWindow, rects: Vec<DeviceRect>) -> Result<(), String> {
    let target = window.clone();
    window
        .run_on_main_thread(move || {
            use gtk::cairo::{RectangleInt, Region};
            use gtk::prelude::*;

            let Ok(gtk_window) = target.gtk_window() else {
                return;
            };
            // Device pixels → GDK units (GDK's scale is an integer, usually 1).
            let scale = gtk_window.scale_factor().max(1);
            let rects: Vec<RectangleInt> = rects
                .iter()
                .map(|r| {
                    RectangleInt::new(
                        r.left / scale,
                        r.top / scale,
                        (r.right - r.left + scale - 1) / scale,
                        (r.bottom - r.top + scale - 1) / scale,
                    )
                })
                .collect();
            gtk_window.input_shape_combine_region(Some(&Region::create_rectangles(&rects)));
        })
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
fn apply_input_shape(_window: &WebviewWindow, _rects: Vec<DeviceRect>) -> Result<(), String> {
    // Windows reads `Regions` from its cursor thread.
    Ok(())
}

/// Must run before the window is shown for the first time.
#[cfg(target_os = "linux")]
pub fn prepare_window(window: &WebviewWindow, stacking: Stacking) -> tauri::Result<()> {
    use gtk::prelude::*;

    let gtk_window = window.gtk_window()?;
    gtk_window.set_accept_focus(false);
    gtk_window.set_focus_on_map(false);
    gtk_window.realize();
    if stacking == Stacking::Override
        && let Some(gdk_window) = gtk_window.window()
    {
        gdk_window.set_override_redirect(true);
    }
    // Click-through until the page reports its widgets.
    let _ = apply_input_shape(window, Vec::new());
    Ok(())
}

#[cfg(windows)]
pub fn prepare_window(window: &WebviewWindow, _stacking: Stacking) -> tauri::Result<()> {
    use tauri::Manager;

    window.set_ignore_cursor_events(true)?;
    let regions = Arc::clone(&window.app_handle().state::<Regions>().0);
    watch_cursor(window.clone(), regions);
    Ok(())
}

#[cfg(not(any(target_os = "linux", windows)))]
pub fn prepare_window(_window: &WebviewWindow, _stacking: Stacking) -> tauri::Result<()> {
    Ok(())
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn watch_cursor(window: WebviewWindow, regions: Arc<Mutex<Vec<DeviceRect>>>) {
    use std::time::Duration;
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let spawned = std::thread::Builder::new()
        .name("overlay-cursor".into())
        .spawn(move || {
            let mut passthrough = true;
            loop {
                std::thread::sleep(Duration::from_millis(30));
                let Ok(origin) = window.inner_position() else {
                    continue;
                };
                let mut cursor = POINT { x: 0, y: 0 };
                // SAFETY: GetCursorPos only writes the POINT we pass.
                if unsafe { GetCursorPos(&mut cursor) } == 0 {
                    continue;
                }
                let (x, y) = (cursor.x - origin.x, cursor.y - origin.y);
                let over_widget = regions
                    .lock()
                    .expect("regions lock")
                    .iter()
                    .any(|r| r.contains(x, y));
                if over_widget == passthrough {
                    passthrough = !over_widget;
                    let _ = window.set_ignore_cursor_events(passthrough);
                }
            }
        });
    if let Err(err) = spawned {
        eprintln!("[overlay-ui] cannot watch the cursor: {err}");
    }
}

/// Keeps an override-redirect overlay above the game (the window manager does not).
#[cfg(target_os = "linux")]
pub fn raise(window: &WebviewWindow) {
    use gtk::prelude::*;

    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Some(gdk_window) = target.gtk_window().ok().and_then(|w| w.window()) {
            gdk_window.raise();
        }
    });
}

#[cfg(not(target_os = "linux"))]
pub fn raise(_window: &WebviewWindow) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_rects_scale_outwards() {
        let rect = Rect {
            x: 1351.0,
            y: 107.5,
            width: 232.4,
            height: 438.0,
        };
        let device = DeviceRect::from_css(&rect, 1.2);
        assert_eq!(
            device,
            DeviceRect {
                left: 1621,
                top: 129,
                right: 1901,
                bottom: 655
            }
        );
        assert!(device.contains(1621, 129));
        assert!(!device.contains(1901, 300));
    }
}
