use std::path::{Path, PathBuf};

use serde::Serialize;
use travel_ledger_cli::{
    get_day_timeline as facade_get_day_timeline, get_trip_detail as facade_get_trip_detail,
    list_trip_summaries as facade_list_trip_summaries, open_db, DayDetail, TripDetail,
    TripSummary,
};

use crate::error::DesktopError;
use crate::state::DesktopState;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub path: String,
    pub trip_count: usize,
}

pub fn validate_db_path(path: &Path) -> Result<(), DesktopError> {
    if !path.exists() {
        return Err(DesktopError::database_path_invalid(
            "Database file does not exist",
        ));
    }
    if !path.is_file() {
        return Err(DesktopError::database_path_invalid(
            "Selected path is not a regular file",
        ));
    }
    Ok(())
}

pub fn probe_database(path: &Path) -> Result<DatabaseInfo, DesktopError> {
    validate_db_path(path)?;
    let conn = open_connection(path)?;
    let summaries = facade_list_trip_summaries(&conn).map_err(DesktopError::from)?;
    Ok(DatabaseInfo {
        path: path.display().to_string(),
        trip_count: summaries.len(),
    })
}

pub fn select_database(state: &DesktopState, path: PathBuf) -> Result<DatabaseInfo, DesktopError> {
    let info = probe_database(&path)?;
    let mut guard = state.selected_db_path.lock().map_err(|_| {
        DesktopError::database_open_failed("Application state lock poisoned")
    })?;
    *guard = Some(path);
    Ok(info)
}

pub fn selected_db_path(state: &DesktopState) -> Result<PathBuf, DesktopError> {
    let guard = state
        .selected_db_path
        .lock()
        .map_err(|_| DesktopError::database_open_failed("Application state lock poisoned"))?;
    guard.clone().ok_or_else(DesktopError::database_not_selected)
}

pub fn list_trip_summaries(state: &DesktopState) -> Result<Vec<TripSummary>, DesktopError> {
    let path = selected_db_path(state)?;
    let conn = open_connection(&path)?;
    facade_list_trip_summaries(&conn).map_err(DesktopError::from)
}

pub fn get_trip_detail(state: &DesktopState, trip_id: i64) -> Result<TripDetail, DesktopError> {
    let path = selected_db_path(state)?;
    let conn = open_connection(&path)?;
    facade_get_trip_detail(&conn, trip_id).map_err(DesktopError::from)
}

pub fn get_day_timeline(
    state: &DesktopState,
    trip_id: i64,
    day_number: i64,
) -> Result<DayDetail, DesktopError> {
    let path = selected_db_path(state)?;
    let conn = open_connection(&path)?;
    facade_get_day_timeline(&conn, trip_id, day_number).map_err(DesktopError::from)
}

