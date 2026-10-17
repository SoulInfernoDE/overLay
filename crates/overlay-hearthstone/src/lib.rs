//! Hearthstone game provider: finds the game's logs, makes sure the game writes them,
//! follows `Power.log` (and `Decks.log` for the chosen deck) and turns them into game
//! events and a game summary for mods.

pub mod client_config;
pub mod decks;
pub mod locate;
pub mod log_config;
pub mod power;
pub mod state;
pub mod tail;

use std::io;
use std::path::PathBuf;

pub use state::{GameEvent, GameInfo, GameResult, GameState, Side};

use crate::decks::DeckLog;
use crate::locate::newest_power_log;
use crate::power::Record;
use crate::tail::LogTail;

/// Follows the newest `Power.log` below a `Logs/` folder, and `Decks.log` next to it.
pub struct Provider {
    logs_dir: PathBuf,
    tail: Option<LogTail>,
    decks_tail: Option<LogTail>,
    decks: DeckLog,
    state: GameState,
}

impl Provider {
    pub fn new(logs_dir: impl Into<PathBuf>) -> Self {
        Self {
            logs_dir: logs_dir.into(),
            tail: None,
            decks_tail: None,
            decks: DeckLog::default(),
            state: GameState::default(),
        }
    }

    /// Reads what was written since the last call and returns the resulting events.
    /// When Hearthstone starts a new log folder (a new game client session), the provider
    /// switches to it.
    pub fn poll(&mut self) -> io::Result<Vec<GameEvent>> {
        let newest = newest_power_log(&self.logs_dir);
        if self.tail.as_ref().is_none_or(|t| t.path() != newest) {
            let decks = newest.with_file_name("Decks.log");
            self.tail = Some(LogTail::new(newest));
            self.decks_tail = Some(LogTail::new(decks));
        }
        let lines = self
            .tail
            .as_mut()
            .expect("tail was just set")
            .read_lines()?;
        let deck_lines = self
            .decks_tail
            .as_mut()
            .expect("set with the tail")
            .read_lines()?;

        // Both logs in time order, so a deck reaches the game it was chosen for even
        // when several games are read at once. (Across midnight within one read the
        // order can be off; a deck is chosen seconds before its game, so this is rare.)
        let mut timed: Vec<(&str, bool, &String)> = lines
            .iter()
            .map(|l| (line_time(l).unwrap_or(""), false, l))
            .chain(
                deck_lines
                    .iter()
                    .map(|l| (line_time(l).unwrap_or(""), true, l)),
            )
            .collect();
        timed.sort_by(|a, b| a.0.cmp(b.0));

        let mut events = Vec::new();
        for (_, from_decks, line) in timed {
            let record = if from_decks {
                self.decks.apply(line).map(Record::Deck)
            } else {
                power::parse_line(line)
            };
            if let Some(record) = record {
                events.extend(self.state.apply(record));
            }
        }
        Ok(events)
    }

    pub fn info(&self) -> GameInfo {
        self.state.info()
    }

    pub fn entities(&self) -> Vec<state::EntitySnapshot> {
        self.state.entities_snapshot()
    }
}

/// The timestamp of a log line (`D 17:20:45.9193400 …` → `17:20:45.9193400`); the
/// fixed format sorts as text.
fn line_time(line: &str) -> Option<&str> {
    line.get(2..)?.split_once(' ').map(|(time, _)| time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_line_times() {
        assert_eq!(
            line_time("I 17:20:45.9193400 Finding Game With Deck:"),
            Some("17:20:45.9193400")
        );
        assert_eq!(line_time("D"), None);
    }
}
