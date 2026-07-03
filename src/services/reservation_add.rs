use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Reservation;

/// CLI mirror of `ReservationAction::Add` fields (not a wire DTO).
pub struct ReservationAddParams {
    pub itinerary: i64,
    pub reservation_type: String,
    pub provider: String,
    pub confirmation: Option<String>,
    pub site_url: Option<String>,
    pub remark: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
}

/// Write `reservation add` use case result (CLI / future GUI).
pub struct ReservationAddServiceResult {
    pub id: i64,
    pub reservation: Reservation,
    pub day_number: Option<i64>,
    pub itinerary_title: Option<String>,
}

/// Adds a reservation and returns the persisted row with display context, without terminal I/O.
pub fn add_reservation(
    conn: &Connection,
    params: ReservationAddParams,
) -> Result<ReservationAddServiceResult> {
    let id = crate::reservation::add_reservation(
        conn,
        params.itinerary,
        &params.reservation_type,
        &params.provider,
        params.confirmation.as_deref(),
        params.site_url.as_deref(),
        params.remark.as_deref(),
        params.start_at.as_deref(),
        params.end_at.as_deref(),
    )?;
    let reservation = crate::reservation::get_reservation(conn, id)?;
    let (day_number, itinerary_title) =
        crate::reservation::load_reservation_display_context(conn, reservation.itinerary_id);
    Ok(ReservationAddServiceResult {
        id,
        reservation,
        day_number,
        itinerary_title,
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

    #[test]
    fn service_add_returns_reservation_with_context() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);

        let result = add_reservation(
            &conn,
            ReservationAddParams {
                itinerary: itinerary_id,
                reservation_type: "hotel".to_string(),
                provider: "Hilton Sesoko Resort".to_string(),
                confirmation: Some("ABC123".to_string()),
                site_url: None,
                remark: None,
                start_at: Some("2026-04-27".to_string()),
                end_at: Some("2026-04-28".to_string()),
            },
        )
        .unwrap();

        assert_eq!(result.id, result.reservation.id);
        assert_eq!(result.reservation.itinerary_id, itinerary_id);
        assert_eq!(result.reservation.reservation_type, "hotel");
        assert_eq!(result.reservation.provider_name, "Hilton Sesoko Resort");
        assert_eq!(result.day_number, Some(2));
        assert_eq!(result.itinerary_title.as_deref(), Some("Check-in"));
    }

    #[test]
    fn service_add_rejects_invalid_type() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);

        let err = add_reservation(
            &conn,
            ReservationAddParams {
                itinerary: itinerary_id,
                reservation_type: "cruise".to_string(),
                provider: "X".to_string(),
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
            },
        )
        .err()
        .expect("expected error");
        assert!(err.to_string().contains("reservation_type"));
    }

    #[test]
    fn service_add_matches_show_after_insert() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);

        let add_result = add_reservation(
            &conn,
            ReservationAddParams {
                itinerary: itinerary_id,
                reservation_type: "hotel".to_string(),
                provider: "Hilton".to_string(),
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
            },
        )
        .unwrap();

        let show_result =
            crate::services::reservation_show::show_reservation(&conn, add_result.id).unwrap();
        assert_eq!(add_result.reservation, show_result.reservation);
        assert_eq!(add_result.day_number, show_result.day_number);
        assert_eq!(add_result.itinerary_title, show_result.itinerary_title);
    }
}
