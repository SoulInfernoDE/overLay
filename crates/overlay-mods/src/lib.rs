//! Mods: manifests, discovery and file access.
//!
//! A mod is a folder with an `overlay.json` manifest, its web files and its Fluent
//! catalogs in `locales/<locale>/*.ftl`. Installed mods live in
//! `<data dir>/mods/<id>/`; in developer mode, extra folders can be loaded directly.

mod manifest;

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

pub use manifest::{
    DefaultPosition, HotkeySpec, Manifest, ManifestError, Permission, SettingKind, SettingSpec,
    WindowKind, WindowSpec, is_relative_inside, is_reverse_dns,
};

pub const MANIFEST_FILE: &str = "overlay.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModSource {
    /// Installed through a mod repository (phase 4).
    Installed,
    /// Loaded from a local folder in developer mode.
    Developer,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModInfo {
    pub manifest: Manifest,
    pub root: PathBuf,
    pub source: ModSource,
}

#[derive(Debug, thiserror::Error)]
pub enum ModError {
    #[error("{path}: {source}")]
    Manifest {
        path: PathBuf,
        source: ManifestError,
    },
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path}: window {window:?} points to {entry:?}, which does not exist")]
    MissingEntry {
        path: PathBuf,
        window: String,
        entry: String,
    },
    #[error("mod {id} is loaded twice ({first} and {second}); the first one is used")]
    Duplicate {
        id: String,
        first: PathBuf,
        second: PathBuf,
    },
}

/// Reads one mod folder.
pub fn load_mod(root: &Path, source: ModSource) -> Result<ModInfo, ModError> {
    let path = root.join(MANIFEST_FILE);
    let json = fs::read_to_string(&path).map_err(|source| ModError::Io {
        path: path.clone(),
        source,
    })?;
    let manifest = Manifest::parse(&json).map_err(|source| ModError::Manifest {
        path: path.clone(),
        source,
    })?;
    for (window, spec) in &manifest.windows {
        if !root.join(&spec.entry).is_file() {
            return Err(ModError::MissingEntry {
                path,
                window: window.clone(),
                entry: spec.entry.clone(),
            });
        }
    }
    Ok(ModInfo {
        manifest,
        root: root.to_path_buf(),
        source,
    })
}

/// Finds all mods: every subfolder of `installed_root`, then the developer folders.
/// Broken mods are reported in the second list and left out; developer folders win
/// over installed mods with the same id, so a mod can be tested before release.
pub fn discover(
    installed_root: Option<&Path>,
    developer: &[PathBuf],
) -> (Vec<ModInfo>, Vec<ModError>) {
    let mut mods: Vec<ModInfo> = Vec::new();
    let mut errors = Vec::new();
    let mut add = |result: Result<ModInfo, ModError>, mods: &mut Vec<ModInfo>| match result {
        Ok(info) => {
            if let Some(existing) = mods.iter_mut().find(|m| m.manifest.id == info.manifest.id) {
                if info.source == ModSource::Developer && existing.source == ModSource::Installed {
                    *existing = info;
                } else {
                    errors.push(ModError::Duplicate {
                        id: info.manifest.id.clone(),
                        first: existing.root.clone(),
                        second: info.root,
                    });
                }
            } else {
                mods.push(info);
            }
        }
        Err(err) => errors.push(err),
    };
    if let Some(root) = installed_root
        && let Ok(entries) = fs::read_dir(root)
    {
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            add(load_mod(&dir, ModSource::Installed), &mut mods);
        }
    }
    for dir in developer {
        add(load_mod(dir, ModSource::Developer), &mut mods);
    }
    mods.sort_by(|a, b| {
        a.manifest
            .name
            .to_lowercase()
            .cmp(&b.manifest.name.to_lowercase())
    });
    (mods, errors)
}

/// Resolves a path requested by a mod's page to a file inside the mod folder, or `None`
/// if it would leave the folder (also through symlinks).
pub fn resolve_file(root: &Path, request_path: &str) -> Option<PathBuf> {
    let relative = request_path.trim_start_matches('/');
    if !is_relative_inside(relative) {
        return None;
    }
    let root = fs::canonicalize(root).ok()?;
    let file = fs::canonicalize(root.join(relative)).ok()?;
    (file.starts_with(&root) && file.is_file()).then_some(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_mod(dir: &Path, id: &str, name: &str) {
        fs::create_dir_all(dir).unwrap();
        let manifest = serde_json::json!({
            "id": id, "version": "1.0.0", "name": name, "author": "Me", "license": "MIT",
            "source": "https://example.org/repo",
            "windows": { "main": { "kind": "desktop", "entry": "index.html" } }
        });
        fs::write(dir.join(MANIFEST_FILE), manifest.to_string()).unwrap();
        fs::write(dir.join("index.html"), "<p>hi</p>").unwrap();
    }

    #[test]
    fn discovers_installed_and_developer_mods() {
        let tmp = tempfile::tempdir().unwrap();
        let installed = tmp.path().join("mods");
        write_mod(&installed.join("b"), "org.example.b", "Beta");
        write_mod(&installed.join("a"), "org.example.a", "Alpha");
        fs::create_dir_all(installed.join("broken")).unwrap();
        fs::write(installed.join("broken").join(MANIFEST_FILE), "{").unwrap();
        let dev = tmp.path().join("dev-a");
        write_mod(&dev, "org.example.a", "Alpha (dev)");

        let (mods, errors) = discover(Some(&installed), std::slice::from_ref(&dev));
        let names: Vec<&str> = mods.iter().map(|m| m.manifest.name.as_str()).collect();
        assert_eq!(names, ["Alpha (dev)", "Beta"]);
        assert_eq!(mods[0].source, ModSource::Developer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ModError::Manifest { .. }));
    }

    #[test]
    fn missing_entry_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_mod(tmp.path(), "org.example.x", "X");
        fs::remove_file(tmp.path().join("index.html")).unwrap();
        assert!(matches!(
            load_mod(tmp.path(), ModSource::Developer),
            Err(ModError::MissingEntry { .. })
        ));
    }

    #[test]
    fn resolve_file_stays_inside() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("mod");
        fs::create_dir_all(root.join("locales/en")).unwrap();
        fs::write(root.join("locales/en/main.ftl"), "x = y").unwrap();
        fs::write(tmp.path().join("secret.txt"), "secret").unwrap();

        assert!(resolve_file(&root, "/locales/en/main.ftl").is_some());
        assert!(resolve_file(&root, "/../secret.txt").is_none());
        assert!(resolve_file(&root, "/locales/../../secret.txt").is_none());
        assert!(resolve_file(&root, "/missing.js").is_none());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(tmp.path().join("secret.txt"), root.join("link.txt"))
                .unwrap();
            assert!(resolve_file(&root, "/link.txt").is_none());
        }
    }
}
