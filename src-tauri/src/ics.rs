//! A small iCalendar (RFC 5545) reader: enough for Google Calendar's private
//! ICS feeds. It does the job pong's `icalendar` + `recurring_ical_events`
//! did:
//!
//! - unfolds long lines and unescapes text;
//! - reads VEVENTs: start/end (all-day, UTC, with a TZID, or floating),
//!   DURATION, SUMMARY, LOCATION, RRULE, RDATE, EXDATE, RECURRENCE-ID,
//!   STATUS;
//! - expands repeating events (the `rrule` crate) inside a window, drops
//!   EXDATEs, and swaps in moved instances (RECURRENCE-ID overrides).
//!
//! Cancelled events and cancelled instances are left out. TZIDs are read as
//! IANA names (what Google sends); an unknown TZID is treated as local time.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

/// A DTSTART / DTEND / EXDATE / RECURRENCE-ID value.
#[derive(Debug, Clone, PartialEq)]
pub enum When {
    /// All-day.
    Date(NaiveDate),
    Utc(DateTime<Utc>),
    /// Wall-clock time in a named zone.
    Zoned(NaiveDateTime, chrono_tz::Tz),
    /// Wall-clock time with no zone: the viewer's local time.
    Floating(NaiveDateTime),
}

impl When {
    pub fn is_date(&self) -> bool {
        matches!(self, When::Date(_))
    }

    /// The instant, in the viewer's zone. An all-day date maps to local
    /// midnight.
    pub fn to_local<L: TimeZone>(&self, local: &L) -> DateTime<L> {
        let naive_local = |n: NaiveDateTime| {
            local
                .from_local_datetime(&n)
                .earliest()
                // a wall time inside a DST gap: step past it
                .unwrap_or_else(|| local.from_utc_datetime(&n))
        };
        match self {
            When::Date(d) => naive_local(d.and_time(NaiveTime::MIN)),
            When::Utc(t) => t.with_timezone(local),
            When::Zoned(n, tz) => tz
                .from_local_datetime(n)
                .earliest()
                .unwrap_or_else(|| tz.from_utc_datetime(n))
                .with_timezone(local),
            When::Floating(n) => naive_local(*n),
        }
    }

    /// Identity for matching an instance against EXDATE / RECURRENCE-ID.
    fn key(&self) -> Key {
        match self {
            When::Date(d) => Key::Date(*d),
            When::Utc(t) => Key::Ts(t.timestamp()),
            When::Zoned(..) => Key::Ts(self.to_local(&Utc).timestamp()),
            When::Floating(n) => Key::Ts(n.and_utc().timestamp()),
        }
    }

    /// Shift by a duration, keeping the kind (wall-clock for zoned and
    /// floating times, whole days for dates).
    fn plus(&self, d: Duration) -> When {
        match self {
            When::Date(x) => When::Date(*x + Duration::days(d.num_days())),
            When::Utc(t) => When::Utc(*t + d),
            When::Zoned(n, tz) => When::Zoned(*n + d, *tz),
            When::Floating(n) => When::Floating(*n + d),
        }
    }

    /// `other - self`, measured the same way `plus` adds.
    fn until(&self, other: &When) -> Duration {
        match (self, other) {
            (When::Date(a), When::Date(b)) => Duration::days((*b - *a).num_days()),
            (When::Zoned(a, _), When::Zoned(b, _)) | (When::Floating(a), When::Floating(b)) => *b - *a,
            _ => other.to_local(&Utc) - self.to_local(&Utc),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Date(NaiveDate),
    Ts(i64),
}

#[derive(Debug, Clone, Default)]
pub struct VEvent {
    pub uid: String,
    pub start: Option<When>,
    pub end: Option<When>,
    pub duration: Option<Duration>,
    pub summary: String,
    pub location: String,
    pub rrule: Option<String>,
    pub rdates: Vec<When>,
    pub exdates: Vec<When>,
    pub recurrence_id: Option<When>,
    pub cancelled: bool,
}

/// One occurrence of an event.
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub start: When,
    /// None when the event gives neither DTEND nor DURATION.
    pub end: Option<When>,
    pub summary: String,
    pub location: String,
}

// ---------------------------------------------------------------------------
// Parsing

/// Undo RFC 5545 line folding: a line starting with a space or tab
/// continues the one before it.
fn unfold(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = line.strip_prefix([' ', '\t']) {
            if let Some(last) = lines.last_mut() {
                last.push_str(rest);
                continue;
            }
        }
        if !line.is_empty() {
            lines.push(line.to_string());
        }
    }
    lines
}

