use std::path::{Path, PathBuf};

use serde::Serialize;
use travel_ledger_cli::{
    create_itinerary as core_create_itinerary, create_trip as core_create_trip,
    get_day_timeline as facade_get_day_timeline, get_trip_detail as facade_get_trip_detail,
    list_trip_summaries as facade_list_trip_summaries, open_db,
    update_itinerary as core_update_itinerary, CreateItineraryParams, CreateItineraryResult,
    CreateTripParams, CreateTripResult, DayDetail, TripDetail, TripSummary, UpdateItineraryParams,
    UpdateItineraryResult,
};

use crate::config::{self, LoadSavedPath};
use crate::error::DesktopError;
use crate::state::DesktopState;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub path: String,
    pub trip_count: usize,
}

/// Tagged restore outcome — frontend discriminates on `status` without string parsing.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RestoreLastDatabaseResult {
    Restored { database: DatabaseInfo },
    NotFound,
    InvalidCleared { code: String, message: String },
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
        path: path_to_info_string(path),
        trip_count: summaries.len(),
    })
}

pub fn select_database(state: &DesktopState, path: PathBuf) -> Result<DatabaseInfo, DesktopError> {
    let info = probe_database(&path)?;
    config::write_last_database_path(&state.settings_path, &path)?;
    {
        let mut guard = state
            .selected_db_path
            .lock()
            .map_err(|_| DesktopError::database_open_failed("Application state lock poisoned"))?;
        *guard = Some(path);
    }
    Ok(info)
}

pub fn restore_last_database(
    state: &DesktopState,
) -> Result<RestoreLastDatabaseResult, DesktopError> {
    let loaded = config::load_saved_database_path(&state.settings_path)?;
    match loaded {
        LoadSavedPath::Absent => Ok(RestoreLastDatabaseResult::NotFound),
        LoadSavedPath::Corrupt => {
            let _ = config::clear_settings(&state.settings_path);
            Ok(RestoreLastDatabaseResult::InvalidCleared {
                code: "DATABASE_CONFIG_READ_FAILED".to_string(),
                message: "Saved desktop settings were invalid and have been cleared.".to_string(),
            })
        }
        LoadSavedPath::Present(path) => match probe_database(&path) {
            Ok(info) => {
                let mut guard = state.selected_db_path.lock().map_err(|_| {
                    DesktopError::database_open_failed("Application state lock poisoned")
                })?;
                *guard = Some(path);
                Ok(RestoreLastDatabaseResult::Restored { database: info })
            }
            Err(err) => {
                clear_selection(state)?;
                config::clear_settings(&state.settings_path)?;
                Ok(RestoreLastDatabaseResult::InvalidCleared {
                    code: err.code,
                    message: err.message,
                })
            }
        },
    }
}

pub fn forget_database(state: &DesktopState) -> Result<(), DesktopError> {
    clear_selection(state)?;
    config::clear_settings(&state.settings_path)?;
    Ok(())
}

