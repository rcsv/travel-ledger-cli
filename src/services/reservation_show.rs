use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Reservation;

/// Read-only `reservation show` use case result (CLI / future GUI).
pub struct ReservationShowServiceResult {
    pub reservation: Reservation,
    pub day_number: Option<i64>,
    pub itinerary_title: Option<String>,
}

/// Loads a reservation and its display context without terminal I/O.
pub fn show_reservation(conn: &Connection, id: i64) -> Result<ReservationShowServiceResult> {
    let reservation = crate::reservation::get_reservation(conn, id)?;

    let (day_number, itinerary_title) =
        crate::reservation::load_reservation_display_context(conn, reservation.itinerary_id);

    Ok(ReservationShowServiceResult {
        reservation,
        day_number,
        itinerary_title,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::{params, Connection};

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn seed_trip_with_itinerary(conn: &Connection) -> (i64, i64) {
        let trip_id =
            crate::trip::add_trip(conn, "Reservation Trip", "2026-04-26", "2026-04-29", None)
                .unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn,
            trip_id,
            2,
            "Check-in",
            None,
            Some("16:40"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        (trip_id, itinerary_id)
    }

    fn add_sample_reservation(conn: &Connection, itinerary_id: i64) -> i64 {
        crate::reservation::add_reservation(
            conn,
            itinerary_id,
            "hotel",
            "Hilton Sesoko Resort",
            Some("ABC123"),
            None,
            None,
            Some("2026-04-27"),
            Some("2026-04-28"),
        )
        .unwrap()
    }

    #[test]
    fn service_returns_existing_reservation_and_preserves_fields() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);
        let id = add_sample_reservation(&conn, itinerary_id);

        let result = show_reservation(&conn, id).unwrap();
        assert_eq!(result.reservation.id, id);
        assert_eq!(result.reservation.itinerary_id, itinerary_id);
        assert_eq!(result.reservation.reservation_type, "hotel");
        assert_eq!(result.reservation.provider_name, "Hilton Sesoko Resort");
        assert_eq!(
            result.reservation.confirmation_code.as_deref(),
            Some("ABC123")
        );
        assert_eq!(result.reservation.start_at.as_deref(), Some("2026-04-27"));
        assert_eq!(result.reservation.end_at.as_deref(), Some("2026-04-28"));
    }

    #[test]
    fn service_loads_itinerary_context_when_available() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);
        let id = add_sample_reservation(&conn, itinerary_id);

        let result = show_reservation(&conn, id).unwrap();
        assert_eq!(result.day_number, Some(2));
        assert_eq!(result.itinerary_title.as_deref(), Some("Check-in"));
    }

    #[test]
    fn service_allows_missing_itinerary_context() {
        let conn = test_db();
        let now = crate::storage::db::now_string();
        conn.execute(
            "INSERT INTO reservations
             (itinerary_id, reservation_type, provider_name, confirmation_code,
              reservation_site_url, remark, start_at, end_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                9999,
                "hotel",
                "Missing Itinerary",
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                &now,
                &now,
            ],
        )
        .unwrap();
        let id = conn.last_insert_rowid();

        let result = show_reservation(&conn, id).unwrap();
        assert_eq!(result.reservation.id, id);
        assert!(result.day_number.is_none());
        assert!(result.itinerary_title.is_none());
    }

    #[test]
    fn service_preserves_not_found_error_message() {
        let conn = test_db();
        let err = show_reservation(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Reservation not found: 9999");
    }
}
