import type { Config } from "tailwindcss";

// Themeable palette — values come from CSS custom properties defined in
// src/index.css. The channel-triple form keeps Tailwind's `/opacity`
// modifiers working (e.g. bg-mauve/10 → rgb(var(--c-mauve) / 0.1)).
const c = (v: string) => `rgb(var(${v}) / <alpha-value>)`;

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: c("--c-bg"),
        panel: c("--c-panel"),
        fg: c("--c-fg"),
        muted: c("--c-muted"),
        accent: c("--c-accent"),
        mauve: c("--c-mauve"),
        auburn: c("--c-auburn"),
        ok: c("--c-ok"),
        alert: c("--c-alert"),
        // Structural: never overridden by a theme.
        net: c("--c-net"),
        gridMain: c("--c-grid-main"),
        gridSub: c("--c-grid-sub"),
        kbDim: c("--c-kb-dim"),
        text: c("--c-text"),
      },
      fontFamily: {
        sans: ["Ubuntu", "Helvetica", "Arial", "system-ui", "sans-serif"],
        mono: ["Ubuntu Mono", "DejaVu Sans Mono", "Menlo", "monospace"],
      },
    },
  },
  plugins: [],
} satisfies Config;
