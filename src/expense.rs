use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::models::{
    Expense, ExpenseBeneficiary, ExportExpenseBeneficiaryV5, ExportExpenseV3, Participant,
};

pub(crate) use crate::money::{
    format_amount_display, parse_amount_for_currency, validate_currency_code,
};

const EXPENSE_SELECT_SQL: &str = "
    SELECT id, itinerary_id, title, amount, currency, paid_by_name, paid_by_participant_id,
           expense_date, note, sort_order, created_at, updated_at
    FROM expenses";

const BENEFICIARY_SELECT_SQL: &str = "
    SELECT id, expense_id, participant_id, sort_order, created_at, updated_at
    FROM expense_beneficiaries";

pub(crate) fn migrate_expenses_shared_expense(conn: &Connection) -> Result<()> {
    crate::storage::db::add_column_if_not_exists(
        conn,
        "expenses",
        "paid_by_participant_id",
        "INTEGER NULL",
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_expenses_paid_by_participant
         ON expenses(paid_by_participant_id)",
        [],
    )
    .context("idx_expenses_paid_by_participant の作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS expense_beneficiaries (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            expense_id      INTEGER NOT NULL,
            participant_id  INTEGER NOT NULL,
            sort_order      INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            UNIQUE (expense_id, participant_id)
        )",
        [],
    )
    .context("expense_beneficiaries テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_expense_beneficiaries_expense
         ON expense_beneficiaries(expense_id)",
        [],
    )
    .context("idx_expense_beneficiaries_expense の作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_expense_beneficiaries_participant
         ON expense_beneficiaries(participant_id)",
        [],
    )
    .context("idx_expense_beneficiaries_participant の作成に失敗しました")?;
    Ok(())
}

pub(crate) fn expense_is_shared(beneficiary_count: usize) -> bool {
    beneficiary_count > 0
}

fn trip_id_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<i64> {
    let trip_id: i64 = conn.query_row(
        "SELECT trip_id FROM itinerary_items WHERE id = ?1",
        params![itinerary_id],
        |row| row.get(0),
    )?;
    Ok(trip_id)
}

pub(crate) fn resolve_participant_for_expense_trip(
    conn: &Connection,
    itinerary_id: i64,
    id_or_name: &str,
) -> Result<i64> {
    let trip_id = trip_id_for_itinerary(conn, itinerary_id)?;
    resolve_participant_for_trip(conn, trip_id, id_or_name)
}

pub(crate) fn resolve_participant_for_trip(
    conn: &Connection,
    trip_id: i64,
    id_or_name: &str,
) -> Result<i64> {
    let trimmed = id_or_name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("participant not found: {id_or_name}");
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        let participant = crate::participant::get_participant(conn, id)?;
        if participant.trip_id != trip_id {
            anyhow::bail!("participant does not belong to this trip");
        }
        return Ok(id);
    }
    resolve_participant_by_name_in_trip(conn, trip_id, trimmed)
}

fn resolve_participant_by_name_in_trip(conn: &Connection, trip_id: i64, name: &str) -> Result<i64> {
    let participants = crate::participant::list_participants_by_trip(conn, trip_id)?;
    let matches: Vec<&Participant> = participants.iter().filter(|p| p.name == name).collect();
    match matches.len() {
        0 => anyhow::bail!("participant not found: {name}"),
        1 => Ok(matches[0].id),
        _ => anyhow::bail!("ambiguous participant name: {name}"),
    }
}

pub(crate) fn resolve_participant_ref_for_trip(
    conn: &Connection,
    trip_id: i64,
    participant_ref: &str,
) -> Result<i64> {
    let name = participant_ref.trim();
    if name.is_empty() {
        anyhow::bail!("unknown participant_ref: \"{participant_ref}\"");
    }
    resolve_participant_by_name_in_trip(conn, trip_id, name).map_err(|err| {
        if err.to_string().contains("ambiguous participant name") {
            err
        } else {
            anyhow::anyhow!("unknown participant_ref: \"{name}\"")
        }
    })
}

fn validate_beneficiary_ids(conn: &Connection, itinerary_id: i64, ids: &[i64]) -> Result<()> {
    let trip_id = trip_id_for_itinerary(conn, itinerary_id)?;
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(*id) {
            anyhow::bail!("duplicate beneficiary for expense");
        }
        let participant = crate::participant::get_participant(conn, *id)?;
        if participant.trip_id != trip_id {
            anyhow::bail!("participant does not belong to this trip");
        }
    }
    Ok(())
}

fn require_participants_for_structured_options(conn: &Connection, itinerary_id: i64) -> Result<()> {
    let trip_id = trip_id_for_itinerary(conn, itinerary_id)?;
    let participants = crate::participant::list_participants_by_trip(conn, trip_id)?;
    if participants.is_empty() {
        anyhow::bail!("no participants registered for this trip");
    }
    Ok(())
}

