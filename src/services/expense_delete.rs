use anyhow::Result;
use rusqlite::Connection;

/// Write `expense delete` use case result — pre-delete snapshot (no enrich).
pub struct ExpenseDeleteServiceResult {
    pub id: i64,
    pub amount: i64,
    pub currency: String,
}

/// Deletes an expense and returns a pre-delete snapshot, without terminal I/O.
pub fn delete_expense(conn: &Connection, id: i64) -> Result<ExpenseDeleteServiceResult> {
    let expense = crate::expense::get_expense(conn, id)?;
    let result = ExpenseDeleteServiceResult {
        id: expense.id,
        amount: expense.amount,
        currency: expense.currency.clone(),
    };
    crate::expense::delete_expense(conn, id)?;
    Ok(result)
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

    fn seed_expense(conn: &Connection) -> i64 {
        let trip_id =
            crate::trip::add_trip(conn, "Expense Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap();
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
    fn service_delete_returns_snapshot() {
        let conn = test_db();
        let id = seed_expense(&conn);

        let result = delete_expense(&conn, id).unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.amount, 2200);
        assert_eq!(result.currency, "JPY");

        let err = crate::expense::get_expense(&conn, id)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), format!("Expense not found: {id}"));
    }

    #[test]
    fn service_delete_not_found() {
        let conn = test_db();
        let err = delete_expense(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Expense not found: 9999");
    }
}
