//! Daemon status for the chain and network cards, ported from pong.
//!
//! Linux reads the local node stack the way the shell's startup report does:
//! `systemctl` state per unit and bitcoind's block height. (pong also
//! checked openclaw's port; dropped 2026-10-24 as unused, which leaves the
//! network card's third row free.) macOS has no systemd or local node, so it reads
//! `brew services` plus TCP port checks, and the results fill the two cards
//! in order. Every read is unprivileged and local-only; a probe that fails
//! reads as unknown ("—"), so one dead daemon never blanks a card.

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tokio::process::Command;

pub const REFRESH: Duration = Duration::from_secs(20);
#[cfg_attr(target_os = "macos", allow(dead_code))]
const TIMEOUT: Duration = Duration::from_secs(4);

/// verificationprogress at or above this reads as synced.
const BTC_SYNCED: f64 = 0.9999;

/// The value's colour, as a palette slot so it follows the theme.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Ok,
    Accent,
    Auburn,
    Muted,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Row {
    pub caption: String,
    pub value: String,
    pub tone: Tone,
}

fn row(caption: &str, value: impl Into<String>, tone: Tone) -> Row {
    Row { caption: caption.to_string(), value: value.into(), tone }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ServicesState {
    pub chain: Vec<Row>,
    pub net: Vec<Row>,
}

impl ServicesState {
    /// Before the first poll: captions with "—".
    pub fn starting() -> Self {
        if cfg!(target_os = "macos") {
            return ServicesState { chain: Vec::new(), net: Vec::new() };
        }
        let dash = |c: &str| row(c, "—", Tone::Muted);
        ServicesState {
            chain: vec![dash("BTCD"), dash("ELEC"), dash("BLOCK")],
            net: vec![dash("NGNX"), dash("TOR")],
        }
    }
}

// ---------------------------------------------------------------------------
// Turning probe output into rows (pure, tested)

/// systemd (ActiveState, SubState) -> row, as pong's _unit_state.
fn unit_row(caption: &str, active: &str, sub: &str) -> Row {
    match active {
        "active" if sub == "running" => row(caption, "UP", Tone::Ok),
        "active" => {
            let s: String = sub.chars().take(5).collect::<String>().to_uppercase();
            row(caption, if s.is_empty() { "ACTIVE".into() } else { s }, Tone::Accent)
        }
        "failed" => row(caption, "FAIL", Tone::Auburn),
        "inactive" => row(caption, "OFF", Tone::Muted),
        _ => row(caption, "—", Tone::Muted),
    }
}

/// A port probe: listening / not / unknown -> UP / DOWN / —. Used by the
/// macOS backend.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn probe_row(caption: &str, up: Option<bool>) -> Row {
    match up {
        Some(true) => row(caption, "UP", Tone::Ok),
        Some(false) => row(caption, "DOWN", Tone::Auburn),
        None => row(caption, "—", Tone::Muted),
    }
}

/// "932145" -> "932,145".
fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `bitcoin-cli getblockchaininfo` output -> the BLOCK row.
fn block_row(info: &str) -> Row {
    let parsed = serde_json::from_str::<Value>(info).ok().and_then(|v| {
        Some((v.get("blocks")?.as_u64()?, v.get("verificationprogress")?.as_f64()?))
    });
    match parsed {
        Some((blocks, progress)) => row(
            "BLOCK",
            thousands(blocks),
            if progress >= BTC_SYNCED { Tone::Ok } else { Tone::Accent },
        ),
        None => row("BLOCK", "—", Tone::Muted),
    }
}

// ---------------------------------------------------------------------------
// Probes

/// Run a short local command; trimmed stdout, or "" on any failure.
async fn probe(cmd: &str, args: &[&str], timeout: Duration) -> String {
    let run = Command::new(cmd).args(args).kill_on_drop(true).output();
    match tokio::time::timeout(timeout, run).await {
        Ok(Ok(out)) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => String::new(),
    }
}

#[cfg(not(target_os = "macos"))]
mod backend {
    use super::*;

    /// (caption, systemd unit) for the four unit rows.
    const UNITS: [(&str, &str); 4] =
        [("BTCD", "bitcoind"), ("ELEC", "electrs"), ("NGNX", "nginx"), ("TOR", "tor@default")];

    async fn unit(caption: &str, unit: &str) -> Row {
        let active = probe("systemctl", &["is-active", unit], TIMEOUT).await;
        let sub = probe("systemctl", &["show", unit, "--property=SubState", "--value"], TIMEOUT).await;
        unit_row(caption, if active.is_empty() { "unknown" } else { &active }, &sub)
    }

    pub async fn fetch() -> ServicesState {
        let [btcd, elec, ngnx, tor] = UNITS;
        let (btcd, elec, ngnx, tor, info) = tokio::join!(
            unit(btcd.0, btcd.1),
            unit(elec.0, elec.1),
            unit(ngnx.0, ngnx.1),
            unit(tor.0, tor.1),
            // finds its cookie under the default datadir, as the shell does
            probe("bitcoin-cli", &["getblockchaininfo"], TIMEOUT),
        );
        ServicesState {
            chain: vec![btcd, elec, block_row(&info)],
            net: vec![ngnx, tor],
        }
    }
}

