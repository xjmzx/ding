//! fizx / upleb theme choice, the same rules as pong:
//!
//! - `theme.json` `{"mode": "fizx" | "upleb" | "alternate"}` in the config
//!   folder. A pinned theme wins.
//! - `alternate` (the default) flips the theme on every launch, using a
//!   one-word `theme` marker in the cache folder. The very first launch
//!   lands on fizx.

use std::fs;
use std::path::Path;

use crate::config;

const THEMES: [&str; 2] = ["fizx", "upleb"];

const TEMPLATE: &str = r#"{
  "_comment": "\"mode\": \"alternate\" flips fizx/upleb on each launch (default). Use \"fizx\" or \"upleb\" to pin one scheme.",
  "mode": "alternate"
}
"#;

/// Write the default `theme.json` if there is none, then pick this launch's
/// theme. Called once at startup, so reloading the page keeps the theme.
pub fn resolve() -> &'static str {
    let cfg = config::config_dir().join("theme.json");
    if !cfg.exists() {
        let _ = fs::write(&cfg, TEMPLATE);
    }
    resolve_with(&cfg, &config::cache_dir().join("theme"))
}

fn resolve_with(config_file: &Path, marker: &Path) -> &'static str {
    let mode = fs::read_to_string(config_file)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("mode").and_then(|m| m.as_str()).map(str::to_owned))
        .unwrap_or_else(|| "alternate".into());

    if let Some(pinned) = THEMES.iter().find(|t| **t == mode) {
        return pinned;
    }

    // No marker yet reads as "upleb last time", so the first launch is fizx.
    let last = fs::read_to_string(marker).unwrap_or_default();
    let current = if last.trim() == "fizx" { "upleb" } else { "fizx" };
    let _ = fs::write(marker, current);
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ding-theme-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn alternates_starting_with_fizx() {
        let d = tmp("alt");
        let (cfg, marker) = (d.join("theme.json"), d.join("theme"));
        fs::write(&cfg, TEMPLATE).unwrap();
        assert_eq!(resolve_with(&cfg, &marker), "fizx");
        assert_eq!(resolve_with(&cfg, &marker), "upleb");
        assert_eq!(resolve_with(&cfg, &marker), "fizx");
    }

    #[test]
    fn pinned_theme_wins_and_leaves_marker_alone() {
        let d = tmp("pin");
        let (cfg, marker) = (d.join("theme.json"), d.join("theme"));
        fs::write(&cfg, r#"{"mode": "upleb"}"#).unwrap();
        assert_eq!(resolve_with(&cfg, &marker), "upleb");
        assert_eq!(resolve_with(&cfg, &marker), "upleb");
        assert!(!marker.exists());
    }

    #[test]
    fn missing_or_broken_config_alternates() {
        let d = tmp("broken");
        let (cfg, marker) = (d.join("theme.json"), d.join("theme"));
        assert_eq!(resolve_with(&cfg, &marker), "fizx");
        fs::write(&cfg, "not json").unwrap();
        assert_eq!(resolve_with(&cfg, &marker), "upleb");
    }
}
