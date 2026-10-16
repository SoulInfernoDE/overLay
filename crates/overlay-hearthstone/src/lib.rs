//! Hearthstone game provider: finds the game's logs, makes sure the game writes them,
//! follows `Power.log` and turns it into game events and a game summary for mods.

pub mod locate;
pub mod log_config;
pub mod power;
pub mod state;
pub mod tail;

use std::io;
use std::path::PathBuf;

pub use state::{GameEvent, GameInfo, GameResult, GameState, Side};

use crate::locate::newest_power_log;
use crate::tail::LogTail;

/// Follows the newest `Power.log` below a `Logs/` folder.
pub struct Provider {
    logs_dir: PathBuf,
    tail: Option<LogTail>,
    state: GameState,
}

impl Provider {
    pub fn new(logs_dir: impl Into<PathBuf>) -> Self {
        Self {
            logs_dir: logs_dir.into(),
            tail: None,
            state: GameState::default(),
        }
    }

    /// Reads what was written since the last call and returns the resulting events.
    /// When Hearthstone starts a new log folder (a new game client session), the provider
    /// switches to it.
    pub fn poll(&mut self) -> io::Result<Vec<GameEvent>> {
        let newest = newest_power_log(&self.logs_dir);
        if self.tail.as_ref().is_none_or(|t| t.path() != newest) {
            self.tail = Some(LogTail::new(newest));
        }
        let lines = self
            .tail
            .as_mut()
            .expect("tail was just set")
            .read_lines()?;
        let mut events = Vec::new();
        for record in lines.iter().filter_map(|l| power::parse_line(l)) {
            events.extend(self.state.apply(record));
        }
        Ok(events)
    }

    pub fn info(&self) -> GameInfo {
        self.state.info()
    }
}
