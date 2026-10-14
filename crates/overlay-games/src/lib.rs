//! Game catalog and running-game detection.
//!
//! The catalog (`games/*.toml` in the repository) says which executables belong to a
//! game on which platform. [`Detector`] compares it with the process list and reports
//! games starting and exiting.

mod process;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
pub use process::list_processes_in;
pub use process::{ProcessInfo, list_processes};

/// Catalog files built into the host.
const BUILTIN: &[&str] = &[include_str!("../../../games/hearthstone.toml")];

/// Linux cuts `/proc/<pid>/comm` to this many bytes.
const LINUX_COMM_LEN: usize = 15;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("invalid game definition: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("duplicate game id {0:?}")]
    DuplicateId(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
}

impl Platform {
    pub const CURRENT: Self = if cfg!(windows) {
        Self::Windows
    } else {
        Self::Linux
    };
}

/// How the game runs: as a program built for this platform, or through Wine/Proton.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Flavor {
    Native,
    Wine,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessMatcher {
    pub executable: String,
    pub platform: Platform,
    pub flavor: Flavor,
}

impl ProcessMatcher {
    fn matches(&self, process: &ProcessInfo) -> bool {
        match self.platform {
            Platform::Linux => {
                let expected = truncate_bytes(&self.executable, LINUX_COMM_LEN);
                process.name == expected
            }
            Platform::Windows => process.name.eq_ignore_ascii_case(&self.executable),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameDefinition {
    pub id: String,
    /// Proper name of the game; shown as is, not translated.
    pub name: String,
    #[serde(default)]
    pub processes: Vec<ProcessMatcher>,
}

#[derive(Clone, Debug, Default)]
pub struct Catalog {
    games: Vec<GameDefinition>,
}

impl Catalog {
    /// The catalog files shipped with the host.
    pub fn builtin() -> Self {
        Self::from_sources(BUILTIN.iter().copied()).expect("built-in game catalog is valid")
    }

    pub fn from_sources<'a>(
        sources: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, CatalogError> {
        let mut games: Vec<GameDefinition> = Vec::new();
        for source in sources {
            let game: GameDefinition = toml::from_str(source)?;
            if games.iter().any(|g| g.id == game.id) {
                return Err(CatalogError::DuplicateId(game.id));
            }
            games.push(game);
        }
        games.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { games })
    }

    pub fn games(&self) -> &[GameDefinition] {
        &self.games
    }

    /// Running games among `processes`, for the given platform.
    pub fn detect(&self, platform: Platform, processes: &[ProcessInfo]) -> Vec<RunningGame> {
        let mut found = Vec::new();
        for game in &self.games {
            for matcher in game.processes.iter().filter(|m| m.platform == platform) {
                for process in processes.iter().filter(|p| matcher.matches(p)) {
                    found.push(RunningGame {
                        game_id: game.id.clone(),
                        pid: process.pid,
                        flavor: matcher.flavor,
                    });
                }
            }
        }
        found
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RunningGame {
    pub game_id: String,
    pub pid: u32,
    pub flavor: Flavor,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum GameEvent {
    Started(RunningGame),
    Exited(RunningGame),
}

/// Tracks running games between process-list snapshots.
pub struct Detector {
    catalog: Catalog,
    platform: Platform,
    running: BTreeMap<(String, u32), RunningGame>,
}

impl Detector {
    pub fn new(catalog: Catalog, platform: Platform) -> Self {
        Self {
            catalog,
            platform,
            running: BTreeMap::new(),
        }
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Applies a new process-list snapshot and returns what changed since the last one.
    pub fn update(&mut self, processes: &[ProcessInfo]) -> Vec<GameEvent> {
        let current: BTreeMap<(String, u32), RunningGame> = self
            .catalog
            .detect(self.platform, processes)
            .into_iter()
            .map(|g| ((g.game_id.clone(), g.pid), g))
            .collect();
        let mut events: Vec<GameEvent> = self
            .running
            .iter()
            .filter(|(key, _)| !current.contains_key(*key))
            .map(|(_, game)| GameEvent::Exited(game.clone()))
            .collect();
        events.extend(
            current
                .iter()
                .filter(|(key, _)| !self.running.contains_key(*key))
                .map(|(_, game)| GameEvent::Started(game.clone())),
        );
        self.running = current;
        events
    }

    pub fn running(&self) -> impl Iterator<Item = &RunningGame> {
        self.running.values()
    }
}

fn truncate_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.into(),
        }
    }

    #[test]
    fn builtin_catalog_contains_hearthstone() {
        let catalog = Catalog::builtin();
        assert!(catalog.games().iter().any(|g| g.id == "hearthstone"));
    }

    #[test]
    fn linux_matches_truncated_comm() {
        let catalog = Catalog::builtin();
        let found = catalog.detect(Platform::Linux, &[process(42, "Hearthstone.x86")]);
        assert_eq!(
            found,
            vec![RunningGame {
                game_id: "hearthstone".into(),
                pid: 42,
                flavor: Flavor::Native
            }]
        );
    }

    #[test]
    fn linux_detects_wine() {
        let catalog = Catalog::builtin();
        let found = catalog.detect(Platform::Linux, &[process(7, "Hearthstone.exe")]);
        assert_eq!(found[0].flavor, Flavor::Wine);
    }

    #[test]
    fn windows_match_ignores_case() {
        let catalog = Catalog::builtin();
        let found = catalog.detect(Platform::Windows, &[process(9, "hearthstone.EXE")]);
        assert_eq!(found[0].flavor, Flavor::Native);
    }

    #[test]
    fn unrelated_processes_are_ignored() {
        let catalog = Catalog::builtin();
        assert!(
            catalog
                .detect(Platform::Linux, &[process(1, "Hearthstone")])
                .is_empty()
        );
    }

    #[test]
    fn rejects_duplicate_ids() {
        let game = "id = \"x\"\nname = \"X\"\n";
        assert!(matches!(
            Catalog::from_sources([game, game]),
            Err(CatalogError::DuplicateId(_))
        ));
    }

    #[test]
    fn detector_reports_start_and_exit_once() {
        let mut detector = Detector::new(Catalog::builtin(), Platform::Linux);
        let started = detector.update(&[process(5, "Hearthstone.x86")]);
        assert!(matches!(started.as_slice(), [GameEvent::Started(g)] if g.pid == 5));
        assert!(detector.update(&[process(5, "Hearthstone.x86")]).is_empty());
        let exited = detector.update(&[]);
        assert!(matches!(exited.as_slice(), [GameEvent::Exited(g)] if g.pid == 5));
        assert_eq!(detector.running().count(), 0);
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        assert_eq!(truncate_bytes("äöü", 3), "ä");
        assert_eq!(truncate_bytes("short", 15), "short");
    }
}
