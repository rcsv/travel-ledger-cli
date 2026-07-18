mod commands;
mod config;
mod error;
mod service;
mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = commands::init_desktop_state(app.handle()).map_err(|err| err.message)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::select_database,
            commands::restore_last_database,
            commands::forget_database,
            commands::list_trip_summaries,
            commands::get_trip_detail,
            commands::get_day_timeline,
            commands::create_trip,
            commands::create_itinerary,
            commands::update_itinerary,
            commands::reorder_itinerary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
