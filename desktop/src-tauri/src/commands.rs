use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::config::settings_file_path;
use crate::error::DesktopError;
use crate::service::{self, DatabaseInfo, RestoreLastDatabaseResult};
use crate::state::DesktopState;
use travel_ledger_cli::{
    CreateItineraryParams, CreateItineraryResult, CreateTripParams, CreateTripResult, DayDetail,
    TripDetail, TripSummary, UpdateItineraryParams, UpdateItineraryResult,
};

pub fn init_desktop_state(app: &AppHandle) -> Result<DesktopState, DesktopError> {
    let config_dir = app.path().app_config_dir().map_err(|err| {
        DesktopError::database_config_write_failed(format!(
            "Failed to resolve app config directory: {err}"
        ))
    })?;
    std::fs::create_dir_all(&config_dir).map_err(|err| {
        DesktopError::database_config_write_failed(format!(
            "Failed to create app config directory: {err}"
        ))
    })?;
    Ok(DesktopState::new(settings_file_path(&config_dir)))
}

#[tauri::command]
pub fn select_database(
    path: String,
    state: State<DesktopState>,
) -> Result<DatabaseInfo, DesktopError> {
    service::select_database(&state, PathBuf::from(path))
}

#[tauri::command]
pub fn restore_last_database(
    state: State<DesktopState>,
) -> Result<RestoreLastDatabaseResult, DesktopError> {
    service::restore_last_database(&state)
}

#[tauri::command]
pub fn forget_database(state: State<DesktopState>) -> Result<(), DesktopError> {
    service::forget_database(&state)
}

#[tauri::command]
pub fn list_trip_summaries(state: State<DesktopState>) -> Result<Vec<TripSummary>, DesktopError> {
    service::list_trip_summaries(&state)
}

#[tauri::command]
pub fn get_trip_detail(
    trip_id: i64,
    state: State<DesktopState>,
) -> Result<TripDetail, DesktopError> {
    service::get_trip_detail(&state, trip_id)
}

#[tauri::command]
pub fn get_day_timeline(
    trip_id: i64,
    day_number: i64,
    state: State<DesktopState>,
) -> Result<DayDetail, DesktopError> {
    service::get_day_timeline(&state, trip_id, day_number)
}

#[tauri::command]
pub fn create_trip(
    input: CreateTripParams,
    state: State<DesktopState>,
) -> Result<CreateTripResult, DesktopError> {
    service::create_trip(&state, input)
}

#[tauri::command]
pub fn create_itinerary(
    input: CreateItineraryParams,
    state: State<DesktopState>,
) -> Result<CreateItineraryResult, DesktopError> {
    service::create_itinerary(&state, input)
}

#[tauri::command]
pub fn update_itinerary(
    input: UpdateItineraryParams,
    state: State<DesktopState>,
) -> Result<UpdateItineraryResult, DesktopError> {
    service::update_itinerary(&state, input)
}
