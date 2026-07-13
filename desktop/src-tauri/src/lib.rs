mod commands;
mod error;
mod service;
mod state;

use state::DesktopState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopState::default())
        .invoke_handler(tauri::generate_handler![
            commands::select_database,
            commands::list_trip_summaries,
            commands::get_trip_detail,
            commands::get_day_timeline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
