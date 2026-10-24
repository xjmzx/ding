//! Google Calendar private ICS feeds, ported from pong.
//!
//! For each calendar in `calendars.json`: fetch the ICS, expand repeats
//! over a window (7 days back, 35 ahead), and produce
//! - its soonest upcoming event (the clock view's NEXT chips), and
//! - an index of every event by local date (the calendar view's tiles),
//!   with multi-day events on every day they span.
//!
//! Differences from pong, all deliberate:
//! - A calendar whose fetch fails keeps its last good events (pong blanked
//!   it until the next cycle) and reports the error.
//! - All-day events compare against local midnight, and floating times are
//!   read as local time (pong read both as UTC).
//! - Cancelled events and cancelled instances are left out.

use std::collections::BTreeMap;
use std::fs;
use std::time::Duration;

use chrono::{DateTime, Duration as Span, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::ics::{self, Instance, When};

pub const REFRESH: Duration = Duration::from_secs(10 * 60);
const TIMEOUT: Duration = Duration::from_secs(10);
const LOOKBACK_DAYS: i64 = 7;
const LOOKAHEAD_DAYS: i64 = 35;

// ---------------------------------------------------------------------------
// Config

#[derive(Debug, Clone, Deserialize)]
pub struct CalConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub color: Option<String>,
}

pub fn config_path() -> std::path::PathBuf {
    config::config_dir().join("calendars.json")
}

/// Write an empty, private template on first run so it can be edited.
pub fn ensure_config() {
    let path = config_path();
    if path.exists() {
        return;
    }
    let body = serde_json::json!({
        "_comment": "Drop one entry per Google Calendar to surface: {\"name\": \"Jog\", \"url\": \"https://calendar.google.com/calendar/ical/.../basic.ics\", \"color\": \"pistachio\"}. Get the URL from Calendar > Settings and sharing > Integrate calendar > Secret address in iCal format. Keep this file private (mode 0600): anyone with a URL can read the calendar.",
        "calendars": [],
    });
    if fs::write(&path, serde_json::to_string_pretty(&body).unwrap() + "\n").is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
}

pub fn load_config() -> Vec<CalConfig> {
    #[derive(Deserialize)]
    struct File {
        #[serde(default)]
        calendars: Vec<CalConfig>,
    }
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str::<File>(&s).ok())
        .map(|f| f.calendars)
        .unwrap_or_default()
}

