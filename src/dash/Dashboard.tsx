// The dashboard, drawn as pong draws it: a fixed 1920×1080 canvas of
// absolutely placed pieces, scaled to fit the window. Pieces are in pong's
// drawing order. See docs/plan.md, step 3.

import { memo, useEffect, useMemo, useRef, useState, type CSSProperties, type ReactNode } from "react";
import type { CalendarState, Snapshot, Tone } from "../lib/data";
import type { Identity } from "../lib/tauri";
import {
  A,
  DATE_COLS,
  DATE_ROWS,
  GAP,
  GROUP_H,
  GROUP_W,
  HL_RADIUS,
  LABEL_GAP,
  LAT_COLS,
  LAT_H,
  LAT_ROWS,
  LAT_W,
  LAT_X,
  LAT_Y,
  LOGICAL_H,
  LOGICAL_W,
  MINI,
  OUTLINED,
  PADDLE_H,
  PADDLE_MARGIN,
  PADDLE_W,
  PITCH,
  RECTS,
  THEME_SWATCHES,
  TILE_INSET,
  VIEW_BTN_CX,
  VIEW_BTN_CY,
  VIEW_BTN_R,
  dayWashAlpha,
  pastFgAlpha,
  type Rect,
} from "./geometry";
import { rgba, type Palette, type Rgb } from "./palette";
import { FONTS, fontBox, textWidth, wrapLines } from "./text";

export type View = "clock" | "calendar";

const box = (r: Rect): CSSProperties => ({ position: "absolute", left: r.x, top: r.y, width: r.w, height: r.h });

// ---------------------------------------------------------------------------
// Stage: the logical canvas, scaled to the window

function useWindowSize() {
  const [size, setSize] = useState({ w: window.innerWidth, h: window.innerHeight });
  useEffect(() => {
    const on = () => setSize({ w: window.innerWidth, h: window.innerHeight });
    window.addEventListener("resize", on);
    return () => window.removeEventListener("resize", on);
  }, []);
  return size;
}