pub fn selected_db_path(state: &DesktopState) -> Result<PathBuf, DesktopError> {
    let guard = state
        .selected_db_path
        .lock()
        .map_err(|_| DesktopError::database_open_failed("Application state lock poisoned"))?;
    guard
        .clone()
        .ok_or_else(DesktopError::database_not_selected)
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

pub fn create_trip(
    state: &DesktopState,
    params: CreateTripParams,
) -> Result<CreateTripResult, DesktopError> {
    let path = selected_db_path(state)?;
    let mut conn = open_connection(&path)?;
    core_create_trip(&mut conn, params).map_err(DesktopError::from)
}

pub fn create_itinerary(
    state: &DesktopState,
    params: CreateItineraryParams,
) -> Result<CreateItineraryResult, DesktopError> {
    let path = selected_db_path(state)?;
    let conn = open_connection(&path)?;
    core_create_itinerary(&conn, params).map_err(DesktopError::from)
}

pub fn update_itinerary(
    state: &DesktopState,
    params: UpdateItineraryParams,
) -> Result<UpdateItineraryResult, DesktopError> {
    let path = selected_db_path(state)?;
    let conn = open_connection(&path)?;
    core_update_itinerary(&conn, params).map_err(DesktopError::from)
}

fn clear_selection(state: &DesktopState) -> Result<(), DesktopError> {
    let mut guard = state
        .selected_db_path
        .lock()
        .map_err(|_| DesktopError::database_open_failed("Application state lock poisoned"))?;
    *guard = None;
    Ok(())
}

fn path_to_info_string(path: &Path) -> String {
    path.to_str()
        .map(str::to_string)
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn open_connection(path: &Path) -> Result<rusqlite::Connection, DesktopError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| DesktopError::database_path_invalid("Database path is not valid UTF-8"))?;
    open_db(path_str).map_err(|err| DesktopError::database_open_failed(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{self, settings_file_path};
    use crate::state::DesktopState;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use travel_ledger_cli::ReadServiceErrorCode;

    fn temp_state() -> (tempfile::TempDir, DesktopState) {
        let dir = tempfile::tempdir().expect("tempdir");
        let settings = settings_file_path(dir.path());
        (dir, DesktopState::new(settings))
    }

    fn temp_db(dir: &Path) -> PathBuf {
        dir.join("test.db")
    }

    fn cli_binary() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/travel-ledger-cli")
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
                db_path.to_str().expect("utf8 path"),
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
        assert!(
            status.success(),
            "trip add failed for {}",
            db_path.display()
        );
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
    fn select_persists_path() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        seed_trip_with_cli(&path, "Desktop Trip", "2026-06-01", "2026-06-02");

        let info = select_database(&state, path.clone()).unwrap();
        assert_eq!(info.trip_count, 1);
        assert_eq!(selected_db_path(&state).unwrap(), path);
        match config::load_saved_database_path(&state.settings_path).unwrap() {
            LoadSavedPath::Present(saved) => assert_eq!(saved, path),
            other => panic!("expected Present, got {other:?}"),
        }
    }

    #[test]
    fn invalid_select_leaves_state_and_settings() {
        let (_dir, state) = temp_state();
        let err = select_database(&state, PathBuf::from("/missing.db")).unwrap_err();
        assert_eq!(err.code, "DATABASE_PATH_INVALID");
        assert!(selected_db_path(&state).is_err());
        assert!(!state.settings_path.exists());
    }

    #[test]
    fn invalid_select_does_not_overwrite_existing_settings() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        seed_trip_with_cli(&path, "Keep Me", "2026-06-01", "2026-06-02");
        select_database(&state, path.clone()).unwrap();

        let err = select_database(&state, PathBuf::from("/still-missing.db")).unwrap_err();
        assert_eq!(err.code, "DATABASE_PATH_INVALID");
        assert_eq!(selected_db_path(&state).unwrap(), path);
        match config::load_saved_database_path(&state.settings_path).unwrap() {
            LoadSavedPath::Present(saved) => assert_eq!(saved, path),
            other => panic!("expected Present, got {other:?}"),
        }
    }

    #[test]
    fn restore_without_settings_is_not_found() {
        let (_dir, state) = temp_state();
        let result = restore_last_database(&state).unwrap();
        assert_eq!(result, RestoreLastDatabaseResult::NotFound);
        assert!(selected_db_path(&state).is_err());
    }

    #[test]
    fn restore_valid_saved_database() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        seed_trip_with_cli(&path, "Restore Trip", "2026-04-26", "2026-04-29");
        select_database(&state, path.clone()).unwrap();

        let restored_state = DesktopState::new(state.settings_path.clone());
        match restore_last_database(&restored_state).unwrap() {
            RestoreLastDatabaseResult::Restored { database } => {
                assert_eq!(database.trip_count, 1);
                assert_eq!(selected_db_path(&restored_state).unwrap(), path);
            }
            other => panic!("expected Restored, got {other:?}"),
        }

        let summaries = list_trip_summaries(&restored_state).unwrap();
        let detail = get_trip_detail(&restored_state, summaries[0].id).unwrap();
        let day = get_day_timeline(&restored_state, summaries[0].id, 1).unwrap();
        assert_eq!(detail.days.len(), 4);
        assert_eq!(day.day_number, 1);
    }

    #[test]
    fn restore_missing_path_clears_without_creating_db() {
        let (dir, state) = temp_state();
        let missing = dir.path().join("gone.db");
        config::write_last_database_path(&state.settings_path, &missing).unwrap();
        assert!(!missing.exists());

        match restore_last_database(&state).unwrap() {
            RestoreLastDatabaseResult::InvalidCleared { code, .. } => {
                assert_eq!(code, "DATABASE_PATH_INVALID");
            }
            other => panic!("expected InvalidCleared, got {other:?}"),
        }
        assert!(!missing.exists());
        assert!(!state.settings_path.exists());
        assert!(selected_db_path(&state).is_err());
    }

    #[test]
    fn restore_directory_path_rejects() {
        let (dir, state) = temp_state();
        config::write_last_database_path(&state.settings_path, dir.path()).unwrap();
        match restore_last_database(&state).unwrap() {
            RestoreLastDatabaseResult::InvalidCleared { code, .. } => {
                assert_eq!(code, "DATABASE_PATH_INVALID");
            }
            other => panic!("expected InvalidCleared, got {other:?}"),
        }
        assert!(selected_db_path(&state).is_err());
    }

    #[test]
    fn restore_corrupt_sqlite_file_rejects() {
        let (dir, state) = temp_state();
        let junk = dir.path().join("junk.db");
        fs::write(&junk, b"not a sqlite database").unwrap();
        config::write_last_database_path(&state.settings_path, &junk).unwrap();

        match restore_last_database(&state).unwrap() {
            RestoreLastDatabaseResult::InvalidCleared { code, .. } => {
                assert_eq!(code, "DATABASE_OPEN_FAILED");
            }
            other => panic!("expected InvalidCleared, got {other:?}"),
        }
        assert!(junk.exists());
        assert!(!state.settings_path.exists());
    }

    #[test]
    fn restore_corrupt_settings_clears_and_stays_unselected() {
        let (_dir, state) = temp_state();
        fs::write(&state.settings_path, "{broken").unwrap();
        match restore_last_database(&state).unwrap() {
            RestoreLastDatabaseResult::InvalidCleared { code, .. } => {
                assert_eq!(code, "DATABASE_CONFIG_READ_FAILED");
            }
            other => panic!("expected InvalidCleared, got {other:?}"),
        }
        assert!(selected_db_path(&state).is_err());
        assert!(!state.settings_path.exists());
    }

    #[test]
    fn forget_clears_state_and_settings_but_keeps_db_file() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        seed_trip_with_cli(&path, "Forget Me Not", "2026-06-01", "2026-06-02");
        select_database(&state, path.clone()).unwrap();
        assert!(state.settings_path.exists());

        forget_database(&state).unwrap();
        assert!(selected_db_path(&state).is_err());
        assert!(!state.settings_path.exists());
        assert!(path.exists());
    }

    #[test]
    fn database_not_selected_before_open() {
        let (_dir, state) = temp_state();
        let err = list_trip_summaries(&state).unwrap_err();
        assert_eq!(err.code, "DATABASE_NOT_SELECTED");
    }

    #[test]
    fn list_and_detail_use_facade() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        seed_trip_with_cli(&path, "Okinawa", "2026-04-26", "2026-04-29");

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
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();
        let err = get_trip_detail(&state, 9999).unwrap_err();
        assert_eq!(err.code, "TRIP_NOT_FOUND");
    }

    #[test]
    fn maps_day_not_found() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        seed_trip_with_cli(&path, "Trip", "2026-06-01", "2026-06-03");
        select_database(&state, path).unwrap();
        let summaries = list_trip_summaries(&state).unwrap();
        let trip_id = summaries[0].id;
        let err = get_day_timeline(&state, trip_id, 99).unwrap_err();
        assert_eq!(err.code, "DAY_NOT_FOUND");
    }

    #[test]
    fn service_error_mapping_uses_core_codes() {
        let err: DesktopError =
            travel_ledger_cli::ServiceError::new(ReadServiceErrorCode::StorageFailure, "db locked")
                .into();
        assert_eq!(err.code, "STORAGE_FAILURE");
    }

    fn create_params(name: &str) -> CreateTripParams {
        CreateTripParams {
            name: name.to_string(),
            start_date: "2026-08-01".to_string(),
            end_date: "2026-08-03".to_string(),
            summary: None,
            main_destination: None,
            main_destination_country_code: None,
            default_currency: None,
        }
    }

    fn itinerary_params(trip_id: i64, title: &str) -> CreateItineraryParams {
        CreateItineraryParams {
            trip_id,
            day_number: 1,
            title: title.to_string(),
            start_time: None,
            location: None,
            note: None,
        }
    }

    fn itinerary_update_params(
        trip_id: i64,
        itinerary_id: i64,
        title: &str,
    ) -> UpdateItineraryParams {
        UpdateItineraryParams {
            trip_id,
            day_number: 1,
            itinerary_id,
            title: title.to_string(),
            start_time: None,
            location: None,
            note: None,
        }
    }

    #[test]
    fn create_requires_selected_database() {
        let (_dir, state) = temp_state();
        let err = create_trip(&state, create_params("Desktop Trip")).unwrap_err();
        assert_eq!(err.code, "DATABASE_NOT_SELECTED");
    }

    #[test]
    fn creates_normalized_trip_readable_through_facade() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();

        let result = create_trip(
            &state,
            CreateTripParams {
                name: "  Desktop Trip  ".to_string(),
                summary: Some("  Created in Tauri  ".to_string()),
                main_destination: Some("  Kyoto  ".to_string()),
                main_destination_country_code: Some("jp".to_string()),
                default_currency: Some("jpy".to_string()),
                ..create_params("ignored")
            },
        )
        .unwrap();

        let summaries = list_trip_summaries(&state).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, result.trip_id);
        assert_eq!(summaries[0].name, "Desktop Trip");
        let detail = get_trip_detail(&state, result.trip_id).unwrap();
        assert_eq!(detail.summary.as_deref(), Some("Created in Tauri"));
        assert_eq!(detail.main_destination.as_deref(), Some("Kyoto"));
        assert_eq!(detail.main_destination_country_code.as_deref(), Some("JP"));
        assert_eq!(detail.default_currency.as_deref(), Some("JPY"));
        assert_eq!(detail.days.len(), 3);
        assert_eq!(get_day_timeline(&state, result.trip_id, 1).unwrap().day_number, 1);
    }

    #[test]
    fn created_trip_survives_database_restore() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path.clone()).unwrap();
        let result = create_trip(&state, create_params("Persistent Trip")).unwrap();

        let restored_state = DesktopState::new(state.settings_path.clone());
        match restore_last_database(&restored_state).unwrap() {
            RestoreLastDatabaseResult::Restored { database } => {
                assert_eq!(database.path, path.to_string_lossy());
                assert_eq!(database.trip_count, 1);
            }
            other => panic!("expected Restored, got {other:?}"),
        }
        let detail = get_trip_detail(&restored_state, result.trip_id).unwrap();
        assert_eq!(detail.name, "Persistent Trip");
        assert_eq!(detail.days.len(), 3);
    }

    #[test]
    fn maps_validation_failure_and_preserves_database() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();

        let err = create_trip(&state, create_params("   ")).unwrap_err();
        assert_eq!(err.code, "TRIP_VALIDATION_FAILED");
        assert!(list_trip_summaries(&state).unwrap().is_empty());
    }

    #[test]
    fn maps_storage_failure_and_rolls_back_trip() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        let conn = open_db(path.to_str().unwrap()).unwrap();
        conn.execute_batch(
            "CREATE TRIGGER fail_desktop_day_insert
             BEFORE INSERT ON days
             BEGIN
               SELECT RAISE(ABORT, 'forced desktop day failure');
             END;",
        )
        .unwrap();
        drop(conn);
        select_database(&state, path).unwrap();

        let err = create_trip(&state, create_params("Atomic Desktop Trip")).unwrap_err();
        assert_eq!(err.code, "STORAGE_FAILURE");
        assert!(list_trip_summaries(&state).unwrap().is_empty());
    }

    #[test]
    fn itinerary_create_requires_selected_database() {
        let (_dir, state) = temp_state();
        let err = create_itinerary(&state, itinerary_params(1, "Activity")).unwrap_err();
        assert_eq!(err.code, "DATABASE_NOT_SELECTED");
    }

    #[test]
    fn creates_normalized_itinerary_readable_in_append_order() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();
        let trip = create_trip(&state, create_params("Itinerary Trip")).unwrap();

        let first = create_itinerary(
            &state,
            CreateItineraryParams {
                title: "  Museum  ".to_string(),
                start_time: Some("09:30".to_string()),
                location: Some("  Naha  ".to_string()),
                note: Some("  Buy tickets  ".to_string()),
                ..itinerary_params(trip.trip_id, "ignored")
            },
        )
        .unwrap();
        let second =
            create_itinerary(&state, itinerary_params(trip.trip_id, "Lunch")).unwrap();

        let timeline = get_day_timeline(&state, trip.trip_id, 1).unwrap();
        assert_eq!(timeline.itineraries.len(), 2);
        assert_eq!(timeline.itineraries[0].id, first.itinerary_id);
        assert_eq!(timeline.itineraries[0].title, "Museum");
        assert_eq!(timeline.itineraries[0].sort_order, 1000);
        assert_eq!(timeline.itineraries[0].location.as_deref(), Some("Naha"));
        assert_eq!(timeline.itineraries[1].id, second.itinerary_id);
        assert_eq!(timeline.itineraries[1].sort_order, 2000);
    }

    #[test]
    fn maps_itinerary_validation_and_target_failures() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();
        let trip = create_trip(&state, create_params("Itinerary Trip")).unwrap();

        let validation =
            create_itinerary(&state, itinerary_params(trip.trip_id, "   ")).unwrap_err();
        assert_eq!(validation.code, "ITINERARY_VALIDATION_FAILED");
        let target = create_itinerary(
            &state,
            CreateItineraryParams {
                day_number: 99,
                ..itinerary_params(trip.trip_id, "Missing day")
            },
        )
        .unwrap_err();
        assert_eq!(target.code, "ITINERARY_TARGET_NOT_FOUND");
        assert!(get_day_timeline(&state, trip.trip_id, 1)
            .unwrap()
            .itineraries
            .is_empty());
    }

    #[test]
    fn maps_itinerary_storage_failure_without_inserting_item() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        let mut conn = open_db(path.to_str().unwrap()).unwrap();
        let trip = core_create_trip(&mut conn, create_params("Itinerary Trip")).unwrap();
        conn.execute_batch(
            "CREATE TRIGGER fail_desktop_itinerary_insert
             BEFORE INSERT ON itinerary_items
             BEGIN
               SELECT RAISE(ABORT, 'forced desktop itinerary failure');
             END;",
        )
        .unwrap();
        drop(conn);
        select_database(&state, path).unwrap();

        let err = create_itinerary(&state, itinerary_params(trip.trip_id, "Blocked")).unwrap_err();
        assert_eq!(err.code, "STORAGE_FAILURE");
        assert!(get_day_timeline(&state, trip.trip_id, 1)
            .unwrap()
            .itineraries
            .is_empty());
    }

    #[test]
    fn itinerary_update_requires_selected_database() {
        let (_dir, state) = temp_state();
        let err = update_itinerary(&state, itinerary_update_params(1, 1, "Activity"))
            .unwrap_err();
        assert_eq!(err.code, "DATABASE_NOT_SELECTED");
    }

    #[test]
    fn updates_normalized_itinerary_and_preserves_other_fields() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();
        let trip = create_trip(&state, create_params("Inspector Trip")).unwrap();
        let created = create_itinerary(
            &state,
            CreateItineraryParams {
                start_time: Some("08:00".to_string()),
                location: Some("Old place".to_string()),
                note: Some("Old note".to_string()),
                ..itinerary_params(trip.trip_id, "Original")
            },
        )
        .unwrap();

        let result = update_itinerary(
            &state,
            UpdateItineraryParams {
                start_time: Some("09:30".to_string()),
                location: Some("  New place  ".to_string()),
                note: Some("  New note  ".to_string()),
                ..itinerary_update_params(trip.trip_id, created.itinerary_id, "  Updated  ")
            },
        )
        .unwrap();

        assert_eq!(result.itinerary_id, created.itinerary_id);
        let timeline = get_day_timeline(&state, trip.trip_id, 1).unwrap();
        let item = &timeline.itineraries[0];
        assert_eq!(item.title, "Updated");
        assert_eq!(item.start_time.as_deref(), Some("09:30"));
        assert_eq!(item.location.as_deref(), Some("New place"));
        assert_eq!(item.note.as_deref(), Some("New note"));
        assert_eq!(item.sort_order, 1000);
    }

    #[test]
    fn maps_itinerary_update_validation_and_target_failures() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path).unwrap();
        let trip = create_trip(&state, create_params("Inspector Trip")).unwrap();
        let created =
            create_itinerary(&state, itinerary_params(trip.trip_id, "Original")).unwrap();

        let validation = update_itinerary(
            &state,
            itinerary_update_params(trip.trip_id, created.itinerary_id, "   "),
        )
        .unwrap_err();
        assert_eq!(validation.code, "ITINERARY_VALIDATION_FAILED");
        let target = update_itinerary(
            &state,
            itinerary_update_params(trip.trip_id + 1, created.itinerary_id, "Changed"),
        )
        .unwrap_err();
        assert_eq!(target.code, "ITINERARY_TARGET_NOT_FOUND");

        let timeline = get_day_timeline(&state, trip.trip_id, 1).unwrap();
        assert_eq!(timeline.itineraries[0].title, "Original");
    }

    #[test]
    fn maps_itinerary_update_storage_failure_without_changing_item() {
        let (dir, state) = temp_state();
        let path = temp_db(dir.path());
        open_db(path.to_str().unwrap()).unwrap();
        select_database(&state, path.clone()).unwrap();
        let trip = create_trip(&state, create_params("Inspector Trip")).unwrap();
        let created =
            create_itinerary(&state, itinerary_params(trip.trip_id, "Original")).unwrap();
        let conn = open_db(path.to_str().unwrap()).unwrap();
        conn.execute_batch(
            "CREATE TRIGGER fail_desktop_itinerary_update
             BEFORE UPDATE ON itinerary_items
             BEGIN
               SELECT RAISE(ABORT, 'forced desktop itinerary update failure');
             END;",
        )
        .unwrap();
        drop(conn);

        let err = update_itinerary(
            &state,
            itinerary_update_params(trip.trip_id, created.itinerary_id, "Blocked"),
        )
        .unwrap_err();
        assert_eq!(err.code, "STORAGE_FAILURE");
        let timeline = get_day_timeline(&state, trip.trip_id, 1).unwrap();
        assert_eq!(timeline.itineraries[0].title, "Original");
    }

    #[test]
    #[ignore = "manual smoke: run with SMOKE_DB=/path/to/okinawa.db"]
    fn smoke_populated_sample_database() {
        let path =
            std::env::var("SMOKE_DB").expect("set SMOKE_DB to a populated Travel Ledger database");
        let settings_dir = tempfile::tempdir().unwrap();
        let state = DesktopState::new(settings_file_path(settings_dir.path()));
        let info = select_database(&state, PathBuf::from(&path)).unwrap();
        assert!(info.trip_count >= 1);

        let restored = DesktopState::new(state.settings_path.clone());
        match restore_last_database(&restored).unwrap() {
            RestoreLastDatabaseResult::Restored { database } => {
                assert_eq!(database.trip_count, info.trip_count);
            }
            other => panic!("expected Restored, got {other:?}"),
        }

        let summaries = list_trip_summaries(&restored).unwrap();
        let first_trip = summaries[0].id;
        let detail = get_trip_detail(&restored, first_trip).unwrap();
        let day_timeline =
            get_day_timeline(&restored, first_trip, detail.days[0].day_number).unwrap();
        assert!(!day_timeline.itineraries.is_empty());
    }
}
