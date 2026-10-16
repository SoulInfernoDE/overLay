//! Follows a growing log file line by line.
//!
//! Hearthstone truncates a log once it reaches its size cap and starts a new log folder
//! on every game start; both are handled by the caller asking again (`reopen`) or by
//! the file shrinking under us.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct LogTail {
    path: PathBuf,
    position: u64,
    /// Bytes after the last newline, kept until the line is complete.
    partial: Vec<u8>,
}

impl LogTail {
    /// Starts at the beginning, so a game in progress is read in full.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            position: 0,
            partial: Vec::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Complete lines written since the last call. A missing file yields nothing yet.
    pub fn read_lines(&mut self) -> io::Result<Vec<String>> {
        let mut file = match File::open(&self.path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let length = file.metadata()?.len();
        if length < self.position {
            // Truncated: Hearthstone started the log over.
            self.position = 0;
            self.partial.clear();
        }
        if length == self.position {
            return Ok(Vec::new());
        }
        file.seek(SeekFrom::Start(self.position))?;
        let mut buffer = Vec::with_capacity((length - self.position) as usize);
        file.read_to_end(&mut buffer)?;
        self.position += buffer.len() as u64;

        self.partial.extend_from_slice(&buffer);
        let Some(last_newline) = self.partial.iter().rposition(|&b| b == b'\n') else {
            return Ok(Vec::new());
        };
        let rest = self.partial.split_off(last_newline + 1);
        let complete = std::mem::replace(&mut self.partial, rest);
        Ok(String::from_utf8_lossy(&complete)
            .lines()
            .map(|l| l.trim_end_matches('\r').to_owned())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::Write;

    use super::*;

    #[test]
    fn reads_growing_and_truncated_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Power.log");
        let mut tail = LogTail::new(&path);
        assert!(tail.read_lines().unwrap().is_empty());

        std::fs::write(&path, "one\ntw").unwrap();
        assert_eq!(tail.read_lines().unwrap(), ["one"]);
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"o\nthree\n").unwrap();
        assert_eq!(tail.read_lines().unwrap(), ["two", "three"]);

        std::fs::write(&path, "new\n").unwrap();
        assert_eq!(tail.read_lines().unwrap(), ["new"]);
    }
}
