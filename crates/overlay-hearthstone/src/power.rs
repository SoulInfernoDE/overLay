//! Splits `Power.log` lines into structured records.
//!
//! Hearthstone writes every change of the game twice: `GameState.DebugPrintPower()` when
//! it arrives from the server, `PowerTaskList.DebugPrintPower()` when the client plays
//! it. The two carry the same records, but the `GameState` stream runs ahead: a whole
//! Battlegrounds combat arrives within a second, before its animation even starts. The
//! game is therefore read from the `PowerTaskList` stream, so it matches what the player
//! sees. The game header (`GameState.DebugPrintGame()`: mode, players) exists only in the
//! `GameState` stream and comes before the game's `CREATE_GAME` in the `PowerTaskList`
//! stream; `GameState`'s own `CREATE_GAME` marks where a new header begins.
//!
//! Example lines:
//!
//! ```text
//! D 20:41:06.32 GameState.DebugPrintPower() - CREATE_GAME
//! D 20:41:06.32 GameState.DebugPrintGame() - GameType=GT_RANKED
//! D 20:41:06.32 GameState.DebugPrintGame() - PlayerID=1, PlayerName=Name#1234
//! D 20:41:06.32 PowerTaskList.DebugPrintPower() -     CREATE_GAME
//! D 20:41:06.32 PowerTaskList.DebugPrintPower() -     Player EntityID=2 PlayerID=1 GameAccountId=[hi=1 lo=2]
//! D 20:41:06.32 PowerTaskList.DebugPrintPower() - FULL_ENTITY - Creating ID=4 CardID=CATA_557
//! D 20:41:06.32 PowerTaskList.DebugPrintPower() -     tag=ZONE value=HAND
//! D 20:41:06.32 PowerTaskList.DebugPrintPower() - TAG_CHANGE Entity=[entityName=X id=35 zone=HAND zonePos=1 cardId= player=2] tag=ZONE value=PLAY
//! ```

/// How a line refers to an entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityRef {
    Id(u32),
    GameEntity,
    /// A player, by the name the log uses for it (BattleTag or AI name).
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    /// The header of a new game follows (`CREATE_GAME` of the `GameState` stream).
    HeaderStart,
    CreateGame,
    GameEntity {
        id: u32,
    },
    Player {
        id: u32,
        player_id: u32,
    },
    /// `FULL_ENTITY`: a new entity; `card_id` is empty for hidden cards.
    FullEntity {
        id: u32,
        card_id: Option<String>,
    },
    /// `SHOW_ENTITY`: a hidden card is revealed.
    ShowEntity {
        entity: EntityRef,
        card_id: String,
    },
    /// `CHANGE_ENTITY`: a card turns into another card.
    ChangeEntity {
        entity: EntityRef,
        card_id: String,
    },
    /// `HIDE_ENTITY`: a card becomes hidden again (e.g. shuffled back).
    HideEntity {
        entity: EntityRef,
    },
    /// `tag=… value=…` belonging to the entity of the preceding record.
    Tag {
        tag: String,
        value: String,
    },
    TagChange {
        entity: EntityRef,
        tag: String,
        value: String,
    },
    BlockStart {
        block_type: String,
        entity: EntityRef,
        target: Option<EntityRef>,
    },
    BlockEnd,
    /// Header from `DebugPrintGame`.
    GameType(String),
    FormatType(String),
    ScenarioId(u32),
    PlayerName {
        player_id: u32,
        name: String,
    },
}

/// Parses one line; `None` for lines of other streams or kinds that do not matter.
pub fn parse_line(line: &str) -> Option<Record> {
    let (method, content) = split_prefix(line)?;
    match method {
        "PowerTaskList.DebugPrintPower()" => parse_power(content.trim()),
        "GameState.DebugPrintGame()" => parse_game(content.trim()),
        "GameState.DebugPrintPower()" if content.trim() == "CREATE_GAME" => {
            Some(Record::HeaderStart)
        }
        _ => None,
    }
}

