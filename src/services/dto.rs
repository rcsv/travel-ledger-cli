//! UI-independent read DTOs for Desktop / CLI adapters (v4.9.2+).
//!
//! Not DB row types, not export schema types, and not CLI display strings.

use serde::{Deserialize, Serialize};

use crate::domain::models::{Day, ItineraryCategory, ItineraryItem, Trip};

/// Trip list row — lightweight summary including optional metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TripSummary {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_destination_country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_currency: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Day row within a trip detail — calendar date derived when trip has `start_date`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaySummary {
    pub id: i64,
    pub trip_id: i64,
    pub day_number: i64,
    pub date: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Trip detail with ordered day summaries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TripDetail {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_destination_country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_currency: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub days: Vec<DaySummary>,
}

/// Itinerary row for day timeline — sequence-first ordering metadata preserved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItineraryDetail {
    pub id: i64,
    pub trip_id: i64,
    pub day_number: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    pub sort_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ItineraryCategory>,
    pub created_at: String,
    pub updated_at: String,
}

/// Day detail with ordered itinerary timeline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayDetail {
    pub trip_id: i64,
    pub trip_name: String,
    pub day_id: i64,
    pub day_number: i64,
    pub date: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub itineraries: Vec<ItineraryDetail>,
}

pub(crate) fn trip_to_summary(trip: &Trip) -> TripSummary {
    TripSummary {
        id: trip.id,
        name: trip.name.clone(),
        start_date: trip.start_date.clone(),
        end_date: trip.end_date.clone(),
        summary: trip.summary.clone(),
        main_destination: trip.main_destination.clone(),
        main_destination_country_code: trip.main_destination_country_code.clone(),
        default_currency: trip.default_currency.clone(),
        created_at: trip.created_at.clone(),
        updated_at: trip.updated_at.clone(),
    }
}

pub(crate) fn trip_to_detail(trip: Trip, days: Vec<DaySummary>) -> TripDetail {
    TripDetail {
        id: trip.id,
        name: trip.name,
        start_date: trip.start_date,
        end_date: trip.end_date,
        summary: trip.summary,
        main_destination: trip.main_destination,
        main_destination_country_code: trip.main_destination_country_code,
        default_currency: trip.default_currency,
        created_at: trip.created_at,
        updated_at: trip.updated_at,
        days,
    }
}

pub(crate) fn day_to_summary(day: &Day, date: String) -> DaySummary {
    DaySummary {
        id: day.id,
        trip_id: day.trip_id,
        day_number: day.day_number,
        date,
        title: day.title.clone(),
        summary: day.summary.clone(),
    }
}

pub(crate) fn itinerary_to_detail(item: &ItineraryItem) -> ItineraryDetail {
    ItineraryDetail {
        id: item.id,
        trip_id: item.trip_id,
        day_number: item.day,
        title: item.title.clone(),
        note: item.note.clone(),
        start_time: item.start_time.clone(),
        sort_order: item.sort_order,
        duration_minutes: item.duration_minutes,
        travel_minutes: item.travel_minutes,
        location: item.location.clone(),
        category: item.category,
        created_at: item.created_at.clone(),
        updated_at: item.updated_at.clone(),
    }
}

pub(crate) fn itinerary_detail_to_domain(item: &ItineraryDetail) -> ItineraryItem {
    ItineraryItem {
        id: item.id,
        trip_id: item.trip_id,
        day: item.day_number,
        title: item.title.clone(),
        note: item.note.clone(),
        start_time: item.start_time.clone(),
        sort_order: item.sort_order,
        duration_minutes: item.duration_minutes,
        travel_minutes: item.travel_minutes,
        location: item.location.clone(),
        category: item.category,
        created_at: item.created_at.clone(),
        updated_at: item.updated_at.clone(),
    }
}
