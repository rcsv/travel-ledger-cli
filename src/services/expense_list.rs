use anyhow::Result;
use rusqlite::Connection;

use crate::expense::{ExpenseEnrichedPart, ExpenseListTarget};

/// Read-only `expense list` use case result (CLI / future GUI).
pub struct ExpenseListServiceResult {
    pub target: ExpenseListTarget,
    pub expenses: Vec<ExpenseEnrichedPart>,
}

/// Resolves the list target and loads enriched expenses without terminal I/O.
pub fn list_expenses(
    conn: &Connection,
    trip: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ExpenseListServiceResult> {
    let target = crate::expense::resolve_expense_list_target(trip, itinerary)?;
    let raw_expenses = match target {
        ExpenseListTarget::Trip(trip_id) => crate::expense::list_expenses_for_trip(conn, trip_id)?,
        ExpenseListTarget::Itinerary(itinerary_id) => {
            crate::expense::list_expenses_for_itinerary(conn, itinerary_id)?
        }
    };
    let expenses = raw_expenses
        .iter()
        .map(|e| crate::expense::enrich_expense(conn, e))
        .collect::<Result<Vec<_>>>()?;
    Ok(ExpenseListServiceResult { target, expenses })
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

    fn add_sample_expense(conn: &Connection, itinerary_id: i64, title: &str) -> i64 {
        crate::expense::add_expense(
            conn,
            itinerary_id,
            "1000",
            "JPY",
            Some(title),
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap()
    }

    #[test]
    fn service_returns_expenses_for_itinerary_target() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);
        add_sample_expense(&conn, itinerary_id, "Lunch");

        let result = list_expenses(&conn, None, Some(itinerary_id)).unwrap();
        assert_eq!(result.target, ExpenseListTarget::Itinerary(itinerary_id));
        assert_eq!(result.expenses.len(), 1);
        assert_eq!(result.expenses[0].expense.title.as_deref(), Some("Lunch"));
    }

    #[test]
    fn service_returns_expenses_for_trip_target() {
        let conn = test_db();
        let (trip_id, itinerary_id) = seed_trip_with_itinerary(&conn);
        add_sample_expense(&conn, itinerary_id, "Lunch");

        let result = list_expenses(&conn, Some(trip_id), None).unwrap();
        assert_eq!(result.target, ExpenseListTarget::Trip(trip_id));
        assert_eq!(result.expenses.len(), 1);
    }

    #[test]
    fn service_returns_empty_list_for_target_without_expenses() {
        let conn = test_db();
        let (trip_id, itinerary_id) = seed_trip_with_itinerary(&conn);

        assert!(list_expenses(&conn, Some(trip_id), None)
            .unwrap()
            .expenses
            .is_empty());
        assert!(list_expenses(&conn, None, Some(itinerary_id))
            .unwrap()
            .expenses
            .is_empty());
    }

    #[test]
    fn service_preserves_ordering() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);
        let first = add_sample_expense(&conn, itinerary_id, "First");
        let second = add_sample_expense(&conn, itinerary_id, "Second");

        let result = list_expenses(&conn, None, Some(itinerary_id)).unwrap();
        assert_eq!(result.expenses.len(), 2);
        assert_eq!(result.expenses[0].expense.id, first);
        assert_eq!(result.expenses[1].expense.id, second);
    }

    #[test]
    fn service_resolves_paid_by_participant_name_in_list() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Shared Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let payer_id =
            crate::participant::create_participant(&conn, trip_id, "Alice", None, true).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Dinner", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::expense::add_expense(
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

        let result = list_expenses(&conn, None, Some(itinerary_id)).unwrap();
        assert_eq!(
            result.expenses[0].paid_by_participant_name.as_deref(),
            Some("Alice")
        );
    }

    #[test]
    fn service_preserves_itinerary_not_found_error_message() {
        let conn = test_db();
        let err = list_expenses(&conn, None, Some(9999))
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Itinerary not found: 9999");
    }

    #[test]
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = list_expenses(&conn, Some(9999), None)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn service_preserves_target_resolution_error() {
        let conn = test_db();
        let err = list_expenses(&conn, None, None)
            .err()
            .expect("expected error");
        assert!(err
            .to_string()
            .contains("--trip または --itinerary のいずれかを指定してください"));
    }

    #[test]
    fn service_does_not_change_trip_stats() {
        let conn = test_db();
        let (trip_id, itinerary_id) = seed_trip_with_itinerary(&conn);
        add_sample_expense(&conn, itinerary_id, "Lunch");

        let stats_before = crate::services::trip_stats::get_trip_stats(&conn, trip_id).unwrap();
        let _ = list_expenses(&conn, Some(trip_id), None).unwrap();
        let stats_after = crate::services::trip_stats::get_trip_stats(&conn, trip_id).unwrap();

        assert_eq!(
            stats_before.stats.expense_count,
            stats_after.stats.expense_count
        );
        assert_eq!(
            stats_before.stats.expense_totals,
            stats_after.stats.expense_totals
        );
    }
}