/// `D 20:41:06.3226730 PowerTaskList.DebugPrintPower() - <content>` → (method, content).
fn split_prefix(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("D ")?;
    let (_time, rest) = rest.split_once(' ')?;
    let (method, content) = rest.split_once(" - ")?;
    Some((method, content))
}

fn parse_power(content: &str) -> Option<Record> {
    if content == "CREATE_GAME" {
        return Some(Record::CreateGame);
    }
    if content == "BLOCK_END" {
        return Some(Record::BlockEnd);
    }
    if let Some(rest) = content.strip_prefix("GameEntity EntityID=") {
        return Some(Record::GameEntity {
            id: rest.trim().parse().ok()?,
        });
    }
    if let Some(rest) = content.strip_prefix("Player ") {
        return Some(Record::Player {
            id: field(rest, "EntityID=")?.parse().ok()?,
            player_id: field(rest, "PlayerID=")?.parse().ok()?,
        });
    }
    if let Some(rest) = content
        .strip_prefix("FULL_ENTITY - Creating ")
        .or_else(|| content.strip_prefix("FULL_ENTITY - Updating "))
    {
        let id = match field(rest, "ID=") {
            Some(id) => id.parse().ok()?,
            None => entity_id_in_brackets(rest)?,
        };
        return Some(Record::FullEntity {
            id,
            card_id: card_id(rest),
        });
    }
    if let Some(rest) = content.strip_prefix("SHOW_ENTITY - Updating Entity=") {
        let (entity, rest) = take_entity(rest)?;
        return Some(Record::ShowEntity {
            entity,
            card_id: card_id(rest)?,
        });
    }
    if let Some(rest) = content.strip_prefix("CHANGE_ENTITY - Updating Entity=") {
        let (entity, rest) = take_entity(rest)?;
        return Some(Record::ChangeEntity {
            entity,
            card_id: card_id(rest)?,
        });
    }
    if let Some(rest) = content.strip_prefix("HIDE_ENTITY - Entity=") {
        let (entity, _) = take_entity(rest)?;
        return Some(Record::HideEntity { entity });
    }
    if let Some(rest) = content.strip_prefix("TAG_CHANGE Entity=") {
        let (entity, rest) = take_entity_until(rest, " tag=")?;
        let (tag, value) = tag_value(rest)?;
        return Some(Record::TagChange { entity, tag, value });
    }
    if content.starts_with("tag=") {
        let (tag, value) = tag_value(content)?;
        return Some(Record::Tag { tag, value });
    }
    if let Some(rest) = content.strip_prefix("BLOCK_START BlockType=") {
        let (block_type, rest) = rest.split_once(' ')?;
        let rest = rest.strip_prefix("Entity=")?;
        let (entity, rest) = take_entity_until(rest, " EffectCardId=")?;
        let target = rest
            .split_once(" Target=")
            .and_then(|(_, t)| take_entity_until(t, " SubOption=").map(|(e, _)| e))
            .filter(|e| *e != EntityRef::Id(0));
        return Some(Record::BlockStart {
            block_type: block_type.to_owned(),
            entity,
            target,
        });
    }
    None
}

fn parse_game(content: &str) -> Option<Record> {
    if let Some(value) = content.strip_prefix("GameType=") {
        return Some(Record::GameType(value.trim().to_owned()));
    }
    if let Some(value) = content.strip_prefix("FormatType=") {
        return Some(Record::FormatType(value.trim().to_owned()));
    }
    if let Some(value) = content.strip_prefix("ScenarioID=") {
        return Some(Record::ScenarioId(value.trim().parse().ok()?));
    }
    if let Some(rest) = content.strip_prefix("PlayerID=") {
        let (id, rest) = rest.split_once(", PlayerName=")?;
        return Some(Record::PlayerName {
            player_id: id.trim().parse().ok()?,
            name: rest.trim().to_owned(),
        });
    }
    None
}

