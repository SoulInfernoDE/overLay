// Types of the Hearthstone provider's events and game summary (`game.events`).
// They mirror `overlay_hearthstone::state` in the host.

export type Side = 'friendly' | 'opposing' | 'unknown';

/** Lowercase zone names, e.g. `deck`, `hand`, `play`, `graveyard`, `secret`, `setaside`. */
export type Zone = string;

export interface PlayerInfo {
  player_id: number;
  name: string | null;
  side: Side;
}

export type GameEvent =
  | {
      name: 'game-start';
      data: {
        /** e.g. `GT_RANKED`, `GT_BATTLEGROUNDS`, `GT_ARENA`, `GT_VS_AI`. */
        game_type: string | null;
        format: string | null;
        scenario_id: number | null;
        players: PlayerInfo[];
      };
    }
  /** Mulligan, or the hero choice in Battlegrounds. */
  | { name: 'mulligan' }
  | { name: 'turn-start'; data: { turn: number; side: Side } }
  /** A card moved; `from` is null for newly created cards. */
  | {
      name: 'zone-change';
      data: { entity_id: number; card_id: string | null; side: Side; from: Zone | null; to: Zone };
    }
  | {
      name: 'card-revealed';
      data: { entity_id: number; card_id: string; side: Side; zone: Zone | null };
    }
  | { name: 'game-end'; data: { result: 'win' | 'loss' | 'tie' | 'unknown' } }
  /** A character attacks; `side` is the attacker's. */
  | { name: 'attack'; data: { attacker: number; defender: number | null; side: Side } }
  /** A card in hand was prepared: it costs `mana` + 1 less in its owner's next turn. */
  | { name: 'prepare'; data: { entity_id: number; side: Side; mana: number } }
  /** A new game begins (the entity mirror was cleared). */
  | { name: 'game-created' };

/**
 * One entity of the game with all its tags, e.g. a minion, a hero, a player or the game
 * itself. Tag names are Hearthstone's (`ZONE`, `CONTROLLER`, `PLAYER_TECH_LEVEL`,
 * `PLAYER_LEADERBOARD_PLACE` …); unnamed tags appear as numbers.
 */
export interface Entity {
  entity_id: number;
  card_id: string | null;
  tags: Record<string, string>;
}

/** Raw provider messages; the SDK turns them into events and the entity mirror. */
export type ProviderMessage =
  | GameEvent
  | {
      name: 'entity-update';
      data: { entity_id: number; card_id: string | null; tags: Record<string, string> };
    };

export interface CardInfo {
  entity_id: number;
  card_id: string | null;
}

export interface SideInfo {
  player_id: number | null;
  /** The player's own entity (gold, tavern tier and similar tags live there). */
  player_entity_id: number | null;
  name: string | null;
  hero: CardInfo | null;
  hand: CardInfo[];
  play: CardInfo[];
  secrets: CardInfo[];
  graveyard: CardInfo[];
  deck_count: number;
}

/** Summary of the running game, for mods that start in the middle of it. */
export interface GameInfo {
  in_game: boolean;
  game_type: string | null;
  format: string | null;
  scenario_id: number | null;
  turn: number;
  current: Side | null;
  game_entity_id: number | null;
  /** The user's deck when they chose one of their own; null for loaner decks and modes without one. */
  deck: DeckChoice | null;
  friendly: SideInfo;
  opposing: SideInfo;
}

/** A deck the user chose for this game. */
export interface DeckChoice {
  name: string;
  deck_id: number | null;
  /** Deck code as Hearthstone exports it (base64 deckstring with card dbf ids). */
  code: string;
}
