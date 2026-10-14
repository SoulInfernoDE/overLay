//! User settings and widget layout of the overLay host, stored as TOML in the platform
//! config folder (`~/.config/overlay/` on Linux).
//!
//! Loading never fails hard: a missing file gives the defaults, and a broken file is
//! moved aside to `<name>.toml.broken` so the user's edits are not lost silently.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A value with its own TOML file in the config folder.
pub trait StoredFile: Default + Serialize + DeserializeOwned {
    const FILE_NAME: &'static str;
}

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
    pub overlay: OverlaySettings,
}

impl StoredFile for Settings {
    const FILE_NAME: &'static str = "settings.toml";
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    /// Shows or hides the launcher window, e.g. `Ctrl+Shift+O`.
    pub toggle_launcher: String,
    /// Shows or hides the in-game overlay.
    pub toggle_overlay: String,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle_launcher: "Ctrl+Shift+O".into(),
            toggle_overlay: "Ctrl+Shift+L".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OverlaySettings {
    /// Show the overlay over running games at all.
    pub enabled: bool,
    pub stacking: Stacking,
    /// Show the built-in demo widget (for testing the overlay without mods).
    pub demo_widget: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            stacking: Stacking::Above,
            demo_widget: false,
        }
    }
}

/// How the overlay stays above the game on X11.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stacking {
    /// Ask the window manager to keep the overlay on top.
    #[default]
    Above,
    /// Bypass the window manager (override-redirect), for window managers that put
    /// fullscreen games above "always on top" windows.
    Override,
}

/// Widget positions: game id → resolution (`1920x1080`) → widget id → position.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Layout(pub BTreeMap<String, BTreeMap<String, BTreeMap<String, Position>>>);

impl StoredFile for Layout {
    const FILE_NAME: &'static str = "layout.toml";
}

/// Top-left corner of a widget in CSS pixels of the overlay page.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Layout {
    /// Widget positions of `game` at `resolution` (`1920x1080`). Without an entry for that
    /// resolution, the entry of the closest stored resolution is scaled to it, so e.g. a
    /// layout made in fullscreen carries over to the slightly smaller windowed mode.
    pub fn widgets(&self, game: &str, resolution: &str) -> BTreeMap<String, Position> {
        let Some(resolutions) = self.0.get(game) else {
            return BTreeMap::new();
        };
        if let Some(exact) = resolutions.get(resolution) {
            return exact.clone();
        }
        let Some((width, height)) = parse_resolution(resolution) else {
            return BTreeMap::new();
        };
        let closest = resolutions
            .iter()
            .filter_map(|(key, widgets)| Some((parse_resolution(key)?, widgets)))
            .min_by(|((w1, h1), _), ((w2, h2), _)| {
                let distance = |w: f64, h: f64| (w - width).abs() + (h - height).abs();
                distance(*w1, *h1).total_cmp(&distance(*w2, *h2))
            });
        let Some(((from_width, from_height), widgets)) = closest else {
            return BTreeMap::new();
        };
        widgets
            .iter()
            .map(|(id, p)| {
                let scaled = Position {
                    x: p.x * width / from_width,
                    y: p.y * height / from_height,
                };
                (id.clone(), scaled)
            })
            .collect()
    }

    pub fn set(&mut self, game: &str, resolution: &str, widget: &str, position: Position) {
        self.0
            .entry(game.to_owned())
            .or_default()
            .entry(resolution.to_owned())
            .or_default()
            .insert(widget.to_owned(), position);
    }
}

/// `"1920x1080"` → `(1920.0, 1080.0)`; rejects empty sizes.
fn parse_resolution(resolution: &str) -> Option<(f64, f64)> {
    let (width, height) = resolution.split_once('x')?;
    let (width, height): (f64, f64) = (width.parse().ok()?, height.parse().ok()?);
    (width > 0.0 && height > 0.0).then_some((width, height))
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("{path} is not valid TOML, it was moved to {backup}: {source}")]
    Invalid {
        path: PathBuf,
        backup: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("cannot write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
}

/// Reads and writes one TOML file.
pub struct TomlStore<T> {
    path: PathBuf,
    _value: PhantomData<fn() -> T>,
}

pub type SettingsStore = TomlStore<Settings>;
pub type LayoutStore = TomlStore<Layout>;

impl<T: StoredFile> TomlStore<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            _value: PhantomData,
        }
    }

    /// The file in the platform config folder, if the platform has one.
    pub fn default_location() -> Option<Self> {
        let dirs = directories::ProjectDirs::from("io.github", "overLay", "overLay")?;
        Some(Self::new(dirs.config_dir().join(T::FILE_NAME)))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The stored value, or the default plus the reason the file could not be used.
    pub fn load(&self) -> (T, Option<SettingsError>) {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return (T::default(), None),
            Err(source) => {
                let error = SettingsError::Read {
                    path: self.path.clone(),
                    source,
                };
                return (T::default(), Some(error));
            }
        };
        match toml::from_str(&text) {
            Ok(value) => (value, None),
            Err(source) => {
                let backup = self.path.with_extension("toml.broken");
                // Best effort: if the rename fails, the next save overwrites the file.
                let _ = fs::rename(&self.path, &backup);
                let error = SettingsError::Invalid {
                    path: self.path.clone(),
                    backup,
                    source: Box::new(source),
                };
                (T::default(), Some(error))
            }
        }
    }

    /// Writes atomically: a temporary file in the same folder is renamed over the old one.
    pub fn save(&self, value: &T) -> Result<(), SettingsError> {
        let write_error = |source| SettingsError::Write {
            path: self.path.clone(),
            source,
        };
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(write_error)?;
        }
        let text = toml::to_string_pretty(value).expect("stored values always serialize to TOML");
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
        let store = SettingsStore::new(dir.path().join("nested").join(Settings::FILE_NAME));
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
                toggle_overlay: "Alt+F10".into(),
            },
            overlay: OverlaySettings {
                enabled: false,
                stacking: Stacking::Override,
                demo_widget: true,
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

    #[test]
    fn layout_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = LayoutStore::new(dir.path().join(Layout::FILE_NAME));
        let mut layout = Layout::default();
        layout.set(
            "hearthstone",
            "1920x1080",
            "demo",
            Position { x: 12.5, y: 40.0 },
        );
        store.save(&layout).unwrap();
        let (loaded, error) = store.load();
        assert!(error.is_none());
        assert_eq!(
            loaded.widgets("hearthstone", "1920x1080").get("demo"),
            Some(&Position { x: 12.5, y: 40.0 })
        );
        // Another resolution falls back to the stored one, scaled.
        let scaled = loaded.widgets("hearthstone", "3840x2160");
        assert_eq!(scaled.get("demo"), Some(&Position { x: 25.0, y: 80.0 }));
    }

    #[test]
    fn missing_resolution_scales_the_closest_one() {
        let mut layout = Layout::default();
        layout.set("hs", "1920x1080", "demo", Position { x: 960.0, y: 540.0 });
        layout.set("hs", "1280x720", "demo", Position { x: 10.0, y: 10.0 });
        let scaled = layout.widgets("hs", "1920x990");
        assert_eq!(scaled.get("demo"), Some(&Position { x: 960.0, y: 495.0 }));
        // An exact entry always wins.
        assert_eq!(
            layout.widgets("hs", "1280x720").get("demo"),
            Some(&Position { x: 10.0, y: 10.0 })
        );
        assert!(layout.widgets("other", "1920x1080").is_empty());
        assert!(layout.widgets("hs", "garbage").is_empty());
    }
}
