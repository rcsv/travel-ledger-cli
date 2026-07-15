//! Atomic Trip creation use case shared by CLI and Desktop.

use std::fmt;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTripParams {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub summary: Option<String>,
    pub main_destination: Option<String>,
    pub main_destination_country_code: Option<String>,
    pub default_currency: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTripResult {
    pub trip_id: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TripCreateErrorCode {
    ValidationFailed,
    StorageFailure,
}

impl TripCreateErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidationFailed => "TRIP_VALIDATION_FAILED",
            Self::StorageFailure => "STORAGE_FAILURE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TripCreateError {
    pub code: TripCreateErrorCode,
    pub message: String,
}

impl TripCreateError {
    fn new(code: TripCreateErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self::new(TripCreateErrorCode::ValidationFailed, message)
    }

    fn storage(message: impl Into<String>) -> Self {
        Self::new(TripCreateErrorCode::StorageFailure, message)
    }
}

impl fmt::Display for TripCreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TripCreateError {}

pub fn create_trip(
    conn: &mut Connection,
    params: CreateTripParams,
) -> Result<CreateTripResult, TripCreateError> {
    let trip = crate::trip::validate_trip_create(
        &params.name,
        &params.start_date,
        &params.end_date,
        params.summary.as_deref(),
        crate::trip::TripMetadataWrite {
            main_destination: params.main_destination.as_deref(),
            main_destination_country_code: params.main_destination_country_code.as_deref(),
            default_currency: params.default_currency.as_deref(),
        },
    )
    .map_err(|err| TripCreateError::validation(err.to_string()))?;

    let tx = conn
        .transaction()
        .map_err(|err| TripCreateError::storage(err.to_string()))?;
    let trip_id = crate::trip::insert_validated_trip(&tx, &trip)
        .map_err(|err| TripCreateError::storage(err.to_string()))?;
    tx.commit()
        .map_err(|err| TripCreateError::storage(err.to_string()))?;
    Ok(CreateTripResult { trip_id })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::storage::db::init_db(&conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn
    }

    fn params(name: &str, start_date: &str, end_date: &str) -> CreateTripParams {
        CreateTripParams {
            name: name.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            summary: None,
            main_destination: None,
            main_destination_country_code: None,
            default_currency: None,
        }
    }

    #[test]
    fn creates_trimmed_trip_and_days_atomically() {
        let mut conn = connection();
        let result = create_trip(
            &mut conn,
            CreateTripParams {
                name: "  Okinawa  ".to_string(),
                summary: Some("  Sea and sun  ".to_string()),
                main_destination: Some("  Naha  ".to_string()),
                main_destination_country_code: Some("jp".to_string()),
                default_currency: Some("jpy".to_string()),
                ..params("ignored", "2026-04-26", "2026-04-29")
            },
        )
        .unwrap();

        let detail = crate::services::get_trip_detail(&conn, result.trip_id).unwrap();
        assert_eq!(detail.name, "Okinawa");
        assert_eq!(detail.summary.as_deref(), Some("Sea and sun"));
        assert_eq!(detail.main_destination.as_deref(), Some("Naha"));
        assert_eq!(detail.main_destination_country_code.as_deref(), Some("JP"));
        assert_eq!(detail.default_currency.as_deref(), Some("JPY"));
        assert_eq!(detail.days.len(), 4);
    }

    #[test]
    fn same_day_trip_has_one_day_and_empty_optionals_are_null() {
        let mut conn = connection();
        let mut input = params("Same Day", "2026-06-01", "2026-06-01");
        input.summary = Some("   ".to_string());
        input.main_destination = Some(String::new());
        let result = create_trip(&mut conn, input).unwrap();
        let detail = crate::services::get_trip_detail(&conn, result.trip_id).unwrap();
        assert_eq!(detail.days.len(), 1);
        assert_eq!(detail.summary, None);
        assert_eq!(detail.main_destination, None);
    }

    #[test]
    fn accepts_metadata_at_maximum_lengths() {
        let mut conn = connection();
        let summary = "s".repeat(crate::summary::TRIP_SUMMARY_MAX_LEN);
        let destination = "d".repeat(crate::trip_metadata::TRIP_MAIN_DESTINATION_MAX_LEN);
        let result = create_trip(
            &mut conn,
            CreateTripParams {
                summary: Some(summary.clone()),
                main_destination: Some(destination.clone()),
                ..params("Limits", "2026-06-01", "2026-06-02")
            },
        )
        .unwrap();
        let detail = crate::services::get_trip_detail(&conn, result.trip_id).unwrap();
        assert_eq!(detail.summary.as_deref(), Some(summary.as_str()));
        assert_eq!(
            detail.main_destination.as_deref(),
            Some(destination.as_str())
        );
    }

    #[test]
    fn rejects_invalid_inputs_as_validation_failures() {
        let cases = [
            params("   ", "2026-06-01", "2026-06-02"),
            params("Bad Date", "06/01/2026", "2026-06-02"),
            params("Bad Range", "2026-06-03", "2026-06-02"),
            CreateTripParams {
                main_destination_country_code: Some("XX".to_string()),
                ..params("Bad Country", "2026-06-01", "2026-06-02")
            },
            CreateTripParams {
                default_currency: Some("ZZZ".to_string()),
                ..params("Bad Currency", "2026-06-01", "2026-06-02")
            },
            CreateTripParams {
                summary: Some("x".repeat(crate::summary::TRIP_SUMMARY_MAX_LEN + 1)),
                ..params("Long Summary", "2026-06-01", "2026-06-02")
            },
            CreateTripParams {
                main_destination: Some(
                    "x".repeat(crate::trip_metadata::TRIP_MAIN_DESTINATION_MAX_LEN + 1),
                ),
                ..params("Long Destination", "2026-06-01", "2026-06-02")
            },
        ];

        for input in cases {
            let mut conn = connection();
            let err = create_trip(&mut conn, input).unwrap_err();
            assert_eq!(err.code, TripCreateErrorCode::ValidationFailed);
            assert!(crate::services::list_trip_summaries(&conn)
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn rolls_back_trip_when_day_creation_fails() {
        let mut conn = connection();
        conn.execute_batch(
            "CREATE TRIGGER fail_day_insert
             BEFORE INSERT ON days
             BEGIN
               SELECT RAISE(ABORT, 'forced day failure');
             END;",
        )
        .unwrap();

        let err =
            create_trip(&mut conn, params("Atomic Trip", "2026-06-01", "2026-06-02")).unwrap_err();
        assert_eq!(err.code, TripCreateErrorCode::StorageFailure);
        let trip_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM trips", [], |row| row.get(0))
            .unwrap();
        let day_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM days", [], |row| row.get(0))
            .unwrap();
        assert_eq!(trip_count, 0);
        assert_eq!(day_count, 0);
    }
}
