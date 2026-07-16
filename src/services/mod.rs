//! Application service layer — use case orchestration with structured results (no terminal I/O).

pub mod dto;
mod itinerary_create;
pub mod read_errors;
mod read_facade;
mod trip_create;

#[allow(unused_imports)] // re-exported at crate root for Desktop consumers
pub use dto::{DayDetail, DaySummary, ItineraryDetail, TripDetail, TripSummary};
pub use itinerary_create::{
    create_itinerary, CreateItineraryParams, CreateItineraryResult, ItineraryCreateError,
    ItineraryCreateErrorCode,
};
pub use read_errors::{ReadServiceErrorCode, ServiceError};
pub use read_facade::{get_day_timeline, get_trip_detail, list_trip_summaries};
pub use trip_create::{
    create_trip, CreateTripParams, CreateTripResult, TripCreateError, TripCreateErrorCode,
};

pub mod checklist_list;
pub mod checklist_show;
pub mod expense_add;
pub mod expense_delete;
pub mod expense_list;
pub mod expense_show;
pub mod expense_update;
pub mod itinerary_list;
pub mod itinerary_show;
pub mod itinerary_timeline;
pub mod note_add;
pub mod note_delete;
pub mod note_list;
pub mod note_show;
pub mod note_update;
pub mod reservation_add;
pub mod reservation_delete;
pub mod reservation_list;
pub mod reservation_show;
pub mod reservation_update;
pub mod trip_stats;
