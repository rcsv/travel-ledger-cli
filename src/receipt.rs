use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::domain::models::{
    ExportReceiptDayRef, ExportReceiptExpenseRef, ExportReceiptItineraryRef, ExportReceiptV7,
    ItineraryItem, Receipt,
};
use crate::money::{format_amount_display, parse_amount_for_currency, validate_currency_code};

pub(crate) const RECEIPT_STATUS_UNREVIEWED: &str = "unreviewed";
pub(crate) const RECEIPT_STATUS_LINKED: &str = "linked";
pub(crate) const RECEIPT_STATUS_CONVERTED: &str = "converted";
pub(crate) const RECEIPT_STATUS_IGNORED: &str = "ignored";

const RECEIPT_SELECT_SQL: &str = "
    SELECT id, trip_id, day_id, itinerary_id, linked_expense_id,
           amount, currency, occurred_date, memo, status, created_at, updated_at
    FROM receipts";

pub(crate) fn migrate_receipts(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS receipts (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id             INTEGER NOT NULL,
            day_id              INTEGER NULL,
            itinerary_id        INTEGER NULL,
            linked_expense_id   INTEGER NULL,
            amount              INTEGER NULL,
            currency            TEXT NULL,
            occurred_date       TEXT NULL,
            memo                TEXT NULL,
            status              TEXT NOT NULL,
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL
        )",
        [],
    )
    .context("receipts テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_receipts_trip ON receipts(trip_id)",
        [],
    )
    .context("idx_receipts_trip の作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_receipts_day ON receipts(day_id)",
        [],
    )
    .context("idx_receipts_day の作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_receipts_itinerary ON receipts(itinerary_id)",
        [],
    )
    .context("idx_receipts_itinerary の作成に失敗しました")?;
    Ok(())
}

pub(crate) fn validate_receipt_status(status: &str) -> Result<()> {
    match status {
        RECEIPT_STATUS_UNREVIEWED
        | RECEIPT_STATUS_LINKED
        | RECEIPT_STATUS_CONVERTED
        | RECEIPT_STATUS_IGNORED => Ok(()),
        _ => anyhow::bail!("invalid receipt status: {status}"),
    }
}