function Stage({ bg, children }: { bg: Rgb; children: ReactNode }) {
  const { w, h } = useWindowSize();
  const scale = Math.min(w / LOGICAL_W, h / LOGICAL_H);
  return (
    <div style={{ position: "fixed", inset: 0, background: rgba(bg), overflow: "hidden" }}>
      <div
        style={{
          position: "absolute",
          left: (w - LOGICAL_W * scale) / 2,
          top: (h - LOGICAL_H * scale) / 2,
          width: LOGICAL_W,
          height: LOGICAL_H,
          transform: `scale(${scale})`,
          transformOrigin: "0 0",
        }}
      >
        {children}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Static layer: mini-tile lattice, weekend side tiles, group outlines

/** Each mini-tile is its own element with its own wash, so a tile-flash
 *  effect can come back later without reworking the layout. */
const Lattice = memo(function Lattice({ pal }: { pal: Palette }) {
  const tiles = [];
  for (let r = 0; r < LAT_ROWS; r++) {
    for (let c = 0; c < LAT_COLS; c++) {
      tiles.push(
        <div
          key={`${c}-${r}`}
          data-tile={`${c},${r}`}
          style={{
            ...box({ x: LAT_X + c * PITCH, y: LAT_Y + r * PITCH, w: MINI, h: MINI }),
            boxSizing: "border-box",
            background: rgba(pal.mauve, A.tileFaint),
            border: `1px solid ${rgba(pal.net)}`,
            borderRadius: HL_RADIUS,
          }}
        />,
      );
    }
  }
  // Weekend brackets: one tile-wide column each side, fading away from
  // the lattice, no outline.
  const weekendH = 2 * MINI + GAP;
  const weekend = (x: number, y: number, towardRight: boolean) => {
    const mask = `linear-gradient(to ${towardRight ? "right" : "left"}, transparent, black)`;
    return (
      <div
        key={`wk-${x}-${y}`}
        style={{
          ...box({ x, y, w: MINI, h: weekendH }),
          background: rgba(pal.mauve, A.tileFaint),
          borderRadius: HL_RADIUS,
          maskImage: mask,
          WebkitMaskImage: mask,
        }}
      />
    );
  };
  const weekends = [0, 2, 4, 6].flatMap((r) => [
    weekend(LAT_X - PITCH, LAT_Y + r * PITCH, true),
    weekend(LAT_X + LAT_W + GAP, LAT_Y + r * PITCH, false),
  ]);
  const outlines = OUTLINED.map((k) => (
    <div
      key={`ol-${k}`}
      style={{ ...box(RECTS[k]), boxSizing: "border-box", border: `1px solid ${rgba(pal.net)}`, borderRadius: HL_RADIUS }}
    />
  ));
  return (
    <>
      {tiles}
      {weekends}
      {outlines}
    </>
  );
});

// ---------------------------------------------------------------------------
// Clock-view cards

function CardTints({ pal }: { pal: Palette }) {
  const tints: [keyof typeof RECTS, Rgb][] = [
    ["weather", pal.auburn],
    ["lookahead", pal.accent],
    ["aqi", pal.muted],
    ["chain", pal.mauve],
    ["net", pal.mauveFade],
  ];
  return (
    <>
      {tints.map(([k, c]) => (
        <div key={k} style={{ ...box(RECTS[k]), background: rgba(c, A.cardTint), borderRadius: HL_RADIUS }} />
      ))}
    </>
  );
}

/** Both themes' signature tones, one solid swatch per mini-tile. */
function PaletteCard() {
  const { x, y } = RECTS.identity;
  return (
    <>
      {THEME_SWATCHES.flatMap((row, ri) =>
        row.map((c, ci) => (
          <div
            key={`${ri}-${ci}`}
            style={{ ...box({ x: x + ci * PITCH, y: y + ri * PITCH, w: MINI, h: MINI }), background: rgba(c), borderRadius: HL_RADIUS }}
          />
        )),
      )}
    </>
  );
}

function HostCard({ pal, ident }: { pal: Palette; ident: Identity | null }) {
  const font = FONTS.ui(24);
  const lh = fontBox(font).height;
  return (
    <div
      style={{
        ...box(RECTS.hostcard),
        background: rgba([0, 0, 0], A.hostcard),
        borderRadius: HL_RADIUS,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 6,
        font,
        lineHeight: `${lh}px`,
        whiteSpace: "pre",
      }}
    >
      <span style={{ color: rgba(pal.text) }}>{ident ? `${ident.user}@${ident.host}` : ""}</span>
      <span style={{ color: rgba(pal.mauveFade) }}>{ident ? `v${ident.version}` : ""}</span>
    </div>
  );
}

/** Large focal time; the colon is pinned to the clock region's centre and
 *  blinks once a second (lit for the first half), so the screen visibly
 *  ticks between minute changes.
 *
 *  The time stays one run of text: Google Sans Flex raises the colon only
 *  when it sits between digits, so splitting it out would drop it onto the
 *  baseline. The blink is a mask over the colon's strip instead. */
function Clock({ pal, now }: { pal: Palette; now: Date }) {
  const font = FONTS.clock(260);
  const hh = String(now.getHours()).padStart(2, "0");
  const mm = String(now.getMinutes()).padStart(2, "0");
  const lead = textWidth(font, hh);
  const through = textWidth(font, `${hh}:`);
  const colon = (lead + through) / 2;
  const { height } = fontBox(font);
  const r = RECTS.clock;

  // dim the colon for the second half of each second
  const [dim, setDim] = useState(false);
  useEffect(() => {
    setDim(false);
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const id = window.setTimeout(() => setDim(true), 500);
    return () => window.clearTimeout(id);
  }, [now]);
  const mask = dim
    ? `linear-gradient(to right, #000 ${lead}px, rgba(0,0,0,0.2) ${lead}px, rgba(0,0,0,0.2) ${through}px, #000 ${through}px)`
    : "none";

  return (
    <div
      style={{
        position: "absolute",
        left: r.x + r.w / 2 - colon,
        top: r.y + (r.h - height) / 2,
        font,
        lineHeight: `${height}px`,
        color: rgba(pal.accent),
        whiteSpace: "pre",
        maskImage: mask,
        WebkitMaskImage: mask,
      }}
    >
      {`${hh}:${mm}`}
    </div>
  );
}

interface CardRow {
  label: string;
  value: string;
  color: Rgb;
}

/** Ellipsise `text` to fit `maxW` (pong's _render_fitted). */
function fitted(font: string, text: string, maxW: number): string {
  if (maxW <= 0) return "";
  if (textWidth(font, text) <= maxW) return text;
  let t = Array.from(text);
  while (t.length && textWidth(font, `${t.join("")}…`) > maxW) t = t.slice(0, -1);
  return `${t.join("")}…`;
}

/** A bottom-band card: label left in white, value right in its colour, each
 *  pair centred on its row (pong's _row_card). `slots` keeps a card's rows
 *  in place when it has fewer rows than slots. */
function RowCard({ rect, rows, slots, unit, pal }: { rect: Rect; rows: CardRow[]; slots?: number; unit?: string; pal: Palette }) {
  const labelFont = FONTS.ui(28);
  const valueFont = FONTS.clock(40);
  const unitFont = FONTS.ui(18);
  const lb = fontBox(labelFont);
  const vb = fontBox(valueFont);
  const ub = fontBox(unitFont);
  const n = slots ?? (rows.length || 1);
  const rowH = rect.h / n;
  const unitW = unit ? textWidth(unitFont, unit) + LABEL_GAP : 0;

  return (
    <div style={box(rect)}>
      {rows.map((row, i) => {
        const cy = Math.floor(rowH * i + rowH / 2);
        let value = row.value;
        // a wide value (the block height) drops its commas rather than
        // push its label out of the row
        if (textWidth(valueFont, value) + unitW > rect.w - 2 * TILE_INSET && value.includes(",")) {
          value = value.replace(/,/g, "");
        }
        const valW = textWidth(valueFont, value);
        const valX = rect.w - TILE_INSET - unitW - valW;
        const valY = cy - Math.floor(vb.height / 2);
        const label = fitted(labelFont, row.label, valX - TILE_INSET - LABEL_GAP);
        const text: CSSProperties = { position: "absolute", whiteSpace: "pre" };
        return (
          <div key={i}>
            <span style={{ ...text, left: TILE_INSET, top: cy - Math.floor(lb.height / 2), font: labelFont, lineHeight: `${lb.height}px`, color: rgba(pal.text) }}>
              {label}
            </span>
            <span style={{ ...text, left: valX, top: valY, font: valueFont, lineHeight: `${vb.height}px`, color: rgba(row.color) }}>
              {value}
            </span>
            {unit && (
              <span
                style={{
                  ...text,
                  left: rect.w - TILE_INSET - textWidth(unitFont, unit),
                  top: valY + vb.ascent - ub.ascent,
                  font: unitFont,
                  lineHeight: `${ub.height}px`,
                  color: rgba(pal.muted),
                }}
              >
                {unit}
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}

function toneColor(t: Tone, pal: Palette): Rgb {
  return { ok: pal.ok, accent: pal.accent, auburn: pal.auburn, muted: pal.muted }[t];
}

function Cards({ snap, pal }: { snap: Snapshot; pal: Palette }) {
  const { weather, services } = snap;
  const bands = [pal.ok, pal.accent, pal.auburn, pal.alert];
  return (
    <>
      <RowCard rect={RECTS.weather} pal={pal} rows={weather.cities.map((c) => ({ label: c.label, value: c.temp || "--", color: pal.auburn }))} />
      <RowCard rect={RECTS.lookahead} pal={pal} rows={weather.forecast.map((f) => ({ label: f.day, value: f.hilo, color: pal.auburn }))} />
      <RowCard
        rect={RECTS.aqi}
        pal={pal}
        unit="AQI"
        rows={weather.aqi.map((a) => ({
          label: a.label,
          value: a.text || "--",
          color: a.band === null ? pal.muted : bands[Math.min(a.band, bands.length - 1)],
        }))}
      />
      <RowCard rect={RECTS.chain} pal={pal} slots={3} rows={services.chain.map((r) => ({ label: r.caption, value: r.value, color: toneColor(r.tone, pal) }))} />
      {/* three slots: the network card's third row is free since openclaw went */}
      <RowCard rect={RECTS.net} pal={pal} slots={3} rows={services.net.map((r) => ({ label: r.caption, value: r.value, color: toneColor(r.tone, pal) }))} />
    </>
  );
}

// ---------------------------------------------------------------------------
// Week strip (clock view: this week; calendar view: four weeks)

const pad2 = (n: number) => String(n).padStart(2, "0");
const isoDate = (d: Date) => `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;
const dayNumber = (d: Date) => Math.round(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()) / 86_400_000);

/** Mon-Fri: this week's Monday. Sat-Sun: next Monday. */
function calendarAnchor(today: Date): Date {
  const wd = (today.getDay() + 6) % 7; // Monday = 0
  const shift = wd <= 4 ? -wd : 7 - wd;
  return new Date(today.getFullYear(), today.getMonth(), today.getDate() + shift);
}

const CHIP = { padX: 11, padY: 7, radius: 9, padRight: 12, padLeft: 12, dateGap: 16, eventGap: 6, lineGap: 2, maxLabels: 4 };

function WeekStrip({ cal, pal, view, washPriority, now }: { cal: CalendarState; pal: Palette; view: View; washPriority: string[]; now: Date }) {
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const anchor = calendarAnchor(today);
  const rows = view === "clock" ? DATE_ROWS.slice(0, 1) : DATE_ROWS;

  const numFont = FONTS.clock(30);
  const numBox = fontBox(numFont);
  const evFont = FONTS.event(24);
  const lineH = fontBox(evFont).height;
  const textWMax = GROUP_W - CHIP.padRight - CHIP.padLeft - 2 * CHIP.padX;

  const washes: ReactNode[] = [];
  const fronts: ReactNode[] = [];

  rows.forEach((rr, ri) => {
    DATE_COLS.forEach((cc, ci) => {
      const d = new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate() + ri * 7 + ci);
      const key = isoDate(d);
      const delta = dayNumber(d) - dayNumber(today);
      const tx = LAT_X + cc * PITCH;
      const ty = LAT_Y + rr * PITCH;
      const evs = cal.by_date[key] ?? [];

      // day wash: the first priority calendar with an event that day,
      // else mauve for any event, else the background
      let wash: Rgb | null = null;
      for (const name of washPriority) {
        const hit = evs.find((e) => e.name === name);
        if (hit) {
          wash = hit.color ?? pal.mauve;
          break;
        }
      }
      wash ??= evs.length ? pal.mauve : pal.bg;
      washes.push(
        <div
          key={`w-${key}`}
          style={{ ...box({ x: tx - 2 * PITCH, y: ty, w: GROUP_W, h: GROUP_H }), background: rgba(wash, dayWashAlpha(delta)), borderRadius: HL_RADIUS }}
        />,
      );

      // date numeral: top of its tile, white for today
      const fg = pastFgAlpha(delta);
      const num = pad2(d.getDate());
      const numX = tx + Math.floor((MINI - textWidth(numFont, num)) / 2);
      const numY = ty + 4;
      fronts.push(
        <span
          key={`n-${key}`}
          style={{
            position: "absolute",
            left: numX,
            top: numY,
            font: numFont,
            lineHeight: `${numBox.height}px`,
            color: rgba(delta === 0 ? pal.text : pal.accent),
            opacity: fg / 255,
            whiteSpace: "pre",
          }}
        >
          {num}
        </span>,
      );

      // event chips, right-aligned under the date, stacked while they fit
      if (!evs.length) return;
      const show = evs.length > CHIP.maxLabels ? CHIP.maxLabels - 1 : evs.length;
      const paneRight = tx + MINI - CHIP.padRight;
      const paneBottom = ty + GROUP_H;
      const chipAlpha = Math.floor((A.chip * fg) / 255);
      let y = numY + numBox.height + CHIP.dateGap;
      let drawn = 0;
      const chip = (k: string, lines: string[], color: Rgb) => {
        const w = Math.max(...lines.map((l) => textWidth(evFont, l))) + 2 * CHIP.padX;
        const h = lines.length * lineH + (lines.length - 1) * CHIP.lineGap + 2 * CHIP.padY;
        return { w, h, el: (top: number) => (
          <div
            key={k}
            style={{
              position: "absolute",
              left: paneRight - w,
              top,
              width: w,
              height: h,
              boxSizing: "border-box",
              padding: `${CHIP.padY}px ${CHIP.padX}px`,
              background: rgba(color, chipAlpha),
              borderRadius: CHIP.radius,
              font: evFont,
              lineHeight: `${lineH}px`,
              color: rgba([0, 0, 0], fg),
              textAlign: "right",
              whiteSpace: "pre",
              display: "flex",
              flexDirection: "column",
              alignItems: "flex-end",
              gap: CHIP.lineGap,
            }}
          >
            {lines.map((l, i) => (
              <span key={i}>{l}</span>
            ))}
          </div>
        ) };
      };
      for (const [i, e] of evs.slice(0, show).entries()) {
        const lines = wrapLines(evFont, (e.summary || e.name).toUpperCase(), textWMax);
        if (!lines.length) continue;
        const c = chip(`c-${key}-${i}`, lines, e.color ?? pal.muted);
        if (drawn > 0 && y + c.h > paneBottom) break;
        fronts.push(c.el(y));
        y += c.h + CHIP.eventGap;
        drawn++;
      }
      const leftover = evs.length - drawn;
      if (leftover > 0) {
        const c = chip(`m-${key}`, [`+${leftover}`], pal.muted);
        if (y + c.h <= paneBottom) fronts.push(c.el(y));
      }
    });
  });

  return (
    <>
      {washes}
      {fronts}
    </>
  );
}

// ---------------------------------------------------------------------------
// Header and view toggle

const DOW = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
const MON = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];

/** "SAT 24 OCT": the date numeral white, the rest in faded mauve. */
function Header({ pal, now }: { pal: Palette; now: Date }) {
  const font = FONTS.ui(24);
  const { height } = fontBox(font);
  const fade = rgba(pal.mauveFade);
  return (
    <div
      style={{
        position: "absolute",
        left: LAT_X + LAT_W / 2,
        top: VIEW_BTN_CY - height / 2,
        transform: "translateX(-50%)",
        font,
        lineHeight: `${height}px`,
        whiteSpace: "pre",
      }}
    >
      <span style={{ color: fade }}>{DOW[now.getDay()]} </span>
      <span style={{ color: rgba(pal.text) }}>{pad2(now.getDate())}</span>
      <span style={{ color: fade }}> {MON[now.getMonth()]}</span>
    </div>
  );
}

/** Clock view: a 2×2 grid icon in accent. Calendar view: a calendar icon in
 *  mauve. Clicking flips the view. */
function ViewToggle({ pal, view, onToggle }: { pal: Palette; view: View; onToggle: () => void }) {
  const hit = VIEW_BTN_R + 6;
  const color = rgba(view === "clock" ? pal.accent : pal.mauve);
  return (
    <button
      aria-label={view === "clock" ? "Show calendar view" : "Show clock view"}
      onClick={onToggle}
      style={{
        position: "absolute",
        left: VIEW_BTN_CX - hit,
        top: VIEW_BTN_CY - hit,
        width: 2 * hit,
        height: 2 * hit,
        padding: 0,
        border: 0,
        borderRadius: "50%",
        background: "transparent",
        cursor: "pointer",
      }}
    >
      <svg width={2 * hit} height={2 * hit} viewBox={`${-hit} ${-hit} ${2 * hit} ${2 * hit}`} style={{ display: "block" }}>
        {view === "clock" ? (
          [-10, 1].flatMap((x) => [-10, 1].map((y) => <rect key={`${x}${y}`} x={x} y={y} width={9} height={9} rx={2} fill={color} />))
        ) : (
          <g fill="none" stroke={color} strokeWidth={2}>
            <rect x={-10} y={-6} width={20} height={18} rx={3} />
            <line x1={-9} y1={0} x2={9} y2={0} />
            <rect x={-7} y={-11} width={3} height={5} fill={color} stroke="none" />
            <rect x={4} y={-11} width={3} height={5} fill={color} stroke="none" />
          </g>
        )}
      </svg>
    </button>
  );
}

/** Paddle travel: resting with its foot on the lattice's bottom edge, a
 *  full hop rises until its head meets the lattice's top edge. */
const PADDLE_REST_Y = LAT_Y + LAT_H - PADDLE_H;
const PADDLE_RISE = PADDLE_REST_Y - LAT_Y; // 732
/** A full hop's duration (up and back). Smaller hops take less time, in
 *  proportion to the square root of their height, as a real throw does. */
const PADDLE_HOP_MS = 2400;

/** How high the right paddle hops when the clock reaches minute `m`, as a
 *  fraction of the full rise: the hour to the top, the half hour halfway,
 *  the quarters a quarter, other 5-minute marks a small bounce. */
function markHeight(m: number): number {
  if (m === 0) return 1;
  if (m === 30) return 0.5;
  if (m % 15 === 0) return 0.25;
  if (m % 5 === 0) return 0.08;
  return 0;
}

/** One rise and fall to `fraction` of the full height: eases out going up,
 *  eases in coming down, so it reads as a throw against gravity. */
function hopPaddle(el: HTMLElement | null, fraction: number) {
  if (!el || fraction <= 0 || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  el.animate(
    [
      { transform: "translateY(0)", easing: "cubic-bezier(0.2, 0.7, 0.3, 1)" },
      { transform: `translateY(${-Math.round(PADDLE_RISE * fraction)}px)`, offset: 0.5, easing: "cubic-bezier(0.6, 0, 0.8, 0.3)" },
      { transform: "translateY(0)" },
    ],
    { duration: Math.round(PADDLE_HOP_MS * Math.sqrt(fraction)) },
  );
}

/** Both paddles rest on the lattice baseline. On each minute change the
 *  left one hops to the top; the right one marks the clock face (see
 *  markHeight), so on the hour both go to the top. Nothing moves at launch,
 *  only on a change. */
function Paddles({ pal, now }: { pal: Palette; now: Date }) {
  const left = useRef<HTMLDivElement>(null);
  const right = useRef<HTMLDivElement>(null);
  const minute = Math.floor(now.getTime() / 60_000);
  const seen = useRef<number | null>(null);

  useEffect(() => {
    const prev = seen.current;
    seen.current = minute;
    if (prev === null || minute === prev) return;
    hopPaddle(left.current, 1);
    hopPaddle(right.current, markHeight(new Date(minute * 60_000).getMinutes()));
  }, [minute]);

  const style = (x: number): CSSProperties => ({
    ...box({ x, y: PADDLE_REST_Y, w: PADDLE_W, h: PADDLE_H }),
    background: rgba(pal.mauve, A.paddle),
    borderRadius: PADDLE_W / 2,
    willChange: "transform",
  });
  return (
    <>
      <div ref={left} data-paddle="left" style={style(PADDLE_MARGIN)} />
      <div ref={right} data-paddle="right" style={style(LOGICAL_W - PADDLE_MARGIN - PADDLE_W)} />
    </>
  );
}

// ---------------------------------------------------------------------------

/** The current time, updated just after each whole second, so the colon
 *  blink and the minute change land on the real tick. */
function useNow(): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    let id = 0;
    const tick = () => {
      const t = new Date();
      setNow(t);
      id = window.setTimeout(tick, 1000 - t.getMilliseconds() + 5);
    };
    id = window.setTimeout(tick, 1000 - new Date().getMilliseconds() + 5);
    return () => window.clearTimeout(id);
  }, []);
  return now;
}

export default function Dashboard({ snap, pal, ident }: { snap: Snapshot | null; pal: Palette; ident: Identity | null }) {
  const [view, setView] = useState<View>("clock");
  const now = useNow();
  // fonts can arrive after first paint; re-measure once they are ready
  const [, setFontsReady] = useState(0);
  useEffect(() => {
    document.fonts.ready.then(() => setFontsReady((n) => n + 1));
  }, []);
  const washPriority = useMemo(() => ident?.wash_priority ?? [], [ident]);
  const clockView = view === "clock";

  return (
    <Stage bg={pal.bg}>
      <Lattice pal={pal} />
      {clockView && <CardTints pal={pal} />}
      {clockView && <PaletteCard />}
      {clockView && <HostCard pal={pal} ident={ident} />}
      {clockView && <Clock pal={pal} now={now} />}
      {clockView && snap && <Cards snap={snap} pal={pal} />}
      {snap && <WeekStrip cal={snap.calendar} pal={pal} view={view} washPriority={washPriority} now={now} />}
      <Header pal={pal} now={now} />
      <ViewToggle pal={pal} view={view} onToggle={() => setView((v) => (v === "clock" ? "calendar" : "clock"))} />
      <Paddles pal={pal} now={now} />
    </Stage>
  );
}
