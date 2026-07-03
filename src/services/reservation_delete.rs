use anyhow::Result;
use rusqlite::Connection;

/// Write `reservation delete` use case result — pre-delete snapshot (no display context).
pub struct ReservationDeleteServiceResult {
    pub id: i64,
    pub provider_name: String,
}

/// CLI mirror of `ReservationAction::Delete` fields (not a wire DTO).
pub struct ReservationDeleteParams {
    pub id: i64,
}

/// Deletes a reservation and returns a pre-delete snapshot, without terminal I/O.
pub fn delete_reservation(
    conn: &Connection,
    params: ReservationDeleteParams,
) -> Result<ReservationDeleteServiceResult> {
    let reservation = crate::reservation::get_reservation(conn, params.id)?;
    let result = ReservationDeleteServiceResult {
        id: reservation.id,
        provider_name: reservation.provider_name.clone(),
    };
    crate::reservation::delete_reservation(conn, params.id)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn seed_reservation(conn: &Connection) -> i64 {
        let trip_id =
            crate::trip::add_trip(conn, "Reservation Trip", "2026-04-26", "2026-04-29", None)
                .unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn, trip_id, 1, "Check-in", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::reservation::add_reservation(
            conn,
            itinerary_id,
            "hotel",
            "Hilton Sesoko",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn service_delete_returns_snapshot() {
        let conn = test_db();
        let id = seed_reservation(&conn);

        let result = delete_reservation(&conn, ReservationDeleteParams { id }).unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.provider_name, "Hilton Sesoko");

        let err = crate::reservation::get_reservation(&conn, id)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), format!("Reservation not found: {id}"));
    }

    #[test]
    fn service_delete_not_found() {
        let conn = test_db();
        let err = delete_reservation(&conn, ReservationDeleteParams { id: 9999 })
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Reservation not found: 9999");
    }
}
