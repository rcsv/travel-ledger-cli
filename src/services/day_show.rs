use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::{Day, ItineraryItem, Trip};

/// Read-only `day show` use case result (CLI / future GUI).
pub struct DayShowServiceResult {
    pub trip: Trip,
    pub day: Day,
    pub date: String,
    pub itineraries: Vec<ItineraryItem>,
}

/// Loads trip/day context and day itineraries without terminal I/O.
pub fn show_day(conn: &Connection, trip_id: i64, day_number: i64) -> Result<DayShowServiceResult> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let day = crate::day::find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    let date = crate::day::day_date_for_trip(&trip, day_number)?;
    let itineraries = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)?;
    Ok(DayShowServiceResult {
        trip,
        day,
        date,
        itineraries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn service_returns_existing_day() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Day Show Trip", "2026-04-26", "2026-04-29", None)
                .unwrap();

        let result = show_day(&conn, trip_id, 2).unwrap();
        assert_eq!(result.trip.id, trip_id);
        assert_eq!(result.day.day_number, 2);
        assert_eq!(result.date, "2026-04-27");
        assert!(result.itineraries.is_empty());
    }

    #[test]
    fn service_preserves_trip_and_day_context() {
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

        let result = show_day(&conn, trip_id, 1).unwrap();
        assert_eq!(result.trip.name, "Context Trip");
        assert_eq!(result.day.trip_id, trip_id);
        assert_eq!(result.itineraries.len(), 1);
        assert_eq!(result.itineraries[0].title, "首里城");
    }

    #[test]
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = show_day(&conn, 9999, 1).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn service_preserves_day_not_found_error_message() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Range Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let err = show_day(&conn, trip_id, 99).err().expect("expected error");
        assert_eq!(
            err.to_string(),
            format!("Day not found: trip {trip_id} day 99")
        );
    }
}
