//! Finds where a running Hearthstone writes its logs and reads its `log.config`.
//!
//! | Client | Logs | `log.config` |
//! |---|---|---|
//! | Native Linux (hearthstone-linux-gui) | `<working dir>/Logs/` | `~/Library/Preferences/Blizzard/Hearthstone/` |
//! | Wine/Proton | `<install dir>/Logs/` | `<prefix>/drive_c/users/<user>/AppData/Local/Blizzard/Hearthstone/` |
//! | Windows | `<install dir>\Logs\` | `%LOCALAPPDATA%\Blizzard\Hearthstone\` |
//!
//! Every game start creates a new `Logs/Hearthstone_<date>_<time>/` folder. The log size
//! limit is set in `client.config` next to the game executable (see `client_config`).

use std::fs;
use std::path::{Path, PathBuf};

use overlay_games::Flavor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Install {
    pub logs_dir: PathBuf,
    pub log_config: PathBuf,
    /// `client.config` in the game folder.
    pub client_config: PathBuf,
    /// hearthstone-linux-gui's own `client.config.overrides` (native client only).
    pub launcher_overrides: Option<PathBuf>,
}

/// Where the Hearthstone process `pid` keeps its logs and log settings.
pub fn locate(pid: u32, flavor: Flavor) -> Option<Install> {
    imp::locate(pid, flavor)
}

/// The `Power.log` of the newest log folder (older clients wrote `Logs/Power.log`).
pub fn newest_power_log(logs_dir: &Path) -> PathBuf {
    let newest = fs::read_dir(logs_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.starts_with("Hearthstone_"))
        .max();
    match newest {
        Some(name) => logs_dir.join(name).join("Power.log"),
        None => logs_dir.join("Power.log"),
    }
}

/// Maps a Windows path inside a Wine prefix to a Unix path (`C:\x` → `<prefix>/drive_c/x`,
/// `Z:\x` → `/x`).
pub fn wine_to_unix(prefix: &Path, windows_path: &str) -> Option<PathBuf> {
    let (drive, rest) = windows_path.split_once(":\\")?;
    let rest = rest.replace('\\', "/");
    match drive.to_ascii_lowercase().as_str() {
        "z" => Some(PathBuf::from("/").join(rest)),
        letter if letter.len() == 1 => Some(prefix.join(format!("drive_{letter}")).join(rest)),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::*;

    pub fn locate(pid: u32, flavor: Flavor) -> Option<Install> {
        let proc_dir = PathBuf::from(format!("/proc/{pid}"));
        let cwd = fs::read_link(proc_dir.join("cwd")).ok();
        match flavor {
            Flavor::Native => {
                let home = std::env::var_os("HOME").map(PathBuf::from)?;
                // The launcher starts the game in its folder; the binary is in `Bin/`.
                let game_dir = cwd
                    .filter(|d| d.join("Logs").is_dir() || d.join("Bin").is_dir())
                    .or_else(|| {
                        fs::read_link(proc_dir.join("exe"))
                            .ok()?
                            .parent()?
                            .parent()
                            .map(Path::to_path_buf)
                    })?;
                let config_home = std::env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".config"));
                Some(Install {
                    logs_dir: game_dir.join("Logs"),
                    log_config: home.join("Library/Preferences/Blizzard/Hearthstone/log.config"),
                    client_config: game_dir.join("client.config"),
                    launcher_overrides: Some(
                        config_home.join("hearthstone-linux-gui/client.config.overrides"),
                    ),
                })
            }
            Flavor::Wine => {
                let environ = fs::read(proc_dir.join("environ")).ok()?;
                let env = |key: &str| -> Option<String> {
                    environ
                        .split(|b| *b == 0)
                        .filter_map(|e| std::str::from_utf8(e).ok())
                        .find_map(|e| e.strip_prefix(key)?.strip_prefix('='))
                        .map(str::to_owned)
                };
                let prefix = env("WINEPREFIX")
                    .map(PathBuf::from)
                    .or_else(|| env("STEAM_COMPAT_DATA_PATH").map(|p| PathBuf::from(p).join("pfx")))
                    .or_else(|| env("HOME").map(|h| PathBuf::from(h).join(".wine")))?;
                let install_dir = cwd
                    .filter(|d| d.join("Hearthstone.exe").is_file())
                    .or_else(|| {
                        let cmdline = fs::read(proc_dir.join("cmdline")).ok()?;
                        let exe = cmdline.split(|b| *b == 0).next()?;
                        let exe = wine_to_unix(&prefix, std::str::from_utf8(exe).ok()?)?;
                        exe.parent().map(Path::to_path_buf)
                    })?;
                let user = wine_user(&prefix).or_else(|| env("USER"))?;
                Some(Install {
                    logs_dir: install_dir.join("Logs"),
                    log_config: prefix
                        .join("drive_c/users")
                        .join(user)
                        .join("AppData/Local/Blizzard/Hearthstone/log.config"),
                    client_config: install_dir.join("client.config"),
                    launcher_overrides: None,
                })
            }
        }
    }

    /// The prefix user that already has Hearthstone settings (Proton uses `steamuser`).
    fn wine_user(prefix: &Path) -> Option<String> {
        fs::read_dir(prefix.join("drive_c/users"))
            .ok()?
            .flatten()
            .filter(|e| e.file_name() != "Public")
            .find(|e| e.path().join("AppData/Local/Blizzard/Hearthstone").is_dir())
            .and_then(|e| e.file_name().into_string().ok())
    }
}

