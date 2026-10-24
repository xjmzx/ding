mod calendar;
mod cities;
mod config;
mod ics;
mod services;
mod theme;
mod weather;

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// This launch's theme, chosen once at startup.
struct Theme(&'static str);

/// Everything the dashboard shows, kept up to date by the timers below.
#[derive(Clone, Serialize)]
struct Snapshot {
    weather: weather::WeatherState,
    calendar: calendar::CalendarState,
    services: services::ServicesState,
}

type Shared = Arc<Mutex<Snapshot>>;

#[tauri::command]
fn theme(state: tauri::State<Theme>) -> &'static str {
    state.0
}

/// The current state, for a window that has just opened. Updates after
/// that arrive as `weather`, `calendar` and `services` events.
#[tauri::command]
fn snapshot(state: tauri::State<Shared>) -> Snapshot {
    state.lock().unwrap().clone()
}

/// Esc / Q in the window.
#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

/// `ding --list-cities`, handled before any window opens.
pub fn list_cities() {
    cities::print_list(&weather::config_path());
}

/// `ding --check`: run each fetch once and print the snapshot as JSON,
/// with no window. For debugging config and connectivity. Uses the same
/// config folders as the app (`ding-dev` in a debug build).
pub fn check() {
    config::init();
    weather::ensure_config();
    calendar::ensure_config();
    let weather_cfg = weather::load_config();
    let cal_cfgs = calendar::load_config();
    let client = reqwest::Client::builder()
        .user_agent(concat!("ding/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("http client");

    let snap = tauri::async_runtime::block_on(async {
        let now = chrono::Utc::now();
        let weather = weather::fetch(&client, &weather_cfg, &weather::WeatherState::seeded(&weather_cfg)).await;
        let mut fetched_inst = Vec::new();
        for cfg in &cal_cfgs {
            fetched_inst.push(calendar::fetch_one(&client, cfg, now).await);
        }
        let fetched: Vec<calendar::Fetched> = cal_cfgs
            .iter()
            .zip(&fetched_inst)
            .map(|(cfg, r)| calendar::Fetched {
                cfg,
                instances: r.as_deref().unwrap_or(&[]),
                error: r.as_ref().err().cloned(),
            })
            .collect();
        Snapshot {
            weather,
            calendar: calendar::build(&fetched, now, &chrono::Local),
            services: services::fetch().await,
        }
    });
    println!("{}", serde_json::to_string_pretty(&snap).unwrap());
}

/// Start the three polling loops. Config is read once, at startup, as pong
/// did: edit the files and restart.
fn start_timers(app: &AppHandle, shared: Shared, weather_cfg: weather::WeatherConfig, cal_cfgs: Vec<calendar::CalConfig>) {
    let client = reqwest::Client::builder()
        .user_agent(concat!("ding/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("http client");

    // weather + air quality
    {
        let (app, shared, client) = (app.clone(), shared.clone(), client.clone());
        tauri::async_runtime::spawn(async move {
            loop {
                let prev = shared.lock().unwrap().weather.clone();
                let next = weather::fetch(&client, &weather_cfg, &prev).await;
                shared.lock().unwrap().weather = next.clone();
                let _ = app.emit("weather", next);
                tokio::time::sleep(weather::REFRESH).await;
            }
        });
    }

    // calendars
    {
        let (app, shared) = (app.clone(), shared.clone());
        tauri::async_runtime::spawn(async move {
            if cal_cfgs.is_empty() {
                // Nothing to poll. Logged once so an unconfigured calendar
                // is not mistaken for a quiet week.
                eprintln!(
                    "[ding] calendar/no-config path={} — no calendars configured; the calendar stays empty",
                    calendar::config_path().display()
                );
                return;
            }
            // Last good events per calendar, kept through a failed fetch.
            let mut last: Vec<Vec<ics::Instance>> = vec![Vec::new(); cal_cfgs.len()];
            loop {
                let now = chrono::Utc::now();
                let mut errors = Vec::with_capacity(cal_cfgs.len());
                for (i, cfg) in cal_cfgs.iter().enumerate() {
                    match calendar::fetch_one(&client, cfg, now).await {
                        Ok(inst) => {
                            last[i] = inst;
                            errors.push(None);
                        }
                        Err(e) => {
                            eprintln!("[ding] calendar/fetch-fail name={:?} err={e}", cfg.name);
                            errors.push(Some(e));
                        }
                    }
                }
                let fetched: Vec<calendar::Fetched> = cal_cfgs
                    .iter()
                    .zip(&last)
                    .zip(errors)
                    .map(|((cfg, inst), error)| calendar::Fetched { cfg, instances: inst, error })
                    .collect();
                let next = calendar::build(&fetched, now, &chrono::Local);
                shared.lock().unwrap().calendar = next.clone();
                let _ = app.emit("calendar", next);
                tokio::time::sleep(calendar::REFRESH).await;
            }
        });
    }

    // daemons
    {
        let (app, shared) = (app.clone(), shared);
        tauri::async_runtime::spawn(async move {
            loop {
                let next = services::fetch().await;
                shared.lock().unwrap().services = next.clone();
                let _ = app.emit("services", next);
                tokio::time::sleep(services::REFRESH).await;
            }
        });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    config::init();
    let theme_name = theme::resolve();
    weather::ensure_config();
    calendar::ensure_config();
    let weather_cfg = weather::load_config();
    let cal_cfgs = calendar::load_config();

    let shared: Shared = Arc::new(Mutex::new(Snapshot {
        weather: weather::WeatherState::seeded(&weather_cfg),
        calendar: calendar::CalendarState::starting(&cal_cfgs),
        services: services::ServicesState::starting(),
    }));

    tauri::Builder::default()
        // Must be the first plugin: a second launch hands over and exits here.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .manage(Theme(theme_name))
        .manage(shared.clone())
        .setup(move |app| {
            start_timers(app.handle(), shared, weather_cfg, cal_cfgs);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![theme, snapshot, quit])
        .run(tauri::generate_context!())
        .expect("error while running ding");
}