pub(crate) fn list_beneficiaries_for_expense(
    conn: &Connection,
    expense_id: i64,
) -> Result<Vec<ExpenseBeneficiary>> {
    let mut stmt = conn
        .prepare(&format!(
            "{BENEFICIARY_SELECT_SQL}
             WHERE expense_id = ?1
             ORDER BY sort_order ASC, id ASC"
        ))
        .context("beneficiary 一覧取得の準備に失敗しました")?;
    let rows = stmt
        .query_map(params![expense_id], row_to_beneficiary)
        .context("beneficiary 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("beneficiary 一覧の読み込みに失敗しました")?;
    Ok(rows)
}

fn row_to_beneficiary(row: &rusqlite::Row) -> rusqlite::Result<ExpenseBeneficiary> {
    Ok(ExpenseBeneficiary {
        id: row.get(0)?,
        expense_id: row.get(1)?,
        participant_id: row.get(2)?,
        sort_order: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub(crate) fn delete_beneficiaries_for_expense(conn: &Connection, expense_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM expense_beneficiaries WHERE expense_id = ?1",
        params![expense_id],
    )
    .context("Expense beneficiary の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_beneficiaries_for_participant(
    conn: &Connection,
    participant_id: i64,
) -> Result<()> {
    conn.execute(
        "DELETE FROM expense_beneficiaries WHERE participant_id = ?1",
        params![participant_id],
    )
    .context("Participant beneficiary 参照の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn clear_paid_by_for_participant(conn: &Connection, participant_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET paid_by_participant_id = NULL WHERE paid_by_participant_id = ?1",
        params![participant_id],
    )
    .context("Expense payer 参照の解除に失敗しました")?;
    Ok(())
}

pub(crate) fn set_expense_beneficiaries(
    conn: &Connection,
    expense_id: i64,
    participant_ids: &[i64],
) -> Result<()> {
    validate_beneficiary_ids(
        conn,
        get_expense(conn, expense_id)?.itinerary_id,
        participant_ids,
    )?;
    delete_beneficiaries_for_expense(conn, expense_id)?;
    let now = crate::storage::db::now_string();
    for (index, participant_id) in participant_ids.iter().enumerate() {
        conn.execute(
            "INSERT INTO expense_beneficiaries
             (expense_id, participant_id, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![expense_id, participant_id, index as i32, &now, &now],
        )
        .context("beneficiary の追加に失敗しました")?;
    }
    Ok(())
}

#[allow(dead_code)] // direct ID remap path; trip duplicate uses export/import ref resolution
pub(crate) fn duplicate_expense_beneficiaries(
    conn: &Connection,
    src_expense_id: i64,
    dst_expense_id: i64,
    participant_id_map: &HashMap<i64, i64>,
) -> Result<()> {
    let beneficiaries = list_beneficiaries_for_expense(conn, src_expense_id)?;
    if beneficiaries.is_empty() {
        return Ok(());
    }
    let mapped: Vec<i64> = beneficiaries
        .iter()
        .map(|b| {
            participant_id_map
                .get(&b.participant_id)
                .copied()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "participant id {} not found in remap table",
                        b.participant_id
                    )
                })
        })
        .collect::<Result<Vec<_>>>()?;
    set_expense_beneficiaries(conn, dst_expense_id, &mapped)
}

pub(crate) fn validate_expense_date(value: &str) -> Result<()> {
    if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
        anyhow::bail!("expense_date は YYYY-MM-DD 形式である必要があります");
    }
    Ok(())
}

pub(crate) fn validate_expense_date_opt(value: &Option<String>) -> Result<()> {
    if let Some(v) = value.as_deref() {
        validate_expense_date(v)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExpenseBeneficiaryJson {
    pub participant_id: i64,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExpenseJson {
    pub id: i64,
    pub itinerary_id: i64,
    pub title: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub paid_by_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_by_participant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_by_participant_name: Option<String>,
    pub shared: bool,
    pub beneficiaries: Vec<ExpenseBeneficiaryJson>,
    pub expense_date: Option<String>,
    pub note: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Service-layer enrichment for read-only expense output (not a CLI wire DTO).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExpenseBeneficiaryPart {
    pub participant_id: i64,
    pub name: String,
    pub sort_order: i32,
}

/// Expense plus resolved participant/beneficiary context for read-only list/show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExpenseEnrichedPart {
    pub expense: Expense,
    pub paid_by_participant_name: Option<String>,
    pub beneficiaries: Vec<ExpenseBeneficiaryPart>,
    pub shared: bool,
}

pub(crate) fn enrich_expense(conn: &Connection, expense: &Expense) -> Result<ExpenseEnrichedPart> {
    let beneficiaries = list_beneficiaries_for_expense(conn, expense.id)?;
    let beneficiary_parts = beneficiaries
        .iter()
        .map(|b| {
            let participant = crate::participant::get_participant(conn, b.participant_id)?;
            Ok(ExpenseBeneficiaryPart {
                participant_id: b.participant_id,
                name: participant.name,
                sort_order: b.sort_order,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let paid_by_participant_name = match expense.paid_by_participant_id {
        Some(id) => Some(crate::participant::get_participant(conn, id)?.name),
        None => None,
    };
    let shared = expense_is_shared(beneficiary_parts.len());
    Ok(ExpenseEnrichedPart {
        expense: expense.clone(),
        paid_by_participant_name,
        beneficiaries: beneficiary_parts,
        shared,
    })
}

pub(crate) fn enriched_expense_to_json(part: &ExpenseEnrichedPart) -> ExpenseJson {
    let expense = &part.expense;
    ExpenseJson {
        id: expense.id,
        itinerary_id: expense.itinerary_id,
        title: expense.title.clone(),
        amount: expense.amount,
        currency: expense.currency.clone(),
        paid_by_name: expense.paid_by_name.clone(),
        paid_by_participant_id: expense.paid_by_participant_id,
        paid_by_participant_name: part.paid_by_participant_name.clone(),
        shared: part.shared,
        beneficiaries: part
            .beneficiaries
            .iter()
            .map(|b| ExpenseBeneficiaryJson {
                participant_id: b.participant_id,
                name: b.name.clone(),
                sort_order: b.sort_order,
            })
            .collect(),
        expense_date: expense.expense_date.clone(),
        note: expense.note.clone(),
        sort_order: expense.sort_order,
        created_at: expense.created_at.clone(),
        updated_at: expense.updated_at.clone(),
    }
}

/// Markdown export 用の Expense 1行
#[allow(dead_code)]
pub(crate) fn format_expense_markdown_line(conn: &Connection, exp: &Expense) -> Result<String> {
    let amount = format_amount_display(exp.amount, &exp.currency);
    let title_part = match exp.title.as_deref() {
        Some(title) if !title.trim().is_empty() => format!("{title}: "),
        _ => String::new(),
    };
    let payer = match exp.paid_by_participant_id {
        Some(id) => crate::participant::get_participant(conn, id)?.name,
        None => exp.paid_by_name.clone().unwrap_or_default(),
    };
    let beneficiaries = list_beneficiaries_for_expense(conn, exp.id)?;
    let mut line = format!("- {title_part}{amount}");
    if !payer.is_empty() {
        line.push_str(&format!(" — Paid by: {payer}"));
    }
    if !beneficiaries.is_empty() {
        let names: Vec<String> = beneficiaries
            .iter()
            .map(|b| crate::participant::get_participant(conn, b.participant_id).map(|p| p.name))
            .collect::<Result<Vec<_>>>()?;
        line.push_str(&format!(" · Shared: {}", names.join(", ")));
    }
    Ok(line)
}

pub(crate) fn fmt_optional_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ExpenseListJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_id: Option<i64>,
    pub expenses: Vec<ExpenseJson>,
}

pub(crate) fn resolve_expense_list_target(
    trip: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ExpenseListTarget> {
    match (trip, itinerary) {
        (Some(trip_id), None) => Ok(ExpenseListTarget::Trip(trip_id)),
        (None, Some(itinerary_id)) => Ok(ExpenseListTarget::Itinerary(itinerary_id)),
        (Some(_), Some(_)) => {
            anyhow::bail!("--trip と --itinerary は同時に指定できません");
        }
        (None, None) => {
            anyhow::bail!("--trip または --itinerary のいずれかを指定してください");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpenseListTarget {
    Trip(i64),
    Itinerary(i64),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ExpenseSharedOptions {
    pub paid_by_participant_id: Option<i64>,
    pub beneficiary_participant_ids: Option<Vec<i64>>,
    pub clear_paid_by: bool,
    pub clear_beneficiaries: bool,
}

pub(crate) fn parse_expense_shared_options_for_add(
    conn: &Connection,
    itinerary_id: i64,
    paid_by_participant: Option<&str>,
    beneficiaries: &[String],
    shared_with: Option<&str>,
) -> Result<ExpenseSharedOptions> {
    if shared_with.is_some() && !beneficiaries.is_empty() {
        anyhow::bail!("cannot combine --shared-with and --beneficiary");
    }
    if paid_by_participant.is_some() || !beneficiaries.is_empty() || shared_with.is_some() {
        require_participants_for_structured_options(conn, itinerary_id)?;
    }
    let paid_by_participant_id = match paid_by_participant {
        Some(value) => Some(resolve_participant_for_expense_trip(
            conn,
            itinerary_id,
            value,
        )?),
        None => None,
    };
    let beneficiary_participant_ids = if let Some(mode) = shared_with {
        if mode.trim() != "all" {
            anyhow::bail!("--shared-with supports only \"all\"");
        }
        let trip_id = trip_id_for_itinerary(conn, itinerary_id)?;
        let participants = crate::participant::list_participants_by_trip(conn, trip_id)?;
        if participants.is_empty() {
            anyhow::bail!("no participants registered for this trip");
        }
        Some(participants.iter().map(|p| p.id).collect::<Vec<_>>())
    } else if beneficiaries.is_empty() {
        None
    } else {
        let mut ids = Vec::with_capacity(beneficiaries.len());
        for value in beneficiaries {
            ids.push(resolve_participant_for_expense_trip(
                conn,
                itinerary_id,
                value,
            )?);
        }
        Some(ids)
    };
    Ok(ExpenseSharedOptions {
        paid_by_participant_id,
        beneficiary_participant_ids,
        clear_paid_by: false,
        clear_beneficiaries: false,
    })
}

pub(crate) fn parse_expense_shared_options_for_update(
    conn: &Connection,
    itinerary_id: i64,
    paid_by_participant: Option<&str>,
    beneficiaries: &[String],
    shared_with: Option<&str>,
    clear_paid_by: bool,
    clear_beneficiaries: bool,
) -> Result<ExpenseSharedOptions> {
    if clear_beneficiaries && (!beneficiaries.is_empty() || shared_with.is_some()) {
        anyhow::bail!("cannot combine --clear-beneficiaries and --beneficiary/--shared-with");
    }
    if shared_with.is_some() && !beneficiaries.is_empty() {
        anyhow::bail!("cannot combine --shared-with and --beneficiary");
    }
    if paid_by_participant.is_some() || !beneficiaries.is_empty() || shared_with.is_some() {
        require_participants_for_structured_options(conn, itinerary_id)?;
    }
    let paid_by_participant_id = match paid_by_participant {
        Some(value) => Some(resolve_participant_for_expense_trip(
            conn,
            itinerary_id,
            value,
        )?),
        None => None,
    };
    let beneficiary_participant_ids = if clear_beneficiaries {
        None
    } else if let Some(mode) = shared_with {
        if mode.trim() != "all" {
            anyhow::bail!("--shared-with supports only \"all\"");
        }
        let trip_id = trip_id_for_itinerary(conn, itinerary_id)?;
        let participants = crate::participant::list_participants_by_trip(conn, trip_id)?;
        if participants.is_empty() {
            anyhow::bail!("no participants registered for this trip");
        }
        Some(participants.iter().map(|p| p.id).collect::<Vec<_>>())
    } else if beneficiaries.is_empty() {
        None
    } else {
        let mut ids = Vec::with_capacity(beneficiaries.len());
        for value in beneficiaries {
            ids.push(resolve_participant_for_expense_trip(
                conn,
                itinerary_id,
                value,
            )?);
        }
        Some(ids)
    };
    Ok(ExpenseSharedOptions {
        paid_by_participant_id,
        beneficiary_participant_ids,
        clear_paid_by,
        clear_beneficiaries,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_expense(
    conn: &Connection,
    itinerary_id: i64,
    amount_input: &str,
    currency_input: &str,
    title: Option<&str>,
    note: Option<&str>,
    paid_by_name: Option<&str>,
    expense_date: Option<&str>,
    shared: &ExpenseSharedOptions,
) -> Result<i64> {
    if shared.paid_by_participant_id.is_some() || shared.beneficiary_participant_ids.is_some() {
        require_participants_for_structured_options(conn, itinerary_id)?;
    }
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    let currency = validate_currency_code(currency_input)?;
    let amount = parse_amount_for_currency(amount_input, &currency)?;
    if let Some(date) = expense_date {
        validate_expense_date(date)?;
    }

    let paid_by_participant_id = shared.paid_by_participant_id;
    let mut synced_paid_by_name = paid_by_name.map(str::to_string);
    if let Some(payer_id) = paid_by_participant_id {
        let participant = crate::participant::get_participant(conn, payer_id)?;
        if participant.trip_id != trip_id_for_itinerary(conn, itinerary_id)? {
            anyhow::bail!("participant does not belong to this trip");
        }
        synced_paid_by_name = Some(participant.name);
    }

    let now = crate::storage::db::now_string();
    let tx = conn
        .unchecked_transaction()
        .context("expense add: トランザクション開始に失敗しました")?;
    tx.execute(
        "INSERT INTO expenses
         (itinerary_id, title, amount, currency, paid_by_name, paid_by_participant_id,
          expense_date, note, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10)",
        params![
            itinerary_id,
            title,
            amount,
            currency,
            synced_paid_by_name,
            paid_by_participant_id,
            expense_date,
            note,
            &now,
            &now,
        ],
    )
    .context("Expense の追加に失敗しました")?;
    let expense_id = tx.last_insert_rowid();
    if let Some(ids) = &shared.beneficiary_participant_ids {
        set_expense_beneficiaries(&tx, expense_id, ids)?;
    }
    tx.commit()
        .context("expense add: トランザクション確定に失敗しました")?;
    Ok(expense_id)
}

pub(crate) fn build_export_expense_v3(
    conn: &Connection,
    expense: &Expense,
) -> Result<ExportExpenseV3> {
    let beneficiaries = list_beneficiaries_for_expense(conn, expense.id)?;
    let paid_by_participant_ref = match expense.paid_by_participant_id {
        Some(id) => Some(crate::participant::get_participant(conn, id)?.name),
        None => None,
    };
    let export_beneficiaries = beneficiaries
        .iter()
        .map(|b| {
            let name = crate::participant::get_participant(conn, b.participant_id)?.name;
            Ok(ExportExpenseBeneficiaryV5 {
                participant_ref: name,
                sort_order: Some(b.sort_order),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ExportExpenseV3 {
        title: expense.title.clone(),
        amount: expense.amount,
        currency: expense.currency.clone(),
        paid_by_name: expense.paid_by_name.clone(),
        paid_by_participant_ref,
        beneficiaries: export_beneficiaries,
        expense_date: expense.expense_date.clone(),
        note: expense.note.clone(),
        sort_order: expense.sort_order,
    })
}

/// export schema v3/v5 の Expense を import する（amount は最小通貨単位整数）
pub(crate) fn import_expense_v3(
    conn: &Connection,
    itinerary_id: i64,
    export: &ExportExpenseV3,
) -> Result<i64> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    let trip_id = trip_id_for_itinerary(conn, itinerary_id)?;
    let currency = validate_currency_code(&export.currency)?;
    validate_expense_date_opt(&export.expense_date)?;

    let paid_by_participant_id = match export.paid_by_participant_ref.as_deref() {
        Some(ref_name) => Some(resolve_participant_ref_for_trip(conn, trip_id, ref_name)?),
        None => None,
    };
    let mut paid_by_name = export.paid_by_name.clone();
    if let Some(payer_id) = paid_by_participant_id {
        if paid_by_name.is_none() {
            paid_by_name = Some(crate::participant::get_participant(conn, payer_id)?.name);
        }
    }

    let beneficiary_ids: Vec<i64> = export
        .beneficiaries
        .iter()
        .map(|b| {
            resolve_participant_ref_for_trip(conn, trip_id, &b.participant_ref).with_context(|| {
                format!(
                    "unknown participant_ref: \"{}\" for expense {:?}",
                    b.participant_ref, export.title
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let now = crate::storage::db::now_string();
    let tx = conn
        .unchecked_transaction()
        .context("expense import: トランザクション開始に失敗しました")?;
    tx.execute(
        "INSERT INTO expenses
         (itinerary_id, title, amount, currency, paid_by_name, paid_by_participant_id,
          expense_date, note, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            itinerary_id,
            export.title,
            export.amount,
            currency,
            paid_by_name,
            paid_by_participant_id,
            export.expense_date,
            export.note,
            export.sort_order,
            &now,
            &now,
        ],
    )
    .context("Expense の import に失敗しました")?;
    let expense_id = tx.last_insert_rowid();
    if !beneficiary_ids.is_empty() {
        set_expense_beneficiaries(&tx, expense_id, &beneficiary_ids)?;
    }
    tx.commit()
        .context("expense import: トランザクション確定に失敗しました")?;
    Ok(expense_id)
}

pub(crate) fn collect_export_expense_validation_errors(
    export: &TripExportV3ForValidation<'_>,
    effective_schema: i32,
) -> (Vec<String>, Vec<String>) {
    use crate::domain::models::{TRIP_EXPORT_SCHEMA_VERSION_V4, TRIP_EXPORT_SCHEMA_VERSION_V5};

    if effective_schema < TRIP_EXPORT_SCHEMA_VERSION_V5 {
        return (Vec::new(), Vec::new());
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let participant_names: Vec<&str> = export
        .participants
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for name in &participant_names {
        *name_counts.entry(name).or_default() += 1;
    }
    let has_ambiguous_names = name_counts.values().any(|count| *count > 1);

    for (d_index, day) in export.days.iter().enumerate() {
        for (i_index, it) in day.itineraries.iter().enumerate() {
            for (e_index, exp) in it.expenses.iter().enumerate() {
                let prefix = format!("days[{d_index}].itineraries[{i_index}].expenses[{e_index}]");
                let uses_ref =
                    exp.paid_by_participant_ref.is_some() || !exp.beneficiaries.is_empty();
                if has_ambiguous_names && uses_ref {
                    errors.push(format!(
                        "{prefix}: ambiguous participant_ref due to duplicate participant names"
                    ));
                }
                if let Some(ref_name) = exp.paid_by_participant_ref.as_deref() {
                    if !participant_names.contains(&ref_name) {
                        errors.push(format!(
                            "{prefix}.paid_by_participant_ref: unknown ref \"{ref_name}\""
                        ));
                    } else if let Some(paid_by_name) = exp.paid_by_name.as_deref() {
                        if paid_by_name != ref_name {
                            warnings.push(format!(
                                "{prefix}: paid_by_name \"{paid_by_name}\" does not match ref \"{ref_name}\""
                            ));
                        }
                    }
                }
                for (b_index, beneficiary) in exp.beneficiaries.iter().enumerate() {
                    let b_prefix = format!("{prefix}.beneficiaries[{b_index}]");
                    let ref_name = beneficiary.participant_ref.as_str();
                    if !participant_names.contains(&ref_name) {
                        errors.push(format!(
                            "{b_prefix}.participant_ref: unknown ref \"{ref_name}\""
                        ));
                    }
                }
            }
        }
    }

    let _ = TRIP_EXPORT_SCHEMA_VERSION_V4;
    (errors, warnings)
}

pub(crate) struct TripExportV3ForValidation<'a> {
    pub participants: &'a [crate::domain::models::ExportParticipantV4],
    pub days: &'a [crate::domain::models::ExportDayV3],
}

pub(crate) fn list_expenses_for_itinerary(
    conn: &Connection,
    itinerary_id: i64,
) -> Result<Vec<Expense>> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    list_expenses_where(conn, "itinerary_id = ?1", params![itinerary_id])
}

pub(crate) fn list_expenses_for_trip(conn: &Connection, trip_id: i64) -> Result<Vec<Expense>> {
    crate::trip::get_trip(conn, trip_id)?;
    list_expenses_where(
        conn,
        "itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
    )
}

fn list_expenses_where<P: rusqlite::Params>(
    conn: &Connection,
    where_clause: &str,
    params: P,
) -> Result<Vec<Expense>> {
    let sql = format!(
        "{EXPENSE_SELECT_SQL}
         WHERE {where_clause}
         ORDER BY itinerary_id ASC, sort_order ASC, id ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .context("Expense 一覧取得の準備に失敗しました")?;
    let expenses = stmt
        .query_map(params, row_to_expense)
        .context("Expense 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Expense 一覧の読み込みに失敗しました")?;
    Ok(expenses)
}

pub(crate) fn get_expense(conn: &Connection, id: i64) -> Result<Expense> {
    crate::storage::db::map_query_row(
        conn.query_row(
            &format!("{EXPENSE_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_expense,
        ),
        || anyhow::anyhow!("Expense not found: {id}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_expense(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    amount_input: Option<&str>,
    currency_input: Option<&str>,
    paid_by_name: Option<&str>,
    expense_date: Option<&str>,
    note: Option<&str>,
    shared: &ExpenseSharedOptions,
) -> Result<()> {
    if shared.clear_beneficiaries && shared.beneficiary_participant_ids.is_some() {
        anyhow::bail!("cannot combine --clear-beneficiaries and --beneficiary/--shared-with");
    }

    let has_field_update = title.is_some()
        || amount_input.is_some()
        || currency_input.is_some()
        || paid_by_name.is_some()
        || expense_date.is_some()
        || note.is_some()
        || shared.paid_by_participant_id.is_some()
        || shared.clear_paid_by
        || shared.clear_beneficiaries
        || shared.beneficiary_participant_ids.is_some();

    if !has_field_update {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください (--title, --amount, --currency, --paid-by-name, --paid-by-participant, --beneficiary, --shared-with, --clear-paid-by, --clear-beneficiaries, --expense-date, --note)"
        );
    }

    if shared.paid_by_participant_id.is_some() || shared.beneficiary_participant_ids.is_some() {
        let expense = get_expense(conn, id)?;
        require_participants_for_structured_options(conn, expense.itinerary_id)?;
    }

    let mut expense = get_expense(conn, id)?;
    if let Some(value) = title {
        expense.title = Some(value.to_string());
    }
    if let Some(value) = note {
        expense.note = Some(value.to_string());
    }
    if shared.clear_paid_by {
        expense.paid_by_name = None;
        expense.paid_by_participant_id = None;
    } else if let Some(payer_id) = shared.paid_by_participant_id {
        let participant = crate::participant::get_participant(conn, payer_id)?;
        if participant.trip_id != trip_id_for_itinerary(conn, expense.itinerary_id)? {
            anyhow::bail!("participant does not belong to this trip");
        }
        expense.paid_by_participant_id = Some(payer_id);
        expense.paid_by_name = Some(participant.name);
    } else if let Some(value) = paid_by_name {
        expense.paid_by_name = Some(value.to_string());
    }
    if let Some(value) = expense_date {
        validate_expense_date(value)?;
        expense.expense_date = Some(value.to_string());
    }

    let currency = match currency_input {
        Some(code) => validate_currency_code(code)?,
        None => expense.currency.clone(),
    };
    if let Some(input) = amount_input {
        expense.amount = parse_amount_for_currency(input, &currency)?;
    }
    if currency_input.is_some() {
        expense.currency = currency;
    }

    crate::storage::db::with_transaction(conn, "expense update", |tx| {
        let now = crate::storage::db::now_string();
        tx.execute(
            "UPDATE expenses
             SET title = ?1, amount = ?2, currency = ?3, paid_by_name = ?4,
                 paid_by_participant_id = ?5, expense_date = ?6, note = ?7, updated_at = ?8
             WHERE id = ?9",
            params![
                expense.title,
                expense.amount,
                expense.currency,
                expense.paid_by_name,
                expense.paid_by_participant_id,
                expense.expense_date,
                expense.note,
                &now,
                id,
            ],
        )
        .context("Expense の更新に失敗しました")?;

        if shared.clear_beneficiaries {
            delete_beneficiaries_for_expense(tx, id)?;
        } else if let Some(ids) = &shared.beneficiary_participant_ids {
            set_expense_beneficiaries(tx, id, ids)?;
        }
        Ok(())
    })
}

pub(crate) fn delete_expense(conn: &Connection, id: i64) -> Result<()> {
    get_expense(conn, id)?;
    crate::storage::db::with_transaction(conn, "expense delete", |tx| {
        delete_beneficiaries_for_expense(tx, id)?;
        tx.execute("DELETE FROM expenses WHERE id = ?1", params![id])
            .context("Expense の削除に失敗しました")?;
        Ok(())
    })
}

pub(crate) fn delete_expenses_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<()> {
    let expense_ids: Vec<i64> = conn
        .prepare("SELECT id FROM expenses WHERE itinerary_id = ?1")?
        .query_map(params![itinerary_id], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for expense_id in &expense_ids {
        delete_beneficiaries_for_expense(conn, *expense_id)?;
    }
    conn.execute(
        "DELETE FROM expenses WHERE itinerary_id = ?1",
        params![itinerary_id],
    )
    .context("Itinerary 配下 Expense の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_expenses_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM expense_beneficiaries
         WHERE expense_id IN (
             SELECT id FROM expenses
             WHERE itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)
         )",
        params![trip_id],
    )
    .context("Trip 配下 beneficiary の削除に失敗しました")?;
    conn.execute(
        "DELETE FROM expenses
         WHERE itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
    )
    .context("Trip 配下 Expense の削除に失敗しました")?;
    Ok(())
}

fn row_to_expense(row: &rusqlite::Row) -> rusqlite::Result<Expense> {
    Ok(Expense {
        id: row.get(0)?,
        itinerary_id: row.get(1)?,
        title: row.get(2)?,
        amount: row.get(3)?,
        currency: row.get(4)?,
        paid_by_name: row.get(5)?,
        paid_by_participant_id: row.get(6)?,
        expense_date: row.get(7)?,
        note: row.get(8)?,
        sort_order: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

pub(crate) fn print_expense_list_from_enriched(
    target: ExpenseListTarget,
    expenses: &[ExpenseEnrichedPart],
) -> Result<()> {
    let label = match target {
        ExpenseListTarget::Trip(id) => format!("Trip {id}"),
        ExpenseListTarget::Itinerary(id) => format!("Itinerary {id}"),
    };
    println!("{label} の Expense ({} 件):", expenses.len());
    if expenses.is_empty() {
        println!("  （なし）");
        return Ok(());
    }
    println!(
        "{:<4} {:<6} {:<16} {:<12} {:<10}",
        "ID", "Itin.", "Amount", "Title", "Paid By"
    );
    for part in expenses {
        let expense = &part.expense;
        let payer = part
            .paid_by_participant_name
            .clone()
            .or_else(|| expense.paid_by_name.clone())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<4} {:<6} {:<16} {:<12} {:<10}",
            expense.id,
            expense.itinerary_id,
            format_amount_display(expense.amount, &expense.currency),
            fmt_optional_text(&expense.title),
            payer,
        );
    }
    Ok(())
}

pub(crate) fn print_expense_detail_from_enriched(part: &ExpenseEnrichedPart) -> Result<()> {
    let json = enriched_expense_to_json(part);
    println!("Expense ID  : {}", json.id);
    println!("Itinerary ID: {}", json.itinerary_id);
    println!("Title       : {}", fmt_optional_text(&json.title));
    println!(
        "Amount      : {}",
        format_amount_display(json.amount, &json.currency)
    );
    let payer = json
        .paid_by_participant_name
        .as_deref()
        .or(json.paid_by_name.as_deref())
        .unwrap_or("-");
    println!("Paid By     : {payer}");
    if json.shared {
        let names: Vec<String> = json.beneficiaries.iter().map(|b| b.name.clone()).collect();
        println!("Shared      : {}", names.join(", "));
    }
    println!("Date        : {}", fmt_optional_text(&json.expense_date));
    println!("Note        : {}", fmt_optional_text(&json.note));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::itinerary::add_itinerary_item;
    use crate::participant::create_participant;
    use crate::storage::db::reset_db;
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        crate::storage::db::open_db_at(":memory:").expect("インメモリ DB")
    }

    fn setup_itinerary(conn: &Connection) -> i64 {
        let trip_id = add_test_trip(conn, "Expense Trip").unwrap();
        add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap()
    }

    fn setup_itinerary_with_participants(conn: &Connection) -> (i64, i64, i64) {
        let trip_id = add_test_trip(conn, "Shared Trip").unwrap();
        let payer = create_participant(conn, trip_id, "Alice", None, true).unwrap();
        let beneficiary = create_participant(conn, trip_id, "Bob", None, false).unwrap();
        let itinerary_id = add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap();
        (itinerary_id, payer, beneficiary)
    }

    #[test]
    fn test_migrate_expenses_shared_expense_idempotent() {
        let conn = test_db();
        migrate_expenses_shared_expense(&conn).unwrap();
        migrate_expenses_shared_expense(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'expense_beneficiaries'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migrate_adds_paid_by_participant_id() {
        let conn = test_db();
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(expenses)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(columns.contains(&"paid_by_participant_id".to_string()));
    }

    #[test]
    fn test_add_list_show_update_delete_expense() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);

        let id = add_expense(
            &conn,
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

        let listed = list_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title.as_deref(), Some("Lunch"));

        let expense = get_expense(&conn, id).unwrap();
        assert_eq!(expense.amount, 2200);
        assert_eq!(expense.currency, "JPY");

        update_expense(
            &conn,
            id,
            None,
            Some("2500"),
            None,
            None,
            None,
            Some("Updated"),
            &ExpenseSharedOptions::default(),
        )
        .unwrap();
        let updated = get_expense(&conn, id).unwrap();
        assert_eq!(updated.amount, 2500);
        assert_eq!(updated.note.as_deref(), Some("Updated"));

        delete_expense(&conn, id).unwrap();
        assert!(get_expense(&conn, id).is_err());
    }

    #[test]
    fn test_create_expense_with_payer_and_beneficiaries() {
        let conn = test_db();
        let (itinerary_id, payer, beneficiary) = setup_itinerary_with_participants(&conn);
        let id = add_expense(
            &conn,
            itinerary_id,
            "4000",
            "JPY",
            Some("Dinner"),
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(payer),
                beneficiary_participant_ids: Some(vec![payer, beneficiary]),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();
        let expense = get_expense(&conn, id).unwrap();
        assert_eq!(expense.paid_by_participant_id, Some(payer));
        assert_eq!(expense.paid_by_name.as_deref(), Some("Alice"));
        let beneficiaries = list_beneficiaries_for_expense(&conn, id).unwrap();
        assert_eq!(beneficiaries.len(), 2);
    }

    #[test]
    fn test_enriched_expense_to_json_shared_fields() {
        let conn = test_db();
        let (itinerary_id, payer, beneficiary) = setup_itinerary_with_participants(&conn);
        let id = add_expense(
            &conn,
            itinerary_id,
            "4000",
            "JPY",
            Some("Dinner"),
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(payer),
                beneficiary_participant_ids: Some(vec![beneficiary]),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();
        let expense = get_expense(&conn, id).unwrap();
        let enriched = enrich_expense(&conn, &expense).unwrap();
        assert!(enriched.shared);
        assert_eq!(enriched.beneficiaries.len(), 1);
        let json = enriched_expense_to_json(&enriched);
        assert_eq!(json.id, id);
        assert_eq!(json.amount, 4000);
        assert_eq!(json.currency, "JPY");
        assert!(json.shared);
        assert_eq!(json.beneficiaries.len(), 1);
        assert_eq!(json.beneficiaries[0].participant_id, beneficiary);
    }

    #[test]
    fn test_duplicate_beneficiary_rejected() {
        let conn = test_db();
        let (itinerary_id, payer, _) = setup_itinerary_with_participants(&conn);
        let err = add_expense(
            &conn,
            itinerary_id,
            "1000",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions {
                beneficiary_participant_ids: Some(vec![payer, payer]),
                ..ExpenseSharedOptions::default()
            },
        )
        .expect_err("expected error");
        assert!(err.to_string().contains("duplicate beneficiary"));
    }

    #[test]
    fn test_structured_options_require_participants() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        let err = add_expense(
            &conn,
            itinerary_id,
            "1000",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(1),
                ..ExpenseSharedOptions::default()
            },
        )
        .expect_err("expected error");
        assert!(err
            .to_string()
            .contains("no participants registered for this trip"));
    }

    #[test]
    fn test_resolve_participant_by_name() {
        let conn = test_db();
        let (itinerary_id, payer, _) = setup_itinerary_with_participants(&conn);
        assert_eq!(
            resolve_participant_for_expense_trip(&conn, itinerary_id, "Alice").unwrap(),
            payer
        );
        assert!(resolve_participant_for_expense_trip(&conn, itinerary_id, "Nobody").is_err());
    }

    #[test]
    fn test_participant_delete_clears_payer_and_beneficiaries() {
        let conn = test_db();
        let (itinerary_id, payer, beneficiary) = setup_itinerary_with_participants(&conn);
        let expense_id = add_expense(
            &conn,
            itinerary_id,
            "3000",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(payer),
                beneficiary_participant_ids: Some(vec![payer, beneficiary]),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();
        crate::participant::delete_participant(&conn, payer).unwrap();
        let expense = get_expense(&conn, expense_id).unwrap();
        assert!(expense.paid_by_participant_id.is_none());
        assert_eq!(expense.paid_by_name.as_deref(), Some("Alice"));
        let beneficiaries = list_beneficiaries_for_expense(&conn, expense_id).unwrap();
        assert_eq!(beneficiaries.len(), 1);
        assert_eq!(beneficiaries[0].participant_id, beneficiary);
    }

    #[test]
    fn test_clear_paid_by_and_beneficiaries_on_update() {
        let conn = test_db();
        let (itinerary_id, payer, beneficiary) = setup_itinerary_with_participants(&conn);
        let expense_id = add_expense(
            &conn,
            itinerary_id,
            "3000",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(payer),
                beneficiary_participant_ids: Some(vec![beneficiary]),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();
        update_expense(
            &conn,
            expense_id,
            None,
            None,
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions {
                clear_paid_by: true,
                clear_beneficiaries: true,
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();
        let expense = get_expense(&conn, expense_id).unwrap();
        assert!(expense.paid_by_participant_id.is_none());
        assert!(expense.paid_by_name.is_none());
        assert!(list_beneficiaries_for_expense(&conn, expense_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_import_export_roundtrip_v5_fields() {
        let conn = test_db();
        let (itinerary_id, payer, beneficiary) = setup_itinerary_with_participants(&conn);
        let expense_id = add_expense(
            &conn,
            itinerary_id,
            "4000",
            "JPY",
            Some("Lunch"),
            None,
            None,
            None,
            &ExpenseSharedOptions {
                paid_by_participant_id: Some(payer),
                beneficiary_participant_ids: Some(vec![payer, beneficiary]),
                ..ExpenseSharedOptions::default()
            },
        )
        .unwrap();
        let expense = get_expense(&conn, expense_id).unwrap();
        let export = build_export_expense_v3(&conn, &expense).unwrap();
        assert_eq!(export.paid_by_participant_ref.as_deref(), Some("Alice"));
        assert_eq!(export.beneficiaries.len(), 2);

        let itinerary2 = add_itinerary_item(
            &conn,
            trip_id_for_itinerary(&conn, itinerary_id).unwrap(),
            2,
            "Dinner",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let imported = import_expense_v3(&conn, itinerary2, &export).unwrap();
        let imported_expense = get_expense(&conn, imported).unwrap();
        assert_eq!(imported_expense.paid_by_participant_id, Some(payer));
        assert_eq!(
            list_beneficiaries_for_expense(&conn, imported)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn test_list_expenses_for_trip() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "100",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        let trip_expenses = list_expenses_for_trip(&conn, 1).unwrap();
        assert_eq!(trip_expenses.len(), 1);
    }

    #[test]
    fn test_delete_expenses_for_itinerary_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "500",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        delete_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        assert!(list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_delete_expenses_for_trip_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "500",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        delete_expenses_for_trip(&conn, 1).unwrap();
        assert!(list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_itinerary_delete_cascades_expenses() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "500",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        crate::itinerary::delete_itinerary_item(&conn, itinerary_id).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_trip_delete_cascades_expenses() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "500",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        crate::trip::delete_trip(&conn, 1).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_expense_list_json_roundtrip() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "12.50",
            "USD",
            Some("Coffee"),
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        let expenses = list_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        let json_expenses: Vec<ExpenseJson> = expenses
            .iter()
            .map(|e| enriched_expense_to_json(&enrich_expense(&conn, e).unwrap()))
            .collect();
        let json = serde_json::to_string_pretty(&ExpenseListJson {
            trip_id: None,
            itinerary_id: Some(itinerary_id),
            expenses: json_expenses,
        })
        .unwrap();
        let parsed: ExpenseListJson = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.expenses[0].amount, 1250);
        assert_eq!(parsed.expenses[0].currency, "USD");
    }

    #[test]
    fn test_reset_db_clears_expenses() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "100",
            "JPY",
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        reset_db(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_legacy_unknown_currency_readable_and_update_without_currency_change() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        let now = crate::storage::db::now_string();
        conn.execute(
            "INSERT INTO expenses
             (itinerary_id, title, amount, currency, paid_by_name, paid_by_participant_id,
              expense_date, note, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, NULL, 0, ?5, ?5)",
            rusqlite::params![itinerary_id, "Legacy", 10_000_i64, "ZZZ", &now],
        )
        .unwrap();
        let id = conn.last_insert_rowid();

        let expense = get_expense(&conn, id).unwrap();
        assert_eq!(expense.currency, "ZZZ");

        let listed = list_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].currency, "ZZZ");

        update_expense(
            &conn,
            id,
            Some("Legacy title"),
            None,
            None,
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();
        let updated = get_expense(&conn, id).unwrap();
        assert_eq!(updated.currency, "ZZZ");
        assert_eq!(updated.title.as_deref(), Some("Legacy title"));
    }
}
