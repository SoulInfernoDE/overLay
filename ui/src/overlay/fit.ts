// Keeps mod windows inside the overlay. Manifest positions are written for one screen
// size; on a smaller game window (or after switching to windowed mode) they could lie
// outside, and a window there can neither be seen nor dragged back.

export interface Point {
  x: number;
  y: number;
}

/** Moves a window of `width` × `height` so it lies inside the viewport; a window larger
 * than the viewport is pinned to its top left corner. */
export function fitInside(
  point: Point,
  width: number,
  height: number,
  viewportWidth: number,
  viewportHeight: number,
): Point {
  return {
    x: clamp(point.x, 0, viewportWidth - width),
    y: clamp(point.y, 0, viewportHeight - height),
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(value, max));
}
