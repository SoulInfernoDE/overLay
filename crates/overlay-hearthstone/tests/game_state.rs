//! A short, hand-written game through the parser and the game model. Like a real log,
//! the fixture has the game twice: the `GameState` stream first, then the
//! `PowerTaskList` stream the parser reads.

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

#[test]
fn the_next_header_does_not_change_the_running_game() {
    // The next game's header can arrive while this game's animations still play.
    let log = format!(
        "{}{}",
        include_str!("fixtures/constructed.log"),
        "D 10:05:00.0000000 GameState.DebugPrintPower() - CREATE_GAME\n\
         D 10:05:00.0000000 GameState.DebugPrintGame() - GameType=GT_BATTLEGROUNDS\n"
    );
    let (state, _) = run(&log);
    assert_eq!(state.info().game_type.as_deref(), Some("GT_RANKED"));

    let next =
        format!("{log}D 10:05:00.0000000 PowerTaskList.DebugPrintPower() -     CREATE_GAME\n");
    let (state, events) = run(&next);
    assert_eq!(events.last(), Some(&GameEvent::GameCreated));
    assert_eq!(state.info().game_type.as_deref(), Some("GT_BATTLEGROUNDS"));
}

#[test]
fn changes_of_a_name_used_before_it_can_be_matched_are_kept() {
    // Battlegrounds names the other side after the innkeeper, not as in the header, and
    // uses that name before the user's, when it cannot be matched yet.
    let power = |c: &str| format!("D 10:00:00.0 PowerTaskList.DebugPrintPower() - {c}\n");
    let log = [
        "D 10:00:00.0 GameState.DebugPrintPower() - CREATE_GAME\n".to_owned(),
        "D 10:00:00.0 GameState.DebugPrintGame() - PlayerID=1, PlayerName=Me#1234\n".into(),
        "D 10:00:00.0 GameState.DebugPrintGame() - PlayerID=2, PlayerName=Someone\n".into(),
        power("    CREATE_GAME"),
        power("    GameEntity EntityID=1"),
        power("    Player EntityID=2 PlayerID=1 GameAccountId=[hi=1 lo=1]"),
        power("    Player EntityID=3 PlayerID=2 GameAccountId=[hi=0 lo=0]"),
        power("    TAG_CHANGE Entity=Innkeeper tag=FIRST_PLAYER value=1 "),
        power("    TAG_CHANGE Entity=Me#1234 tag=NUM_TURNS_LEFT value=1 "),
        power("    TAG_CHANGE Entity=Innkeeper tag=NUM_TURNS_LEFT value=1 "),
    ]
    .concat();
    let (state, _) = run(&log);
    let innkeeper = state.entity(3).expect("player entity");
    assert_eq!(innkeeper.tag("FIRST_PLAYER"), Some("1"));
    assert_eq!(innkeeper.tag("NUM_TURNS_LEFT"), Some("1"));
    assert_eq!(state.entity(2).unwrap().tag("FIRST_PLAYER"), None);
}

#[test]
fn the_user_is_the_player_whose_draws_are_shown() {
    // The user goes first, so the opponent gets the Coin, created face up in their hand.
    let power = |c: &str| format!("D 10:00:00.0 PowerTaskList.DebugPrintPower() - {c}\n");
    let before = [
        power("    CREATE_GAME"),
        power("    GameEntity EntityID=1"),
        power("    Player EntityID=2 PlayerID=1 GameAccountId=[hi=1 lo=1]"),
        power("    Player EntityID=3 PlayerID=2 GameAccountId=[hi=2 lo=2]"),
        power("FULL_ENTITY - Creating ID=4 CardID="),
        power("    tag=CONTROLLER value=1"),
        power("    tag=ZONE value=DECK"),
        power("FULL_ENTITY - Creating ID=5 CardID="),
        power("    tag=CONTROLLER value=2"),
        power("    tag=ZONE value=DECK"),
        // The opponent draws first; their card stays hidden.
        power("TAG_CHANGE Entity=5 tag=ZONE value=HAND"),
    ]
    .concat();
    let (state, _) = run(&before);
    assert_eq!(state.info().opposing.player_entity_id, None);

    let after = [
        before,
        power("SHOW_ENTITY - Updating Entity=[entityName=UNKNOWN ENTITY [cardType=INVALID] id=4 zone=DECK zonePos=0 cardId= player=1] CardID=CS2_029"),
        power("    tag=CONTROLLER value=1"),
        power("    tag=CARDTYPE value=SPELL"),
        power("TAG_CHANGE Entity=4 tag=ZONE value=HAND"),
        power("FULL_ENTITY - Creating ID=6 CardID=GAME_005"),
        power("    tag=CONTROLLER value=2"),
        power("    tag=CARDTYPE value=SPELL"),
        power("    tag=ZONE value=HAND"),
    ]
    .concat();
    let (state, _) = run(&after);
    let info = state.info();
    assert_eq!(info.friendly.player_entity_id, Some(2));
    assert_eq!(info.opposing.player_entity_id, Some(3));
}

#[test]
fn attacks_become_events() {
    let power = |c: &str| format!("D 10:00:00.0 PowerTaskList.DebugPrintPower() - {c}\n");
    let log = [
        power("    CREATE_GAME"),
        power("    GameEntity EntityID=1"),
        power("    Player EntityID=2 PlayerID=1 GameAccountId=[hi=1 lo=1]"),
        power("    Player EntityID=3 PlayerID=2 GameAccountId=[hi=2 lo=2]"),
        power("FULL_ENTITY - Creating ID=10 CardID=CS2_182"),
        power("    tag=CONTROLLER value=1"),
        power("    tag=CARDTYPE value=MINION"),
        power("    tag=ZONE value=PLAY"),
        power("FULL_ENTITY - Creating ID=11 CardID=HERO_08"),
        power("    tag=CONTROLLER value=2"),
        power("    tag=CARDTYPE value=HERO"),
        power("    tag=ZONE value=PLAY"),
        power("BLOCK_START BlockType=ATTACK Entity=[entityName=Ogre id=10 zone=PLAY zonePos=1 cardId=CS2_182 player=1] EffectCardId=System.Collections.Generic.List`1[System.String] EffectIndex=0 Target=[entityName=Jaina id=11 zone=PLAY zonePos=0 cardId=HERO_08 player=2] SubOption=-1 "),
        power("BLOCK_END"),
    ]
    .concat();
    let (_, events) = run(&log);
    let attack = events
        .iter()
        .find(|e| matches!(e, GameEvent::Attack { .. }))
        .expect("attack event");
    assert!(matches!(
        attack,
        GameEvent::Attack {
            attacker: 10,
            defender: Some(11),
            ..
        }
    ));
}
