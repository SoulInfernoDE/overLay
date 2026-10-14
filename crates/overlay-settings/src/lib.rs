//! User settings of the overLay host, stored as TOML in the platform config folder
//! (`~/.config/overlay/settings.toml` on Linux).
//!
//! Loading never fails hard: a missing file gives the defaults, and a broken file is
//! moved aside to `settings.toml.broken` so the user's edits are not lost silently.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "settings.toml";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// UI language (BCP 47, e.g. `de`); `None` follows the system language.
    pub language: Option<String>,
    /// Start in the tray without opening the launcher window.
    pub start_minimized: bool,
    /// Start overLay when the user logs in.
    pub autostart: bool,
    pub hotkeys: Hotkeys,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    /// Shows or hides the launcher window, e.g. `Ctrl+Shift+O`.
    pub toggle_launcher: String,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle_launcher: "Ctrl+Shift+O".into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("{path} is not valid settings TOML, it was moved to {backup}: {source}")]
    Invalid {
        path: PathBuf,
        backup: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("cannot write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
}

/// Reads and writes one settings file.
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// `settings.toml` in the platform config folder, if the platform has one.
    pub fn default_location() -> Option<Self> {
        let dirs = directories::ProjectDirs::from("io.github", "overLay", "overLay")?;
        Some(Self::new(dirs.config_dir().join(FILE_NAME)))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The stored settings, or the defaults plus the reason they could not be used.
    pub fn load(&self) -> (Settings, Option<SettingsError>) {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return (Settings::default(), None);
            }
            Err(source) => {
                let error = SettingsError::Read {
                    path: self.path.clone(),
                    source,
                };
                return (Settings::default(), Some(error));
            }
        };
        match toml::from_str(&text) {
            Ok(settings) => (settings, None),
            Err(source) => {
                let backup = self.path.with_extension("toml.broken");
                // Best effort: if the rename fails, the next save overwrites the file.
                let _ = fs::rename(&self.path, &backup);
                let error = SettingsError::Invalid {
                    path: self.path.clone(),
                    backup,
                    source: Box::new(source),
                };
                (Settings::default(), Some(error))
            }
        }
    }

    /// Writes atomically: a temporary file in the same folder is renamed over the old one.
    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        let write_error = |source| SettingsError::Write {
            path: self.path.clone(),
            source,
        };
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(write_error)?;
        }
        let text = toml::to_string_pretty(settings).expect("settings always serialize to TOML");
        let temp = self.path.with_extension("toml.tmp");
        fs::write(&temp, text).map_err(write_error)?;
        fs::rename(&temp, &self.path).map_err(write_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, SettingsStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("nested").join(FILE_NAME));
        (dir, store)
    }

    #[test]
    fn missing_file_gives_defaults() {
        let (_dir, store) = store();
        let (settings, error) = store.load();
        assert_eq!(settings, Settings::default());
        assert!(error.is_none());
    }

    #[test]
    fn save_and_load_round_trip() {
        let (_dir, store) = store();
        let settings = Settings {
            language: Some("de".into()),
            start_minimized: true,
            autostart: true,
            hotkeys: Hotkeys {
                toggle_launcher: "Alt+F9".into(),
            },
        };
        store.save(&settings).unwrap();
        let (loaded, error) = store.load();
        assert!(error.is_none());
        assert_eq!(loaded, settings);
    }

    #[test]
    fn missing_fields_use_defaults_and_unknown_fields_are_ignored() {
        let (_dir, store) = store();
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        fs::write(store.path(), "language = \"de\"\nfuture_option = 1\n").unwrap();
        let (settings, error) = store.load();
        assert!(error.is_none());
        assert_eq!(settings.language.as_deref(), Some("de"));
        assert_eq!(settings.hotkeys, Hotkeys::default());
    }

    #[test]
    fn broken_file_is_moved_aside() {
        let (_dir, store) = store();
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        fs::write(store.path(), "language = [").unwrap();
        let (settings, error) = store.load();
        assert_eq!(settings, Settings::default());
        assert!(matches!(error, Some(SettingsError::Invalid { .. })));
        assert!(!store.path().exists());
        assert!(store.path().with_extension("toml.broken").exists());
    }
}
