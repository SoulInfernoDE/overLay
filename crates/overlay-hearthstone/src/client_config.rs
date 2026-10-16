//! Lifts Hearthstone's log size limit (`client.config`).
//!
//! Without this, Hearthstone stops writing logs once one of them reaches 10 MB; a busy
//! `Zone.log` gets there within minutes of a Battlegrounds game, and `Power.log` stops
//! with it, so the end of the game is never logged. The setting is
//!
//! ```text
//! [Log]
//! FileSizeLimit.Int=-1
//! ```
//!
//! Hearthstone also reads the flat form `Log.FileSizeLimit.Int=-1`, which the native
//! Linux launcher (hearthstone-linux-gui) uses for all its entries. Its own file of user
//! entries (`client.config.overrides`, `key=value` lines only) keeps the setting when
//! the launcher writes a new `client.config` after an update.

use std::fs;
use std::io;
use std::path::Path;

const SECTION: &str = "Log";
const KEY: &str = "FileSizeLimit.Int";
const VALUE: &str = "-1";

/// Makes sure the limit is lifted in `client.config`; returns whether the file changed.
pub fn ensure(path: &Path) -> io::Result<bool> {
    update(path, merge)
}

/// Adds the flat entry to the launcher's `client.config.overrides`, if that launcher is
/// installed (its config folder exists); returns whether the file changed.
pub fn ensure_launcher_override(path: &Path) -> io::Result<bool> {
    if !path.parent().is_some_and(Path::is_dir) {
        return Ok(false);
    }
    update(path, merge_flat)
}

fn update(path: &Path, merge: fn(&str) -> String) -> io::Result<bool> {
    let current = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };
    let updated = merge(&current);
    if updated == current {
        return Ok(false);
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, &updated)?;
    fs::rename(&temp, path)?;
    Ok(true)
}

/// Uses the style the file already has: a `[Log]` section in sectioned files, the flat
/// `Log.FileSizeLimit.Int` key otherwise.
pub fn merge(text: &str) -> String {
    let sectioned = text.lines().any(|l| l.trim().starts_with('['));
    if sectioned {
        merge_sectioned(text)
    } else {
        merge_flat(text)
    }
}

fn merge_flat(text: &str) -> String {
    let flat_key = format!("{SECTION}.{KEY}");
    let wanted = format!("{flat_key}={VALUE}");
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    match lines
        .iter()
        .position(|l| key_of(l).is_some_and(|k| k.eq_ignore_ascii_case(&flat_key)))
    {
        Some(index) => lines[index] = wanted,
        None => lines.push(wanted),
    }
    finish(lines)
}

fn merge_sectioned(text: &str) -> String {
    let header = format!("[{SECTION}]");
    let wanted = format!("{KEY}={VALUE}");
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let Some(start) = lines
        .iter()
        .position(|l| l.trim().eq_ignore_ascii_case(&header))
    else {
        if lines.last().is_some_and(|l| !l.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push(header);
        lines.push(wanted);
        return finish(lines);
    };
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim().starts_with('['))
        .map_or(lines.len(), |i| start + 1 + i);
    match lines[start + 1..end]
        .iter()
        .position(|l| key_of(l).is_some_and(|k| k.eq_ignore_ascii_case(KEY)))
    {
        Some(i) => lines[start + 1 + i] = wanted,
        None => lines.insert(start + 1, wanted),
    }
    finish(lines)
}

fn key_of(line: &str) -> Option<&str> {
    line.split_once('=').map(|(k, _)| k.trim())
}

fn finish(lines: Vec<String>) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_files_get_the_flat_key() {
        let config = "Aurora.Env=eu.actual.battle.net\nvsync:b=False\n";
        assert_eq!(
            merge(config),
            "Aurora.Env=eu.actual.battle.net\nvsync:b=False\nLog.FileSizeLimit.Int=-1\n"
        );
        let limited = "Log.FileSizeLimit.Int=10000\n";
        assert_eq!(merge(limited), "Log.FileSizeLimit.Int=-1\n");
    }

    #[test]
    fn sectioned_files_get_a_log_section() {
        assert_eq!(
            merge("[Aurora]\nEnv=eu\n"),
            "[Aurora]\nEnv=eu\n\n[Log]\nFileSizeLimit.Int=-1\n"
        );
        assert_eq!(
            merge("[Log]\nFileSizeLimit.Int=10000\nOther=1\n"),
            "[Log]\nFileSizeLimit.Int=-1\nOther=1\n"
        );
    }

    #[test]
    fn empty_file_and_stability() {
        assert_eq!(merge(""), "Log.FileSizeLimit.Int=-1\n");
        let once = merge("[Aurora]\nEnv=eu\n");
        assert_eq!(merge(&once), once);
    }

    #[test]
    fn launcher_override_only_when_the_launcher_exists() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("not-installed/client.config.overrides");
        assert!(!ensure_launcher_override(&missing).unwrap());
        assert!(!missing.exists());
        let present = dir.path().join("client.config.overrides");
        fs::write(&present, "# my entries\nvsync:b=False\n").unwrap();
        assert!(ensure_launcher_override(&present).unwrap());
        assert_eq!(
            fs::read_to_string(&present).unwrap(),
            "# my entries\nvsync:b=False\nLog.FileSizeLimit.Int=-1\n"
        );
        assert!(!ensure_launcher_override(&present).unwrap());
    }
}
