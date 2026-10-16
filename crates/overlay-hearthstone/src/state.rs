//! The game model built from `Power.log` records, and the events mods receive.
//!
//! Every entity keeps all of its tags, not only the ones the events below use. Mods for
//! other modes (Battlegrounds tavern, leaderboard, buddies …) are derived from the same
//! model later, without changing the parser.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::power::{EntityRef, Record};

/// Which side of the table an entity belongs to, seen from the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    Friendly,
    Opposing,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameResult {
    Win,
    Loss,
    Tie,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerInfo {
    pub player_id: u32,
    pub name: Option<String>,
    pub side: Side,
}

/// Events delivered to mods with the `game.events` permission, as `{ name, data }`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "name", content = "data", rename_all = "kebab-case")]
pub enum GameEvent {
    /// The game has its header (mode, players) and starts.
    GameStart {
        game_type: Option<String>,
        format: Option<String>,
        scenario_id: Option<u32>,
        players: Vec<PlayerInfo>,
    },
    /// The mulligan (or, in Battlegrounds, the hero choice) begins.
    Mulligan,
    TurnStart {
        turn: u32,
        side: Side,
    },
    /// A card moved between zones; `from` is `None` for newly created cards. Zones are
    /// lowercase: `deck`, `hand`, `play`, `graveyard`, `secret`, `setaside` …
    ZoneChange {
        entity_id: u32,
        card_id: Option<String>,
        side: Side,
        from: Option<String>,
        to: String,
    },
    /// A hidden card became known (or turned into another card).
    CardRevealed {
        entity_id: u32,
        card_id: String,
        side: Side,
        zone: Option<String>,
    },
    GameEnd {
        result: GameResult,
    },
}

/// A card as part of [`GameInfo`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CardInfo {
    pub entity_id: u32,
    pub card_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SideInfo {
    pub player_id: Option<u32>,
    pub name: Option<String>,
    pub hero: Option<CardInfo>,
    pub hand: Vec<CardInfo>,
    pub play: Vec<CardInfo>,
    pub secrets: Vec<CardInfo>,
    pub graveyard: Vec<CardInfo>,
    pub deck_count: usize,
}

/// A summary of the current game for mods that start in the middle of it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct GameInfo {
    pub in_game: bool,
    pub game_type: Option<String>,
    pub format: Option<String>,
    pub scenario_id: Option<u32>,
    pub turn: u32,
    pub current: Option<Side>,
    pub friendly: SideInfo,
    pub opposing: SideInfo,
}

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub id: u32,
    pub card_id: Option<String>,
    pub tags: HashMap<String, String>,
}

impl Entity {
    pub fn tag(&self, tag: &str) -> Option<&str> {
        self.tags.get(tag).map(String::as_str)
    }

    fn number(&self, tag: &str) -> Option<u32> {
        self.tag(tag)?.parse().ok()
    }

