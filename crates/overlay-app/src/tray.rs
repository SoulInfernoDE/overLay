//! Tray icon with a localized menu. On Linux the icon goes through AppIndicator, which
//! only offers the menu (no click events), so "open" is a menu entry as well.

use overlay_i18n::FluentArgs;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::state::AppState;
use crate::window;

const TRAY_ID: &str = "main";

pub struct TrayMenu {
    open: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let open = MenuItem::with_id(app, "open", state.t("tray-open"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", state.t("tray-quit"), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(state.t("tray-tooltip-idle"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => window::open(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::toggle(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    app.manage(TrayMenu { open, quit });
    Ok(())
}

/// Re-reads texts and tooltip after a language change or a game starting or exiting.
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Some(menu) = app.try_state::<TrayMenu>() {
        let _ = menu.open.set_text(state.t("tray-open"));
        let _ = menu.quit.set_text(state.t("tray-quit"));
    }
    let tooltip = match running_game_name(&state) {
        Some(game) => {
            let mut args = FluentArgs::new();
            args.set("game", game);
            state.t_args("tray-tooltip-running", &args)
        }
        None => state.t("tray-tooltip-idle"),
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn running_game_name(state: &AppState) -> Option<String> {
    let running = state.running.lock().expect("running lock");
    let first = running.first()?;
    state
        .catalog
        .games()
        .iter()
        .find(|g| g.id == first.game_id)
        .map(|g| g.name.clone())
}
