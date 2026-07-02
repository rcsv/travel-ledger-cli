use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Reservation;
use crate::reservation::{ReservationListTarget, ReservationWithContext};

/// Read-only `reservation list` use case result (CLI / future GUI).
pub struct ReservationListServiceResult {
    pub target: ReservationListTarget,
    pub reservations: Vec<Reservation>,
    pub trip_context: Option<Vec<ReservationWithContext>>,
}

/// Resolves the list target and loads reservations without terminal I/O.
pub fn list_reservations(
    conn: &Connection,
    trip: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ReservationListServiceResult> {
    let target = crate::reservation::resolve_reservation_list_target(trip, itinerary)?;
    match target {
        ReservationListTarget::Trip(trip_id) => {
            let trip_context = crate::reservation::list_reservations_for_trip(conn, trip_id)?;
            let reservations = trip_context
                .iter()
                .map(|row| row.reservation.clone())
                .collect();
            Ok(ReservationListServiceResult {
                target,
                reservations,
                trip_context: Some(trip_context),
            })
        }
        ReservationListTarget::Itinerary(itinerary_id) => {
            let reservations =
                crate::reservation::list_reservations_for_itinerary(conn, itinerary_id)?;
            Ok(ReservationListServiceResult {
                target,
                reservations,
                trip_context: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

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
            1,
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
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn service_returns_reservations_for_itinerary_target() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);
        add_sample_reservation(&conn, itinerary_id);

        let result = list_reservations(&conn, None, Some(itinerary_id)).unwrap();
        assert_eq!(
            result.target,
            ReservationListTarget::Itinerary(itinerary_id)
        );
        assert_eq!(result.reservations.len(), 1);
        assert_eq!(
            result.reservations[0].confirmation_code.as_deref(),
            Some("ABC123")
        );
        assert!(result.trip_context.is_none());
    }

    #[test]
    fn service_returns_reservations_for_trip_target_with_context() {
        let conn = test_db();
        let (trip_id, itinerary_id) = seed_trip_with_itinerary(&conn);
        add_sample_reservation(&conn, itinerary_id);

        let result = list_reservations(&conn, Some(trip_id), None).unwrap();
        assert_eq!(result.target, ReservationListTarget::Trip(trip_id));
        assert_eq!(result.reservations.len(), 1);
        let context = result.trip_context.expect("expected trip context");
        assert_eq!(context.len(), 1);
        assert_eq!(context[0].day_number, 1);
        assert_eq!(context[0].itinerary_title, "Check-in");
    }

    #[test]
    fn service_returns_empty_list_for_target_without_reservations() {
        let conn = test_db();
        let (trip_id, itinerary_id) = seed_trip_with_itinerary(&conn);

        let trip_result = list_reservations(&conn, Some(trip_id), None).unwrap();
        assert!(trip_result.reservations.is_empty());
        assert_eq!(trip_result.trip_context.unwrap().len(), 0);

        let itinerary_result = list_reservations(&conn, None, Some(itinerary_id)).unwrap();
        assert!(itinerary_result.reservations.is_empty());
    }

    #[test]
    fn service_preserves_itinerary_not_found_error_message() {
        let conn = test_db();
        let err = list_reservations(&conn, None, Some(9999))
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Itinerary not found: 9999");
    }

    #[test]
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = list_reservations(&conn, Some(9999), None)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn service_preserves_target_resolution_error() {
        let conn = test_db();
        let err = list_reservations(&conn, None, None)
            .err()
            .expect("expected error");
        assert!(err
            .to_string()
            .contains("--trip または --itinerary のいずれかを指定してください"));
    }
}
