// Text measurement, so layout decisions match pong's: pong sizes and
// places text from the font's own box (SDL_ttf's ascent + descent), wraps
// event titles greedily to fit, and ellipsises labels that overrun.

export const FONTS = {
  // pong's UBUNTU_STACK
  ui: (px: number) => `${px}px Ubuntu, Helvetica, Arial, "Nimbus Sans", "Liberation Sans", sans-serif`,
  // Google Sans Flex Bold (clock, temperatures, date numerals)
  clock: (px: number) =>
    `bold ${px}px "Google Sans Flex", Roboto, Inter, "IBM Plex Sans", Ubuntu, Helvetica, sans-serif`,
  // pong's SANS_STACK, bold (event chips). pygame resolves "dejavu sans"
  // bold to DejaVuSansCondensed-Bold, so the condensed face is what pong
  // draws.
  event: (px: number) =>
    `bold ${px}px "DejaVu Sans Condensed", "DejaVu Sans", Helvetica, Arial, "Nimbus Sans", "Liberation Sans", sans-serif`,
};

let ctx: CanvasRenderingContext2D | null = null;
function context(): CanvasRenderingContext2D {
  if (!ctx) ctx = document.createElement("canvas").getContext("2d")!;
  return ctx;
}

export function textWidth(font: string, text: string): number {
  const c = context();
  c.font = font;
  return c.measureText(text).width;
}

/** The font's box: ascent + descent, as pygame's get_height(). */
export function fontBox(font: string): { ascent: number; height: number } {
  const c = context();
  c.font = font;
  const m = c.measureText("Hg");
  const ascent = m.fontBoundingBoxAscent ?? m.actualBoundingBoxAscent;
  const descent = m.fontBoundingBoxDescent ?? m.actualBoundingBoxDescent;
  return { ascent, height: ascent + descent };
}

/** Greedy word-wrap to at most `maxLines` lines that each fit `maxW`; an
 *  over-long word is hard-truncated (pong's _wrap_lines). */
export function wrapLines(font: string, text: string, maxW: number, maxLines = 2): string[] {
  const words = text.split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  const lines: string[] = [];
  let cur = words[0];
  for (const w of words.slice(1)) {
    if (textWidth(font, `${cur} ${w}`) <= maxW) {
      cur += ` ${w}`;
    } else {
      lines.push(cur);
      cur = w;
      if (lines.length === maxLines) break;
    }
  }
  if (lines.length < maxLines) lines.push(cur);
  return lines.slice(0, maxLines).map((ln) => {
    const chars = Array.from(ln);
    while (chars.length > 1 && textWidth(font, chars.join("")) > maxW) chars.pop();
    return chars.join("");
  });
}
