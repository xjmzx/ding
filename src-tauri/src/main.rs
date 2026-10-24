// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == "--list-cities") {
        ding_lib::list_cities();
        return;
    }
    if std::env::args().any(|a| a == "--check") {
        ding_lib::check();
        return;
    }
    ding_lib::run()
}
