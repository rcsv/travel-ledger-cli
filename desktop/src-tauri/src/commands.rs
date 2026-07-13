use std::path::PathBuf;

use tauri::State;

use crate::error::DesktopError;
use crate::service::{self, DatabaseInfo};
use crate::state::DesktopState;
use travel_ledger_cli::{DayDetail, TripDetail, TripSummary};

#[tauri::command]
pub fn select_database(path: String, state: State<DesktopState>) -> Result<DatabaseInfo, DesktopError> {
    service::select_database(&state, PathBuf::from(path))
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
