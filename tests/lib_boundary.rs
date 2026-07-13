//! Verifies the library target exposes read facade + DB boundary to external crates.

use travel_ledger_cli::{
    get_day_timeline, get_trip_detail, list_trip_summaries, open_db, ReadServiceErrorCode,
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