/// A property's parameters: [(P1, a), (P2, b:c)].
type Params = Vec<(String, String)>;

/// "NAME;P1=a;P2=\"b:c\":value" -> (NAME, params, value).
fn split_property(line: &str) -> Option<(String, Params, String)> {
    let mut in_quotes = false;
    let colon = line.char_indices().find(|&(_, c)| {
        if c == '"' {
            in_quotes = !in_quotes;
        }
        c == ':' && !in_quotes
    })?;
    let (head, value) = (&line[..colon.0], &line[colon.0 + 1..]);
    let mut parts = head.split(';');
    let name = parts.next()?.trim().to_ascii_uppercase();
    let params = parts
        .filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            Some((k.trim().to_ascii_uppercase(), v.trim().trim_matches('"').to_string()))
        })
        .collect();
    Some((name, params, value.to_string()))
}

fn unescape(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn param<'a>(params: &'a [(String, String)], key: &str) -> Option<&'a str> {
    params.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

fn parse_zone(tzid: &str) -> Option<chrono_tz::Tz> {
    // some producers prefix the id with "/"
    tzid.trim_start_matches('/').parse().ok()
}

/// One date or date-time value.
fn parse_when(value: &str, tzid: Option<&str>) -> Option<When> {
    let v = value.trim();
    if v.len() == 8 {
        return NaiveDate::parse_from_str(v, "%Y%m%d").ok().map(When::Date);
    }
    let (body, zulu) = match v.strip_suffix('Z').or_else(|| v.strip_suffix('z')) {
        Some(b) => (b, true),
        None => (v, false),
    };
    let n = NaiveDateTime::parse_from_str(body, "%Y%m%dT%H%M%S").ok()?;
    Some(if zulu {
        When::Utc(n.and_utc())
    } else if let Some(tz) = tzid.and_then(parse_zone) {
        When::Zoned(n, tz)
    } else {
        When::Floating(n)
    })
}

/// A property that may hold a comma-separated list (EXDATE, RDATE).
fn parse_when_list(value: &str, params: &[(String, String)]) -> Vec<When> {
    if param(params, "VALUE") == Some("PERIOD") {
        return Vec::new(); // RDATE periods: not used by Google
    }
    value.split(',').filter_map(|v| parse_when(v, param(params, "TZID"))).collect()
}

/// "P1D", "PT1H30M", "P2W", "-PT15M" (sign ignored: an event can't end
/// before it starts).
fn parse_duration(v: &str) -> Option<Duration> {
    let v = v.trim().trim_start_matches(['+', '-']);
    let v = v.strip_prefix('P')?;
    let mut total = Duration::zero();
    let mut num = String::new();
    let mut in_time = false;
    for c in v.chars() {
        match c {
            'T' => in_time = true,
            '0'..='9' => num.push(c),
            _ => {
                let n: i64 = num.parse().ok()?;
                num.clear();
                total += match (c, in_time) {
                    ('W', false) => Duration::weeks(n),
                    ('D', false) => Duration::days(n),
                    ('H', true) => Duration::hours(n),
                    ('M', true) => Duration::minutes(n),
                    ('S', true) => Duration::seconds(n),
                    _ => return None,
                };
            }
        }
    }
    num.is_empty().then_some(total)
}

pub fn parse(text: &str) -> Vec<VEvent> {
    let mut events = Vec::new();
    let mut cur: Option<VEvent> = None;
    // depth inside the current VEVENT (VALARM etc. nest inside it)
    let mut nested = 0usize;
    for line in unfold(text) {
        let Some((name, params, value)) = split_property(&line) else { continue };
        match (name.as_str(), value.trim().to_ascii_uppercase().as_str()) {
            ("BEGIN", "VEVENT") if cur.is_none() => {
                cur = Some(VEvent::default());
                nested = 0;
                continue;
            }
            ("END", "VEVENT") if nested == 0 => {
                if let Some(ev) = cur.take() {
                    events.push(ev);
                }
                continue;
            }
            ("BEGIN", _) if cur.is_some() => {
                nested += 1;
                continue;
            }
            ("END", _) if cur.is_some() => {
                nested = nested.saturating_sub(1);
                continue;
            }
            _ => {}
        }
        let Some(ev) = cur.as_mut() else { continue };
        if nested > 0 {
            continue; // a property of a VALARM, not of the event
        }
        let tzid = param(&params, "TZID");
        match name.as_str() {
            "UID" => ev.uid = value.trim().to_string(),
            "DTSTART" => ev.start = parse_when(&value, tzid),
            "DTEND" => ev.end = parse_when(&value, tzid),
            "DURATION" => ev.duration = parse_duration(&value),
            "SUMMARY" => ev.summary = unescape(&value).trim().to_string(),
            "LOCATION" => ev.location = unescape(&value).trim().to_string(),
            "RRULE" => ev.rrule = Some(value.trim().to_string()),
            "RDATE" => ev.rdates.extend(parse_when_list(&value, &params)),
            "EXDATE" => ev.exdates.extend(parse_when_list(&value, &params)),
            "RECURRENCE-ID" => ev.recurrence_id = parse_when(&value, tzid),
            "STATUS" => ev.cancelled = value.trim().eq_ignore_ascii_case("CANCELLED"),
            _ => {}
        }
    }
    events
}

