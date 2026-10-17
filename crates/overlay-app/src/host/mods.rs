//! Mods in the core: loading, switching on and off, setting values, storage and the
//! calls mods make through the SDK. Every call is checked against the mod's permissions.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

use overlay_i18n::Localizer;
use overlay_ipc::{CoreMessage, ModCall, ModList, ModView};
use overlay_mods::{ModInfo, Permission, SettingKind, discover};
use overlay_settings::{SettingValue, Settings};
use tauri::{AppHandle, Manager};

use crate::host::children::UiProcesses;
use crate::host::hotkeys;
use crate::host::state::AppState;
use crate::l10n;

/// Upper limit for one mod's storage file.
const STORAGE_LIMIT: usize = 1024 * 1024;
const KEY_LIMIT: usize = 200;
const TOAST_LIMIT: usize = 300;

pub struct ModRegistry {
    mods: RwLock<Vec<ModInfo>>,
    errors: RwLock<Vec<String>>,
    /// `<data dir>/mods`: mods installed from repositories (phase 4).
    installed_root: Option<PathBuf>,
    /// `<data dir>/mod-data/<id>`: storage of each mod.
    data_root: Option<PathBuf>,
    storage_lock: Mutex<()>,
}

impl ModRegistry {
    pub fn new() -> Self {
        let dirs = directories::ProjectDirs::from("io.github", "overLay", "overLay");
        Self {
            mods: RwLock::new(Vec::new()),
            errors: RwLock::new(Vec::new()),
            installed_root: dirs.as_ref().map(|d| d.data_dir().join("mods")),
            data_root: dirs.as_ref().map(|d| d.data_dir().join("mod-data")),
            storage_lock: Mutex::new(()),
        }
    }

    /// Reads all mod folders again: installed ones, plus developer folders if enabled.
    pub fn reload(&self, settings: &Settings) {
        let developer: &[PathBuf] = if settings.developer.enabled {
            &settings.developer.mod_paths
        } else {
            &[]
        };
        let (mods, errors) = discover(self.installed_root.as_deref(), developer);
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        for error in &errors {
            eprintln!("[overlay] mod not loaded: {error}");
        }
        *self.mods.write().expect("mods lock") = mods;
        *self.errors.write().expect("errors lock") = errors;
    }

    pub fn get(&self, id: &str) -> Option<ModInfo> {
        self.mods
            .read()
            .expect("mods lock")
            .iter()
            .find(|m| m.manifest.id == id)
            .cloned()
    }

    pub fn enabled_mods(&self, settings: &Settings) -> Vec<ModInfo> {
        self.mods
            .read()
            .expect("mods lock")
            .iter()
            .filter(|m| settings.mod_enabled(&m.manifest.id))
            .cloned()
            .collect()
    }

    pub fn list(&self, settings: &Settings) -> ModList {
        let requested = l10n::requested_languages(settings.language.as_deref());
        let mods = self
            .mods
            .read()
            .expect("mods lock")
            .iter()
            .map(|info| ModView {
                description: describe(info, &requested),
                values: effective_values(info, settings),
                enabled: settings.mod_enabled(&info.manifest.id),
                origin: info.source,
                path: info.root.display().to_string(),
                manifest: info.manifest.clone(),
            })
            .collect();
        ModList {
            mods,
            errors: self.errors.read().expect("errors lock").clone(),
        }
    }

    fn storage_file(&self, id: &str) -> Result<PathBuf, String> {
        let root = self
            .data_root
            .as_ref()
            .ok_or("no data folder on this system")?;
        Ok(root.join(id).join("storage.json"))
    }

    fn read_storage(&self, id: &str) -> Result<BTreeMap<String, serde_json::Value>, String> {
        let path = self.storage_file(id)?;
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display())),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(err) => Err(format!("{}: {err}", path.display())),
        }
    }

    fn write_storage(
        &self,
        id: &str,
        data: &BTreeMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        let text = serde_json::to_string(data).map_err(|e| e.to_string())?;
        if text.len() > STORAGE_LIMIT {
            return Err(format!("storage limit of {STORAGE_LIMIT} bytes reached"));
        }
        let path = self.storage_file(id)?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, text).map_err(|e| e.to_string())?;
        fs::rename(&temp, &path).map_err(|e| e.to_string())
    }
}

/// The mod's description from its own catalogs (`locales/`), or nothing if it has none.
fn describe(info: &ModInfo, requested: &[unic_langid::LanguageIdentifier]) -> String {
    let Ok(localizer) = Localizer::load(&info.root.join("locales"), requested) else {
        return String::new();
    };
    let id = &info.manifest.description;
    let text = localizer.format(id, None);
    if &text == id { String::new() } else { text }
}

/// Stored values where they fit the declared type, defaults everywhere else.
pub fn effective_values(info: &ModInfo, settings: &Settings) -> BTreeMap<String, SettingValue> {
    let stored = settings.mods.get(&info.manifest.id).map(|m| &m.values);
    info.manifest
        .settings
        .iter()
        .map(|spec| {
            let value = stored
                .and_then(|values| values.get(&spec.id))
                .filter(|value| fits(&spec.kind, value))
                .cloned()
                .unwrap_or_else(|| default_value(&spec.kind));
            (spec.id.clone(), value)
        })
        .collect()
}