/// Google Calendar's named palette (approximate RGB), or "#rrggbb".
pub fn color(name: Option<&str>) -> Option<[u8; 3]> {
    let name = name?.trim();
    if let Some(hex) = name.strip_prefix('#') {
        if hex.len() != 6 {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        return Some([byte(0)?, byte(2)?, byte(4)?]);
    }
    let rgb = match name.to_ascii_lowercase().as_str() {
        "tomato" => [213, 0, 0],
        "flamingo" => [230, 124, 115],
        "tangerine" => [244, 81, 30],
        "pumpkin" => [239, 108, 0],
        "mango" => [255, 173, 70],
        "banana" => [246, 191, 38],
        "citron" => [235, 224, 72],
        "avocado" => [158, 199, 44],
        "pistachio" => [125, 188, 88],
        "basil" => [11, 128, 67],
        "sage" => [51, 182, 121],
        "eucalyptus" => [15, 152, 113],
        "peacock" => [3, 155, 229],
        "cobalt" => [41, 121, 255],
        "blueberry" => [63, 81, 181],
        "lavender" => [121, 134, 203],
        "wisteria" => [197, 202, 233],
        "amethyst" => [149, 117, 205],
        "grape" => [142, 36, 170],
        "graphite" => [97, 97, 97],
        "birch" => [167, 155, 142],
        "cocoa" => [121, 85, 72],
        _ => return None,
    };
    Some(rgb)
}

// ---------------------------------------------------------------------------
// State sent to the window

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Not fetched yet.
    Starting,
    Ok,
    /// calendars.json missing or empty: the calendar stays empty. Shown so
    /// an unconfigured calendar is not mistaken for a quiet week.
    NoConfig,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NextEvent {
    /// Local time, RFC 3339.
    pub start: String,
    pub all_day: bool,
    /// Local "HH:MM", or None for all-day.
    pub time: Option<String>,
    pub summary: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CalendarSummary {
    pub name: String,
    pub color: Option<[u8; 3]>,
    pub next: Option<NextEvent>,
    /// The last fetch's error, if it failed.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DayEvent {
    pub color: Option<[u8; 3]>,
    /// The calendar's name.
    pub name: String,
    pub summary: String,
    /// Local "HH:MM" on the event's first day; None for all-day events and
    /// on the later days of a multi-day event.
    pub time: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CalendarState {
    pub status: Status,
    pub calendars: Vec<CalendarSummary>,
    /// Local date ("YYYY-MM-DD") -> that day's events: all-day first, then
    /// timed events by start.
    pub by_date: BTreeMap<NaiveDate, Vec<DayEvent>>,
    pub updated: Option<DateTime<Utc>>,
}

impl CalendarState {
    pub fn starting(cfgs: &[CalConfig]) -> Self {
        CalendarState {
            status: if cfgs.is_empty() { Status::NoConfig } else { Status::Starting },
            calendars: cfgs
                .iter()
                .map(|c| CalendarSummary {
                    name: c.name.clone(),
                    color: color(c.color.as_deref()),
                    next: None,
                    error: None,
                })
                .collect(),
            by_date: BTreeMap::new(),
            updated: None,
        }
    }
}

/// One calendar's input to `build`.
pub struct Fetched<'a> {
    pub cfg: &'a CalConfig,
    pub instances: &'a [Instance],
    pub error: Option<String>,
}

/// Local calendar date of an instance's end, as pong: an all-day DTEND is
/// exclusive; a timed event ending exactly at midnight does not reach into
/// that day; no end means the start date.
fn last_day<L: TimeZone>(inst: &Instance, first: NaiveDate, local: &L) -> NaiveDate {
    let last = match &inst.end {
        None => first,
        Some(When::Date(d)) => *d - Span::days(1),
        Some(end) => {
            let e = end.to_local(local);
            let d = e.date_naive();
            if e.time() == NaiveTime::MIN && d > first { d - Span::days(1) } else { d }
        }
    };
    last.max(first)
}

/// Fold fetched instances into the window state. Pure: `now` and the local
/// zone are passed in.
pub fn build<L: TimeZone>(fetched: &[Fetched], now: DateTime<Utc>, local: &L) -> CalendarState
where
    L::Offset: std::fmt::Display,
{
    let mut by_date: BTreeMap<NaiveDate, Vec<DayEvent>> = BTreeMap::new();
    let mut calendars = Vec::new();

    for f in fetched {
        let col = color(f.cfg.color.as_deref());
        let mut next: Option<(DateTime<Utc>, NextEvent)> = None;

        for inst in f.instances {
            let start = inst.start.to_local(local);
            let first = match &inst.start {
                When::Date(d) => *d,
                _ => start.date_naive(),
            };
            let time = (!inst.start.is_date()).then(|| start.format("%H:%M").to_string());

            let mut day = first;
            while day <= last_day(inst, first, local) {
                by_date.entry(day).or_default().push(DayEvent {
                    color: col,
                    name: f.cfg.name.clone(),
                    summary: inst.summary.clone(),
                    time: if day == first { time.clone() } else { None },
                });
                day += Span::days(1);
            }

            let start_utc = start.with_timezone(&Utc);
            if start_utc >= now && next.as_ref().is_none_or(|(t, _)| start_utc < *t) {
                next = Some((
                    start_utc,
                    NextEvent {
                        start: start.to_rfc3339(),
                        all_day: inst.start.is_date(),
                        time: time.clone(),
                        summary: inst.summary.clone(),
                        location: inst.location.clone(),
                    },
                ));
            }
        }

        calendars.push(CalendarSummary {
            name: f.cfg.name.clone(),
            color: col,
            next: next.map(|(_, n)| n),
            error: f.error.clone(),
        });
    }

    // All-day context first ("this day is X"), then timed by start. Stable,
    // so ties keep calendar order.
    for events in by_date.values_mut() {
        events.sort_by(|a, b| (a.time.is_some(), &a.time).cmp(&(b.time.is_some(), &b.time)));
    }

    CalendarState {
        status: if fetched.is_empty() { Status::NoConfig } else { Status::Ok },
        calendars,
        by_date,
        updated: Some(now),
    }
}

// ---------------------------------------------------------------------------
// Network

/// Fetch and expand one calendar.
pub async fn fetch_one(client: &reqwest::Client, cfg: &CalConfig, now: DateTime<Utc>) -> Result<Vec<Instance>, String> {
    if cfg.url.trim().is_empty() {
        return Err("no url".into());
    }
    let text = client
        .get(cfg.url.trim())
        .timeout(TIMEOUT)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        // never echo the URL: it is a secret
        .map_err(|e| e.without_url().to_string())?
        .text()
        .await
        .map_err(|e| e.without_url().to_string())?;
    let events = ics::parse(&text);
    Ok(ics::expand(
        &events,
        now - Span::days(LOOKBACK_DAYS),
        now + Span::days(LOOKAHEAD_DAYS),
        &chrono::Local,
    ))
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Europe::London;

    const FIXTURE: &str = include_str!("../tests/fixtures/calendar.ics");

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn d(m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, day).unwrap()
    }

    fn state_at(now: &str) -> CalendarState {
        let now = utc(now);
        let events = ics::parse(FIXTURE);
        let inst = ics::expand(&events, now - Span::days(LOOKBACK_DAYS), now + Span::days(LOOKAHEAD_DAYS), &London);
        let cfg = CalConfig { name: "Jog".into(), url: String::new(), color: Some("pistachio".into()) };
        build(&[Fetched { cfg: &cfg, instances: &inst, error: None }], now, &London)
    }

    #[test]
    fn colours() {
        assert_eq!(color(Some("Mango")), Some([255, 173, 70]));
        assert_eq!(color(Some("#0a0B0c")), Some([10, 11, 12]));
        assert_eq!(color(Some("#123")), None);
        assert_eq!(color(Some("chartreuse")), None);
        assert_eq!(color(None), None);
    }

    #[test]
    fn next_event_is_the_soonest_upcoming() {
        // Fri 23 Oct 2026, 08:00 London (07:00Z)
        let s = state_at("2026-10-23T07:00:00Z");
        let next = s.calendars[0].next.as_ref().unwrap();
        assert_eq!(next.summary, "Dentist");
        // 14:00Z on 26 Oct: the clocks went back on the 25th, so London = UTC
        assert_eq!(next.time.as_deref(), Some("14:00"));
        assert!(!next.all_day);
        assert_eq!(s.calendars[0].color, Some([125, 188, 88]));
        assert_eq!(s.status, Status::Ok);
    }

    #[test]
    fn multi_day_events_land_on_every_day() {
        let s = state_at("2026-10-20T07:00:00Z");
        // holiday 28 Sep - 2 Oct is outside the 7-day lookback; the
        // conference spans 29-30 Oct with its time only on day one
        let conf = |day| {
            s.by_date.get(&day).unwrap().iter().find(|e| e.summary == "Conference").cloned().unwrap()
        };
        assert_eq!(conf(d(10, 29)).time.as_deref(), Some("09:00"));
        assert_eq!(conf(d(10, 30)).time, None);
        assert!(!s.by_date.get(&d(10, 31)).is_some_and(|v| v.iter().any(|e| e.summary == "Conference")));
    }

    #[test]
    fn ending_at_midnight_stays_on_one_day() {
        let s = state_at("2026-10-20T07:00:00Z");
        assert!(s.by_date[&d(10, 27)].iter().any(|e| e.summary == "Late shift"));
        assert!(!s.by_date.get(&d(10, 28)).is_some_and(|v| v.iter().any(|e| e.summary == "Late shift")));
    }

    #[test]
    fn all_day_end_is_exclusive() {
        let s = state_at("2026-10-01T07:00:00Z");
        let has = |day| s.by_date.get(&day).is_some_and(|v| v.iter().any(|e| e.summary == "Holiday"));
        assert!(has(d(9, 28)) && has(d(10, 2)));
        assert!(!has(d(10, 3)));
    }

    #[test]
    fn day_order_is_all_day_first_then_by_time() {
        let s = state_at("2026-10-01T07:00:00Z");
        // Fri 2 Oct: Holiday (all-day), Bins out (all-day), nothing timed;
        // Tue 27 Oct: Run club 07:00, Late shift 20:00
        let s27 = state_at("2026-10-26T07:00:00Z");
        let order: Vec<&str> = s27.by_date[&d(10, 27)].iter().map(|e| e.summary.as_str()).collect();
        assert_eq!(order, ["Run club", "Late shift"]);
        let fri: Vec<Option<&str>> = s.by_date[&d(10, 2)].iter().map(|e| e.time.as_deref()).collect();
        assert!(fri.iter().all(Option::is_none));
    }

    #[test]
    fn no_calendars_reads_as_no_config() {
        let s = build::<chrono_tz::Tz>(&[], utc("2026-10-23T07:00:00Z"), &London);
        assert_eq!(s.status, Status::NoConfig);
        assert_eq!(CalendarState::starting(&[]).status, Status::NoConfig);
    }

    #[test]
    fn duration_events_get_an_end() {
        let s = state_at("2026-10-20T07:00:00Z");
        let call = s.by_date[&d(10, 22)].iter().find(|e| e.summary == "Call").unwrap();
        assert_eq!(call.time.as_deref(), Some("11:00")); // 10:00Z in BST
    }
}
