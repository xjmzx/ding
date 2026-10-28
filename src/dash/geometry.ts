// The dashboard's layout, computed from pong's constants exactly as
// pong_lock.py does. Everything is in pong's logical 1920×1080 canvas,
// which the window scales to fit (letterboxed, aspect kept).

export const LOGICAL_W = 1920;
export const LOGICAL_H = 1080;

// Paddles (static in udash; reserved for a future effect)
export const PADDLE_W = 48;
export const PADDLE_H = 144;
export const PADDLE_MARGIN = 60;
const PADDLE_GAP = 20;
const PADDLE_CLEAR = PADDLE_MARGIN + PADDLE_W + PADDLE_GAP; // 128

// Lattice of square mini-tiles
const DASH_INSET_Y = 60;
const HL_INSET = 10;
export const HL_RADIUS = 14;
export const MINI = 92; // DASH_MINI_SIZE
export const GAP = 2 * HL_INSET; // DASH_MINI_GAP = 20
export const PITCH = MINI + GAP; // 112

const activeL = PADDLE_CLEAR;
const activeT = DASH_INSET_Y;
const activeW = LOGICAL_W - 2 * PADDLE_CLEAR;
const activeH = LOGICAL_H - 2 * DASH_INSET_Y;

export const LAT_COLS = Math.floor((activeW + GAP) / PITCH); // 15
export const LAT_ROWS = Math.floor((activeH + GAP) / PITCH); // 8
export const LAT_W = LAT_COLS * MINI + (LAT_COLS - 1) * GAP; // 1660
export const LAT_H = LAT_ROWS * MINI + (LAT_ROWS - 1) * GAP; // 876
export const LAT_X = activeL + Math.floor((activeW - LAT_W) / 2); // 130
export const LAT_Y = activeT + Math.floor((activeH - LAT_H) / 2); // 102

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** A span of lattice tiles, as pong's _lat_rect. */
export function latRect(c: number, r: number, cspan = 1, rspan = 1): Rect {
  return {
    x: LAT_X + c * PITCH,
    y: LAT_Y + r * PITCH,
    w: cspan * MINI + (cspan - 1) * GAP,
    h: rspan * MINI + (rspan - 1) * GAP,
  };
}

// Dashboard arrangement (pong's dashboard-mode rects)
const CLOCK_COLS = 9;
const CLOCK_ROWS = 2;
const clockC = Math.floor((LAT_COLS - CLOCK_COLS) / 2); // 3
const clockR = Math.floor((LAT_ROWS - CLOCK_ROWS) / 2); // 3
const BAND = LAT_ROWS - 2; // bottom band, rows 6-7
const GRP = 3; // columns per group

export const RECTS = {
  clock: latRect(clockC, clockR, CLOCK_COLS, CLOCK_ROWS),
  identity: latRect(0, clockR, GRP, 2), // palette card
  hostcard: latRect(LAT_COLS - GRP, clockR, GRP, 2),
  weather: latRect(0, BAND, GRP, 2),
  aqi: latRect(GRP, BAND, GRP, 2),
  lookahead: latRect(2 * GRP, BAND, GRP, 2),
  chain: latRect(3 * GRP, BAND, GRP, 2),
  net: latRect(4 * GRP, BAND, GRP, 2),
};

/** Group outlines drawn on the static layer (the forecast card has none). */
export const OUTLINED: (keyof typeof RECTS)[] = ["weather", "identity", "hostcard", "chain", "net", "aqi"];

// Week strip: each date sits in the top-right tile of a 3×2 group.
export const DATE_COLS = [2, 5, 8, 11, 14];
export const DATE_ROWS = [0, 2, 4, 6];
export const GROUP_W = 3 * MINI + 2 * GAP; // 316
export const GROUP_H = 2 * MINI + GAP; // 204

// Header strip and the view toggle, above the lattice
export const VIEW_BTN_R = 14;
export const VIEW_BTN_CX = LAT_X + LAT_W - VIEW_BTN_R;
export const VIEW_BTN_CY = LAT_Y - VIEW_BTN_R - 8; // 80; also the header's centre line

// Text and card insets
export const TILE_INSET = 20;
export const LABEL_GAP = 8;

// Alphas (0-255, as pong)
export const A = {
  tileFaint: 25, // mini-tile wash
  tileBg: 38, // day-group wash baseline
  todayBoost: 51,
  tomorrowBoost: 26,
  past1Drop: 13,
  past2Drop: 26,
  cardTint: 40,
  hostcard: 150,
  paddle: 25 + 90, // a mini-tile at peak flash
  chip: 235,
};

/** Glyph alpha for past days in the week strip. */
export function pastFgAlpha(deltaDays: number): number {
  if (deltaDays >= 0) return 255;
  if (deltaDays === -1) return 230;
  if (deltaDays === -2) return 204;
  if (deltaDays === -3) return 179;
  return 153;
}

/** Day-group wash alpha: today > tomorrow > others > yesterday > older. */
export function dayWashAlpha(deltaDays: number): number {
  if (deltaDays === 0) return Math.min(255, A.tileBg + A.todayBoost);
  if (deltaDays === 1) return Math.min(255, A.tileBg + A.tomorrowBoost);
  if (deltaDays === -1) return Math.max(0, A.tileBg - A.past1Drop);
  if (deltaDays < -1) return Math.max(0, A.tileBg - A.past2Drop);
  return A.tileBg;
}

/** pong's palette card: both themes' signature tones, fixed. */
export const THEME_SWATCHES: [number, number, number][][] = [
  // fizx: accent, mauve, auburn
  [
    [122, 240, 205],
    [189, 168, 251],
    [178, 96, 138],
  ],
  // upleb
  [
    [255, 182, 158],
    [255, 198, 117],
    [178, 96, 58],
  ],
];
