use anyhow::Result;
use rusqlite::Connection;

use crate::expense::ExpenseEnrichedPart;

/// CLI mirror of `ExpenseAction::Update` fields (not a wire DTO).
pub struct ExpenseUpdateParams {
    pub id: i64,
    pub title: Option<String>,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub paid_by_name: Option<String>,
    pub paid_by_participant: Option<String>,
    pub beneficiary: Vec<String>,
    pub shared_with: Option<String>,
    pub clear_paid_by: bool,
    pub clear_beneficiaries: bool,
    pub expense_date: Option<String>,
    pub note: Option<String>,
}

/// Write `expense update` use case result (CLI / future GUI).
pub struct ExpenseUpdateServiceResult {
    pub expense: ExpenseEnrichedPart,
}

/// Updates an expense with enriched output context, without terminal I/O.
pub fn update_expense(
    conn: &Connection,
    params: ExpenseUpdateParams,
) -> Result<ExpenseUpdateServiceResult> {
    let existing = crate::expense::get_expense(conn, params.id)?;
    let shared = crate::expense::parse_expense_shared_options_for_update(
        conn,
        existing.itinerary_id,
        params.paid_by_participant.as_deref(),
        &params.beneficiary,
        params.shared_with.as_deref(),
        params.clear_paid_by,
        params.clear_beneficiaries,
    )?;
    crate::expense::update_expense(
        conn,
        params.id,
        params.title.as_deref(),
        params.amount.as_deref(),
        params.currency.as_deref(),
        params.paid_by_name.as_deref(),
        params.expense_date.as_deref(),
        params.note.as_deref(),
        &shared,
    )?;
    let expense = crate::expense::get_expense(conn, params.id)?;
    let enriched = crate::expense::enrich_expense(conn, &expense)?;
    Ok(ExpenseUpdateServiceResult { expense: enriched })
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

    fn seed_expense(conn: &Connection) -> (i64, i64) {
        let trip_id =
            crate::trip::add_trip(conn, "Expense Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap();
        let id = crate::expense::add_expense(
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
        .unwrap();
        (id, itinerary_id)
    }

    #[test]
    fn service_update_returns_enriched_expense() {
        let conn = test_db();
        let (id, _) = seed_expense(&conn);

        let result = update_expense(
            &conn,
            ExpenseUpdateParams {
                id,
                title: Some("Updated Lunch".to_string()),
                amount: None,
                currency: None,
                paid_by_name: None,
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                clear_paid_by: false,
                clear_beneficiaries: false,
                expense_date: None,
                note: None,
            },
        )
        .unwrap();

        assert_eq!(
            result.expense.expense.title.as_deref(),
            Some("Updated Lunch")
        );
    }

    #[test]
    fn service_update_clear_beneficiaries() {
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
                beneficiary_participant_ids: Some(vec![beneficiary_id]),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();

        let result = update_expense(
            &conn,
            ExpenseUpdateParams {
                id,
                title: None,
                amount: None,
                currency: None,
                paid_by_name: None,
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                clear_paid_by: false,
                clear_beneficiaries: true,
                expense_date: None,
                note: None,
            },
        )
        .unwrap();

        assert!(!result.expense.shared);
        assert!(result.expense.beneficiaries.is_empty());
    }

    #[test]
    fn service_update_not_found() {
        let conn = test_db();
        let err = update_expense(
            &conn,
            ExpenseUpdateParams {
                id: 9999,
                title: Some("X".to_string()),
                amount: None,
                currency: None,
                paid_by_name: None,
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                clear_paid_by: false,
                clear_beneficiaries: false,
                expense_date: None,
                note: None,
            },
        )
        .err()
        .expect("expected error");
        assert_eq!(err.to_string(), "Expense not found: 9999");
    }

    #[test]
    fn service_update_matches_show_after_update() {
        let conn = test_db();
        let (id, _) = seed_expense(&conn);

        let update_result = update_expense(
            &conn,
            ExpenseUpdateParams {
                id,
                title: Some("Updated".to_string()),
                amount: Some("2500".to_string()),
                currency: None,
                paid_by_name: None,
                paid_by_participant: None,
                beneficiary: vec![],
                shared_with: None,
                clear_paid_by: false,
                clear_beneficiaries: false,
                expense_date: None,
                note: None,
            },
        )
        .unwrap();

        let show_result = crate::services::expense_show::show_expense(&conn, id).unwrap();
        assert_eq!(update_result.expense, show_result.expense);
    }
}
