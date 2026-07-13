//! Desktop-ready read use case facade (v4.9.2+).
//!
//! Accepts `&rusqlite::Connection` from the caller — no global DB path or singleton.

use rusqlite::Connection;

use super::dto::{
    day_to_summary, itinerary_to_detail, trip_to_detail, trip_to_summary, DayDetail, DaySummary,
    ItineraryDetail, TripDetail, TripSummary,
};
use super::read_errors::{classify_read_error, ServiceError};

/// Lists trips as UI-independent summaries.
///
/// Ordering: `trips.id` ascending (same as `trip list`).
pub fn list_trip_summaries(conn: &Connection) -> Result<Vec<TripSummary>, ServiceError> {
    let trips = crate::trip::list_trips(conn).map_err(classify_read_error)?;
    Ok(trips.iter().map(trip_to_summary).collect())
}

/// Loads trip detail with ordered day summaries.
///
/// Day ordering: `day_number` ascending.
/// Day `date` is derived from trip `start_date` when present.
pub fn get_trip_detail(conn: &Connection, trip_id: i64) -> Result<TripDetail, ServiceError> {
    let trip = crate::trip::get_trip(conn, trip_id).map_err(classify_read_error)?;
    let days = crate::day::list_days(conn, trip_id).map_err(classify_read_error)?;
    let day_summaries = map_days_to_summaries(&trip, &days)?;
    Ok(trip_to_detail(trip, day_summaries))
}

/// Loads a single day timeline with ordered itineraries.
///
/// Itinerary ordering: `sort_order` ascending, then `id` ascending (sequence-first).
pub fn get_day_timeline(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<DayDetail, ServiceError> {
    let trip = crate::trip::get_trip(conn, trip_id).map_err(classify_read_error)?;
    let day = crate::day::find_day_by_trip_and_day_number(conn, trip_id, day_number)
        .map_err(classify_read_error)?;
    let date = crate::day::day_date_for_trip(&trip, day_number).map_err(classify_read_error)?;
    let items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)
        .map_err(classify_read_error)?;
    let itineraries: Vec<ItineraryDetail> = items.iter().map(itinerary_to_detail).collect();
    Ok(DayDetail {
        trip_id,
        trip_name: trip.name,
        day_id: day.id,
        day_number,
        date,
        title: day.title,
        summary: day.summary,
        itineraries,
    })
}

