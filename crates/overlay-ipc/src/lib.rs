//! Messages between the overLay core process and its UI processes.
//!
//! The core starts each UI process (launcher, overlay) itself with piped stdin and
//! stdout, so no socket or port is exposed. Every message is one line of JSON. UI
//! processes must therefore never print to stdout; they log to stderr.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use overlay_games::Flavor;
use overlay_settings::{Position, Settings};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Command-line flag that starts a UI process: `overlay --ui launcher`.
pub const UI_FLAG: &str = "--ui";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiRole {
    Launcher,
    Overlay,
}

impl UiRole {
    pub fn as_arg(self) -> &'static str {
        match self {
            Self::Launcher => "launcher",
            Self::Overlay => "overlay",
        }
    }

    /// The role passed on the command line, if this is a UI process.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Self> {
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if arg == UI_FLAG {
                return match args.next()?.as_str() {
                    "launcher" => Some(Self::Launcher),
                    "overlay" => Some(Self::Overlay),
                    _ => None,
                };
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameView {
    pub id: String,
    pub name: String,
    /// How the game currently runs, or `None` when it is not running.
    pub running: Option<Flavor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: String,
    pub settings: Settings,
    pub settings_path: Option<String>,
    pub games: Vec<GameView>,
}

/// The game an overlay process draws over.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayTarget {
    pub game_id: String,
    pub game_name: String,
    pub pid: u32,
}

/// Core → UI process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CoreMessage {
    Response {
        id: u64,
        result: Result<serde_json::Value, String>,
    },
    GamesChanged {
        games: Vec<GameView>,
    },
    SettingsChanged {
        settings: Settings,
    },
    /// Overlay only: the game to follow. Sent right after the process starts.
    Target {
        target: OverlayTarget,
    },
    /// Overlay only: the user pressed the overlay hotkey.
    ToggleOverlay,
    /// Bring the window to the front (e.g. the user started overLay a second time).
    Focus,
    /// Close the window and exit.
    Close,
}

/// UI process → core.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum UiMessage {
    Request { id: u64, request: Request },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Request {
    Snapshot,
    SetLanguage {
        language: Option<String>,
    },
    SetAutostart {
        enabled: bool,
    },
    SetStartMinimized {
        enabled: bool,
    },
    SetOverlayEnabled {
        enabled: bool,
    },
    SetDemoWidget {
        enabled: bool,
    },
    /// Widget positions of a game at one resolution (`1920x1080`).
    LoadLayout {
        game_id: String,
        resolution: String,
    },
    SaveWidgetPosition {
        game_id: String,
        resolution: String,
        widget: String,
        position: Position,
    },
}

/// Positions returned by [`Request::LoadLayout`].
pub type WidgetPositions = BTreeMap<String, Position>;

/// Writes one message as a JSON line and flushes.
pub fn write_message<W: Write, T: Serialize>(writer: &mut W, message: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, message)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Reads messages until the stream ends. Lines that do not parse are reported as errors
/// and skipped, so one bad line does not end the connection.
pub fn read_messages<R: BufRead, T: DeserializeOwned>(
    reader: R,
) -> impl Iterator<Item = Result<T, String>> {
    reader
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            serde_json::from_str(&line).map_err(|e| format!("invalid message {line:?}: {e}"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_from_args() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            UiRole::from_args(args(&["overlay", "--ui", "launcher"])),
            Some(UiRole::Launcher)
        );
        assert_eq!(
            UiRole::from_args(args(&["overlay", "--ui", "overlay"])),
            Some(UiRole::Overlay)
        );
        assert_eq!(UiRole::from_args(args(&["overlay", "--minimized"])), None);
        assert_eq!(UiRole::from_args(args(&["overlay", "--ui"])), None);
    }

    #[test]
    fn messages_round_trip_as_json_lines() {
        let messages = vec![
            UiMessage::Request {
                id: 1,
                request: Request::SetLanguage {
                    language: Some("de".into()),
                },
            },
            UiMessage::Request {
                id: 2,
                request: Request::SaveWidgetPosition {
                    game_id: "hearthstone".into(),
                    resolution: "1920x1080".into(),
                    widget: "demo".into(),
                    position: Position { x: 1.0, y: 2.0 },
                },
            },
        ];
        let mut buffer = Vec::new();
        for message in &messages {
            write_message(&mut buffer, message).unwrap();
        }
        buffer.extend_from_slice(b"not json\n");
        let read: Vec<Result<UiMessage, String>> = read_messages(buffer.as_slice()).collect();
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].as_ref().unwrap(), &messages[0]);
        assert_eq!(read[1].as_ref().unwrap(), &messages[1]);
        assert!(read[2].is_err());
    }

    #[test]
    fn response_carries_errors() {
        let message = CoreMessage::Response {
            id: 7,
            result: Err("disk full".into()),
        };
        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(serde_json::from_str::<CoreMessage>(&json).unwrap(), message);
    }
}