#[cfg(target_os = "macos")]
mod backend {
    use super::*;

    const MAX_ROWS: usize = 6;
    /// TCP port checks for daemons brew does not manage: (caption, ports);
    /// a row reads UP if anything listens on any of them.
    const PORTS: [(&str, &[u16]); 2] = [("BTCD", &[8332, 8333]), ("ELEC", &[50001])];
    /// `brew services` starts Ruby: allow longer.
    const BREW_TIMEOUT: Duration = Duration::from_secs(8);

    pub async fn fetch() -> ServicesState {
        let raw = probe("brew", &["services", "list", "--json"], BREW_TIMEOUT).await;
        let mut rows = brew_rows(&raw, MAX_ROWS - PORTS.len());
        for (caption, ports) in PORTS {
            let mut up = false;
            for p in ports {
                let spec = format!("-iTCP:{p}");
                if !probe("lsof", &["-nP", &spec, "-sTCP:LISTEN"], TIMEOUT).await.is_empty() {
                    up = true;
                    break;
                }
            }
            rows.push(probe_row(caption, Some(up)));
        }
        let net = rows.split_off(rows.len().min(3));
        ServicesState { chain: rows, net }
    }
}

pub use backend::fetch;

/// `brew services list --json` -> rows, at most `cap`.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn brew_rows(raw: &str, cap: usize) -> Vec<Row> {
    let caption = |name: &str| -> String {
        match name {
            "tor" => "TOR",
            "mysql" => "MYSQL",
            "unbound" => "UNBND",
            "yubikey-agent" => "YUBI",
            "postgresql" => "PGSQL",
            "redis" => "REDIS",
            "nginx" => "NGNX",
            "dnsmasq" => "DNSMQ",
            _ => return name.chars().take(5).collect::<String>().to_uppercase(),
        }
        .to_string()
    };
    let list: Vec<Value> = serde_json::from_str(raw).unwrap_or_default();
    list.iter()
        .filter_map(|svc| {
            let name = svc.get("name")?.as_str().filter(|n| !n.is_empty())?;
            let status = svc.get("status").and_then(Value::as_str).unwrap_or("").to_lowercase();
            let (active, sub) = match status.as_str() {
                "started" => ("active", "running"),
                "none" | "stopped" => ("inactive", ""),
                "error" | "unknown" => ("failed", ""),
                _ => ("unknown", ""),
            };
            Some(unit_row(&caption(name), active, sub))
        })
        .take(cap)
        .collect()
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCKCHAININFO: &str = include_str!("../tests/fixtures/bitcoin-getblockchaininfo.json");
    const BREW: &str = include_str!("../tests/fixtures/brew-services.json");

    #[test]
    fn unit_states() {
        assert_eq!(unit_row("TOR", "active", "running"), row("TOR", "UP", Tone::Ok));
        assert_eq!(unit_row("TOR", "active", "exited"), row("TOR", "EXITE", Tone::Accent));
        assert_eq!(unit_row("TOR", "active", ""), row("TOR", "ACTIVE", Tone::Accent));
        assert_eq!(unit_row("TOR", "failed", "failed"), row("TOR", "FAIL", Tone::Auburn));
        assert_eq!(unit_row("TOR", "inactive", "dead"), row("TOR", "OFF", Tone::Muted));
        assert_eq!(unit_row("TOR", "unknown", ""), row("TOR", "—", Tone::Muted));
    }

    #[test]
    fn block_height() {
        assert_eq!(block_row(BLOCKCHAININFO), row("BLOCK", "932,145", Tone::Ok));
        let syncing = BLOCKCHAININFO.replace("0.99999812", "0.42");
        assert_eq!(block_row(&syncing).tone, Tone::Accent);
        assert_eq!(block_row(""), row("BLOCK", "—", Tone::Muted));
        assert_eq!(thousands(7), "7");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
    }

    #[test]
    fn port_probe() {
        assert_eq!(probe_row("ELEC", Some(true)), row("ELEC", "UP", Tone::Ok));
        assert_eq!(probe_row("ELEC", Some(false)), row("ELEC", "DOWN", Tone::Auburn));
        assert_eq!(probe_row("ELEC", None), row("ELEC", "—", Tone::Muted));
    }

    #[test]
    fn brew_services() {
        let rows = brew_rows(BREW, 4);
        assert_eq!(
            rows,
            vec![
                row("TOR", "UP", Tone::Ok),
                row("PGSQL", "OFF", Tone::Muted),
                row("UNBND", "FAIL", Tone::Auburn),
                row("SYNCT", "OFF", Tone::Muted),
            ]
        );
        assert!(brew_rows("not json", 4).is_empty());
    }
}
