//! State of the core process, managed by Tauri.

use std::sync::{Mutex, MutexGuard, RwLock};

use overlay_games::{Catalog, RunningGame};
use overlay_i18n::{FluentArgs, Localizer};
use overlay_ipc::{GameView, Snapshot, WidgetPositions};
use overlay_settings::{Layout, LayoutStore, Position, Settings, SettingsStore};

use crate::l10n;

pub struct AppState {
    store: Option<SettingsStore>,
    settings: Mutex<Settings>,
    layout_store: Option<LayoutStore>,
    layout: Mutex<Layout>,
    l10n: RwLock<Localizer>,
    pub catalog: Catalog,
    pub running: Mutex<Vec<RunningGame>>,
}

impl AppState {
    pub fn new(
        store: Option<SettingsStore>,
        settings: Settings,
        layout_store: Option<LayoutStore>,
        layout: Layout,
        catalog: Catalog,
    ) -> Self {
        let l10n = l10n::localizer(settings.language.as_deref());
        Self {
            store,
            settings: Mutex::new(settings),
            layout_store,
            layout: Mutex::new(layout),
            l10n: RwLock::new(l10n),
            catalog,
            running: Mutex::new(Vec::new()),
        }
    }

    pub fn settings(&self) -> MutexGuard<'_, Settings> {
        self.settings.lock().expect("settings lock")
    }

    /// Applies `change`, saves the result and rebuilds the localizer if the language changed.
    pub fn update_settings(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings, String> {
        let mut settings = self.settings();
        let mut next = settings.clone();
        change(&mut next);
        if let Some(store) = &self.store {
            store.save(&next).map_err(|e| e.to_string())?;
        }
        if next.language != settings.language {
            *self.l10n.write().expect("l10n lock") = l10n::localizer(next.language.as_deref());
        }
        *settings = next.clone();
        Ok(next)
    }

    pub fn widget_positions(&self, game: &str, resolution: &str) -> WidgetPositions {
        self.layout
            .lock()
            .expect("layout lock")
            .widgets(game, resolution)
    }

    pub fn save_widget_position(
        &self,
        game: &str,
        resolution: &str,
        widget: &str,
        position: Position,
    ) -> Result<(), String> {
        let mut layout = self.layout.lock().expect("layout lock");
        layout.set(game, resolution, widget, position);
        match &self.layout_store {
            Some(store) => store.save(&layout).map_err(|e| e.to_string()),
            None => Ok(()),
        }
    }

    pub fn games_view(&self) -> Vec<GameView> {
        let running = self.running.lock().expect("running lock");
        self.catalog
            .games()
            .iter()
            .map(|game| GameView {
                id: game.id.clone(),
                name: game.name.clone(),
                running: running
                    .iter()
                    .find(|r| r.game_id == game.id)
                    .map(|r| r.flavor),
            })
            .collect()
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            settings: self.settings().clone(),
            settings_path: self.store.as_ref().map(|s| s.path().display().to_string()),
            games: self.games_view(),
        }
    }

    pub fn game_name(&self, game_id: &str) -> Option<String> {
        self.catalog
            .games()
            .iter()
            .find(|g| g.id == game_id)
            .map(|g| g.name.clone())
    }

    pub fn t(&self, id: &str) -> String {
        self.l10n.read().expect("l10n lock").format(id, None)
    }

    pub fn t_args(&self, id: &str, args: &FluentArgs) -> String {
        self.l10n.read().expect("l10n lock").format(id, Some(args))
    }
}
