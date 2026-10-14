//! The mod manifest, `overlay.json`, and its validation.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Everything the host needs to know about a mod before running it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Reverse-DNS id, e.g. `io.github.soulinfernode.flint`.
    pub id: String,
    pub version: semver::Version,
    /// Proper name of the mod; shown as is, like an app name.
    pub name: String,
    /// Fluent message id of the mod's description in its own catalogs.
    #[serde(default = "default_description")]
    pub description: String,
    pub author: String,
    /// SPDX license identifier; the index CI checks that it is OSI-approved.
    pub license: String,
    /// Public source repository (https).
    pub source: String,
    /// Game ids from the host's game catalog; empty means the mod has no in-game part.
    #[serde(default)]
    pub games: Vec<String>,
    #[serde(default)]
    pub permissions: BTreeSet<Permission>,
    /// Hosts the mod may contact over https, e.g. `api.hearthstonejson.com`.
    #[serde(default)]
    pub network: Vec<String>,
    pub windows: BTreeMap<String, WindowSpec>,
    #[serde(default)]
    pub hotkeys: BTreeMap<String, HotkeySpec>,
    #[serde(default)]
    pub settings: Vec<SettingSpec>,
}

fn default_description() -> String {
    "mod-description".into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Permission {
    /// Running games and the game window's size.
    #[serde(rename = "game.state")]
    GameState,
    /// Live game events from the host's game providers.
    #[serde(rename = "game.events")]
    GameEvents,
    /// The mod's own key-value storage.
    #[serde(rename = "storage")]
    Storage,
    /// Toast notifications in the overlay.
    #[serde(rename = "notifications")]
    Notifications,
    /// Global hotkeys declared in the manifest.
    #[serde(rename = "hotkeys")]
    Hotkeys,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSpec {
    pub kind: WindowKind,
    /// HTML file inside the mod folder.
    pub entry: String,
    /// Size in CSS pixels (overlay windows only).
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    /// Default position of an overlay window; users can drag it elsewhere.
    #[serde(default)]
    pub position: Option<DefaultPosition>,
    /// The whole window lets clicks through to the game (purely visual overlays).
    #[serde(default)]
    pub click_through: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowKind {
    /// In-game, inside the shared overlay.
    Overlay,
    /// In the launcher, e.g. statistics or settings pages.
    Desktop,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DefaultPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotkeySpec {
    /// Default key combination, e.g. `Ctrl+Shift+H`.
    pub default: String,
    /// Fluent message id describing the hotkey.
    pub label: String,
}

/// One entry of the settings page the launcher generates for the mod.
///
/// Unlike the rest of the manifest, unknown keys are ignored here: serde cannot combine
/// `deny_unknown_fields` with the flattened `kind`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingSpec {
    pub id: String,
    /// Fluent message id of the label.
    pub label: String,
    #[serde(flatten)]
    pub kind: SettingKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SettingKind {
    Boolean {
        default: bool,
    },
    Number {
        default: f64,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
    },
    Text {
        default: String,
    },
    /// One of `options`; each option is a Fluent message id in the mod's catalogs.
    Choice {
        default: String,
        options: Vec<String>,
    },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("overlay.json is not valid: {0}")]
    Syntax(String),
    #[error("invalid id {0:?}: use lowercase reverse-DNS like io.github.name.mod")]
    Id(String),
    #[error("{field} must not be empty")]
    Empty { field: &'static str },
    #[error("source {0:?} must be an https URL to the public repository")]
    Source(String),
    #[error("invalid network host {0:?}: give a plain host name like api.example.com")]
    NetworkHost(String),
    #[error("the mod declares no windows")]
    NoWindows,
    #[error("invalid {kind} id {id:?}: use lowercase letters, digits and dashes")]
    LocalId { kind: &'static str, id: String },
    #[error("window {window:?}: entry {entry:?} must be a relative path inside the mod")]
    Entry { window: String, entry: String },
    #[error("window {0:?}: overlay windows need a positive width and height")]
    OverlaySize(String),
    #[error("hotkeys need the \"hotkeys\" permission")]
    HotkeysWithoutPermission,
    #[error("setting {0:?} is declared twice")]
    DuplicateSetting(String),
    #[error("setting {id:?}: {reason}")]
    Setting { id: String, reason: &'static str },
}

impl Manifest {
    pub fn parse(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self =
            serde_json::from_str(json).map_err(|e| ManifestError::Syntax(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if !is_reverse_dns(&self.id) {
            return Err(ManifestError::Id(self.id.clone()));
        }
        for (field, value) in [
            ("name", &self.name),
            ("author", &self.author),
            ("license", &self.license),
            ("description", &self.description),
        ] {
            if value.trim().is_empty() {
                return Err(ManifestError::Empty { field });
            }
        }
        if !self.source.starts_with("https://") || self.source.len() <= "https://".len() {
            return Err(ManifestError::Source(self.source.clone()));
        }
        if let Some(host) = self.network.iter().find(|h| !is_host_name(h)) {
            return Err(ManifestError::NetworkHost(host.clone()));
        }
        if self.windows.is_empty() {
            return Err(ManifestError::NoWindows);
        }
        for (id, window) in &self.windows {
            check_local_id("window", id)?;
            if !is_relative_inside(&window.entry) {
                return Err(ManifestError::Entry {
                    window: id.clone(),
                    entry: window.entry.clone(),
                });
            }
            if window.kind == WindowKind::Overlay {
                let positive = |v: Option<f64>| v.is_some_and(|v| v.is_finite() && v > 0.0);
                if !positive(window.width) || !positive(window.height) {
                    return Err(ManifestError::OverlaySize(id.clone()));
                }
            }
        }
        for id in self.hotkeys.keys() {
            check_local_id("hotkey", id)?;
        }
        if !self.hotkeys.is_empty() && !self.permissions.contains(&Permission::Hotkeys) {
            return Err(ManifestError::HotkeysWithoutPermission);
        }
        let mut seen = BTreeSet::new();
        for setting in &self.settings {
            check_local_id("setting", &setting.id)?;
            if !seen.insert(&setting.id) {
                return Err(ManifestError::DuplicateSetting(setting.id.clone()));
            }
            check_setting(setting)?;
        }
        Ok(())
    }

    pub fn has(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

fn check_setting(setting: &SettingSpec) -> Result<(), ManifestError> {
    let error = |reason| ManifestError::Setting {
        id: setting.id.clone(),
        reason,
    };
    match &setting.kind {
        SettingKind::Number { default, min, max } => {
            if min.is_some_and(|min| *default < min) || max.is_some_and(|max| *default > max) {
                return Err(error("the default lies outside min..max"));
            }
        }
        SettingKind::Choice { default, options } => {
            if !options.contains(default) {
                return Err(error("the default is not one of the options"));
            }
        }
        SettingKind::Boolean { .. } | SettingKind::Text { .. } => {}
    }
    Ok(())
}

fn check_local_id(kind: &'static str, id: &str) -> Result<(), ManifestError> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-');
    if valid {
        Ok(())
    } else {
        Err(ManifestError::LocalId {
            kind,
            id: id.to_owned(),
        })
    }
}

/// `io.github.name.mod`: at least two lowercase segments of letters, digits and dashes.
pub fn is_reverse_dns(id: &str) -> bool {
    let segments: Vec<&str> = id.split('.').collect();
    id.len() <= 100
        && segments.len() >= 2
        && segments.iter().all(|s| {
            !s.is_empty()
                && !s.starts_with('-')
                && !s.ends_with('-')
                && s.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        })
}

/// A plain host name: no scheme, port, path or wildcard.
fn is_host_name(host: &str) -> bool {
    is_reverse_dns(host) && host.len() <= 253
}

/// A relative path that stays inside the mod folder.
pub fn is_relative_inside(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"{
        "id": "io.github.example.hello",
        "version": "0.1.0",
        "name": "Hello Overlay",
        "author": "Example",
        "license": "MIT",
        "source": "https://github.com/example/hello",
        "games": ["hearthstone"],
        "permissions": ["game.state", "storage", "hotkeys"],
        "network": ["api.hearthstonejson.com"],
        "windows": {
            "main": { "kind": "overlay", "entry": "index.html", "width": 280, "height": 160 },
            "desktop": { "kind": "desktop", "entry": "desktop/index.html" }
        },
        "hotkeys": { "toggle": { "default": "Ctrl+Shift+H", "label": "hotkey-toggle" } },
        "settings": [
            { "id": "show-clock", "label": "setting-clock", "type": "boolean", "default": true },
            { "id": "size", "label": "setting-size", "type": "choice", "default": "small",
              "options": ["small", "large"] }
        ]
    }"#;

    fn with(change: impl FnOnce(&mut serde_json::Value)) -> Result<Manifest, ManifestError> {
        let mut value: serde_json::Value = serde_json::from_str(VALID).unwrap();
        change(&mut value);
        Manifest::parse(&value.to_string())
    }

    #[test]
    fn example_mod_manifest_is_valid() {
        let json = include_str!("../../../mods/hello-overlay/public/overlay.json");
        let manifest = Manifest::parse(json).unwrap();
        assert_eq!(manifest.id, "io.github.soulinfernode.hello-overlay");
    }

    #[test]
    fn valid_manifest_parses() {
        let manifest = Manifest::parse(VALID).unwrap();
        assert_eq!(manifest.version, semver::Version::new(0, 1, 0));
        assert!(manifest.has(Permission::Storage));
        assert_eq!(manifest.description, "mod-description");
    }

    #[test]
    fn rejects_unknown_fields_and_permissions() {
        assert!(matches!(
            with(|v| v["telemetry"] = true.into()),
            Err(ManifestError::Syntax(_))
        ));
        assert!(matches!(
            with(|v| v["permissions"] = serde_json::json!(["filesystem"])),
            Err(ManifestError::Syntax(_))
        ));
    }

    #[test]
    fn rejects_bad_ids_and_sources() {
        assert!(matches!(
            with(|v| v["id"] = "Hello".into()),
            Err(ManifestError::Id(_))
        ));
        assert!(matches!(
            with(|v| v["id"] = "io..x".into()),
            Err(ManifestError::Id(_))
        ));
        assert!(matches!(
            with(|v| v["source"] = "http://github.com/x".into()),
            Err(ManifestError::Source(_))
        ));
    }

    #[test]
    fn network_hosts_must_be_plain_names() {
        for host in [
            "https://api.example.com",
            "*.example.com",
            "example.com/path",
            "localhost",
            "a.b:8080",
        ] {
            assert!(
                matches!(
                    with(|v| v["network"] = serde_json::json!([host])),
                    Err(ManifestError::NetworkHost(_))
                ),
                "{host} should be rejected"
            );
        }
    }

    #[test]
    fn entries_must_stay_inside_the_mod() {
        for entry in [
            "../x.html",
            "/etc/passwd",
            "a/../../b",
            "C:/x.html",
            "a\\b.html",
            "",
        ] {
            let result = with(|v| v["windows"]["main"]["entry"] = entry.into());
            assert!(
                matches!(result, Err(ManifestError::Entry { .. })),
                "{entry} should be rejected"
            );
        }
    }

    #[test]
    fn overlay_windows_need_a_size() {
        let result = with(|v| {
            v["windows"]["main"]
                .as_object_mut()
                .unwrap()
                .remove("width")
                .map(|_| ())
                .unwrap()
        });
        assert_eq!(result, Err(ManifestError::OverlaySize("main".into())));
    }

    #[test]
    fn hotkeys_need_their_permission() {
        let result = with(|v| v["permissions"] = serde_json::json!(["storage"]));
        assert_eq!(result, Err(ManifestError::HotkeysWithoutPermission));
    }

    #[test]
    fn settings_are_checked() {
        let result = with(|v| v["settings"][1]["default"] = "huge".into());
        assert!(matches!(result, Err(ManifestError::Setting { .. })));
        let result = with(|v| {
            let first = v["settings"][0].clone();
            v["settings"].as_array_mut().unwrap().push(first);
        });
        assert!(matches!(result, Err(ManifestError::DuplicateSetting(_))));
    }
}
