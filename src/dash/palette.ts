// The active theme's colours as RGB triples, read from the CSS tokens in
// src/index.css (the single source), plus pong's derived and fixed tones.

export type Rgb = [number, number, number];

export interface Palette {
  bg: Rgb;
  fg: Rgb;
  muted: Rgb;
  accent: Rgb;
  mauve: Rgb;
  /** mauve at 70% brightness (pong's MAUVE_FADE) */
  mauveFade: Rgb;
  auburn: Rgb;
  ok: Rgb;
  alert: Rgb;
  net: Rgb;
  text: Rgb;
}

function token(style: CSSStyleDeclaration, name: string): Rgb {
  const parts = style.getPropertyValue(name).trim().split(/\s+/).map(Number);
  return [parts[0] || 0, parts[1] || 0, parts[2] || 0];
}

/** Read after the theme class is on <html>. */
export function readPalette(): Palette {
  const s = getComputedStyle(document.documentElement);
  const mauve = token(s, "--c-mauve");
  return {
    bg: token(s, "--c-bg"),
    fg: token(s, "--c-fg"),
    muted: token(s, "--c-muted"),
    accent: token(s, "--c-accent"),
    mauve,
    mauveFade: mauve.map((c) => Math.floor(c * 0.7)) as Rgb,
    auburn: token(s, "--c-auburn"),
    ok: token(s, "--c-ok"),
    alert: token(s, "--c-alert"),
    net: token(s, "--c-net"),
    text: token(s, "--c-text"),
  };
}

/** CSS colour from an RGB triple and pong's 0-255 alpha. */
export function rgba(c: Rgb, alpha = 255): string {
  return `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${+(alpha / 255).toFixed(4)})`;
}
