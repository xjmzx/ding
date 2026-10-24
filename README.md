# ding

A dashboard on one screen: clock, date, weather and air quality, two
calendars, and the status of the daemons on this machine (*d* for daemons;
*ding* like a notification). The successor to **pong**'s dashboard mode,
rebuilt as a Tauri app without the Pong animation. Same layout, same
fizx/upleb theme alternation.

**Status:** early. Step 1 of [docs/plan.md](docs/plan.md) — the app shell,
config folders and theme switching. The dashboard itself is not built yet.

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

## Keys

**Esc** or **Q** quits.