pub fn fits(kind: &SettingKind, value: &SettingValue) -> bool {
    match (kind, value) {
        (SettingKind::Boolean { .. }, SettingValue::Bool(_)) => true,
        (SettingKind::Number { min, max, .. }, SettingValue::Number(n)) => {
            n.is_finite() && min.is_none_or(|min| *n >= min) && max.is_none_or(|max| *n <= max)
        }
        (SettingKind::Text { .. }, SettingValue::Text(t)) => t.len() <= 1000,
        (SettingKind::Choice { options, .. }, SettingValue::Text(t)) => options.contains(t),
        _ => false,
    }
}

fn default_value(kind: &SettingKind) -> SettingValue {
    match kind {
        SettingKind::Boolean { default } => SettingValue::Bool(*default),
        SettingKind::Number { default, .. } => SettingValue::Number(*default),
        SettingKind::Text { default } | SettingKind::Choice { default, .. } => {
            SettingValue::Text(default.clone())
        }
    }
}

/// Tells every UI process about the current mod list and re-registers mod hotkeys.
pub fn mods_changed(app: &AppHandle) -> ModList {
    let state = app.state::<AppState>();
    let settings = state.settings().clone();
    let list = app.state::<ModRegistry>().list(&settings);
    app.state::<UiProcesses>()
        .broadcast(&CoreMessage::ModsChanged { list: list.clone() });
    hotkeys::register_all(app);
    list
}

/// Runs a call a mod made through the SDK.
pub fn call(app: &AppHandle, mod_id: &str, call: ModCall) -> Result<serde_json::Value, String> {
    let registry = app.state::<ModRegistry>();
    let state = app.state::<AppState>();
    let info = registry
        .get(mod_id)
        .ok_or_else(|| format!("unknown mod {mod_id}"))?;
    if !state.settings().mod_enabled(mod_id) {
        return Err(format!("mod {mod_id} is switched off"));
    }
    let require = |permission: Permission| {
        if info.manifest.has(permission) {
            Ok(())
        } else {
            Err(format!("mod {mod_id} lacks the {permission:?} permission"))
        }
    };
    match call {
        ModCall::StorageGet { key } => {
            require(Permission::Storage)?;
            let _guard = registry.storage_lock.lock().expect("storage lock");
            Ok(registry
                .read_storage(mod_id)?
                .remove(&key)
                .unwrap_or(serde_json::Value::Null))
        }
        ModCall::StorageSet { key, value } => {
            require(Permission::Storage)?;
            if key.is_empty() || key.len() > KEY_LIMIT {
                return Err(format!("storage keys need 1 to {KEY_LIMIT} characters"));
            }
            let _guard = registry.storage_lock.lock().expect("storage lock");
            let mut data = registry.read_storage(mod_id)?;
            data.insert(key, value);
            registry.write_storage(mod_id, &data)?;
            Ok(serde_json::Value::Null)
        }
        ModCall::StorageRemove { key } => {
            require(Permission::Storage)?;
            let _guard = registry.storage_lock.lock().expect("storage lock");
            let mut data = registry.read_storage(mod_id)?;
            if data.remove(&key).is_some() {
                registry.write_storage(mod_id, &data)?;
            }
            Ok(serde_json::Value::Null)
        }
        ModCall::StorageKeys => {
            require(Permission::Storage)?;
            let _guard = registry.storage_lock.lock().expect("storage lock");
            let keys: Vec<String> = registry.read_storage(mod_id)?.into_keys().collect();
            serde_json::to_value(keys).map_err(|e| e.to_string())
        }
        ModCall::Notify { text } => {
            require(Permission::Notifications)?;
            let text: String = text.chars().take(TOAST_LIMIT).collect();
            crate::host::toasts::push(
                app,
                overlay_ipc::Toast {
                    text,
                    source: Some(info.manifest.name.clone()),
                },
            );
            Ok(serde_json::Value::Null)
        }
        ModCall::GameEntities => {
            require(Permission::GameEvents)?;
            Ok(app
                .state::<crate::host::providers::Providers>()
                .entities()
                .unwrap_or(serde_json::Value::Null))
        }
        ModCall::GameHistory => {
            require(Permission::GameEvents)?;
            Ok(app.state::<crate::host::providers::Providers>().history())
        }
        ModCall::GameInfo => {
            require(Permission::GameEvents)?;
            Ok(app
                .state::<crate::host::providers::Providers>()
                .info()
                .unwrap_or(serde_json::Value::Null))
        }
        ModCall::BusPost { message } => {
            app.state::<UiProcesses>()
                .broadcast(&CoreMessage::ModEvent {
                    mod_id: Some(mod_id.to_owned()),
                    name: "bus".into(),
                    data: message,
                    permission: None,
                });
            Ok(serde_json::Value::Null)
        }
    }
}
