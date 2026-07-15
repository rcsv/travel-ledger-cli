//! Verifies the library target exposes application services + DB boundary to external crates.

use travel_ledger_cli::{
    create_trip, get_day_timeline, get_trip_detail, list_trip_summaries, open_db, CreateTripParams,
    ReadServiceErrorCode,
};

#[test]
fn lib_open_db_and_list_trip_summaries() {
    let conn = open_db(":memory:").expect("open in-memory db");
    let summaries = list_trip_summaries(&conn).expect("list trips");
    assert!(summaries.is_empty());
}

#[test]
fn lib_get_trip_detail_not_found() {
    let conn = open_db(":memory:").expect("open in-memory db");
    let err = get_trip_detail(&conn, 1).unwrap_err();
    assert_eq!(err.code, ReadServiceErrorCode::TripNotFound);
}

#[test]
fn lib_get_day_timeline_not_found() {
    let conn = open_db(":memory:").expect("open in-memory db");
    let err = get_day_timeline(&conn, 1, 1).unwrap_err();
    assert_eq!(err.code, ReadServiceErrorCode::TripNotFound);
}

#[test]
fn lib_create_trip_is_public_and_readable() {
    let mut conn = open_db(":memory:").expect("open in-memory db");
    let result = create_trip(
        &mut conn,
        CreateTripParams {
            name: "Library Trip".to_string(),
            start_date: "2026-10-01".to_string(),
            end_date: "2026-10-02".to_string(),
            summary: None,
            main_destination: None,
            main_destination_country_code: None,
            default_currency: None,
        },
    )
    .expect("create trip");
    let detail = get_trip_detail(&conn, result.trip_id).expect("read created trip");
    assert_eq!(detail.name, "Library Trip");
    assert_eq!(detail.days.len(), 2);
}