/// The value after `key` up to the next space.
fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let start = find_key(text, key)? + key.len();
    let rest = &text[start..];
    Some(rest.split(' ').next().unwrap_or(rest))
}

/// Finds `key` at the start of `text` or after a space, so `ID=` does not match `EntityID=`.
fn find_key(text: &str, key: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(pos) = text[from..].find(key) {
        let at = from + pos;
        if at == 0 || text.as_bytes()[at - 1] == b' ' {
            return Some(at);
        }
        from = at + key.len();
    }
    None
}

fn card_id(text: &str) -> Option<String> {
    field(text, "CardID=")
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
}

fn tag_value(text: &str) -> Option<(String, String)> {
    let rest = text.trim_start().strip_prefix("tag=")?;
    let (tag, rest) = rest.split_once(" value=")?;
    let value = rest.split(' ').next().unwrap_or(rest).trim();
    Some((tag.to_owned(), value.to_owned()))
}

/// Reads an entity reference at the start of `text` and returns the rest.
fn take_entity(text: &str) -> Option<(EntityRef, &str)> {
    if text.starts_with('[') {
        let end = matching_bracket(text)?;
        let id = entity_id_in_brackets(&text[..=end])?;
        return Some((EntityRef::Id(id), &text[end + 1..]));
    }
    let (word, rest) = text.split_once(' ').unwrap_or((text, ""));
    Some((plain_ref(word), rest))
}

/// Like [`take_entity`], but a plain reference (a player name) may contain spaces and
/// ends at `terminator`.
fn take_entity_until<'a>(text: &'a str, terminator: &str) -> Option<(EntityRef, &'a str)> {
    if text.starts_with('[') {
        let (entity, rest) = take_entity(text)?;
        return Some((entity, rest));
    }
    let end = text.find(terminator)?;
    Some((plain_ref(text[..end].trim()), &text[end..]))
}

fn plain_ref(word: &str) -> EntityRef {
    if word == "GameEntity" {
        EntityRef::GameEntity
    } else if let Ok(id) = word.parse() {
        EntityRef::Id(id)
    } else {
        EntityRef::Name(word.to_owned())
    }
}