#[cfg(windows)]
mod imp {
    use super::*;

    #[allow(unsafe_code)]
    pub fn locate(pid: u32, _flavor: Flavor) -> Option<Install> {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        };

        let mut buffer = [0u16; 1024];
        let mut length = buffer.len() as u32;
        // SAFETY: the handle is checked and closed; the buffer outlives the call and its
        // length is passed in.
        let ok = unsafe {
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() {
                return None;
            }
            let ok = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                buffer.as_mut_ptr(),
                &mut length,
            );
            CloseHandle(process);
            ok
        };
        if ok == 0 {
            return None;
        }
        let exe = PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]));
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
        let install_dir = exe.parent()?;
        Some(Install {
            logs_dir: install_dir.join("Logs"),
            log_config: local.join("Blizzard\\Hearthstone\\log.config"),
            client_config: install_dir.join("client.config"),
            launcher_overrides: None,
        })
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    use super::*;

    pub fn locate(_pid: u32, _flavor: Flavor) -> Option<Install> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_wine_paths() {
        let prefix = Path::new("/home/me/Games/battlenet");
        assert_eq!(
            wine_to_unix(
                prefix,
                "C:\\Program Files (x86)\\Hearthstone\\Hearthstone.exe"
            ),
            Some(PathBuf::from(
                "/home/me/Games/battlenet/drive_c/Program Files (x86)/Hearthstone/Hearthstone.exe"
            ))
        );
        assert_eq!(
            wine_to_unix(prefix, "Z:\\opt\\games\\Hearthstone.exe"),
            Some(PathBuf::from("/opt/games/Hearthstone.exe"))
        );
        assert_eq!(wine_to_unix(prefix, "relative\\path"), None);
    }

    #[test]
    fn picks_the_newest_log_folder() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "Hearthstone_2026_10_15_00_33_23",
            "Hearthstone_2026_10_16_20_40_07",
        ] {
            fs::create_dir(dir.path().join(name)).unwrap();
        }
        assert_eq!(
            newest_power_log(dir.path()),
            dir.path().join("Hearthstone_2026_10_16_20_40_07/Power.log")
        );
        let empty = tempfile::tempdir().unwrap();
        assert_eq!(
            newest_power_log(empty.path()),
            empty.path().join("Power.log")
        );
    }
}
