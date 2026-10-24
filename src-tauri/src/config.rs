//! Per-OS config and cache folders, in the same places pong used:
//!
//! | | config | cache |
//! |---|---|---|
//! | Linux | `$XDG_CONFIG_HOME/ding` or `~/.config/ding` | `$XDG_CACHE_HOME/ding` or `~/.cache/ding` |
//! | macOS | `~/Library/Application Support/ding` | `~/Library/Caches/ding` |
//! | Windows | `%APPDATA%\ding` | `%LOCALAPPDATA%\ding` |
//!
//! Debug builds use `ding-dev` instead, so a dev run never touches the
//! installed app's state.

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(debug_assertions)]
const DIR_NAME: &str = "ding-dev";
#[cfg(not(debug_assertions))]
const DIR_NAME: &str = "ding";

/// The config files carried over from pong on first run.
const PONG_FILES: [&str; 3] = ["calendars.json", "weather.json", "theme.json"];

fn base_or_home(base: Option<PathBuf>, fallback: &str) -> PathBuf {
    base.or_else(|| dirs::home_dir().map(|h| h.join(fallback)))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_dir() -> PathBuf {
    base_or_home(dirs::config_dir(), ".config").join(DIR_NAME)
}

pub fn cache_dir() -> PathBuf {
    base_or_home(dirs::cache_dir(), ".cache").join(DIR_NAME)
}

fn pong_config_dir() -> PathBuf {
    base_or_home(dirs::config_dir(), ".config").join("pong")
}

/// Create both folders, then copy pong's config files over the first time
/// ding runs. Copies only files ding does not have yet, so it never
/// overwrites ding's own config, and leaves pong's copies where they are.
pub fn init() {
    let cfg = config_dir();
    let _ = fs::create_dir_all(&cfg);
    let _ = fs::create_dir_all(cache_dir());
    // Private, as pong's was: calendars.json holds secret ICS URLs.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&cfg, fs::Permissions::from_mode(0o700));
    }
    import_from(&pong_config_dir(), &cfg);
}

fn import_from(from: &Path, to: &Path) {
    for name in PONG_FILES {
        let src = from.join(name);
        let dst = to.join(name);
        if dst.exists() || !src.is_file() {
            continue;
        }
        // fs::copy keeps the permission bits, so calendars.json stays 0600:
        // its private ICS URLs work like passwords.
        match fs::copy(&src, &dst) {
            Ok(_) => eprintln!("[ding] imported {} from {}", name, from.display()),
            Err(e) => eprintln!("[ding] could not import {}: {e}", src.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ding-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn import_copies_missing_files_only() {
        let from = tmp("from");
        let to = tmp("to");
        fs::write(from.join("calendars.json"), "pong-cal").unwrap();
        fs::write(from.join("theme.json"), "pong-theme").unwrap();
        fs::write(to.join("theme.json"), "ding-theme").unwrap();

        import_from(&from, &to);

        assert_eq!(fs::read_to_string(to.join("calendars.json")).unwrap(), "pong-cal");
        // ding's own file wins
        assert_eq!(fs::read_to_string(to.join("theme.json")).unwrap(), "ding-theme");
        // absent in pong, so absent in ding
        assert!(!to.join("weather.json").exists());
        // pong's copies stay put
        assert!(from.join("calendars.json").exists());
    }
}