    fn is_card(&self) -> bool {
        !matches!(
            self.tag("CARDTYPE"),
            Some("GAME" | "PLAYER" | "ENCHANTMENT" | "MOVE_MINION_HOVER_TARGET")
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingKind {
    Created,
    Revealed,
    Plain,
}

/// The entity that the following `tag=` lines describe.
#[derive(Clone, Debug)]
struct Pending {
    id: u32,
    kind: PendingKind,
    zone_before: Option<String>,
}

#[derive(Debug, Default)]
pub struct GameState {
    entities: HashMap<u32, Entity>,
    /// Player id → entity id.
    players: BTreeMap<u32, u32>,
    /// Names from the game header, by player id.
    header_names: BTreeMap<u32, String>,
    /// Names the log uses in entity references, by player id.
    ref_names: HashMap<String, u32>,
    game_entity: Option<u32>,
    friendly: Option<u32>,
    game_type: Option<String>,
    format: Option<String>,
    scenario_id: Option<u32>,
    in_game: bool,
    started: bool,
    ended: bool,
    turn: u32,
    current_player: Option<u32>,
    pending: Option<Pending>,
}

impl GameState {
    /// Applies one record and returns the events it causes.
    pub fn apply(&mut self, record: Record) -> Vec<GameEvent> {
        let mut events = Vec::new();
        if !matches!(record, Record::Tag { .. }) {
            self.finish_pending(&mut events);
        }
        match record {
            Record::CreateGame => {
                *self = GameState {
                    in_game: true,
                    ..GameState::default()
                };
            }
            Record::GameEntity { id } => {
                self.game_entity = Some(id);
                self.entity_mut(id);
                self.pending = Some(Pending {
                    id,
                    kind: PendingKind::Plain,
                    zone_before: None,
                });
            }
            Record::Player { id, player_id } => {
                self.players.insert(player_id, id);
                self.entity_mut(id);
                self.pending = Some(Pending {
                    id,
                    kind: PendingKind::Plain,
                    zone_before: None,
                });
            }
            Record::FullEntity { id, card_id } => {
                let entity = self.entity_mut(id);
                let zone_before = entity.tag("ZONE").map(str::to_owned);
                if card_id.is_some() {
                    entity.card_id = card_id;
                }
                self.pending = Some(Pending {
                    id,
                    kind: PendingKind::Created,
                    zone_before,
                });
            }
            Record::ShowEntity { entity, card_id } | Record::ChangeEntity { entity, card_id } => {
                if let Some(id) = self.resolve(&entity) {
                    let entity = self.entity_mut(id);
                    let zone_before = entity.tag("ZONE").map(str::to_owned);
                    entity.card_id = Some(card_id);
                    self.pending = Some(Pending {
                        id,
                        kind: PendingKind::Revealed,
                        zone_before,
                    });
                }
            }
            Record::HideEntity { entity } => {
                if let Some(id) = self.resolve(&entity) {
                    let zone_before = self.entity_mut(id).tag("ZONE").map(str::to_owned);
                    self.pending = Some(Pending {
                        id,
                        kind: PendingKind::Plain,
                        zone_before,
                    });
                }
            }
            Record::Tag { tag, value } => {
                if let Some(pending) = &self.pending {
                    let id = pending.id;
                    self.entity_mut(id).tags.insert(tag, value);
                }
            }
            Record::TagChange { entity, tag, value } => {
                if let Some(id) = self.resolve(&entity) {
                    self.tag_change(id, tag, value, &mut events);
                }
            }
            Record::GameType(value) => self.game_type = Some(value),
            Record::FormatType(value) => self.format = Some(value),
            Record::ScenarioId(value) => self.scenario_id = Some(value),
            Record::PlayerName { player_id, name } => {
                self.header_names.insert(player_id, name);
            }
            Record::BlockStart { .. } | Record::BlockEnd => {}
        }
        events
    }

    /// Completes the entity whose `tag=` lines just ended.
    fn finish_pending(&mut self, events: &mut Vec<GameEvent>) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let Some(entity) = self.entities.get(&pending.id) else {
            return;
        };
        if !entity.is_card() {
            return;
        }
        let zone = entity.tag("ZONE").map(str::to_owned);
        let card_id = entity.card_id.clone();
        let controller = entity.number("CONTROLLER");
        // The user's own starting cards are the only ones created face up in the hand.
        if pending.kind == PendingKind::Created
            && self.friendly.is_none()
            && zone.as_deref() == Some("HAND")
            && card_id.is_some()
        {
            self.friendly = controller;
        }
        let side = self.side(controller);
        if pending.kind == PendingKind::Revealed
            && let Some(card_id) = &card_id
        {
            events.push(GameEvent::CardRevealed {
                entity_id: pending.id,
                card_id: card_id.clone(),
                side,
                zone: zone.as_deref().map(str::to_lowercase),
            });
        }
        if let Some(to) = zone
            && (pending.kind == PendingKind::Created || pending.zone_before.as_ref() != Some(&to))
        {
            events.push(GameEvent::ZoneChange {
                entity_id: pending.id,
                card_id,
                side,
                from: pending.zone_before.as_deref().map(str::to_lowercase),
                to: to.to_lowercase(),
            });
        }
    }

    fn tag_change(&mut self, id: u32, tag: String, value: String, events: &mut Vec<GameEvent>) {
        let is_game = Some(id) == self.game_entity;
        let player_id = self.player_id_of(id);
        let entity = self.entity_mut(id);
        let previous = entity.tags.insert(tag.clone(), value.clone());
        let is_card = entity.is_card();
        let card_id = entity.card_id.clone();
        let controller = entity.number("CONTROLLER");

        if is_game {
            match tag.as_str() {
                "STEP" => {
                    if !self.started {
                        self.started = true;
                        events.push(self.game_start());
                    }
                    match value.as_str() {
                        "BEGIN_MULLIGAN" => events.push(GameEvent::Mulligan),
                        "MAIN_READY" => events.push(GameEvent::TurnStart {
                            turn: self.turn,
                            side: self.side(self.current_player),
                        }),
                        _ => {}
                    }
                }
                "TURN" => self.turn = value.parse().unwrap_or(self.turn),
                "STATE" if value == "COMPLETE" && !self.ended => {
                    self.ended = true;
                    self.in_game = false;
                    events.push(GameEvent::GameEnd {
                        result: self.result(),
                    });
                }
                _ => {}
            }
            return;
        }
        if let Some(player_id) = player_id {
            if tag == "CURRENT_PLAYER" && value == "1" {
                self.current_player = Some(player_id);
            }
            return;
        }
        if tag == "ZONE" && is_card && previous.as_ref() != Some(&value) {
            events.push(GameEvent::ZoneChange {
                entity_id: id,
                card_id,
                side: self.side(controller),
                from: previous.map(|z| z.to_lowercase()),
                to: value.to_lowercase(),
            });
        }
    }

    fn game_start(&self) -> GameEvent {
        GameEvent::GameStart {
            game_type: self.game_type.clone(),
            format: self.format.clone(),
            scenario_id: self.scenario_id,
            players: self
                .players
                .keys()
                .map(|&player_id| PlayerInfo {
                    player_id,
                    name: self.header_names.get(&player_id).cloned(),
                    side: self.side(Some(player_id)),
                })
                .collect(),
        }
    }

    fn result(&self) -> GameResult {
        let Some(entity) = self
            .friendly
            .and_then(|p| self.players.get(&p))
            .and_then(|id| self.entities.get(id))
        else {
            return GameResult::Unknown;
        };
        match entity.tag("PLAYSTATE") {
            Some("WON") => GameResult::Win,
            Some("LOST" | "CONCEDED") => GameResult::Loss,
            Some("TIED") => GameResult::Tie,
            _ => GameResult::Unknown,
        }
    }

    fn side(&self, player_id: Option<u32>) -> Side {
        match (player_id, self.friendly) {
            (Some(p), Some(f)) if p == f => Side::Friendly,
            (Some(_), Some(_)) => Side::Opposing,
            _ => Side::Unknown,
        }
    }

    fn player_id_of(&self, entity_id: u32) -> Option<u32> {
        self.players
            .iter()
            .find(|(_, id)| **id == entity_id)
            .map(|(player_id, _)| *player_id)
    }

    fn entity_mut(&mut self, id: u32) -> &mut Entity {
        self.entities.entry(id).or_insert_with(|| Entity {
            id,
            ..Entity::default()
        })
    }

    /// Entity id for a reference. Player names are matched to players:
    /// 1. a name from the game header belongs to that player;
    /// 2. in a two-player game, any other name belongs to the opponent once the user's
    ///    side is known (the user always appears under the header name);
    /// 3. otherwise only if exactly one player has no name yet.
    ///
    /// Anything else stays unresolved rather than guessed: Battlegrounds has more
    /// player-like names than player entities.
    fn resolve(&mut self, entity: &EntityRef) -> Option<u32> {
        let name = match entity {
            EntityRef::Id(id) => return Some(*id),
            EntityRef::GameEntity => return self.game_entity,
            EntityRef::Name(name) => name,
        };
        if let Some(player_id) = self.ref_names.get(name) {
            return self.players.get(player_id).copied();
        }
        let from_header = self
            .header_names
            .iter()
            .find(|(_, n)| *n == name)
            .map(|(id, _)| *id);
        let player_id = from_header.or_else(|| {
            if self.players.len() != 2 {
                return None;
            }
            if let Some(friendly) = self.friendly {
                return self.players.keys().copied().find(|p| *p != friendly);
            }
            let unnamed: Vec<u32> = self
                .players
                .keys()
                .copied()
                .filter(|p| !self.ref_names.values().any(|v| v == p))
                .collect();
            (unnamed.len() == 1).then(|| unnamed[0])
        })?;
        self.ref_names.insert(name.clone(), player_id);
        self.players.get(&player_id).copied()
    }

    pub fn entity(&self, id: u32) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Summary for mods that join mid-game.
    pub fn info(&self) -> GameInfo {
        let side_info = |side: Side| -> SideInfo {
            let player_id = match side {
                Side::Friendly => self.friendly,
                _ => self
                    .players
                    .keys()
                    .copied()
                    .find(|p| Some(*p) != self.friendly),
            };
            let mut info = SideInfo {
                player_id,
                name: player_id.and_then(|p| self.header_names.get(&p).cloned()),
                ..SideInfo::default()
            };
            let Some(player_id) = player_id else {
                return info;
            };
            let mut cards: Vec<&Entity> = self
                .entities
                .values()
                .filter(|e| e.is_card() && e.number("CONTROLLER") == Some(player_id))
                .collect();
            cards.sort_by_key(|e| (e.number("ZONE_POSITION").unwrap_or(0), e.id));
            for entity in cards {
                let card = CardInfo {
                    entity_id: entity.id,
                    card_id: entity.card_id.clone(),
                };
                match (entity.tag("ZONE"), entity.tag("CARDTYPE")) {
                    (Some("DECK"), _) => info.deck_count += 1,
                    (Some("HAND"), _) => info.hand.push(card),
                    (Some("PLAY"), Some("HERO")) => info.hero = Some(card),
                    (Some("PLAY"), _) => info.play.push(card),
                    (Some("SECRET"), _) => info.secrets.push(card),
                    (Some("GRAVEYARD"), _) => info.graveyard.push(card),
                    _ => {}
                }
            }
            info
        };
        GameInfo {
            in_game: self.in_game,
            game_type: self.game_type.clone(),
            format: self.format.clone(),
            scenario_id: self.scenario_id,
            turn: self.turn,
            current: self.current_player.map(|p| self.side(Some(p))),
            friendly: side_info(Side::Friendly),
            opposing: side_info(Side::Opposing),
        }
    }
}
