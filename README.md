# ding

A dashboard on one screen: clock, date, weather and air quality, two
calendars, and the status of the daemons on this machine (*d* for daemons;
*ding* like a notification). The successor to **pong**'s dashboard mode,
rebuilt as a Tauri app without the Pong animation. Same layout, same
fizx/upleb theme alternation.

**Status:** early. Steps 1–2 of [docs/plan.md](docs/plan.md) are done: the
app shell, config folders, theme switching, and the data back end (weather,
air quality, calendars, daemon status). The window shows a plain readout of
that data; the dashboard layout comes next.

## Build and install

```
make deps       # once: npm install + cargo fetch
make dev        # run with hot reload
make check      # typecheck + cargo check
make install    # Linux: binary + .desktop + icon under ~/.local
make deb        # Linux: a .deb for another machine
./install.sh    # macOS: build the .app and install it to /Applications
```

Debug builds keep their config and cache in `ding-dev` folders, so a dev run
never touches the installed app's state.

## Configuration

ding reads its config from the per-OS config folder, the same places pong
used, under `ding` instead of `pong`:

| OS | Config | Cache |
| --- | --- | --- |
| Linux | `$XDG_CONFIG_HOME/ding` or `~/.config/ding` | `$XDG_CACHE_HOME/ding` or `~/.cache/ding` |
| macOS | `~/Library/Application Support/ding` | `~/Library/Caches/ding` |
| Windows | `%APPDATA%\ding` | `%LOCALAPPDATA%\ding` |

On first run ding copies `calendars.json`, `weather.json` and `theme.json`
from pong's config folder, if they are there and ding has none of its own.
pong's copies are left in place.

Everything private stays in those folders and never in this repo: calendar
URLs (treat them as passwords), cities, the theme choice and every cache
file. The daemon checks probe `localhost` only.

### Theme (`theme.json`)

```json
{ "mode": "alternate" }
```

`alternate` (the default) flips between fizx and upleb on each launch; the
first launch is fizx. `"fizx"` or `"upleb"` pins one.

## Command line

```
ding --list-cities   # the 162 bundled cities for weather.json
ding --check         # fetch everything once, print it as JSON, no window
```

`--check` reads the same config as the app (a debug build reads `ding-dev`).
Its output includes your calendar events, so treat it as private.

### Weather (`weather.json`)

Up to three cities; the first also drives sunrise/sunset and the 3-day
forecast. An entry is a name (`"Edinburgh"`), coordinates
(`"55.95,-3.19"`), or an object with `name`, `latitude` + `longitude` and
`label`. Bundled names resolve offline; other names are geocoded once via
Open-Meteo and cached. `"model"` optionally pins an Open-Meteo model.
Refreshed every 30 minutes.

### Calendars (`calendars.json`)

```json
{ "calendars": [
  { "name": "Jog", "url": "https://calendar.google.com/calendar/ical/.../basic.ics", "color": "pistachio" }
] }
```

Google Calendar's private iCal address (Settings and sharing → Integrate
calendar → Secret address in iCal format). Treat it as a password. `color` is
a Google colour name or `#rrggbb`. Refreshed every 10 minutes; a calendar
whose fetch fails keeps its last events.

### Daemons

Linux: `systemctl` state for bitcoind, electrs, nginx and tor, and
bitcoind's block height via `bitcoin-cli`. macOS: `brew
services` plus port checks for bitcoind and electrs. Every 20 seconds, local
only.

## Keys

**Esc** or **Q** quits.
