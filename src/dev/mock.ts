// Made-up data for working on the layout in a plain browser (`npm run dev`,
// no Tauri). Dates are relative to today so the week strip always has
// something on it. Nothing here is real: invented events, generic weather.

import type { Snapshot } from "../lib/data";
import type { Identity } from "../lib/tauri";

const pad2 = (n: number) => String(n).padStart(2, "0");
const iso = (d: Date) => `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;

const PEACOCK: [number, number, number] = [3, 155, 229];
const FLAMINGO: [number, number, number] = [230, 124, 115];
const PISTACHIO: [number, number, number] = [125, 188, 88];
const MANGO: [number, number, number] = [255, 173, 70];

export function mockIdentity(): Identity {
  return { user: "you", host: "this-box", version: "0.1.0", wash_priority: ["Leave", "Gym", "Run"] };
}

export function mockSnapshot(now = new Date()): Snapshot {
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const wd = (today.getDay() + 6) % 7;
  const monday = new Date(today.getFullYear(), today.getMonth(), today.getDate() + (wd <= 4 ? -wd : 7 - wd));
  const day = (offset: number) => iso(new Date(monday.getFullYear(), monday.getMonth(), monday.getDate() + offset));

  // [offset from the strip's Monday, calendar, colour, summary, time]
  const events: [number, string, [number, number, number], string, string | null][] = [
    [0, "Run", PISTACHIO, "Morning run", "07:00"],
    [1, "Work", MANGO, "Team call about the quarterly plan and budget", "10:00"],
    [1, "Gym", FLAMINGO, "Legs", "18:00"],
    [2, "Leave", PEACOCK, "Off", null],
    [3, "Run", PISTACHIO, "Intervals 🏃", "06:30"],
    [3, "Work", MANGO, "Review", "11:00"],
    [3, "Work", MANGO, "Planning", "14:00"],
    [3, "Work", MANGO, "Retro", "16:00"],
    [3, "Work", MANGO, "Drinks", "18:00"],
    [7, "Work", MANGO, "Dentist", "09:30"],
    [9, "Gym", FLAMINGO, "Swim", "07:30"],
    [11, "Leave", PEACOCK, "Holiday", null],
    [12, "Leave", PEACOCK, "Holiday", null],
    [16, "Run", PISTACHIO, "Long run", "08:00"],
    [22, "Work", MANGO, "Conference ✈️", "09:00"],
  ];
  const by_date: Snapshot["calendar"]["by_date"] = {};
  for (const [off, name, color, summary, time] of events) {
    (by_date[day(off)] ??= []).push({ color, name, summary, time });
  }
  for (const list of Object.values(by_date)) {
    list.sort((a, b) => Number(a.time !== null) - Number(b.time !== null) || (a.time ?? "").localeCompare(b.time ?? ""));
  }

  return {
    weather: {
      cities: [
        { label: "HANOI", temp: "30°C" },
        { label: "EDINB", temp: "4°C" },
        { label: "BEIJING", temp: "16°C" },
      ],
      aqi: [
        { label: "HANOI", text: "88", band: 1 },
        { label: "EDINB", text: "29", band: 0 },
        { label: "BEIJING", text: "243", band: 3 },
      ],
      sun: ["05:55", "17:26"],
      forecast: [
        { day: "TODAY", hilo: "32/25" },
        { day: "SUN", hilo: "31/26" },
        { day: "MON", hilo: "31/26" },
      ],
      moon: "🌔",
      updated: now.toISOString(),
    },
    calendar: {
      status: "ok",
      calendars: [
        { name: "Run", color: PISTACHIO, next: null, error: null },
        { name: "Work", color: MANGO, next: null, error: null },
      ],
      by_date,
      updated: now.toISOString(),
    },
    services: {
      chain: [
        { caption: "BTCD", value: "UP", tone: "ok" },
        { caption: "ELEC", value: "UP", tone: "ok" },
        { caption: "BLOCK", value: "972,877", tone: "ok" },
      ],
      net: [
        { caption: "NGNX", value: "UP", tone: "ok" },
        { caption: "TOR", value: "OFF", tone: "muted" },
      ],
    },
  };
}
