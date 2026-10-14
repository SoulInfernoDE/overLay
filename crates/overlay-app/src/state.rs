//! Shared application state, managed by Tauri.

use std::sync::{Mutex, MutexGuard, RwLock};

use overlay_games::{Catalog, RunningGame};
use overlay_i18n::{FluentArgs, Localizer};
use overlay_settings::{Settings, SettingsStore};

use crate::l10n;

pub struct AppState {
    store: Option<SettingsStore>,
    settings: Mutex<Settings>,
    l10n: RwLock<Localizer>,
    pub catalog: Catalog,
    pub running: Mutex<Vec<RunningGame>>,
}

impl AppState {
    pub fn new(store: Option<SettingsStore>, settings: Settings, catalog: Catalog) -> Self {
        let l10n = l10n::localizer(settings.language.as_deref());
        Self {
            store,
            settings: Mutex::new(settings),
            l10n: RwLock::new(l10n),
            catalog,
            running: Mutex::new(Vec::new()),
        }
    }

    pub fn settings(&self) -> MutexGuard<'_, Settings> {
        self.settings.lock().expect("settings lock")
    }

    pub fn settings_path(&self) -> Option<String> {
        self.store.as_ref().map(|s| s.path().display().to_string())
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

    pub fn t(&self, id: &str) -> String {
        self.l10n.read().expect("l10n lock").format(id, None)
    }

    pub fn t_args(&self, id: &str, args: &FluentArgs) -> String {
        self.l10n.read().expect("l10n lock").format(id, Some(args))
    }
}
