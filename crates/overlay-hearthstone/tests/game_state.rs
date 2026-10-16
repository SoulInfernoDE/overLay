//! A short, hand-written game through the parser and the game model.

use overlay_hearthstone::power::parse_line;
use overlay_hearthstone::state::{GameEvent, GameResult, GameState, Side};

fn run(log: &str) -> (GameState, Vec<GameEvent>) {
    let mut state = GameState::default();
    let mut events = Vec::new();
    for record in log.lines().filter_map(parse_line) {
        events.extend(state.apply(record));
    }
    (state, events)
}

#[test]
fn follows_a_constructed_game() {
    let (state, events) = run(include_str!("fixtures/constructed.log"));

    let GameEvent::GameStart {
        game_type, players, ..
    } = &events
        .iter()
        .find(|e| matches!(e, GameEvent::GameStart { .. }))
        .expect("game start")
    else {
        unreachable!()
    };
    assert_eq!(game_type.as_deref(), Some("GT_RANKED"));
    assert_eq!(players[0].side, Side::Friendly);
    assert_eq!(players[1].side, Side::Opposing);

    assert!(events.contains(&GameEvent::Mulligan));
    assert!(events.contains(&GameEvent::TurnStart {
        turn: 1,
        side: Side::Friendly
    }));
    assert!(events.contains(&GameEvent::TurnStart {
        turn: 2,
        side: Side::Opposing
    }));
    // Draw of a hidden card.
    assert!(events.contains(&GameEvent::ZoneChange {
        entity_id: 5,
        card_id: None,
        side: Side::Friendly,
        from: Some("deck".into()),
        to: "hand".into(),
    }));
    // Friendly card played.
    assert!(events.contains(&GameEvent::ZoneChange {
        entity_id: 4,
        card_id: Some("CS2_029".into()),
        side: Side::Friendly,
        from: Some("hand".into()),
        to: "play".into(),
    }));
    // Opponent card revealed and played.
    assert!(events.contains(&GameEvent::CardRevealed {
        entity_id: 6,
        card_id: "CS2_182".into(),
        side: Side::Opposing,
        zone: Some("play".into()),
    }));
    assert!(events.contains(&GameEvent::ZoneChange {
        entity_id: 6,
        card_id: Some("CS2_182".into()),
        side: Side::Opposing,
        from: Some("hand".into()),
        to: "play".into(),
    }));
    assert_eq!(
        events.last(),
        Some(&GameEvent::GameEnd {
            result: GameResult::Win
        })
    );

    let info = state.info();
    assert!(!info.in_game);
    assert_eq!(info.turn, 2);
    assert_eq!(info.friendly.name.as_deref(), Some("Friendly#1234"));
    assert_eq!(
        info.friendly
            .hero
            .as_ref()
            .and_then(|h| h.card_id.as_deref()),
        Some("HERO_08")
    );
    assert_eq!(info.friendly.hand.len(), 1);
    assert_eq!(info.opposing.play.len(), 1);
}

#[test]
fn unknown_names_are_not_guessed_in_bigger_games() {
    // Battlegrounds-like: more than two players, a name that is not in the header.
    let log = "\
D 10:00:00.0 GameState.DebugPrintPower() - CREATE_GAME
D 10:00:00.0 GameState.DebugPrintPower() -     GameEntity EntityID=1
D 10:00:00.0 GameState.DebugPrintPower() -     Player EntityID=2 PlayerID=1 GameAccountId=[hi=1 lo=1]
D 10:00:00.0 GameState.DebugPrintPower() -     Player EntityID=3 PlayerID=2 GameAccountId=[hi=2 lo=2]
D 10:00:00.0 GameState.DebugPrintPower() -     Player EntityID=4 PlayerID=3 GameAccountId=[hi=3 lo=3]
D 10:00:01.0 GameState.DebugPrintPower() - TAG_CHANGE Entity=Bob tag=CURRENT_PLAYER value=1
";
    let (state, _) = run(log);
    assert_eq!(state.info().current, None);
}