fn row_to_receipt(row: &rusqlite::Row) -> rusqlite::Result<Receipt> {
    Ok(Receipt {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        day_id: row.get(2)?,
        itinerary_id: row.get(3)?,
        linked_expense_id: row.get(4)?,
        amount: row.get(5)?,
        currency: row.get(6)?,
        occurred_date: row.get(7)?,
        memo: row.get(8)?,
        status: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn normalize_optional_memo(memo: Option<&str>) -> Result<Option<String>> {
    match memo {
        None => Ok(None),
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
    }
}

pub(crate) fn validate_receipt_amount_currency_pair(
    amount: Option<i64>,
    currency: &Option<String>,
) -> Result<()> {
    match (amount, currency.as_deref()) {
        (Some(_), None) => anyhow::bail!("currency is required when amount is set"),
        (None, Some(c)) if !c.trim().is_empty() => {
            anyhow::bail!("amount is required when currency is set")
        }
        _ => Ok(()),
    }
}

fn validate_receipt_has_content(memo: &Option<String>, amount: Option<i64>) -> Result<()> {
    if memo.is_some() || amount.is_some() {
        Ok(())
    } else {
        anyhow::bail!("receipt requires memo and/or amount with currency")
    }
}

pub(crate) fn ensure_itinerary_belongs_to_trip(
    conn: &Connection,
    trip_id: i64,
    itinerary_id: i64,
) -> Result<ItineraryItem> {
    let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    if item.trip_id != trip_id {
        anyhow::bail!("itinerary does not belong to this trip");
    }
    Ok(item)
}

fn day_id_for_itinerary_item(conn: &Connection, item: &ItineraryItem) -> Result<i64> {
    crate::day::find_day_id_by_trip_and_day_number(conn, item.trip_id, item.day)
}

pub(crate) struct AddReceiptParams<'a> {
    pub trip_id: i64,
    pub day_number: Option<i64>,
    pub itinerary_id: Option<i64>,
    pub amount_input: Option<&'a str>,
    pub currency_input: Option<&'a str>,
    pub occurred_date: Option<&'a str>,
    pub memo: Option<&'a str>,
}

pub(crate) fn add_receipt(conn: &Connection, params: AddReceiptParams<'_>) -> Result<i64> {
    crate::trip::get_trip(conn, params.trip_id)?;

    let memo = normalize_optional_memo(params.memo)?;
    let (amount, currency) = match (params.amount_input, params.currency_input) {
        (Some(amount_str), Some(currency_str)) => {
            let currency = validate_currency_code(currency_str)?;
            let amount = parse_amount_for_currency(amount_str, &currency)?;
            (Some(amount), Some(currency))
        }
        (None, None) => (None, None),
        _ => anyhow::bail!("amount and currency must be provided together"),
    };
    validate_receipt_amount_currency_pair(amount, &currency)?;
    validate_receipt_has_content(&memo, amount)?;

    if let Some(date) = params.occurred_date {
        crate::expense::validate_expense_date(date)?;
    }

    let mut day_id = None;
    if let Some(day_number) = params.day_number {
        day_id = Some(crate::day::find_day_id_by_trip_and_day_number(
            conn,
            params.trip_id,
            day_number,
        )?);
    }

    let itinerary_id = params.itinerary_id;
    if let Some(it_id) = itinerary_id {
        let item = ensure_itinerary_belongs_to_trip(conn, params.trip_id, it_id)?;
        if day_id.is_none() {
            day_id = Some(day_id_for_itinerary_item(conn, &item)?);
        }
    }

    let status = RECEIPT_STATUS_UNREVIEWED;
    let now = crate::storage::db::now_string();
    conn.execute(
        "INSERT INTO receipts
         (trip_id, day_id, itinerary_id, linked_expense_id, amount, currency,
          occurred_date, memo, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            params.trip_id,
            day_id,
            itinerary_id,
            amount,
            currency,
            params.occurred_date,
            memo,
            status,
            &now,
            &now,
        ],
    )
    .context("Receipt の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_receipts_for_trip(
    conn: &Connection,
    trip_id: i64,
    status_filter: Option<&str>,
) -> Result<Vec<Receipt>> {
    crate::trip::get_trip(conn, trip_id)?;
    if let Some(status) = status_filter {
        validate_receipt_status(status)?;
    }

    let sql = if status_filter.is_some() {
        format!(
            "{RECEIPT_SELECT_SQL}
             WHERE trip_id = ?1 AND status = ?2
             ORDER BY id ASC"
        )
    } else {
        format!(
            "{RECEIPT_SELECT_SQL}
             WHERE trip_id = ?1
             ORDER BY id ASC"
        )
    };

    let mut stmt = conn
        .prepare(&sql)
        .context("Receipt 一覧取得の準備に失敗しました")?;

    let rows = if let Some(status) = status_filter {
        stmt.query_map(params![trip_id, status], row_to_receipt)
    } else {
        stmt.query_map(params![trip_id], row_to_receipt)
    }
    .context("Receipt 一覧取得に失敗しました")?
    .collect::<std::result::Result<Vec<_>, _>>()
    .context("Receipt 一覧の読み込みに失敗しました")?;
    Ok(rows)
}

pub(crate) fn get_receipt(conn: &Connection, id: i64) -> Result<Receipt> {
    crate::storage::db::map_query_row(
        conn.query_row(
            &format!("{RECEIPT_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_receipt,
        ),
        || anyhow::anyhow!("receipt not found: {id}"),
    )
}

pub(crate) struct UpdateReceiptParams<'a> {
    pub day_number: Option<i64>,
    pub itinerary_id: Option<i64>,
    pub amount_input: Option<&'a str>,
    pub currency_input: Option<&'a str>,
    pub occurred_date: Option<Option<&'a str>>,
    pub memo: Option<Option<&'a str>>,
    pub clear_day: bool,
    pub clear_itinerary: bool,
    pub clear_amount_currency: bool,
}

pub(crate) fn update_receipt(
    conn: &Connection,
    id: i64,
    params: UpdateReceiptParams<'_>,
) -> Result<()> {
    let existing = get_receipt(conn, id)?;
    let mut day_id = existing.day_id;
    let mut itinerary_id = existing.itinerary_id;
    let mut amount = existing.amount;
    let mut currency = existing.currency;
    let mut occurred_date = existing.occurred_date.clone();
    let mut memo = existing.memo.clone();

    if params.clear_day {
        day_id = None;
    }
    if params.clear_itinerary {
        itinerary_id = None;
    }
    if let Some(day_number) = params.day_number {
        day_id = Some(crate::day::find_day_id_by_trip_and_day_number(
            conn,
            existing.trip_id,
            day_number,
        )?);
    }
    if let Some(it_id) = params.itinerary_id {
        let item = ensure_itinerary_belongs_to_trip(conn, existing.trip_id, it_id)?;
        itinerary_id = Some(it_id);
        if params.day_number.is_none() && !params.clear_day {
            day_id = Some(day_id_for_itinerary_item(conn, &item)?);
        }
    }

    if params.clear_amount_currency {
        amount = None;
        currency = None;
    } else if params.amount_input.is_some() || params.currency_input.is_some() {
        let amount_str = params
            .amount_input
            .ok_or_else(|| anyhow::anyhow!("amount is required when updating currency"))?;
        let currency_str = params
            .currency_input
            .ok_or_else(|| anyhow::anyhow!("currency is required when updating amount"))?;
        let parsed_currency = validate_currency_code(currency_str)?;
        amount = Some(parse_amount_for_currency(amount_str, &parsed_currency)?);
        currency = Some(parsed_currency);
    }

    if let Some(date_opt) = params.occurred_date {
        match date_opt {
            Some(date) => {
                crate::expense::validate_expense_date(date)?;
                occurred_date = Some(date.to_string());
            }
            None => occurred_date = None,
        }
    }

    if let Some(memo_opt) = params.memo {
        memo = normalize_optional_memo(memo_opt)?;
    }

    validate_receipt_amount_currency_pair(amount, &currency)?;
    validate_receipt_has_content(&memo, amount)?;

    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET day_id = ?1, itinerary_id = ?2, amount = ?3, currency = ?4,
         occurred_date = ?5, memo = ?6, updated_at = ?7
         WHERE id = ?8",
        params![
            day_id,
            itinerary_id,
            amount,
            currency,
            occurred_date,
            memo,
            &now,
            id,
        ],
    )
    .context("Receipt の更新に失敗しました")?;
    Ok(())
}

pub(crate) fn link_receipt_day(conn: &Connection, id: i64, day_number: i64) -> Result<()> {
    let receipt = get_receipt(conn, id)?;
    let day_id = crate::day::find_day_id_by_trip_and_day_number(conn, receipt.trip_id, day_number)?;
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET day_id = ?1, status = ?2, updated_at = ?3 WHERE id = ?4",
        params![day_id, RECEIPT_STATUS_LINKED, &now, id],
    )
    .context("Receipt の紐づけに失敗しました")?;
    Ok(())
}

pub(crate) fn link_receipt_itinerary(conn: &Connection, id: i64, itinerary_id: i64) -> Result<()> {
    let receipt = get_receipt(conn, id)?;
    let item = ensure_itinerary_belongs_to_trip(conn, receipt.trip_id, itinerary_id)?;
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET day_id = ?1, itinerary_id = ?2, status = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            day_id_for_itinerary_item(conn, &item)?,
            itinerary_id,
            RECEIPT_STATUS_LINKED,
            &now,
            id,
        ],
    )
    .context("Receipt の紐づけに失敗しました")?;
    Ok(())
}

