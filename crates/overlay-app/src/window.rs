//! The launcher window exists only while it is shown. Closing it destroys the webview,
//! so a minimised overLay costs no WebKit processes.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const MAIN: &str = "main";

pub fn open(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("overLay")
        .inner_size(1100.0, 720.0)
        .min_inner_size(820.0, 560.0)
        .decorations(false)
        .center()
        .build();
    if let Err(err) = built {
        eprintln!("[overlay] cannot open the launcher window: {err}");
    }
}

pub fn toggle(app: &AppHandle) {
    match app.get_webview_window(MAIN) {
        Some(window) => {
            let _ = window.destroy();
        }
        None => open(app),
    }
}
