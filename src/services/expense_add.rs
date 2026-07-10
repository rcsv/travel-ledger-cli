use anyhow::Result;
use rusqlite::Connection;

use crate::expense::ExpenseEnrichedPart;

/// CLI mirror of `ExpenseAction::Add` fields (not a wire DTO).
pub struct ExpenseAddParams {
    pub itinerary: i64,
    pub amount: String,
    pub currency: String,
    pub title: Option<String>,
    pub note: Option<String>,
    pub paid_by_name: Option<String>,
    pub paid_by_participant: Option<String>,
    pub beneficiary: Vec<String>,
    pub shared_with: Option<String>,
    pub expense_date: Option<String>,
}

/// Write `expense add` use case result (CLI / future GUI).
pub struct ExpenseAddServiceResult {
    pub id: i64,
    pub expense: ExpenseEnrichedPart,
}

/// Adds an expense with enriched output context, without terminal I/O.
pub fn add_expense(conn: &Connection, params: ExpenseAddParams) -> Result<ExpenseAddServiceResult> {
    let shared = crate::expense::parse_expense_shared_options_for_add(
        conn,
        params.itinerary,
        params.paid_by_participant.as_deref(),
        &params.beneficiary,
        params.shared_with.as_deref(),
    )?;
    let id = crate::expense::add_expense(
        conn,
        params.itinerary,
        &params.amount,
        &params.currency,
        params.title.as_deref(),
        params.note.as_deref(),
        params.paid_by_name.as_deref(),
        params.expense_date.as_deref(),
        &shared,
    )?;
    let expense = crate::expense::get_expense(conn, id)?;
    let enriched = crate::expense::enrich_expense(conn, &expense)?;
    Ok(ExpenseAddServiceResult {
        id,
        expense: enriched,
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
            crate::trip::add_trip(conn, "Expense Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap();
        (trip_id, itinerary_id)
    }

    #[test]
    fn service_add_returns_enriched_expense() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);

        let result = add_expense(
            &conn,
            ExpenseAddParams {
                itinerary: itinerary_id,
                amount: "2200".to_string(),
                currency: "JPY".to_string(),
                title: Some("Lunch".to_string()),
                note: None,
                paid_by_name: Some("Tomo".to_string()),
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                expense_date: Some("2026-04-27".to_string()),
            },
        )
        .unwrap();

        assert_eq!(result.id, result.expense.expense.id);
        assert_eq!(result.expense.expense.itinerary_id, itinerary_id);
        assert_eq!(result.expense.expense.amount, 2200);
        assert_eq!(result.expense.expense.currency, "JPY");
        assert!(!result.expense.shared);
    }

    #[test]
    fn service_add_with_beneficiaries_sets_shared() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Shared Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let payer_id =
            crate::participant::create_participant(&conn, trip_id, "Alice", None, true).unwrap();
        let beneficiary_id =
            crate::participant::create_participant(&conn, trip_id, "Bob", None, false).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Dinner", None, None, None, None, None, None, None,
        )
        .unwrap();

        let result = add_expense(
            &conn,
            ExpenseAddParams {
                itinerary: itinerary_id,
                amount: "3000".to_string(),
                currency: "JPY".to_string(),
                title: None,
                note: None,
                paid_by_name: None,
                paid_by_participant: Some(payer_id.to_string()),
                beneficiary: vec![beneficiary_id.to_string()],
                shared_with: None,
                expense_date: None,
            },
        )
        .unwrap();

        assert!(result.expense.shared);
        assert_eq!(result.expense.beneficiaries.len(), 1);
        assert_eq!(
            result.expense.paid_by_participant_name.as_deref(),
            Some("Alice")
        );
    }

    #[test]
    fn service_add_not_found_itinerary() {
        let conn = test_db();
        let err = add_expense(
            &conn,
            ExpenseAddParams {
                itinerary: 9999,
                amount: "100".to_string(),
                currency: "JPY".to_string(),
                title: None,
                note: None,
                paid_by_name: None,
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                expense_date: None,
            },
        )
        .err()
        .expect("expected error");
        assert!(err.to_string().contains("Itinerary not found"));
    }

    #[test]
    fn service_add_matches_show_after_insert() {
        let conn = test_db();
        let (_, itinerary_id) = seed_trip_with_itinerary(&conn);

        let add_result = add_expense(
            &conn,
            ExpenseAddParams {
                itinerary: itinerary_id,
                amount: "2200".to_string(),
                currency: "JPY".to_string(),
                title: Some("Lunch".to_string()),
                note: None,
                paid_by_name: Some("Tomo".to_string()),
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                expense_date: Some("2026-04-27".to_string()),
            },
        )
        .unwrap();

        let show_result =
            crate::services::expense_show::show_expense(&conn, add_result.id).unwrap();
        assert_eq!(add_result.expense, show_result.expense);
    }
}
