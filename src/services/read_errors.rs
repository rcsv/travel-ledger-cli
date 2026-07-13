//! Structured read-service errors for CLI / future Desktop (v4.9.2+).
//!
//! Public API returns [`ServiceError`] instead of exposing `rusqlite::Error` directly.
//! CLI maps `Display` to stderr while preserving existing human message contracts.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Machine-readable error kind for read use cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReadServiceErrorCode {
    TripNotFound,
    DayNotFound,
    StorageFailure,
    DataMappingFailure,
}

impl ReadServiceErrorCode {
    #[allow(dead_code)] // public API parity with ApplyErrorCode::as_str
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TripNotFound => "TRIP_NOT_FOUND",
            Self::DayNotFound => "DAY_NOT_FOUND",
            Self::StorageFailure => "STORAGE_FAILURE",
            Self::DataMappingFailure => "DATA_MAPPING_FAILURE",
        }
    }
}

/// Structured read-service error with stable code and human message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceError {
    pub code: ReadServiceErrorCode,
    pub message: String,
}

impl ServiceError {
    pub fn new(code: ReadServiceErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn trip_not_found(trip_id: i64) -> Self {
        Self::new(
            ReadServiceErrorCode::TripNotFound,
            format!("Trip not found: {trip_id}"),
        )
    }

    #[allow(dead_code)]
    pub fn day_not_found(trip_id: i64, day_number: i64) -> Self {
        Self::new(
            ReadServiceErrorCode::DayNotFound,
            format!("Day not found: trip {trip_id} day {day_number}"),
        )
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ServiceError {}

/// Classifies domain/storage failures into structured read errors.
pub(crate) fn classify_read_error(err: anyhow::Error) -> ServiceError {
    let message = err.to_string();
    if message.starts_with("Trip not found: ") {
        return ServiceError::new(ReadServiceErrorCode::TripNotFound, message);
    }
    if message.starts_with("Day not found: ") {
        return ServiceError::new(ReadServiceErrorCode::DayNotFound, message);
    }
    if err
        .chain()
        .any(|cause| cause.downcast_ref::<rusqlite::Error>().is_some())
    {
        return ServiceError::new(ReadServiceErrorCode::StorageFailure, message);
    }
    if message.contains("start_date")
        || message.contains("日付の形式")
        || message.contains("Trip ") && message.contains(" has no start_date")
    {
        return ServiceError::new(ReadServiceErrorCode::DataMappingFailure, message);
    }
    ServiceError::new(ReadServiceErrorCode::StorageFailure, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_message_contract() {
        let err = ServiceError::trip_not_found(42);
        assert_eq!(err.code, ReadServiceErrorCode::TripNotFound);
        assert_eq!(err.to_string(), "Trip not found: 42");
    }

    #[test]
    fn classifies_day_not_found() {
        let err = classify_read_error(anyhow::anyhow!("Day not found: trip 1 day 99"));
        assert_eq!(err.code, ReadServiceErrorCode::DayNotFound);
    }
}
