# Changelog

## Unreleased

- App shell from dnsubs: Tauri 2, React 19, Tailwind v3.
- Per-OS config and cache folders (`ding`, `ding-dev` for debug builds).
- First-run import of pong's `calendars.json`, `weather.json`, `theme.json`.
- fizx/upleb themes, alternating per launch or pinned in `theme.json`.
- Single instance: a second launch focuses the running window.
- Esc / Q quit.
- Data back end ported from pong to Rust: Open-Meteo weather + air quality
  (162 bundled cities, geocode cache), Google Calendar ICS feeds (own ICS
  reader; repeats, exclusions, moved and cancelled instances, multi-day
  events), daemon status (systemd + bitcoind on Linux, brew on macOS).
- Calendar matches pong's per-day output on a real feed. Changes from pong:
  a failed fetch keeps the calendar's last events; all-day events and
  floating times are read in local time; cancelled events are left out.
- `ding --check` prints one fetch of everything as JSON; `ding --list-cities`.
- Window shows a plain readout of the live data (layout in step 3).
- Dropped pong's openclaw port check (unused); the network card shows
  nginx and tor.
- Tailwind font families for pong's three faces: Ubuntu, Google Sans Flex,
  DejaVu Sans.
