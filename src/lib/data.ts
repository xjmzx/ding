import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { inTauri } from "./tauri";
import { mockSnapshot } from "../dev/mock";

// Mirrors the Rust types in src-tauri/src/{weather,calendar,services}.rs.

export type Rgb = [number, number, number];

export interface WeatherState {
  cities: { label: string; temp: string }[];
  /** band: 0 good, 1 moderate, 2 unhealthy for sensitive groups, 3 unhealthy+ */
  aqi: { label: string; text: string; band: number | null }[];
  /** primary city's sunrise, sunset: local "HH:MM" */
  sun: [string, string] | null;
  forecast: { day: string; hilo: string }[];
  moon: string;
  updated: string | null;
}

export interface NextEvent {
  start: string;
  all_day: boolean;
  time: string | null;
  summary: string;
  location: string;
}

export interface DayEvent {
  color: Rgb | null;
  name: string;
  summary: string;
  time: string | null;
}

export interface CalendarState {
  status: "starting" | "ok" | "no-config";
  calendars: { name: string; color: Rgb | null; next: NextEvent | null; error: string | null }[];
  /** "YYYY-MM-DD" -> events: all-day first, then by time */
  by_date: Record<string, DayEvent[]>;
  updated: string | null;
}

export type Tone = "ok" | "accent" | "auburn" | "muted";

export interface ServicesState {
  chain: { caption: string; value: string; tone: Tone }[];
  net: { caption: string; value: string; tone: Tone }[];
}

export interface Snapshot {
  weather: WeatherState;
  calendar: CalendarState;
  services: ServicesState;
}

/** The live dashboard data: the current snapshot, then each update. */
export function useSnapshot(): Snapshot | null {
  // a plain browser in dev mode gets made-up data (src/dev/mock.ts)
  const [snap, setSnap] = useState<Snapshot | null>(() => (!inTauri && import.meta.env.DEV ? mockSnapshot() : null));

  useEffect(() => {
    if (!inTauri) return;
    let alive = true;
    const unlisten: Array<() => void> = [];
    const on = <K extends keyof Snapshot>(key: K) =>
      listen<Snapshot[K]>(key, (e) => setSnap((s) => (s ? { ...s, [key]: e.payload } : s))).then((u) =>
        alive ? unlisten.push(u) : u(),
      );

    // Listen first, then fetch the snapshot, so no update falls between.
    Promise.all([on("weather"), on("calendar"), on("services")])
      .then(() => invoke<Snapshot>("snapshot"))
      .then((s) => alive && setSnap(s));

    return () => {
      alive = false;
      unlisten.forEach((u) => u());
    };
  }, []);

  return snap;
}
