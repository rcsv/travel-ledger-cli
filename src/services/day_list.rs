use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::{Day, Trip};

/// Read-only `day list` use case result (CLI / future GUI).
pub struct DayListServiceResult {
    pub trip: Trip,
    pub days: Vec<Day>,
}

/// Lists days for a trip without terminal I/O.
pub fn list_days(conn: &Connection, trip_id: i64) -> Result<DayListServiceResult> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let days = crate::day::list_days(conn, trip_id)?;
    Ok(DayListServiceResult { trip, days })
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
    fn service_returns_generated_days_for_trip() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Day List Trip", "2026-04-26", "2026-04-29", None)
                .unwrap();

        let result = list_days(&conn, trip_id).unwrap();
        assert_eq!(result.trip.id, trip_id);
        assert_eq!(result.days.len(), 4);
    }

    #[test]
    fn service_preserves_day_ordering() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Ordered Days", "2026-06-01", "2026-06-03", None).unwrap();

        let result = list_days(&conn, trip_id).unwrap();
        assert_eq!(result.days.len(), 3);
        assert_eq!(result.days[0].day_number, 1);
        assert_eq!(result.days[1].day_number, 2);
        assert_eq!(result.days[2].day_number, 3);
    }

    #[test]
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = list_days(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }
}
