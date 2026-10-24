import { invoke } from "@tauri-apps/api/core";
import { mockIdentity } from "../dev/mock";

export type ThemeName = "fizx" | "upleb";

/** The user@host and version card, plus calendars.json's wash priority. */
export interface Identity {
  user: string;
  host: string;
  version: string;
  wash_priority: string[];
}

/** True inside the Tauri window; false in a plain browser (vite dev). */
export const inTauri = "__TAURI_INTERNALS__" in window;

export async function getTheme(): Promise<ThemeName> {
  // a plain browser: ?theme=upleb previews the other theme
  if (!inTauri) return new URLSearchParams(location.search).get("theme") === "upleb" ? "upleb" : "fizx";
  return invoke<ThemeName>("theme");
}

export async function getIdentity(): Promise<Identity | null> {
  if (!inTauri) return import.meta.env.DEV ? mockIdentity() : null;
  return invoke<Identity>("identity");
}

export async function quit(): Promise<void> {
  if (inTauri) await invoke("quit");
}
