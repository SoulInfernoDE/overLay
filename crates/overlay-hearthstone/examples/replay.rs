//! Runs a `Power.log` through the provider and prints a summary (no player names), to
//! check the parser against real games locally:
//!
//!     cargo run -p overlay-hearthstone --example replay -- path/to/Power.log

use std::collections::BTreeMap;

use overlay_hearthstone::power::parse_line;
use overlay_hearthstone::{GameEvent, GameState};

fn main() {
    let path = std::env::args().nth(1).expect("usage: replay <Power.log>");
    let text = std::fs::read_to_string(&path).expect("readable log");
    let mut state = GameState::default();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut records = 0;
    for record in text.lines().filter_map(parse_line) {
        records += 1;
        for event in state.apply(record) {
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
    println!("friendly: {}", summary(&info.friendly));
    println!("opposing: {}", summary(&info.opposing));
}
