//! Makes Hearthstone write the logs the provider needs (`log.config`).
//!
//! The file is an INI file. Missing sections and keys are added; everything the user
//! (or another tracker) set up stays, except that the needed sections must print to a
//! file at log level 1. Hearthstone reads it only at start, so a change needs a restart
//! of the game.

use std::fs;
use std::io;
use std::path::Path;

/// Sections the provider reads, and the settings each needs.
const SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Power",
        &[
            ("LogLevel", "1"),
            ("FilePrinting", "true"),
            ("Verbose", "true"),
        ],
    ),
    (
        "LoadingScreen",
        &[("LogLevel", "1"), ("FilePrinting", "true")],
    ),
    ("Decks", &[("LogLevel", "1"), ("FilePrinting", "true")]),
    ("Arena", &[("LogLevel", "1"), ("FilePrinting", "true")]),
];

/// Brings the file at `path` up to date; returns whether it changed.
pub fn ensure(path: &Path) -> io::Result<bool> {
    let current = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };
    let updated = merge(&current);
    if updated == current {
        return Ok(false);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let temp = path.with_extension("config.tmp");
    fs::write(&temp, &updated)?;
    fs::rename(&temp, path)?;
    Ok(true)
}

/// Returns `text` with every needed section and key present and set.
pub fn merge(text: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    for (section, settings) in SECTIONS {
        let header = format!("[{section}]");
        let start = match lines
            .iter()
            .position(|l| l.trim().eq_ignore_ascii_case(&header))
        {
            Some(index) => index,
            None => {
                if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                    lines.push(String::new());
                }
                lines.push(header);
                lines.len() - 1
            }
        };
        for (key, value) in *settings {
            let end = lines[start + 1..]
                .iter()
                .position(|l| l.trim().starts_with('['))
                .map_or(lines.len(), |i| start + 1 + i);
            let existing = lines[start + 1..end].iter().position(|l| {
                l.split_once('=')
                    .is_some_and(|(k, _)| k.trim().eq_ignore_ascii_case(key))
            });
            let wanted = format!("{key}={value}");
            match existing {
                Some(i) => {
                    let line = &mut lines[start + 1 + i];
                    let current = line.split_once('=').map(|(_, v)| v.trim()).unwrap_or("");
                    if !current.eq_ignore_ascii_case(value) {
                        *line = wanted;
                    }
                }
                None => {
                    // Insert after the section's last non-empty line.
                    let mut at = end;
                    while at > start + 1 && lines[at - 1].trim().is_empty() {
                        at -= 1;
                    }
                    lines.insert(at, wanted);
                }
            }
        }
    }
    let mut result = lines.join("\n");
    result.push('\n');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_all_sections_from_nothing() {
        let text = merge("");
        assert!(text.contains("[Power]\nLogLevel=1\nFilePrinting=true\nVerbose=true\n"));
        assert!(text.contains("[Decks]\nLogLevel=1\nFilePrinting=true"));
    }

    #[test]
    fn keeps_user_settings_and_fixes_what_is_needed() {
        let original = "[Power]\nLogLevel=1\nFilePrinting=false\nConsolePrinting=true\n\n[Zone]\nLogLevel=1\nFilePrinting=true\n";
        let text = merge(original);
        assert!(text.contains(
            "[Power]\nLogLevel=1\nFilePrinting=true\nConsolePrinting=true\nVerbose=true\n"
        ));
        assert!(text.contains("[Zone]\nLogLevel=1\nFilePrinting=true\n"));
        assert!(text.contains("[LoadingScreen]"));
    }

    #[test]
    fn is_stable() {
        let once = merge("[Asset]\nLogLevel=1\n");
        assert_eq!(merge(&once), once);
    }

    #[test]
    fn ensure_reports_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Blizzard/Hearthstone/log.config");
        assert!(ensure(&path).unwrap());
        assert!(!ensure(&path).unwrap());
    }
}
