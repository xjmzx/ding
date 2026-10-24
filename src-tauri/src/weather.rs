//! Weather and air quality from Open-Meteo, ported from pong.
//!
//! - Up to three cities from `weather.json`; the first is the primary and
//!   also drives sunrise/sunset and the 3-day forecast.
//! - One forecast request and one air-quality request per cycle cover every
//!   city (Open-Meteo takes comma-joined coordinate lists).
//! - Place names are geocoded once and cached in `geocode.json`; bundled
//!   cities and explicit coordinates need no geocoding at all.
//! - A city that fails a cycle keeps its previous reading.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::{cities, config};

pub const REFRESH: Duration = Duration::from_secs(30 * 60);
const TIMEOUT: Duration = Duration::from_secs(6);
const MAX_LOCATIONS: usize = 3;
const DEFAULT_LOCATIONS: [&str; 3] = ["Hanoi", "Edinburgh", "Beijing"];
/// US AQI band edges (EPA): good / moderate / unhealthy for sensitive
/// groups / unhealthy and worse.
const AQI_BANDS: [i64; 3] = [50, 100, 150];

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_URL: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";
const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

const CONFIG_COMMENT: &str = "Cities for the dashboard weather tile, up to three. \
An entry is either a plain string — a place name (\"Edinburgh\") or explicit \
coordinates (\"55.95,-3.19\") — or an object with any of name / \
latitude+longitude / label, where label is the text drawn in the tile. Run \
`ding --list-cities` for the bundled names: those resolve offline to pinned \
coordinates and a label sized to fit the tile. Any other name is geocoded once \
via Open-Meteo and cached; give coordinates instead to pin a name the geocoder \
gets wrong. The FIRST entry is the primary: it also drives the sunrise/sunset \
readout and the 3-day forecast. \"model\" is optional: empty lets Open-Meteo \
pick a regional model per location, or pin one (e.g. \"ukmo_seamless\") — see \
https://open-meteo.com/en/docs";

// ---------------------------------------------------------------------------
// Config

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    /// What gets resolved: a place name or "lat,lon".
    pub spec: String,
    /// What the tile draws.
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct WeatherConfig {
    pub locations: Vec<Location>,
    pub model: String,
}

pub fn config_path() -> std::path::PathBuf {
    config::config_dir().join("weather.json")
}

/// Write a template seeded with the defaults on first run. Seeded rather
/// than empty: an empty weather tile looks identical to a broken one.
pub fn ensure_config() {
    let path = config_path();
    if path.exists() {
        return;
    }
    let body = serde_json::json!({
        "_comment": CONFIG_COMMENT,
        "locations": DEFAULT_LOCATIONS,
        "model": "",
    });
    let _ = fs::write(&path, serde_json::to_string_pretty(&body).unwrap() + "\n");
}

pub fn load_config() -> WeatherConfig {
    parse_config(fs::read_to_string(config_path()).ok().as_deref())
}