/// Index of the `]` closing the `[` at the start; names may contain nested brackets
/// (`UNKNOWN ENTITY [cardType=INVALID]`).
fn matching_bracket(text: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn entity_id_in_brackets(text: &str) -> Option<u32> {
    // `id=` follows the (possibly odd) entity name; take the last match to be safe.
    let mut found = None;
    let mut from = 0;
    while let Some(pos) = text[from..].find(" id=") {
        let start = from + pos + " id=".len();
        let digits: String = text[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(id) = digits.parse() {
            found = Some(id);
        }
        from = start;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power(content: &str) -> String {
        format!("D 20:41:06.3226730 PowerTaskList.DebugPrintPower() - {content}")
    }

    #[test]
    fn parses_entities() {
        assert_eq!(parse_line(&power("CREATE_GAME")), Some(Record::CreateGame));
        assert_eq!(
            parse_line(&power("    GameEntity EntityID=1")),
            Some(Record::GameEntity { id: 1 })
        );
        assert_eq!(
            parse_line(&power(
                "    Player EntityID=2 PlayerID=1 GameAccountId=[hi=1 lo=2]"
            )),
            Some(Record::Player {
                id: 2,
                player_id: 1
            })
        );
        assert_eq!(
            parse_line(&power("FULL_ENTITY - Creating ID=4 CardID=CATA_557")),
            Some(Record::FullEntity {
                id: 4,
                card_id: Some("CATA_557".into())
            })
        );
        assert_eq!(
            parse_line(&power("FULL_ENTITY - Creating ID=5 CardID=")),
            Some(Record::FullEntity {
                id: 5,
                card_id: None
            })
        );
        assert_eq!(
            parse_line(&power("        tag=ZONE value=HAND")),
            Some(Record::Tag {
                tag: "ZONE".into(),
                value: "HAND".into()
            })
        );
    }

    #[test]
    fn parses_entity_references() {
        let line = power(
            "    SHOW_ENTITY - Updating Entity=[entityName=UNKNOWN ENTITY [cardType=INVALID] id=35 zone=HAND zonePos=1 cardId= player=2] CardID=FP1_005",
        );
        assert_eq!(
            parse_line(&line),
            Some(Record::ShowEntity {
                entity: EntityRef::Id(35),
                card_id: "FP1_005".into()
            })
        );
        assert_eq!(
            parse_line(&power(
                "TAG_CHANGE Entity=GameEntity tag=STEP value=MAIN_READY "
            )),
            Some(Record::TagChange {
                entity: EntityRef::GameEntity,
                tag: "STEP".into(),
                value: "MAIN_READY".into()
            })
        );
        assert_eq!(
            parse_line(&power(
                "    TAG_CHANGE Entity=4 tag=NEXT_STEP value=BEGIN_MULLIGAN "
            )),
            Some(Record::TagChange {
                entity: EntityRef::Id(4),
                tag: "NEXT_STEP".into(),
                value: "BEGIN_MULLIGAN".into()
            })
        );
        assert_eq!(
            parse_line(&power(
                "    TAG_CHANGE Entity=The Lich King tag=CURRENT_PLAYER value=1 "
            )),
            Some(Record::TagChange {
                entity: EntityRef::Name("The Lich King".into()),
                tag: "CURRENT_PLAYER".into(),
                value: "1".into()
            })
        );
    }

    #[test]
    fn parses_blocks() {
        let line = power(
            "BLOCK_START BlockType=ATTACK Entity=[entityName=Pfeilmacher id=10 zone=PLAY zonePos=1 cardId=TIME_606 player=1] EffectCardId=System.Collections.Generic.List`1[System.String] EffectIndex=0 Target=[entityName=Krabbler id=54 zone=PLAY zonePos=1 cardId=FP1_002 player=2] SubOption=-1 ",
        );
        assert_eq!(
            parse_line(&line),
            Some(Record::BlockStart {
                block_type: "ATTACK".into(),
                entity: EntityRef::Id(10),
                target: Some(EntityRef::Id(54))
            })
        );
        let line = power(
            "BLOCK_START BlockType=TRIGGER Entity=GameEntity EffectCardId=System.Collections.Generic.List`1[System.String] EffectIndex=-1 Target=0 SubOption=-1 TriggerKeyword=TAG_NOT_SET",
        );
        assert_eq!(
            parse_line(&line),
            Some(Record::BlockStart {
                block_type: "TRIGGER".into(),
                entity: EntityRef::GameEntity,
                target: None
            })
        );
    }

    #[test]
    fn parses_the_game_header() {
        let game = |c: &str| format!("D 20:41:06.32 GameState.DebugPrintGame() - {c}");
        assert_eq!(
            parse_line(&game("GameType=GT_BATTLEGROUNDS")),
            Some(Record::GameType("GT_BATTLEGROUNDS".into()))
        );
        assert_eq!(
            parse_line(&game("PlayerID=2, PlayerName=Some Name#1234")),
            Some(Record::PlayerName {
                player_id: 2,
                name: "Some Name#1234".into()
            })
        );
    }

    #[test]
    fn reads_the_game_from_the_power_task_list() {
        let ahead = |c: &str| format!("D 20:41:06.32 GameState.DebugPrintPower() - {c}");
        assert_eq!(parse_line(&ahead("CREATE_GAME")), Some(Record::HeaderStart));
        assert_eq!(
            parse_line(&ahead("TAG_CHANGE Entity=GameEntity tag=TURN value=2")),
            None
        );
        assert_eq!(
            parse_line(&power("    CREATE_GAME")),
            Some(Record::CreateGame)
        );
        assert_eq!(
            parse_line("D 20:41:06.32 PowerTaskList.DebugDump() - ID=1 ParentID=0"),
            None
        );
        assert_eq!(parse_line("garbage"), None);
    }
}
