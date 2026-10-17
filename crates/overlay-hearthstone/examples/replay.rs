//! Runs a `Power.log` through the provider and prints a summary (no player names), to
//! check the parser against real games locally:
//!
//!     cargo run -p overlay-hearthstone --example replay -- path/to/Power.log [entities.json] [events.json]
//!
//! With a second path, all entities at the end of the log are written there as JSON;
//! with a third, the game events as mods receive them (without the many entity updates).
//! Pass `-` to skip the entities.

use std::collections::BTreeMap;

use overlay_hearthstone::power::parse_line;
use overlay_hearthstone::{GameEvent, GameState};

fn main() {
    let path = std::env::args().nth(1).expect("usage: replay <Power.log>");
    let text = std::fs::read_to_string(&path).expect("readable log");
    let mut state = GameState::default();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut records = 0;
    let mut kept = Vec::new();
    for record in text.lines().filter_map(parse_line) {
        records += 1;
        for event in state.apply(record) {
            if !matches!(event, GameEvent::EntityUpdate { .. }) {
                kept.push(event.clone());
            }
            let kind = match &event {
                GameEvent::GameStart { game_type, .. } => {
                    println!("game start: {}", game_type.as_deref().unwrap_or("?"));
                    "game-start"
                }
                GameEvent::Mulligan => "mulligan",
                GameEvent::TurnStart { .. } => "turn-start",
                GameEvent::ZoneChange { .. } => "zone-change",
                GameEvent::CardRevealed { .. } => "card-revealed",
                GameEvent::GameEnd { result } => {
                    println!("game end: {result:?}");
                    "game-end"
                }
                GameEvent::Attack { .. } => "attack",
                GameEvent::GameCreated => "game-created",
                GameEvent::EntityUpdate { .. } => "entity-update",
            };
            *counts.entry(kind).or_default() += 1;
        }
    }
    println!("records: {records}");
    for (kind, count) in counts {
        println!("  {kind}: {count}");
    }
    let info = state.info();
    let summary = |side: &overlay_hearthstone::state::SideInfo| {
        format!(
            "player {:?}, hero {:?}, hand {}, play {}, secrets {}, graveyard {}, deck {}",
            side.player_id,
            side.hero.as_ref().and_then(|h| h.card_id.clone()),
            side.hand.len(),
            side.play.len(),
            side.secrets.len(),
            side.graveyard.len(),
            side.deck_count
        )
    };
    println!(
        "in game: {}, turn {}, current {:?}",
        info.in_game, info.turn, info.current
    );
    println!("entities: {}", state.entities_snapshot().len());
    if let Some(out) = std::env::args().nth(2).filter(|p| p != "-") {
        let json = serde_json::to_string_pretty(&state.entities_snapshot()).expect("serializable");
        std::fs::write(&out, json).expect("writable output");
        println!("entities written to {out}");
    }
    if let Some(out) = std::env::args().nth(3) {
        let json = serde_json::to_string_pretty(&kept).expect("serializable");
        std::fs::write(&out, json).expect("writable output");
        println!("{} events written to {out}", kept.len());
    }
    println!("friendly: {}", summary(&info.friendly));
    println!("opposing: {}", summary(&info.opposing));
}