fn open_connection(path: &Path) -> Result<rusqlite::Connection, DesktopError> {
    open_db(&path.to_string_lossy())
        .map_err(|err| DesktopError::database_open_failed(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DesktopState;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use travel_ledger_cli::ReadServiceErrorCode;

    fn temp_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        (dir, path)
    }

    fn cli_binary() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug/travel-ledger-cli")
    }

    fn seed_trip_with_cli(db_path: &Path, name: &str, start: &str, end: &str) {
        let cli = cli_binary();
        assert!(
            cli.exists(),
            "travel-ledger-cli binary missing at {}; run `cargo build` at repo root first",
            cli.display()
        );
        let status = Command::new(cli)
            .args([
                "--db",
                &db_path.to_string_lossy(),
                "trip",
                "add",
                name,
                "--start",
                start,
                "--end",
                end,
            ])
            .status()
            .expect("spawn travel-ledger-cli");
        assert!(status.success(), "trip add failed for {}", db_path.display());
    }

    #[test]
    fn rejects_missing_path() {
        let err = validate_db_path(Path::new("/no/such/travel-ledger.db")).unwrap_err();
        assert_eq!(err.code, "DATABASE_PATH_INVALID");
    }

    #[test]
    fn rejects_directory_path() {
        let dir = tempfile::tempdir().unwrap();
        let err = validate_db_path(dir.path()).unwrap_err();
        assert_eq!(err.code, "DATABASE_PATH_INVALID");
    }

    #[test]
    fn probe_and_select_valid_db() {
        let (_dir, path) = temp_db();
        seed_trip_with_cli(&path, "Desktop Trip", "2026-06-01", "2026-06-02");

        let state = DesktopState::default();
        let info = select_database(&state, path.clone()).unwrap();
        assert_eq!(info.trip_count, 1);
        assert_eq!(selected_db_path(&state).unwrap(), path);
    }

    #[test]
    fn database_not_selected_before_open() {
        let state = DesktopState::default();
        let err = list_trip_summaries(&state).unwrap_err();
        assert_eq!(err.code, "DATABASE_NOT_SELECTED");
    }

    #[test]
    fn list_and_detail_use_facade() {
        let (_dir, path) = temp_db();
        seed_trip_with_cli(&path, "Okinawa", "2026-04-26", "2026-04-29");

        let state = DesktopState::default();
        select_database(&state, path).unwrap();
        let summaries = list_trip_summaries(&state).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].name, "Okinawa");

        let detail = get_trip_detail(&state, summaries[0].id).unwrap();
        assert_eq!(detail.days.len(), 4);

        let day = get_day_timeline(&state, summaries[0].id, 1).unwrap();
        assert_eq!(day.day_number, 1);
        assert!(day.itineraries.is_empty());
    }

    #[test]
    fn maps_trip_not_found() {
        let (_dir, path) = temp_db();
        open_db(path.to_str().unwrap()).unwrap();
        let state = DesktopState::default();
        select_database(&state, path).unwrap();
        let err = get_trip_detail(&state, 9999).unwrap_err();
        assert_eq!(err.code, "TRIP_NOT_FOUND");
    }

    #[test]
    fn maps_day_not_found() {
        let (_dir, path) = temp_db();
        seed_trip_with_cli(&path, "Trip", "2026-06-01", "2026-06-03");
        let state = DesktopState::default();
        select_database(&state, path.clone()).unwrap();
        let summaries = list_trip_summaries(&state).unwrap();
        let trip_id = summaries[0].id;
        let err = get_day_timeline(&state, trip_id, 99).unwrap_err();
        assert_eq!(err.code, "DAY_NOT_FOUND");
    }

    #[test]
    fn state_only_set_after_successful_validation() {
        let state = DesktopState::default();
        let err = select_database(&state, PathBuf::from("/missing.db")).unwrap_err();
        assert_eq!(err.code, "DATABASE_PATH_INVALID");
        assert!(selected_db_path(&state).is_err());
    }

    #[test]
    fn service_error_mapping_uses_core_codes() {
        let err: DesktopError = travel_ledger_cli::ServiceError::new(
            ReadServiceErrorCode::StorageFailure,
            "db locked",
        )
        .into();
        assert_eq!(err.code, "STORAGE_FAILURE");
    }

    #[test]
    #[ignore = "manual smoke: run with SMOKE_DB=/path/to/okinawa.db"]
    fn smoke_populated_sample_database() {
        let path = std::env::var("SMOKE_DB").expect("set SMOKE_DB to a populated Travel Ledger database");
        let state = DesktopState::default();
        let info = select_database(&state, PathBuf::from(&path)).unwrap();
        assert!(info.trip_count >= 1, "expected at least one trip");

        let summaries = list_trip_summaries(&state).unwrap();
        assert!(!summaries.is_empty());
        let first_trip = summaries[0].id;
        let second_trip = summaries.get(1).map(|trip| trip.id);

        let detail = get_trip_detail(&state, first_trip).unwrap();
        assert!(!detail.days.is_empty());
        let first_day = detail.days[0].day_number;
        let day_timeline = get_day_timeline(&state, first_trip, first_day).unwrap();
        assert!(
            !day_timeline.itineraries.is_empty(),
            "expected populated itinerary timeline"
        );

        if let Some(second_id) = second_trip {
            let other = get_trip_detail(&state, second_id).unwrap();
            assert_ne!(other.id, detail.id);
        }
    }
}
