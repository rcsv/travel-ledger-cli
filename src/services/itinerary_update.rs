//! Narrow Itinerary content update use case for the Desktop Inspector.

use std::fmt;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateItineraryParams {
    pub trip_id: i64,
    pub day_number: i64,
    pub itinerary_id: i64,
    pub title: String,
    pub start_time: Option<String>,
    pub location: Option<String>,
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateItineraryResult {
    pub itinerary_id: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItineraryUpdateErrorCode {
    ValidationFailed,
    TargetNotFound,
    StorageFailure,
}

impl ItineraryUpdateErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidationFailed => "ITINERARY_VALIDATION_FAILED",
            Self::TargetNotFound => "ITINERARY_TARGET_NOT_FOUND",
            Self::StorageFailure => "STORAGE_FAILURE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItineraryUpdateError {
    pub code: ItineraryUpdateErrorCode,
    pub message: String,
}

impl ItineraryUpdateError {
    fn new(code: ItineraryUpdateErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self::new(ItineraryUpdateErrorCode::ValidationFailed, message)
    }

    fn target_not_found(message: impl Into<String>) -> Self {
        Self::new(ItineraryUpdateErrorCode::TargetNotFound, message)
    }

    fn storage(message: impl Into<String>) -> Self {
        Self::new(ItineraryUpdateErrorCode::StorageFailure, message)
    }
}

impl fmt::Display for ItineraryUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ItineraryUpdateError {}

pub fn update_itinerary(
    conn: &Connection,
    params: UpdateItineraryParams,
) -> Result<UpdateItineraryResult, ItineraryUpdateError> {
    let validated = crate::itinerary::validate_itinerary_content_fields(
        &params.title,
        params.note.as_deref(),
        params.start_time.as_deref(),
        params.location.as_deref(),
    )
    .map_err(|err| ItineraryUpdateError::validation(err.to_string()))?;

    crate::itinerary::resolve_itinerary_update_target(
        conn,
        params.itinerary_id,
        params.trip_id,
        params.day_number,
    )
    .map_err(|err| {
        if err
            .chain()
            .any(|cause| cause.downcast_ref::<rusqlite::Error>().is_some())
        {
            ItineraryUpdateError::storage(err.to_string())
        } else {
            ItineraryUpdateError::target_not_found(err.to_string())
        }
    })?;

    let updated = crate::itinerary::update_validated_itinerary_content(
        conn,
        params.itinerary_id,
        params.trip_id,
        params.day_number,
        &validated,
    )
    .map_err(|err| ItineraryUpdateError::storage(err.to_string()))?;

    if updated != 1 {
        return Err(ItineraryUpdateError::target_not_found(format!(
            "Itinerary target not found during update: itinerary {}, trip {}, day {}",
            params.itinerary_id, params.trip_id, params.day_number
        )));
    }

    Ok(UpdateItineraryResult {
        itinerary_id: params.itinerary_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ItineraryCategory;

    fn connection() -> Connection {
        crate::storage::db::open_db_at(":memory:").unwrap()
    }

    fn setup_item(conn: &Connection) -> (i64, i64) {
        let trip_id = crate::trip::add_test_trip(conn, "Inspector Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn,
            trip_id,
            1,
            "Original",
            Some("Original note"),
            Some("08:00"),
            Some(4200),
            Some(90),
            Some(15),
            Some("Original place"),
            Some(ItineraryCategory::Museum),
        )
        .unwrap();
        (trip_id, itinerary_id)
    }

    fn params(trip_id: i64, itinerary_id: i64) -> UpdateItineraryParams {
        UpdateItineraryParams {
            trip_id,
            day_number: 1,
            itinerary_id,
            title: "  Updated activity  ".to_string(),
            start_time: Some("09:30".to_string()),
            location: Some("  Updated place  ".to_string()),
            note: Some("  Updated note  ".to_string()),
        }
    }

    #[test]
    fn updates_four_fields_and_preserves_non_scope_fields() {
        let conn = connection();
        let (trip_id, itinerary_id) = setup_item(&conn);

        let result = update_itinerary(&conn, params(trip_id, itinerary_id)).unwrap();
        assert_eq!(result.itinerary_id, itinerary_id);

        let timeline = crate::services::get_day_timeline(&conn, trip_id, 1).unwrap();
        let item = &timeline.itineraries[0];
        assert_eq!(item.title, "Updated activity");
        assert_eq!(item.start_time.as_deref(), Some("09:30"));
        assert_eq!(item.location.as_deref(), Some("Updated place"));
        assert_eq!(item.note.as_deref(), Some("Updated note"));
        assert_eq!(item.day_number, 1);
        assert_eq!(item.sort_order, 4200);
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(15));
        assert_eq!(item.category, Some(ItineraryCategory::Museum));
    }

    #[test]
    fn clears_optional_whitespace_and_rejects_invalid_content() {
        let conn = connection();
        let (trip_id, itinerary_id) = setup_item(&conn);
        let cleared = UpdateItineraryParams {
            title: "Kept".to_string(),
            start_time: Some("  ".to_string()),
            location: Some("  ".to_string()),
            note: Some("  ".to_string()),
            ..params(trip_id, itinerary_id)
        };
        update_itinerary(&conn, cleared).unwrap();
        let item = crate::itinerary::get_itinerary_item(&conn, itinerary_id).unwrap();
        assert_eq!(item.start_time, None);
        assert_eq!(item.location, None);
        assert_eq!(item.note, None);

        let blank = UpdateItineraryParams {
            title: "   ".to_string(),
            ..params(trip_id, itinerary_id)
        };
        assert_eq!(
            update_itinerary(&conn, blank).unwrap_err().code,
            ItineraryUpdateErrorCode::ValidationFailed
        );
        let invalid_time = UpdateItineraryParams {
            start_time: Some("25:00".to_string()),
            ..params(trip_id, itinerary_id)
        };
        assert_eq!(
            update_itinerary(&conn, invalid_time).unwrap_err().code,
            ItineraryUpdateErrorCode::ValidationFailed
        );
    }

    #[test]
    fn rejects_missing_and_mismatched_targets() {
        let conn = connection();
        let (trip_id, itinerary_id) = setup_item(&conn);
        let missing = UpdateItineraryParams {
            itinerary_id: 999,
            ..params(trip_id, itinerary_id)
        };
        assert_eq!(
            update_itinerary(&conn, missing).unwrap_err().code,
            ItineraryUpdateErrorCode::TargetNotFound
        );
        let wrong_trip = UpdateItineraryParams {
            trip_id: 999,
            ..params(trip_id, itinerary_id)
        };
        assert_eq!(
            update_itinerary(&conn, wrong_trip).unwrap_err().code,
            ItineraryUpdateErrorCode::TargetNotFound
        );
        let wrong_day = UpdateItineraryParams {
            day_number: 2,
            ..params(trip_id, itinerary_id)
        };
        assert_eq!(
            update_itinerary(&conn, wrong_day).unwrap_err().code,
            ItineraryUpdateErrorCode::TargetNotFound
        );
    }

    #[test]
    fn context_scoped_zero_row_does_not_change_item() {
        let conn = connection();
        let (trip_id, itinerary_id) = setup_item(&conn);
        let validated = crate::itinerary::validate_itinerary_content_fields(
            "Should not save",
            None,
            None,
            None,
        )
        .unwrap();
        let updated = crate::itinerary::update_validated_itinerary_content(
            &conn,
            itinerary_id,
            trip_id,
            2,
            &validated,
        )
        .unwrap();
        assert_eq!(updated, 0);
        assert_eq!(
            crate::itinerary::get_itinerary_item(&conn, itinerary_id)
                .unwrap()
                .title,
            "Original"
        );
    }

    #[test]
    fn storage_failure_leaves_original_item_unchanged() {
        let conn = connection();
        let (trip_id, itinerary_id) = setup_item(&conn);
        conn.execute_batch(
            "CREATE TRIGGER fail_itinerary_update
             BEFORE UPDATE ON itinerary_items
             BEGIN
               SELECT RAISE(ABORT, 'forced itinerary update failure');
             END;",
        )
        .unwrap();

        let err = update_itinerary(&conn, params(trip_id, itinerary_id)).unwrap_err();
        assert_eq!(err.code, ItineraryUpdateErrorCode::StorageFailure);
        let item = crate::itinerary::get_itinerary_item(&conn, itinerary_id).unwrap();
        assert_eq!(item.title, "Original");
        assert_eq!(item.start_time.as_deref(), Some("08:00"));
    }
}
