use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::models::Estimate;
use crate::money::{format_amount_display, parse_amount_for_currency, validate_currency_code};

const ESTIMATE_SELECT_SQL: &str = "
    SELECT id, itinerary_id, title, amount, currency, note, sort_order, created_at, updated_at
    FROM estimates";

pub(crate) fn migrate_estimates(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS estimates (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            itinerary_id    INTEGER NOT NULL,
            title           TEXT,
            amount          INTEGER NOT NULL,
            currency        TEXT NOT NULL,
            note            TEXT,
            sort_order      INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        )",
        [],
    )
    .context("estimates テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_estimates_itinerary
         ON estimates(itinerary_id)",
        [],
    )
    .context("idx_estimates_itinerary の作成に失敗しました")?;
    Ok(())
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    match value {
        None => None,
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EstimateJson {
    pub id: i64,
    pub itinerary_id: i64,
    pub title: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub amount_display: String,
    pub note: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct EstimateListJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_id: Option<i64>,
    pub estimates: Vec<EstimateJson>,
}

pub(crate) fn resolve_estimate_list_target(
    trip: Option<i64>,
    itinerary: Option<i64>,
) -> Result<EstimateListTarget> {
    match (trip, itinerary) {
        (Some(trip_id), None) => Ok(EstimateListTarget::Trip(trip_id)),
        (None, Some(itinerary_id)) => Ok(EstimateListTarget::Itinerary(itinerary_id)),
        (Some(_), Some(_)) => {
            anyhow::bail!("--trip と --itinerary は同時に指定できません");
        }
        (None, None) => {
            anyhow::bail!("--trip または --itinerary のいずれかを指定してください");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EstimateListTarget {
    Trip(i64),
    Itinerary(i64),
}

pub(crate) fn estimate_to_json(estimate: &Estimate) -> EstimateJson {
    EstimateJson {
        id: estimate.id,
        itinerary_id: estimate.itinerary_id,
        title: estimate.title.clone(),
        amount: estimate.amount,
        currency: estimate.currency.clone(),
        amount_display: format_amount_display(estimate.amount, &estimate.currency),
        note: estimate.note.clone(),
        sort_order: estimate.sort_order,
        created_at: estimate.created_at.clone(),
        updated_at: estimate.updated_at.clone(),
    }
}

pub(crate) fn add_estimate(
    conn: &Connection,
    itinerary_id: i64,
    amount_input: &str,
    currency_input: &str,
    title: Option<&str>,
    note: Option<&str>,
    sort_order: Option<i64>,
) -> Result<i64> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    let currency = validate_currency_code(currency_input)?;
    let amount = parse_amount_for_currency(amount_input, &currency)?;
    let title = normalize_optional_text(title);
    let note = normalize_optional_text(note);
    let sort_order = sort_order.unwrap_or(0);
    let now = crate::db::now_string();

    conn.execute(
        "INSERT INTO estimates
         (itinerary_id, title, amount, currency, note, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            itinerary_id,
            title,
            amount,
            currency,
            note,
            sort_order,
            &now,
            &now,
        ],
    )
    .context("Estimate の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_estimates_for_itinerary(
    conn: &Connection,
    itinerary_id: i64,
) -> Result<Vec<Estimate>> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    list_estimates_where(
        conn,
        "itinerary_id = ?1",
        params![itinerary_id],
        "e.sort_order ASC, e.id ASC",
    )
}

pub(crate) fn list_estimates_for_trip(conn: &Connection, trip_id: i64) -> Result<Vec<Estimate>> {
    crate::trip::get_trip(conn, trip_id)?;
    list_estimates_where(
        conn,
        "e.itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
        "d.day_number ASC, i.sort_order ASC, i.id ASC, e.sort_order ASC, e.id ASC",
    )
}

fn list_estimates_where<P: rusqlite::Params>(
    conn: &Connection,
    where_clause: &str,
    params: P,
    order_by: &str,
) -> Result<Vec<Estimate>> {
    let sql = format!(
        "SELECT e.id, e.itinerary_id, e.title, e.amount, e.currency, e.note, e.sort_order,
                e.created_at, e.updated_at
         FROM estimates e
         JOIN itinerary_items i ON e.itinerary_id = i.id
         JOIN days d ON i.day_id = d.id
         WHERE {where_clause}
         ORDER BY {order_by}"
    );
    let mut stmt = conn
        .prepare(&sql)
        .context("Estimate 一覧取得の準備に失敗しました")?;
    let estimates = stmt
        .query_map(params, row_to_estimate)
        .context("Estimate 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Estimate 一覧の読み込みに失敗しました")?;
    Ok(estimates)
}

pub(crate) fn get_estimate(conn: &Connection, id: i64) -> Result<Estimate> {
    crate::db::map_query_row(
        conn.query_row(
            &format!("{ESTIMATE_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_estimate,
        ),
        || anyhow::anyhow!("Estimate not found: {id}"),
    )
}

#[derive(Debug, Default)]
pub(crate) struct UpdateEstimateParams<'a> {
    pub title: Option<&'a str>,
    pub note: Option<&'a str>,
    pub amount_input: Option<&'a str>,
    pub currency_input: Option<&'a str>,
    pub sort_order: Option<i64>,
    pub clear_title: bool,
    pub clear_note: bool,
}

pub(crate) fn update_estimate(
    conn: &Connection,
    id: i64,
    params: &UpdateEstimateParams<'_>,
) -> Result<()> {
    if params.clear_title && params.title.is_some() {
        anyhow::bail!("cannot combine --clear-title and --title");
    }
    if params.clear_note && params.note.is_some() {
        anyhow::bail!("cannot combine --clear-note and --note");
    }

    let has_field_update = params.title.is_some()
        || params.note.is_some()
        || params.amount_input.is_some()
        || params.currency_input.is_some()
        || params.sort_order.is_some()
        || params.clear_title
        || params.clear_note;

    if !has_field_update {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください (--title, --note, --amount, --currency, --sort-order, --clear-title, --clear-note)"
        );
    }

    if params.currency_input.is_some() && params.amount_input.is_none() {
        anyhow::bail!("currency を変更する場合は --amount も指定してください");
    }

    let mut estimate = get_estimate(conn, id)?;

    if params.clear_title {
        estimate.title = None;
    } else if let Some(value) = params.title {
        estimate.title = normalize_optional_text(Some(value));
    }

    if params.clear_note {
        estimate.note = None;
    } else if let Some(value) = params.note {
        estimate.note = normalize_optional_text(Some(value));
    }

    if let Some(sort_order) = params.sort_order {
        estimate.sort_order = sort_order;
    }

    let currency = match params.currency_input {
        Some(code) => validate_currency_code(code)?,
        None => estimate.currency.clone(),
    };

    if let Some(input) = params.amount_input {
        estimate.amount = parse_amount_for_currency(input, &currency)?;
    }
    if params.currency_input.is_some() {
        estimate.currency = currency;
    }

    let now = crate::db::now_string();
    conn.execute(
        "UPDATE estimates
         SET title = ?1, amount = ?2, currency = ?3, note = ?4, sort_order = ?5, updated_at = ?6
         WHERE id = ?7",
        params![
            estimate.title,
            estimate.amount,
            estimate.currency,
            estimate.note,
            estimate.sort_order,
            &now,
            id,
        ],
    )
    .context("Estimate の更新に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_estimate(conn: &Connection, id: i64) -> Result<()> {
    get_estimate(conn, id)?;
    conn.execute("DELETE FROM estimates WHERE id = ?1", params![id])
        .context("Estimate の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_estimates_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM estimates WHERE itinerary_id = ?1",
        params![itinerary_id],
    )
    .context("Itinerary 配下 Estimate の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_estimates_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM estimates
         WHERE itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
    )
    .context("Trip 配下 Estimate の削除に失敗しました")?;
    Ok(())
}

fn row_to_estimate(row: &rusqlite::Row) -> rusqlite::Result<Estimate> {
    Ok(Estimate {
        id: row.get(0)?,
        itinerary_id: row.get(1)?,
        title: row.get(2)?,
        amount: row.get(3)?,
        currency: row.get(4)?,
        note: row.get(5)?,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn fmt_optional_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

pub(crate) fn print_estimate_list(
    target: EstimateListTarget,
    estimates: &[Estimate],
) -> Result<()> {
    let label = match target {
        EstimateListTarget::Trip(id) => format!("Trip {id}"),
        EstimateListTarget::Itinerary(id) => format!("Itinerary {id}"),
    };
    println!("{label} の Estimate ({} 件):", estimates.len());
    if estimates.is_empty() {
        println!("  （なし）");
        return Ok(());
    }
    println!(
        "{:<4} {:<6} {:<16} {:<12}",
        "ID", "Itin.", "Amount", "Title"
    );
    for estimate in estimates {
        println!(
            "{:<4} {:<6} {:<16} {:<12}",
            estimate.id,
            estimate.itinerary_id,
            format_amount_display(estimate.amount, &estimate.currency),
            fmt_optional_text(&estimate.title),
        );
    }
    Ok(())
}

pub(crate) fn print_estimate_detail(estimate: &Estimate) -> Result<()> {
    let json = estimate_to_json(estimate);
    println!("Estimate ID  : {}", json.id);
    println!("Itinerary ID : {}", json.itinerary_id);
    println!("Title        : {}", fmt_optional_text(&json.title));
    println!("Amount       : {}", json.amount_display);
    println!("Note         : {}", fmt_optional_text(&json.note));
    println!("Sort Order   : {}", json.sort_order);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::trip::add_test_trip;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB")
    }

    fn setup_itinerary(conn: &Connection) -> i64 {
        let trip_id = add_test_trip(conn, "Estimate Trip").unwrap();
        add_itinerary_item(
            conn,
            trip_id,
            1,
            "Breakfast",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_migrate_estimates_idempotent() {
        let conn = test_db();
        migrate_estimates(&conn).unwrap();
        migrate_estimates(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'estimates'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_add_list_show_update_delete_estimate() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);

        let id = add_estimate(
            &conn,
            itinerary_id,
            "14000",
            "JPY",
            Some("Hotel breakfast"),
            Some("5 people"),
            None,
        )
        .unwrap();

        let listed = list_estimates_for_itinerary(&conn, itinerary_id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title.as_deref(), Some("Hotel breakfast"));

        let estimate = get_estimate(&conn, id).unwrap();
        assert_eq!(estimate.amount, 14000);
        assert_eq!(estimate.currency, "JPY");

        update_estimate(
            &conn,
            id,
            &UpdateEstimateParams {
                amount_input: Some("15000"),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(get_estimate(&conn, id).unwrap().amount, 15000);

        update_estimate(
            &conn,
            id,
            &UpdateEstimateParams {
                clear_title: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(get_estimate(&conn, id).unwrap().title.is_none());

        delete_estimate(&conn, id).unwrap();
        assert!(get_estimate(&conn, id).is_err());
    }

    #[test]
    fn test_list_estimates_for_trip_ordering() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Multi Day").unwrap();
        let itin_day1 = add_itinerary_item(
            &conn, trip_id, 1, "Day1", None, None, None, None, None, None, None,
        )
        .unwrap();
        let itin_day2 = add_itinerary_item(
            &conn, trip_id, 2, "Day2", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_estimate(&conn, itin_day2, "1000", "JPY", None, None, None).unwrap();
        add_estimate(&conn, itin_day1, "2000", "JPY", None, None, None).unwrap();

        let trip_estimates = list_estimates_for_trip(&conn, trip_id).unwrap();
        assert_eq!(trip_estimates.len(), 2);
        assert_eq!(trip_estimates[0].itinerary_id, itin_day1);
        assert_eq!(trip_estimates[1].itinerary_id, itin_day2);
    }

    #[test]
    fn test_update_estimate_no_fields_rejects() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        let id = add_estimate(&conn, itinerary_id, "1000", "JPY", None, None, None).unwrap();
        assert!(update_estimate(&conn, id, &UpdateEstimateParams::default()).is_err());
    }

    #[test]
    fn test_update_estimate_title_clear_conflict() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        let id = add_estimate(&conn, itinerary_id, "1000", "JPY", Some("A"), None, None).unwrap();
        assert!(update_estimate(
            &conn,
            id,
            &UpdateEstimateParams {
                title: Some("B"),
                clear_title: true,
                ..Default::default()
            },
        )
        .is_err());
    }

    #[test]
    fn test_update_estimate_currency_without_amount_rejects() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        let id = add_estimate(&conn, itinerary_id, "1000", "JPY", None, None, None).unwrap();
        assert!(update_estimate(
            &conn,
            id,
            &UpdateEstimateParams {
                currency_input: Some("USD"),
                ..Default::default()
            },
        )
        .is_err());
    }

    #[test]
    fn test_delete_estimates_for_itinerary_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_estimate(&conn, itinerary_id, "1000", "JPY", None, None, None).unwrap();
        delete_estimates_for_itinerary(&conn, itinerary_id).unwrap();
        assert!(list_estimates_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_delete_estimates_for_trip_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_estimate(&conn, itinerary_id, "1000", "JPY", None, None, None).unwrap();
        delete_estimates_for_trip(&conn, 1).unwrap();
        assert!(list_estimates_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_add_estimate_usd_decimal() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        let id = add_estimate(&conn, itinerary_id, "12.50", "USD", None, None, None).unwrap();
        assert_eq!(get_estimate(&conn, id).unwrap().amount, 1250);
    }
}
