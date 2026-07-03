use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Reservation;

/// CLI mirror of `ReservationAction::Update` fields (not a wire DTO).
pub struct ReservationUpdateParams {
    pub id: i64,
    pub reservation_type: Option<String>,
    pub provider: Option<String>,
    pub confirmation: Option<String>,
    pub site_url: Option<String>,
    pub remark: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub clear_confirmation: bool,
    pub clear_site_url: bool,
    pub clear_remark: bool,
    pub clear_start_at: bool,
    pub clear_end_at: bool,
}

/// Write `reservation update` use case result (CLI / future GUI).
pub struct ReservationUpdateServiceResult {
    pub reservation: Reservation,
    pub day_number: Option<i64>,
    pub itinerary_title: Option<String>,
}

fn optional_str_field_update(clear: bool, value: &Option<String>) -> Option<Option<&str>> {
    if clear {
        Some(None)
    } else {
        value.as_ref().map(|v| Some(v.as_str()))
    }
}

/// Updates a reservation and returns the persisted row with display context, without terminal I/O.
pub fn update_reservation(
    conn: &Connection,
    params: ReservationUpdateParams,
) -> Result<ReservationUpdateServiceResult> {
    let confirmation_update =
        optional_str_field_update(params.clear_confirmation, &params.confirmation);
    let site_url_update = optional_str_field_update(params.clear_site_url, &params.site_url);
    let remark_update = optional_str_field_update(params.clear_remark, &params.remark);
    let start_at_update = optional_str_field_update(params.clear_start_at, &params.start_at);
    let end_at_update = optional_str_field_update(params.clear_end_at, &params.end_at);

    crate::reservation::update_reservation(
        conn,
        params.id,
        params.reservation_type.as_deref(),
        params.provider.as_deref(),
        confirmation_update,
        site_url_update,
        remark_update,
        start_at_update,
        end_at_update,
    )?;
    let reservation = crate::reservation::get_reservation(conn, params.id)?;
    let (day_number, itinerary_title) =
        crate::reservation::load_reservation_display_context(conn, reservation.itinerary_id);
    Ok(ReservationUpdateServiceResult {
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

    fn seed_reservation(conn: &Connection) -> (i64, i64) {
        let trip_id =
            crate::trip::add_trip(conn, "Reservation Trip", "2026-04-26", "2026-04-29", None)
                .unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn, trip_id, 1, "Check-in", None, None, None, None, None, None, None,
        )
        .unwrap();
        let id = crate::reservation::add_reservation(
            conn,
            itinerary_id,
            "hotel",
            "Hilton",
            Some("ABC123"),
            Some("https://example.com"),
            Some("remark"),
            Some("2026-04-26"),
            Some("2026-04-29"),
        )
        .unwrap();
        (id, itinerary_id)
    }

    #[test]
    fn service_update_returns_reservation_with_context() {
        let conn = test_db();
        let (id, _) = seed_reservation(&conn);

        let result = update_reservation(
            &conn,
            ReservationUpdateParams {
                id,
                reservation_type: None,
                provider: Some("Updated Hilton".to_string()),
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
                clear_confirmation: false,
                clear_site_url: false,
                clear_remark: false,
                clear_start_at: false,
                clear_end_at: false,
            },
        )
        .unwrap();

        assert_eq!(result.reservation.id, id);
        assert_eq!(result.reservation.provider_name, "Updated Hilton");
        assert_eq!(result.day_number, Some(1));
        assert_eq!(result.itinerary_title.as_deref(), Some("Check-in"));
    }

    #[test]
    fn service_update_clear_confirmation() {
        let conn = test_db();
        let (id, _) = seed_reservation(&conn);

        let result = update_reservation(
            &conn,
            ReservationUpdateParams {
                id,
                reservation_type: None,
                provider: None,
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
                clear_confirmation: true,
                clear_site_url: false,
                clear_remark: false,
                clear_start_at: false,
                clear_end_at: false,
            },
        )
        .unwrap();

        assert!(result.reservation.confirmation_code.is_none());
    }

    #[test]
    fn service_update_clear_site_url_and_remark() {
        let conn = test_db();
        let (id, _) = seed_reservation(&conn);

        let result = update_reservation(
            &conn,
            ReservationUpdateParams {
                id,
                reservation_type: None,
                provider: None,
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
                clear_confirmation: false,
                clear_site_url: true,
                clear_remark: true,
                clear_start_at: false,
                clear_end_at: false,
            },
        )
        .unwrap();

        assert!(result.reservation.reservation_site_url.is_none());
        assert!(result.reservation.remark.is_none());
    }

    #[test]
    fn service_update_clear_start_at_and_end_at() {
        let conn = test_db();
        let (id, _) = seed_reservation(&conn);

        let result = update_reservation(
            &conn,
            ReservationUpdateParams {
                id,
                reservation_type: None,
                provider: None,
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
                clear_confirmation: false,
                clear_site_url: false,
                clear_remark: false,
                clear_start_at: true,
                clear_end_at: true,
            },
        )
        .unwrap();

        assert!(result.reservation.start_at.is_none());
        assert!(result.reservation.end_at.is_none());
    }

    #[test]
    fn service_update_not_found() {
        let conn = test_db();
        let err = update_reservation(
            &conn,
            ReservationUpdateParams {
                id: 9999,
                reservation_type: Some("hotel".to_string()),
                provider: None,
                confirmation: None,
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
                clear_confirmation: false,
                clear_site_url: false,
                clear_remark: false,
                clear_start_at: false,
                clear_end_at: false,
            },
        )
        .err()
        .expect("expected error");
        assert_eq!(err.to_string(), "Reservation not found: 9999");
    }

    #[test]
    fn service_update_matches_show_after_update() {
        let conn = test_db();
        let (id, _) = seed_reservation(&conn);

        let update_result = update_reservation(
            &conn,
            ReservationUpdateParams {
                id,
                reservation_type: None,
                provider: Some("New Provider".to_string()),
                confirmation: Some("NEW123".to_string()),
                site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
                clear_confirmation: false,
                clear_site_url: false,
                clear_remark: false,
                clear_start_at: false,
                clear_end_at: false,
            },
        )
        .unwrap();

        let show_result = crate::services::reservation_show::show_reservation(&conn, id).unwrap();
        assert_eq!(update_result.reservation, show_result.reservation);
        assert_eq!(update_result.day_number, show_result.day_number);
        assert_eq!(update_result.itinerary_title, show_result.itinerary_title);
    }
}
