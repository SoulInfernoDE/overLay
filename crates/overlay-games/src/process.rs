//! Platform process lists: `/proc` on Linux, a ToolHelp snapshot on Windows.

/// A running process. `name` is `/proc/<pid>/comm` on Linux (at most 15 bytes) and the
/// executable file name on Windows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

#[cfg(target_os = "linux")]
pub fn list_processes() -> Vec<ProcessInfo> {
    list_processes_in(std::path::Path::new("/proc"))
}

/// Reads a `/proc`-like directory; separate from [`list_processes`] for tests.
#[cfg(target_os = "linux")]
pub fn list_processes_in(root: &std::path::Path) -> Vec<ProcessInfo> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let pid = entry.file_name().to_str()?.parse::<u32>().ok()?;
            // Processes can exit between read_dir and this read; skip them.
            let comm = std::fs::read_to_string(entry.path().join("comm")).ok()?;
            Some(ProcessInfo {
                pid,
                name: comm.trim_end_matches('\n').to_owned(),
            })
        })
        .collect()
}

#[cfg(windows)]
#[allow(unsafe_code)]
pub fn list_processes() -> Vec<ProcessInfo> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    let mut processes = Vec::new();
    // SAFETY: plain Win32 calls; the snapshot handle is checked and closed below, and
    // `entry` is a zeroed POD struct with `dwSize` set as the API requires.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return processes;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snapshot, &mut entry) != 0;
        while ok {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            processes.push(ProcessInfo {
                pid: entry.th32ProcessID,
                name: String::from_utf16_lossy(&entry.szExeFile[..len]),
            });
            ok = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
    }
    processes
}

#[cfg(not(any(target_os = "linux", windows)))]
pub fn list_processes() -> Vec<ProcessInfo> {
    Vec::new()
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn reads_fake_proc_tree() {
        let root = tempfile::tempdir().unwrap();
        for (pid, comm) in [("12", "Hearthstone.x86\n"), ("99", "bash\n")] {
            let dir = root.path().join(pid);
            std::fs::create_dir(&dir).unwrap();
            std::fs::write(dir.join("comm"), comm).unwrap();
        }
        std::fs::create_dir(root.path().join("self")).unwrap();

        let mut processes = list_processes_in(root.path());
        processes.sort_by_key(|p| p.pid);
        assert_eq!(
            processes,
            vec![
                ProcessInfo {
                    pid: 12,
                    name: "Hearthstone.x86".into()
                },
                ProcessInfo {
                    pid: 99,
                    name: "bash".into()
                },
            ]
        );
    }

    #[test]
    fn real_proc_lists_this_process() {
        let me = std::process::id();
        assert!(list_processes().iter().any(|p| p.pid == me));
    }
}