pub(crate) fn ignore_receipt(conn: &Connection, id: i64, memo: Option<&str>) -> Result<()> {
    let existing = get_receipt(conn, id)?;
    let new_memo = if let Some(text) = memo {
        normalize_optional_memo(Some(text))?
    } else {
        existing.memo
    };
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET status = ?1, memo = ?2, updated_at = ?3 WHERE id = ?4",
        params![RECEIPT_STATUS_IGNORED, new_memo, &now, id],
    )
    .context("Receipt の ignore に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_receipt(conn: &Connection, id: i64) -> Result<()> {
    get_receipt(conn, id)?;
    conn.execute("DELETE FROM receipts WHERE id = ?1", params![id])
        .context("Receipt の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_receipts_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute("DELETE FROM receipts WHERE trip_id = ?1", params![trip_id])
        .context("Receipt の Trip 削除に失敗しました")?;
    Ok(())
}

pub(crate) fn nullify_receipts_for_day(conn: &Connection, day_id: i64) -> Result<()> {
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET day_id = NULL, updated_at = ?1 WHERE day_id = ?2",
        params![&now, day_id],
    )
    .context("Receipt day_id のクリアに失敗しました")?;
    Ok(())
}

pub(crate) fn nullify_receipts_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<()> {
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET itinerary_id = NULL, updated_at = ?1 WHERE itinerary_id = ?2",
        params![&now, itinerary_id],
    )
    .context("Receipt itinerary_id のクリアに失敗しました")?;
    Ok(())
}

