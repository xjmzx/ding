# ding — plan

**Status:** plan only — nothing built. Written 2026-10-24.

**ding** replaces **pong** (`xjmzx/pong`, Python + pygame, v0.5.0) as a fresh
repo and a separate app bundle. The name: *d* for daemons — the dashboard's
job is checking them — and *ding* like a notification sound. Variants, when
there are more than one, are **ding-dash** and **ding-lock**.

## Scope

**Now: the dashboard only (ding-dash).** Same cards, same layout, same theme
switching as `pong --dashboard`. The one thing dropped is the Pong animation.

**Later: the lock (ding-lock).** A deterrent, as pong's is: one fullscreen
window per monitor so the GNOME top bar is covered, our own password field
checked through PAM. Not a true lock — on GNOME/Wayland only the compositor
can lock the session (see pong's `docs/proposals/wayland-true-lock.md`).
Design notes for it are kept below so they are not lost.

## Stack

**Tauri 2** (React 19 + TypeScript + Tailwind v3, Rust back end), the same as
the rest of the n-suite.

- The dashboard is cards in a grid — CSS grid replaces most of pong's
  hand-placed pygame drawing (tile maths, text fitting, emoji runs).
- Native Wayland, no XWayland.
- macOS gets a real `.app` from the same code, replacing the PyInstaller bundle.
- Suite conventions carry over: `make dev/install/check/deb/version`, the icon
  pipeline (89% Linux crop), `--c-*` theme tokens, tag-triggered release CI,
  `-dev` isolation for debug builds.

Ruled out: GTK4 + Rust (poor on macOS, so the UI gets written twice); a shared
Rust core with GTK4 + SwiftUI shells (about twice the UI work).

## What carries over unchanged

- **Layout:** the dashboard arrangement exactly as pong draws it — the 5×3
  lattice of 3-column groups and 2-row bands, the clock band (palette card ·
  clock · `user@host` + version card) and the bottom band (temperatures · air
  quality · forecast · chain · network), the graph-paper sub-grid and the
  group highlight frames. pong's geometry constants are the reference.
- **Themes:** fizx and upleb, alternating on each launch, pinned by
  `theme.json` `{"mode": "fizx" | "upleb" | "alternate"}`. The palette is
  pong's, which mirrors the suite's `index.css`. This is an existing colour
  set, so it keeps its themes — the monochrome default for new apps does not
  apply here.
- **Config:** the same `calendars.json`, `weather.json`, `theme.json`
  formats.

## The paddles

- Kept **in place and static**, at pong's size, margin and colour (clock-tone,
  ~50% alpha). Reserved for a future feature; no behaviour for now.
- The ball goes. It was already invisible; it only drove the paddles and the
  mini-tile flashes. **The tile flashes go with it** for now, since nothing
  moves to trigger them.
- **Later (decided 2026-10-24):** a new effect may come back — something with
  the paddles, a tile-flashing loop, or both. So keep the mini-tile lattice as
  separate addressable elements (one per tile, with its own alpha) rather
  than a painted background, and give the paddles their own components, so
  either can be animated later without reworking the layout.
- The tile clearance that keeps cards off the paddles stays, so the layout
  does not shift.

## Shape of the app

**Rust does the work; React only draws.**

| Rust module | Ported from `pong_lock.py` |
|---|---|
| `config.rs` | The three config files, in each OS's config folder (`ding`). Debug builds use `-dev` folders. On first run, read pong's config folder once so calendar URLs and cities carry over. |
| `weather.rs` | Open-Meteo forecast + air quality, geocode cache, the 162 bundled cities. One shared HTTP client. |
| `calendar.rs` | ICS fetch, recurring events, 7-day lookahead, Google colour names. |
| `services.rs` | `systemctl`, port probes, bitcoind block height on Linux; `brew services` on macOS. |
| `theme.rs` | The alternation marker in the cache folder, as pong's `theme` file. |

- Each data source runs on its own timer and pushes updates to the window as
  events. On open, the window asks for the current state once.
- `ding --list-cities` is handled before Tauri starts, as in pong.
- Single-instance plugin; Esc/Q quit, as in pong's dashboard.

**Frontend:** CSS grid for the lattice, one component per card, themes as
`--c-*` tokens switched by a class on the root.

## Steps

1. **Set up the repo.** Copy the dnsubs shell; `make` targets, icon pipeline,
   config paths, theme tokens. New icon from the Figma masters (pong-dash's
   `icon-dash.svg` until there is one).
2. **Port the data back end**, with tests against saved sample files (an ICS
   file, Open-Meteo JSON, `systemctl` output).
3. **Match the dashboard** — every card, the layout and both themes, checked
   side by side against `pong --dashboard`.
4. **Ship** — tag-triggered release for Linux `.deb`, macOS `.dmg`, Windows.
   Then point pong's README at ding; pong stays private as an archive.
5. **macOS** — the dashboard comes with step 3; check `brew services` and the
   port probes on the Mac.

## Later: ding-lock

Kept from the rebuild plan, for when the lock comes back.

- `ding-lock` (or `ding --lock`): one undecorated, always-on-top window per
  monitor, each sent fullscreen onto its own monitor through GTK's
  `fullscreen_on_monitor` (Wayland does not let a client place windows).
  A true fullscreen window should cover the top bar.
- **First, a go/no-go test** of just those windows on GNOME Wayland: top bar
  hidden on both monitors, focus lands, close blocked. Needs eyes on the
  screen — GNOME refuses scripted screenshots of the session.
- Every window shows the password strip; the focused one forwards keystrokes
  to Rust, which holds the buffer and broadcasts the strip to all windows.
- `lock.rs`: PAM (`pam-client`, service `login`), wrong-attempt cooloff
  persisted to the cache folder, 8-second input timeout.
- Close requests blocked, Esc/Q ignored, windows rebuilt on monitor hotplug,
  systemd user unit with `Restart=on-failure`.
- Limits: Super+Esc and a TTY switch still get past it — a deterrent.
- macOS: system kiosk options (hide Dock and menu bar, block Cmd-Tab and
  Force Quit) plus PAM.

## Still open

- **Windows build** — cheap with Tauri; drop it if no longer wanted.
- **GitHub repo** — not created yet. **Public, as long as all private data
  stays local** (decided 2026-10-24). If anything private ever has to be
  committed to the repo, the repo goes private instead. In practice:
  - Calendar URLs, cities, theme choice and every cache file live only in
    the per-OS config and cache folders, never in the repo.
  - Test fixtures (step 2) are made up: a synthetic ICS file, invented
    events, no real calendar export or real `systemctl` capture.
  - No server address, SSH user or port (suite rule). The daemon checks
    probe `localhost` only, by unit name and default port.
  - `.gitignore` refuses the config file names and `*.ics` outside the
    fixtures folder, as a guard against copying real config in by mistake.