fn parse_config(text: Option<&str>) -> WeatherConfig {
    let data: Option<Value> = text.and_then(|t| serde_json::from_str(t).ok());
    let mut locations: Vec<Location> = data
        .as_ref()
        .and_then(|d| d.get("locations"))
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(parse_entry).collect())
        .unwrap_or_default();
    let model = data
        .as_ref()
        .and_then(|d| d.get("model"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if locations.is_empty() {
        locations = DEFAULT_LOCATIONS
            .iter()
            .map(|c| Location { spec: c.to_string(), label: c.to_uppercase() })
            .collect();
    }
    locations.truncate(MAX_LOCATIONS);
    WeatherConfig { locations, model }
}

/// "21.03,105.83" (or wttr's "~21.03,105.83") -> (lat, lon). None for a
/// place name, which is the cue to geocode.
fn parse_latlon(spec: &str) -> Option<(f64, f64)> {
    let s = spec.trim().trim_start_matches('~').trim();
    let (a, b) = s.split_once(',')?;
    let num = |x: &str| -> Option<f64> {
        let x = x.trim();
        // digits with an optional sign and one optional decimal point, as
        // pong's regex: no exponents, no "inf".
        let body = x.strip_prefix('-').unwrap_or(x);
        let ok = !body.is_empty()
            && body.chars().all(|c| c.is_ascii_digit() || c == '.')
            && body.matches('.').count() <= 1
            && !body.starts_with('.')
            && !body.ends_with('.');
        if ok { x.parse().ok() } else { None }
    };
    let (lat, lon) = (num(a)?, num(b)?);
    ((-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon)).then_some((lat, lon))
}

/// A coordinate entry with no label: "39.9N,116.4E", which fits the tile.
fn compact_coord_label(lat: f64, lon: f64) -> String {
    format!(
        "{:.1}{},{:.1}{}",
        lat.abs(),
        if lat >= 0.0 { 'N' } else { 'S' },
        lon.abs(),
        if lon >= 0.0 { 'E' } else { 'W' }
    )
}

/// Python's str(float): "55.95" stays "55.95", "39.0" keeps its ".0".
fn py_float(x: f64) -> String {
    let s = format!("{x}");
    if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
}

/// One `locations` entry -> Location, as pong's _parse_weather_entry.
fn parse_entry(entry: &Value) -> Option<Location> {
    let obj = match entry {
        Value::String(s) => serde_json::json!({ "name": s }),
        Value::Object(_) => entry.clone(),
        _ => return None,
    };
    let name = match obj.get("name") {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string().trim().to_string(),
    };
    let as_f64 = |v: Option<&Value>| -> Option<f64> {
        match v? {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.trim().parse().ok(),
            _ => None,
        }
    };
    let (lat, lon) = (as_f64(obj.get("latitude")), as_f64(obj.get("longitude")));

    let mut spec = String::new();
    let mut derived = String::new();
    if let (Some(lat), Some(lon)) = (lat, lon) {
        spec = format!("{},{}", py_float(lat), py_float(lon));
        // Explicit coordinates win; the name, if any, only titles them.
        derived = match cities::find(&name).filter(|_| !name.is_empty()) {
            Some(c) => c.label.to_string(),
            None => name.to_uppercase(),
        };
    } else if !name.is_empty() {
        if let Some((la, lo)) = parse_latlon(&name) {
            spec = name.clone();
            derived = compact_coord_label(la, lo);
        } else if let Some(c) = cities::find(&name) {
            spec = format!("{},{}", py_float(c.lat), py_float(c.lon));
            derived = c.label.to_string();
        } else {
            spec = name.clone();
            derived = name.to_uppercase();
        }
    }
    if spec.is_empty() {
        return None;
    }
    if derived.is_empty() {
        derived = match parse_latlon(&spec) {
            Some((la, lo)) => compact_coord_label(la, lo),
            None => spec.to_uppercase(),
        };
    }
    let label = obj
        .get("label")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .unwrap_or(derived);
    Some(Location { spec, label })
}

// ---------------------------------------------------------------------------
// State sent to the window

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CityTemp {
    pub label: String,
    /// "29°C", or "" before the first reading.
    pub temp: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CityAqi {
    pub label: String,
    /// US AQI as shown, or "".
    pub text: String,
    /// 0 good, 1 moderate, 2 unhealthy for sensitive groups, 3 unhealthy+.
    pub band: Option<u8>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ForecastDay {
    /// "TODAY", then weekday abbreviations ("MON").
    pub day: String,
    /// "31/24", no unit.
    pub hilo: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WeatherState {
    pub cities: Vec<CityTemp>,
    pub aqi: Vec<CityAqi>,
    /// Primary city's sunrise and sunset, local "HH:MM".
    pub sun: Option<(String, String)>,
    pub forecast: Vec<ForecastDay>,
    /// Moon phase emoji, computed locally.
    pub moon: String,
    pub updated: Option<DateTime<Utc>>,
}

impl WeatherState {
    /// City names show before the first fetch lands.
    pub fn seeded(cfg: &WeatherConfig) -> Self {
        WeatherState {
            cities: cfg
                .locations
                .iter()
                .map(|l| CityTemp { label: l.label.clone(), temp: String::new() })
                .collect(),
            aqi: cfg
                .locations
                .iter()
                .map(|l| CityAqi { label: l.label.clone(), text: String::new(), band: None })
                .collect(),
            sun: None,
            forecast: Vec::new(),
            moon: moon_glyph(Utc::now()).to_string(),
            updated: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting (pure, tested)

const MOON_GLYPHS: [&str; 8] = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
const MOON_SYNODIC_DAYS: f64 = 29.530588853;

/// Moon phase from the mean synodic month since a known new moon
/// (2000-01-06 18:14 UTC). Survives a network outage.
pub fn moon_glyph(now: DateTime<Utc>) -> &'static str {
    let epoch = DateTime::parse_from_rfc3339("2000-01-06T18:14:00Z").unwrap();
    let days = (now - epoch.with_timezone(&Utc)).num_milliseconds() as f64 / 86_400_000.0;
    let frac = days.rem_euclid(MOON_SYNODIC_DAYS) / MOON_SYNODIC_DAYS;
    MOON_GLYPHS[((frac * 8.0 + 0.5) as usize) % 8]
}

fn fmt_temp(v: Option<&Value>) -> String {
    v.and_then(Value::as_f64).map(|t| format!("{t:.0}°C")).unwrap_or_default()
}

fn fmt_aqi(v: Option<&Value>) -> Option<(String, u8)> {
    // round half to even, as Python's round()
    let aqi = v?.as_f64()?.round_ties_even() as i64;
    let band = AQI_BANDS.iter().position(|edge| aqi <= *edge).unwrap_or(AQI_BANDS.len());
    Some((aqi.to_string(), band as u8))
}

fn forecast_rows(daily: Option<&Value>) -> Vec<ForecastDay> {
    let get = |k: &str| -> Vec<Value> {
        daily
            .and_then(|d| d.get(k))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let (times, highs, lows) = (get("time"), get("temperature_2m_max"), get("temperature_2m_min"));
    let mut out = Vec::new();
    for (i, day) in times.iter().take(3).enumerate() {
        let (Some(hi), Some(lo)) = (
            highs.get(i).and_then(Value::as_f64),
            lows.get(i).and_then(Value::as_f64),
        ) else {
            continue;
        };
        let label = if i == 0 {
            "TODAY".to_string()
        } else {
            day.as_str()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .map(|d| d.format("%a").to_string().to_uppercase())
                .unwrap_or_else(|| format!("+{i}"))
        };
        out.push(ForecastDay { day: label, hilo: format!("{hi:.0}/{lo:.0}") });
    }
    out
}

/// An Open-Meteo body: one object per point, or a bare object for one point.
fn as_list(v: Value) -> Option<Vec<Value>> {
    match v {
        Value::Array(a) => Some(a),
        Value::Object(_) => Some(vec![v]),
        _ => None,
    }
}

/// Fold one cycle's responses into the previous state. `point_of[i]` is the
/// index into the responses for location i, or None if it did not resolve.
fn apply(
    prev: &WeatherState,
    labels: &[String],
    point_of: &[Option<usize>],
    forecast: Option<&[Value]>,
    air: Option<&[Value]>,
    now: DateTime<Utc>,
) -> WeatherState {
    let entry = |list: Option<&[Value]>, i: usize| -> Option<Value> {
        list?.get(point_of.get(i).copied().flatten()?).cloned()
    };
    let prev_temp: HashMap<&str, &str> =
        prev.cities.iter().map(|c| (c.label.as_str(), c.temp.as_str())).collect();
    let prev_aqi: HashMap<&str, &CityAqi> =
        prev.aqi.iter().map(|c| (c.label.as_str(), c)).collect();

    let cities = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let fresh = entry(forecast, i)
                .map(|e| fmt_temp(e.pointer("/current/temperature_2m")))
                .unwrap_or_default();
            let temp = if fresh.is_empty() {
                prev_temp.get(label.as_str()).copied().unwrap_or("").to_string()
            } else {
                fresh
            };
            CityTemp { label: label.clone(), temp }
        })
        .collect();

    let mut sun = prev.sun.clone();
    let mut fc = prev.forecast.clone();
    if let Some(primary) = entry(forecast, 0) {
        let daily = primary.get("daily");
        let first = |k: &str| -> String {
            daily
                .and_then(|d| d.get(k))
                .and_then(|a| a.get(0))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        let (rise, set) = (first("sunrise"), first("sunset"));
        // ISO local time, "YYYY-MM-DDTHH:MM" -> "HH:MM"
        if rise.len() >= 16 && set.len() >= 16 {
            sun = Some((rise[11..16].to_string(), set[11..16].to_string()));
        }
        let rows = forecast_rows(daily);
        if !rows.is_empty() {
            fc = rows;
        }
    }

    let aqi = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            match entry(air, i).and_then(|e| fmt_aqi(e.pointer("/current/us_aqi"))) {
                Some((text, band)) => CityAqi { label: label.clone(), text, band: Some(band) },
                None => prev_aqi
                    .get(label.as_str())
                    .map(|c| (*c).clone())
                    .unwrap_or(CityAqi { label: label.clone(), text: String::new(), band: None }),
            }
        })
        .collect();

    WeatherState {
        cities,
        aqi,
        sun,
        forecast: fc,
        moon: moon_glyph(now).to_string(),
        updated: if forecast.is_some() || air.is_some() { Some(now) } else { prev.updated },
    }
}

// ---------------------------------------------------------------------------
// Network

fn load_geocode_cache(path: &Path) -> HashMap<String, (f64, f64)> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<HashMap<String, Vec<f64>>>(&s).ok())
        .map(|m| {
            m.into_iter()
                .filter(|(_, v)| v.len() == 2)
                .map(|(k, v)| (k, (v[0], v[1])))
                .collect()
        })
        .unwrap_or_default()
}

fn save_geocode_cache(path: &Path, cache: &HashMap<String, (f64, f64)>) {
    let m: HashMap<&String, [f64; 2]> = cache.iter().map(|(k, v)| (k, [v.0, v.1])).collect();
    if let Ok(s) = serde_json::to_string(&m) {
        let _ = fs::write(path, s);
    }
}

async fn get_json(client: &reqwest::Client, url: &str, query: &[(&str, String)]) -> Result<Value, String> {
    client
        .get(url)
        .query(query)
        .timeout(TIMEOUT)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())
}

async fn geocode(
    client: &reqwest::Client,
    spec: &str,
    cache: &mut HashMap<String, (f64, f64)>,
) -> Result<Option<(f64, f64)>, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Ok(None);
    }
    if let Some(p) = parse_latlon(spec) {
        return Ok(Some(p));
    }
    if let Some(p) = cache.get(&spec.to_lowercase()) {
        return Ok(Some(*p));
    }
    let q = [
        ("name", spec.to_string()),
        ("count", "1".into()),
        ("language", "en".into()),
        ("format", "json".into()),
    ];
    let data = get_json(client, GEOCODE_URL, &q).await?;
    let hit = data.pointer("/results/0").and_then(|r| {
        Some((r.get("latitude")?.as_f64()?, r.get("longitude")?.as_f64()?))
    });
    if let Some(p) = hit {
        cache.insert(spec.to_lowercase(), p);
    }
    Ok(hit)
}

