import { invoke } from "@tauri-apps/api/core";

export type ThemeName = "fizx" | "upleb";

/** True inside the Tauri window; false in a plain browser (vite dev). */
export const inTauri = "__TAURI_INTERNALS__" in window;

export async function getTheme(): Promise<ThemeName> {
  if (!inTauri) return "fizx";
  return invoke<ThemeName>("theme");
}

export async function quit(): Promise<void> {
  if (inTauri) await invoke("quit");
}