pub(crate) fn nullify_receipts_for_expense(conn: &Connection, expense_id: i64) -> Result<()> {
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET linked_expense_id = NULL, updated_at = ?1
         WHERE linked_expense_id = ?2",
        params![&now, expense_id],
    )
    .context("Receipt linked_expense_id のクリアに失敗しました")?;
    Ok(())
}

fn day_number_for_receipt(conn: &Connection, receipt: &Receipt) -> Result<Option<i64>> {
    if let Some(day_id) = receipt.day_id {
        let day_number: i64 = conn.query_row(
            "SELECT day_number FROM days WHERE id = ?1",
            params![day_id],
            |row| row.get(0),
        )?;
        return Ok(Some(day_number));
    }
    if let Some(itinerary_id) = receipt.itinerary_id {
        let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
        return Ok(Some(item.day));
    }
    Ok(None)
}

pub(crate) fn build_export_receipt_v7(
    conn: &Connection,
    receipt: &Receipt,
) -> Result<ExportReceiptV7> {
    validate_receipt_status(&receipt.status)?;

    let day_ref = if let Some(day_id) = receipt.day_id {
        let day_number: i64 = conn.query_row(
            "SELECT day_number FROM days WHERE id = ?1",
            params![day_id],
            |row| row.get(0),
        )?;
        Some(ExportReceiptDayRef { day_number })
    } else {
        None
    };

    let itinerary_ref = if let Some(itinerary_id) = receipt.itinerary_id {
        let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
        Some(ExportReceiptItineraryRef {
            day_number: item.day,
            sort_order: item.sort_order,
            start_time: item.start_time.clone(),
            title: item.title.clone(),
        })
    } else {
        None
    };

    let linked_expense_ref = if let Some(expense_id) = receipt.linked_expense_id {
        let expense = crate::expense::get_expense(conn, expense_id)?;
        let item = crate::itinerary::get_itinerary_item(conn, expense.itinerary_id)?;
        Some(ExportReceiptExpenseRef {
            itinerary_ref: ExportReceiptItineraryRef {
                day_number: item.day,
                sort_order: item.sort_order,
                start_time: item.start_time.clone(),
                title: item.title.clone(),
            },
            expense_sort_order: expense.sort_order,
            expense_title: expense.title.clone(),
            amount: expense.amount,
            currency: expense.currency.clone(),
        })
    } else {
        None
    };

    Ok(ExportReceiptV7 {
        day_ref,
        itinerary_ref,
        linked_expense_ref,
        amount: receipt.amount,
        currency: receipt.currency.clone(),
        occurred_date: receipt.occurred_date.clone(),
        memo: receipt.memo.clone(),
        status: receipt.status.clone(),
    })
}