// ---------------------------------------------------------------------------
// Expansion

/// Cap on occurrences per repeating event, as a guard against a runaway
/// rule. A daily event over ten years is ~3,650.
const MAX_OCCURRENCES: usize = 20_000;

/// The `rrule` crate wants UNTIL in UTC (with a `Z`) whenever DTSTART has a
/// zone; Google writes a bare date for all-day series. Rewrite UNTIL to the
/// UTC end of that day / that wall time in the event's zone.
fn normalise_until(rule: &str, start: &When) -> String {
    rule.split(';')
        .map(|part| {
            let Some(val) = part.strip_prefix("UNTIL=").or_else(|| part.strip_prefix("until=")) else {
                return part.to_string();
            };
            if val.ends_with('Z') || val.ends_with('z') {
                return part.to_string();
            }
            let wall = if val.len() == 8 {
                NaiveDate::parse_from_str(val, "%Y%m%d").ok().map(|d| d.and_hms_opt(23, 59, 59).unwrap())
            } else {
                NaiveDateTime::parse_from_str(val, "%Y%m%dT%H%M%S").ok()
            };
            let Some(wall) = wall else { return part.to_string() };
            let utc = match start {
                When::Zoned(_, tz) => When::Zoned(wall, *tz).to_local(&Utc),
                _ => wall.and_utc(), // dates and floating times expand on UTC
            };
            format!("UNTIL={}", utc.format("%Y%m%dT%H%M%SZ"))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Occurrence starts of a repeating event, up to `until` (UTC). Falls back
/// to the first occurrence alone if the rule cannot be read.
fn expand_rule(start: &When, rule: &str, until: DateTime<Utc>) -> Vec<When> {
    use rrule::{RRule, Tz as RTz, Unvalidated};

    // Dates and floating times expand on UTC, which has no DST, so whole
    // days and wall-clock times survive; they are mapped back afterwards.
    let dt_start: DateTime<RTz> = match start {
        When::Date(d) => RTz::UTC.from_utc_datetime(&d.and_time(NaiveTime::MIN)),
        When::Utc(t) => t.with_timezone(&RTz::UTC),
        When::Floating(n) => RTz::UTC.from_utc_datetime(n),
        When::Zoned(n, tz) => {
            let z = RTz::Tz(*tz);
            match z.from_local_datetime(n).earliest() {
                Some(t) => t,
                None => z.from_utc_datetime(n),
            }
        }
    };

    let set = normalise_until(rule, start)
        .parse::<RRule<Unvalidated>>()
        .map_err(|e| e.to_string())
        .and_then(|r| r.build(dt_start).map_err(|e| e.to_string()));
    let set = match set {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[udash] calendar/rrule-unreadable rule={rule:?} err={e}");
            return vec![start.clone()];
        }
    };

    (&set)
        .into_iter()
        .take_while(|t| t.with_timezone(&Utc) <= until)
        .take(MAX_OCCURRENCES)
        .map(|t| match start {
            When::Date(_) => When::Date(t.date_naive()),
            When::Utc(_) => When::Utc(t.with_timezone(&Utc)),
            When::Floating(_) => When::Floating(t.naive_utc()),
            When::Zoned(_, tz) => When::Zoned(t.with_timezone(tz).naive_local(), *tz),
        })
        .collect()
}

/// An instance's end: DTEND, else DTSTART + DURATION, else (all-day) the
/// next day, else none.
fn end_for(ev: &VEvent, start: &When) -> Option<When> {
    match (&ev.end, ev.duration) {
        (Some(e), _) => Some(e.clone()),
        (None, Some(d)) => Some(start.plus(d)),
        (None, None) if start.is_date() => Some(start.plus(Duration::days(1))),
        _ => None,
    }
}

fn overlaps<L: TimeZone>(inst: &Instance, win_start: DateTime<Utc>, win_end: DateTime<Utc>, local: &L) -> bool {
    let s = inst.start.to_local(local).with_timezone(&Utc);
    let e = inst.end.as_ref().map(|e| e.to_local(local).with_timezone(&Utc)).unwrap_or(s);
    s < win_end && (e > win_start || s >= win_start)
}

/// Every occurrence overlapping [win_start, win_end), in no set order.
pub fn expand<L: TimeZone>(
    events: &[VEvent],
    win_start: DateTime<Utc>,
    win_end: DateTime<Utc>,
    local: &L,
) -> Vec<Instance> {
    // Moved / cancelled single occurrences, by series UID.
    let mut overridden: HashMap<&str, HashSet<Key>> = HashMap::new();
    for ev in events {
        if let Some(rid) = &ev.recurrence_id {
            overridden.entry(ev.uid.as_str()).or_default().insert(rid.key());
        }
    }

    let mut out = Vec::new();
    for ev in events {
        let Some(start) = &ev.start else { continue };
        if ev.cancelled {
            continue;
        }
        if ev.recurrence_id.is_some() {
            // An override stands on its own start and end.
            out.push(Instance {
                start: start.clone(),
                end: end_for(ev, start),
                summary: ev.summary.clone(),
                location: ev.location.clone(),
            });
            continue;
        }

        let length = end_for(ev, start).map(|e| start.until(&e));
        let mut starts = match &ev.rrule {
            Some(rule) => expand_rule(start, rule, win_end),
            None => vec![start.clone()],
        };
        starts.extend(ev.rdates.iter().cloned());

        let excluded: HashSet<Key> = ev.exdates.iter().map(When::key).collect();
        let moved = overridden.get(ev.uid.as_str());
        let mut seen = HashSet::new();
        for s in starts {
            let k = s.key();
            if excluded.contains(&k) || moved.is_some_and(|m| m.contains(&k)) || !seen.insert(k) {
                continue;
            }
            out.push(Instance {
                end: length.map(|d| s.plus(d)),
                start: s,
                summary: ev.summary.clone(),
                location: ev.location.clone(),
            });
        }
    }
    out.retain(|i| overlaps(i, win_start, win_end, local));
    out
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

    fn by_summary<'a>(v: &'a [Instance], s: &str) -> Vec<&'a Instance> {
        let mut out: Vec<&Instance> = v.iter().filter(|i| i.summary == s).collect();
        out.sort_by_key(|i| i.start.to_local(&Utc));
        out
    }

    #[test]
    fn unfolds_and_unescapes() {
        let evs = parse(FIXTURE);
        let long = evs.iter().find(|e| e.uid == "folded@udash.test").unwrap();
        assert_eq!(long.summary, "A long summary that Google folds across lines, with commas; and semicolons");
        assert_eq!(long.location, "Room 4, Level 2");
    }

    #[test]
    fn alarms_do_not_leak_into_the_event() {
        let evs = parse(FIXTURE);
        let ev = evs.iter().find(|e| e.uid == "alarm@udash.test").unwrap();
        assert_eq!(ev.summary, "Dentist");
    }

    #[test]
    fn weekly_series_with_exdate_and_moved_instance() {
        let evs = parse(FIXTURE);
        let got = expand(&evs, utc("2026-10-01T00:00:00Z"), utc("2026-11-30T00:00:00Z"), &London);
        let runs = by_summary(&got, "Run club");
        let starts: Vec<String> = runs.iter().map(|i| i.start.to_local(&London).format("%m-%d %H:%M").to_string()).collect();
        // Tuesdays 07:00 London from 6 Oct; 20 Oct excluded; 3 Nov moved to
        // the Wednesday by an override; series ends 24 Nov (UNTIL).
        assert_eq!(starts, ["10-06 07:00", "10-13 07:00", "10-27 07:00", "11-10 07:00", "11-17 07:00", "11-24 07:00"]);
        let moved = by_summary(&got, "Run club (moved)");
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].start.to_local(&London).format("%m-%d %H:%M").to_string(), "11-04 07:30");
    }

    #[test]
    fn wall_clock_holds_across_dst() {
        // The UK leaves BST on 25 Oct 2026: 07:00 London is 06:00Z before
        // and 07:00Z after.
        let evs = parse(FIXTURE);
        let got = expand(&evs, utc("2026-10-01T00:00:00Z"), utc("2026-11-30T00:00:00Z"), &London);
        let runs = by_summary(&got, "Run club");
        assert_eq!(runs[0].start.to_local(&Utc).format("%H:%M").to_string(), "06:00");
        assert_eq!(runs[3].start.to_local(&Utc).format("%H:%M").to_string(), "07:00");
    }

    #[test]
    fn cancelled_instances_and_events_are_dropped() {
        let evs = parse(FIXTURE);
        let got = expand(&evs, utc("2026-10-01T00:00:00Z"), utc("2026-11-30T00:00:00Z"), &London);
        // the weekly standup's 15 Oct instance is cancelled by an override
        let standups = by_summary(&got, "Standup");
        assert!(standups.iter().all(|i| i.start.to_local(&London).date_naive() != NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()));
        assert!(by_summary(&got, "Called off").is_empty());
    }

    #[test]
    fn all_day_series_with_date_until() {
        let evs = parse(FIXTURE);
        let got = expand(&evs, utc("2026-10-01T00:00:00Z"), utc("2026-11-30T00:00:00Z"), &London);
        let bins: Vec<NaiveDate> = by_summary(&got, "Bins out")
            .iter()
            .map(|i| match i.start {
                When::Date(d) => d,
                _ => panic!("all-day stays a date"),
            })
            .collect();
        // every 2 weeks from Fri 2 Oct, UNTIL=20261113 (inclusive)
        let d = |m, day| NaiveDate::from_ymd_opt(2026, m, day).unwrap();
        assert_eq!(bins, [d(10, 2), d(10, 16), d(10, 30), d(11, 13)]);
    }

    #[test]
    fn window_keeps_overlapping_events() {
        let evs = parse(FIXTURE);
        // the holiday runs 28 Sep - 2 Oct; a window starting 1 Oct still sees it
        let got = expand(&evs, utc("2026-10-01T00:00:00Z"), utc("2026-10-10T00:00:00Z"), &London);
        assert_eq!(by_summary(&got, "Holiday").len(), 1);
        // a window that ends before it does not
        let got = expand(&evs, utc("2026-09-01T00:00:00Z"), utc("2026-09-20T00:00:00Z"), &London);
        assert!(by_summary(&got, "Holiday").is_empty());
    }

    #[test]
    fn duration_and_formats() {
        assert_eq!(parse_duration("PT1H30M"), Some(Duration::minutes(90)));
        assert_eq!(parse_duration("P1W"), Some(Duration::weeks(1)));
        assert_eq!(parse_duration("P1DT2H"), Some(Duration::hours(26)));
        assert_eq!(parse_duration("1H"), None);
        assert_eq!(parse_when("20261023", None), Some(When::Date(NaiveDate::from_ymd_opt(2026, 10, 23).unwrap())));
        assert!(matches!(parse_when("20261023T090000", Some("Europe/London")), Some(When::Zoned(..))));
        assert!(matches!(parse_when("20261023T090000", Some("Not/AZone")), Some(When::Floating(_))));
        assert!(matches!(parse_when("20261023T090000Z", None), Some(When::Utc(_))));
    }

    #[test]
    fn until_is_normalised_to_utc() {
        let start = When::Zoned(NaiveDate::from_ymd_opt(2026, 10, 6).unwrap().and_hms_opt(7, 0, 0).unwrap(), London);
        assert_eq!(normalise_until("FREQ=WEEKLY;UNTIL=20261124T070000", &start), "FREQ=WEEKLY;UNTIL=20261124T070000Z");
        assert_eq!(normalise_until("FREQ=WEEKLY;UNTIL=20260801T070000", &start), "FREQ=WEEKLY;UNTIL=20260801T060000Z");
        let day = When::Date(NaiveDate::from_ymd_opt(2026, 10, 2).unwrap());
        assert_eq!(normalise_until("FREQ=WEEKLY;UNTIL=20261113", &day), "FREQ=WEEKLY;UNTIL=20261113T235959Z");
        assert_eq!(normalise_until("FREQ=DAILY;COUNT=3", &day), "FREQ=DAILY;COUNT=3");
    }
}
