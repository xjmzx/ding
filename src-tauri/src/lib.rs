mod config;
mod theme;

use tauri::Manager;

/// This launch's theme, chosen once at startup.
struct Theme(&'static str);

#[tauri::command]
fn theme(state: tauri::State<Theme>) -> &'static str {
    state.0
}

/// Esc / Q in the window.
#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    config::init();
    let theme_name = theme::resolve();

    tauri::Builder::default()
        // Must be the first plugin: a second launch hands over and exits here.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .manage(Theme(theme_name))
        .invoke_handler(tauri::generate_handler![theme, quit])
        .run(tauri::generate_context!())
        .expect("error while running ding");
}
