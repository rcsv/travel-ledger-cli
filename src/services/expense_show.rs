use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Expense;

/// Read-only `expense show` use case result (CLI / future GUI).
pub struct ExpenseShowServiceResult {
    pub expense: Expense,
}

/// Loads an expense without terminal I/O.
pub fn show_expense(conn: &Connection, id: i64) -> Result<ExpenseShowServiceResult> {
    let expense = crate::expense::get_expense(conn, id)?;
    Ok(ExpenseShowServiceResult { expense })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expense::ExpenseSharedOptions;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn seed_trip_with_itinerary(conn: &Connection) -> (i64, i64) {
        let trip_id =
            crate::trip::add_trip(conn, "Expense Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap();
        (trip_id, itinerary_id)
    }

    fn add_sample_expense(conn: &Connection, itinerary_id: i64) -> i64 {
        crate::expense::add_expense(
            conn,
            itinerary_id,
            "2200",
            "JPY",
            Some("Lunch"),
            None,
            Some("Tomo"),
            Some("2026-04-27"),
            &ExpenseSharedOptions::default(),
        )
        .unwrap()
    }

    #[test]
    fn service_returns_existing_expense_and_preserves_fields() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);
        let id = add_sample_expense(&conn, itinerary_id);

        let result = show_expense(&conn, id).unwrap();
        assert_eq!(result.expense.id, id);
        assert_eq!(result.expense.itinerary_id, itinerary_id);
        assert_eq!(result.expense.title.as_deref(), Some("Lunch"));
        assert_eq!(result.expense.amount, 2200);
        assert_eq!(result.expense.currency, "JPY");
        assert_eq!(result.expense.paid_by_name.as_deref(), Some("Tomo"));
        assert_eq!(result.expense.expense_date.as_deref(), Some("2026-04-27"));
    }

    #[test]
    fn service_preserves_participant_payer_field() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Shared Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let payer_id =
            crate::participant::create_participant(&conn, trip_id, "Alice", None, true).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Dinner", None, None, None, None, None, None, None,
        )
        .unwrap();
        let id = crate::expense::add_expense(
            &conn,
            itinerary_id,
            "3000",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(payer_id),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();

        let result = show_expense(&conn, id).unwrap();
        assert_eq!(result.expense.paid_by_participant_id, Some(payer_id));
    }

    #[test]
    fn service_preserves_not_found_error_message() {
        let conn = test_db();
        let err = show_expense(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Expense not found: 9999");
    }
}
