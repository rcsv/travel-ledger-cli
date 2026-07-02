use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::{ItineraryItem, Trip};

/// Read-only `itinerary timeline` use case result (CLI / future GUI).
pub struct ItineraryTimelineServiceResult {
    pub trip: Trip,
    pub items: Vec<ItineraryItem>,
}

/// Loads itinerary timeline data without terminal I/O.
pub fn get_timeline(conn: &Connection, trip_id: i64) -> Result<ItineraryTimelineServiceResult> {
    let items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let trip = crate::trip::get_trip(conn, trip_id)?;
    Ok(ItineraryTimelineServiceResult { trip, items })
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
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = get_timeline(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn service_preserves_sequence_first_ordering() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Timeline Trip", "2026-06-01", "2026-06-02", None)
                .unwrap();

        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            Some("10:50"),
            Some(2),
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
            "首里城",
            None,
            Some("09:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let result = get_timeline(&conn, trip_id).unwrap();
        assert_eq!(result.trip.name, "Timeline Trip");
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].title, "首里城");
        assert_eq!(result.items[1].title, "国際通り");
        assert_eq!(result.items[2].title, "2日目");
    }

    #[test]
    fn service_orders_by_sort_order_not_start_time() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Sort Order Trip", "2026-06-01", "2026-06-01", None)
                .unwrap();

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
            "Middle no time",
            None,
            None,
            Some(10),
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

        let result = get_timeline(&conn, trip_id).unwrap();
        assert_eq!(
            result
                .items
                .iter()
                .map(|i| i.title.as_str())
                .collect::<Vec<_>>(),
            vec!["First", "Middle no time", "Last"]
        );
    }
}
