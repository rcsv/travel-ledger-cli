use serde::Serialize;
use travel_ledger_cli::{
    ItineraryCreateError, ItineraryReorderError, ItineraryUpdateError, ReadServiceErrorCode,
    ServiceError, TripCreateError,
};

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

    pub fn database_config_read_failed(message: impl Into<String>) -> Self {
        Self::new("DATABASE_CONFIG_READ_FAILED", message)
    }

    pub fn database_config_write_failed(message: impl Into<String>) -> Self {
        Self::new("DATABASE_CONFIG_WRITE_FAILED", message)
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

impl From<TripCreateError> for DesktopError {
    fn from(err: TripCreateError) -> Self {
        Self {
            code: err.code.as_str().to_string(),
            message: err.message,
        }
    }
}

impl From<ItineraryCreateError> for DesktopError {
    fn from(err: ItineraryCreateError) -> Self {
        Self {
            code: err.code.as_str().to_string(),
            message: err.message,
        }
    }
}

impl From<ItineraryUpdateError> for DesktopError {
    fn from(err: ItineraryUpdateError) -> Self {
        Self {
            code: err.code.as_str().to_string(),
            message: err.message,
        }
    }
}

impl From<ItineraryReorderError> for DesktopError {
    fn from(err: ItineraryReorderError) -> Self {
        Self {
            code: err.code.as_str().to_string(),
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

    #[test]
    fn maps_trip_create_error_codes_without_rename() {
        let err = TripCreateError {
            code: travel_ledger_cli::TripCreateErrorCode::ValidationFailed,
            message: "invalid trip".to_string(),
        };
        let desktop: DesktopError = err.into();
        assert_eq!(desktop.code, "TRIP_VALIDATION_FAILED");
        assert_eq!(desktop.message, "invalid trip");
    }

    #[test]
    fn maps_itinerary_create_error_codes_without_rename() {
        let err = ItineraryCreateError {
            code: travel_ledger_cli::ItineraryCreateErrorCode::TargetNotFound,
            message: "missing target".to_string(),
        };
        let desktop: DesktopError = err.into();
        assert_eq!(desktop.code, "ITINERARY_TARGET_NOT_FOUND");
        assert_eq!(desktop.message, "missing target");
    }

    #[test]
    fn maps_itinerary_update_error_codes_without_rename() {
        let err = ItineraryUpdateError {
            code: travel_ledger_cli::ItineraryUpdateErrorCode::TargetNotFound,
            message: "missing target".to_string(),
        };
        let desktop: DesktopError = err.into();
        assert_eq!(desktop.code, "ITINERARY_TARGET_NOT_FOUND");
        assert_eq!(desktop.message, "missing target");
    }

    #[test]
    fn maps_itinerary_reorder_error_codes_without_rename() {
        let err = ItineraryReorderError {
            code: travel_ledger_cli::ItineraryReorderErrorCode::PlacementConflict,
            message: "stale order".to_string(),
        };
        let desktop: DesktopError = err.into();
        assert_eq!(desktop.code, "ITINERARY_PLACEMENT_CONFLICT");
        assert_eq!(desktop.message, "stale order");
    }
}