fn map_days_to_summaries(
    trip: &crate::domain::models::Trip,
    days: &[crate::domain::models::Day],
) -> Result<Vec<DaySummary>, ServiceError> {
    days.iter()
        .map(|day| {
            let date =
                crate::day::day_date_for_trip(trip, day.day_number).map_err(classify_read_error)?;
            Ok(day_to_summary(day, date))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::read_errors::ReadServiceErrorCode;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn list_summaries_without_metadata() {
        let conn = test_db();
        let id =
            crate::trip::add_trip(&conn, "Plain Trip", "2026-06-01", "2026-06-02", None).unwrap();

        let summaries = list_trip_summaries(&conn).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, id);
        assert_eq!(summaries[0].name, "Plain Trip");
        assert!(summaries[0].main_destination.is_none());
    }

    #[test]
    fn list_summaries_with_metadata() {
        let conn = test_db();
        let id = crate::trip::add_trip_with_metadata(
            &conn,
            "Meta Trip",
            "2026-06-01",
            "2026-06-02",
            None,
            crate::trip::TripMetadataWrite {
                main_destination: Some("Okinawa"),
                main_destination_country_code: Some("JP"),
                default_currency: Some("JPY"),
            },
        )
        .unwrap();

        let summaries = list_trip_summaries(&conn).unwrap();
        let summary = summaries.iter().find(|s| s.id == id).unwrap();
        assert_eq!(summary.main_destination.as_deref(), Some("Okinawa"));
        assert_eq!(summary.main_destination_country_code.as_deref(), Some("JP"));
        assert_eq!(summary.default_currency.as_deref(), Some("JPY"));
    }

    #[test]
    fn trip_detail_includes_ordered_days() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Detail Trip", "2026-04-26", "2026-04-29", None).unwrap();

        let detail = get_trip_detail(&conn, trip_id).unwrap();
        assert_eq!(detail.id, trip_id);
        assert_eq!(detail.days.len(), 4);
        assert_eq!(detail.days[0].day_number, 1);
        assert_eq!(detail.days[0].date, "2026-04-26");
        assert_eq!(detail.days[3].day_number, 4);
        assert_eq!(detail.days[3].date, "2026-04-29");
    }

    #[test]
    fn trip_detail_with_metadata() {
        let conn = test_db();
        let trip_id = crate::trip::add_trip_with_metadata(
            &conn,
            "Meta Detail",
            "2026-06-01",
            "2026-06-01",
            None,
            crate::trip::TripMetadataWrite {
                main_destination: Some("Kyoto"),
                main_destination_country_code: Some("JP"),
                default_currency: Some("JPY"),
            },
        )
        .unwrap();

        let detail = get_trip_detail(&conn, trip_id).unwrap();
        assert_eq!(detail.main_destination.as_deref(), Some("Kyoto"));
        assert_eq!(detail.days.len(), 1);
    }

    #[test]
    fn trip_detail_supports_single_day_without_itineraries() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Quiet Trip", "2026-06-01", "2026-06-01", None).unwrap();

        let detail = get_trip_detail(&conn, trip_id).unwrap();
        assert_eq!(detail.days.len(), 1);
        let day = get_day_timeline(&conn, trip_id, 1).unwrap();
        assert!(day.itineraries.is_empty());
    }

    #[test]
    fn day_timeline_includes_ordered_itineraries() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Timeline", "2026-06-01", "2026-06-01", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Last",
            None,
            Some("18:00"),
            Some(20),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "First",
            None,
            Some("09:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let detail = get_day_timeline(&conn, trip_id, 1).unwrap();
        assert_eq!(detail.itineraries.len(), 2);
        assert_eq!(detail.itineraries[0].title, "First");
        assert_eq!(detail.itineraries[1].title, "Last");
    }

    #[test]
    fn empty_day_timeline_is_ok() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Empty Day", "2026-06-01", "2026-06-01", None).unwrap();

        let detail = get_day_timeline(&conn, trip_id, 1).unwrap();
        assert!(detail.itineraries.is_empty());
        assert_eq!(detail.date, "2026-06-01");
    }

    #[test]
    fn list_summaries_empty_db() {
        let conn = test_db();
        let summaries = list_trip_summaries(&conn).unwrap();
        assert!(summaries.is_empty());
    }

    #[test]
    fn day_timeline_preserves_trip_context() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Context Trip", "2026-06-01", "2026-06-03", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let detail = get_day_timeline(&conn, trip_id, 1).unwrap();
        assert_eq!(detail.trip_name, "Context Trip");
        assert_eq!(detail.day_number, 1);
        assert_eq!(detail.itineraries.len(), 1);
        assert_eq!(detail.itineraries[0].title, "首里城");
    }

    #[test]
    fn day_timeline_existing_day_date() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Day Show Trip", "2026-04-26", "2026-04-29", None)
                .unwrap();

        let detail = get_day_timeline(&conn, trip_id, 2).unwrap();
        assert_eq!(detail.date, "2026-04-27");
        assert!(detail.itineraries.is_empty());
    }

    #[test]
    fn trip_list_ordering_by_id() {
        let conn = test_db();
        let id1 = crate::trip::add_trip(&conn, "A", "2026-06-01", "2026-06-01", None).unwrap();
        let id2 = crate::trip::add_trip(&conn, "B", "2026-06-02", "2026-06-02", None).unwrap();

        let summaries = list_trip_summaries(&conn).unwrap();
        assert_eq!(summaries[0].id, id1);
        assert_eq!(summaries[1].id, id2);
    }

    #[test]
    fn trip_not_found() {
        let conn = test_db();
        let err = get_trip_detail(&conn, 9999).unwrap_err();
        assert_eq!(err.code, ReadServiceErrorCode::TripNotFound);
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn day_not_found() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Range", "2026-06-01", "2026-06-03", None).unwrap();
        let err = get_day_timeline(&conn, trip_id, 99).unwrap_err();
        assert_eq!(err.code, ReadServiceErrorCode::DayNotFound);
        assert_eq!(
            err.to_string(),
            format!("Day not found: trip {trip_id} day 99")
        );
    }
}