fn coord_query(points: &[(f64, f64)]) -> Vec<(&'static str, String)> {
    let join = |f: fn(&(f64, f64)) -> f64| {
        points.iter().map(|p| format!("{:.4}", f(p))).collect::<Vec<_>>().join(",")
    };
    vec![
        ("latitude", join(|p| p.0)),
        ("longitude", join(|p| p.1)),
        ("timezone", "auto".into()),
    ]
}

/// One weather cycle. Never fails: problems are logged and the previous
/// readings carry over.
pub async fn fetch(client: &reqwest::Client, cfg: &WeatherConfig, prev: &WeatherState) -> WeatherState {
    let cache_path = config::cache_dir().join("geocode.json");
    let mut cache = load_geocode_cache(&cache_path);
    let before = cache.clone();

    let mut points = Vec::new();
    let mut point_of = Vec::new();
    for loc in &cfg.locations {
        match geocode(client, &loc.spec, &mut cache).await {
            Ok(Some(p)) => {
                point_of.push(Some(points.len()));
                points.push(p);
            }
            Ok(None) => {
                eprintln!("[ding] weather/geocode-miss loc={:?}", loc.spec);
                point_of.push(None);
            }
            Err(e) => {
                eprintln!("[ding] weather/geocode-fail loc={:?} err={e}", loc.spec);
                point_of.push(None);
            }
        }
    }
    if cache != before {
        save_geocode_cache(&cache_path, &cache);
    }

    let (mut forecast, mut air) = (None, None);
    if !points.is_empty() {
        let mut q = coord_query(&points);
        q.push(("current", "temperature_2m".into()));
        q.push(("daily", "temperature_2m_max,temperature_2m_min,sunrise,sunset".into()));
        q.push(("forecast_days", "3".into()));
        if !cfg.model.is_empty() {
            q.push(("models", cfg.model.clone()));
        }
        match get_json(client, FORECAST_URL, &q).await {
            Ok(v) => forecast = as_list(v),
            Err(e) => eprintln!("[ding] weather/fetch-fail n={} err={e}", points.len()),
        }
        // Separate host, separate failure: a dead AQI card must not cost
        // the temperatures.
        let mut q = coord_query(&points);
        q.push(("current", "us_aqi".into()));
        match get_json(client, AIR_URL, &q).await {
            Ok(v) => air = as_list(v),
            Err(e) => eprintln!("[ding] aqi/fetch-fail n={} err={e}", points.len()),
        }
    }

    let labels: Vec<String> = cfg.locations.iter().map(|l| l.label.clone()).collect();
    apply(prev, &labels, &point_of, forecast.as_deref(), air.as_deref(), Utc::now())
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const FORECAST: &str = include_str!("../tests/fixtures/open-meteo-forecast.json");
    const FORECAST_ONE: &str = include_str!("../tests/fixtures/open-meteo-forecast-single.json");
    const AIR: &str = include_str!("../tests/fixtures/open-meteo-air.json");

    fn labels(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn config_entries_match_pong() {
        let e = |v: Value| parse_entry(&v).unwrap();
        // bundled city: pinned coordinates + short label
        assert_eq!(e(json!("Edinburgh")), Location { spec: "55.9533,-3.1883".into(), label: "EDINB".into() });
        // unknown name: geocoded later, label is the upper-cased name
        assert_eq!(e(json!("Ulverston")), Location { spec: "Ulverston".into(), label: "ULVERSTON".into() });
        // coordinates as a string
        assert_eq!(e(json!("39.9,116.4")).label, "39.9N,116.4E");
        assert_eq!(e(json!("-33.87,151.21")).label, "33.9S,151.2E");
        // object with coordinates and a label
        let o = e(json!({"label": "beijing", "latitude": 39.9075, "longitude": 116.3972}));
        assert_eq!(o, Location { spec: "39.9075,116.3972".into(), label: "BEIJING".into() });
        // object with a bundled name and coordinates: coordinates win, preset label
        let o = e(json!({"name": "Hanoi", "latitude": 21.0, "longitude": 105.8}));
        assert_eq!(o, Location { spec: "21.0,105.8".into(), label: "HANOI".into() });
        // unusable
        assert!(parse_entry(&json!(42)).is_none());
        assert!(parse_entry(&json!({})).is_none());
    }

    #[test]
    fn config_falls_back_and_caps_at_three() {
        let c = parse_config(None);
        assert_eq!(c.locations.len(), 3);
        assert_eq!(c.locations[0].label, "HANOI");
        let c = parse_config(Some(r#"{"locations": ["Oslo","Rome","Lima","Quito"], "model": " ukmo_seamless "}"#));
        assert_eq!(c.locations.iter().map(|l| l.label.as_str()).collect::<Vec<_>>(), ["OSLO", "ROME", "LIMA"]);
        assert_eq!(c.model, "ukmo_seamless");
        let c = parse_config(Some(r#"{"locations": []}"#));
        assert_eq!(c.locations.len(), 3, "empty list falls back to defaults");
    }

    #[test]
    fn latlon_parsing() {
        assert_eq!(parse_latlon("21.03,105.83"), Some((21.03, 105.83)));
        assert_eq!(parse_latlon("~ -1.5 , 2"), Some((-1.5, 2.0)));
        assert_eq!(parse_latlon("91,0"), None);
        assert_eq!(parse_latlon("Edinburgh"), None);
        assert_eq!(parse_latlon("1e2,3"), None);
    }

    #[test]
    fn formatting() {
        assert_eq!(fmt_temp(Some(&json!(28.6))), "29°C");
        assert_eq!(fmt_temp(Some(&json!(2.5))), "2°C", "ties to even, as Python");
        assert_eq!(fmt_temp(None), "");
        assert_eq!(fmt_aqi(Some(&json!(50.0))), Some(("50".into(), 0)));
        assert_eq!(fmt_aqi(Some(&json!(50.5))), Some(("50".into(), 0)));
        assert_eq!(fmt_aqi(Some(&json!(101))), Some(("101".into(), 2)));
        assert_eq!(fmt_aqi(Some(&json!(320))), Some(("320".into(), 3)));
        assert_eq!(fmt_aqi(Some(&Value::Null)), None);
    }

    #[test]
    fn moon_phases() {
        let at = |s: &str| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc);
        assert_eq!(moon_glyph(at("2000-01-06T18:14:00Z")), "🌑");
        assert_eq!(moon_glyph(at("2000-01-21T12:00:00Z")), "🌕");
    }

    #[test]
    fn applies_a_multi_point_cycle() {
        let cfg = parse_config(Some(r#"{"locations": ["Hanoi", "Edinburgh", "Atlantis"]}"#));
        let prev = WeatherState::seeded(&cfg);
        let f = as_list(serde_json::from_str(FORECAST).unwrap()).unwrap();
        let a = as_list(serde_json::from_str(AIR).unwrap()).unwrap();
        // Atlantis did not geocode
        let s = apply(&prev, &labels(&["HANOI", "EDINB", "ATLANTIS"]), &[Some(0), Some(1), None], Some(&f), Some(&a), Utc::now());
        assert_eq!(s.cities.iter().map(|c| c.temp.as_str()).collect::<Vec<_>>(), ["31°C", "9°C", ""]);
        assert_eq!(s.sun, Some(("05:58".into(), "17:21".into())));
        assert_eq!(
            s.forecast,
            vec![
                ForecastDay { day: "TODAY".into(), hilo: "32/24".into() },
                ForecastDay { day: "SAT".into(), hilo: "30/23".into() },
                ForecastDay { day: "SUN".into(), hilo: "29/23".into() },
            ]
        );
        assert_eq!(s.aqi[0], CityAqi { label: "HANOI".into(), text: "154".into(), band: Some(3) });
        assert_eq!(s.aqi[1], CityAqi { label: "EDINB".into(), text: "18".into(), band: Some(0) });
        assert_eq!(s.aqi[2].band, None);
        assert!(s.updated.is_some());
    }

    #[test]
    fn failed_cycle_keeps_previous_readings() {
        let cfg = parse_config(Some(r#"{"locations": ["Hanoi", "Edinburgh"]}"#));
        let f = as_list(serde_json::from_str(FORECAST).unwrap()).unwrap();
        let a = as_list(serde_json::from_str(AIR).unwrap()).unwrap();
        let l = labels(&["HANOI", "EDINB"]);
        let good = apply(&WeatherState::seeded(&cfg), &l, &[Some(0), Some(1)], Some(&f), Some(&a), Utc::now());
        // network down: nothing came back
        let after = apply(&good, &l, &[Some(0), Some(1)], None, None, Utc::now());
        assert_eq!(after.cities, good.cities);
        assert_eq!(after.aqi, good.aqi);
        assert_eq!(after.sun, good.sun);
        assert_eq!(after.forecast, good.forecast);
        assert_eq!(after.updated, good.updated);
    }

    #[test]
    fn single_point_answer_is_a_bare_object() {
        let f = as_list(serde_json::from_str(FORECAST_ONE).unwrap()).unwrap();
        assert_eq!(f.len(), 1);
        let cfg = parse_config(Some(r#"{"locations": ["Oslo"]}"#));
        let s = apply(&WeatherState::seeded(&cfg), &labels(&["OSLO"]), &[Some(0)], Some(&f), None, Utc::now());
        assert_eq!(s.cities[0].temp, "-3°C");
    }
}
