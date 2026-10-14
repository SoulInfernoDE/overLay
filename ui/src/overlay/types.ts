// Mirrors `overlay_ipc::OverlayTarget` and the overlay process's `GameWindow`.

export interface OverlayTarget {
  game_id: string;
  game_name: string;
  pid: number;
}

/** The game window's client area in physical pixels. */
export interface GameWindow {
  x: number;
  y: number;
  width: number;
  height: number;
  hidden: boolean;
  active: boolean;
}

export interface OverlayStatus {
  target: OverlayTarget | null;
  window: GameWindow | null;
  user_hidden: boolean;
}
