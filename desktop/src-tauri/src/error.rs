use serde::Serialize;
use travel_ledger_cli::{ReadServiceErrorCode, ServiceError};

/// Desktop error envelope for the frontend (`{ code, message }`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DesktopError {
    pub code: String,
    pub message: String,
}

impl DesktopError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn database_not_selected() -> Self {
        Self::new(
            "DATABASE_NOT_SELECTED",
            "No Travel Ledger database is selected. Open a database to continue.",
        )
    }

    pub fn database_path_invalid(message: impl Into<String>) -> Self {
        Self::new("DATABASE_PATH_INVALID", message)
    }

    pub fn database_open_failed(message: impl Into<String>) -> Self {
        Self::new("DATABASE_OPEN_FAILED", message)
    }
}

impl From<ServiceError> for DesktopError {
    fn from(err: ServiceError) -> Self {
        Self {
            code: service_error_code(err.code).to_string(),
            message: err.message,
        }
    }
}

pub fn service_error_code(code: ReadServiceErrorCode) -> &'static str {
    match code {
        ReadServiceErrorCode::TripNotFound => "TRIP_NOT_FOUND",
        ReadServiceErrorCode::DayNotFound => "DAY_NOT_FOUND",
        ReadServiceErrorCode::StorageFailure => "STORAGE_FAILURE",
        ReadServiceErrorCode::DataMappingFailure => "DATA_MAPPING_FAILURE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use travel_ledger_cli::ReadServiceErrorCode;

    #[test]
    fn maps_service_error_codes_without_rename() {
        let err = ServiceError::new(ReadServiceErrorCode::TripNotFound, "Trip not found: 9");
        let desktop: DesktopError = err.into();
        assert_eq!(desktop.code, "TRIP_NOT_FOUND");
        assert_eq!(desktop.message, "Trip not found: 9");
    }
}