pub(crate) fn build_export_receipts_for_trip(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ExportReceiptV7>> {
    list_receipts_for_trip(conn, trip_id, None)?
        .iter()
        .map(|r| build_export_receipt_v7(conn, r))
        .collect()
}

fn resolve_itinerary_id_from_ref(
    conn: &Connection,
    trip_id: i64,
    itinerary_ref: &ExportReceiptItineraryRef,
) -> Result<i64> {
    let items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let export_items: Vec<ItineraryItem> = items
        .into_iter()
        .map(|mut item| {
            item.trip_id = trip_id;
            item
        })
        .collect();
    let key = crate::domain::models::ItineraryNoteKey {
        day_number: itinerary_ref.day_number,
        sort_order: itinerary_ref.sort_order,
        start_time: itinerary_ref.start_time.clone(),
        title: itinerary_ref.title.clone(),
    };
    crate::note::resolve_itinerary_id_from_export_items(&export_items, &key)
}

fn resolve_expense_id_from_ref(
    conn: &Connection,
    trip_id: i64,
    expense_ref: &ExportReceiptExpenseRef,
) -> Result<i64> {
    let itinerary_id = resolve_itinerary_id_from_ref(conn, trip_id, &expense_ref.itinerary_ref)?;
    let expenses = crate::expense::list_expenses_for_itinerary(conn, itinerary_id)?;
    let mut matches: Vec<i64> = expenses
        .iter()
        .filter(|e| {
            e.sort_order == expense_ref.expense_sort_order
                && e.amount == expense_ref.amount
                && e.currency == expense_ref.currency
                && e.title == expense_ref.expense_title
        })
        .map(|e| e.id)
        .collect();
    if matches.len() == 1 {
        Ok(matches.remove(0))
    } else if matches.is_empty() {
        anyhow::bail!("linked expense not found in export ref")
    } else {
        anyhow::bail!("linked expense ref is ambiguous")
    }
}

pub(crate) fn import_receipt_v7(
    conn: &Connection,
    trip_id: i64,
    export: &ExportReceiptV7,
) -> Result<i64> {
    validate_receipt_status(&export.status)?;
    validate_receipt_amount_currency_pair(export.amount, &export.currency)?;
    if let Some(date) = export.occurred_date.as_deref() {
        crate::expense::validate_expense_date(date)?;
    }
    let memo = export.memo.clone();
    validate_receipt_has_content(&memo, export.amount)?;

    let mut day_id = None;
    if let Some(day_ref) = &export.day_ref {
        day_id = Some(crate::day::find_day_id_by_trip_and_day_number(
            conn,
            trip_id,
            day_ref.day_number,
        )?);
    }

    let mut itinerary_id = None;
    if let Some(itinerary_ref) = &export.itinerary_ref {
        itinerary_id = Some(resolve_itinerary_id_from_ref(conn, trip_id, itinerary_ref)?);
        if day_id.is_none() {
            let item = crate::itinerary::get_itinerary_item(conn, itinerary_id.unwrap())?;
            day_id = Some(day_id_for_itinerary_item(conn, &item)?);
        }
    }

    let linked_expense_id = if let Some(expense_ref) = &export.linked_expense_ref {
        Some(resolve_expense_id_from_ref(conn, trip_id, expense_ref)?)
    } else {
        None
    };

    let now = crate::storage::db::now_string();
    conn.execute(
        "INSERT INTO receipts
         (trip_id, day_id, itinerary_id, linked_expense_id, amount, currency,
          occurred_date, memo, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            trip_id,
            day_id,
            itinerary_id,
            linked_expense_id,
            export.amount,
            export.currency,
            export.occurred_date,
            memo,
            export.status,
            &now,
            &now,
        ],
    )
    .context("Receipt import に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn collect_export_receipt_validation_errors(
    receipts: &[ExportReceiptV7],
    effective_schema: i32,
) -> Vec<String> {
    use crate::domain::models::TRIP_EXPORT_SCHEMA_VERSION;

    if effective_schema < TRIP_EXPORT_SCHEMA_VERSION {
        return Vec::new();
    }

    let mut errors = Vec::new();
    for (index, receipt) in receipts.iter().enumerate() {
        let prefix = format!("receipts[{index}]");
        if let Err(error) = validate_receipt_status(&receipt.status) {
            errors.push(format!("{prefix}.status: {error}"));
        }
        if let Err(error) = validate_receipt_amount_currency_pair(receipt.amount, &receipt.currency)
        {
            errors.push(format!("{prefix}: {error}"));
        }
        if receipt.amount.is_none() && receipt.memo.is_none() {
            errors.push(format!("{prefix}: memo and/or amount required"));
        }
        if let Some(currency) = receipt.currency.as_deref() {
            if let Err(error) = validate_currency_code(currency) {
                errors.push(format!("{prefix}.currency: {error}"));
            }
        }
        if let Some(date) = receipt.occurred_date.as_deref() {
            if let Err(error) = crate::expense::validate_expense_date(date) {
                errors.push(format!("{prefix}.occurred_date: {error}"));
            }
        }
    }
    errors
}

#[derive(Serialize)]
pub(crate) struct ReceiptJson {
    pub id: i64,
    pub trip_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_expense_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

pub(crate) fn receipt_to_json(conn: &Connection, receipt: &Receipt) -> Result<ReceiptJson> {
    Ok(ReceiptJson {
        id: receipt.id,
        trip_id: receipt.trip_id,
        day_id: receipt.day_id,
        day_number: day_number_for_receipt(conn, receipt)?,
        itinerary_id: receipt.itinerary_id,
        linked_expense_id: receipt.linked_expense_id,
        amount: receipt.amount,
        currency: receipt.currency.clone(),
        occurred_date: receipt.occurred_date.clone(),
        memo: receipt.memo.clone(),
        status: receipt.status.clone(),
        created_at: receipt.created_at.clone(),
        updated_at: receipt.updated_at.clone(),
    })
}

#[derive(Serialize)]
pub(crate) struct ReceiptListJson {
    pub trip_id: i64,
    pub receipts: Vec<ReceiptJson>,
}

fn format_amount_optional(amount: Option<i64>, currency: &Option<String>) -> String {
    match (amount, currency.as_deref()) {
        (Some(value), Some(cur)) => format_amount_display(value, cur),
        _ => "-".to_string(),
    }
}

pub(crate) fn print_receipt_list(conn: &Connection, receipts: &[Receipt]) -> Result<()> {
    if receipts.is_empty() {
        println!("Receipt はまだ登録されていません。");
        return Ok(());
    }
    println!(
        "{:<4} {:<10} {:<6} {:<6} {:<16} {:<12} Memo",
        "ID", "Status", "Day", "Itin", "Amount", "Date"
    );
    println!("{}", "-".repeat(72));
    for receipt in receipts {
        let day_number = day_number_for_receipt(conn, receipt)?
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        let itinerary = receipt
            .itinerary_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string());
        let amount = format_amount_optional(receipt.amount, &receipt.currency);
        let date = receipt.occurred_date.as_deref().unwrap_or("-");
        let memo = receipt.memo.as_deref().unwrap_or("-");
        println!(
            "{:<4} {:<10} {:<6} {:<6} {:<16} {:<12} {}",
            receipt.id, receipt.status, day_number, itinerary, amount, date, memo,
        );
    }
    println!();
    println!("合計: {} 件", receipts.len());
    Ok(())
}

pub(crate) fn print_receipt_detail(conn: &Connection, receipt: &Receipt) -> Result<()> {
    let day_number = day_number_for_receipt(conn, receipt)?;
    println!("ID              : {}", receipt.id);
    println!("Trip ID         : {}", receipt.trip_id);
    println!(
        "Day             : {}",
        day_number
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Itinerary ID    : {}",
        receipt
            .itinerary_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Linked Expense  : {}",
        receipt
            .linked_expense_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Amount          : {}",
        format_amount_optional(receipt.amount, &receipt.currency)
    );
    println!(
        "Currency        : {}",
        receipt.currency.as_deref().unwrap_or("-")
    );
    println!(
        "Occurred date   : {}",
        receipt.occurred_date.as_deref().unwrap_or("-")
    );
    println!(
        "Memo            : {}",
        receipt.memo.as_deref().unwrap_or("-")
    );
    println!("Status          : {}", receipt.status);
    println!("Created at      : {}", receipt.created_at);
    println!("Updated at      : {}", receipt.updated_at);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::{init_db, open_db_at};
    use std::path::PathBuf;

    fn memory_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    fn temp_conn() -> (Connection, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "caglla-receipt-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("caglla.db");
        let conn = open_db_at(path.to_str().unwrap()).unwrap();
        (conn, dir)
    }

    fn setup_trip(conn: &Connection) -> i64 {
        crate::trip::add_trip(conn, "Receipt Trip", "2026-04-26", "2026-04-29", None).unwrap()
    }

    #[test]
    fn test_add_list_show_update_delete_receipt() {
        let conn = memory_conn();
        let trip_id = setup_trip(&conn);
        let id = add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: Some(1),
                itinerary_id: None,
                amount_input: Some("1700"),
                currency_input: Some("JPY"),
                occurred_date: Some("2026-04-26"),
                memo: Some("これなんだっけ？"),
            },
        )
        .unwrap();
        let receipts = list_receipts_for_trip(&conn, trip_id, None).unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].status, RECEIPT_STATUS_UNREVIEWED);

        update_receipt(
            &conn,
            id,
            UpdateReceiptParams {
                day_number: None,
                itinerary_id: None,
                amount_input: None,
                currency_input: None,
                occurred_date: None,
                memo: Some(Some("おかんのお土産っぽい")),
                clear_day: false,
                clear_itinerary: false,
                clear_amount_currency: false,
            },
        )
        .unwrap();
        let updated = get_receipt(&conn, id).unwrap();
        assert_eq!(updated.memo.as_deref(), Some("おかんのお土産っぽい"));

        link_receipt_day(&conn, id, 1).unwrap();
        let linked = get_receipt(&conn, id).unwrap();
        assert_eq!(linked.status, RECEIPT_STATUS_LINKED);

        ignore_receipt(&conn, id, Some("旅行費用ではない")).unwrap();
        let ignored = get_receipt(&conn, id).unwrap();
        assert_eq!(ignored.status, RECEIPT_STATUS_IGNORED);
        assert_eq!(ignored.amount, Some(1700));

        delete_receipt(&conn, id).unwrap();
        assert!(get_receipt(&conn, id).is_err());
    }

    #[test]
    fn test_receipt_amount_currency_pair_validation() {
        let conn = memory_conn();
        let trip_id = setup_trip(&conn);
        assert!(add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: None,
                itinerary_id: None,
                amount_input: Some("100"),
                currency_input: None,
                occurred_date: None,
                memo: None,
            },
        )
        .is_err());
        assert!(add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: None,
                itinerary_id: None,
                amount_input: None,
                currency_input: Some("JPY"),
                occurred_date: None,
                memo: None,
            },
        )
        .is_err());
    }

    #[test]
    fn test_export_import_receipt_roundtrip() {
        let (conn, _dir) = temp_conn();
        let trip_id = setup_trip(&conn);
        let it_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Shop", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: None,
                itinerary_id: Some(it_id),
                amount_input: Some("1700"),
                currency_input: Some("JPY"),
                occurred_date: None,
                memo: Some("memo"),
            },
        )
        .unwrap();

        let exports = build_export_receipts_for_trip(&conn, trip_id).unwrap();
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].status, RECEIPT_STATUS_UNREVIEWED);
        assert!(exports[0].itinerary_ref.is_some());

        crate::storage::db::reset_db(&conn).unwrap();
        let new_trip = setup_trip(&conn);
        crate::itinerary::add_itinerary_item(
            &conn, new_trip, 1, "Shop", None, None, None, None, None, None, None,
        )
        .unwrap();
        import_receipt_v7(&conn, new_trip, &exports[0]).unwrap();
        let imported = list_receipts_for_trip(&conn, new_trip, None).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].memo.as_deref(), Some("memo"));
    }

    #[test]
    fn test_nullify_on_itinerary_delete() {
        let conn = memory_conn();
        let trip_id = setup_trip(&conn);
        let it_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Shop", None, None, None, None, None, None, None,
        )
        .unwrap();
        let receipt_id = add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: None,
                itinerary_id: Some(it_id),
                amount_input: None,
                currency_input: None,
                occurred_date: None,
                memo: Some("keep"),
            },
        )
        .unwrap();
        nullify_receipts_for_itinerary(&conn, it_id).unwrap();
        crate::itinerary::delete_itinerary_item(&conn, it_id).unwrap();
        let receipt = get_receipt(&conn, receipt_id).unwrap();
        assert!(receipt.itinerary_id.is_none());
        assert_eq!(receipt.memo.as_deref(), Some("keep"));
    }

    #[test]
    fn test_format_amount_optional_negative_decimal_currency_display() {
        let display = format_amount_optional(Some(-50), &Some("USD".to_string()));
        assert_eq!(display, "-0.50 USD");
        assert!(!display.contains("-0.500"));
    }
}
