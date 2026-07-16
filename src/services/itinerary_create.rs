//! Append-only Itinerary creation use case shared by CLI and Desktop.

use std::fmt;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::read_errors::{classify_read_error, ReadServiceErrorCode};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateItineraryParams {
    pub trip_id: i64,
    pub day_number: i64,
    pub title: String,
    pub start_time: Option<String>,
    pub location: Option<String>,
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateItineraryResult {
    pub itinerary_id: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItineraryCreateErrorCode {
    ValidationFailed,
    TargetNotFound,
    StorageFailure,
}

impl ItineraryCreateErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidationFailed => "ITINERARY_VALIDATION_FAILED",
            Self::TargetNotFound => "ITINERARY_TARGET_NOT_FOUND",
            Self::StorageFailure => "STORAGE_FAILURE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItineraryCreateError {
    pub code: ItineraryCreateErrorCode,
    pub message: String,
}

impl ItineraryCreateError {
    fn new(code: ItineraryCreateErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self::new(ItineraryCreateErrorCode::ValidationFailed, message)
    }

    fn target_not_found(message: impl Into<String>) -> Self {
        Self::new(ItineraryCreateErrorCode::TargetNotFound, message)
    }

    fn storage(message: impl Into<String>) -> Self {
        Self::new(ItineraryCreateErrorCode::StorageFailure, message)
    }
}

impl fmt::Display for ItineraryCreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ItineraryCreateError {}

pub fn create_itinerary(
    conn: &Connection,
    params: CreateItineraryParams,
) -> Result<CreateItineraryResult, ItineraryCreateError> {
    let validated = crate::itinerary::validate_itinerary_create_fields(
        &params.title,
        params.note.as_deref(),
        params.start_time.as_deref(),
        params.location.as_deref(),
    )
    .map_err(|err| ItineraryCreateError::validation(err.to_string()))?;

    let day_id =
        crate::itinerary::resolve_itinerary_create_target(conn, params.trip_id, params.day_number)
            .map_err(classify_target_error)?;
    let sort_order =
        crate::itinerary::max_sort_order_in_day(conn, params.trip_id, params.day_number)
            .and_then(|max| {
                max.checked_add(crate::itinerary::SORT_ORDER_STEP)
                    .ok_or_else(|| anyhow::anyhow!("Itinerary sort order overflow"))
            })
            .map_err(|err| ItineraryCreateError::storage(err.to_string()))?;
    let itinerary_id = crate::itinerary::insert_validated_itinerary_item(
        conn,
        params.trip_id,
        day_id,
        params.day_number,
        &validated,
        sort_order,
        None,
        None,
        None,
    )
    .map_err(|err| ItineraryCreateError::storage(err.to_string()))?;

    Ok(CreateItineraryResult { itinerary_id })
}

fn classify_target_error(err: anyhow::Error) -> ItineraryCreateError {
    let classified = classify_read_error(err);
    match classified.code {
        ReadServiceErrorCode::TripNotFound | ReadServiceErrorCode::DayNotFound => {
            ItineraryCreateError::target_not_found(classified.message)
        }
        ReadServiceErrorCode::StorageFailure | ReadServiceErrorCode::DataMappingFailure => {
            ItineraryCreateError::storage(classified.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        crate::storage::db::open_db_at(":memory:").unwrap()
    }

    fn trip(conn: &Connection) -> i64 {
        crate::trip::add_test_trip(conn, "Quick Add Trip").unwrap()
    }

    fn params(trip_id: i64, title: &str) -> CreateItineraryParams {
        CreateItineraryParams {
            trip_id,
            day_number: 1,
            title: title.to_string(),
            start_time: None,
            location: None,
            note: None,
        }
    }

    #[test]
    fn appends_title_only_to_day_end() {
        let conn = connection();
        let trip_id = trip(&conn);
        let first = create_itinerary(&conn, params(trip_id, "First")).unwrap();
        let second = create_itinerary(&conn, params(trip_id, "Second")).unwrap();

        let timeline = crate::services::get_day_timeline(&conn, trip_id, 1).unwrap();
        assert_eq!(timeline.itineraries[0].id, first.itinerary_id);
        assert_eq!(timeline.itineraries[0].sort_order, 1000);
        assert_eq!(timeline.itineraries[1].id, second.itinerary_id);
        assert_eq!(timeline.itineraries[1].sort_order, 2000);
    }

    #[test]
    fn normalizes_fields_and_preserves_time_location_and_note() {
        let conn = connection();
        let trip_id = trip(&conn);
        let result = create_itinerary(
            &conn,
            CreateItineraryParams {
                title: "  Museum  ".to_string(),
                start_time: Some("09:30".to_string()),
                location: Some("  Naha  ".to_string()),
                note: Some("  Buy tickets  ".to_string()),
                ..params(trip_id, "ignored")
            },
        )
        .unwrap();

        let item = crate::itinerary::get_itinerary_item(&conn, result.itinerary_id).unwrap();
        assert_eq!(item.title, "Museum");
        assert_eq!(item.start_time.as_deref(), Some("09:30"));
        assert_eq!(item.location.as_deref(), Some("Naha"));
        assert_eq!(item.note.as_deref(), Some("Buy tickets"));
    }

    #[test]
    fn converts_optional_whitespace_to_null() {
        let conn = connection();
        let trip_id = trip(&conn);
        let result = create_itinerary(
            &conn,
            CreateItineraryParams {
                start_time: Some("  ".to_string()),
                location: Some("  ".to_string()),
                note: Some("  ".to_string()),
                ..params(trip_id, "Quiet time")
            },
        )
        .unwrap();
        let item = crate::itinerary::get_itinerary_item(&conn, result.itinerary_id).unwrap();
        assert_eq!(item.start_time, None);
        assert_eq!(item.location, None);
        assert_eq!(item.note, None);
    }

    #[test]
    fn rejects_blank_title_and_invalid_time() {
        let conn = connection();
        let trip_id = trip(&conn);
        let blank = create_itinerary(&conn, params(trip_id, "   ")).unwrap_err();
        assert_eq!(blank.code, ItineraryCreateErrorCode::ValidationFailed);

        let invalid_time = create_itinerary(
            &conn,
            CreateItineraryParams {
                start_time: Some("25:00".to_string()),
                ..params(trip_id, "Late")
            },
        )
        .unwrap_err();
        assert_eq!(
            invalid_time.code,
            ItineraryCreateErrorCode::ValidationFailed
        );
        assert!(crate::services::get_day_timeline(&conn, trip_id, 1)
            .unwrap()
            .itineraries
            .is_empty());
    }

    #[test]
    fn maps_missing_trip_and_day_to_target_not_found() {
        let conn = connection();
        let missing_trip = create_itinerary(&conn, params(999, "Missing")).unwrap_err();
        assert_eq!(missing_trip.code, ItineraryCreateErrorCode::TargetNotFound);

        let trip_id = trip(&conn);
        let missing_day = create_itinerary(
            &conn,
            CreateItineraryParams {
                day_number: 99,
                ..params(trip_id, "Missing day")
            },
        )
        .unwrap_err();
        assert_eq!(missing_day.code, ItineraryCreateErrorCode::TargetNotFound);
    }

    #[test]
    fn storage_failure_leaves_day_unchanged() {
        let conn = connection();
        let trip_id = trip(&conn);
        conn.execute_batch(
            "CREATE TRIGGER fail_itinerary_insert
             BEFORE INSERT ON itinerary_items
             BEGIN
               SELECT RAISE(ABORT, 'forced itinerary failure');
             END;",
        )
        .unwrap();

        let err = create_itinerary(&conn, params(trip_id, "Blocked")).unwrap_err();
        assert_eq!(err.code, ItineraryCreateErrorCode::StorageFailure);
        assert!(crate::services::get_day_timeline(&conn, trip_id, 1)
            .unwrap()
            .itineraries
            .is_empty());
    }
}
